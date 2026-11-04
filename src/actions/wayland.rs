// Copyright (c) 2026 Umberto Giacobbi. SPDX-License-Identifier: MIT.

//! One explicitly authorized keyboard session for the controller's Wayland shortcuts.
use anyhow::{Context, Result};
use enigo::{Direction, Key};
use futures_lite::{future, StreamExt};
use std::{collections::HashMap, sync::atomic::{AtomicU32, Ordering}, time::Duration};
use zbus::{blocking::Connection, zvariant::{OwnedObjectPath, OwnedValue, Value}};

const SERVICE: &str = "org.freedesktop.portal.Desktop";
const PATH: &str = "/org/freedesktop/portal/desktop";
const INTERFACE: &str = "org.freedesktop.portal.RemoteDesktop";
type Results = HashMap<String, OwnedValue>;
static SERIAL: AtomicU32 = AtomicU32::new(0);

fn token() -> String {
    format!("ulanzi{}_{}", std::process::id(), SERIAL.fetch_add(1, Ordering::Relaxed))
}

async fn request<B: serde::Serialize + zbus::zvariant::DynamicType>(
    bus: &zbus::Connection, method: &str, token: &str, body: &B,
) -> Result<Results> {
    let sender = bus.unique_name().context("portal connection has no unique name")?
        .as_str().trim_start_matches(':').replace('.', "_");
    let path = format!("/org/freedesktop/portal/desktop/request/{sender}/{token}");
    let proxy = zbus::Proxy::new(bus, SERVICE, path.as_str(), "org.freedesktop.portal.Request").await?;
    let mut replies = proxy.receive_signal("Response").await?;
    let reply = bus.call_method(Some(SERVICE), PATH, Some(INTERFACE), method, body).await?;
    let returned: OwnedObjectPath = reply.body().deserialize()
        .with_context(|| format!("decoding the {method} request handle"))?;
    anyhow::ensure!(returned.as_str() == path, "portal returned an unexpected request handle");
    let response = future::or(
        async { replies.next().await.context("portal response stream closed") },
        async { async_io::Timer::after(Duration::from_secs(90)).await; anyhow::bail!("keyboard authorization timed out") },
    ).await;
    let message = match response {
        Ok(message) => message,
        Err(error) => { let _ = proxy.call_method("Close", &()).await; return Err(error); }
    };
    let (code, results): (u32, Results) = message.body().deserialize()
        .with_context(|| format!("decoding the {method} response"))?;
    anyhow::ensure!(code == 0, "keyboard authorization cancelled or denied (response {code})");
    Ok(results)
}

pub(super) struct Keyboard { bus: Connection, session: OwnedObjectPath }
impl Keyboard {
    pub(super) fn new() -> Result<Self> {
        future::block_on(async {
            let bus = zbus::connection::Builder::session()?
                .method_timeout(Duration::from_secs(3)).build().await?;
            let handle = token();
            let session_token = token();
            let options = HashMap::from([
                ("handle_token", Value::from(handle.as_str())),
                ("session_handle_token", Value::from(session_token.as_str())),
            ]);
            let mut results = request(&bus, "CreateSession", &handle, &(options,)).await?;
            // The portal returns session_handle as a D-Bus string in its result
            // dictionary, even though following calls require an object path.
            let handle_value = results.remove("session_handle")
                .context("portal returned no keyboard session")?;
            let session_string = String::try_from(handle_value)
                .context("decoding the portal session handle string")?;
            let session = OwnedObjectPath::try_from(session_string)
                .context("validating the portal session object path")?;
            let handle = token();
            let options = HashMap::from([
                ("handle_token", Value::from(handle.as_str())),
                ("types", Value::from(1u32)), // Keyboard only.
            ]);
            request(&bus, "SelectDevices", &handle, &(&session, options)).await?;
            let handle = token();
            let options = HashMap::from([("handle_token", Value::from(handle.as_str()))]);
            println!("Allow keyboard interaction in the GNOME dialog to use dock shortcuts.");
            let results = request(&bus, "Start", &handle, &(&session, "", options)).await?;
            let devices = results.get("devices").and_then(|value| u32::try_from(value).ok()).unwrap_or(0);
            anyhow::ensure!(devices & 1 != 0, "GNOME did not grant keyboard access");
            println!("Wayland keyboard session authorized.");
            Ok(Self { bus: Connection::from(bus), session })
        })
    }

    pub(super) fn key(&self, key: Key, direction: Direction) -> Result<()> {
        let symbol = keysym(key)?;
        let states: &[u32] = match direction { Direction::Press => &[1], Direction::Release => &[0], Direction::Click => &[1, 0] };
        for state in states {
            self.bus.call_method(Some(SERVICE), PATH, Some(INTERFACE), "NotifyKeyboardKeysym",
                &(&self.session, HashMap::<&str, Value<'_>>::new(), symbol as i32, *state))
                .context("sending the shortcut through the authorized Wayland session")?;
        }
        Ok(())
    }
}
impl Drop for Keyboard {
    fn drop(&mut self) {
        let _ = self.bus.call_method(Some(SERVICE), self.session.as_str(),
            Some("org.freedesktop.portal.Session"), "Close", &());
    }
}

fn keysym(key: Key) -> Result<u32> {
    Ok(match key {
        Key::Control => 0xffe3, Key::Shift => 0xffe1, Key::Alt => 0xffe9, Key::Meta => 0xffeb,
        Key::Tab => 0xff09, Key::Return => 0xff0d, Key::Escape => 0xff1b, Key::Space => 0x20,
        Key::F2 => 0xffbf, Key::F5 => 0xffc2, Key::F7 => 0xffc4, Key::F9 => 0xffc6, Key::F12 => 0xffc9,
        Key::Unicode(c) if u32::from(c) <= 0xff => u32::from(c),
        Key::Unicode(c) => 0x01000000 | u32::from(c),
        _ => anyhow::bail!("unsupported Wayland shortcut key: {key:?}"),
    })
}
