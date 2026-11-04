// Copyright (c) 2026 Umberto Giacobbi
// SPDX-License-Identifier: MIT

//! Send one playback command to an existing player on the user's session bus.

use anyhow::{Context, Result};
use std::time::Duration;
use zbus::blocking::{connection::Builder, fdo::DBusProxy, Connection};
use zbus::zvariant::OwnedValue;

const PATH: &str = "/org/mpris/MediaPlayer2";
const PLAYER: &str = "org.mpris.MediaPlayer2.Player";

fn playback_status(bus: &Connection, owner: &str) -> Result<String> {
    let reply = bus.call_method(
        Some(owner),
        PATH,
        Some("org.freedesktop.DBus.Properties"),
        "Get",
        &(PLAYER, "PlaybackStatus"),
    )?;
    Ok(String::try_from(reply.body().deserialize::<OwnedValue>()?)?)
}

pub(super) fn dispatch(method: &str) -> Result<()> {
    let bus = Builder::session()?
        .method_timeout(Duration::from_millis(750))
        .build()
        .context("connecting to the media session bus")?;
    let dbus = DBusProxy::new(&bus)?;
    let mut names = dbus.list_names()?;
    names.retain(|name| name.as_str().starts_with("org.mpris.MediaPlayer2."));
    names.sort_by(|a, b| a.as_str().cmp(b.as_str()));

    let mut chosen = None;
    let mut best = 3;
    for name in names {
        // A player may exit between discovery and dispatch. Its unique owner
        // prevents a key press from launching it again or controlling its replacement.
        let Ok(owner) = dbus.get_name_owner(name.into_inner()) else {
            continue;
        };
        let rank = match playback_status(&bus, owner.as_str()).as_deref() {
            Ok("Playing") => 0,
            Ok("Paused") => 1,
            Ok("Stopped") => 2,
            _ => continue,
        };
        if rank < best {
            best = rank;
            chosen = Some(owner);
            if rank == 0 {
                break;
            }
        }
    }
    let owner = chosen.context("no responding MPRIS player; open a media player first")?;
    bus.call_method(Some(owner.as_str()), PATH, Some(PLAYER), method, &())
        .with_context(|| format!("sending {method} to the media player"))?;
    Ok(())
}
