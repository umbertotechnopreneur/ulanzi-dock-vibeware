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

pub enum MenuKey {
    Up,
    Down,
    Confirm,
    Cancel,
    Other,
}

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
