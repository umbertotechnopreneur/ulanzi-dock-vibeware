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

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

// Errors: malformed shortcut arguments or failure to start the controller.
// This GUI-subsystem helper never creates a console or opens the dock itself.
#[cfg(target_os = "windows")]
fn launch() -> anyhow::Result<()> {
    use anyhow::Context;
    use std::{
        env,
        os::windows::process::CommandExt,
        path::PathBuf,
        process::{Command, Stdio},
    };
    use ulanzi_dock_vibeware::paths;
    use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

    let arguments: Vec<_> = env::args_os().skip(1).collect();
    let (controller, working_directory, settings, background) = if arguments.is_empty() {
        let launcher = env::current_exe()?;
        let settings = paths::package_settings(&launcher)?
            .context("the no-argument launcher requires an MSIX package")?;
        let working_directory = settings
            .parent()
            .context("settings path has no parent folder")?
            .to_owned();

        (
            launcher.with_file_name("ulanzi-dock-vibeware.exe"),
            working_directory,
            settings,
            false,
        )
    } else if arguments.len() == 3 {
        (
            PathBuf::from(&arguments[0]),
            PathBuf::from(&arguments[1]),
            PathBuf::from(&arguments[2]),
            true,
        )
    } else {
        anyhow::bail!(
            "Expected executable, working directory and settings path. Use 'startup --enable' to repair the shortcut."
        );
    };

    let mut command = Command::new(controller);
    command
        .args(["run", "--config"])
        .arg(settings)
        .arg("--status-display")
        // Keep interactive setup while showing errors hidden by CREATE_NO_WINDOW.
        .env("ULANZI_DOCK_ERROR_DIALOG", "1")
        .current_dir(working_directory)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW);

    if background {
        command.arg("--background");
    }
    command.spawn()?;
    Ok(())
}

// Startup failures use a native dialog because this helper has no terminal.
fn main() -> std::process::ExitCode {
    #[cfg(target_os = "windows")]
    {
        if let Err(error) = launch() {
            use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
            let message: Vec<u16> = format!("UlanziDock could not start:\n{error:#}")
                .encode_utf16()
                .chain(Some(0))
                .collect();
            let title: Vec<u16> = "UlanziDock VibeWare"
                .encode_utf16()
                .chain(Some(0))
                .collect();
            unsafe {
                MessageBoxW(
                    std::ptr::null_mut(),
                    message.as_ptr(),
                    title.as_ptr(),
                    MB_OK | MB_ICONERROR,
                )
            };
            return std::process::ExitCode::FAILURE;
        }
        std::process::ExitCode::SUCCESS
    }
    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("The sign-in launcher is available only on Windows.");
        std::process::ExitCode::FAILURE
    }
}
