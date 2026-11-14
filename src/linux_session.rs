// Copyright (c) 2026 Umberto Giacobbi. SPDX-License-Identifier: MIT.

//! Refresh graphical child processes from the current desktop session.

use anyhow::{Context, Result};
use std::{process::Command, time::Duration};
use zbus::blocking::{connection::Builder, Proxy};

const VARIABLES: &[&str] = &[
    "DISPLAY", "WAYLAND_DISPLAY", "XAUTHORITY", "XDG_RUNTIME_DIR",
    "DBUS_SESSION_BUS_ADDRESS", "XDG_SESSION_TYPE", "XDG_CURRENT_DESKTOP",
];

pub(crate) fn prepare(command: &mut Command) -> Result<()> {
    // A sign-in service can start before GNOME imports its display environment.
    let environment = session_environment();
    match environment {
        Ok(values) => {
            for value in values {
                if let Some((name, value)) = value.split_once('=') {
                    if VARIABLES.contains(&name) {
                        command.env(name, value);
                    }
                }
            }
        }
        Err(error) if std::env::var_os("DISPLAY").is_some() || std::env::var_os("WAYLAND_DISPLAY").is_some() => {
            eprintln!("Could not refresh the desktop environment: {error:#}");
        }
        Err(error) => return Err(error).context("locating the active graphical session"),
    }
    // This names a caller-owned file descriptor, not a reusable display address.
    command.env_remove("WAYLAND_SOCKET");
    Ok(())
}

fn session_environment() -> Result<Vec<String>> {
    let bus = Builder::session()?.method_timeout(Duration::from_secs(1)).build()?;
    let manager = Proxy::new(&bus, "org.freedesktop.systemd1", "/org/freedesktop/systemd1",
        "org.freedesktop.systemd1.Manager")?;
    Ok(manager.get_property("Environment")?)
}
