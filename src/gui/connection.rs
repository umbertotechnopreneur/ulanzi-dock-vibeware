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
#[derive(Clone, Copy, PartialEq, Eq)]
enum ControllerStatus {
    #[cfg(target_os = "linux")]
    Unavailable,
    #[cfg(target_os = "linux")]
    Stopped,
    #[cfg(target_os = "linux")]
    Starting,
    Dock(ConnectionState),
}

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn show_state(window: &MainWindow, state: ControllerStatus, can_retry: bool) {
    use super::DockStatus;
    let status = match state {
        #[cfg(target_os = "linux")]
        ControllerStatus::Unavailable => DockStatus::Unavailable,
        #[cfg(target_os = "linux")]
        ControllerStatus::Stopped => DockStatus::ControllerStopped,
        #[cfg(target_os = "linux")]
        ControllerStatus::Starting => DockStatus::ControllerStarting,
        ControllerStatus::Dock(ConnectionState::Connected) => DockStatus::Connected,
        ControllerStatus::Dock(ConnectionState::Waiting) => DockStatus::Waiting,
        ControllerStatus::Dock(ConnectionState::Disconnected) => DockStatus::Disconnected,
    };
    window.set_dock_status(status);
    window.set_can_reconnect(can_retry);
}

#[cfg(target_os = "linux")]
pub(super) fn bind(
    window: &MainWindow,
    profile: &Path,
    _control: Arc<Control>,
    _resident: bool,
    start_controller: bool,
) -> Option<slint::Timer> {
    use crate::tray::linux::connection as link;
    use std::{sync::mpsc, thread, time::{Duration, Instant}};
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
        let mut start_deadline = None;
        let mut error = None;
        if start_controller {
            if weak.upgrade_in_event_loop(|window| {
                show_state(&window, ControllerStatus::Starting, false);
            }).is_err() {
                return;
            }
            let result = if managed {
                crate::startup::launch_controller()
            } else {
                Err(anyhow::anyhow!("Only the installed default profile can start the controller from Settings."))
            };
            match result {
                Ok(()) => start_deadline = Some(Instant::now() + Duration::from_secs(5)),
                Err(cause) => error = Some(format!("Could not start the controller: {cause:#}")),
            }
        }
        loop {
            if bus.is_none() {
                bus = Builder::session()
                    .and_then(|builder| builder.method_timeout(Duration::from_millis(750)).build())
                    .ok();
            }
            if retry {
                let result = match &bus {
                    Some(bus) => link::reconnect(bus, &name, managed),
                    None => Err(anyhow::anyhow!("Could not connect to the desktop session bus.")),
                };
                match result {
                    Ok(true) => start_deadline = Some(Instant::now() + Duration::from_secs(5)),
                    Ok(false) => {}
                    Err(cause) => error = Some(format!("Could not reconnect: {cause:#}")),
                }
            }
            let (state, can_retry) = match bus.as_ref().map(|bus| link::state(bus, &name)) {
                Some(Ok(Some(state))) => {
                    start_deadline = None;
                    (ControllerStatus::Dock(state), state == ConnectionState::Disconnected)
                }
                Some(Ok(None)) if start_deadline.is_some_and(|end| Instant::now() < end) => {
                    (ControllerStatus::Starting, false)
                }
                Some(Ok(None)) => {
                    if start_deadline.take().is_some() {
                        error = Some("The controller did not start for this profile. Check the application log.".into());
                    }
                    (ControllerStatus::Stopped, managed)
                }
                _ => {
                    bus = None;
                    (ControllerStatus::Unavailable, false)
                }
            };
            let view = (state, can_retry);
            let changed = previous != Some(view);
            if error.is_some() || retry || changed {
                let message = if let Some(error) = error.take() {
                    eprintln!("{error}");
                    Some(error)
                } else {
                    (retry || (changed && matches!(state, ControllerStatus::Dock(_))))
                        .then(String::new)
                };
                if weak.upgrade_in_event_loop(move |window| {
                    show_state(&window, state, can_retry);
                    if let Some(message) = message {
                        window.set_dock_message(message.into());
                    }
                }).is_err() {
                    break;
                }
                previous = Some(view);
            }
            // Dropping the dialog releases the sender and stops the worker.
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
    window.on_reconnect(move || {
        let _ = request.try_send(());
    });
    None
}

#[cfg(target_os = "windows")]
pub(super) fn bind(
    window: &MainWindow,
    _profile: &Path,
    control: Arc<Control>,
    resident: bool,
    _start_controller: bool,
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
            let state = control.connection();
            show_state(&window, ControllerStatus::Dock(state), state == ConnectionState::Disconnected);
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
    _start_controller: bool,
) -> Option<slint::Timer> {
    None
}
