// Copyright (c) 2026 Umberto Giacobbi
// SPDX-License-Identifier: MIT

//! Share the live controller state with the settings window on the user bus.

use super::{ConnectionState, Control};
use crate::startup::LINUX_UNIT;
use anyhow::Result;
use std::{
    env, fs,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    sync::Arc,
};
use zbus::{blocking::{connection::Builder, Connection}, zvariant::OwnedObjectPath};

const INTERFACE: &str = "io.github.umbertotechnopreneur.UlanziDeck.Controller";
const PATH: &str = "/io/github/umbertotechnopreneur/UlanziDeck/Controller";

pub(crate) fn name(profile: &Path) -> Result<String> {
    let path = fs::canonicalize(profile).or_else(|_| std::path::absolute(profile))?;
    let mut hash = DefaultHasher::new();
    path.hash(&mut hash);
    Ok(format!("{INTERFACE}.p{:016x}", hash.finish()))
}

pub(super) fn publish(profile: &Path, control: Arc<Control>) -> Result<Connection> {
    let name = name(profile)?;
    let bus = Builder::session()?.serve_at(PATH, Endpoint(control))?.build()?;
    bus.request_name(name.as_str())?;
    Ok(bus)
}

struct Endpoint(Arc<Control>);

#[zbus::interface(name = "io.github.umbertotechnopreneur.UlanziDeck.Controller")]
impl Endpoint {
    fn get_state(&self) -> u8 {
        self.0.connection() as u8
    }

    fn reconnect(&self) {
        self.0.request_reconnect();
    }
}

/// A missing bus owner means the controller for this profile is absent.
pub(crate) fn state(bus: &Connection, name: &str) -> Result<Option<ConnectionState>> {
    let reply = match bus.call_method(Some(name), PATH, Some(INTERFACE), "GetState", &()) {
        Ok(reply) => reply,
        Err(zbus::Error::MethodError(error, _, _))
            if matches!(
                error.as_str(),
                "org.freedesktop.DBus.Error.ServiceUnknown" | "org.freedesktop.DBus.Error.NameHasNoOwner"
            ) => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    let state = match reply.body().deserialize::<u8>()? {
        0 => ConnectionState::Waiting,
        1 => ConnectionState::Connected,
        2 => ConnectionState::Disconnected,
        value => anyhow::bail!("Unknown controller state {value}"),
    };
    Ok(Some(state))
}

pub(crate) fn can_start(profile: &Path) -> bool {
    let directory = env::var_os("XDG_CONFIG_HOME")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|path| PathBuf::from(path).join(".config")));
    Path::new("/usr/lib/systemd/user").join(LINUX_UNIT).is_file()
        && directory.is_some_and(|directory| {
            let default = directory.join("ulanzideck/settings.json");
            name(&default).is_ok_and(|default| name(profile).is_ok_and(|profile| default == profile))
        })
}

pub(crate) fn reconnect(bus: &Connection, name: &str, managed: bool) -> Result<()> {
    if state(bus, name)?.is_some() {
        bus.call_method(Some(name), PATH, Some(INTERFACE), "Reconnect", &())?;
    } else if managed {
        let reply = bus.call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "StartUnit",
            &(LINUX_UNIT, "replace"),
        )?;
        let _: OwnedObjectPath = reply.body().deserialize()?;
    } else {
        anyhow::bail!("Start the controller for this profile first.");
    }
    Ok(())
}
