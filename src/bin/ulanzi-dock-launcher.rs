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

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

// Errors: malformed shortcut arguments or failure to start the controller.
// This GUI-subsystem helper never creates a console or opens the dock itself.
#[cfg(target_os = "windows")]
fn launch() -> std::io::Result<()> {
    use std::{
        env, io,
        os::windows::process::CommandExt,
        process::{Command, Stdio},
    };
    use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;
    let arguments: Vec<_> = env::args_os().skip(1).collect();
    if arguments.len() != 3 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Expected executable, working directory and settings path. Use 'startup --enable' to repair the shortcut."));
    }
    Command::new(&arguments[0])
        .args(["run", "--background", "--config"])
        .arg(&arguments[2])
        .arg("--status-display")
        .current_dir(&arguments[1])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    Ok(())
}

// Startup failures use a native dialog because this helper has no terminal.
fn main() -> std::process::ExitCode {
    #[cfg(target_os = "windows")]
    {
        if let Err(error) = launch() {
            use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
            let message: Vec<u16> = format!("UlanziDock could not start:\n{error}")
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
