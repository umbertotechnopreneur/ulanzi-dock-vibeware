/* VBWR B
 *
 * Project: UlanziDock VibeWare version
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

//! Keep one controller instance per Windows user session for its entire lifetime.
//! The owned mutex is released when the guard drops. This module never kills competing
//! processes and does not arbitrate physical USB ownership across Windows sessions.

use anyhow::Result;

/// Held on the main thread until the controller, HID worker and tray have stopped.
/// A second launch must never kill an editor or another installation by filename.
pub struct Instance {
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
/// An existing controller or native mutex error is reported; other platforms return
/// a no-op guard because this Windows-specific ownership rule is not implemented there.
pub fn acquire() -> Result<Instance> {
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
            WAIT_OBJECT_0 | WAIT_ABANDONED => Ok(Instance {
                handle,
                _thread: std::marker::PhantomData,
            }),
            _ => {
                let error = std::io::Error::last_os_error();
                unsafe { CloseHandle(handle) };
                if wait == WAIT_TIMEOUT {
                    anyhow::bail!("UlanziDock is already running in this Windows session. Use its tray menu to configure, restart or exit it before starting another controller.");
                }
                Err(error).context("acquiring the controller mutex")
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    Ok(Instance {})
}
