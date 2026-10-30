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

//! Adapt console ownership, dimensions and attached-console input to Windows.
//! RAII guards keep redirected handles alive and restore temporary input modes.
//! No global keyboard hooks are installed and another process's terminal is never hidden.

// Use the visible viewport, rather than the scrollback buffer's width.
pub fn columns() -> Option<usize> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Console::{
            GetConsoleScreenBufferInfo, GetStdHandle, CONSOLE_SCREEN_BUFFER_INFO, STD_OUTPUT_HANDLE,
        };
        let output = unsafe { GetStdHandle(STD_OUTPUT_HANDLE) };
        let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
        if unsafe { GetConsoleScreenBufferInfo(output, &mut info) } != 0 {
            let width = i32::from(info.srWindow.Right) - i32::from(info.srWindow.Left) + 1;
            if width > 0 {
                return Some(width as usize);
            }
        }
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|&width| width > 0)
}

// Explorer creates a console containing only this process. An existing terminal
// contains its shell as well, and must not be paused by ordinary CLI commands.
pub fn owns_console() -> bool {
    #[cfg(target_os = "windows")]
    {
        use std::io::{self, IsTerminal};
        use windows_sys::Win32::System::Console::GetConsoleProcessList;

        let mut processes = [0u32; 2];
        let count = unsafe { GetConsoleProcessList(processes.as_mut_ptr(), 2) };
        count == 1
            && processes[0] == std::process::id()
            && io::stdin().is_terminal()
            && io::stdout().is_terminal()
    }
    #[cfg(not(target_os = "windows"))]
    false
}

/// Keep NUL handles alive for as long as a detached/background process needs them.
pub struct BackgroundConsole {
    #[cfg(target_os = "windows")]
    _input: std::fs::File,
    #[cfg(target_os = "windows")]
    _output: std::fs::File,
}

// Detach only the private console created for an Explorer/sign-in launch.
// Redirect standard handles to NUL so resident logging cannot fail after detaching.
// Errors: NUL handle creation, console detachment, or standard-handle redirection failure.
pub fn detach_if_owned() -> anyhow::Result<Option<BackgroundConsole>> {
    if owns_console() {
        detach().map(Some)
    } else {
        Ok(None)
    }
}

// Release this process's console without closing or hiding another process's terminal.
// Also supply NUL handles for a CreateNoWindow sign-in launch that has no console at all.
// Errors: NUL handle creation, console detachment, or standard-handle redirection failure.
pub fn detach() -> anyhow::Result<BackgroundConsole> {
    #[cfg(target_os = "windows")]
    {
        use anyhow::{ensure, Context};
        use std::{fs::OpenOptions, io, os::windows::io::AsRawHandle};
        use windows_sys::Win32::System::Console::{
            FreeConsole, GetConsoleProcessList, SetStdHandle, STD_ERROR_HANDLE, STD_INPUT_HANDLE,
            STD_OUTPUT_HANDLE,
        };
        let input = OpenOptions::new()
            .read(true)
            .open("NUL")
            .context("opening background input")?;
        let output = OpenOptions::new()
            .write(true)
            .open("NUL")
            .context("opening background output")?;
        let mut attached_process = 0u32;
        if unsafe { GetConsoleProcessList(&mut attached_process, 1) } != 0
            && unsafe { FreeConsole() } == 0
        {
            return Err(io::Error::last_os_error()).context("detaching the private console");
        }
        for (kind, handle) in [
            (STD_INPUT_HANDLE, input.as_raw_handle()),
            (STD_OUTPUT_HANDLE, output.as_raw_handle()),
            (STD_ERROR_HANDLE, output.as_raw_handle()),
        ] {
            ensure!(
                unsafe { SetStdHandle(kind, handle) } != 0,
                "redirecting a background console handle failed"
            );
        }
        Ok(BackgroundConsole {
            _input: input,
            _output: output,
        })
    }
    #[cfg(not(target_os = "windows"))]
    Ok(BackgroundConsole {})
}

/// The small set of attached-console inputs understood by the welcome menu.
pub enum MenuKey {
    Up,
    Down,
    Confirm,
    Cancel,
    Other,
}

/// Own a temporary console input mode and restore the original mode on drop.
pub struct MenuInput {
    #[cfg(target_os = "windows")]
    handle: windows_sys::Win32::Foundation::HANDLE,
    #[cfg(target_os = "windows")]
    original_mode: u32,
}

impl MenuInput {
    // Input is restricted to the attached console. Restore its original mode
    // before the controller installs its Ctrl+C handler.
    pub fn open() -> std::io::Result<Option<Self>> {
        #[cfg(target_os = "windows")]
        {
            use std::io::{self, IsTerminal};
            use windows_sys::Win32::System::Console::{
                FlushConsoleInputBuffer, GetConsoleMode, GetStdHandle, SetConsoleMode,
                ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT, STD_INPUT_HANDLE,
            };
            if !io::stdin().is_terminal() || std::env::var("TERM").is_ok_and(|term| term == "dumb")
            {
                return Ok(None);
            }
            let handle = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
            let mut original_mode = 0;
            if unsafe { GetConsoleMode(handle, &mut original_mode) } == 0 {
                return Ok(None);
            }
            let mode =
                original_mode & !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT | ENABLE_PROCESSED_INPUT);
            if unsafe { SetConsoleMode(handle, mode) } == 0 {
                return Err(io::Error::last_os_error());
            }
            let input = Self {
                handle,
                original_mode,
            };
            if unsafe { FlushConsoleInputBuffer(handle) } == 0 {
                return Err(io::Error::last_os_error());
            }
            Ok(Some(input))
        }
        #[cfg(not(target_os = "windows"))]
        Ok(None)
    }

    /// Block for the next console event and map it to a menu action.
    /// Ignore unrelated events; propagate native read failures without global capture.
    pub fn read_key(&mut self) -> std::io::Result<MenuKey> {
        #[cfg(target_os = "windows")]
        {
            use windows_sys::Win32::System::Console::{
                ReadConsoleInputW, INPUT_RECORD, KEY_EVENT, LEFT_CTRL_PRESSED, RIGHT_CTRL_PRESSED,
            };
            loop {
                let mut event = INPUT_RECORD::default();
                let mut count = 0;
                if unsafe { ReadConsoleInputW(self.handle, &mut event, 1, &mut count) } == 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if count == 0 || event.EventType != KEY_EVENT as u16 {
                    continue;
                }
                let key = unsafe { event.Event.KeyEvent };
                if key.bKeyDown == 0 {
                    continue;
                }
                let control = key.dwControlKeyState & (LEFT_CTRL_PRESSED | RIGHT_CTRL_PRESSED) != 0;
                return Ok(match key.wVirtualKeyCode {
                    0x26 => MenuKey::Up,
                    0x28 => MenuKey::Down,
                    0x0d => MenuKey::Confirm,
                    0x1b | 0x51 => MenuKey::Cancel,
                    0x43 if control => MenuKey::Cancel,
                    _ => MenuKey::Other,
                });
            }
        }
        #[cfg(not(target_os = "windows"))]
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "console key navigation is unavailable",
        ))
    }
}

impl Drop for MenuInput {
    /// Restore the attached console mode; cleanup cannot report errors to the caller.
    fn drop(&mut self) {
        #[cfg(target_os = "windows")]
        unsafe {
            windows_sys::Win32::System::Console::SetConsoleMode(self.handle, self.original_mode);
        }
    }
}

// Read only this console's input records, never global keyboard state.
pub fn wait_for_close() -> anyhow::Result<()> {
    #[cfg(target_os = "windows")]
    {
        use anyhow::Context;
        use std::io::{self, Write};
        use windows_sys::Win32::{
            Foundation::INVALID_HANDLE_VALUE,
            System::Console::{
                FlushConsoleInputBuffer, GetStdHandle, ReadConsoleInputW, INPUT_RECORD, KEY_EVENT,
                STD_INPUT_HANDLE,
            },
        };

        print!("\nPremi un tasto per chiudere...");
        io::stdout().flush()?;
        let input = unsafe { GetStdHandle(STD_INPUT_HANDLE) };
        anyhow::ensure!(
            !input.is_null() && input != INVALID_HANDLE_VALUE,
            "console input is unavailable"
        );
        if unsafe { FlushConsoleInputBuffer(input) } == 0 {
            return Err(io::Error::last_os_error()).context("clearing pending console input");
        }
        loop {
            let mut event = INPUT_RECORD::default();
            let mut count = 0;
            if unsafe { ReadConsoleInputW(input, &mut event, 1, &mut count) } == 0 {
                return Err(io::Error::last_os_error()).context("waiting for a console key");
            }
            if count > 0 && event.EventType == KEY_EVENT as u16 {
                let key = unsafe { event.Event.KeyEvent };
                if key.bKeyDown != 0 && !matches!(key.wVirtualKeyCode, 0x10..=0x12) {
                    println!();
                    break;
                }
            }
        }
    }
    Ok(())
}
