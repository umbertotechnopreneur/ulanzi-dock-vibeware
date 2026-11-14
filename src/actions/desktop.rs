// Copyright (c) 2026 Umberto Giacobbi. SPDX-License-Identifier: MIT.

//! Native GNOME desktop commands. Arbitrary application shortcuts keep their input backend.
use anyhow::{Context, Result};
use std::{collections::HashMap, process::{Command, Stdio}, sync::OnceLock, time::{Duration, Instant}};
use zbus::{blocking::{Connection, connection::Builder}, zvariant::Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DesktopAction { Files, Settings, Screenshot, TaskView, ShowDesktop, Emoji, Clipboard, Search }
impl DesktopAction {
    pub(crate) fn parse(token: &str) -> Result<Self> {
        Ok(match token {
            "files" => Self::Files, "settings" => Self::Settings, "screenshot" => Self::Screenshot,
            "task-view" => Self::TaskView, "show-desktop" => Self::ShowDesktop,
            "emoji" => Self::Emoji, "clipboard" => Self::Clipboard, "search" => Self::Search,
            _ => anyhow::bail!("unknown desktop action: {token}"),
        })
    }
}

// Retain the caller connection for asynchronous portal screenshot requests.
static BUS: OnceLock<Connection> = OnceLock::new();
fn bus() -> Result<&'static Connection> {
    if BUS.get().is_none() {
        let connection = Builder::session()?.method_timeout(Duration::from_secs(3)).build()?;
        let _ = BUS.set(connection);
    }
    Ok(BUS.get().expect("initialized desktop bus"))
}

fn spawn(program: &str, args: &[&std::ffi::OsStr]) -> Result<()> {
    let mut command = Command::new(program);
    command.args(args).stdin(Stdio::null());
    crate::linux_session::prepare(&mut command)?;
    let mut child = command.spawn()
        .with_context(|| format!("starting {program}"))?;
    std::thread::spawn(move || { let _ = child.wait(); });
    Ok(())
}

pub(super) fn execute(action: DesktopAction) -> Result<()> {
    use DesktopAction::*;
    match action {
        Files => {
            let home = std::env::var_os("HOME").context("HOME is not set")?;
            spawn("gio", &["open".as_ref(), home.as_ref()])
        }
        Settings => spawn("gnome-control-center", &[]),
        Emoji => spawn("gnome-characters", &[]),
        Screenshot => {
            let options = HashMap::from([("interactive", Value::from(true))]);
            bus()?.call_method(Some("org.freedesktop.portal.Desktop"), "/org/freedesktop/portal/desktop",
                Some("org.freedesktop.portal.Screenshot"), "Screenshot", &("", options))?;
            Ok(())
        }
        TaskView | Search => {
            bus()?.call_method(Some("org.gnome.Shell"), "/org/gnome/Shell",
                Some("org.freedesktop.DBus.Properties"), "Set",
                &("org.gnome.Shell", "OverviewActive", Value::from(true)))?;
            Ok(())
        }
        ShowDesktop => show_desktop(),
        Clipboard => super::execute_immediate(&format!("hotkey:{}", clipboard_chord()?)),
    }
}

fn show_desktop() -> Result<()> {
    use x11rb::{connection::Connection as _, protocol::xproto::{ConnectionExt, AtomEnum, ClientMessageEvent, EventMask}};
    let (connection, screen) = x11rb::connect(None).context("connecting to the desktop window manager")?;
    let root = connection.setup().roots[screen].root;
    let atom = connection.intern_atom(false, b"_NET_SHOWING_DESKTOP")?.reply()?.atom;
    let supported = connection.intern_atom(false, b"_NET_SUPPORTED")?.reply()?.atom;
    let capabilities = connection.get_property(false, root, supported, AtomEnum::ATOM, 0, 4096)?.reply()?;
    anyhow::ensure!(capabilities.value32().is_some_and(|mut values| values.any(|value| value == atom)),
        "this window manager does not expose Show Desktop");
    let current = connection.get_property(false, root, atom, AtomEnum::CARDINAL, 0, 1)?.reply()?;
    let showing = current.value32().and_then(|mut values| values.next()).unwrap_or(0);
    let event = ClientMessageEvent::new(32, root, atom, [u32::from(showing == 0), 0, 0, 0, 0]);
    connection.send_event(false, root, EventMask::SUBSTRUCTURE_REDIRECT | EventMask::SUBSTRUCTURE_NOTIFY, event)?.check()?;
    connection.flush()?;
    Ok(())
}

pub(super) fn clipboard_chord() -> Result<String> {
    // Use the shortcut owned by the user's existing Clipboard Indicator extension.
    let info = bus()?.call_method(Some("org.gnome.Shell"), "/org/gnome/Shell",
        Some("org.gnome.Shell.Extensions"), "GetExtensionInfo", &("clipboard-indicator@tudmotu.com",))?;
    let info: HashMap<String, zbus::zvariant::OwnedValue> = info.body().deserialize()?;
    anyhow::ensure!(info.get("state").and_then(|state| f64::try_from(state).ok()) == Some(1.0),
        "enable Clipboard Indicator in GNOME Extensions to use clipboard history");
    let home = std::env::var_os("HOME").context("HOME is not set")?;
    let schemas = std::path::PathBuf::from(home).join(".local/share/gnome-shell/extensions/clipboard-indicator@tudmotu.com/schemas");
    let mut child = Command::new("gsettings").arg("--schemadir").arg(schemas)
        .args(["get", "org.gnome.shell.extensions.clipboard-indicator", "toggle-menu"])
        .stdin(Stdio::null()).stdout(Stdio::piped()).spawn()?;
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            result => {
                let _ = child.kill();
                let _ = child.wait();
                result.context("waiting for the Clipboard Indicator shortcut")?;
                anyhow::bail!("Reading the Clipboard Indicator shortcut timed out");
            }
        }
    }
    let output = child.wait_with_output()?;
    anyhow::ensure!(output.status.success(), "could not read the Clipboard Indicator shortcut");
    let binding = String::from_utf8(output.stdout)?;
    let binding = binding.split('\'').nth(1).context("assign a Clipboard Indicator shortcut first")?;
    Ok(binding.replace("<Primary>", "ctrl+").replace("<Control>", "ctrl+")
        .replace("<Shift>", "shift+").replace("<Alt>", "alt+").replace("<Super>", "super+"))
}
