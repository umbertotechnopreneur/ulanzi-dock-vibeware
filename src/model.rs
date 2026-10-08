/* VBWR B
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 * VibeWare is Human intent. AI implementation. Accountable human review.
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
pub const THEME_KEYS: [usize; 12] = [0, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
pub enum Theme {
    DarkClassic,
    DarkAbstract,
    LightClassic,
    LightAbstract,
    MangaInk,
    SteampunkBrass,
    CyberpunkNeon,
    Moire,
    Cubism,
    ArtDeco,
    UkiyoE,
    Solarpunk,
    Memphis,
}

impl Theme {
    pub const ALL: [Self; 13] = [
        Self::DarkClassic,
        Self::DarkAbstract,
        Self::LightClassic,
        Self::LightAbstract,
        Self::MangaInk,
        Self::SteampunkBrass,
        Self::CyberpunkNeon,
        Self::Moire,
        Self::Cubism,
        Self::ArtDeco,
        Self::UkiyoE,
        Self::Solarpunk,
        Self::Memphis,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::DarkClassic => "dark-classic",
            Self::DarkAbstract => "dark-abstract",
            Self::LightClassic => "light-classic",
            Self::LightAbstract => "light-abstract",
            Self::MangaInk => "manga-ink",
            Self::SteampunkBrass => "steampunk-brass",
            Self::CyberpunkNeon => "cyberpunk-neon",
            Self::Moire => "moire",
            Self::Cubism => "cubism",
            Self::ArtDeco => "art-deco",
            Self::UkiyoE => "ukiyo-e",
            Self::Solarpunk => "solarpunk",
            Self::Memphis => "memphis",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::DarkClassic => "MATRIX",
            Self::DarkAbstract => "BLADE RUNNER",
            Self::LightClassic => "IVORY BLUE",
            Self::LightAbstract => "MINT TEAL",
            Self::MangaInk => "MANGA",
            Self::SteampunkBrass => "STEAMPUNK",
            Self::CyberpunkNeon => "CYBERPUNK",
            Self::Moire => "MOIRE",
            Self::Cubism => "CUBISM",
            Self::ArtDeco => "ART DECO",
            Self::UkiyoE => "UKIYO-E",
            Self::Solarpunk => "SOLARPUNK",
            Self::Memphis => "MEMPHIS",
        }
    }

    // slug: kebab-case theme ID from a selector action or settings.
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|theme| theme.slug() == slug)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct KeyConfig {
    pub label: String,
    pub action: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub asset: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PageConfig {
    pub name: String,
    pub keys: Vec<KeyConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub executables: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    pub theme: Theme,
    pub pages: Vec<PageConfig>,
    #[serde(default)]
    pub setup_completed: bool,
    #[serde(skip)]
    pub runtime: crate::applications::RuntimeSettings,
}

// name: user-facing page title.
// keys: labels and actions in physical key order.
fn page(name: &str, keys: [(&str, &str); KEY_COUNT]) -> PageConfig {
    PageConfig {
        name: name.into(),
        executables: match name {
            "Codex" => vec!["Codex.exe".into(), "ChatGPT.exe".into()],
            "VS Code" => vec!["Code.exe".into(), "Code - Insiders.exe".into()],
            "Spotify" => vec!["Spotify.exe".into()],
            _ => Vec::new(),
        },
        keys: keys
            .into_iter()
            .map(|(label, action)| KeyConfig {
                label: label.into(),
                action: action.into(),
                description: String::new(),
                asset: String::new(),
            })
            .collect(),
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: Theme::DarkClassic,
            setup_completed: false,
            runtime: crate::applications::RuntimeSettings::default(),
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
                Self::utility_page(),
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
                Self::spotify_page(),
                Self::office_page("Word"),
                Self::office_page("PowerPoint"),
                Self::office_page("Excel"),
                Self::theme_selector(Theme::DarkClassic),
            ],
        }
    }
}

impl Config {
    // name: one of the three desktop Office applications, each with its own page.
    // Shortcuts use Windows/English defaults and remain editable in settings.json.
    pub fn office_page(name: &str) -> PageConfig {
        let last = match name {
            "Word" => [
                ("BOLD", "hotkey:ctrl+b"),
                ("ITALIC", "hotkey:ctrl+i"),
                ("UNDERLINE", "hotkey:ctrl+u"),
                ("SPELLING", "hotkey:f7"),
                ("SAVE AS", "hotkey:f12"),
            ],
            "PowerPoint" => [
                ("START SHOW", "hotkey:f5"),
                ("CURRENT SLIDE", "hotkey:shift+f5"),
                ("NEW SLIDE", "hotkey:ctrl+m"),
                ("DUP SLIDE", "hotkey:ctrl+shift+d"),
                ("END SHOW", "hotkey:escape"),
            ],
            _ => [
                ("AUTO SUM", "hotkey:alt+="),
                ("FORMAT CELLS", "hotkey:ctrl+1"),
                ("INSERT DATE", "hotkey:ctrl+;"),
                ("EDIT CELL", "hotkey:f2"),
                ("FILTER", "hotkey:ctrl+shift+l"),
            ],
        };
        let mut office = page(
            name,
            [
                ("SAVE", "hotkey:ctrl+s"),
                ("OPEN", "hotkey:ctrl+o"),
                ("NEW FILE", "hotkey:ctrl+n"),
                ("PRINT", "hotkey:ctrl+p"),
                ("NEXT PAGE", "page:next"),
                ("UNDO", "hotkey:ctrl+z"),
                ("REDO", "hotkey:ctrl+y"),
                ("FIND", "hotkey:ctrl+f"),
                last[0],
                last[1],
                last[2],
                last[3],
                last[4],
                ("CLOCK", "none"),
            ],
        );
        office.executables = vec![match name {
            "Word" => "WINWORD.EXE",
            "PowerPoint" => "POWERPNT.EXE",
            _ => "EXCEL.EXE",
        }
        .into()];
        for (index, key) in office.keys.iter_mut().enumerate() {
            if index != NEXT_KEY && index != CLOCK_KEY {
                key.asset = format!("{}:{index}", name.to_ascii_lowercase());
                key.description = format!("{} in the foreground {name} application.", key.label);
            }
        }
        office
    }

    // Spotify precedes the final Themes selector in the default page order.
    pub fn spotify_page() -> PageConfig {
        page(
            "Spotify",
            [
                ("PLAY / PAUSE", "hotkey:space"),
                ("PREVIOUS", "media:previous"),
                ("NEXT TRACK", "media:next"),
                ("SHUFFLE", "hotkey:ctrl+s"),
                ("NEXT PAGE", "page:next"),
                ("REPEAT", "hotkey:ctrl+r"),
                ("SEARCH", "hotkey:ctrl+k"),
                ("LIBRARY", "hotkey:alt+shift+0"),
                ("LIKE", "hotkey:alt+shift+b"),
                ("QUEUE", "hotkey:alt+shift+q"),
                ("NOW PLAYING", "hotkey:alt+shift+j"),
                ("LIKED SONGS", "hotkey:alt+shift+s"),
                ("HOME", "hotkey:alt+shift+h"),
                ("CLOCK", "none"),
            ],
        )
    }

    // Returns a Windows utility page with editable, non-destructive defaults.
    pub fn utility_page() -> PageConfig {
        page(
            "Utility",
            [
                ("CLIPBOARD", "hotkey:super+v"),
                ("SEARCH", "hotkey:super+s"),
                ("EMOJI", "hotkey:super+."),
                ("SCREENSHOT", "hotkey:super+shift+s"),
                ("NEXT PAGE", "page:next"),
                ("EXPLORER", "hotkey:super+e"),
                ("DESKTOP", "hotkey:super+d"),
                ("TASK VIEW", "hotkey:super+tab"),
                ("SETTINGS", "hotkey:super+i"),
                (
                    "PROJECT",
                    "open:https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware",
                ),
                ("COPY", "hotkey:ctrl+c"),
                ("PASTE", "hotkey:ctrl+v"),
                ("UNDO", "hotkey:ctrl+z"),
                ("CLOCK", "none"),
            ],
        )
    }

    // current: active appearance, omitted because selecting it would not change the dock.
    // Returns twelve theme choices plus fixed navigation and clock positions.
    pub fn theme_selector(current: Theme) -> PageConfig {
        let mut keys = vec![
            KeyConfig {
                label: String::new(),
                action: "none".into(),
                description: String::new(),
                asset: String::new(),
            };
            KEY_COUNT
        ];
        keys[NEXT_KEY] = KeyConfig {
            label: "NEXT PAGE".into(),
            action: "page:next".into(),
            description: String::new(),
            asset: String::new(),
        };
        keys[CLOCK_KEY] = KeyConfig {
            label: "CLOCK".into(),
            action: "none".into(),
            description: String::new(),
            asset: String::new(),
        };
        for (position, theme) in THEME_KEYS
            .into_iter()
            .zip(Theme::ALL.into_iter().filter(|theme| *theme != current))
        {
            keys[position] = KeyConfig {
                label: theme.label().into(),
                action: format!("theme:{}", theme.slug()),
                description: String::new(),
                asset: String::new(),
            };
        }
        PageConfig {
            name: "Themes".into(),
            keys,
            executables: Vec::new(),
        }
    }

    // current: appearance used to rebuild the final theme selector page.
    // Errors: missing or relocated theme selector page.
    pub fn refresh_theme_selector(&mut self, current: Theme) -> Result<()> {
        anyhow::ensure!(
            self.pages.last().is_some_and(|page| page.name == "Themes"),
            "the final page must be the Themes selector"
        );
        let last = self.pages.len() - 1;
        self.pages[last] = Self::theme_selector(current);
        Ok(())
    }

    // path: JSON settings file to load, or a missing path for defaults.
    // Errors: malformed JSON, failed file access, or invalid layout.
    pub fn load(path: &Path) -> Result<Self> {
        let mut config = if path.exists() {
            serde_json::from_slice(
                &fs::read(path).with_context(|| format!("reading {}", path.display()))?,
            )
            .with_context(|| format!("parsing {}", path.display()))?
        } else {
            Self::default()
        };
        if !config
            .pages
            .iter()
            .any(|page| page.name.eq_ignore_ascii_case("Utility"))
        {
            config.pages.push(Self::utility_page());
        }
        if !config.pages.iter().any(|page| page.name == "Themes") {
            config.pages.push(Self::theme_selector(config.theme));
        }
        if !config
            .pages
            .iter()
            .any(|page| page.name.eq_ignore_ascii_case("Spotify"))
        {
            config.pages.push(Self::spotify_page());
        }
        for name in ["Word", "PowerPoint", "Excel"] {
            if !config.pages.iter().any(|page| page.name == name) {
                config.pages.push(Self::office_page(name));
            }
        }
        // Reorder whole pages in memory so old settings retain their custom keys/actions.
        config.pages.sort_by_key(|page| match page.name.as_str() {
            "Windows / media" => 0,
            "Utility" => 1,
            "Codex" => 2,
            "VS Code" => 3,
            "Spotify" => 4,
            "Word" => 5,
            "PowerPoint" => 6,
            "Excel" => 7,
            "Themes" => 9,
            _ => 8,
        });
        config.refresh_theme_selector(config.theme)?;
        // Older JSON files have no affinity metadata; keep application actions gated.
        for page in &mut config.pages {
            if page.executables.is_empty() {
                page.executables = match page.name.as_str() {
                    "Codex" => vec!["Codex.exe".into(), "ChatGPT.exe".into()],
                    "VS Code" => vec!["Code.exe".into(), "Code - Insiders.exe".into()],
                    "Spotify" => vec!["Spotify.exe".into()],
                    _ => Vec::new(),
                };
            }
        }
        crate::applications::load(path)?.apply(&mut config)?;
        config.validate()?;
        Ok(config)
    }

    // path: settings file to update or create after a theme selection.
    // Errors: invalid layout, serialization failure, backup failure, or failed write.
    pub fn save_selected_theme(&self, path: &Path) -> Result<()> {
        self.validate()?;
        if path.exists() {
            fs::copy(path, path.with_extension("json.bak"))
                .with_context(|| format!("backing up {}", path.display()))?;
        }
        fs::write(path, serde_json::to_vec_pretty(self)?)
            .with_context(|| format!("writing {}", path.display()))
    }

    // path: destination JSON settings file; existing files are preserved.
    // Errors: invalid layout, serialization failure, or failed file write.
    pub fn save_new(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let json = serde_json::to_vec_pretty(self)?;
        fs::write(path, json).with_context(|| format!("writing {}", path.display()))
    }

    // Errors: a missing page, wrong key count, or reassigned navigation/clock key.
    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            self.pages.last().is_some_and(|page| page.name == "Themes"),
            "the final page must be the Themes selector"
        );
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn selector_reaches_every_other_theme_without_reassigning_navigation_or_clock() {
        for current in Theme::ALL {
            let page = Config::theme_selector(current);
            let choices: HashSet<_> = THEME_KEYS
                .iter()
                .map(|position| {
                    page.keys[*position]
                        .action
                        .strip_prefix("theme:")
                        .and_then(Theme::from_slug)
                        .expect("theme selector action")
                        .slug()
                })
                .collect();
            assert_eq!(choices.len(), 12);
            assert!(!choices.contains(current.slug()));
            assert_eq!(page.keys[NEXT_KEY].action, "page:next");
            assert_eq!(page.keys[CLOCK_KEY].action, "none");
        }
    }

    #[test]
    fn older_three_page_settings_gain_utility_and_persistent_theme_selection() -> Result<()> {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("ulanzi-vibeware-config-{unique}.json"));
        let mut old = Config::default();
        old.pages
            .retain(|page| matches!(page.name.as_str(), "Windows / media" | "Codex" | "VS Code"));
        fs::write(&path, serde_json::to_vec(&old)?)?;

        let mut upgraded = Config::load(&path)?;
        assert_eq!(upgraded.pages.len(), 9);
        assert_eq!(upgraded.pages[1].name, "Utility");
        assert_eq!(upgraded.pages[4].name, "Spotify");
        assert_eq!(upgraded.pages[8].name, "Themes");
        upgraded.theme = Theme::CyberpunkNeon;
        upgraded.refresh_theme_selector(upgraded.theme)?;
        upgraded.save_selected_theme(&path)?;
        let saved = Config::load(&path)?;
        assert_eq!(saved.theme, Theme::CyberpunkNeon);
        assert_eq!(saved.pages.len(), 9);

        fs::remove_file(&path)?;
        fs::remove_file(path.with_extension("json.bak"))?;
        Ok(())
    }
}
