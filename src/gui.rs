/* VBWR B
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 * VibeWare is Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * VBWR E */

use crate::{
    model::{RuntimeSettings, Theme},
    settings::SettingsStore,
    tray::Control,
};
use anyhow::{Context, Result};
use slint::{ComponentHandle, Model};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

slint::include_modules!();

thread_local! {
    static ABOUT: RefCell<Option<slint::Weak<AboutDialog>>> = const { RefCell::new(None) };
    static RECOVERY: RefCell<Option<slint::Weak<RecoveryDialog>>> = const { RefCell::new(None) };
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
    recovery: Option<RecoveryDialog>,
}

impl Drop for Dialogs {
    // Hide any visible windows before restart and release the tray's weak reference.
    fn drop(&mut self) {
        let _ = self.config.hide();
        let _ = self.about.hide();
        if let Some(recovery) = &self.recovery {
            let _ = recovery.hide();
        }
        ABOUT.with(|slot| *slot.borrow_mut() = None);
        RECOVERY.with(|slot| *slot.borrow_mut() = None);
    }
}

// Errors: queuing a request while the Slint event loop is unavailable.
// The tray never constructs a dialog or runs a nested UI loop on its own thread.
pub fn show_about() -> Result<()> {
    slint::invoke_from_event_loop(|| {
        ABOUT.with(|about| {
            if let Some(about) = about.borrow().as_ref().and_then(slint::Weak::upgrade) {
                if let Err(error) = present_dialog(&about) {
                    eprintln!("Could not open the About dialog: {error:#}");
                }
            }
        });
    })
    .context("opening the About dialog")
}

// Use the same presentation path for tray requests and standalone UI commands.
// Defer native placement until Slint has created the window in its event loop.
fn present_dialog<T: ComponentHandle + 'static>(dialog: &T) -> Result<()> {
    dialog.window().set_minimized(false);
    dialog.window().set_maximized(false);
    dialog.show()?;
    #[cfg(target_os = "windows")]
    {
        let weak = dialog.as_weak();
        slint::Timer::single_shot(std::time::Duration::ZERO, move || {
            present_native_dialog(weak, 3);
        });
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn present_native_dialog<T: ComponentHandle + 'static>(weak: slint::Weak<T>, retries: u8) {
    let Some(dialog) = weak.upgrade() else {
        return;
    };
    if !dialog.window().is_visible() {
        return;
    }
    if let Err(error) = center_and_activate(dialog.window()) {
        // At standalone startup, Slint can dispatch timers before winit's first
        // Resumed event creates the HWND. Wait for readiness, not a fixed delay
        // assumed to be sufficient on every machine. Never retry focus denial.
        if retries > 0
            && matches!(
                error.downcast_ref::<raw_window_handle::HandleError>(),
                Some(raw_window_handle::HandleError::Unavailable)
            )
        {
            slint::Timer::single_shot(std::time::Duration::from_millis(16), move || {
                present_native_dialog(weak, retries - 1);
            });
        } else {
            eprintln!("Could not position or activate the dialog: {error:#}");
        }
    }
}

// Centre the complete native frame on the monitor where the user invoked the UI.
// Work-area coordinates include negative monitor origins and exclude the taskbar.
#[cfg(target_os = "windows")]
fn center_and_activate(window: &slint::Window) -> Result<()> {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::{
        Foundation::{POINT, RECT},
        Graphics::Gdi::{
            GetMonitorInfoW, MonitorFromPoint, MonitorFromWindow, MONITORINFO,
            MONITOR_DEFAULTTONEAREST,
        },
        UI::{
            HiDpi::GetDpiForWindow,
            WindowsAndMessaging::{
                GetCursorPos, GetWindowRect, SetForegroundWindow, SetWindowPos, HWND_TOP,
                SWP_SHOWWINDOW,
            },
        },
    };

    let handle = window.window_handle();
    let native = handle
        .window_handle()
        .context("accessing the native dialog")?;
    let RawWindowHandle::Win32(native) = native.as_raw() else {
        anyhow::bail!("the dialog has no Win32 window handle");
    };
    let hwnd = native.hwnd.get() as windows_sys::Win32::Foundation::HWND;
    // All handles and stack buffers remain valid during these synchronous calls;
    // this function runs exclusively on the Slint UI thread for this window.
    unsafe {
        let mut pointer = POINT::default();
        let monitor = if GetCursorPos(&mut pointer) != 0 {
            MonitorFromPoint(pointer, MONITOR_DEFAULTTONEAREST)
        } else {
            MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST)
        };
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        anyhow::ensure!(
            GetMonitorInfoW(monitor, &mut info) != 0,
            "reading the monitor work area"
        );
        let work = info.rcWork;
        // Moving between displays may synchronously resize the frame for its new
        // DPI. Re-read its resulting bounds before the final centring pass.
        for _ in 0..2 {
            let mut frame = RECT::default();
            anyhow::ensure!(
                GetWindowRect(hwnd, &mut frame) != 0,
                "reading the dialog bounds"
            );
            let margin = (16 * GetDpiForWindow(hwnd).max(96) / 96) as i32;
            let available_width = (work.right - work.left - 2 * margin).max(1);
            let available_height = (work.bottom - work.top - 2 * margin).max(1);
            let width = (frame.right - frame.left).clamp(1, available_width);
            let height = (frame.bottom - frame.top).clamp(1, available_height);
            let x = work.left + (work.right - work.left - width) / 2;
            let y = work.top + (work.bottom - work.top - height) / 2;
            anyhow::ensure!(
                SetWindowPos(hwnd, HWND_TOP, x, y, width, height, SWP_SHOWWINDOW) != 0,
                "centring the dialog"
            );
        }
        anyhow::ensure!(SetForegroundWindow(hwnd) != 0, "activating the dialog");
    }
    Ok(())
}

// connection: completed initialization or the start of a disconnected episode.
// Errors: queuing a request while the Slint event loop is unavailable.
#[cfg(target_os = "windows")]
pub fn connection_changed(connection: crate::tray::ConnectionState) -> Result<()> {
    // This modeless window leaves both the HID worker and the native tray thread free.
    slint::invoke_from_event_loop(move || {
        RECOVERY.with(|slot| {
            if let Some(recovery) = slot.borrow().as_ref().and_then(slint::Weak::upgrade) {
                let result = if connection == crate::tray::ConnectionState::Disconnected {
                    recovery.show()
                } else {
                    recovery.hide()
                };
                if let Err(error) = result {
                    eprintln!("Could not update the dock connection dialog: {error}");
                }
            }
        });
    })
    .context("updating the dock connection dialog")
}

/// Owns the editable settings, not the window. Callbacks hold this state without
/// keeping a closed Slint window alive.
struct Configurator {
    window: slint::Weak<MainWindow>,
    settings: RefCell<Option<SettingsStore>>,
    app_pages: Rc<slint::VecModel<AppPage>>,
    path: PathBuf,
    control: Arc<Control>,
    mode: Mode,
}

impl Configurator {
    fn reload(&self) {
        let Some(window) = self.window.upgrade() else {
            return;
        };

        match SettingsStore::load(&self.path) {
            Ok(loaded) => {
                let pages = loaded
                    .config
                    .pages
                    .iter()
                    .filter(|page| !page.executables.is_empty())
                    .map(|page| AppPage {
                        name: page.name.clone().into(),
                        enabled: page.enabled,
                    })
                    .collect::<Vec<_>>();
                self.app_pages.set_vec(pages);

                let theme_index = Theme::ALL
                    .iter()
                    .position(|&theme| theme == loaded.config.theme)
                    .expect("every theme is listed in Theme::ALL");
                window.set_theme_index(theme_index as i32);
                window.set_auto_switch(loaded.config.runtime.auto_switch);
                window.set_detect_applications(loaded.config.runtime.detect_applications);
                window.set_poll_seconds(loaded.config.runtime.focus_poll_seconds as i32);
                window.set_message("Choose your settings, then save.".into());
                window.set_ready(true);
                *self.settings.borrow_mut() = Some(loaded);
            }
            Err(error) => {
                window.set_ready(false);
                window.set_message(format!("{error:#}").into());
                *self.settings.borrow_mut() = None;
            }
        }
    }

    fn show(&self) {
        let Some(window) = self.window.upgrade() else {
            return;
        };

        // A second Configure click must not discard edits in the open window.
        if !window.window().is_visible() {
            self.reload();
        }
        window
            .window()
            .set_size(slint::LogicalSize::new(600., 740.));
        if let Err(error) = present_dialog(&window) {
            window.set_message(format!("{error:#}").into());
        }
    }

    fn save(&self, restart: bool) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let Some(&theme) = Theme::ALL.get(window.get_theme_index() as usize) else {
            return;
        };
        let mut stored = self.settings.borrow_mut();
        let Some(settings) = stored.as_mut() else {
            return;
        };

        let selected_pages = self
            .app_pages
            .iter()
            .map(|page| (page.name.to_string(), page.enabled))
            .collect::<Vec<_>>();
        let runtime = RuntimeSettings {
            detect_applications: window.get_detect_applications(),
            auto_switch: window.get_auto_switch(),
            focus_poll_seconds: window.get_poll_seconds() as u64,
        };
        let result = self
            .control
            .lock_settings()
            .and_then(|_lock| settings.save(theme, runtime, &selected_pages));

        match result {
            Ok(()) => {
                window.set_message("Saved. Restart the controller to apply changes.".into());
                if restart && self.mode == Mode::Resident {
                    self.control.request_restart();
                }
            }
            Err(error) => window.set_message(format!("{error:#}").into()),
        }
    }

    fn open_file(&self, catalogue: bool) {
        let file = if catalogue {
            self.path.with_file_name("applications.yaml")
        } else {
            self.path.clone()
        };
        let result = (|| -> Result<()> {
            // The editor needs a real file, but normal preference saves never rewrite YAML.
            if catalogue && self.path.is_file() && !file.exists() {
                crate::applications::create_default(&file)?;
            }
            anyhow::ensure!(
                file.is_file(),
                "{} does not exist yet. Save settings first.",
                file.display()
            );
            crate::tray::open_file(&file)
        })();
        if let (Err(error), Some(window)) = (result, self.window.upgrade()) {
            window.set_message(format!("{error:#}").into());
        }
    }
}

fn bind_configurator(
    window: &MainWindow,
    path: PathBuf,
    control: Arc<Control>,
    mode: Mode,
) -> Rc<Configurator> {
    window.set_resident(mode == Mode::Resident);
    let themes = Theme::ALL
        .iter()
        .map(|theme| theme.label().into())
        .collect::<Vec<slint::SharedString>>();
    window.set_themes(slint::ModelRc::new(slint::VecModel::from(themes)));

    let app_pages = Rc::new(slint::VecModel::<AppPage>::default());
    window.set_app_pages(slint::ModelRc::from(Rc::clone(&app_pages)));
    let state = Rc::new(Configurator {
        window: window.as_weak(),
        settings: RefCell::new(None),
        app_pages,
        path,
        control,
        mode,
    });

    let toggled = Rc::clone(&state);
    window.on_page_toggled(move |index, enabled| {
        if let Some(mut page) = toggled.app_pages.row_data(index as usize) {
            page.enabled = enabled;
            toggled.app_pages.set_row_data(index as usize, page);
        }
    });
    let reload = Rc::clone(&state);
    window.on_reload(move || reload.reload());
    let save = Rc::clone(&state);
    window.on_save(move |restart| save.save(restart));
    let open = Rc::clone(&state);
    window.on_open_file(move |catalogue| open.open_file(catalogue));

    let weak = window.as_weak();
    window.on_learn_vibeware(move || {
        let result = crate::tray::open_browser("https://umbertogiacobbi.biz/vibeware/manifesto");
        if let (Err(error), Some(window)) = (result, weak.upgrade()) {
            window.set_message(format!("{error:#}").into());
        }
    });

    state
}

fn bind_about_links(about: &AboutDialog) {
    let weak = about.as_weak();
    about.on_open_link(move |target| {
        let url = match target.as_str() {
            "project" => env!("CARGO_PKG_REPOSITORY"),
            "vibeware" => "https://github.com/umbertotechnopreneur/VibeWare",
            "manifesto" => "https://umbertogiacobbi.biz/vibeware/manifesto",
            "website" => "https://umbertogiacobbi.biz",
            "terms" => "https://umbertogiacobbi.biz/terms/",
            "privacy" => "https://umbertogiacobbi.biz/privacy/",
            _ => return,
        };
        let result = crate::tray::open_browser(url);
        if let Some(about) = weak.upgrade() {
            about.set_message(match result {
                Ok(()) => "Opened in your default browser.".into(),
                Err(error) => format!("{error:#}").into(),
            });
        }
    });
}

/// The close button and title-bar X must have the same lifecycle behavior.
fn bind_close_handlers(about: &AboutDialog, window: &MainWindow, mode: Mode) {
    let about_weak = about.as_weak();
    let config_weak = window.as_weak();
    let close_about: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(about) = about_weak.upgrade() {
            let _ = about.hide();
        }
        if mode != Mode::Resident {
            if let Some(config) = config_weak.upgrade() {
                let _ = config.hide();
            }
            let _ = slint::quit_event_loop();
        }
    });
    let dismiss_about = Rc::clone(&close_about);
    about.on_dismiss(move || dismiss_about());
    about.window().on_close_requested(move || {
        close_about();
        slint::CloseRequestResponse::HideWindow
    });

    let weak = window.as_weak();
    let close_config: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(config) = weak.upgrade() {
            let _ = config.hide();
        }
        if mode == Mode::Configure {
            let _ = slint::quit_event_loop();
        }
    });
    let dismiss_config = Rc::clone(&close_config);
    window.on_dismiss(move || dismiss_config());
    window.window().on_close_requested(move || {
        close_config();
        slint::CloseRequestResponse::HideWindow
    });
}

fn create_recovery() -> Result<RecoveryDialog> {
    let recovery = RecoveryDialog::new().context("creating the recovery dialog")?;
    let weak = recovery.as_weak();
    recovery.on_dismiss(move || {
        if let Some(recovery) = weak.upgrade() {
            let _ = recovery.hide();
        }
    });
    recovery
        .window()
        .on_close_requested(|| slint::CloseRequestResponse::HideWindow);
    RECOVERY.with(|slot| *slot.borrow_mut() = Some(recovery.as_weak()));
    Ok(recovery)
}

fn create(path: PathBuf, control: Arc<Control>, mode: Mode) -> Result<Dialogs> {
    let about = AboutDialog::new().context("creating the About dialog")?;
    let config = MainWindow::new().context("creating the configurator")?;
    let state = bind_configurator(&config, path, control, mode);

    about.set_version(env!("CARGO_PKG_VERSION").into());
    let configure = Rc::clone(&state);
    about.on_configure(move || configure.show());
    bind_about_links(&about);
    bind_close_handlers(&about, &config, mode);

    let recovery = if mode == Mode::Resident {
        Some(create_recovery()?)
    } else {
        None
    };
    ABOUT.with(|slot| *slot.borrow_mut() = Some(about.as_weak()));
    if mode == Mode::Configure {
        state.reload();
    }

    Ok(Dialogs {
        about,
        config,
        recovery,
    })
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
        present_dialog(&dialogs.config)?;
    } else {
        present_dialog(&dialogs.about)?;
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
    // All windows stay hidden at startup. Recovery is shown only after a connected dock fails.
    let _console = crate::console::detach_if_owned()?;
    if _console.is_some() {
        control.enter_background();
    }
    // Install the tray before the controller discovers or writes to the dock.
    let _tray = crate::tray::Tray::start(Arc::clone(&control))?;
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
