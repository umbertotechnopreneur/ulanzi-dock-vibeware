/* VBWR B
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 * VibeWare is Human intent. AI implementation. Accountable human review.
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * VBWR E */

use crate::{gui_settings::Settings, model::Theme, tray::Control};
use anyhow::{Context, Result};
use slint::ComponentHandle;
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

slint::include_modules!();

thread_local! {
    static ABOUT: RefCell<Option<slint::Weak<AboutDialog>>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Resident,
    About,
    Configure,
}

struct Dialogs {
    about: AboutDialog,
    config: MainWindow,
}

impl Drop for Dialogs {
    // Hide any visible windows before restart and release the tray's weak reference.
    fn drop(&mut self) {
        let _ = self.config.hide();
        let _ = self.about.hide();
        ABOUT.with(|slot| *slot.borrow_mut() = None);
    }
}

// Errors: queuing a request while the Slint event loop is unavailable.
// The tray never constructs a dialog or runs a nested UI loop on its own thread.
pub fn show_about() -> Result<()> {
    slint::invoke_from_event_loop(|| {
        ABOUT.with(|about| {
            if let Some(about) = about.borrow().as_ref().and_then(slint::Weak::upgrade) {
                let _ = about.show();
            }
        });
    })
    .context("opening the About dialog")
}

// path: settings file shared with the resident controller.
// control: shared lifecycle state for Save + Restart.
// mode: decides whether closing a dialog hides it or ends a standalone UI command.
// Errors: UI initialization or embedded icon decoding.
fn create(path: PathBuf, control: Arc<Control>, mode: Mode) -> Result<Dialogs> {
    let about = AboutDialog::new().context("creating the About dialog")?;
    let config = MainWindow::new().context("creating the configurator")?;
    let pixels = image::load_from_memory(include_bytes!(concat!(
        env!("OUT_DIR"),
        "/vibeware-pixel.png"
    )))?
    .to_rgba8();
    let icon = slint::Image::from_rgba8(
        slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
            pixels.as_raw(),
            pixels.width(),
            pixels.height(),
        ),
    );
    about.set_app_icon(icon.clone());
    config.set_app_icon(icon);
    about.set_version(env!("CARGO_PKG_VERSION").into());
    config.set_resident(mode == Mode::Resident);
    config.set_themes(slint::ModelRc::new(slint::VecModel::from(
        Theme::ALL
            .iter()
            .map(|theme| theme.label().into())
            .collect::<Vec<slint::SharedString>>(),
    )));
    let settings = Rc::new(RefCell::new(None::<Settings>));
    let reload: Rc<dyn Fn()> = {
        let weak = config.as_weak();
        let settings = Rc::clone(&settings);
        let path = path.clone();
        Rc::new(move || {
            let Some(config) = weak.upgrade() else {
                return;
            };
            match Settings::load(&path) {
                Ok(loaded) => {
                    config.set_theme_index(
                        Theme::ALL
                            .iter()
                            .position(|&theme| theme == loaded.config.theme)
                            .unwrap_or(0) as i32,
                    );
                    config.set_auto_switch(loaded.config.runtime.auto_switch);
                    config.set_detect_applications(loaded.config.runtime.detect_applications);
                    config.set_poll_seconds(loaded.config.runtime.focus_poll_seconds as i32);
                    config.set_message("Choose your settings, then save.".into());
                    config.set_ready(true);
                    *settings.borrow_mut() = Some(loaded);
                }
                Err(error) => {
                    config.set_ready(false);
                    config.set_message(format!("{error:#}").into());
                    *settings.borrow_mut() = None;
                }
            }
        })
    };
    let reload_callback = Rc::clone(&reload);
    config.on_reload(move || reload_callback());
    let weak = config.as_weak();
    let open_config = config.as_weak();
    let configure_reload = Rc::clone(&reload);
    about.on_configure(move || {
        if let Some(config) = open_config.upgrade() {
            // Clicking Configure again must not discard edits in an already open window.
            if !config.window().is_visible() {
                configure_reload();
            }
            config.window().set_minimized(false);
            let _ = config.show();
        }
    });
    let save_control = Arc::clone(&control);
    config.on_save(move |restart| {
        let Some(config) = weak.upgrade() else {
            return;
        };
        let index = config.get_theme_index() as usize;
        let Some(&theme) = Theme::ALL.get(index) else {
            return;
        };
        let mut settings = settings.borrow_mut();
        let Some(settings) = settings.as_mut() else {
            return;
        };
        let result = save_control.lock_settings().and_then(|_lock| {
            settings.save(
                theme,
                config.get_detect_applications(),
                config.get_auto_switch(),
                config.get_poll_seconds() as u64,
            )
        });
        match result {
            Ok(()) => {
                config.set_message("Saved. Restart the controller to apply changes.".into());
                if restart && mode == Mode::Resident {
                    save_control.request_restart();
                }
            }
            Err(error) => config.set_message(format!("{error:#}").into()),
        }
    });
    let weak = config.as_weak();
    config.on_open_file(move |catalogue| {
        let file = if catalogue {
            path.with_file_name("applications.yaml")
        } else {
            path.clone()
        };
        // Opening a missing file must not silently create or save configuration.
        let result = if file.is_file() {
            crate::tray::open_file(&file)
        } else {
            Err(anyhow::anyhow!(
                "{} does not exist yet. Save settings first.",
                file.display()
            ))
        };
        if let (Err(error), Some(config)) = (result, weak.upgrade()) {
            config.set_message(format!("{error:#}").into());
        }
    });
    let weak = about.as_weak();
    about.on_open_link(move |target| {
        let url = match target.as_str() {
            "project" => env!("CARGO_PKG_REPOSITORY"),
            "vibeware" => "https://github.com/umbertotechnopreneur/VibeWare",
            "manifesto" => "https://umbertogiacobbi.biz/vibeware/manifesto",
            _ => return,
        };
        if let (Err(error), Some(about)) = (crate::tray::open_browser(url), weak.upgrade()) {
            about.set_message(format!("{error:#}").into());
        }
    });
    let weak = about.as_weak();
    let config_weak = config.as_weak();
    about.on_dismiss(move || {
        if let Some(about) = weak.upgrade() {
            let _ = about.hide();
        }
        if mode != Mode::Resident {
            if let Some(config) = config_weak.upgrade() {
                let _ = config.hide();
            }
            let _ = slint::quit_event_loop();
        }
    });
    let weak = config.as_weak();
    config.on_dismiss(move || {
        if let Some(config) = weak.upgrade() {
            let _ = config.hide();
        }
        if mode == Mode::Configure {
            let _ = slint::quit_event_loop();
        }
    });
    let config_weak = config.as_weak();
    about.window().on_close_requested(move || {
        if mode != Mode::Resident {
            if let Some(config) = config_weak.upgrade() {
                let _ = config.hide();
            }
            let _ = slint::quit_event_loop();
        }
        slint::CloseRequestResponse::HideWindow
    });
    config.window().on_close_requested(move || {
        if mode == Mode::Configure {
            let _ = slint::quit_event_loop();
        }
        slint::CloseRequestResponse::HideWindow
    });
    ABOUT.with(|slot| *slot.borrow_mut() = Some(about.as_weak()));
    if mode == Mode::Configure {
        reload();
    }
    Ok(Dialogs { about, config })
}

// path: configuration file used by the standalone dialog.
// configure: open the configurator directly instead of the About dialog.
// control: application lifecycle shared with the caller.
// Errors: UI creation or event-loop failure. Never discovers or opens HID.
pub fn standalone(path: PathBuf, configure: bool, control: Arc<Control>) -> Result<()> {
    let dialogs = create(
        path,
        Arc::clone(&control),
        if configure {
            Mode::Configure
        } else {
            Mode::About
        },
    )?;
    let _console = crate::console::detach_if_owned()?;
    control.enter_background();
    if configure {
        dialogs.config.show()?;
    } else {
        dialogs.about.show()?;
    }
    slint::run_event_loop_until_quit().context("running the dialogs")
}

// path: configuration shared between the controller and configurator.
// control: stop/restart state, used to end the worker before the process exits.
// controller: blocking HID operation, kept off the main Slint UI thread.
// Errors: dialog creation, thread startup, UI loop, panic, or controller operation failure.
#[cfg(target_os = "windows")]
pub fn resident<F>(path: PathBuf, control: Arc<Control>, controller: F) -> Result<()>
where
    F: FnOnce() -> Result<()> + Send + 'static,
{
    let _dialogs = create(path, Arc::clone(&control), Mode::Resident)?;
    // Neither window is shown at startup. Only an explicit About/menu action shows UI.
    let worker = std::thread::Builder::new()
        .name("ulanzi-controller".into())
        .spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(controller))
                .unwrap_or_else(|_| Err(anyhow::anyhow!("The controller thread panicked.")));
            let _ = slint::invoke_from_event_loop(|| {
                let _ = slint::quit_event_loop();
            });
            result
        })
        .context("starting the controller thread")?;
    let ui_result = slint::run_event_loop_until_quit().context("running the dialog event loop");
    control.stop();
    let result = worker
        .join()
        .map_err(|_| anyhow::anyhow!("The controller thread panicked."))?;
    result?;
    ui_result
}
