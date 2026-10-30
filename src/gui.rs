/* VBWR B
 *
 * Project: UlanziDeck
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Modified with AI: OpenAI Codex; dialog placement, Store bindings and callback contracts and readability; 2026-10-27
 * Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
 * Evidence: docs/CODE-REVIEW.md; current working-tree changes
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 *
 * VBWR E */

//! Own Slint dialogs and translate UI gestures into settings/lifecycle requests.
//! Callbacks retain editor snapshots, while weak handles avoid window ownership
//! cycles. Placement and Store bindings have separate modules. Standalone dialogs
//! never discover HID devices; resident mode runs the controller on another thread.

use crate::{
    model::Theme,
    settings::SettingsStore,
    tray::Control,
};
use anyhow::{Context, Result};
use slint::{ComponentHandle, Model};
use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

slint::include_modules!();

mod placement;
mod store_ui;
use placement::DialogKind;

thread_local! {
    static ABOUT: RefCell<Option<slint::Weak<AboutDialog>>> = const { RefCell::new(None) };
    static CONFIGURATOR: RefCell<Option<std::rc::Weak<Configurator>>> = const { RefCell::new(None) };
}

#[derive(Clone, Copy, PartialEq, Eq)]
/// Determines which close actions end the UI loop and whether restart is offered.
enum Mode {
    Resident,
    About,
    Configure,
}

/// Strong window owners and timer lifetime for one GUI session.
struct Dialogs {
    about: AboutDialog,
    config: MainWindow,
    _store_timer: slint::Timer,
}

impl Drop for Dialogs {
    // Hide any visible windows before restart and release the tray's weak reference.
    fn drop(&mut self) {
        placement::remember(self.config.window(), DialogKind::Configuration);
        placement::remember(self.about.window(), DialogKind::About);
        let _ = self.config.hide();
        let _ = self.about.hide();
        ABOUT.with(|slot| *slot.borrow_mut() = None);
        CONFIGURATOR.with(|slot| *slot.borrow_mut() = None);
    }
}

// Errors: queuing a request while the Slint event loop is unavailable.
// The tray never constructs a dialog or runs a nested UI loop on its own thread.
pub fn show_about() -> Result<()> {
    slint::invoke_from_event_loop(|| {
        ABOUT.with(|about| {
            if let Some(about) = about.borrow().as_ref().and_then(slint::Weak::upgrade) {
                if let Err(error) = present_dialog(&about, DialogKind::About) {
                    eprintln!("Could not open the About dialog: {error:#}");
                }
            }
        });
    })
    .context("opening the About dialog")
}

// Use the same presentation path for tray requests and standalone UI commands.
// Defer native placement until Slint has created the window in its event loop.
fn present_dialog<T: ComponentHandle + 'static>(dialog: &T, kind: DialogKind) -> Result<()> {
    #[cfg(target_os = "windows")]
    let restore = !dialog.window().is_visible();
    #[cfg(not(target_os = "windows"))]
    let _ = kind;
    dialog.window().set_minimized(false);
    dialog.show()?;
    #[cfg(target_os = "windows")]
    {
        let weak = dialog.as_weak();
        slint::Timer::single_shot(std::time::Duration::ZERO, move || {
            present_native_dialog(weak, kind, restore, 32);
        });
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn present_native_dialog<T: ComponentHandle + 'static>(
    weak: slint::Weak<T>,
    kind: DialogKind,
    restore: bool,
    retries: u8,
) {
    let Some(dialog) = weak.upgrade() else {
        return;
    };
    if !dialog.window().is_visible() {
        return;
    }
    if let Err(error) = placement::present(dialog.window(), kind, restore, false) {
        // At standalone startup, Slint can dispatch timers before winit's first
        // Resumed event creates the HWND. Wait for readiness, not a fixed delay
        // assumed to be sufficient on every machine. Never retry focus denial.
        if retries > 0
            && matches!(
                error.downcast_ref::<raw_window_handle::HandleError>(),
                Some(
                    raw_window_handle::HandleError::Unavailable
                        | raw_window_handle::HandleError::NotSupported
                )
            )
        {
            slint::Timer::single_shot(std::time::Duration::from_millis(16), move || {
                present_native_dialog(weak, kind, restore, retries - 1);
            });
        } else {
            eprintln!("Could not position or activate the dialog: {error:#}");
        }
    }
}

/// Present the first welcome on the primary display, then persist completion.
/// `path` identifies the real profile; `retries` only covers HWND creation latency.
#[cfg(target_os = "windows")]
fn finish_first_welcome(weak: slint::Weak<AboutDialog>, path: PathBuf, retries: u8) {
    let Some(about) = weak.upgrade() else {
        return;
    };
    let result = placement::present(about.window(), DialogKind::About, true, true);
    if let Err(error) = result {
        let handle_pending = matches!(
            error.downcast_ref::<raw_window_handle::HandleError>(),
            Some(
                raw_window_handle::HandleError::Unavailable
                    | raw_window_handle::HandleError::NotSupported
            )
        );
        if retries > 0 && handle_pending {
            slint::Timer::single_shot(std::time::Duration::from_millis(16), move || {
                finish_first_welcome(weak, path, retries - 1);
            });
        } else {
            about.set_message(format!("Could not position the welcome: {error:#}").into());
        }
        return;
    }
    if let Err(error) =
        SettingsStore::load(&path).and_then(|mut settings| settings.save_first_run_completed())
    {
        about.set_message(format!("Could not remember the welcome: {error:#}").into());
    }
}

/// Show the initial About before its native placement/first-run save can complete.
/// `path` selects the profile to mark; no controller or HID session starts here.
#[cfg(target_os = "windows")]
fn show_first_welcome(about: &AboutDialog, path: PathBuf) -> Result<()> {
    about.show()?;
    let weak = about.as_weak();
    slint::Timer::single_shot(std::time::Duration::ZERO, move || {
        finish_first_welcome(weak, path, 32);
    });
    Ok(())
}

/// Owns the editable settings, not the window. Callbacks hold this state without
/// keeping a closed Slint window alive.
struct Configurator {
    window: slint::Weak<MainWindow>,
    settings: RefCell<Option<SettingsStore>>,
    app_pages: Rc<slint::VecModel<AppPage>>,
    path: PathBuf,
    control: Arc<Control>,
}

impl Configurator {
    /// Replace editor fields from disk, including stale-write snapshots. Display
    /// parse/validation errors and disable saving rather than fabricating defaults.
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
                window.set_message("Changes are saved and applied automatically.".into());
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

    /// Open or activate configuration; keep edits intact if it is already visible.
    fn show(&self) {
        let Some(window) = self.window.upgrade() else {
            return;
        };

        // A second Configure click must not discard edits in the open window.
        if !window.window().is_visible() {
            self.reload();
            crate::licensing::refresh();
        }
        if let Err(error) = present_dialog(&window, DialogKind::Configuration) {
            window.set_message(format!("{error:#}").into());
        }
    }

    /// Save a user gesture atomically; failures remain visible for retry.
    fn save(&self, theme_changed: bool) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let Some(&theme) = Theme::ALL.get(window.get_theme_index() as usize) else {
            return;
        };
        let stored = self.settings.borrow();
        let Some(previous) = stored.as_ref() else { return; };
        let selected_pages = self.app_pages.iter()
            .filter(|page| previous.config.pages.iter().any(|old|
                old.name == page.name.as_str() && old.enabled != page.enabled))
            .map(|page| (page.name.to_string(), page.enabled)).collect::<Vec<_>>();
        // Merge only changed fields into the latest disk snapshot. A dock theme
        // selection must survive an unrelated checkbox change in this window.
        let result = (|| -> Result<()> {
            let _lock = self.control.lock_settings()?;
            let mut latest = SettingsStore::load(&self.path)?;
            let mut runtime = latest.config.runtime;
            if window.get_detect_applications() != previous.config.runtime.detect_applications {
                runtime.detect_applications = window.get_detect_applications();
            }
            if window.get_auto_switch() != previous.config.runtime.auto_switch {
                runtime.auto_switch = window.get_auto_switch();
            }
            if window.get_poll_seconds() as u64 != previous.config.runtime.focus_poll_seconds {
                runtime.focus_poll_seconds = window.get_poll_seconds() as u64;
            }
            let theme = if theme_changed { theme } else { latest.config.theme };
            latest.save(theme, runtime, &selected_pages)
        })();
        drop(stored);
        match result {
            Ok(()) => self.reload(),
            Err(error) => window.set_message(format!("Could not save changes: {error:#}").into()),
        }
    }

    /// Open JSON or, when `catalogue`, YAML in its associated editor. Seed missing
    /// YAML only beside an existing settings file. Report shell/persistence errors.
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

/// Attach `window` callbacks to editor state for `path`; `control` carries restart
/// requests and `mode` determines resident-only features. Does not run a UI loop.
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
    });
    CONFIGURATOR.with(|slot| *slot.borrow_mut() = Some(Rc::downgrade(&state)));

    let toggled = Rc::clone(&state);
    window.on_page_toggled(move |index, enabled| {
        if let Some(mut page) = toggled.app_pages.row_data(index as usize) {
            page.enabled = enabled;
            toggled.app_pages.set_row_data(index as usize, page);
            toggled.save(false);
        }
    });
    let save = Rc::clone(&state);
    window.on_preferences_changed(move || save.save(false));
    let theme = Rc::clone(&state);
    window.on_theme_changed(move || theme.save(true));
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

/// Bind fixed trusted link targets on `about`, reporting browser-launch failures.
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
            placement::remember(about.window(), DialogKind::About);
            let _ = about.hide();
        }
        if mode != Mode::Resident {
            if let Some(config) = config_weak.upgrade() {
                placement::remember(config.window(), DialogKind::Configuration);
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
            placement::remember(config.window(), DialogKind::Configuration);
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

/// Construct and bind windows for `path` and `mode`, sharing lifecycle `control`.
/// Return strong owners; UI creation failures occur before controller startup.
fn create(path: PathBuf, control: Arc<Control>, mode: Mode) -> Result<Dialogs> {
    placement::initialize(&path);
    let about = AboutDialog::new().context("creating the About dialog")?;
    let config = MainWindow::new().context("creating the configurator")?;
    let state = bind_configurator(&config, path, control, mode);
    let store_timer = store_ui::bind(&config);

    about.set_version(env!("CARGO_PKG_VERSION").into());
    let configure = Rc::clone(&state);
    about.on_configure(move || configure.show());
    bind_about_links(&about);
    bind_close_handlers(&about, &config, mode);

    ABOUT.with(|slot| *slot.borrow_mut() = Some(about.as_weak()));
    if mode == Mode::Configure {
        state.reload();
    }

    Ok(Dialogs {
        about,
        config,
        _store_timer: store_timer,
    })
}

// path: configuration file used by the standalone dialog.
// configure: open the configurator directly instead of the About dialog.
// control: application lifecycle shared with the caller.
// Errors: UI creation or event-loop failure. Never discovers or opens HID.
pub fn standalone(path: PathBuf, configure: bool, control: Arc<Control>) -> Result<()> {
    #[cfg(target_os = "windows")]
    let first_welcome = !configure && SettingsStore::load(&path)?.config.is_first_run;
    let dialogs = create(
        path.clone(),
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
        present_dialog(&dialogs.config, DialogKind::Configuration)?;
    } else {
        #[cfg(target_os = "windows")]
        if first_welcome {
            show_first_welcome(&dialogs.about, path)?;
        } else {
            present_dialog(&dialogs.about, DialogKind::About)?;
        }
        #[cfg(not(target_os = "windows"))]
        present_dialog(&dialogs.about, DialogKind::About)?;
    }
    slint::run_event_loop_until_quit().context("running the dialogs")
}

// path: configuration shared between the controller and configurator.
// control: stop/restart state, used to end the worker before the process exits.
// controller: blocking HID operation, kept off the main Slint UI thread.
// Errors: dialog creation, thread startup, UI loop, panic, or controller operation failure.
#[cfg(target_os = "windows")]
pub fn resident<F>(
    path: PathBuf,
    control: Arc<Control>,
    first_run: bool,
    controller: F,
) -> Result<()>
where
    F: FnOnce() -> Result<()> + Send + 'static,
{
    let dialogs = create(path.clone(), Arc::clone(&control), Mode::Resident)?;
    if first_run {
        show_first_welcome(&dialogs.about, path)?;
    }
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
