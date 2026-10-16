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

use crate::model::{Config, RuntimeSettings, Theme};
use anyhow::{ensure, Context, Result};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

/// Saves only the requested preferences, preserving custom keys and metadata.
/// Snapshots prevent a dialog from overwriting edits made since it was opened.
pub struct SettingsStore {
    pub config: Config,
    path: PathBuf,
    json: Option<Vec<u8>>,
    catalogue: Option<Vec<u8>>,
}

fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("reading {}", path.display())),
    }
}

impl SettingsStore {
    pub fn load(path: &Path) -> Result<Self> {
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

    fn check_current(&self) -> Result<()> {
        let current_json = read_optional(&self.path)?;
        let current_catalogue = read_optional(&self.path.with_file_name("applications.yaml"))?;

        ensure!(
            current_json == self.json && current_catalogue == self.catalogue,
            "Settings changed outside this dialog. Click Reload before saving."
        );
        Ok(())
    }

    fn document(&self) -> Result<serde_json::Value> {
        match &self.json {
            Some(bytes) => Ok(serde_json::from_slice(bytes)?),
            None => Ok(serde_json::to_value(&self.config)?),
        }
    }

    /// Back up and atomically replace one file. The application catalogue is read-only here.
    fn save_document(&mut self, document: serde_json::Value) -> Result<()> {
        self.check_current()?;

        let config: Config = serde_json::from_value(document.clone())?;
        config.validate()?;

        let mut bytes = serde_json::to_vec_pretty(&document)?;
        bytes.push(b'\n');

        let change = Change::stage(&self.path, self.json.clone(), bytes)?;
        change.backup()?;
        self.check_current()?;
        change.apply()?;

        *self = Self::load(&self.path)?;
        Ok(())
    }

    fn save_field(&mut self, name: &str, value: serde_json::Value) -> Result<()> {
        let mut document = self.document()?;
        document[name] = value;

        self.save_document(document)
    }

    pub fn save_brightness(&mut self, level: u8) -> Result<()> {
        ensure!(level <= 100, "brightness must be 0-100");
        self.save_field("brightness", level.into())
    }

    pub fn save_theme(&mut self, theme: Theme) -> Result<()> {
        self.save_field("theme", serde_json::to_value(theme)?)
    }

    /// Persist completion before the caller marks the in-memory setup as finished.
    pub fn save_setup_completed(&mut self) -> Result<()> {
        self.save_field("setup_completed", true.into())
    }

    pub fn save(
        &mut self,
        theme: Theme,
        runtime: RuntimeSettings,
        app_pages: &[(String, bool)],
    ) -> Result<()> {
        runtime.validate()?;

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

/// A flushed replacement file kept beside its destination for an atomic rename.
struct Change {
    path: PathBuf,
    temporary: PathBuf,
    original: Option<Vec<u8>>,
}

impl Change {
    // path: destination next to the temporary file, ensuring replacement stays on one volume.
    // original: bytes captured when the dialog loaded, used for its backup.
    // bytes: validated replacement content.
    // Errors: creating, writing, or flushing the temporary file.
    fn stage(path: &Path, original: Option<Vec<u8>>, bytes: Vec<u8>) -> Result<Self> {
        let temporary = path.with_extension(format!("gui-{}.tmp", std::process::id()));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .with_context(|| format!("staging {}", path.display()))?;
        let change = Self {
            path: path.to_owned(),
            temporary,
            original,
        };
        file.write_all(&bytes)?;
        file.sync_all()?;
        Ok(change)
    }

    // Errors: writing the adjacent .gui.bak file before any destination is changed.
    fn backup(&self) -> Result<()> {
        if let Some(bytes) = &self.original {
            fs::write(
                self.path.with_extension(format!(
                    "{}.gui.bak",
                    self.path.extension().unwrap_or_default().to_string_lossy()
                )),
                bytes,
            )?;
        }
        Ok(())
    }

    // Errors: atomically replacing the destination file.
    fn apply(&self) -> Result<()> {
        replace(&self.temporary, &self.path)
    }
}

impl Drop for Change {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.temporary);
    }
}

// source: completed temporary file in the destination's directory.
// destination: settings file to create or replace.
// Errors: OS replacement failure.
fn replace(source: &Path, destination: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{
            MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
        };
        let source: Vec<_> = source.as_os_str().encode_wide().chain(Some(0)).collect();
        let destination: Vec<_> = destination
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        ensure!(
            unsafe {
                MoveFileExW(
                    source.as_ptr(),
                    destination.as_ptr(),
                    MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
                )
            } != 0,
            "Replacing settings failed: {}",
            std::io::Error::last_os_error()
        );
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        fs::rename(source, destination).context("replacing settings")
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
                "applications.yaml",
                "settings.json.gui.bak",
            ] {
                let _ = fs::remove_file(self.0.join(name));
            }
            let temporary = format!("settings.gui-{}.tmp", std::process::id());
            let _ = fs::remove_file(self.0.join(temporary));
            let _ = fs::remove_dir(&self.0);
        }
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
    fn staging_failure_preserves_settings_and_the_existing_temporary_file() -> Result<()> {
        let fixture = Fixture::create()?;
        let path = fixture.0.join("settings.json");
        let mut settings = SettingsStore::load(&path)?;
        let original = fs::read(&path)?;
        let temporary = fixture
            .0
            .join(format!("settings.gui-{}.tmp", std::process::id()));
        fs::write(&temporary, b"keep existing staging data")?;

        assert!(settings
            .save(Theme::Memphis, RuntimeSettings::default(), &[])
            .is_err());
        assert_eq!(fs::read(&path)?, original);
        assert_eq!(fs::read(&temporary)?, b"keep existing staging data");
        Ok(())
    }
}
