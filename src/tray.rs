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

//! Coordinate controller lifetime and expose Windows tray and desktop operations.
//! `Control` is shared across threads; native tray resources stay on their owning thread.
//! Shell helpers open links/files or restart this executable. Device I/O and Slint dialog
//! construction remain in their own modules.

#[cfg(not(target_os = "windows"))]
use anyhow::Result;
#[cfg(target_os = "windows")]
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
#[cfg(not(target_os = "windows"))]
use std::sync::Arc;

#[cfg(target_os = "windows")]
const EXIT: u8 = 1;
const RESTART: u8 = 2;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
/// Connection episodes shown by the tray; Waiting is distinct from a lost connection.
pub enum ConnectionState {
    Waiting = 0,
    Connected = 1,
    Disconnected = 2,
}

/// Shared stop/restart flags and process-local settings serialization.
/// This object carries decisions between UI/controller threads, never the HID handle.
pub struct Control {
    running: AtomicBool,
    action: AtomicU8,
    background: AtomicBool,
    settings: std::sync::Mutex<()>,
    connection: AtomicU8,
    #[cfg(target_os = "windows")]
    tray_window: AtomicUsize,
}

impl Control {
    /// Start in the running/waiting state with no pending restart or tray window.
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(true),
            action: AtomicU8::new(0),
            background: AtomicBool::new(false),
            settings: std::sync::Mutex::new(()),
            connection: AtomicU8::new(ConnectionState::Waiting as u8),
            #[cfg(target_os = "windows")]
            tray_window: AtomicUsize::new(0),
        }
    }

    /// Read the shared stop signal without owning the controller thread.
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    // connection: USB lifecycle transition, published once to the native tray.
    pub fn set_connection(&self, connection: ConnectionState) {
        let previous = self.connection.swap(connection as u8, Ordering::SeqCst);
        if previous == connection as u8 {
            return;
        }
        #[cfg(target_os = "windows")]
        {
            windows::notify_connection(self, connection, previous);
        }
    }

    #[cfg(target_os = "windows")]
    /// Decode the connection state atomically for native tray presentation.
    fn connection(&self) -> ConnectionState {
        match self.connection.load(Ordering::SeqCst) {
            1 => ConnectionState::Connected,
            2 => ConnectionState::Disconnected,
            _ => ConnectionState::Waiting,
        }
    }

    /// Ask the controller to leave its loop; this does not terminate a process.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// Report whether the accepted lifecycle request is a restart.
    pub fn restart_requested(&self) -> bool {
        self.action.load(Ordering::SeqCst) == RESTART
    }

    /// Distinguish an explicit tray/UI lifecycle request from other shutdown paths.
    pub fn stopped_from_tray(&self) -> bool {
        self.action.load(Ordering::SeqCst) != 0
    }

    /// Mark this launch as background so failures can use an appropriate UI.
    pub fn enter_background(&self) {
        self.background.store(true, Ordering::SeqCst);
    }

    /// Report whether this launch is running without an interactive console.
    pub fn is_background(&self) -> bool {
        self.background.load(Ordering::SeqCst)
    }

    // Request a restart only after the configurator has saved successfully.
    pub fn request_restart(&self) {
        self.request(RESTART);
    }

    // Serialize configurator saves with theme changes made on the physical dock.
    // Errors: a previous settings operation panicked while holding the lock.
    pub fn lock_settings(&self) -> anyhow::Result<std::sync::MutexGuard<'_, ()>> {
        self.settings
            .lock()
            .map_err(|_| anyhow::anyhow!("The settings lock is unavailable."))
    }

    // action: either EXIT or RESTART, delivered before the controller stops.
    fn request(&self, action: u8) {
        // The first menu command wins, including during a nested dialog loop.
        let _ = self
            .action
            .compare_exchange(0, action, Ordering::SeqCst, Ordering::SeqCst);
        self.stop();
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use super::{ConnectionState, Control, EXIT, RESTART};
    use anyhow::{ensure, Context, Result};
    use image::imageops::FilterType;
    use std::{
        env, io,
        mem::size_of,
        os::windows::ffi::OsStrExt,
        os::windows::process::CommandExt,
        path::PathBuf,
        process::{Command, Stdio},
        ptr,
        sync::{mpsc, Arc},
        thread::{self, JoinHandle},
    };
    use windows_sys::Win32::{
        Foundation::{
            CloseHandle, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WAIT_OBJECT_0, WPARAM,
        },
        Graphics::Gdi::{
            CreateBitmap, CreateDIBSection, DeleteObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
            DIB_RGB_COLORS,
        },
        System::{
            LibraryLoader::GetModuleHandleW,
            Threading::{OpenProcess, WaitForSingleObject, CREATE_NO_WINDOW},
        },
        UI::{
            HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
            Shell::{
                ShellExecuteW, Shell_NotifyIconGetRect, Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE,
                NIF_INFO, NIF_REALTIME, NIF_SHOWTIP, NIF_TIP, NIIF_INFO, NIIF_NOSOUND,
                NIIF_RESPECT_QUIET_TIME, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETFOCUS,
                NIM_SETVERSION, NIN_SELECT, NOTIFYICONDATAW, NOTIFYICONDATAW_0,
                NOTIFYICONIDENTIFIER, NOTIFYICON_VERSION_4,
            },
            WindowsAndMessaging::{
                AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
                DestroyIcon, DestroyMenu, DestroyWindow, DispatchMessageW, EndMenu, GetCursorPos,
                GetMessageW, GetSystemMetrics, GetWindowLongPtrW, MessageBoxW, PostMessageW,
                PostQuitMessage, RegisterClassW, RegisterWindowMessageW, SetForegroundWindow,
                SetWindowLongPtrW, TrackPopupMenu, TranslateMessage, UnregisterClassW,
                CREATESTRUCTW, GWLP_USERDATA, HICON, HMENU, ICONINFO, MB_ICONERROR, MB_OK,
                MB_SETFOREGROUND, MF_SEPARATOR, MF_STRING, MSG, SM_CXSMICON, SM_CYSMICON,
                SW_SHOWNORMAL, TPM_BOTTOMALIGN, TPM_NONOTIFY, TPM_RETURNCMD, TPM_RIGHTALIGN,
                TPM_RIGHTBUTTON, WM_APP, WM_CLOSE, WM_CONTEXTMENU, WM_DESTROY, WM_ENDSESSION,
                WM_NCCREATE, WM_NCDESTROY, WM_NULL, WM_QUERYENDSESSION, WNDCLASSW,
            },
        },
    };

    const CALLBACK: u32 = WM_APP + 1;
    const CONNECTION_CHANGED: u32 = WM_APP + 2;
    const OPEN_CLI: i32 = 10;
    const RESTART_APP: i32 = 11;
    const GITHUB: i32 = 12;
    const MANIFESTO: i32 = 13;
    const ABOUT: i32 = 14;
    const EXIT_APP: i32 = 15;
    const RESTART_PARENT: &str = "ULANZIDOCK_VIBEWARE_RESTART_PARENT";

    /// Owned icon handle, destroyed exactly once when its guard is dropped.
    struct Icon(HICON);

    impl Drop for Icon {
        /// Release the native resource owned by this guard on its tray thread.
        fn drop(&mut self) {
            unsafe { DestroyIcon(self.0) };
        }
    }

    /// Owned native menu tree, released with its child menus on drop.
    struct Menu(HMENU);

    impl Drop for Menu {
        /// Release the native resource owned by this guard on its tray thread.
        fn drop(&mut self) {
            unsafe { DestroyMenu(self.0) };
        }
    }

    /// Registration identity retained until the hidden tray window has been destroyed.
    struct WindowClass {
        name: Vec<u16>,
        instance: HINSTANCE,
    }

    impl Drop for WindowClass {
        /// Release the native resource owned by this guard on its tray thread.
        fn drop(&mut self) {
            unsafe { UnregisterClassW(self.name.as_ptr(), self.instance) };
        }
    }

    /// Hidden tray window; cleanup removes the shell icon before destroying the HWND.
    struct Window(HWND);

    impl Drop for Window {
        /// Release the native resource owned by this guard on its tray thread.
        fn drop(&mut self) {
            unsafe {
                Shell_NotifyIconW(NIM_DELETE, &notification(self.0, ptr::null_mut()));
                DestroyWindow(self.0);
            }
        }
    }

    /// Tray-thread resources borrowed by the native window procedure during dispatch.
    struct State {
        control: Arc<Control>,
        icon: Icon,
        menu: Menu,
        taskbar_created: u32,
    }

    /// Handle to the tray thread; drop requests shutdown and waits for resource cleanup.
    pub struct Tray {
        window: usize,
        worker: Option<JoinHandle<()>>,
    }

    impl Tray {
        // control: shared stop/restart state for the HID loop and tray thread.
        // Errors: native resource creation, icon registration, or thread startup failure.
        pub fn start(control: Arc<Control>) -> Result<Self> {
            let (ready, receiver) = mpsc::sync_channel(1);
            let worker = thread::Builder::new()
                .name("ulanzi-system-tray".into())
                .spawn(move || {
                    if let Err(error) = message_loop(Arc::clone(&control), &ready) {
                        control.stop();
                        let _ = ready.send(Err(format!("{error:#}")));
                        if control.is_background() {
                            show_error(&format!("The system tray could not continue:\n{error:#}"));
                        } else {
                            eprintln!("System tray failed: {error:#}");
                        }
                    }
                })
                .context("starting the Windows system tray thread")?;
            match receiver.recv().context("waiting for the system tray")? {
                Ok(window) => Ok(Self {
                    window,
                    worker: Some(worker),
                }),
                Err(error) => {
                    let _ = worker.join();
                    anyhow::bail!("{error}")
                }
            }
        }
    }

    impl Drop for Tray {
        /// Release the native resource owned by this guard on its tray thread.
        fn drop(&mut self) {
            // Close on the owning UI thread, including any open menu or owned dialog.
            unsafe { PostMessageW(self.window as HWND, WM_CLOSE, 0, 0) };
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
        }
    }

    // text: UTF-8 text to encode as a nul-terminated Windows string.
    fn wide(text: &str) -> Vec<u16> {
        text.encode_utf16().chain(Some(0)).collect()
    }

    // window: hidden top-level owner of this notification icon.
    // icon: the runtime derivative of the embedded VibeWare PNG.
    fn notification(window: HWND, icon: HICON) -> NOTIFYICONDATAW {
        let mut tip = [0u16; 128];
        let label = wide(&format!(
            "UlanziDeck v{}",
            env!("CARGO_PKG_VERSION")
        ));
        tip[..label.len()].copy_from_slice(&label);
        NOTIFYICONDATAW {
            cbSize: size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: window,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_SHOWTIP,
            uCallbackMessage: CALLBACK,
            hIcon: icon,
            szTip: tip,
            Anonymous: NOTIFYICONDATAW_0 {
                uVersion: NOTIFYICON_VERSION_4,
            },
            ..Default::default()
        }
    }

    // control: shared connection state and the tray's currently registered window.
    pub(super) fn notify_connection(control: &Control, connection: ConnectionState, previous: u8) {
        let window = control.tray_window.load(super::Ordering::SeqCst);
        if window != 0 {
            if unsafe {
                PostMessageW(
                    window as HWND,
                    CONNECTION_CHANGED,
                    connection as usize,
                    previous as isize,
                )
            } == 0 {
                eprintln!("Could not queue the dock connection notification");
            }
        }
    }

    /// Replace the previous notice immediately. Never open or activate a window.
    /// Real-time notices are discarded rather than queued for a later, stale state.
    fn show_connection_notification(window: HWND, state: &State, reconnected: bool) {
        let mut data = notification(window, state.icon.0);
        data.uFlags = NIF_INFO;
        // Empty szInfo removes the previous balloon before publishing the new state.
        if unsafe { Shell_NotifyIconW(NIM_MODIFY, &data) } == 0 {
            eprintln!("Could not clear the previous dock connection notification");
        }
        let (title, message) = match state.control.connection() {
            ConnectionState::Disconnected => (
                "UlanziDeck - Dock disconnected",
                "Your dock is disconnected. UlanziDeck will reconnect automatically. No action is needed.",
            ),
            ConnectionState::Connected if reconnected => (
                "UlanziDeck - Dock reconnected",
                "Your dock is reconnected and ready to use.",
            ),
            _ => return,
        };
        let title = wide(title);
        let message = wide(message);
        data.szInfoTitle[..title.len()].copy_from_slice(&title);
        data.szInfo[..message.len()].copy_from_slice(&message);
        data.uFlags = NIF_INFO | NIF_REALTIME;
        data.dwInfoFlags = NIIF_INFO | NIIF_NOSOUND | NIIF_RESPECT_QUIET_TIME;
        if unsafe { Shell_NotifyIconW(NIM_MODIFY, &data) } == 0 {
            eprintln!("Could not show the dock connection notification");
        }
    }

    // window: owner of notification icon ID 1.
    // state: current connection status and icon handle, owned by the tray thread.
    fn update_connection_tip(window: HWND, state: &State) {
        let status = match state.control.connection() {
            ConnectionState::Waiting => "Waiting for dock",
            ConnectionState::Connected => "Dock connected",
            ConnectionState::Disconnected => "Dock disconnected - reconnecting",
        };
        let mut data = notification(window, state.icon.0);
        data.uFlags = NIF_TIP | NIF_SHOWTIP;
        data.szTip.fill(0);
        let label = wide(&format!(
            "UlanziDeck v{}\n{status}",
            env!("CARGO_PKG_VERSION")
        ));
        let length = label.len().min(data.szTip.len() - 1);
        data.szTip[..length].copy_from_slice(&label[..length]);
        if unsafe { Shell_NotifyIconW(NIM_MODIFY, &data) } == 0 {
            eprintln!("Could not update the dock connection tray tooltip");
        }
    }

    // window: hidden top-level owner of this notification icon.
    // icon: native icon handle kept alive by State.
    // Errors: Windows cannot add the icon or enable its modern callback behavior.
    fn add_icon(window: HWND, icon: HICON) -> Result<()> {
        let data = notification(window, icon);
        ensure!(
            unsafe { Shell_NotifyIconW(NIM_ADD, &data) } != 0,
            "Windows could not add the UlanziDeck notification icon"
        );
        if unsafe { Shell_NotifyIconW(NIM_SETVERSION, &data) } == 0 {
            // Leave no partially registered icon behind if startup needs to retry.
            unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
            anyhow::bail!("Windows could not configure the notification icon");
        }
        Ok(())
    }

    // Errors: embedded PNG decoding or native bitmap/icon creation failure.
    /// Convert the embedded logo to a native HICON with explicit bitmap ownership.
    /// Return an owning guard; decoding, allocation and Win32 creation errors propagate.
    fn logo_icon() -> Result<Icon> {
        let width = unsafe { GetSystemMetrics(SM_CXSMICON) }.max(16) as u32;
        let height = unsafe { GetSystemMetrics(SM_CYSMICON) }.max(16) as u32;
        let source = image::load_from_memory(include_bytes!(concat!(
            env!("OUT_DIR"),
            "/ulanzi-dock-icon-32.png"
        )))
        .context("decoding the UlanziDeck tray icon")?;
        let resized = source.resize(width, height, FilterType::Nearest).to_rgba8();
        let mut rgba = image::RgbaImage::new(width, height);
        image::imageops::overlay(
            &mut rgba,
            &resized,
            ((width - resized.width()) / 2) as i64,
            ((height - resized.height()) / 2) as i64,
        );
        let mut bgra = Vec::with_capacity((width * height * 4) as usize);
        for pixel in rgba.pixels() {
            // Windows expects premultiplied BGRA in the 32-bit icon DIB.
            let [r, g, b, a] = pixel.0;
            let premultiply = |color: u8| (u16::from(color) * u16::from(a) / 255) as u8;
            bgra.extend_from_slice(&[premultiply(b), premultiply(g), premultiply(r), a]);
        }
        let info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32),
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut pixels = ptr::null_mut();
        let bitmap = unsafe {
            CreateDIBSection(
                ptr::null_mut(),
                &info,
                DIB_RGB_COLORS,
                &mut pixels,
                ptr::null_mut(),
                0,
            )
        };
        ensure!(!bitmap.is_null(), "creating the tray color bitmap failed");
        unsafe { ptr::copy_nonoverlapping(bgra.as_ptr(), pixels.cast::<u8>(), bgra.len()) };
        // A zero-filled AND mask lets the DIB alpha channel supply transparency.
        let mask_bytes = vec![0u8; (width as usize).div_ceil(16) * 2 * height as usize];
        let mask = unsafe {
            CreateBitmap(
                width as i32,
                height as i32,
                1,
                1,
                mask_bytes.as_ptr().cast(),
            )
        };
        if mask.is_null() {
            unsafe { DeleteObject(bitmap) };
            anyhow::bail!("creating the tray transparency bitmap failed");
        }
        let icon = unsafe {
            CreateIconIndirect(&ICONINFO {
                fIcon: 1,
                xHotspot: 0,
                yHotspot: 0,
                hbmMask: mask,
                hbmColor: bitmap,
            })
        };
        unsafe {
            DeleteObject(bitmap);
            DeleteObject(mask);
        }
        ensure!(!icon.is_null(), "creating the VibeWare tray icon failed");
        Ok(Icon(icon))
    }

    // Errors: native menu creation or insertion failure.
    /// Create the native command menu; transfer ownership only after every item succeeds.
    fn menu() -> Result<Menu> {
        let handle = unsafe { CreatePopupMenu() };
        ensure!(!handle.is_null(), "creating the tray menu failed");
        let menu = Menu(handle);
        for (command, label) in [
            (OPEN_CLI, "Open CLI"),
            (RESTART_APP, "Restart"),
            (0, ""),
            (GITHUB, "GitHub"),
            (MANIFESTO, "VibeWare manifesto"),
            (ABOUT, "About"),
            (0, ""),
            (EXIT_APP, "Exit"),
        ] {
            let label = wide(label);
            ensure!(
                unsafe {
                    AppendMenuW(
                        handle,
                        if command == 0 {
                            MF_SEPARATOR
                        } else {
                            MF_STRING
                        },
                        command as usize,
                        label.as_ptr(),
                    )
                } != 0,
                "adding a tray menu entry failed"
            );
        }
        Ok(menu)
    }

    // control: shared lifecycle state used to stop the controller.
    // ready: reports startup success only after the notification icon is installed.
    // Errors: resource initialization or native message-loop failure.
    fn message_loop(
        control: Arc<Control>,
        ready: &mpsc::SyncSender<Result<usize, String>>,
    ) -> Result<()> {
        // Notification rectangles use physical screen coordinates. Make menus and dialogs use
        // the same coordinate system, including on monitors with different Windows scaling.
        ensure!(
            !unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) }
                .is_null(),
            "enabling per-monitor DPI awareness for the tray failed"
        );
        let taskbar_created = unsafe { RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()) };
        ensure!(
            taskbar_created != 0,
            "registering the taskbar restart event failed"
        );
        let state = Box::new(State {
            control,
            icon: logo_icon()?,
            menu: menu()?,
            taskbar_created,
        });
        let instance = unsafe { GetModuleHandleW(ptr::null()) };
        let name = wide(&format!("UlanziDockVibeWareTray-{}", std::process::id()));
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: name.as_ptr(),
            ..Default::default()
        };
        ensure!(
            unsafe { RegisterClassW(&class) } != 0,
            "registering the tray window failed"
        );
        let class = WindowClass { name, instance };
        // A hidden top-level window receives TaskbarCreated broadcasts after Explorer restarts.
        let window = unsafe {
            CreateWindowExW(
                0,
                class.name.as_ptr(),
                wide("UlanziDeck").as_ptr(),
                0,
                0,
                0,
                0,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                instance,
                (&*state as *const State).cast(),
            )
        };
        ensure!(!window.is_null(), "creating the tray window failed");
        let window = Window(window);
        // Explorer's notification area may not be ready when startup entries run.
        // Wait briefly for the shell before declaring tray initialization a failure.
        let tray_started = std::time::Instant::now();
        loop {
            match add_icon(window.0, state.icon.0) {
                Ok(()) => break,
                Err(error) if tray_started.elapsed() >= std::time::Duration::from_secs(30) => {
                    return Err(error);
                }
                Err(_) => thread::sleep(std::time::Duration::from_millis(250)),
            }
        }
        state
            .control
            .tray_window
            .store(window.0 as usize, super::Ordering::SeqCst);
        update_connection_tip(window.0, &state);
        ready
            .send(Ok(window.0 as usize))
            .context("reporting tray startup")?;
        loop {
            let mut message = MSG::default();
            match unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) } {
                -1 => return Err(io::Error::last_os_error()).context("reading tray messages"),
                0 => break,
                _ => unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                },
            }
        }
        Ok(())
    }

    // window: hidden owner window created on the tray thread.
    // message: Windows notification or lifecycle message.
    // wparam: notification coordinates or message-specific value.
    // lparam: callback event, or the initial CREATESTRUCTW during WM_NCCREATE.
    // Safety: Windows supplies these values; State outlives the window and its nested dialogs.
    unsafe extern "system" fn window_proc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        if message == WM_NCCREATE {
            let creation = &*(lparam as *const CREATESTRUCTW);
            SetWindowLongPtrW(window, GWLP_USERDATA, creation.lpCreateParams as isize);
        }
        let state = GetWindowLongPtrW(window, GWLP_USERDATA) as *const State;
        if let Some(state) = state.as_ref() {
            if message == state.taskbar_created {
                if let Err(error) = add_icon(window, state.icon.0) {
                    show_error(&format!("Could not restore the tray icon:\n{error:#}"));
                }
                update_connection_tip(window, state);
                return 0;
            }
            if message == CONNECTION_CHANGED {
                update_connection_tip(window, state);
                // A fast reconnect may overtake a queued disconnect. Do not show
                // an obsolete notice, and retain the transition's previous state.
                if wparam == state.control.connection() as usize {
                    show_connection_notification(
                        window,
                        state,
                        lparam == ConnectionState::Disconnected as isize,
                    );
                }
                return 0;
            }
            if message == CALLBACK {
                // Version 4 packs the event in the low word and the icon ID in the high word.
                if (lparam as u32 >> 16) != 1 {
                    return 0;
                }
                let event = (lparam as u32) & 0xffff;
                if event == WM_CONTEXTMENU {
                    show_menu(window, state, wparam);
                } else if event == NIN_SELECT || event == (NIN_SELECT | 1) {
                    if let Err(error) = crate::gui::show_about() {
                        eprintln!("Could not open About from the tray: {error:#}");
                    }
                }
                return 0;
            }
            if message == WM_ENDSESSION && wparam != 0 {
                state.control.request(EXIT);
            }
        }
        match message {
            WM_QUERYENDSESSION => 1,
            WM_CLOSE => {
                EndMenu();
                DestroyWindow(window);
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            WM_NCDESTROY => {
                if let Some(state) = state.as_ref() {
                    state.control.tray_window.store(0, super::Ordering::SeqCst);
                }
                SetWindowLongPtrW(window, GWLP_USERDATA, 0);
                DefWindowProcW(window, message, wparam, lparam)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }

    // window: owner of notification icon ID 1.
    // coordinates: fallback screen point packed by notification API version 4.
    fn icon_anchor(window: HWND, coordinates: Option<WPARAM>) -> POINT {
        let identifier = NOTIFYICONIDENTIFIER {
            cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
            hWnd: window,
            uID: 1,
            ..Default::default()
        };
        let mut rect = RECT::default();
        if unsafe { Shell_NotifyIconGetRect(&identifier, &mut rect) } >= 0 {
            return POINT {
                x: rect.right,
                y: rect.top,
            };
        }
        let mut point = coordinates.map_or(POINT { x: -1, y: -1 }, |coordinates| POINT {
            x: (coordinates as u16 as i16) as i32,
            y: ((coordinates >> 16) as u16 as i16) as i32,
        });
        if point.x == -1 && point.y == -1 {
            unsafe { GetCursorPos(&mut point) };
        }
        point
    }

    // window: native owner required to dismiss the context menu correctly.
    // state: menu handles and lifecycle signals for the running controller.
    // coordinates: signed screen coordinates packed by notification API version 4.
    fn show_menu(window: HWND, state: &State, coordinates: WPARAM) {
        let point = icon_anchor(window, Some(coordinates));
        let selected = unsafe {
            SetForegroundWindow(window);
            let selected = TrackPopupMenu(
                state.menu.0,
                TPM_RETURNCMD | TPM_NONOTIFY | TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_RIGHTALIGN,
                point.x,
                point.y,
                0,
                window,
                ptr::null(),
            );
            PostMessageW(window, WM_NULL, 0, 0);
            // About hands focus to the Slint UI thread. Returning focus to the
            // notification area here can race that user-requested activation.
            if selected != ABOUT {
                Shell_NotifyIconW(NIM_SETFOCUS, &notification(window, state.icon.0));
            }
            selected
        };
        let result = match selected {
            OPEN_CLI => open_cli(),
            RESTART_APP => {
                state.control.request(RESTART);
                Ok(())
            }
            EXIT_APP => {
                state.control.request(EXIT);
                Ok(())
            }
            GITHUB => open_link(window, env!("CARGO_PKG_REPOSITORY")),
            MANIFESTO => open_link(window, "https://umbertogiacobbi.biz/vibeware/manifesto"),
            ABOUT => crate::gui::show_about(),
            _ => Ok(()),
        };
        if let Err(error) = result {
            show_error(&format!("Could not complete this menu action:\n{error:#}"));
        }
    }

    // window: owner used by the Windows shell if it needs to display an error.
    // url: fixed project/manifesto address opened by the default browser.
    // Errors: the Windows shell cannot find or launch the URL handler.
    fn open_link(window: HWND, url: &str) -> Result<()> {
        let result = unsafe {
            ShellExecuteW(
                window,
                wide("open").as_ptr(),
                wide(url).as_ptr(),
                ptr::null(),
                ptr::null(),
                SW_SHOWNORMAL,
            )
        } as isize;
        ensure!(
            result > 32,
            "opening the browser failed (Windows shell code {result})"
        );
        Ok(())
    }

    // url: fixed application or brand address, opened after a user's click.
    // Errors: the default browser cannot be launched.
    pub fn open_browser(url: &str) -> Result<()> {
        open_link(ptr::null_mut(), url)
    }

    // path: configuration file opened for editing, without invoking a command shell.
    // Errors: locating or starting Windows Notepad.
    pub fn open_file(path: &std::path::Path) -> Result<()> {
        let system = env::var_os("SystemRoot").context("SystemRoot is unavailable")?;
        Command::new(PathBuf::from(system).join("System32/notepad.exe"))
            .arg(path)
            .spawn()
            .context("opening the configuration editor")?;
        Ok(())
    }

    // Errors: executable-folder discovery or interactive shell startup failure.
    fn open_cli() -> Result<()> {
        let executable = env::current_exe().context("locating the executable")?;
        let folder = executable
            .parent()
            .context("locating the executable folder")?;
        let system = env::var_os("SystemRoot").context("SystemRoot is unavailable")?;
        let shell: Vec<_> = PathBuf::from(system)
            .join("System32/cmd.exe")
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let folder: Vec<_> = folder.as_os_str().encode_wide().chain(Some(0)).collect();
        // ShellExecute creates an interactive console without inheriting the daemon's NUL I/O.
        // No interpolated shell command: the Unicode working directory is a separate parameter.
        let result = unsafe {
            ShellExecuteW(
                ptr::null_mut(),
                wide("open").as_ptr(),
                shell.as_ptr(),
                wide("/D /K").as_ptr(),
                folder.as_ptr(),
                SW_SHOWNORMAL,
            )
        } as isize;
        ensure!(
            result > 32,
            "opening a CLI shell failed (Windows shell code {result})"
        );
        Ok(())
    }

    // control: marks the internally restarted child as a background launch.
    // Errors: malformed restart marker or failure to wait for the previous process.
    pub fn wait_for_restart_parent(control: &Control) -> Result<()> {
        let Some(parent) = env::var_os(RESTART_PARENT) else {
            return Ok(());
        };
        env::remove_var(RESTART_PARENT);
        control.enter_background();
        let parent: u32 = parent
            .to_str()
            .context("invalid restart parent")?
            .parse()
            .context("invalid restart parent PID")?;
        ensure!(
            parent != 0 && parent != std::process::id(),
            "invalid restart parent PID"
        );
        // SYNCHRONIZE permits waiting without terminating or inspecting the previous instance.
        let handle = unsafe { OpenProcess(0x0010_0000, 0, parent) };
        if handle.is_null() {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(87) {
                return Ok(());
            }
            return Err(error).context("opening the previous UlanziDeck process");
        }
        let waited = unsafe { WaitForSingleObject(handle, 10_000) };
        unsafe { CloseHandle(handle) };
        ensure!(
            waited == WAIT_OBJECT_0,
            "the previous UlanziDeck process did not exit within ten seconds"
        );
        Ok(())
    }

    // Errors: locating the current executable/cwd or spawning the replacement process.
    pub fn restart_current_process() -> Result<()> {
        // Run and its tray guard have already returned, closing HID and removing the old icon.
        // OOBE is a one-time interactive request; the restarted resident uses its saved settings.
        let arguments = env::args_os().skip(1).filter(|arg| arg != "--oobe");
        Command::new(env::current_exe().context("locating the executable for restart")?)
            .args(arguments)
            .current_dir(env::current_dir().context("locating the working directory for restart")?)
            .env(RESTART_PARENT, std::process::id().to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .context("restarting UlanziDeck")?;
        Ok(())
    }

    // text: failure shown to users when the controller has no visible console.
    pub fn show_error(text: &str) {
        unsafe {
            MessageBoxW(
                ptr::null_mut(),
                wide(text).as_ptr(),
                wide("UlanziDeck").as_ptr(),
                MB_OK | MB_ICONERROR | MB_SETFOREGROUND,
            );
        }
    }
}

#[cfg(target_os = "windows")]
pub use windows::{
    open_browser, open_file, restart_current_process, show_error, wait_for_restart_parent, Tray,
};

#[cfg(not(target_os = "windows"))]
// target: URL or local file passed directly to the OS opener, without shell interpolation.
// Errors: spawning the platform's default opener.
fn open_target(target: &std::ffi::OsStr) -> Result<()> {
    use anyhow::Context;
    #[cfg(target_os = "macos")]
    let executable = "open";
    #[cfg(not(target_os = "macos"))]
    let executable = "xdg-open";
    std::process::Command::new(executable)
        .arg(target)
        .spawn()
        .context("opening the browser or editor")?;
    Ok(())
}

#[cfg(not(target_os = "windows"))]
// url: fixed application or brand address requested by a user.
// Errors: starting the default browser.
pub fn open_browser(url: &str) -> Result<()> {
    open_target(std::ffi::OsStr::new(url))
}

#[cfg(not(target_os = "windows"))]
// path: existing configuration file requested by a user.
// Errors: starting the default editor.
pub fn open_file(path: &std::path::Path) -> Result<()> {
    open_target(path.as_os_str())
}

#[cfg(not(target_os = "windows"))]
/// No-op tray placeholder for console-only platforms.
pub struct Tray;

#[cfg(not(target_os = "windows"))]
impl Tray {
    // _control: unused on platforms where the controller remains console-only.
    pub fn start(_control: Arc<Control>) -> Result<Self> {
        Ok(Self)
    }
}

#[cfg(not(target_os = "windows"))]
// _control: unused because non-Windows launches do not restart through a tray.
pub fn wait_for_restart_parent(_control: &Control) -> Result<()> {
    Ok(())
}

#[cfg(not(target_os = "windows"))]
// Errors: this operation requires Windows.
pub fn restart_current_process() -> Result<()> {
    anyhow::bail!("system tray restart is available only on Windows")
}

#[cfg(not(target_os = "windows"))]
// text: error to print when a caller requests a native error dialog.
pub fn show_error(text: &str) {
    eprintln!("{text}");
}
