/* VBWR B
 *
 * Project: UlanziDeck
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Modified with AI: OpenAI Codex; dialog placement, Store bindings and installed Linux app choices; 2026-11-02
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
    model::{Config, Theme},
    settings::{SettingsChange, SettingsStore},
    tray::Control,
};
use anyhow::{Context, Result};
use slint::ComponentHandle;
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    sync::Arc,
    thread::{self, JoinHandle},
};

slint::include_modules!();

mod connection;
#[cfg(target_os = "linux")]
pub(crate) mod linux;
#[cfg(target_os = "linux")]
pub(crate) mod shutdown;
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
    _store_timer: Option<slint::Timer>,
    _connection_timer: Option<slint::Timer>,
    #[cfg(target_os = "linux")]
    _shutdown: ShutdownDialog,
}

impl Drop for Dialogs {
    // Hide any visible windows before restart and release the tray's weak reference.
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        shutdown::clear();
        placement::remember(self.config.window(), DialogKind::Configuration);
        placement::remember(self.about.window(), DialogKind::About);
        let _ = self.config.hide();
        let _ = self.about.hide();
        ABOUT.with(|slot| *slot.borrow_mut() = None);
        CONFIGURATOR.with(|slot| {
            if let Some(state) = slot.borrow_mut().take().and_then(|state| state.upgrade()) {
                state.wait_for_worker();
            }
        });
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

#[cfg(any(target_os = "linux", target_os = "windows"))]
pub fn show_settings() -> Result<()> {
    slint::invoke_from_event_loop(|| {
        if let Some(configurator) = CONFIGURATOR.with(|slot| {
            slot.borrow().as_ref().and_then(std::rc::Weak::upgrade)
        }) {
            configurator.show();
        }
    }).context("opening the configurator")
}

/// Reuse the resident window without restoring saved geometry or maximizing it.
#[cfg(target_os = "windows")]
pub fn activate_existing() -> Result<()> {
    slint::invoke_from_event_loop(|| {
        let config = CONFIGURATOR.with(|slot| slot.borrow().as_ref().and_then(std::rc::Weak::upgrade));
        if let Some(window) = config.and_then(|state| state.window.upgrade()) {
            if window.window().is_visible() {
                activate_dialog(&window, DialogKind::Configuration);
                return;
            }
        }
        ABOUT.with(|slot| {
            if let Some(about) = slot.borrow().as_ref().and_then(slint::Weak::upgrade) {
                activate_dialog(&about, DialogKind::About);
            }
        });
    })
    .context("activating the existing window")
}

#[cfg(target_os = "windows")]
fn activate_dialog<T: ComponentHandle + 'static>(dialog: &T, kind: DialogKind) {
    dialog.window().set_minimized(false);
    if let Err(error) = dialog.show() {
        eprintln!("Could not show the existing window: {error}");
        return;
    }
    let weak = dialog.as_weak();
    slint::Timer::single_shot(std::time::Duration::ZERO, move || {
        present_native_dialog(weak, kind, false, 32);
    });
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

#[cfg(target_os = "linux")]
struct LinuxPageChoices {
    page: String,
    ids: Vec<Option<String>>,
}

struct SettingsSnapshot {
    config: Config,
    #[cfg(target_os = "linux")]
    installed: Result<Vec<&'static crate::linux_apps::Profile>>,
}

/// Own the confirmed settings and send snapshots to the window.
struct Configurator {
    window: slint::Weak<MainWindow>,
    confirmed: RefCell<Option<Config>>,
    busy: Cell<bool>,
    worker: RefCell<Option<JoinHandle<()>>>,
    #[cfg(target_os = "linux")]
    linux_choices: RefCell<Vec<LinuxPageChoices>>,
    path: PathBuf,
    control: Arc<Control>,
}

impl Configurator {
    /// Load or edit the latest settings away from the event loop.
    fn request(&self, change: Option<SettingsChange>) {
        let Some(window) = self.window.upgrade() else {
            return;
        };

        if self.busy.get() || (change.is_some() && self.confirmed.borrow().is_none()) {
            return;
        }
        self.busy.set(true);
        window.set_ready(false);
        window.set_settings_message(if change.is_some() {
            "Saving settings…"
        } else {
            "Loading settings…"
        }.into());
        let path = self.path.clone();
        let control = Arc::clone(&self.control);
        let weak = self.window.clone();
        let worker = thread::Builder::new().name("ulanzi-settings".into()).spawn(move || {
            let saved = change.map(|change| -> Result<SettingsStore> {
                let _lock = control.lock_settings()?;
                let mut latest = SettingsStore::load(&path)?;
                latest.apply_change(change)?;
                Ok(latest)
            });
            let (loaded, message) = match saved {
                Some(Ok(settings)) => (Ok(settings), None),
                Some(Err(error)) => (
                    SettingsStore::load(&path),
                    Some(format!("Could not save changes: {error:#}")),
                ),
                None => (SettingsStore::load(&path), None),
            };
            let snapshot = loaded.map(|settings| SettingsSnapshot {
                config: settings.config,
                #[cfg(target_os = "linux")]
                installed: crate::linux_apps::installed(),
            });
            let _ = weak.upgrade_in_event_loop(move |_| {
                if let Some(state) = CONFIGURATOR.with(|slot| {
                    slot.borrow().as_ref().and_then(std::rc::Weak::upgrade)
                }) {
                    state.publish(snapshot, message);
                }
            });
        });
        match worker {
            Ok(worker) => *self.worker.borrow_mut() = Some(worker),
            Err(error) => {
                self.busy.set(false);
                window.set_ready(self.confirmed.borrow().is_some());
                window.set_settings_message(format!("Could not load or save settings: {error}").into());
            }
        }
    }

    fn wait_for_worker(&self) {
        if let Some(worker) = self.worker.borrow_mut().take() {
            let _ = worker.join();
        }
    }

    /// Display a confirmed snapshot, or disable editing if it could not be read.
    fn publish(&self, snapshot: Result<SettingsSnapshot>, error: Option<String>) {
        let Some(window) = self.window.upgrade() else {
            return;
        };
        let message = match snapshot {
            Ok(snapshot) => {
                let config = snapshot.config;
                let pages = config
                    .pages
                    .iter()
                    .filter(|page| !page.executables.is_empty())
                    .map(|page| AppPage {
                        name: page.name.clone().into(),
                        enabled: page.enabled,
                    })
                    .collect::<Vec<_>>();
                window.set_app_pages(slint::ModelRc::new(slint::VecModel::from(pages)));
                let theme_index = Theme::ALL
                    .iter()
                    .position(|&theme| theme == config.theme)
                    .expect("every theme is listed in Theme::ALL");
                window.set_theme_index(theme_index as i32);
                window.set_auto_switch(config.runtime.auto_switch);
                window.set_detect_applications(config.runtime.detect_applications);
                window.set_poll_seconds(config.runtime.focus_poll_seconds as i32);
                let message = error.unwrap_or_default();
                #[cfg(target_os = "linux")]
                let message = match snapshot.installed.and_then(|installed| {
                    self.publish_linux_apps(&window, &config, &installed)
                }) {
                    Ok(()) => message,
                    Err(error) => {
                        window.set_linux_app_pages(slint::ModelRc::default());
                        self.linux_choices.borrow_mut().clear();
                        if message.is_empty() {
                            format!("Could not read installed applications: {error:#}")
                        } else {
                            message
                        }
                    }
                };
                *self.confirmed.borrow_mut() = Some(config);
                message
            }
            Err(load_error) => {
                *self.confirmed.borrow_mut() = None;
                error.map_or_else(
                    || format!("Could not load settings: {load_error:#}"),
                    |error| format!("{error}. Could not reload settings: {load_error:#}"),
                )
            }
        };
        self.busy.set(false);
        window.set_settings_message(message.into());
        window.set_ready(self.confirmed.borrow().is_some());
    }

    #[cfg(target_os = "linux")]
    fn publish_linux_apps(
        &self,
        window: &MainWindow,
        config: &Config,
        installed: &[&crate::linux_apps::Profile],
    ) -> Result<()> {
        let mut rows = Vec::new();
        let mut choices = Vec::new();
        for page in &config.pages {
            if !crate::linux_apps::PROFILES.iter().any(|profile| profile.page == page.name) {
                continue;
            }
            let selected = config.linux_applications.get(&page.name).map(|choice| &choice.application);
            let mut labels: Vec<slint::SharedString> = vec!["Use shared app mapping".into()];
            let mut ids = vec![None];
            let mut selected_index = 0;
            let mut available = false;
            for profile in installed.iter().filter(|profile| profile.page == page.name) {
                labels.push(profile.name.into());
                ids.push(Some(profile.id.to_owned()));
                if selected.is_some_and(|id| id == profile.id) {
                    selected_index = (ids.len() - 1) as i32;
                    available = true;
                }
            }
            let status = if let Some(id) = selected {
                if available {
                    "Installed. Suggested shortcuts can be applied to this page.".to_owned()
                } else {
                    let profile = crate::linux_apps::profile(id).context("Unknown Linux application")?;
                    labels.push(format!("{} (not found)", profile.name).into());
                    ids.push(Some(id.clone()));
                    selected_index = (ids.len() - 1) as i32;
                    "The selected app was not found. Your choice and shortcuts are saved.".to_owned()
                }
            } else if labels.len() == 1 {
                "No supported app found for this page.".to_owned()
            } else {
                "Choose an installed app. Current shortcuts will be kept.".to_owned()
            };
            rows.push(LinuxAppPage {
                page: page.name.clone().into(),
                choices: slint::ModelRc::new(slint::VecModel::from(labels)),
                selected: selected_index,
                available,
                status: status.into(),
            });
            choices.push(LinuxPageChoices { page: page.name.clone(), ids });
        }
        window.set_linux_app_pages(slint::ModelRc::new(slint::VecModel::from(rows)));
        *self.linux_choices.borrow_mut() = choices;
        Ok(())
    }

    /// Persist only an explicit app selection or request for suggested shortcuts.
    #[cfg(target_os = "linux")]
    fn edit_linux_application(&self, row: i32, selection: Option<i32>) {
        let choices = self.linux_choices.borrow();
        let Some(choice) = choices.get(row as usize) else {
            return;
        };
        let change = if let Some(index) = selection {
            let Some(id) = choice.ids.get(index as usize) else {
                return;
            };
            SettingsChange::LinuxApplication(choice.page.clone(), id.clone())
        } else {
            let confirmed = self.confirmed.borrow();
            let Some(application) = confirmed.as_ref().and_then(|config| {
                config.linux_applications.get(&choice.page)
            }) else {
                return;
            };
            SettingsChange::LinuxShortcuts {
                page: choice.page.clone(),
                application: application.application.clone(),
            }
        };
        drop(choices);
        self.request(Some(change));
    }

    /// Open or activate configuration; keep edits intact if it is already visible.
    fn show(&self) {
        let Some(window) = self.window.upgrade() else {
            return;
        };

        // Reload when a hidden window is opened again.
        if !window.window().is_visible() {
            self.request(None);
            crate::licensing::refresh();
        }
        if let Err(error) = present_dialog(&window, DialogKind::Configuration) {
            window.set_message(format!("{error:#}").into());
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
    window.set_foreground_supported(cfg!(target_os = "windows"));
    window.set_save_hint(if cfg!(target_os = "windows") {
        "These preferences apply automatically. Restart the controller after editing app mappings or shortcuts."
    } else {
        "Theme and page changes apply automatically. Restart the controller after changing apps or shortcuts."
    }.into());
    #[cfg(target_os = "linux")]
    window.set_linux_app_selection(true);
    let themes = Theme::ALL
        .iter()
        .map(|theme| theme.label().into())
        .collect::<Vec<slint::SharedString>>();
    window.set_themes(slint::ModelRc::new(slint::VecModel::from(themes)));

    let state = Rc::new(Configurator {
        window: window.as_weak(),
        confirmed: RefCell::new(None),
        busy: Cell::new(false),
        worker: RefCell::new(None),
        #[cfg(target_os = "linux")]
        linux_choices: RefCell::new(Vec::new()),
        path,
        control,
    });
    CONFIGURATOR.with(|slot| *slot.borrow_mut() = Some(Rc::downgrade(&state)));

    let toggled = Rc::clone(&state);
    window.on_page_toggled(move |page, enabled| {
        toggled.request(Some(SettingsChange::PageEnabled(page.to_string(), enabled)));
    });
    #[cfg(target_os = "linux")]
    {
        let select = Rc::clone(&state);
        window.on_linux_app_selected(move |row, index| select.edit_linux_application(row, Some(index)));
        let shortcuts = Rc::clone(&state);
        window.on_linux_shortcuts_requested(move |row| shortcuts.edit_linux_application(row, None));
        let refresh = Rc::clone(&state);
        window.on_refresh_linux_apps(move || refresh.request(None));
    }
    let theme = Rc::clone(&state);
    window.on_theme_changed(move |index| {
        if let Some(&selected) = Theme::ALL.get(index as usize) {
            theme.request(Some(SettingsChange::Theme(selected)));
        }
    });
    let detection = Rc::clone(&state);
    window.on_detect_applications_requested(move |enabled| {
        detection.request(Some(SettingsChange::DetectApplications(enabled)));
    });
    let switching = Rc::clone(&state);
    window.on_auto_switch_requested(move |enabled| {
        switching.request(Some(SettingsChange::AutoSwitch(enabled)));
    });
    let interval = Rc::clone(&state);
    window.on_poll_seconds_requested(move |seconds| {
        if let Ok(seconds) = u64::try_from(seconds) {
            interval.request(Some(SettingsChange::FocusPollSeconds(seconds)));
        }
    });
    let open = Rc::clone(&state);
    window.on_open_file(move |catalogue| open.open_file(catalogue));

    let weak = window.as_weak();
    window.on_learn_vibeware(move || {
        let result = crate::tray::open_browser("https://github.com/umbertotechnopreneur/VibeWare");
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
            "website" => "https://umbertogiacobbi.biz",
            "terms" => "https://umbertogiacobbi.biz/terms/",
            "privacy" => "https://umbertogiacobbi.biz/privacy/",
            _ => return,
        };
        let result = crate::tray::open_browser(url);
        if let Some(about) = weak.upgrade() {
            about.set_message(match result {
                Ok(()) => "".into(),
                Err(error) => format!("Could not open the link: {error:#}").into(),
            });
        }
    });
}

/// The close button and title-bar X must have the same lifecycle behavior.
fn bind_close_handlers(about: &AboutDialog, window: &MainWindow, mode: Mode) {
    let about_weak = about.as_weak();
    let config_weak = window.as_weak();
    about.window().on_close_requested(move || {
        if let Some(about) = about_weak.upgrade() {
            placement::remember(about.window(), DialogKind::About);
            let _ = about.hide();
        }
        if mode != Mode::Resident {
            if cfg!(target_os = "linux")
                && config_weak.upgrade().is_some_and(|config| config.window().is_visible())
            {
                return slint::CloseRequestResponse::HideWindow;
            }
            if let Some(config) = config_weak.upgrade() {
                placement::remember(config.window(), DialogKind::Configuration);
                let _ = config.hide();
            }
            let _ = slint::quit_event_loop();
        }
        slint::CloseRequestResponse::HideWindow
    });

    let weak = window.as_weak();
    let about_weak = about.as_weak();
    let close_config: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(config) = weak.upgrade() {
            placement::remember(config.window(), DialogKind::Configuration);
            let _ = config.hide();
        }
        let standalone_without_about = mode != Mode::Resident
            && !about_weak.upgrade().is_some_and(|about| about.window().is_visible());
        if if cfg!(target_os = "linux") { standalone_without_about } else { mode == Mode::Configure } {
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
fn create(path: PathBuf, control: Arc<Control>, mode: Mode, start_controller: bool) -> Result<Dialogs> {
    placement::initialize(&path);
    let about = AboutDialog::new().context("creating the About dialog")?;
    // Slint has initialized its backend, but neither native window is shown yet.
    // Match ulanzideck.desktop so Wayland/X11 shells use the installed app icon.
    #[cfg(target_os = "linux")]
    slint::set_xdg_app_id("ulanzideck").context("setting the Linux desktop application ID")?;
    let config = MainWindow::new().context("creating the configurator")?;
    #[cfg(target_os = "linux")]
    let shutdown_dialog = ShutdownDialog::new().context("creating the shutdown dialog")?;
    #[cfg(target_os = "linux")]
    shutdown::bind(&shutdown_dialog, path.clone());
    #[cfg(target_os = "linux")]
    linux::bind_startup(&config, &path);
    let connection_timer = connection::bind(
        &config,
        &path,
        Arc::clone(&control),
        mode == Mode::Resident,
        start_controller,
    );
    let state = bind_configurator(&config, path, control, mode);
    let store_timer = store_ui::bind(&config);

    about.set_version(env!("CARGO_PKG_VERSION").into());
    bind_about_links(&about);
    bind_close_handlers(&about, &config, mode);

    ABOUT.with(|slot| *slot.borrow_mut() = Some(about.as_weak()));
    if mode == Mode::Configure {
        state.request(None);
    }

    Ok(Dialogs {
        about,
        config,
        _store_timer: store_timer,
        _connection_timer: connection_timer,
        #[cfg(target_os = "linux")]
        _shutdown: shutdown_dialog,
    })
}

// path: configuration file used by the standalone dialog.
// configure: open the configurator directly instead of the About dialog.
// control: application lifecycle shared with the caller.
// Errors: UI creation or event-loop failure. Never discovers or opens HID.
pub fn standalone(
    path: PathBuf,
    configure: bool,
    control: Arc<Control>,
    _start_controller: bool,
) -> Result<()> {
    #[cfg(target_os = "linux")]
    let Some(_instance) = crate::singleton::acquire_configurator()? else {
        let message = "UlanziDeck already has an open window.";
        println!("{message}");
        if let Err(error) = crate::tray::linux::notify_message("UlanziDeck is already open", message) {
            eprintln!("Could not show the duplicate launch notification: {error:#}");
        }
        return Ok(());
    };
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
        _start_controller,
    )?;
    #[cfg(target_os = "linux")]
    let _tray_commands = linux::publish(&path).map_err(|error| {
        eprintln!("Could not receive commands from the tray: {error:#}");
    }).ok();
    let _console = crate::console::detach_if_owned()?;
    control.enter_background();
    eprintln!("Opening UlanziDeck {}: {}", if configure { "configurator" } else { "About dialog" }, path.display());
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
    let dialogs = create(path.clone(), Arc::clone(&control), Mode::Resident, false)?;
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
