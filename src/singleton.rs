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

#[cfg(target_os = "windows")]
mod windows {
    use anyhow::{bail, Context, Result};
    use std::{io, mem::size_of, ptr};
    use windows_sys::Win32::{
        Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE, WAIT_ABANDONED, WAIT_OBJECT_0},
        System::{
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
            Threading::{
                CreateMutexW, OpenProcess, ReleaseMutex, TerminateProcess, WaitForSingleObject,
                INFINITE, PROCESS_TERMINATE,
            },
        },
    };

    struct Handle(HANDLE);

    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe { CloseHandle(self.0) };
        }
    }

    pub(super) fn replace_existing() -> Result<()> {
        // Serialize simultaneous takeovers. The mutex is released before opening HID.
        let name: Vec<u16> = "Local\\UlanziDockVibeWareTakeover"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let mutex = unsafe { CreateMutexW(ptr::null(), 0, name.as_ptr()) };
        if mutex.is_null() {
            return Err(io::Error::last_os_error()).context("creating singleton takeover mutex");
        }
        let mutex = Handle(mutex);
        match unsafe { WaitForSingleObject(mutex.0, INFINITE) } {
            WAIT_OBJECT_0 | WAIT_ABANDONED => {}
            _ => return Err(io::Error::last_os_error()).context("waiting for singleton takeover"),
        }
        let result = terminate_other_instances();
        if unsafe { ReleaseMutex(mutex.0) } == 0 {
            return Err(io::Error::last_os_error()).context("releasing singleton takeover mutex");
        }
        result
    }

    fn terminate_other_instances() -> Result<()> {
        let executable = std::env::current_exe()?
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(io::Error::last_os_error()).context("listing running processes");
        }
        let snapshot = Handle(snapshot);
        let mut entry = PROCESSENTRY32W {
            dwSize: size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if unsafe { Process32FirstW(snapshot.0, &mut entry) } == 0 {
            return Err(io::Error::last_os_error()).context("reading running processes");
        }
        let mut replaced = 0;
        loop {
            let end = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..end]);
            let pid = entry.th32ProcessID;
            if pid != std::process::id() && name.eq_ignore_ascii_case(&executable) {
                println!("Singleton: existing {name} process PID {pid}; terminating it...");
                // SYNCHRONIZE is a standard access right (0x0010_0000).
                let process = unsafe { OpenProcess(PROCESS_TERMINATE | 0x0010_0000, 0, pid) };
                if process.is_null() {
                    let error = io::Error::last_os_error();
                    if error.raw_os_error() != Some(87) {
                        return Err(error).with_context(|| format!("opening existing PID {pid}"));
                    }
                    println!("Singleton: PID {pid} already exited.");
                } else {
                    let process = Handle(process);
                    if unsafe { WaitForSingleObject(process.0, 0) } != WAIT_OBJECT_0 {
                        if unsafe { TerminateProcess(process.0, 1) } == 0 {
                            let error = io::Error::last_os_error();
                            if unsafe { WaitForSingleObject(process.0, 0) } != WAIT_OBJECT_0 {
                                return Err(error)
                                    .with_context(|| format!("terminating existing PID {pid}"));
                            }
                        }
                        if unsafe { WaitForSingleObject(process.0, 5000) } != WAIT_OBJECT_0 {
                            bail!("existing PID {pid} did not exit within five seconds");
                        }
                    }
                    println!("Singleton: previous PID {pid} has exited.");
                    replaced += 1;
                }
            }
            if unsafe { Process32NextW(snapshot.0, &mut entry) } == 0 {
                break;
            }
        }
        if replaced == 0 {
            println!("Singleton: no other {executable} process is running.");
        } else {
            println!("Singleton: replaced {replaced} previous instance(s).");
        }
        Ok(())
    }
}

pub fn replace_existing() -> Result<()> {
    #[cfg(target_os = "windows")]
    return windows::replace_existing();

    #[cfg(not(target_os = "windows"))]
    Ok(())
}
