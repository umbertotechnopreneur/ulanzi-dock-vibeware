/* VBWR B
 *
 * Project: UlanziDeck
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Modified with AI: OpenAI Codex; installed Linux application choices; 2026-11-02
 * Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed Linux support
 * Evidence: current source
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 *
 * VBWR E */

//! Find supported Linux desktop apps without launching them or inspecting running apps.
//! Known desktop IDs cover native packages, Flatpak exports and Snap launchers. Native
//! commands are considered only when no known desktop entry exists for that profile:
//! a command in PATH must not undo a user's Hidden override. Separate visible installs
//! with different desktop IDs can still make a profile available.
//!
//! This is installation evidence, not proof that an app starts or accepts shortcuts.
//! NoDisplay and desktop-menu filters do not mean an app is uninstalled. Custom desktop
//! IDs and unexported Flatpak installations are not discovered. LibreOffice modules need
//! a module launcher; the shared soffice command cannot establish which modules exist.
//! Their shared executable name must never be used to distinguish active modules.

pub mod shortcuts;

use anyhow::{Context, Result};
use std::{
    collections::HashMap,
    env,
    fs,
    io::ErrorKind,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

#[derive(Debug)]
/// A supported app that can be assigned to one existing dock page.
pub struct Profile {
    pub id: &'static str,
    pub name: &'static str,
    pub page: &'static str,
    pub executables: &'static [&'static str],
    desktop_files: &'static [&'static str],
    commands: &'static [&'static str],
}

/// Supported choices, in display order. These entries never enable pages by themselves.
pub static PROFILES: &[Profile] = &[
    Profile {
        id: "code",
        name: "VS Code",
        page: "VS Code",
        executables: &["code"],
        desktop_files: &[
            "com.microsoft.VSCode.desktop",
            "code.desktop",
            "com.visualstudio.code.desktop",
            "code_code.desktop",
        ],
        commands: &["code"],
    },
    Profile {
        id: "code-insiders",
        name: "VS Code Insiders",
        page: "VS Code",
        executables: &["code-insiders"],
        desktop_files: &[
            "com.microsoft.VSCodeInsiders.desktop",
            "code-insiders.desktop",
            "com.visualstudio.code.insiders.desktop",
            "code-insiders_code-insiders.desktop",
        ],
        commands: &["code-insiders"],
    },
    Profile {
        id: "codium",
        name: "VSCodium",
        page: "VS Code",
        executables: &["codium"],
        desktop_files: &[
            "codium.desktop",
            "com.vscodium.codium.desktop",
            "codium_codium.desktop",
        ],
        commands: &["codium"],
    },
    Profile {
        id: "spotify",
        name: "Spotify",
        page: "Spotify",
        executables: &["spotify"],
        desktop_files: &[
            "spotify.desktop",
            "com.spotify.Client.desktop",
            "spotify_spotify.desktop",
        ],
        commands: &["spotify"],
    },
    Profile {
        id: "obs",
        name: "OBS Studio",
        page: "OBS Creator",
        executables: &["obs"],
        desktop_files: &[
            "com.obsproject.Studio.desktop",
            "obs.desktop",
            "obs-studio_obs-studio.desktop",
        ],
        commands: &["obs", "obs-studio"],
    },
    Profile {
        id: "libreoffice-writer",
        name: "LibreOffice Writer",
        page: "Word",
        executables: &["soffice.bin"],
        desktop_files: &[
            "libreoffice-writer.desktop",
            "org.libreoffice.LibreOffice.writer.desktop",
            "libreoffice_writer.desktop",
        ],
        commands: &["lowriter"],
    },
    Profile {
        id: "libreoffice-impress",
        name: "LibreOffice Impress",
        page: "PowerPoint",
        executables: &["soffice.bin"],
        desktop_files: &[
            "libreoffice-impress.desktop",
            "org.libreoffice.LibreOffice.impress.desktop",
            "libreoffice_impress.desktop",
        ],
        commands: &["loimpress"],
    },
    Profile {
        id: "libreoffice-calc",
        name: "LibreOffice Calc",
        page: "Excel",
        executables: &["soffice.bin"],
        desktop_files: &[
            "libreoffice-calc.desktop",
            "org.libreoffice.LibreOffice.calc.desktop",
            "libreoffice_calc.desktop",
        ],
        commands: &["localc"],
    },
];

/// Look up a stable saved choice without discovering or starting applications.
pub fn profile(id: &str) -> Option<&'static Profile> {
    PROFILES.iter().find(|profile| profile.id == id)
}

/// Read installation evidence. Missing paths are normal; other I/O errors are reported.
/// The caller can show the error instead of claiming that no supported app is installed.
pub fn installed() -> Result<Vec<&'static Profile>> {
    let paths = SearchPaths::from_environment();
    let mut found = Vec::new();
    for candidate in PROFILES {
        if is_installed(candidate, &paths)? {
            found.push(candidate);
        }
    }
    Ok(found)
}

struct SearchPaths {
    applications: Vec<PathBuf>,
    commands: Vec<PathBuf>,
}

impl SearchPaths {
    fn from_environment() -> Self {
        let home = absolute_variable("HOME");
        let data_home = absolute_variable("XDG_DATA_HOME")
            .or_else(|| home.as_ref().map(|home| home.join(".local/share")));
        let mut applications = Vec::new();
        if let Some(directory) = &data_home {
            push_unique(&mut applications, directory.join("applications"));
        }
        let data_dirs = env::var_os("XDG_DATA_DIRS")
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
        // XDG requires absolute paths. An invalid entry is ignored, never resolved
        // against the controller's current working directory.
        for directory in env::split_paths(&data_dirs).filter(|path| path.is_absolute()) {
            push_unique(&mut applications, directory.join("applications"));
        }
        // Preserve the supplied XDG order, then cover ordinary exported installs
        // when their package manager has not added its directories to this session.
        if let Some(directory) = data_home {
            push_unique(
                &mut applications,
                directory.join("flatpak/exports/share/applications"),
            );
        }
        push_unique(
            &mut applications,
            PathBuf::from("/var/lib/flatpak/exports/share/applications"),
        );
        push_unique(
            &mut applications,
            PathBuf::from("/var/lib/snapd/desktop/applications"),
        );
        let commands = env::var_os("PATH")
            .map(|value| env::split_paths(&value).collect())
            .unwrap_or_default();
        Self {
            applications,
            commands,
        }
    }
}

fn absolute_variable(name: &str) -> Option<PathBuf> {
    env::var_os(name)
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
}

fn push_unique(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.contains(&path) {
        paths.push(path);
    }
}

fn is_installed(profile: &Profile, paths: &SearchPaths) -> Result<bool> {
    let mut has_entry = false;
    let mut available = false;
    for filename in profile.desktop_files {
        for directory in &paths.applications {
            let path = directory.join(filename);
            let Some(contents) = read_desktop(&path)? else {
                continue;
            };
            has_entry = true;
            let entry = DesktopEntry::parse(&contents)
                .with_context(|| format!("reading desktop entry {}", path.display()))?;
            if entry.available {
                let try_exec_available = match entry.try_exec {
                    Some(command) => command_available(&command, &paths.commands)?,
                    None => true,
                };
                available |= try_exec_available;
            }
            // The first existing file owns this desktop ID, including Hidden=true,
            // invalid application entries and missing TryExec targets.
            break;
        }
    }
    if has_entry {
        return Ok(available);
    }
    for command in profile.commands {
        if command_available(command, &paths.commands)? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn read_desktop(path: &Path) -> Result<Option<String>> {
    match fs::metadata(path) {
        Ok(metadata) => anyhow::ensure!(
            metadata.is_file(),
            "desktop entry is not a regular file: {}",
            path.display()
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(error).with_context(|| format!("inspecting {}", path.display()));
        }
    }
    fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))
        .map(Some)
}

struct DesktopEntry {
    available: bool,
    try_exec: Option<String>,
}

impl DesktopEntry {
    fn parse(contents: &str) -> Result<Self> {
        let mut in_entry = false;
        let mut fields = HashMap::new();
        for line in contents.lines() {
            let line = line.trim();
            if line.starts_with('[') {
                in_entry = line == "[Desktop Entry]";
                continue;
            }
            if !in_entry || line.starts_with('#') || line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                if matches!(key, "Type" | "Exec" | "Hidden" | "TryExec") {
                    anyhow::ensure!(fields.insert(key, value.trim()).is_none(), "duplicate {key}");
                }
            }
        }
        let hidden = match fields.get("Hidden").copied() {
            Some("true") => true,
            Some("false") | None => false,
            Some(_) => anyhow::bail!("invalid Hidden value"),
        };
        Ok(Self {
            available: !hidden
                && fields.get("Type") == Some(&"Application")
                && fields.get("Exec").is_some_and(|value| !value.is_empty()),
            try_exec: fields
                .get("TryExec")
                .map(|command| unescape_string(command))
                .transpose()?,
        })
    }
}

// Desktop strings use their own escapes; they are never shell commands here.
fn unescape_string(value: &str) -> Result<String> {
    let mut decoded = String::new();
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        decoded.push(if character == '\\' {
            match chars.next() {
                Some('s') => ' ',
                Some('n') => '\n',
                Some('t') => '\t',
                Some('r') => '\r',
                Some('\\') => '\\',
                _ => anyhow::bail!("invalid TryExec escape"),
            }
        } else {
            character
        });
    }
    Ok(decoded)
}

fn command_available(command: &str, directories: &[PathBuf]) -> Result<bool> {
    let path = Path::new(command);
    if path.is_absolute() {
        return executable_file(path);
    }
    if command.is_empty() || command.contains('/') {
        return Ok(false);
    }
    for directory in directories {
        if executable_file(&directory.join(path))? {
            return Ok(true);
        }
    }
    Ok(false)
}

fn executable_file(path: &Path) -> Result<bool> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.is_file() && metadata.permissions().mode() & 0o111 != 0),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("inspecting {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    #[test]
    fn every_profile_preset_preserves_navigation_and_has_valid_actions() {
        for profile in PROFILES {
            let preset = shortcuts::for_profile(profile.id);
            assert!(!preset.is_empty(), "missing preset for {}", profile.id);
            let mut assigned = std::collections::HashSet::new();
            for shortcut in preset {
                assert!(
                    shortcut.key < crate::model::CLOCK_KEY
                        && shortcut.key != crate::model::NEXT_KEY,
                    "reserved or invalid key {} in {}",
                    shortcut.key,
                    profile.id
                );
                assert!(
                    assigned.insert(shortcut.key),
                    "duplicate key {} in {}",
                    shortcut.key,
                    profile.id
                );
                crate::actions::validate(shortcut.action).unwrap_or_else(|error| {
                    panic!("invalid preset action in {}: {error}", profile.id)
                });
            }
        }
    }

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = env::temp_dir().join(format!(
                "ulanzi-linux-apps-{}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }

        fn directory(&self, name: &str) -> PathBuf {
            let path = self.0.join(name);
            fs::create_dir_all(&path).unwrap();
            path
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn hidden_override_blocks_lower_entry_and_path_command() {
        let fixture = Fixture::new();
        let user = fixture.directory("user");
        let system = fixture.directory("system");
        let bin = fixture.directory("bin");
        fs::write(user.join("code.desktop"), "[Desktop Entry]\nHidden=true\n").unwrap();
        fs::write(
            system.join("code.desktop"),
            "[Desktop Entry]\nType=Application\nExec=code %F\n",
        )
        .unwrap();
        fs::write(bin.join("code"), "fixture only; never executed").unwrap();
        fs::set_permissions(bin.join("code"), fs::Permissions::from_mode(0o755)).unwrap();
        let paths = SearchPaths {
            applications: vec![user, system],
            commands: vec![bin],
        };
        assert!(!is_installed(profile("code").unwrap(), &paths).unwrap());
    }

    #[test]
    fn desktop_actions_and_links_are_not_application_evidence() {
        let action_only = DesktopEntry::parse(
            "[Desktop Entry]\nType=Application\n[Desktop Action Open]\nExec=code\n",
        )
        .unwrap();
        assert!(!action_only.available);
        assert!(!DesktopEntry::parse("[Desktop Entry]\nType=Link\nExec=code\n")
            .unwrap()
            .available);
        assert!(DesktopEntry::parse("[Desktop Entry]\nHidden=maybe\n").is_err());
    }

    #[test]
    fn libreoffice_shared_command_does_not_imply_all_modules() {
        let fixture = Fixture::new();
        let apps = fixture.directory("applications");
        let bin = fixture.directory("bin");
        fs::write(bin.join("soffice"), "fixture only; never executed").unwrap();
        fs::set_permissions(bin.join("soffice"), fs::Permissions::from_mode(0o755)).unwrap();
        fs::write(
            apps.join("org.libreoffice.LibreOffice.writer.desktop"),
            "[Desktop Entry]\nType=Application\nExec=flatpak run org.libreoffice.LibreOffice --writer\n",
        )
        .unwrap();
        let paths = SearchPaths {
            applications: vec![apps],
            commands: vec![bin],
        };
        assert!(is_installed(profile("libreoffice-writer").unwrap(), &paths).unwrap());
        assert!(!is_installed(profile("libreoffice-calc").unwrap(), &paths).unwrap());
        assert!(!is_installed(profile("libreoffice-impress").unwrap(), &paths).unwrap());
    }
}
