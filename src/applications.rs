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

use crate::model::{Config, CLOCK_KEY, NEXT_KEY};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{collections::HashSet, fs, path::Path};

#[derive(Clone, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RuntimeSettings {
    pub auto_switch: bool,
    pub focus_poll_seconds: u64,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            auto_switch: true,
            focus_poll_seconds: 1,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub version: u32,
    #[serde(default)]
    pub runtime: RuntimeSettings,
    pub applications: Vec<Application>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Application {
    pub name: String,
    pub page: String,
    pub executables: Vec<String>,
    #[serde(default)]
    pub commands: Vec<Command>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub key: usize,
    pub name: String,
    pub description: String,
    pub asset: String,
    // Omitting action keeps the shortcut already customized in settings.json.
    pub action: Option<String>,
}

impl Catalog {
    pub fn apply(self, config: &mut Config) -> Result<()> {
        anyhow::ensure!(self.version == 1, "unsupported applications.yaml version");
        anyhow::ensure!(
            (1..=60).contains(&self.runtime.focus_poll_seconds),
            "YAML focus_poll_seconds must be 1-60"
        );
        config.runtime = self.runtime;
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

pub fn validate_asset(asset: &str) -> Result<(&str, usize)> {
    let (family, index) = asset
        .split_once(':')
        .context("asset must be family:key, e.g. spotify:3")?;
    anyhow::ensure!(
        matches!(
            family,
            "windows" | "codex" | "vscode" | "utility" | "spotify"
        ),
        "unknown asset family '{family}'"
    );
    let index: usize = index.parse().context("asset key must be a number")?;
    anyhow::ensure!(index < CLOCK_KEY, "asset key must be 0-12");
    Ok((family, index))
}

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
