/* VBWR B
 *
 * Project: UlanziDeck
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

//! Read the application catalogue and merge its mappings into the current configuration.
//! This layer validates names, key positions and artwork references. It neither launches
//! applications nor writes the catalogue during ordinary preference saves.

use crate::model::{Config, CLOCK_KEY, NEXT_KEY};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
/// Versioned collection of application-to-page mappings from one YAML document.
pub struct Catalog {
    pub version: u32,
    pub applications: Vec<Application>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
/// One named app mapping: executable filenames select its existing page.
pub struct Application {
    pub name: String,
    pub page: String,
    pub executables: Vec<String>,
    #[serde(default)]
    pub commands: Vec<Command>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
/// Metadata and optional action replacement for a physical key on a mapped page.
pub struct Command {
    pub key: usize,
    pub name: String,
    pub description: String,
    pub asset: String,
    // Omitting action keeps the shortcut already customized in settings.json.
    pub action: Option<String>,
}

impl Catalog {
    /// Merge this catalogue into `config` in memory, preserving actions omitted by YAML.
    /// Errors identify duplicate mappings, unknown pages, reserved keys or bad artwork.
    /// The caller discards the partially updated configuration if validation fails.
    pub fn apply(self, config: &mut Config) -> Result<()> {
        anyhow::ensure!(self.version == 1, "unsupported applications.yaml version");
        let mut assigned_pages = HashSet::new();
        let mut assigned_processes = HashSet::new();
        for application in self.applications {
            anyhow::ensure!(
                !application.name.trim().is_empty(),
                "application name is empty"
            );
            let page = config
                .pages
                .iter_mut()
                .find(|page| page.name.eq_ignore_ascii_case(&application.page))
                .with_context(|| {
                    format!(
                        "application '{}' refers to missing page '{}'",
                        application.name, application.page
                    )
                })?;
            anyhow::ensure!(
                assigned_pages.insert(page.name.to_ascii_lowercase()),
                "duplicate application page '{}'",
                page.name
            );
            for executable in &application.executables {
                anyhow::ensure!(
                    !executable.trim().is_empty() && !executable.contains(['/', '\\']),
                    "use an executable filename, not a path: {executable}"
                );
                anyhow::ensure!(
                    assigned_processes.insert(executable.to_ascii_lowercase()),
                    "executable '{executable}' belongs to more than one page"
                );
            }
            page.executables = application.executables;
            let mut assigned_keys = HashSet::new();
            for command in application.commands {
                anyhow::ensure!(
                    command.key < page.keys.len()
                        && command.key != NEXT_KEY
                        && command.key != CLOCK_KEY,
                    "application '{}' has invalid command key {}",
                    application.name,
                    command.key
                );
                anyhow::ensure!(
                    assigned_keys.insert(command.key),
                    "duplicate command key {} on '{}'",
                    command.key,
                    page.name
                );
                anyhow::ensure!(
                    !command.name.trim().is_empty() && !command.description.trim().is_empty(),
                    "command name and description must not be empty"
                );
                validate_asset(&command.asset)?;
                let key = &mut page.keys[command.key];
                key.label = command.name;
                key.description = command.description;
                key.asset = command.asset;
                if let Some(action) = command.action {
                    key.action = action;
                }
            }
        }
        Ok(())
    }
}

/// Parse `asset` as `family:key` and check that the renderer supports both parts.
/// Return the borrowed family and physical icon index; no image is loaded here.
pub fn validate_asset(asset: &str) -> Result<(&str, usize)> {
    let (family, index) = asset
        .split_once(':')
        .context("asset must be family:key, e.g. spotify:3")?;
    anyhow::ensure!(
        matches!(
            family,
            "windows"
                | "codex"
                | "vscode"
                | "utility"
                | "spotify"
                | "word"
                | "powerpoint"
                | "excel"
                | "obs"
                | "index"
        ),
        "unknown asset family '{family}'"
    );
    let index: usize = index.parse().context("asset key must be a number")?;
    anyhow::ensure!(index < CLOCK_KEY, "asset key must be 0-12");
    anyhow::ensure!(
        family != "index" || index < 10,
        "page index asset must be 0-9"
    );
    Ok((family, index))
}

/// An explicit editor request can create the current catalogue, never replace one.
pub fn create_default(path: &Path) -> Result<()> {
    ulanzi_dock_vibeware::persistence::create_new(path, include_bytes!("../applications.yaml"))
}

/// Read the catalogue beside `settings`, or use the bundled current default if absent.
/// Propagate file/parse errors; an invalid existing catalogue is never replaced.
pub fn load(settings: &Path) -> Result<Catalog> {
    let path = settings.with_file_name("applications.yaml");
    let yaml = if path.exists() {
        fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?
    } else {
        include_str!("../applications.yaml").to_owned()
    };
    serde_yaml_ng::from_str(&yaml)
        .with_context(|| format!("parsing application catalog {}", path.display()))
}
