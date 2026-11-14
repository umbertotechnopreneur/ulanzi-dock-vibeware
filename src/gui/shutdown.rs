// Copyright (c) 2026 Umberto Giacobbi. SPDX-License-Identifier: MIT.

//! Show shutdown progress while the controller saves state and releases USB.

use super::{ShutdownDialog, ABOUT, CONFIGURATOR};
use crate::tray::linux::connection;
use anyhow::{Context, Result};
use slint::ComponentHandle;
use std::{cell::RefCell, path::PathBuf, rc::Rc, thread, time::{Duration, Instant}};
use zbus::blocking::connection::Builder;

thread_local! {
    static START: RefCell<Option<Rc<dyn Fn()>>> = const { RefCell::new(None) };
}

pub(super) fn clear() {
    START.with(|slot| slot.borrow_mut().take());
}

pub(super) fn bind(dialog: &ShutdownDialog, profile: PathBuf) {
    let weak = dialog.as_weak();
    START.with(|slot| *slot.borrow_mut() = Some(Rc::new(move || {
        let Some(dialog) = weak.upgrade() else { return; };
        if dialog.get_busy() {
            return;
        }
        CONFIGURATOR.with(|slot| {
            if let Some(state) = slot.borrow().as_ref().and_then(std::rc::Weak::upgrade) {
                state.wait_for_worker();
                if let Some(window) = state.window.upgrade() {
                    let _ = window.hide();
                }
            }
        });
        ABOUT.with(|slot| {
            if let Some(about) = slot.borrow().as_ref().and_then(slint::Weak::upgrade) {
                let _ = about.hide();
            }
        });
        dialog.set_busy(true);
        dialog.set_message("Shutting down UlanziDeck…".into());
        if let Err(error) = dialog.show() {
            eprintln!("Could not show shutdown progress: {error}");
            dialog.set_busy(false);
            return;
        }
        let profile = profile.clone();
        let weak = dialog.as_weak();
        let worker = thread::Builder::new().name("dock-shutdown".into()).spawn(move || {
            let result = stop_controller(&profile);
            let _ = weak.upgrade_in_event_loop(move |dialog| {
                match result {
                    Ok(()) => { let _ = slint::quit_event_loop(); }
                    Err(error) => {
                        eprintln!("Could not shut down UlanziDeck: {error:#}");
                        dialog.set_message(format!("Could not shut down UlanziDeck: {error:#}").into());
                        dialog.set_busy(false);
                    }
                }
            });
        });
        if let Err(error) = worker {
            dialog.set_message(format!("Could not start shutdown: {error}").into());
            dialog.set_busy(false);
        }
    })));
    dialog.on_dismiss(|| { let _ = slint::quit_event_loop(); });
    let weak = dialog.as_weak();
    dialog.window().on_close_requested(move || {
        if weak.upgrade().is_some_and(|dialog| dialog.get_busy()) {
            return slint::CloseRequestResponse::KeepWindowShown;
        }
        let _ = slint::quit_event_loop();
        slint::CloseRequestResponse::HideWindow
    });
}

pub(super) fn show() -> Result<()> {
    slint::invoke_from_event_loop(|| START.with(|slot| {
        if let Some(start) = slot.borrow().as_ref() {
            start();
        }
    })).context("showing shutdown progress")
}

pub(crate) fn standalone(profile: PathBuf) -> Result<()> {
    let Some(_instance) = crate::singleton::acquire_configurator()? else {
        let bus = zbus::Connection::session();
        return futures_lite::future::block_on(async {
            let bus = bus.await?;
            anyhow::ensure!(super::linux::request(&bus, &profile, "Shutdown").await?,
                "UlanziDeck is already closing. Wait for the open window.");
            Ok(())
        });
    };
    let dialog = ShutdownDialog::new().context("creating shutdown progress")?;
    slint::set_xdg_app_id("ulanzideck")?;
    bind(&dialog, profile);
    show()?;
    slint::run_event_loop_until_quit().context("showing shutdown progress")
}

fn stop_controller(profile: &std::path::Path) -> Result<()> {
    let bus = Builder::session()?.method_timeout(Duration::from_secs(1)).build()?;
    let name = connection::name(profile)?;
    connection::quit(&bus, &name)?;
    let deadline = Instant::now() + Duration::from_secs(15);
    while connection::state(&bus, &name)?.is_some() {
        anyhow::ensure!(Instant::now() < deadline, "The controller is still closing. Check the application log.");
        thread::sleep(Duration::from_millis(100));
    }
    Ok(())
}
