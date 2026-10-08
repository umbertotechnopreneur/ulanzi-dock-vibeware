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

use crate::{
    applications::Catalog,
    model::{Config, Theme},
};
use anyhow::{ensure, Context, Result};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

pub struct Settings {
    pub config: Config,
    path: PathBuf,
    json: Option<Vec<u8>>,
    yaml: Option<Vec<u8>>,
}

// path: configuration file, which may not exist before first-run setup.
// Errors: read failure other than a missing file.
fn read_optional(path: &Path) -> Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("reading {}", path.display())),
    }
}

impl Settings {
    // path: settings.json, with applications.yaml in the same directory.
    // Errors: invalid settings/catalogue, filesystem failure, or concurrent change.
    pub fn load(path: &Path) -> Result<Self> {
        let json = read_optional(path)?;
        let yaml = read_optional(&path.with_file_name("applications.yaml"))?;
        let config = Config::load(path)?;
        let settings = Self {
            config,
            path: path.to_owned(),
            json,
            yaml,
        };
        settings.check_current()?;
        Ok(settings)
    }

    // Errors: reading a configuration file, or an edit made since this dialog loaded.
    fn check_current(&self) -> Result<()> {
        ensure!(
            read_optional(&self.path)? == self.json
                && read_optional(&self.path.with_file_name("applications.yaml"))? == self.yaml,
            "Settings changed outside this dialog. Click Reload before saving."
        );
        Ok(())
    }

    // theme: one of the thirteen existing dock themes.
    // auto_switch: enable automatic foreground application page selection.
    // detect_applications: enable application availability checks and inactive overlays.
    // poll_seconds: foreground check interval, limited to 1-60 seconds.
    // Errors: stale settings, validation, backup, staging, or replacement failure.
    pub fn save(
        &mut self,
        theme: Theme,
        detect_applications: bool,
        auto_switch: bool,
        poll_seconds: u64,
    ) -> Result<()> {
        ensure!(
            (1..=60).contains(&poll_seconds),
            "The interval must be 1-60 seconds."
        );
        self.check_current()?;
        let mut changes = Vec::new();
        {
            // Reuse each existing raw page object: retain shortcuts and unknown nested fields.
            let mut value = match &self.json {
                Some(bytes) => serde_json::from_slice::<serde_json::Value>(bytes)?,
                None => serde_json::to_value(&self.config)?,
            };
            let original = value.clone();
            let pages = value
                .get_mut("pages")
                .and_then(serde_json::Value::as_array_mut)
                .context("settings.json must contain a pages array")?;
            let mut existing = std::mem::take(pages);
            for page in &self.config.pages {
                if let Some(index) = existing
                    .iter()
                    .position(|raw| raw["name"].as_str() == Some(&page.name))
                {
                    pages.push(existing.remove(index));
                } else {
                    pages.push(serde_json::to_value(page)?);
                }
            }
            pages.extend(existing);
            value
                .as_object_mut()
                .context("settings.json must contain an object")?
                .insert("theme".into(), serde_json::to_value(theme)?);
            if self.json.is_none() || value != original {
                let mut bytes = serde_json::to_vec_pretty(&value)?;
                bytes.push(b'\n');
                changes.push(Change::stage(&self.path, self.json.clone(), bytes)?);
            }
        }
        if self.yaml.is_none()
            || detect_applications != self.config.runtime.detect_applications
            || auto_switch != self.config.runtime.auto_switch
            || poll_seconds != self.config.runtime.focus_poll_seconds
        {
            let source = match &self.yaml {
                Some(bytes) => std::str::from_utf8(bytes)?,
                None => include_str!("../applications.yaml"),
            };
            let text = patch_runtime(source, detect_applications, auto_switch, poll_seconds)?;
            let catalogue: Catalog = serde_yaml_ng::from_str(&text)?;
            catalogue.apply(&mut self.config.clone())?;
            changes.push(Change::stage(
                &self.path.with_file_name("applications.yaml"),
                self.yaml.clone(),
                text.into_bytes(),
            )?);
        }
        // Prepare every file and backup before changing either destination.
        for change in &changes {
            change.backup()?;
        }
        self.check_current()?;
        for (index, change) in changes.iter().enumerate() {
            if let Err(error) = change.apply() {
                // Restore earlier destinations if the second replacement fails.
                for previous in changes[..index].iter().rev() {
                    previous
                        .restore()
                        .context("restoring settings after an incomplete save")?;
                }
                return Err(error);
            }
        }
        *self = Self::load(&self.path)?;
        Ok(())
    }
}

// source: original YAML; comments, application actions, and mappings are retained.
// auto_switch: new automatic switching value.
// detect_applications: new application availability detection value.
// poll_seconds: validated foreground interval.
// Errors: inline/anchored runtime maps require manual editing instead of rewriting the file.
fn patch_runtime(
    source: &str,
    detect_applications: bool,
    auto_switch: bool,
    poll_seconds: u64,
) -> Result<String> {
    let newline = if source.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    // Keep each original line ending, including mixed CRLF/LF files and no final newline.
    let mut lines: Vec<String> = source.split_inclusive('\n').map(str::to_owned).collect();
    let start = lines.iter().position(|line| line.starts_with("runtime:"));
    if let Some(start) = start {
        let suffix = lines[start]["runtime:".len()..].trim();
        ensure!(
            suffix.is_empty() || suffix.starts_with('#'),
            "Edit the inline runtime mapping in applications.yaml manually."
        );
        let end = (start + 1..lines.len())
            .find(|&index| {
                let line = &lines[index];
                !line.trim().is_empty()
                    && !line.trim_start().starts_with('#')
                    && !line.starts_with([' ', '\t'])
            })
            .unwrap_or(lines.len());
        let mut auto_found = false;
        let mut poll_found = false;
        let mut detect_found = false;
        let indent = lines[start + 1..end]
            .iter()
            .find(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
            .map(|line| line[..line.len() - line.trim_start().len()].to_owned())
            .unwrap_or_else(|| "  ".into());
        for line in &mut lines[start + 1..end] {
            let ending = if line.ends_with("\r\n") {
                "\r\n"
            } else if line.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            let content = line.trim_end_matches(['\r', '\n']);
            let trimmed = content.trim_start();
            let replacement = if trimmed.starts_with("detect_applications:") {
                detect_found = true;
                Some(format!("detect_applications: {detect_applications}"))
            } else if trimmed.starts_with("auto_switch:") {
                auto_found = true;
                Some(format!("auto_switch: {auto_switch}"))
            } else if trimmed.starts_with("focus_poll_seconds:") {
                poll_found = true;
                Some(format!("focus_poll_seconds: {poll_seconds}"))
            } else {
                None
            };
            if let Some(replacement) = replacement {
                let indent = &content[..content.len() - trimmed.len()];
                let comment = trimmed
                    .find('#')
                    .map(|offset| format!(" {}", &trimmed[offset..]))
                    .unwrap_or_default();
                *line = format!("{indent}{replacement}{comment}{ending}");
            }
        }
        let mut missing = Vec::new();
        if !detect_found {
            missing.push(format!(
                "{indent}detect_applications: {detect_applications}{newline}"
            ));
        }
        if !auto_found {
            missing.push(format!("{indent}auto_switch: {auto_switch}{newline}"));
        }
        if !poll_found {
            missing.push(format!(
                "{indent}focus_poll_seconds: {poll_seconds}{newline}"
            ));
        }
        if !missing.is_empty() && end > 0 && !lines[end - 1].ends_with('\n') {
            lines[end - 1].push_str(newline);
        }
        lines.splice(end..end, missing);
    } else {
        if let Some(last) = lines.last_mut() {
            if !last.ends_with('\n') {
                last.push_str(newline);
            }
        }
        lines.extend([
            format!("runtime:{newline}"),
            format!("  detect_applications: {detect_applications}{newline}"),
            format!("  auto_switch: {auto_switch}{newline}"),
            format!("  focus_poll_seconds: {poll_seconds}{newline}"),
        ]);
    }
    Ok(lines.concat())
}

struct Change {
    path: PathBuf,
    temporary: PathBuf,
    original: Option<Vec<u8>>,
}

impl Change {
    // path: destination next to the temporary file, ensuring replacement stays on one volume.
    // original: bytes captured when the dialog loaded, used for backup and rollback.
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

    // Errors: restoring an original destination after a later replacement failed.
    fn restore(&self) -> Result<()> {
        if let Some(bytes) = &self.original {
            fs::write(&self.temporary, bytes)?;
            self.apply()
        } else {
            fs::remove_file(&self.path).context("removing a newly created settings file")
        }
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
        // Creates only isolated settings copies, with no GUI, HID, or user-file access.
        // Errors: filesystem or serialization failure propagated to the test.
        fn create() -> Result<Self> {
            let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ulanzi-gui-settings-{}-{stamp}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path)?;
            let fixture = Self(path);
            let mut json = serde_json::to_value(Config::default())?;
            let pages = json["pages"].as_array_mut().expect("fixture pages");
            pages.retain(|page| {
                !matches!(page["name"].as_str(), Some("Word" | "PowerPoint" | "Excel"))
            });
            pages.sort_by_key(|page| match page["name"].as_str().unwrap_or("") {
                "Windows / media" => 0,
                "Codex" => 1,
                "VS Code" => 2,
                "Utility" => 3,
                "Themes" => 4,
                _ => 5,
            });
            json["pages"][0]["keys"][0]["action"] = "hotkey:ctrl+shift+f12".into();
            json["owner_extension"] = serde_json::json!({"keep": "custom metadata"});
            fs::write(fixture.0.join("settings.json"), serde_json::to_vec(&json)?)?;
            let yaml = include_str!("../applications.yaml")
                .replace("auto_switch: true", "auto_switch: true # owner preference")
                .replace("focus_poll_seconds: 1", "focus_poll_seconds: 1 # seconds");
            fs::write(fixture.0.join("applications.yaml"), yaml)?;
            Ok(fixture)
        }
    }

    impl Drop for Fixture {
        // Remove only the named files created by this isolated fixture, never recurse.
        fn drop(&mut self) {
            for name in [
                "settings.json",
                "applications.yaml",
                "settings.json.gui.bak",
                "applications.yaml.gui.bak",
            ] {
                let _ = fs::remove_file(self.0.join(name));
            }
            let _ = fs::remove_file(
                self.0
                    .join(format!("applications.gui-{}.tmp", std::process::id())),
            );
            let _ = fs::remove_dir(&self.0);
        }
    }

    // Errors: fixture or settings save failures; assertions report lost user content.
    #[test]
    fn save_retains_custom_actions_metadata_and_catalogue_comments() -> Result<()> {
        let fixture = Fixture::create()?;
        let path = fixture.0.join("settings.json");
        let original_json = fs::read(&path)?;
        let original_yaml = fs::read_to_string(path.with_file_name("applications.yaml"))?;
        let mut settings = Settings::load(&path)?;
        settings.save(Theme::LightClassic, false, false, 7)?;
        let expected = serde_json::from_slice::<serde_json::Value>(&original_json)?;
        let saved = serde_json::from_slice::<serde_json::Value>(&fs::read(&path)?)?;
        assert_eq!(saved["theme"], "light-classic");
        assert_eq!(saved["owner_extension"], expected["owner_extension"]);
        for original_page in expected["pages"].as_array().expect("original pages") {
            let saved_page = saved["pages"]
                .as_array()
                .expect("saved pages")
                .iter()
                .find(|page| page["name"] == original_page["name"])
                .expect("preserved page");
            assert_eq!(saved_page, original_page);
        }
        let yaml = fs::read_to_string(path.with_file_name("applications.yaml"))?;
        assert!(
            yaml == original_yaml
                .replace("detect_applications: true", "detect_applications: false")
                .replace("auto_switch: true", "auto_switch: false")
                .replace("focus_poll_seconds: 1", "focus_poll_seconds: 7"),
            "YAML changed outside the three runtime values"
        );
        assert_eq!(
            fs::read(fixture.0.join("settings.json.gui.bak"))?,
            original_json
        );
        assert_eq!(
            fs::read_to_string(fixture.0.join("applications.yaml.gui.bak"))?,
            original_yaml
        );
        let loaded = Config::load(&path)?;
        assert_eq!(loaded.theme, Theme::LightClassic);
        assert!(!loaded.runtime.auto_switch);
        assert!(!loaded.runtime.detect_applications);
        assert_eq!(
            loaded
                .pages
                .iter()
                .map(|page| page.name.as_str())
                .collect::<Vec<_>>(),
            [
                "Windows / media",
                "Utility",
                "Codex",
                "VS Code",
                "Spotify",
                "Word",
                "PowerPoint",
                "Excel",
                "Themes"
            ]
        );
        assert_eq!(loaded.runtime.focus_poll_seconds, 7);
        Ok(())
    }

    // Errors: fixture or file access failures; assertions report overwritten external edits.
    #[test]
    fn stale_dialog_refuses_to_overwrite_external_changes() -> Result<()> {
        let fixture = Fixture::create()?;
        let path = fixture.0.join("settings.json");
        let mut settings = Settings::load(&path)?;
        let yaml = fs::read(path.with_file_name("applications.yaml"))?;
        let mut external = fs::read(&path)?;
        external.push(b'\n');
        fs::write(&path, &external)?;
        assert!(settings.save(Theme::Memphis, false, false, 8).is_err());
        assert_eq!(fs::read(&path)?, external);
        assert_eq!(fs::read(path.with_file_name("applications.yaml"))?, yaml);
        Ok(())
    }

    // Errors: fixture or file access failures; assertions report partial writes or lost staging files.
    #[test]
    fn staging_failure_preserves_both_settings_and_existing_temporary_file() -> Result<()> {
        let fixture = Fixture::create()?;
        let path = fixture.0.join("settings.json");
        let mut settings = Settings::load(&path)?;
        let json = fs::read(&path)?;
        let yaml = fs::read(path.with_file_name("applications.yaml"))?;
        let temporary = fixture
            .0
            .join(format!("applications.gui-{}.tmp", std::process::id()));
        fs::write(&temporary, b"keep existing staging data")?;
        assert!(settings.save(Theme::Memphis, false, false, 8).is_err());
        assert_eq!(fs::read(&path)?, json);
        assert_eq!(fs::read(path.with_file_name("applications.yaml"))?, yaml);
        assert_eq!(fs::read(&temporary)?, b"keep existing staging data");
        Ok(())
    }
}
