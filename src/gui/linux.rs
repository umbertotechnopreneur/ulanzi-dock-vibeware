// Copyright (c) 2026 Umberto Giacobbi
// SPDX-License-Identifier: MIT

//! Linux tray commands for the running GUI and the sign-in preference.

use super::MainWindow;
use crate::{startup, tray::linux::connection};
use anyhow::Result;
use slint::ComponentHandle;
use std::{
    path::Path,
    sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc},
    thread,
    time::Duration,
};
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
    let (request, requests) = mpsc::sync_channel::<bool>(1);
    let pending = Arc::new(AtomicBool::new(true));
    let busy = Arc::clone(&pending);
    let weak = window.as_weak();
    let worker = thread::Builder::new().name("sign-in-startup".into()).spawn(move || {
        let mut state = StartupState::read();
        busy.store(false, Ordering::Release);
        if !state.publish(&weak, false) {
            return;
        }
        while let Ok(enabled) = requests.recv() {
            state.message = "Updating sign-in startup…".into();
            if !state.publish(&weak, true) {
                return;
            }
            state = match startup::linux_set_enabled(enabled) {
                Ok(()) => StartupState { enabled: Some(enabled), message: String::new() },
                Err(error) => {
                    let mut actual = StartupState::read();
                    actual.message = format!("Could not update sign-in startup: {error:#}");
                    actual
                }
            };
            busy.store(false, Ordering::Release);
            if !state.publish(&weak, false) {
                return;
            }
        }
    });
    if let Err(error) = worker {
        window.set_startup_message(format!("Could not read sign-in startup: {error}").into());
        return;
    }
    window.on_startup_toggled(move |enabled| {
        if pending.swap(true, Ordering::AcqRel) {
            return;
        }
        if request.try_send(enabled).is_err() {
            pending.store(false, Ordering::Release);
        }
    });
}

struct StartupState {
    enabled: Option<bool>,
    message: String,
}

impl StartupState {
    fn read() -> Self {
        match startup::linux_enabled() {
            Ok(enabled) => Self { enabled: Some(enabled), message: String::new() },
            Err(error) => Self { enabled: None, message: format!("{error:#}") },
        }
    }

    fn publish(&self, window: &slint::Weak<MainWindow>, updating: bool) -> bool {
        let enabled = self.enabled;
        let message = self.message.clone();
        window.upgrade_in_event_loop(move |window| {
            window.set_startup_enabled(enabled.unwrap_or(false));
            window.set_startup_ready(enabled.is_some() && !updating);
            window.set_startup_message(message.into());
        }).is_ok()
    }
}
