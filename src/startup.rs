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

use anyhow::Result;

#[cfg(windows)]
pub fn manage(enable: bool, disable: bool) -> Result<()> {
    use anyhow::{ensure, Context};
    use std::{env, fs, path::PathBuf};

    const MARKER: &str = "rem UlanziDock VibeWare startup entry";
    let appdata = PathBuf::from(env::var_os("APPDATA").context("APPDATA is unavailable")?);
    let startup =
        appdata.join("Microsoft/Windows/Start Menu/Programs/Startup/UlanziDock VibeWare.cmd");
    let existing = if startup.exists() {
        let content = fs::read_to_string(&startup)
            .with_context(|| format!("reading {}", startup.display()))?;
        ensure!(
            content.lines().any(|line| line == MARKER),
            "{} exists but was not created by UlanziDock; preserving it",
            startup.display()
        );
        true
    } else {
        false
    };
    if disable {
        if existing {
            fs::remove_file(&startup).with_context(|| format!("removing {}", startup.display()))?;
            println!("Disabled launch at Windows sign-in for this user.");
        } else {
            println!("Launch at Windows sign-in is already disabled for this user.");
        }
        return Ok(());
    }
    if !enable {
        println!(
            "Launch at Windows sign-in: {}",
            if existing { "enabled" } else { "disabled" }
        );
        if existing {
            println!("Entry: {}", startup.display());
        }
        return Ok(());
    }

    let executable = env::current_exe().context("finding this executable")?;
    let workdir = env::current_dir().context("finding the working directory")?;
    let config = workdir.join("settings.json");
    ensure!(
        config.is_file(),
        "{} is missing; run 'init' in the intended working directory first",
        config.display()
    );
    for path in [&executable, &workdir, &config] {
        let value = path.to_string_lossy();
        ensure!(
            !value
                .chars()
                .any(|c| matches!(c, '"' | '%' | '!' | '\n' | '\r')),
            "a path contains characters unsafe for a Windows startup command file: {}",
            path.display()
        );
    }
    let content = format!(
        "@echo off\r\n{MARKER}\r\ncd /d \"{}\"\r\n\"{}\" run --config \"{}\" --status-display\r\n",
        workdir.display(),
        executable.display(),
        config.display()
    );
    fs::write(&startup, content).with_context(|| format!("writing {}", startup.display()))?;
    println!("Enabled launch at Windows sign-in for this user.");
    println!("Executable: {}", executable.display());
    println!("Entry: {}", startup.display());
    Ok(())
}

#[cfg(not(windows))]
pub fn manage(_enable: bool, _disable: bool) -> Result<()> {
    anyhow::bail!("Windows sign-in startup is available only on Windows")
}
