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

//! Keep one controller per user session on Windows and Linux.
//! Linux also gives the standalone settings window a separate instance guard.

use anyhow::Result;

/// Held on the main thread until the controller, HID worker and tray have stopped.
/// A second launch must never kill an editor or another installation by filename.
pub struct Instance {
    #[cfg(target_os = "linux")]
    _lock: std::fs::File,
    #[cfg(target_os = "windows")]
    handle: windows_sys::Win32::Foundation::HANDLE,
    #[cfg(target_os = "windows")]
    _thread: std::marker::PhantomData<std::rc::Rc<()>>,
}

#[cfg(target_os = "windows")]
impl Drop for Instance {
    /// Release mutex ownership and close its handle on the acquiring thread.
    fn drop(&mut self) {
        // SAFETY: acquire owns the mutex on this thread; Instance cannot be sent elsewhere.
        unsafe {
            windows_sys::Win32::System::Threading::ReleaseMutex(self.handle);
            windows_sys::Win32::Foundation::CloseHandle(self.handle);
        }
    }
}

/// Try to acquire the session guard without waiting.
/// An existing controller returns None. Errors acquiring the guard are reported.
pub fn acquire() -> Result<Option<Instance>> {
    #[cfg(target_os = "windows")]
    {
        use anyhow::Context;
        use windows_sys::Win32::{
            Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT},
            System::Threading::{CreateMutexW, WaitForSingleObject},
        };
        let name: Vec<u16> = "Local\\UlanziDockVibeWareController"
            .encode_utf16()
            .chain(Some(0))
            .collect();
        // SAFETY: the name is terminated; the returned handle is checked and owned below.
        let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
        if handle.is_null() {
            return Err(std::io::Error::last_os_error()).context("creating controller mutex");
        }
        let wait = unsafe { WaitForSingleObject(handle, 0) };
        match wait {
            WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(Some(Instance {
                handle,
                _thread: std::marker::PhantomData,
            })),
            _ => {
                let error = std::io::Error::last_os_error();
                unsafe { CloseHandle(handle) };
                if wait == WAIT_TIMEOUT {
                    return Ok(None);
                }
                Err(error).context("acquiring the controller mutex")
            }
        }
    }
    #[cfg(target_os = "linux")]
    return linux_lock("ulanzideck-controller.lock");
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    Ok(Some(Instance {}))
}

#[cfg(target_os = "linux")]
pub fn acquire_configurator() -> Result<Option<Instance>> {
    linux_lock("ulanzideck-configurator.lock")
}

#[cfg(target_os = "linux")]
fn linux_lock(name: &str) -> Result<Option<Instance>> {
    use anyhow::{ensure, Context};
    use std::{
        env,
        fs::{self, OpenOptions, TryLockError},
        os::unix::fs::OpenOptionsExt,
        path::PathBuf,
    };

    // The systemd service and terminals use the same directory even when a
    // terminal did not inherit the desktop session's environment.
    let user = unsafe { libc::geteuid() };
    let system_runtime = PathBuf::from(format!("/run/user/{user}"));
    let directory = Some(system_runtime).filter(|path| path.is_dir()).or_else(|| env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute() && path.is_dir()));
    let directory = match directory {
        Some(directory) => directory,
        None => {
            let home = env::var_os("HOME").context("locating the user instance lock directory")?;
            let directory = PathBuf::from(home).join(".local/state/ulanzideck");
            fs::create_dir_all(&directory).context("creating the user instance lock directory")?;
            directory
        }
    };
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_NOCTTY)
        .open(directory.join(name))
        .context("opening the UlanziDeck instance lock")?;
    ensure!(lock.metadata()?.is_file(), "The instance lock must be a regular file.");
    match lock.try_lock() {
        Ok(()) => Ok(Some(Instance { _lock: lock })),
        Err(TryLockError::WouldBlock) => Ok(None),
        Err(TryLockError::Error(error)) => Err(error).context("locking the UlanziDeck instance"),
    }
}
