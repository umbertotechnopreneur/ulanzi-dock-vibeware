// Copyright (c) 2026 Umberto Giacobbi
// SPDX-License-Identifier: MIT

//! Present controller state and forward a retry request without opening HID.

use super::MainWindow;
use crate::tray::Control;
use std::{path::Path, sync::Arc};

#[cfg(any(target_os = "linux", target_os = "windows"))]
use crate::tray::ConnectionState;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use slint::ComponentHandle;

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn show_state(window: &MainWindow, state: Option<ConnectionState>, can_retry: bool) {
    let label = match state {
        Some(ConnectionState::Connected) => "Connected",
        Some(ConnectionState::Waiting) => "Connecting",
        Some(ConnectionState::Disconnected) => "Disconnected",
        None => "Unavailable",
    };
    window.set_dock_status(label.into());
    window.set_can_reconnect(can_retry && matches!(state, None | Some(ConnectionState::Disconnected)));
}

#[cfg(target_os = "linux")]
pub(super) fn bind(
    window: &MainWindow,
    profile: &Path,
    _control: Arc<Control>,
    _resident: bool,
) -> Option<slint::Timer> {
    use crate::tray::linux::connection as link;
    use std::{sync::mpsc, thread, time::Duration};
    use zbus::blocking::connection::Builder;

    let name = match link::name(profile) {
        Ok(name) => name,
        Err(error) => {
            eprintln!("Could not identify the controller profile: {error:#}");
            return None;
        }
    };
    let managed = link::can_start(profile);
    let (request, requests) = mpsc::sync_channel(1);
    let weak = window.as_weak();
    let worker = thread::Builder::new().name("dock-status".into()).spawn(move || {
        let mut bus = None;
        let mut previous = None;
        let mut retry = false;
        loop {
            if bus.is_none() {
                bus = Builder::session()
                    .and_then(|builder| builder.method_timeout(Duration::from_millis(750)).build())
                    .ok();
            }
            let error = if retry {
                match &bus {
                    Some(bus) => link::reconnect(bus, &name, managed)
                        .err()
                        .map(|error| format!("Could not reconnect: {error:#}")),
                    None => Some("Could not connect to the desktop session bus.".into()),
                }
            } else {
                None
            };
            let (state, can_retry) = match bus.as_ref().map(|bus| link::state(bus, &name)) {
                Some(Ok(Some(state))) => (Some(state), true),
                Some(Ok(None)) => (None, managed),
                _ => {
                    bus = None;
                    (None, false)
                }
            };
            let view = (state, can_retry);
            if retry || previous != Some(view) {
                if weak.upgrade_in_event_loop(move |window| {
                    show_state(&window, state, can_retry);
                    if let Some(error) = error {
                        window.set_message(error.into());
                    }
                }).is_err() {
                    break;
                }
                previous = Some(view);
            }
            // The callback owns the sender. Closing the dialog releases the worker.
            retry = match requests.recv_timeout(Duration::from_millis(750)) {
                Ok(()) => true,
                Err(mpsc::RecvTimeoutError::Timeout) => false,
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
            };
        }
    });
    if let Err(error) = worker {
        eprintln!("Could not start the dock status reader: {error}");
        return None;
    }
    let weak = window.as_weak();
    window.on_reconnect(move || {
        if request.try_send(()).is_ok() {
            if let Some(window) = weak.upgrade() {
                show_state(&window, Some(ConnectionState::Waiting), false);
            }
        }
    });
    None
}

#[cfg(target_os = "windows")]
pub(super) fn bind(
    window: &MainWindow,
    _profile: &Path,
    control: Arc<Control>,
    resident: bool,
) -> Option<slint::Timer> {
    if !resident {
        return None;
    }
    let retry = Arc::clone(&control);
    window.on_reconnect(move || retry.request_reconnect());
    let weak = window.as_weak();
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, std::time::Duration::from_millis(250), move || {
        if let Some(window) = weak.upgrade() {
            show_state(&window, Some(control.connection()), true);
        }
    });
    Some(timer)
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub(super) fn bind(
    _window: &MainWindow,
    _profile: &Path,
    _control: Arc<Control>,
    _resident: bool,
) -> Option<slint::Timer> {
    None
}
