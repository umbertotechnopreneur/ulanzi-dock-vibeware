// Copyright (c) 2026 Umberto Giacobbi
// SPDX-License-Identifier: MIT

//! Linux tray commands for the running GUI and the sign-in preference.

use super::MainWindow;
use crate::{startup, tray::linux::connection};
use anyhow::Result;
use slint::ComponentHandle;
use std::{path::Path, time::Duration};
use zbus::{blocking, fdo, Connection};

const INTERFACE: &str = "io.github.umbertotechnopreneur.UlanziDeck.Gui";
const PATH: &str = "/io/github/umbertotechnopreneur/UlanziDeck/Gui";

fn name(profile: &Path) -> Result<String> {
    Ok(format!("{}.Gui", connection::name(profile)?))
}

pub(super) fn publish(profile: &Path) -> Result<blocking::Connection> {
    Ok(blocking::connection::Builder::session()?
        .method_timeout(Duration::from_secs(1))
        .name(name(profile)?)?
        .serve_at(PATH, Endpoint)?
        .build()?)
}

pub(crate) async fn request(bus: &Connection, profile: &Path, method: &str) -> Result<bool> {
    match bus.call_method(Some(name(profile)?.as_str()), PATH, Some(INTERFACE), method, &()).await {
        Ok(_) => Ok(true),
        Err(zbus::Error::MethodError(error, _, _))
            if matches!(error.as_str(),
                "org.freedesktop.DBus.Error.ServiceUnknown" | "org.freedesktop.DBus.Error.NameHasNoOwner")
            => Ok(false),
        Err(error) => Err(error.into()),
    }
}

struct Endpoint;

#[zbus::interface(name = "io.github.umbertotechnopreneur.UlanziDeck.Gui")]
impl Endpoint {
    fn show_about(&self) -> fdo::Result<()> {
        super::show_about().map_err(|error| fdo::Error::Failed(error.to_string()))
    }

    fn show_settings(&self) -> fdo::Result<()> {
        super::show_settings().map_err(|error| fdo::Error::Failed(error.to_string()))
    }

    fn quit(&self) -> fdo::Result<()> {
        slint::invoke_from_event_loop(|| {
            let _ = slint::quit_event_loop();
        }).map_err(|error| fdo::Error::Failed(error.to_string()))
    }
}

pub(super) fn bind_startup(window: &MainWindow, profile: &Path) {
    if !connection::can_start(profile) {
        return;
    }
    window.set_startup_visible(true);
    match startup::linux_enabled() {
        Ok(enabled) => {
            window.set_startup_enabled(enabled);
            window.set_startup_ready(true);
        }
        Err(error) => window.set_startup_message(format!("{error:#}").into()),
    }
    let weak = window.as_weak();
    window.on_startup_toggled(move |enabled| {
        let Some(window) = weak.upgrade() else { return; };
        match startup::linux_set_enabled(enabled) {
            Ok(()) => window.set_startup_message("".into()),
            Err(error) => {
                match startup::linux_enabled() {
                    Ok(actual) => window.set_startup_enabled(actual),
                    Err(_) => {
                        window.set_startup_enabled(!enabled);
                        window.set_startup_ready(false);
                    }
                }
                window.set_startup_message(format!("{error:#}").into());
            }
        }
    });
}
