/* VBWR B
 *
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Modified with AI: OpenAI Codex; shared entitlement checks and readable controller contracts and readability; 2026-10-27
 * Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
 * Evidence: docs/CODE-REVIEW.md; current working-tree changes
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 *
 * VBWR E */

//! Own the dock event loop: connection recovery, page navigation, input release
//! pairing, redraws, and settings polling. All HID writes stay on this worker.
//! UI and Store calls are queued elsewhere; this module never displays purchase UI
//! synchronously or reads global keyboard input/window titles.

use crate::{
    actions::{self, ForegroundProcess},
    dock::{self, ButtonEvent, Dock},
    model::{Config, PageConfig, Theme, CLOCK_KEY, NEXT_KEY, PAGE_INDEX_NAME},
    settings::{self, SettingsStore},
    tray::{ConnectionState, Control},
};
use anyhow::{Context, Result};
use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

/// Named options keep command-line switches out of the device event loop.
pub struct RunOptions {
    pub config_path: PathBuf,
    pub theme: Theme,
    pub seconds: Option<u64>,
    pub no_actions: bool,
    pub clock: bool,
    pub status_display: bool,
    pub focus_poll_seconds: u64,
    pub auto_page: bool,
    pub detect_applications: bool,
}

/// State that survives USB disconnections. A held key never survives reconnection.
struct Controller {
    config: Config,
    options: RunOptions,
    control: Arc<Control>,
    theme: Theme,
    current_page: usize,
    held_key: Option<usize>,
    page_is_active: bool,
    last_known_process: Option<ForegroundProcess>,
    started: Instant,
    boot_shown: bool,
    last_saved_page: Option<String>,
    pending_page: Option<String>,
    page_changed_at: Instant,
}

/// Run validated `config` using resolved CLI `options` until `control` requests exit.
/// Restores the last enabled page, applies licensing, and propagates fatal I/O errors.
pub fn run(config: Config, options: RunOptions, control: Arc<Control>) -> Result<()> {
    let foreground = foreground_process(options.detect_applications);
    let last_saved_page = settings::load_last_page(&options.config_path)?;
    let current_page = restored_page(&config, last_saved_page.as_deref());

    let page_is_active = actions::page_available(&config.pages[current_page], &foreground);
    let last_known_process =
        matches!(foreground, ForegroundProcess::Known(_)).then(|| foreground.clone());

    Controller {
        theme: crate::licensing::effective(options.theme),
        config,
        options,
        control,
        current_page,
        held_key: None,
        page_is_active,
        last_known_process,
        started: Instant::now(),
        boot_shown: false,
        last_saved_page,
        pending_page: None,
        page_changed_at: Instant::now(),
    }
    .run()
}

// The page index is temporary. A disabled or removed saved page cannot be opened.
/// Resolve saved `name` within `config`; missing/disabled/index pages use home.
fn restored_page(config: &Config, name: Option<&str>) -> usize {
    config
        .pages
        .iter()
        .position(|page| {
            page.enabled && page.name != PAGE_INDEX_NAME && Some(page.name.as_str()) == name
        })
        .unwrap_or_else(|| config.home_page())
}

impl Drop for Controller {
    fn drop(&mut self) {
        self.save_pending_page(true);
    }
}

/// Query the foreground executable only when `detect_applications` is enabled.
fn foreground_process(detect_applications: bool) -> ForegroundProcess {
    if detect_applications {
        actions::foreground_process()
    } else {
        ForegroundProcess::Unsupported
    }
}

/// Disabled application pages must still allow NEXT PAGE.
/// Check physical `index` against `page` availability; `page_is_active` does not
/// disable NEXT PAGE. The wide display is handled separately before this check.
fn can_press_key(page: &PageConfig, index: usize, page_is_active: bool) -> bool {
    let is_next_page = index == NEXT_KEY;
    let page_allows_press = page_is_active || is_next_page;
    let key_is_available = page.key_available(index);

    page_allows_press && key_is_available
}

impl Controller {
    /// Borrow the current page from the controller's validated page collection.
    fn page(&self) -> &PageConfig {
        &self.config.pages[self.current_page]
    }

    /// Combine the user exit request with the optional command-line duration limit.
    fn is_running(&self) -> bool {
        let within_deadline = self
            .options
            .seconds
            .is_none_or(|limit| self.started.elapsed() < Duration::from_secs(limit));

        self.control.is_running() && within_deadline
    }

    /// The page index always replaces the wide-key clock with navigation context.
    fn uses_status_display(&self) -> bool {
        self.options.status_display || self.page().name == PAGE_INDEX_NAME
    }

    /// Read focus using the resolved detection setting, without changing page state.
    fn foreground(&self) -> ForegroundProcess {
        foreground_process(self.options.detect_applications)
    }

    /// Own connection sessions until shutdown. Recover only recognized connection
    /// failures on Windows; configuration/protocol errors are returned to the caller.
    fn run(mut self) -> Result<()> {
        self.config.refresh_theme_selector(self.theme)?;
        let stop = Arc::clone(&self.control);
        ctrlc::set_handler(move || stop.stop())?;

        while self.is_running() {
            let Some(dock) = self.connect()? else {
                break;
            };
            let session = self.session(&dock);

            // Release the failed handle before discovery opens the next USB session.
            drop(dock);
            if let Err(error) = session {
                if !dock::is_connection_error(&error) {
                    return Err(error);
                }
                #[cfg(not(target_os = "windows"))]
                return Err(error);
                #[cfg(target_os = "windows")]
                {
                    self.held_key = None;
                    self.control.set_connection(ConnectionState::Disconnected);
                    eprintln!("D200H disconnected; waiting to reconnect: {error:#}");
                    self.wait_for_connection_retry();
                }
            }
        }

        println!("Stopped. The dock may clear its display when the program exits.");
        Ok(())
    }

    /// Retry a missing Windows dock while the app remains running. `None` means
    /// shutdown was requested before a connection, not that initialization succeeded.
    fn connect(&mut self) -> Result<Option<Dock>> {
        #[cfg(target_os = "windows")]
        let mut last_error = String::new();

        while self.is_running() {
            match self.connect_once() {
                Ok(dock) => {
                    self.control.set_connection(ConnectionState::Connected);
                    return Ok(Some(dock));
                }
                Err(error) => {
                    if !dock::is_connection_error(&error) {
                        return Err(error);
                    }
                    #[cfg(not(target_os = "windows"))]
                    return Err(error);
                    #[cfg(target_os = "windows")]
                    {
                        let message = format!("{error:#}");
                        if message != last_error {
                            eprintln!("Waiting for D200H: {message}");
                            last_error = message;
                        }
                        self.wait_for_connection_retry();
                    }
                }
            }
        }

        Ok(None)
    }

    /// Open the validated consumer interface, refresh persisted brightness and
    /// entitlement, draw boot/current state, then discard stale input reports.
    fn connect_once(&mut self) -> Result<Dock> {
        self.theme = crate::licensing::effective(self.options.theme);
        self.config.refresh_theme_selector(self.theme)?;
        let (api, _) = dock::discover()?;
        let dock = Dock::open(&api)?;
        self.config.brightness = Config::load_brightness(&self.options.config_path)?;
        self.config.refresh_key_availability();

        let foreground = self.foreground();
        if matches!(foreground, ForegroundProcess::Known(_)) {
            // The first connection restores the previous page. App following
            // resumes on subsequent foreground changes, not over the restored view.
            if self.boot_shown {
                self.follow_foreground(&foreground);
            }
            self.last_known_process = Some(foreground.clone());
        }
        self.page_is_active = actions::page_available(self.page(), &foreground);

        if !self.boot_shown {
            dock.show_boot(
                self.config.brightness,
                || self.is_running(),
                || {
                    dock.prepare_transition(
                        self.page(),
                        self.theme,
                        self.page_is_active,
                        self.options.status_display,
                        || self.is_running(),
                    )
                },
            )?;
            self.boot_shown = true;
        } else {
            dock.set_brightness(self.config.brightness)?;
        }
        if self.is_running() {
            self.show_page(&dock)?;
        }
        // Never replay input buffered before the page was ready.
        dock.discard_input()?;
        Ok(dock)
    }

    #[cfg(target_os = "windows")]
    /// Back off for at most two seconds, checking shutdown every 100 ms.
    fn wait_for_connection_retry(&self) {
        for _ in 0..20 {
            if !self.is_running() {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }

    /// Draw current state through `dock` and queue the regular page for persistence.
    /// A generated page index is transient and must never become the startup page.
    fn show_page(&mut self, dock: &Dock) -> Result<()> {
        dock.show_transition(
            self.page(),
            self.theme,
            self.page_is_active,
            self.options.status_display,
            || self.is_running(),
        )?;
        if self.is_running() && self.options.clock && !self.uses_status_display() {
            dock.clock()?;
        }
        if self.is_running() && self.page().name != PAGE_INDEX_NAME {
            let name = self.page().name.clone();
            self.pending_page =
                (self.last_saved_page.as_deref() != Some(name.as_str())).then_some(name);
            self.page_changed_at = Instant::now();
        }
        Ok(())
    }

    /// Debounce navigation writes; `force` flushes at shutdown. Saving errors are
    /// reported without turning a functioning HID connection into a device failure.
    fn save_pending_page(&mut self, force: bool) {
        if !force && self.page_changed_at.elapsed() < Duration::from_millis(500) {
            return;
        }
        let Some(page) = self.pending_page.take() else {
            return;
        };
        let result = self
            .control
            .lock_settings()
            .and_then(|_lock| settings::save_last_page(&self.options.config_path, &page));
        match result {
            Ok(()) => self.last_saved_page = Some(page),
            Err(error) => eprintln!("Could not remember the current page: {error:#}"),
        }
    }

    /// Process input on connected `dock` before lower-priority polling/redraw work.
    /// Poll license snapshots without network waits; connection errors bubble up
    /// to recovery. Only a normal exit attempts the shutdown fade.
    fn session(&mut self, dock: &Dock) -> Result<()> {
        let mut last_clock = Instant::now();
        let mut last_focus = Instant::now();
        let mut last_brightness = Instant::now();
        let mut brightness_error = String::new();

        println!(
            "D200H active | page: {} | theme: {} | Ctrl+C to stop",
            self.page().name,
            self.theme.slug()
        );
        println!(
            "Foreground polling: {}s | automatic pages: {}",
            self.options.focus_poll_seconds,
            if self.options.auto_page { "on" } else { "off" }
        );

        while self.is_running() {
            // Handle queued presses/releases before disk polling, clock writes or
            // focus redraws. read_timeout returns immediately when input is ready.
            if let Some(event) = dock.read_button(Duration::ZERO)? {
                if self.handle_button(dock, event)? {
                    last_focus = Instant::now();
                }
                continue;
            }
            if last_brightness.elapsed() >= Duration::from_millis(500) {
                let licensed_theme = crate::licensing::effective(self.options.theme);
                if licensed_theme != self.theme {
                    self.held_key = None;
                    self.theme = licensed_theme;
                    self.config.refresh_theme_selector(self.theme)?;
                    self.show_page(dock)?;
                }
                // Keep disk writes away from a burst of navigation button events.
                self.save_pending_page(false);
                match Config::load_brightness(&self.options.config_path) {
                    Ok(level) => {
                        brightness_error.clear();
                        if level != self.config.brightness {
                            dock.set_brightness(level)?;
                            self.apply_brightness(dock, level)?;
                        }
                    }
                    Err(error) => {
                        let message = format!("{error:#}");
                        if message != brightness_error {
                            eprintln!("Keeping current brightness: {message}");
                            brightness_error = message;
                        }
                    }
                }
                last_brightness = Instant::now();
            }

            let focus_check_due =
                last_focus.elapsed() >= Duration::from_secs(self.options.focus_poll_seconds);
            if self.options.detect_applications && focus_check_due {
                self.refresh_focus(dock)?;
                last_focus = Instant::now();
            }

            if (self.options.clock || self.uses_status_display())
                && last_clock.elapsed() >= Duration::from_secs(1)
            {
                if self.uses_status_display() {
                    dock.background()?;
                } else {
                    dock.clock()?;
                }
                last_clock = Instant::now();
            }

            if let Some(event) = dock.read_button(Duration::from_millis(100))? {
                let page_changed = self.handle_button(dock, event)?;
                if page_changed {
                    last_focus = Instant::now();
                }
            }
        }

        // Only a normal exit reaches this point. Connection and controller
        // errors return earlier, so they do not pretend to be graceful stops.
        if let Err(error) = dock.fade_out(self.config.brightness) {
            eprintln!("Could not fade the dock during shutdown: {error:#}");
        }

        Ok(())
    }

    /// Select a page for `foreground` when automatic switching is enabled.
    /// Returns whether navigation changed; leaves the page index undisturbed and
    /// clears held input so a release cannot execute an action on a different page.
    fn follow_foreground(&mut self, foreground: &ForegroundProcess) -> bool {
        if !self.options.auto_page || self.page().name == PAGE_INDEX_NAME {
            return false;
        }
        let next_page = actions::matching_page(&self.config, foreground)
            .unwrap_or_else(|| self.config.home_page());
        if next_page == self.current_page {
            return false;
        }

        // A release from the old page must not execute an action from the new page.
        self.held_key = None;
        self.current_page = next_page;
        self.page_is_active = actions::page_available(self.page(), foreground);
        true
    }

    /// Detect executable changes and redraw `dock` only when page/availability changes.
    fn refresh_focus(&mut self, dock: &Dock) -> Result<()> {
        let foreground = self.foreground();
        let known_process_changed = matches!(foreground, ForegroundProcess::Known(_))
            && self.last_known_process.as_ref() != Some(&foreground);

        if known_process_changed {
            if self.follow_foreground(&foreground) {
                self.show_page(dock)?;
                println!("Automatic page: {}", self.page().name);
            }
            // Unknown focus never resets a manual page choice.
            self.last_known_process = Some(foreground.clone());
        }
        self.refresh_availability(dock, &foreground)
    }

    /// Reconcile current page availability against `foreground` on `dock`.
    /// A transition to inactive cancels held actions but preserves navigation input.
    fn refresh_availability(&mut self, dock: &Dock, foreground: &ForegroundProcess) -> Result<()> {
        let active = actions::page_available(self.page(), foreground);
        if active == self.page_is_active {
            return Ok(());
        }

        let navigation_is_held = matches!(self.held_key, Some(NEXT_KEY) | Some(CLOCK_KEY));
        if !active && !navigation_is_held {
            self.held_key = None;
        }
        dock.refresh_availability(
            self.page(),
            self.theme,
            self.held_key,
            active,
            self.options.status_display,
        )?;
        self.page_is_active = active;
        println!(
            "Page availability: {} | {}",
            self.page().name,
            if active { "active" } else { "inactive" }
        );
        Ok(())
    }

    /// Update the model and redraw boundary keys after `level` was sent to `dock`.
    /// Cancel a held +/- key if the new 0/100 bound makes that action unavailable.
    fn apply_brightness(&mut self, dock: &Dock, level: u8) -> Result<()> {
        self.config.brightness = level;
        self.config.refresh_key_availability();
        if self
            .held_key
            .is_some_and(|index| !self.page().key_available(index))
        {
            self.held_key = None;
        }
        dock.refresh_brightness_keys(self.page(), self.theme, self.held_key, self.page_is_active)?;
        println!("Brightness USB write completed: {level}%");
        Ok(())
    }

    /// Read the latest saved level, move ten points in the `increase` direction,
    /// persist it, and write to `dock`. Propagate USB failures; report save conflicts.
    fn adjust_brightness(&mut self, dock: &Dock, increase: bool) -> Result<()> {
        let saved = (|| -> Result<u8> {
            let _lock = self.control.lock_settings()?;
            let mut settings = SettingsStore::load(&self.options.config_path)?;
            let current = settings.config.brightness;
            let level = if increase {
                current.saturating_add(10).min(100)
            } else {
                current.saturating_sub(10)
            };
            if level != current {
                settings.save_brightness(level)?;
            }
            dock.set_brightness(level)?;
            Ok(level)
        })();

        match saved {
            Ok(level) => self.apply_brightness(dock, level),
            Err(error) if dock::is_connection_error(&error) => Err(error),
            Err(error) => {
                eprintln!("Could not change brightness: {error:#}");
                Ok(())
            }
        }
    }

    /// Execute only a release matching an accepted press on the current page.
    /// `event` is a validated consumer-interface report; `dock` receives visual
    /// feedback. Return true for a page transition. Locked themes open configuration
    /// only; purchasing requires a separate explicit click in the desktop UI.
    fn handle_button(&mut self, dock: &Dock, event: ButtonEvent) -> Result<bool> {
        if event.index == CLOCK_KEY {
            if event.pressed {
                self.held_key = Some(CLOCK_KEY);
            } else if self.held_key == Some(CLOCK_KEY) {
                self.held_key = None;
                self.current_page = self.config.page_index_position();
                self.page_is_active = true;
                self.show_page(dock)?;
                println!("Page: {}", self.page().name);
                return Ok(true);
            }
            return Ok(false);
        }

        self.refresh_availability(dock, &self.foreground())?;
        if event.pressed {
            if can_press_key(self.page(), event.index, self.page_is_active)
                && self.held_key != Some(event.index)
            {
                self.held_key = Some(event.index);
                dock.show_key(
                    self.page(),
                    self.theme,
                    event.index,
                    true,
                    self.page_is_active,
                )?;
            }
            return Ok(false);
        }

        if self.held_key != Some(event.index) {
            return Ok(false);
        }
        self.held_key = None;
        if !self.page().key_available(event.index) {
            return Ok(false);
        }

        let action = self.page().keys[event.index].action.clone();
        let mut page_changed = false;
        if event.index == NEXT_KEY {
            self.current_page = self.config.next_enabled_page(self.current_page);
            page_changed = true;
        } else if let Some(target) = action.strip_prefix("page:go:") {
            if let Some(destination) = self
                .config
                .pages
                .iter()
                .position(|page| page.enabled && page.name == target)
            {
                self.current_page = destination;
                page_changed = true;
            }
        } else if let Some(slug) = action.strip_prefix("theme:") {
            let selected = Theme::from_slug(slug)
                .with_context(|| format!("unknown selected theme '{slug}'"))?;
            if !crate::licensing::allows(selected) {
                // A dock button only opens the theme manager; it never buys anything.
                let _ = crate::gui::show_theme_pack();
                dock.show_key(
                    self.page(),
                    self.theme,
                    event.index,
                    false,
                    self.page_is_active,
                )?;
                return Ok(false);
            }
            self.options.theme = selected;
            self.theme = selected;
            self.config.theme = selected;
            self.config.refresh_theme_selector(selected)?;
            self.current_page = self.config.home_page();
            page_changed = true;
            println!("Theme: {}", selected.slug());

            if let Err(error) = self.control.lock_settings().and_then(|_lock| {
                SettingsStore::load(&self.options.config_path)?.save_theme(selected)
            }) {
                eprintln!("Could not save theme selection: {error:#}");
            }
        } else if !self.options.no_actions && self.page_is_active {
            if matches!(action.as_str(), "brightness:up" | "brightness:down") {
                self.adjust_brightness(dock, action == "brightness:up")?;
            } else if actions::page_available(self.page(), &self.foreground()) {
                if let Err(error) = actions::execute(&action) {
                    eprintln!(
                        "Action '{}' failed: {error:#}",
                        self.page().keys[event.index].label
                    );
                }
            }
        }

        if page_changed {
            self.page_is_active = actions::page_available(self.page(), &self.foreground());
            self.show_page(dock)?;
            println!("Page: {}", self.page().name);
        } else {
            dock.show_key(
                self.page(),
                self.theme,
                event.index,
                false,
                self.page_is_active,
            )?;
        }
        Ok(page_changed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_restores_only_an_enabled_regular_page() {
        let mut config = Config::default();
        let spotify = config
            .pages
            .iter()
            .position(|page| page.name == "Spotify")
            .unwrap();
        assert_eq!(restored_page(&config, Some("Spotify")), spotify);
        config.pages[spotify].enabled = false;
        assert_eq!(restored_page(&config, Some("Spotify")), config.home_page());
        assert_eq!(
            restored_page(&config, Some(PAGE_INDEX_NAME)),
            config.home_page()
        );
        assert_eq!(
            restored_page(&config, Some("Removed custom page")),
            config.home_page()
        );
        assert_eq!(restored_page(&config, None), config.home_page());
    }
}
