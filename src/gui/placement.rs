/* VBWR B
 *
 * Project: UlanziDeck
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Created with AI: OpenAI Codex; Windows placement; 2026-10-27
 * Modified with AI: OpenAI Codex; comments and readability; 2026-10-27
 * Human guidance: Umberto Giacobbi; AGENTS.md; readability/documentation request
 * Evidence: docs/CODE-REVIEW.md and current working-tree changes
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 *
 * VBWR E */

//! Remember and restore each Windows dialog's monitor, size, position and display scale.
//! Saved normal bounds use Windows workspace coordinates; DPI rescales geometry instead
//! of changing the user's display settings. Other platforms keep their toolkit behavior.
//! No controller preferences or HID operations belong in this module.

#[derive(Clone, Copy)]
/// Stable state keys let different dialog types retain independent placements.
pub(super) enum DialogKind {
    About,
    Configuration,
}

#[cfg(target_os = "windows")]
mod windows {
    use super::DialogKind;
    use anyhow::{ensure, Context, Result};
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use serde::{Deserialize, Serialize};
    use std::{
        cell::RefCell,
        collections::BTreeMap,
        fs,
        mem::size_of,
        path::{Path, PathBuf},
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, POINT, RECT},
        Graphics::Gdi::{
            EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, MonitorFromPoint,
            MonitorFromWindow, DISPLAY_DEVICEW, HDC, HMONITOR, MONITORINFOEXW,
            MONITOR_DEFAULTTONEAREST, MONITOR_DEFAULTTOPRIMARY,
        },
        UI::{
            HiDpi::GetDpiForWindow,
            WindowsAndMessaging::{
                GetCursorPos, GetWindowPlacement, GetWindowRect, SetForegroundWindow,
                SetWindowPlacement, SetWindowPos, EDD_GET_DEVICE_INTERFACE_NAME, HWND_TOP,
                SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, SW_SHOWMAXIMIZED,
                SW_SHOWNORMAL, WINDOWPLACEMENT, WPF_RESTORETOMAXIMIZED,
            },
        },
    };

    thread_local! {
        static STATE_PATH: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    }

    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    /// Normal (unmaximized) bounds and display identity captured for one dialog.
    struct Placement {
        monitor: String,
        dpi: u32,
        // Offsets use WINDOWPLACEMENT workspace coordinates, never screen coordinates.
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        maximized: bool,
    }

    type SavedWindows = BTreeMap<String, Placement>;

    /// Map `kind` to the stable JSON entry used across process restarts.
    fn key(kind: DialogKind) -> &'static str {
        match kind {
            DialogKind::About => "about",
            DialogKind::Configuration => "configuration",
        }
    }

    /// Choose the sidecar beside `settings` for windows created on this UI thread.
    pub(crate) fn initialize(settings: &Path) {
        STATE_PATH.with(|slot| *slot.borrow_mut() = Some(settings.with_extension("windows.json")));
    }

    /// Retrieve the thread-local sidecar path; initialization must precede window use.
    fn state_path() -> Result<PathBuf> {
        STATE_PATH
            .with(|slot| slot.borrow().clone())
            .context("window state path is unavailable")
    }

    /// Read `path` without mutation; a missing file means first use, not corruption.
    fn load(path: &Path) -> Result<SavedWindows> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(BTreeMap::new())
            }
            Err(error) => return Err(error).context("reading window positions"),
        };
        decode(Some(&bytes))
    }

    /// Parse optional saved bytes and reject impossible dimensions/DPI values.
    /// Returning an error lets callers preserve the invalid file for inspection.
    fn decode(bytes: Option<&[u8]>) -> Result<SavedWindows> {
        let Some(bytes) = bytes else {
            return Ok(BTreeMap::new());
        };
        let saved: SavedWindows =
            serde_json::from_slice(bytes).context("parsing window positions")?;
        for placement in saved.values() {
            ensure!(
                (48..=960).contains(&placement.dpi)
                    && (1..=100_000).contains(&placement.width)
                    && (1..=100_000).contains(&placement.height),
                "invalid saved window dimensions or DPI"
            );
        }
        Ok(saved)
    }

    /// Borrow the native handle of an already created Slint window.
    /// Handle-unavailable errors let the caller defer until the event loop creates it.
    fn hwnd(window: &slint::Window) -> Result<HWND> {
        let handle = window.window_handle();
        let native = handle
            .window_handle()
            .context("accessing the native dialog")?;
        let RawWindowHandle::Win32(native) = native.as_raw() else {
            anyhow::bail!("the dialog has no Win32 window handle");
        };
        Ok(native.hwnd.get() as HWND)
    }

    /// Read monitor/work rectangles and display name for `monitor`; report Win32 failure.
    fn monitor_info(monitor: HMONITOR) -> Result<MONITORINFOEXW> {
        let mut info = MONITORINFOEXW::default();
        info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
        // SAFETY: the structure includes the extended monitor-name buffer.
        ensure!(
            unsafe { GetMonitorInfoW(monitor, &mut info.monitorInfo) } != 0,
            "reading the monitor work area"
        );
        Ok(info)
    }

    /// Decode a fixed-size, NUL-terminated Win32 UTF-16 buffer without its padding.
    fn wide_string(value: &[u16]) -> String {
        let end = value.iter().position(|&c| c == 0).unwrap_or(value.len());
        String::from_utf16_lossy(&value[..end])
    }

    /// Prefer the physical interface identity in `info`; display numbering may change.
    fn monitor_id(info: &MONITORINFOEXW) -> String {
        let mut device = DISPLAY_DEVICEW {
            cb: size_of::<DISPLAY_DEVICEW>() as u32,
            ..Default::default()
        };
        // Prefer the monitor interface identity: DISPLAY1/DISPLAY2 can change after unplugging.
        if unsafe {
            EnumDisplayDevicesW(
                info.szDevice.as_ptr(),
                0,
                &mut device,
                EDD_GET_DEVICE_INTERFACE_NAME,
            )
        } != 0
        {
            let id = wide_string(&device.DeviceID);
            if !id.is_empty() {
                return id;
            }
        }
        wide_string(&info.szDevice)
    }

    /// Borrowed search state valid only during synchronous monitor enumeration.
    struct MonitorSearch<'a> {
        id: &'a str,
        found: Option<MONITORINFOEXW>,
    }

    /// Win32 enumeration callback: inspect each monitor and update the borrowed search.
    /// `data` must point to a live MonitorSearch for the duration of enumeration.
    unsafe extern "system" fn find_monitor(
        monitor: HMONITOR,
        _: HDC,
        _: *mut RECT,
        data: LPARAM,
    ) -> i32 {
        // SAFETY: EnumDisplayMonitors calls synchronously while the search value is alive.
        let search = &mut *(data as *mut MonitorSearch<'_>);
        if let Ok(info) = monitor_info(monitor) {
            if monitor_id(&info) == search.id {
                search.found = Some(info);
            }
        }
        1
    }

    /// Resolve `saved` by identity, otherwise use mouse/primary monitor.
    /// The boolean says whether the saved display matched and its offsets can be reused.
    fn target_monitor(saved: Option<&Placement>) -> Result<(MONITORINFOEXW, bool)> {
        if let Some(saved) = saved {
            let mut search = MonitorSearch {
                id: &saved.monitor,
                found: None,
            };
            ensure!(
                unsafe {
                    EnumDisplayMonitors(
                        ptr::null_mut(),
                        ptr::null(),
                        Some(find_monitor),
                        &mut search as *mut MonitorSearch<'_> as LPARAM,
                    )
                } != 0,
                "listing displays"
            );
            if let Some(info) = search.found {
                return Ok((info, true));
            }
        }
        let mut pointer = POINT::default();
        let monitor = unsafe {
            if GetCursorPos(&mut pointer) != 0 {
                MonitorFromPoint(pointer, MONITOR_DEFAULTTONEAREST)
            } else {
                MonitorFromPoint(POINT::default(), MONITOR_DEFAULTTOPRIMARY)
            }
        };
        Ok((monitor_info(monitor)?, false))
    }

    /// Read normal bounds and show state for `hwnd` in workspace coordinates.
    fn native_placement(hwnd: HWND) -> Result<WINDOWPLACEMENT> {
        let mut placement = WINDOWPLACEMENT {
            length: size_of::<WINDOWPLACEMENT>() as u32,
            ..Default::default()
        };
        ensure!(
            unsafe { GetWindowPlacement(hwnd, &mut placement) } != 0,
            "reading window placement"
        );
        Ok(placement)
    }

    /// Save a visible `window` under `kind`; report errors without blocking its close.
    pub(crate) fn remember(window: &slint::Window, kind: DialogKind) {
        if !window.is_visible() {
            return;
        }
        if let Err(error) = save(window, kind) {
            eprintln!("Could not save window position: {error:#}");
        }
    }

    /// Capture native bounds and merge `kind` into the latest locked sidecar snapshot.
    /// Invalid existing state or filesystem errors are returned, not overwritten.
    fn save(window: &slint::Window, kind: DialogKind) -> Result<()> {
        let hwnd = hwnd(window)?;
        let position = native_placement(hwnd)?;
        let info = monitor_info(unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) })?;
        let rect = position.rcNormalPosition;
        let path = state_path()?;
        // Merge other dialogs' saved entries rather than keeping a stale process snapshot.
        // Invalid existing state is preserved and reported, never silently overwritten.
        crate::settings::update_auxiliary_file(&path, |original| {
            let mut saved = decode(original)?;
            saved.insert(
                key(kind).into(),
                Placement {
                    monitor: monitor_id(&info),
                    dpi: unsafe { GetDpiForWindow(hwnd) }.max(96),
                    x: rect.left.saturating_sub(info.monitorInfo.rcMonitor.left),
                    y: rect.top.saturating_sub(info.monitorInfo.rcMonitor.top),
                    width: rect.right - rect.left,
                    height: rect.bottom - rect.top,
                    maximized: position.showCmd == SW_SHOWMAXIMIZED as u32
                        || position.flags & WPF_RESTORETOMAXIMIZED != 0,
                },
            );
            let mut bytes = serde_json::to_vec_pretty(&saved)?;
            bytes.push(b'\n');
            Ok(bytes)
        })
    }

    /// Convert a signed offset/size between DPI scales with overflow-safe arithmetic.
    fn scaled(value: i32, dpi: u32, previous_dpi: u32) -> i32 {
        (i64::from(value) * i64::from(dpi) / i64::from(previous_dpi))
            .clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
    }

    /// Position and focus `window`; restore saved `kind` only when `restore` is true.
    /// Clamp normal bounds to the chosen work area. Native geometry failures propagate;
    /// Windows focus refusal is logged separately and never triggers another resize.
    /// `primary_welcome` ignores saved placement and centers the initial welcome
    /// within the primary display's work area using the dialog's preferred size.
    pub(crate) fn present(
        window: &slint::Window,
        kind: DialogKind,
        restore: bool,
        primary_welcome: bool,
    ) -> Result<()> {
        let hwnd = hwnd(window)?;
        if restore {
            let states = state_path()
                .and_then(|path| load(&path))
                .unwrap_or_else(|error| {
                    eprintln!("Could not restore window positions: {error:#}");
                    BTreeMap::new()
                });
            let saved = if primary_welcome {
                None
            } else {
                states.get(key(kind))
            };
            let (info, same_monitor) = if primary_welcome {
                (
                    monitor_info(unsafe {
                        MonitorFromWindow(ptr::null_mut(), MONITOR_DEFAULTTOPRIMARY)
                    })?,
                    false,
                )
            } else {
                target_monitor(saved)?
            };
            let work = info.monitorInfo.rcWork;
            // Move first so Windows/Slint adopt the destination monitor's DPI.
            ensure!(
                unsafe {
                    SetWindowPos(
                        hwnd,
                        ptr::null_mut(),
                        work.left,
                        work.top,
                        0,
                        0,
                        SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
                    )
                } != 0,
                "moving the dialog to its monitor"
            );
            let dpi = unsafe { GetDpiForWindow(hwnd) }.max(96);
            let margin = scaled(16, dpi, 96);
            let available_width = (work.right - work.left).max(1);
            let available_height = (work.bottom - work.top).max(1);
            let max_width = (available_width - 2 * margin).max(1);
            let max_height = (available_height - 2 * margin).max(1);
            if let Some(saved) = saved {
                let minimum_width = if matches!(kind, DialogKind::Configuration) { scaled(820, dpi, 96) } else { 1 };
                let minimum_height = if matches!(kind, DialogKind::Configuration) { scaled(900, dpi, 96) } else { 1 };
                let width = scaled(saved.width, dpi, saved.dpi).max(minimum_width).clamp(1, max_width);
                let height = scaled(saved.height, dpi, saved.dpi).max(minimum_height).clamp(1, max_height);
                let x = if same_monitor {
                    scaled(saved.x, dpi, saved.dpi)
                } else {
                    (available_width - width) / 2
                };
                let y = if same_monitor {
                    scaled(saved.y, dpi, saved.dpi)
                } else {
                    (available_height - height) / 2
                };
                // Get/SetWindowPlacement use workspace coordinates. Do not pass this
                // rectangle to SetWindowPos, whose coordinates include the taskbar offset.
                let left = info.monitorInfo.rcMonitor.left + x.clamp(0, available_width - width);
                let top = info.monitorInfo.rcMonitor.top + y.clamp(0, available_height - height);
                let mut placement = native_placement(hwnd)?;
                placement.flags = 0;
                placement.showCmd = SW_SHOWNORMAL as u32;
                placement.rcNormalPosition = RECT {
                    left,
                    top,
                    right: left + width,
                    bottom: top + height,
                };
                ensure!(
                    unsafe { SetWindowPlacement(hwnd, &placement) } != 0,
                    "restoring window placement"
                );
                window.set_maximized(saved.maximized);
            } else {
                let mut frame = RECT::default();
                ensure!(
                    unsafe { GetWindowRect(hwnd, &mut frame) } != 0,
                    "reading dialog bounds"
                );
                let width = (frame.right - frame.left).clamp(1, max_width);
                let height = (frame.bottom - frame.top).clamp(1, max_height);
                ensure!(
                    unsafe {
                        SetWindowPos(
                            hwnd,
                            HWND_TOP,
                            work.left + (available_width - width) / 2,
                            work.top + (available_height - height) / 2,
                            width,
                            height,
                            SWP_SHOWWINDOW,
                        )
                    } != 0,
                    "centring the dialog"
                );
            }
        }
        // Focus denial is not a geometry failure and must not trigger another restore.
        if unsafe { SetForegroundWindow(hwnd) } == 0 {
            eprintln!("Windows did not allow the dialog to take foreground focus.");
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub(super) use windows::{initialize, present, remember};

#[cfg(not(target_os = "windows"))]
/// Other platforms do not persist native monitor placement; `settings` is unused.
pub(super) fn initialize(_settings: &std::path::Path) {}

#[cfg(not(target_os = "windows"))]
/// Other platforms retain toolkit placement; this hook performs no filesystem writes.
pub(super) fn remember(_window: &slint::Window, _kind: DialogKind) {}
