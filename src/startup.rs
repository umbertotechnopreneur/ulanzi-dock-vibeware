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

//! Inspect or change this app's Windows sign-in entry after an explicit startup command.
//! Packaged apps use Windows startup settings; portable apps use the owned shortcut
//! helper. This module does not install drivers, elevate privileges or start the dock.

use anyhow::Result;

#[cfg(windows)]
// enable: install or repair this user's hidden sign-in shortcut.
// disable: remove the owned sign-in shortcut.
// Errors: missing settings, unsafe paths, or shortcut setup failure.
pub fn manage(enable: bool, disable: bool) -> Result<()> {
    use anyhow::{ensure, Context};
    use std::{env, os::windows::process::CommandExt, path::PathBuf, process::Command};
    use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

    if crate::paths::is_packaged()? {
        if disable {
            println!("MSIX startup is controlled by Windows Startup apps. Disable UlanziDeck there.");
        } else if enable {
            println!("MSIX startup is registered by the package. Enable UlanziDeck in Windows Startup apps if it is disabled.");
        } else {
            println!(
                "Launch at Windows sign-in: managed by the MSIX package and Windows Startup apps."
            );
        }
        return Ok(());
    }

    let executable = env::current_exe().context("finding this executable")?;
    let launcher = executable.with_file_name("ulanzi-dock-launcher.exe");
    let workdir = env::current_dir().context("finding the working directory")?;
    let config = workdir.join("settings.json");
    ensure!(
        !enable || config.is_file(),
        "{} is missing; run 'init' in the intended working directory first",
        config.display()
    );
    ensure!(
        !enable || launcher.is_file(),
        "{} is missing; keep the Windows launcher beside the controller executable",
        launcher.display()
    );
    for path in [&executable, &workdir, &config] {
        let value = path.to_string_lossy();
        ensure!(
            !value.chars().any(|c| matches!(c, '"' | '\n' | '\r')),
            "a path contains characters unsafe for a Windows startup shortcut: {}",
            path.display()
        );
    }
    let powershell = PathBuf::from(env::var_os("SystemRoot").context("SystemRoot is unavailable")?)
        .join("System32/WindowsPowerShell/v1.0/powershell.exe");
    // Paths are process environment data, never interpolated into executable script text.
    // PowerShell only manages the shortcut. The native GUI launcher starts the controller
    // without a console; no shell or script runs at sign-in.
    let output = Command::new(&powershell)
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"])
        .arg(include_str!("startup.ps1"))
        .env("ULANZIDOCK_STARTUP_EXE", &executable)
        .env("ULANZIDOCK_STARTUP_LAUNCHER", &launcher)
        .env("ULANZIDOCK_STARTUP_WORKDIR", &workdir)
        .env("ULANZIDOCK_STARTUP_CONFIG", &config)
        .env(
            "ULANZIDOCK_STARTUP_MODE",
            if disable {
                "disable"
            } else if enable {
                "enable"
            } else {
                "status"
            },
        )
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .context("managing the Windows sign-in shortcut")?;
    ensure!(
        output.status.success(),
        "Windows sign-in shortcut operation failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}

#[cfg(not(windows))]
/// Reject Windows startup changes on unsupported platforms; flags have no side effects.
pub fn manage(_enable: bool, _disable: bool) -> Result<()> {
    anyhow::bail!("Windows sign-in startup is available only on Windows")
}
