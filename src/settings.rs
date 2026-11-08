/* VBWR B
 *
 * Project: UlanziDeck
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Modified with AI: OpenAI Codex; transactional persistence and explicit Linux app choices; 2026-11-02
 * Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
 * Evidence: docs/CODE-REVIEW.md; current working-tree changes
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 *
 * VBWR E */

//! Persist current-format preferences and small session/window sidecars.
//! Settings snapshots reject stale dialogs; `persistence` supplies interprocess
//! locking, backups and atomic replacement. This module preserves unknown JSON
//! metadata and never rewrites the read-only application catalogue on normal saves.

use crate::model::{Config, Theme, PAGE_INDEX_NAME};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::fs;
use ulanzi_dock_vibeware::persistence::{read_optional, Transaction};

/// Saves only the requested preferences, preserving custom keys and metadata.
/// Snapshots prevent a dialog from overwriting edits made since it was opened.
pub struct SettingsStore {
    pub config: Config,
    path: PathBuf,
    json: Option<Vec<u8>>,
    catalogue: Option<Vec<u8>>,
}

/// One explicit change to the latest profile.
pub(crate) enum SettingsChange {
    Theme(Theme),
    DetectApplications(bool),
    AutoSwitch(bool),
    FocusPollSeconds(u64),
    PageEnabled(String, bool),
    #[cfg(target_os = "linux")]
    LinuxApplication(String, Option<String>),
    #[cfg(target_os = "linux")]
    LinuxShortcuts { page: String, application: String },
}

// Session state is kept apart from settings, so page navigation cannot make an
// open Configure dialog stale or overwrite a user's shortcut edits.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionState {
    page: String,
}

/// Derive a sibling session sidecar from the selected `settings` filename.
fn session_path(settings: &Path) -> PathBuf {
    settings.with_extension("session.json")
}

/// Read the remembered regular page beside `settings` without altering preferences.
/// Malformed state is an error; an absent sidecar leaves startup at the home page.
pub fn load_last_page(settings: &Path) -> Result<Option<String>> {
    let path = session_path(settings);
    let Some(bytes) = read_optional(&path)? else {
        return Ok(None);
    };
    let session: SessionState = serde_json::from_slice(&bytes)
        .with_context(|| format!("reading session state {}", path.display()))?;
    ensure!(
        !session.page.trim().is_empty() && session.page != PAGE_INDEX_NAME,
        "session state must name a regular page"
    );
    Ok(Some(session.page))
}

/// Save regular `page` beside `settings`; reject empty names and the temporary index.
/// The shared transaction keeps backup/atomic-write behavior for this small sidecar.
pub fn save_last_page(settings: &Path, page: &str) -> Result<()> {
    ensure!(
        !page.trim().is_empty() && page != PAGE_INDEX_NAME,
        "only a regular page can be remembered"
    );
    let path = session_path(settings);
    let mut bytes = serde_json::to_vec_pretty(&SessionState {
        page: page.to_owned(),
    })?;
    bytes.push(b'\n');
    save_auxiliary_file(&path, bytes)
}

/// Save session or window state with the same backup and atomic replacement as preferences.
pub(crate) fn save_auxiliary_file(path: &Path, bytes: Vec<u8>) -> Result<()> {
    update_auxiliary_file(path, |_| Ok(bytes))
}

/// Hold the cross-process lock around reading, merging and writing one sidecar.
/// `path`: destination; `update`: derives new bytes from the latest optional contents.
/// Closure, lock and I/O failures leave the current file unchanged.
pub(crate) fn update_auxiliary_file(
    path: &Path,
    update: impl FnOnce(Option<&[u8]>) -> Result<Vec<u8>>,
) -> Result<()> {
    let transaction = Transaction::begin(path)?;
    let original = transaction.read()?;
    let bytes = update(original.as_deref())?;
    transaction.replace(original.as_deref(), &bytes)
}

impl SettingsStore {
    /// Load `path` with a stable snapshot under its canonical transaction lock.
    /// Return validation/I/O errors instead of substituting incompatible defaults.
    pub fn load(path: &Path) -> Result<Self> {
        let transaction = Transaction::begin(path)?;
        let path = transaction.path();
        let json = transaction.read()?;
        let catalogue = read_optional(&path.with_file_name("applications.yaml"))?;
        let config = Config::from_documents(path, json.as_deref(), catalogue.as_deref())?;

        let settings = Self {
            config,
            path: path.to_owned(),
            json,
            catalogue,
        };

        settings.check_current()?;
        Ok(settings)
    }

    /// Reject JSON or catalogue changes since this editor snapshot was loaded.
    fn check_current(&self) -> Result<()> {
        let current_json = read_optional(&self.path)?;
        let current_catalogue = read_optional(&self.path.with_file_name("applications.yaml"))?;

        ensure!(
            current_json == self.json && current_catalogue == self.catalogue,
            "Settings changed during saving. Please try again."
        );
        Ok(())
    }

    /// Preserve original JSON extension fields, or serialize defaults for a new file.
    fn document(&self) -> Result<serde_json::Value> {
        match &self.json {
            Some(bytes) => Ok(serde_json::from_slice(bytes)?),
            // The effective config already includes catalogue overrides.
            None => Ok(serde_json::to_value(Config::default())?),
        }
    }

    /// Apply the requested change without copying other values from the editor.
    pub(crate) fn apply_change(&mut self, change: SettingsChange) -> Result<()> {
        let mut runtime = self.config.runtime;
        match change {
            SettingsChange::Theme(theme) => return self.save_theme(theme),
            SettingsChange::DetectApplications(enabled) => runtime.detect_applications = enabled,
            SettingsChange::AutoSwitch(enabled) => runtime.auto_switch = enabled,
            SettingsChange::FocusPollSeconds(seconds) => runtime.focus_poll_seconds = seconds,
            SettingsChange::PageEnabled(page, enabled) => {
                return self.save_page_enabled(&page, enabled);
            }
            #[cfg(target_os = "linux")]
            SettingsChange::LinuxApplication(page, id) => {
                return self.save_linux_application(&page, id.as_deref());
            }
            #[cfg(target_os = "linux")]
            SettingsChange::LinuxShortcuts { page, application } => {
                ensure!(
                    self.config
                        .linux_applications
                        .get(&page)
                        .is_some_and(|choice| choice.application == application),
                    "The selected application changed. Check the selection before applying shortcuts."
                );
                return self.save_linux_shortcuts(&page);
            }
        }
        self.save_field("runtime", serde_json::to_value(runtime)?)
    }

    /// Back up and atomically replace one file. The application catalogue is read-only here.
    fn save_document(&mut self, document: serde_json::Value) -> Result<()> {
        let transaction = Transaction::begin(&self.path)?;
        self.check_current()?;

        let mut bytes = serde_json::to_vec_pretty(&document)?;
        bytes.push(b'\n');
        let config = Config::from_documents(&self.path, Some(&bytes), self.catalogue.as_deref())?;

        self.check_current()?;
        transaction.replace(self.json.as_deref(), &bytes)?;

        self.config = config;
        self.json = Some(bytes);
        Ok(())
    }

    /// Replace top-level `name` with `value`, retaining all unrelated JSON content.
    /// Full validation and stale-file checks still run through `save_document`.
    fn save_field(&mut self, name: &str, value: serde_json::Value) -> Result<()> {
        let mut document = self.document()?;
        document[name] = value;

        self.save_document(document)
    }

    /// Save `level` in the inclusive 0–100 range; this method does not write to HID.
    pub fn save_brightness(&mut self, level: u8) -> Result<()> {
        ensure!(level <= 100, "brightness must be 0-100");
        self.save_field("brightness", level.into())
    }

    /// Persist explicitly selected `theme` only after the shared licensing policy allows it.
    pub fn save_theme(&mut self, theme: Theme) -> Result<()> {
        crate::licensing::require(theme)?;
        self.save_field("theme", serde_json::to_value(theme)?)
    }

    /// Persist completion before the caller marks the in-memory setup as finished.
    pub fn save_setup_completed(&mut self) -> Result<()> {
        self.save_field("setup_completed", true.into())
    }

    /// Mark the graphical welcome as shown only after its native window is presented.
    pub fn save_first_run_completed(&mut self) -> Result<()> {
        self.save_field("is_first_run", false.into())
    }

    /// Select an installed app while keeping the effective keys in a Linux-only profile.
    /// Existing Linux key metadata and all shared Windows pages remain unchanged.
    #[cfg(target_os = "linux")]
    pub fn save_linux_application(&mut self, page: &str, id: Option<&str>) -> Result<()> {
        let mut document = self.document()?;
        if document.get("linux_applications").is_none() {
            document["linux_applications"] = serde_json::json!({});
        }
        let choices = document["linux_applications"].as_object_mut()
            .context("Linux applications must be an object")?;
        if let Some(id) = id {
            let profile = crate::linux_apps::profile(id).context("Unknown Linux application")?;
            ensure!(profile.page == page, "Application does not belong to this page");
            ensure!(crate::linux_apps::installed()?.iter().any(|entry| entry.id == id),
                "This application is no longer installed. Refresh the list.");
            let effective = self.config.pages.iter().find(|entry| entry.name == page)
                .context("Application page is missing")?;
            let selection = choices.entry(page.to_owned()).or_insert_with(||
                serde_json::json!({ "keys": effective.keys }));
            selection["application"] = id.into();
        } else {
            // Returning to the shared catalogue is an explicit separate choice.
            choices.remove(page);
        }
        self.save_document(document)
    }

    /// Replace only the selected Linux profile's action keys after an explicit UI request.
    /// Navigation, artwork, enabled flags, shared pages and extension fields survive.
    #[cfg(target_os = "linux")]
    pub fn save_linux_shortcuts(&mut self, page: &str) -> Result<()> {
        let selection = self.config.linux_applications.get(page).context("Choose an application first")?;
        let id = &selection.application;
        ensure!(crate::linux_apps::installed()?.iter().any(|entry| entry.id == id),
            "This application is no longer installed. Refresh the list.");
        let shortcuts = crate::linux_apps::shortcuts::for_profile(id);
        ensure!(!shortcuts.is_empty(), "No suggested shortcuts for this application");
        let mut document = self.document()?;
        let keys = document["linux_applications"][page]["keys"].as_array_mut()
            .context("Selected Linux application needs keys")?;
        for shortcut in shortcuts {
            ensure!(shortcut.key < keys.len() && shortcut.key != crate::model::NEXT_KEY
                && shortcut.key != crate::model::CLOCK_KEY, "Invalid suggested shortcut position");
            let key = &mut keys[shortcut.key];
            key["label"] = shortcut.label.into();
            key["action"] = shortcut.action.into();
            key["description"] = shortcut.description.into();
        }
        self.save_document(document)
    }

    /// Change one application page's enabled flag while preserving its keys.
    fn save_page_enabled(&mut self, name: &str, enabled: bool) -> Result<()> {
        ensure!(
            self.config
                .pages
                .iter()
                .any(|page| page.name == name && !page.executables.is_empty()),
            "Unknown application page: {name}"
        );
        let mut document = self.document()?;
        let page = document["pages"]
            .as_array_mut()
            .context("settings.json must contain a pages array")?
            .iter_mut()
            .find(|page| page["name"].as_str() == Some(name))
            .context("application page missing from settings")?;
        page["enabled"] = enabled.into();
        self.save_document(document)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct Fixture(PathBuf);

    impl Fixture {
        fn create() -> Result<Self> {
            let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ulanzi-settings-{}-{stamp}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            #[cfg(target_os = "linux")]
            {
                use std::os::unix::fs::DirBuilderExt;
                fs::DirBuilder::new().mode(0o700).create(&path)?;
            }
            #[cfg(not(target_os = "linux"))]
            fs::create_dir(&path)?;
            let fixture = Self(path);

            let mut document = serde_json::to_value(Config::default())?;
            document["pages"][0]["keys"][0]["action"] = "hotkey:ctrl+shift+f12".into();
            document["owner_extension"] = serde_json::json!({"keep": "custom metadata"});
            fs::write(
                fixture.0.join("settings.json"),
                serde_json::to_vec(&document)?,
            )?;
            fs::write(
                fixture.0.join("applications.yaml"),
                include_str!("../applications.yaml"),
            )?;

            Ok(fixture)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            // Remove only this fixture's named files, never recurse over a shared directory.
            for name in [
                "settings.json",
                "settings.session.json",
                "settings.session.json.gui.bak",
                "applications.yaml",
                "settings.json.gui.bak",
                "settings.json.lock",
                "settings.session.json.lock",
            ] {
                let _ = fs::remove_file(self.0.join(name));
            }
            let temporary = format!("settings.gui-{}.tmp", std::process::id());
            let _ = fs::remove_file(self.0.join(temporary));
            let temporary = format!("settings.session.gui-{}.tmp", std::process::id());
            let _ = fs::remove_file(self.0.join(temporary));
            let _ = fs::remove_dir(&self.0);
        }
    }

    #[test]
    fn remembering_pages_keeps_settings_and_open_dialogs_independent() -> Result<()> {
        let fixture = Fixture::create()?;
        let path = fixture.0.join("settings.json");
        let original = fs::read(&path)?;
        let mut dialog = SettingsStore::load(&path)?;
        assert_eq!(load_last_page(&path)?, None);
        save_last_page(&path, "Spotify")?;
        assert_eq!(load_last_page(&path)?.as_deref(), Some("Spotify"));
        assert_eq!(fs::read(&path)?, original);

        save_last_page(&path, "VS Code")?;
        let previous: SessionState =
            serde_json::from_slice(&fs::read(fixture.0.join("settings.session.json.gui.bak"))?)?;
        assert_eq!(previous.page, "Spotify");
        dialog.save_theme(Theme::Memphis)?;
        assert_eq!(load_last_page(&path)?.as_deref(), Some("VS Code"));
        assert_eq!(Config::load(&path)?.theme, Theme::Memphis);
        assert!(save_last_page(&path, PAGE_INDEX_NAME).is_err());
        assert_eq!(load_last_page(&path)?.as_deref(), Some("VS Code"));
        Ok(())
    }

    #[test]
    fn save_changes_preferences_without_rewriting_the_catalogue() -> Result<()> {
        let fixture = Fixture::create()?;
        let path = fixture.0.join("settings.json");
        let original_json = fs::read(&path)?;
        let catalogue = fs::read(path.with_file_name("applications.yaml"))?;
        let mut settings = SettingsStore::load(&path)?;

        settings.apply_change(SettingsChange::Theme(Theme::LightClassic))?;
        assert_eq!(
            fs::read(fixture.0.join("settings.json.gui.bak"))?,
            original_json
        );
        for change in [
            SettingsChange::DetectApplications(false),
            SettingsChange::AutoSwitch(false),
            SettingsChange::FocusPollSeconds(7),
            SettingsChange::PageEnabled("Word".into(), false),
            SettingsChange::PageEnabled("Codex".into(), false),
        ] {
            settings.apply_change(change)?;
        }

        let original: serde_json::Value = serde_json::from_slice(&original_json)?;
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        assert_eq!(saved["owner_extension"], original["owner_extension"]);
        assert_eq!(saved["pages"][0]["keys"], original["pages"][0]["keys"]);
        assert_eq!(
            fs::read(path.with_file_name("applications.yaml"))?,
            catalogue
        );
        let loaded = Config::load(&path)?;
        assert_eq!(loaded.theme, Theme::LightClassic);
        assert!(!loaded.runtime.auto_switch);
        assert!(!loaded.runtime.detect_applications);
        assert_eq!(loaded.runtime.focus_poll_seconds, 7);
        assert!(!loaded.pages[2].enabled);
        assert!(!loaded.pages[5].enabled);
        assert_eq!(loaded.next_enabled_page(1), 3);
        assert_eq!(loaded.next_enabled_page(4), 6);
        Ok(())
    }

    #[test]
    fn first_save_does_not_copy_catalogue_overrides_into_settings() -> Result<()> {
        let fixture = Fixture::create()?;
        let path = fixture.0.join("settings.json");
        fs::remove_file(&path)?;
        let catalogue = include_str!("../applications.yaml").replacen(
            "action: \"hotkey:ctrl+n\"",
            "action: \"hotkey:ctrl+shift+f12\"",
            1,
        );
        fs::write(path.with_file_name("applications.yaml"), catalogue)?;

        let mut settings = SettingsStore::load(&path)?;
        assert_eq!(settings.config.pages[2].keys[0].action, "hotkey:ctrl+shift+f12");
        settings.apply_change(SettingsChange::PageEnabled("Word".into(), false))?;

        let saved: Config = serde_json::from_slice(&fs::read(&path)?)?;
        assert_eq!(saved.pages[2].keys[0].action, "hotkey:ctrl+n");
        assert!(!saved.pages[5].enabled);
        Ok(())
    }

    #[test]
    fn linux_keys_survive_preference_saves_without_replacing_shared_keys() -> Result<()> {
        let fixture = Fixture::create()?;
        let path = fixture.0.join("settings.json");
        let mut document: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        let shared_keys = document["pages"][5]["keys"].clone();
        let mut linux_keys = shared_keys.clone();
        linux_keys[12]["action"] = "hotkey:ctrl+shift+s".into();
        linux_keys[12]["label"] = "WRITER SAVE AS".into();
        linux_keys[12]["owner_key_metadata"] = "keep".into();
        document["linux_applications"] = serde_json::json!({
            "Word": {
                "application": "libreoffice-writer",
                "keys": linux_keys,
                "owner_profile_metadata": "keep"
            }
        });
        fs::write(&path, serde_json::to_vec(&document)?)?;
        let original = fs::read(&path)?;
        let catalogue = fs::read(path.with_file_name("applications.yaml"))?;
        let mut settings = SettingsStore::load(&path)?;
        let effective = &settings.config.pages[5].keys[12];
        if cfg!(target_os = "linux") {
            assert_eq!(effective.action, "hotkey:ctrl+shift+s");
            assert_eq!(effective.label, "WRITER SAVE AS");
        } else {
            assert_eq!(effective.action, "hotkey:f12");
        }
        settings.apply_change(SettingsChange::PageEnabled("Word".into(), false))?;
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        assert_eq!(saved["pages"][5]["keys"], shared_keys);
        assert_eq!(saved["linux_applications"], document["linux_applications"]);
        assert!(!Config::load(&path)?.pages[5].enabled);
        assert_eq!(fs::read(path.with_file_name("applications.yaml"))?, catalogue);
        assert_eq!(fs::read(fixture.0.join("settings.json.gui.bak"))?, original);
        Ok(())
    }

    #[test]
    fn setup_completion_is_persisted_without_changing_the_theme() -> Result<()> {
        let fixture = Fixture::create()?;
        let path = fixture.0.join("settings.json");
        let mut settings = SettingsStore::load(&path)?;
        let theme = settings.config.theme;

        assert!(!settings.config.setup_completed);
        settings.save_setup_completed()?;

        let saved = Config::load(&path)?;
        assert!(saved.setup_completed);
        assert_eq!(saved.theme, theme);
        Ok(())
    }

    #[test]
    fn stale_dialog_refuses_to_overwrite_external_changes() -> Result<()> {
        let fixture = Fixture::create()?;
        let path = fixture.0.join("settings.json");
        let mut settings = SettingsStore::load(&path)?;
        let mut external = fs::read(&path)?;
        external.push(b'\n');
        fs::write(&path, &external)?;

        assert!(settings.apply_change(SettingsChange::Theme(Theme::Memphis)).is_err());
        assert_eq!(fs::read(&path)?, external);
        Ok(())
    }

    #[test]
    fn stale_temporary_file_does_not_block_saving_or_get_deleted() -> Result<()> {
        let fixture = Fixture::create()?;
        let path = fixture.0.join("settings.json");
        let mut settings = SettingsStore::load(&path)?;
        let temporary = fixture
            .0
            .join(format!("settings.gui-{}.tmp", std::process::id()));
        fs::write(&temporary, b"keep existing staging data")?;

        settings.apply_change(SettingsChange::Theme(Theme::Memphis))?;
        assert_eq!(Config::load(&path)?.theme, Theme::Memphis);
        assert_eq!(fs::read(&temporary)?, b"keep existing staging data");
        Ok(())
    }
}
