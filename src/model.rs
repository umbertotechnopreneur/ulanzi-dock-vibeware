/* VBWR B
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 * VibeWare is Human intent, AI, and plenty of tokens ;-)
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 * AI Tooling: May include OpenAI Codex, GitHub Copilot and AI-assisted CI/CD pipelines.
 * AI Versions: Tools and models may vary by contributor and execution environment.
 * AI Traceability: Refer to Git history and CI/CD logs for recorded provenance.
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 * VBWR E */

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const KEY_COUNT: usize = 14;
pub const NEXT_KEY: usize = 4;
pub const CLOCK_KEY: usize = 13;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum Theme {
    DarkClassic,
    DarkAbstract,
    LightClassic,
    LightAbstract,
}

impl Theme {
    pub const ALL: [Self; 4] = [
        Self::DarkClassic,
        Self::DarkAbstract,
        Self::LightClassic,
        Self::LightAbstract,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::DarkClassic => "dark-classic",
            Self::DarkAbstract => "dark-abstract",
            Self::LightClassic => "light-classic",
            Self::LightAbstract => "light-abstract",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyConfig {
    pub label: String,
    pub action: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PageConfig {
    pub name: String,
    pub keys: Vec<KeyConfig>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: Theme,
    pub pages: Vec<PageConfig>,
}

impl Default for Config {
    fn default() -> Self {
        fn page(name: &str, keys: [(&str, &str); KEY_COUNT]) -> PageConfig {
            PageConfig {
                name: name.into(),
                keys: keys
                    .into_iter()
                    .map(|(label, action)| KeyConfig {
                        label: label.into(),
                        action: action.into(),
                    })
                    .collect(),
            }
        }

        Self {
            theme: Theme::DarkClassic,
            pages: vec![
                page(
                    "Windows / media",
                    [
                        ("PLAY / PAUSE", "media:play-pause"),
                        ("PREVIOUS", "media:previous"),
                        ("NEXT TRACK", "media:next"),
                        ("MUTE", "media:mute"),
                        ("NEXT PAGE", "page:next"),
                        ("VOL DOWN", "media:volume-down"),
                        ("VOL UP", "media:volume-up"),
                        ("FILE EXPLORER", "hotkey:super+e"),
                        ("DESKTOP", "hotkey:super+d"),
                        ("SETTINGS", "hotkey:super+i"),
                        ("SCREENSHOT", "hotkey:super+shift+s"),
                        ("TASK VIEW", "hotkey:super+tab"),
                        ("BROWSER", "open:https://www.google.com"),
                        ("CLOCK", "none"),
                    ],
                ),
                page(
                    "Codex",
                    [
                        ("NEW CHAT", "hotkey:ctrl+n"),
                        ("PROJECTS", "hotkey:ctrl+alt+shift+o"),
                        ("ATTENTION", "hotkey:ctrl+alt+a"),
                        ("SIDE CHAT", "hotkey:ctrl+alt+s"),
                        ("NEXT PAGE", "page:next"),
                        ("REVIEW", "hotkey:ctrl+shift+g"),
                        ("TERMINAL", "hotkey:ctrl+grave"),
                        ("FILES", "hotkey:ctrl+p"),
                        ("COMMANDS", "hotkey:ctrl+shift+p"),
                        ("COPY LINK", "hotkey:ctrl+alt+l"),
                        ("COPY", "hotkey:ctrl+c"),
                        ("PASTE", "hotkey:ctrl+v"),
                        ("FIND", "hotkey:ctrl+f"),
                        ("CLOCK", "none"),
                    ],
                ),
                page(
                    "VS Code",
                    [
                        ("QUICK OPEN", "hotkey:ctrl+p"),
                        ("COMMANDS", "hotkey:ctrl+shift+p"),
                        ("TERMINAL", "hotkey:ctrl+grave"),
                        ("SEARCH", "hotkey:ctrl+shift+f"),
                        ("NEXT PAGE", "page:next"),
                        ("EXPLORER", "hotkey:ctrl+shift+e"),
                        ("SOURCE CTRL", "hotkey:ctrl+shift+g"),
                        ("DEBUG", "hotkey:ctrl+shift+d"),
                        ("EXTENSIONS", "hotkey:ctrl+shift+x"),
                        ("SAVE", "hotkey:ctrl+s"),
                        ("NEW FILE", "hotkey:ctrl+n"),
                        ("FORMAT", "hotkey:shift+alt+f"),
                        ("PROBLEMS", "hotkey:ctrl+shift+m"),
                        ("CLOCK", "none"),
                    ],
                ),
            ],
        }
    }
}

impl Config {
    // path: JSON settings file to load, or a missing path for defaults.
    // Errors: malformed JSON, failed file access, or invalid layout.
    pub fn load(path: &Path) -> Result<Self> {
        let config = if path.exists() {
            serde_json::from_slice(
                &fs::read(path).with_context(|| format!("reading {}", path.display()))?,
            )
            .with_context(|| format!("parsing {}", path.display()))?
        } else {
            Self::default()
        };
        config.validate()?;
        Ok(config)
    }

    // path: destination JSON settings file; existing files are preserved.
    // Errors: invalid layout, serialization failure, or failed file write.
    pub fn save_new(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let json = serde_json::to_vec_pretty(self)?;
        fs::write(path, json).with_context(|| format!("writing {}", path.display()))
    }

    // Errors: a missing page, wrong key count, or reassigned navigation key.
    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(!self.pages.is_empty(), "at least one page is required");
        for page in &self.pages {
            anyhow::ensure!(
                page.keys.len() == KEY_COUNT,
                "page '{}' must have 14 keys",
                page.name
            );
            anyhow::ensure!(
                page.keys[NEXT_KEY].action == "page:next",
                "key 4 must remain page:next"
            );
            anyhow::ensure!(
                page.keys[CLOCK_KEY].action == "none",
                "key 13 is reserved for the clock"
            );
        }
        Ok(())
    }
}
