/* VBWR B
 *
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Modified with AI: OpenAI Codex; comments and readability; 2026-10-27
 * Human guidance: Umberto Giacobbi; AGENTS.md; readability/documentation request
 * Evidence: docs/CODE-REVIEW.md and current working-tree changes
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 *
 * VBWR E */

//! Define the current settings format, built-in pages and derived navigation state.
//! Loading validates user input before drawing or input dispatch. Generated index entries
//! and key availability stay in memory; this module does not send keys or write to HID.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const KEY_COUNT: usize = 14;
pub const NEXT_KEY: usize = 4;
pub const CLOCK_KEY: usize = 13;
pub const PAGE_INDEX_NAME: &str = "Page index";
pub const THEME_KEYS: [usize; 12] = [0, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "kebab-case")]
#[clap(rename_all = "kebab-case")]
/// Stable serialized theme identifiers; display labels may differ from their slugs.
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

    /// Return the stable settings/action token for this theme.
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

    /// Return the short human-facing theme name shown on the dock and in settings.
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

/// Label, action and optional pictogram for one physical key.
#[derive(Clone, Serialize, Deserialize)]
pub struct KeyConfig {
    pub label: String,
    pub action: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub asset: String,
}

/// Saved page contents with availability calculated only for the running controller.
#[derive(Clone, Serialize, Deserialize)]
pub struct PageConfig {
    pub name: String,
    pub enabled: bool,
    pub keys: Vec<KeyConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub executables: Vec<String>,
    #[serde(skip)]
    pub key_availability: Vec<bool>,
}

impl PageConfig {
    /// Check the derived availability for physical `index`; an absent entry is available.
    pub fn key_available(&self, index: usize) -> bool {
        self.key_availability.get(index).copied().unwrap_or(true)
    }
}

/// Preferences saved in settings.json, separate from application shortcuts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
/// User preferences for executable detection and automatic page following.
pub struct RuntimeSettings {
    pub detect_applications: bool,
    pub auto_switch: bool,
    pub focus_poll_seconds: u64,
}

impl Default for RuntimeSettings {
    /// Supply current defaults for a new profile; do not migrate an existing file.
    fn default() -> Self {
        Self {
            detect_applications: true,
            auto_switch: true,
            focus_poll_seconds: 1,
        }
    }
}

impl RuntimeSettings {
    /// Reject unsupported values before callers use this configuration.
    pub fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            (1..=60).contains(&self.focus_poll_seconds),
            "focus_poll_seconds must be 1-60"
        );
        Ok(())
    }
}

/// Saved dock settings plus availability calculated for the running controller.
#[derive(Clone, Serialize, Deserialize)]
/// Current on-disk preferences plus generated page and availability state.
pub struct Config {
    pub theme: Theme,
    pub brightness: u8,
    pub pages: Vec<PageConfig>,
    pub setup_completed: bool,
    /// Show the graphical welcome once. Missing flags start the new welcome flow.
    #[serde(default = "first_run_default")]
    pub is_first_run: bool,
    pub runtime: RuntimeSettings,
}

/// Initial percentage for a genuinely new profile, not a migration fallback.
fn default_brightness() -> u8 {
    100
}

/// New and not-yet-welcomed profiles must display About on their first GUI launch.
fn first_run_default() -> bool {
    true
}

// name: user-facing page title.
// keys: labels and actions in physical key order.
/// Build a regular page from `name` and exactly fourteen `(label, action)` slots.
fn page(name: &str, keys: [(&str, &str); KEY_COUNT]) -> PageConfig {
    PageConfig {
        name: name.into(),
        key_availability: Vec::new(),
        enabled: true,
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
    /// Supply current defaults for a new profile; do not migrate an existing file.
    fn default() -> Self {
        let mut config = Self {
            theme: Theme::DarkClassic,
            brightness: default_brightness(),
            setup_completed: false,
            is_first_run: true,
            runtime: RuntimeSettings::default(),
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
                Self::obs_page(),
                Self::theme_selector(Theme::DarkClassic),
            ],
        };
        config.refresh_page_index();
        config.refresh_key_availability();
        config
    }
}

impl Config {
    /// Decode the bundled OBS preset; malformed bundled data is a build/source defect.
    fn obs_page() -> PageConfig {
        serde_json::from_str(include_str!("../presets/obs-creator/page.json"))
            .expect("bundled OBS Creator page must be valid")
    }

    // Return the general Windows page, or the first enabled custom page.
    // Config validation guarantees that at least one page is enabled.
    pub fn home_page(&self) -> usize {
        self.pages
            .iter()
            .position(|page| page.enabled && page.name == "Windows / media")
            .or_else(|| self.pages.iter().position(|page| page.enabled))
            .unwrap_or(0)
    }

    // current: current page index. Disabled pages retain their actions but are skipped.
    pub fn next_enabled_page(&self, current: usize) -> usize {
        (1..=self.pages.len())
            .map(|offset| (current + offset) % self.pages.len())
            .find(|&index| self.pages[index].enabled && self.pages[index].name != PAGE_INDEX_NAME)
            .unwrap_or(current)
    }

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
                ("BRIGHT +", "brightness:up"),
                ("BRIGHT -", "brightness:down"),
                ("EMOJI", "hotkey:super+."),
                ("SCREENSHOT", "hotkey:super+shift+s"),
                ("NEXT PAGE", "page:next"),
                ("EXPLORER", "hotkey:super+e"),
                ("DESKTOP", "hotkey:super+d"),
                ("TASK VIEW", "hotkey:super+tab"),
                ("SETTINGS", "hotkey:super+i"),
                ("CLIPBOARD", "hotkey:super+v"),
                ("COPY", "hotkey:ctrl+c"),
                ("PASTE", "hotkey:ctrl+v"),
                ("SEARCH", "hotkey:super+s"),
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
            key_availability: Vec::new(),
            enabled: true,
            keys,
            executables: Vec::new(),
        }
    }

    // The generated index is separate from NEXT PAGE's regular page cycle.
    pub fn page_index_position(&self) -> usize {
        self.pages
            .iter()
            .position(|page| page.name == PAGE_INDEX_NAME)
            .expect("configuration includes a generated page index")
    }

    /// Rebuild direct page choices from the current ordered pages.
    /// Keep navigation/clock slots fixed; validation enforces the twelve-choice limit.
    fn refresh_page_index(&mut self) {
        let mut keys = vec![
            KeyConfig {
                label: String::new(),
                action: "none".into(),
                description: String::new(),
                asset: String::new(),
            };
            KEY_COUNT
        ];
        keys[NEXT_KEY].label = "NEXT PAGE".into();
        keys[NEXT_KEY].action = "page:next".into();
        keys[CLOCK_KEY].label = "PAGES".into();
        for (slot, target) in THEME_KEYS.into_iter().zip(
            self.pages
                .iter()
                .filter(|page| page.name != PAGE_INDEX_NAME),
        ) {
            let (label, icon) = match target.name.as_str() {
                "Windows / media" => ("WINDOWS", 0),
                "Utility" => ("UTILITY", 1),
                "Codex" => ("CODEX", 2),
                "VS Code" => ("VS CODE", 3),
                "Spotify" => ("SPOTIFY", 4),
                "Word" => ("WORD", 5),
                "PowerPoint" => ("POWERPOINT", 6),
                "Excel" => ("EXCEL", 7),
                "Themes" => ("THEMES", 8),
                _ => (target.name.as_str(), 9),
            };
            keys[slot] = KeyConfig {
                label: label.into(),
                action: format!("page:go:{}", target.name),
                description: format!("Open the {} page", target.name),
                asset: format!("index:{icon}"),
            };
        }
        let index = PageConfig {
            name: PAGE_INDEX_NAME.into(),
            enabled: true,
            keys,
            executables: Vec::new(),
            key_availability: Vec::new(),
        };
        if let Some(position) = self
            .pages
            .iter()
            .position(|page| page.name == PAGE_INDEX_NAME)
        {
            self.pages[position] = index;
        } else {
            self.pages.push(index);
        }
    }

    // Per-key availability is derived in memory; no disabled images or flags are saved.
    pub fn refresh_key_availability(&mut self) {
        let targets: Vec<_> = self
            .pages
            .iter()
            .map(|page| (page.name.clone(), page.enabled))
            .collect();
        for page in &mut self.pages {
            page.key_availability = page
                .keys
                .iter()
                .map(|key| match key.action.as_str() {
                    "brightness:up" => self.brightness < 100,
                    "brightness:down" => self.brightness > 0,
                    "none" if page.name == PAGE_INDEX_NAME => false,
                    action => action.strip_prefix("page:go:").map_or(true, |name| {
                        targets
                            .iter()
                            .any(|(target, enabled)| target == name && *enabled)
                    }),
                })
                .collect();
        }
    }

    // current: appearance used to rebuild the theme selector and its index entry.
    // Errors: missing theme selector page.
    pub fn refresh_theme_selector(&mut self, current: Theme) -> Result<()> {
        let position = self
            .pages
            .iter()
            .position(|page| page.name == "Themes")
            .context("the Themes selector page is missing")?;
        self.pages[position] = Self::theme_selector(current);
        self.refresh_page_index();
        self.refresh_key_availability();
        Ok(())
    }

    /// Read the current settings format. Missing files use the initial defaults.
    /// Old or incomplete settings are rejected rather than silently rewritten.
    pub fn load(path: &Path) -> Result<Self> {
        let mut config: Self = match fs::read(path) {
            Ok(bytes) => serde_json::from_slice(&bytes).with_context(|| {
                format!(
                    "parsing {}: use the current settings format",
                    path.display()
                )
            })?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(error) => {
                return Err(error).with_context(|| format!("reading {}", path.display()));
            }
        };

        config.validate()?;
        config.refresh_theme_selector(config.theme)?;
        crate::applications::load(path)?.apply(&mut config)?;
        config.refresh_key_availability();
        config.validate()?;

        Ok(config)
    }

    // Read only brightness so a running controller does not reload pages or shortcuts.
    // A missing file uses the initial default; an existing file must contain brightness.
    pub fn load_brightness(path: &Path) -> Result<u8> {
        #[derive(Deserialize)]
        struct Brightness {
            brightness: u8,
        }
        let level = match fs::read(path) {
            Ok(bytes) => {
                serde_json::from_slice::<Brightness>(&bytes)
                    .with_context(|| format!("reading brightness from {}", path.display()))?
                    .brightness
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => default_brightness(),
            Err(error) => return Err(error).context("reading brightness settings"),
        };
        anyhow::ensure!(level <= 100, "brightness must be 0-100");
        Ok(level)
    }

    // path: destination JSON settings file; existing files are preserved.
    // Errors: invalid layout, serialization failure, or failed file write.
    pub fn save_new(&self, path: &Path) -> Result<()> {
        self.validate()?;
        let json = serde_json::to_vec_pretty(self)?;
        ulanzi_dock_vibeware::persistence::create_new(path, &json)
    }

    // Errors: a missing page, wrong key count, or reassigned navigation/clock key.
    /// Reject unsupported values before callers use this configuration.
    pub fn validate(&self) -> Result<()> {
        self.runtime.validate()?;
        anyhow::ensure!(
            self.pages
                .iter()
                .filter(|page| page.name != PAGE_INDEX_NAME)
                .count()
                <= THEME_KEYS.len(),
            "the page index supports at most {} regular pages",
            THEME_KEYS.len()
        );
        anyhow::ensure!(self.brightness <= 100, "brightness must be 0-100");
        anyhow::ensure!(
            self.pages
                .iter()
                .any(|page| page.enabled && page.name != PAGE_INDEX_NAME),
            "enable at least one page"
        );
        anyhow::ensure!(
            self.pages.iter().any(|page| page.name == "Themes"),
            "the Themes selector page is missing"
        );
        let mut names = std::collections::HashSet::new();
        for page in &self.pages {
            anyhow::ensure!(
                names.insert(page.name.as_str()),
                "duplicate page name '{}'",
                page.name
            );
        }

        for name in [
            "Windows / media",
            "Utility",
            "Codex",
            "VS Code",
            "Spotify",
            "Word",
            "PowerPoint",
            "Excel",
            "Themes",
        ] {
            anyhow::ensure!(
                names.contains(name),
                "page '{name}' is missing; use the current settings format"
            );
        }

        for page in &self.pages {
            anyhow::ensure!(
                page.keys.len() == KEY_COUNT,
                "page '{}' must have 14 keys",
                page.name
            );
            for (index, key) in page.keys.iter().enumerate() {
                let action = key.action.as_str();
                let valid = if matches!(action, "page:next" | "brightness:up" | "brightness:down") {
                    Ok(())
                } else if let Some(target) = action.strip_prefix("page:go:") {
                    anyhow::ensure!(
                        names.contains(target),
                        "unknown page target '{target}' on '{}' key {index}",
                        page.name
                    );
                    Ok(())
                } else if let Some(theme) = action.strip_prefix("theme:") {
                    Theme::from_slug(theme)
                        .context("unknown theme action")
                        .map(|_| ())
                } else {
                    crate::actions::validate(action)
                };
                valid.with_context(|| format!("invalid action on '{}' key {index}", page.name))?;
                if !key.asset.is_empty() {
                    crate::applications::validate_asset(&key.asset).with_context(|| {
                        format!("invalid artwork on '{}' key {index}", page.name)
                    })?;
                }
            }
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
    fn invalid_custom_actions_and_assets_fail_before_controller_start() {
        for action in [
            "theme:missing",
            "page:go:missing",
            "hotkey:ctrl+unsupported",
            "open:file:///x",
        ] {
            let mut config = Config::default();
            config.pages[0].keys[0].action = action.into();
            assert!(config.validate().is_err(), "accepted {action}");
        }
        let mut config = Config::default();
        config.pages[0].keys[0].asset = "index:12".into();
        assert!(config.validate().is_err());
    }

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
    fn incomplete_settings_are_rejected_without_adding_pages() {
        let mut config = Config::default();
        config.pages.retain(|page| page.name != "Utility");

        assert!(config.validate().is_err());
        assert!(!config.pages.iter().any(|page| page.name == "Utility"));
    }

    #[test]
    fn page_index_capacity_is_enforced_without_changing_defaults() {
        let mut config = Config::default();
        assert_eq!(
            config
                .pages
                .iter()
                .filter(|page| page.name != PAGE_INDEX_NAME)
                .count(),
            10
        );
        for index in 0..2 {
            let mut custom = config.pages[0].clone();
            custom.name = format!("Custom {index}");
            config.pages.insert(config.pages.len() - 2, custom);
        }
        assert!(config.validate().is_ok());
        let mut extra = config.pages[0].clone();
        extra.name = "Overflow".into();
        config.pages.push(extra);
        assert!(config.validate().is_err());
    }
}
