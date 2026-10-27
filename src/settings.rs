/* VBWR B
 *
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Modified with AI: OpenAI Codex; transactional persistence and theme entitlement checks and readability; 2026-10-27
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

use crate::model::{Config, RuntimeSettings, Theme, PAGE_INDEX_NAME};
use anyhow::{ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
use ulanzi_dock_vibeware::persistence::Transaction;

/// Saves only the requested preferences, preserving custom keys and metadata.
/// Snapshots prevent a dialog from overwriting edits made since it was opened.
pub struct SettingsStore {
    pub config: Config,
    path: PathBuf,
    json: Option<Vec<u8>>,
    catalogue: Option<Vec<u8>>,
}

/// Read `path`; absence is `None`, while permission and other I/O failures propagate.
fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("reading {}", path.display())),
    }
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
        Self::load_unlocked(transaction.path())
    }

    /// Load `path` while the caller already owns its transaction (no nested lock).
    fn load_unlocked(path: &Path) -> Result<Self> {
        let json = read_optional(path)?;
        let catalogue = read_optional(&path.with_file_name("applications.yaml"))?;
        let config = Config::load(path)?;

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
            "Settings changed outside this dialog. Click Reload before saving."
        );
        Ok(())
    }

    /// Preserve original JSON extension fields, or serialize defaults for a new file.
    fn document(&self) -> Result<serde_json::Value> {
        match &self.json {
            Some(bytes) => Ok(serde_json::from_slice(bytes)?),
            None => Ok(serde_json::to_value(&self.config)?),
        }
    }

    /// Back up and atomically replace one file. The application catalogue is read-only here.
    fn save_document(&mut self, document: serde_json::Value) -> Result<()> {
        let transaction = Transaction::begin(&self.path)?;
        self.check_current()?;

        let config: Config = serde_json::from_value(document.clone())?;
        config.validate()?;

        let mut bytes = serde_json::to_vec_pretty(&document)?;
        bytes.push(b'\n');

        self.check_current()?;
        transaction.replace(self.json.as_deref(), &bytes)?;

        *self = Self::load_unlocked(transaction.path())?;
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

    /// Persist configurator preferences without deleting shortcuts or rewriting YAML.
    /// `theme`: desired appearance; a newly selected premium theme needs entitlement.
    /// `runtime`: validated detection/polling settings.
    /// `app_pages`: known application page names paired with their enabled flags.
    /// Validation, stale-editor and write failures propagate to the dialog.
    pub fn save(
        &mut self,
        theme: Theme,
        runtime: RuntimeSettings,
        app_pages: &[(String, bool)],
    ) -> Result<()> {
        runtime.validate()?;
        // Keep a previously chosen theme on disk when changing unrelated settings.
        // Runtime policy independently checks it before drawing, including hand-edited JSON.
        if theme != self.config.theme {
            crate::licensing::require(theme)?;
        }

        for (name, _) in app_pages {
            let known_page = self
                .config
                .pages
                .iter()
                .any(|page| page.name == *name && !page.executables.is_empty());
            ensure!(known_page, "Unknown application page: {name}");
        }

        let mut document = self.document()?;
        document["theme"] = serde_json::to_value(theme)?;
        document["runtime"] = serde_json::to_value(runtime)?;

        let pages = document["pages"]
            .as_array_mut()
            .context("settings.json must contain a pages array")?;

        for (name, enabled) in app_pages {
            let page = pages
                .iter_mut()
                .find(|page| page["name"].as_str() == Some(name))
                .context("application page missing from settings")?;
            page["enabled"] = (*enabled).into();
        }

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

        settings.save(
            Theme::LightClassic,
            RuntimeSettings {
                detect_applications: false,
                auto_switch: false,
                focus_poll_seconds: 7,
            },
            &[("Word".into(), false), ("Codex".into(), false)],
        )?;

        let original: serde_json::Value = serde_json::from_slice(&original_json)?;
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        assert_eq!(saved["owner_extension"], original["owner_extension"]);
        assert_eq!(saved["pages"][0]["keys"], original["pages"][0]["keys"]);
        assert_eq!(
            fs::read(path.with_file_name("applications.yaml"))?,
            catalogue
        );
        assert_eq!(
            fs::read(fixture.0.join("settings.json.gui.bak"))?,
            original_json
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

        assert!(settings
            .save(Theme::Memphis, RuntimeSettings::default(), &[])
            .is_err());
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

        settings.save(Theme::Memphis, RuntimeSettings::default(), &[])?;
        assert_eq!(Config::load(&path)?.theme, Theme::Memphis);
        assert_eq!(fs::read(&temporary)?, b"keep existing staging data");
        Ok(())
    }
}
