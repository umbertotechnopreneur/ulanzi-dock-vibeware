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

use anyhow::{Context, Result};
use enigo::{Direction, Enigo, Key, Keyboard, Settings};
use std::process::Command;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ForegroundProcess {
    Known(String),
    Unknown,
    Unsupported,
}

pub fn page_available(page: &crate::model::PageConfig, foreground: &ForegroundProcess) -> bool {
    if page.executables.is_empty() || *foreground == ForegroundProcess::Unsupported {
        return true;
    }
    match foreground {
        ForegroundProcess::Known(process) => page
            .executables
            .iter()
            .any(|name| name.eq_ignore_ascii_case(process)),
        _ => false,
    }
}

pub fn matching_page(
    config: &crate::model::Config,
    foreground: &ForegroundProcess,
) -> Option<usize> {
    if !matches!(foreground, ForegroundProcess::Known(_)) {
        return None;
    }
    config
        .pages
        .iter()
        .position(|page| !page.executables.is_empty() && page_available(page, foreground))
}

// Inspect only the owner of the foreground window; never read titles or keyboard input.
#[cfg(target_os = "windows")]
pub fn foreground_process() -> ForegroundProcess {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
        },
        UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
    };
    // SAFETY: Win32 handles are checked, the output buffer is valid for its declared
    // length, and every successfully opened process handle is closed before return.
    unsafe {
        let window = GetForegroundWindow();
        if window.is_null() {
            return ForegroundProcess::Unknown;
        }
        let mut pid = 0;
        if GetWindowThreadProcessId(window, &mut pid) == 0 || pid == 0 {
            return ForegroundProcess::Unknown;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process.is_null() {
            return ForegroundProcess::Unknown;
        }
        let mut path = [0u16; 32768];
        let mut length = path.len() as u32;
        let success = QueryFullProcessImageNameW(process, 0, path.as_mut_ptr(), &mut length);
        CloseHandle(process);
        if success == 0 || GetForegroundWindow() != window {
            return ForegroundProcess::Unknown;
        }
        let path = String::from_utf16_lossy(&path[..length as usize]);
        ForegroundProcess::Known(
            path.rsplit(['\\', '/'])
                .next()
                .unwrap_or("")
                .to_ascii_lowercase(),
        )
    }
}

#[cfg(not(target_os = "windows"))]
pub fn foreground_process() -> ForegroundProcess {
    ForegroundProcess::Unsupported
}

// action: one configured action, excluding page navigation.
// Errors: invalid action or failed OS input/open operation.
pub fn execute(action: &str) -> Result<()> {
    if let Some(keys) = action.strip_prefix("hotkey:") {
        return hotkey(keys);
    }
    if let Some(media) = action.strip_prefix("media:") {
        let key = match media {
            "play-pause" => Key::MediaPlayPause,
            "previous" => Key::MediaPrevTrack,
            "next" => Key::MediaNextTrack,
            "mute" => Key::VolumeMute,
            "volume-down" => Key::VolumeDown,
            "volume-up" => Key::VolumeUp,
            _ => anyhow::bail!("unsupported media action: {media}"),
        };
        return Enigo::new(&Settings::default())?
            .key(key, Direction::Click)
            .map_err(Into::into);
    }
    if let Some(url) = action.strip_prefix("open:") {
        anyhow::ensure!(
            url.starts_with("https://") || url.starts_with("http://"),
            "only HTTP(S) URLs are supported"
        );
        #[cfg(target_os = "windows")]
        let mut command = {
            let mut c = Command::new("explorer.exe");
            c.arg(url);
            c
        };
        #[cfg(target_os = "macos")]
        let mut command = {
            let mut c = Command::new("open");
            c.arg(url);
            c
        };
        #[cfg(target_os = "linux")]
        let mut command = {
            let mut c = Command::new("xdg-open");
            c.arg(url);
            c
        };
        command.spawn().with_context(|| format!("opening {url}"))?;
        return Ok(());
    }
    anyhow::bail!("unsupported action: {action}")
}

// chord: plus-separated modifiers and final key, e.g. ctrl+shift+p.
// Errors: unsupported key name or failed OS keyboard injection.
fn hotkey(chord: &str) -> Result<()> {
    let mut keys = Vec::new();
    for token in chord.split('+') {
        let key = match token.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => Key::Control,
            "alt" => Key::Alt,
            "shift" => Key::Shift,
            "super" | "win" | "meta" => Key::Meta,
            "tab" => Key::Tab,
            "enter" => Key::Return,
            "escape" => Key::Escape,
            "grave" => Key::Unicode('`'),
            "space" => Key::Space,
            "f2" => Key::F2,
            "f5" => Key::F5,
            "f7" => Key::F7,
            "f12" => Key::F12,
            single if single.chars().count() == 1 => Key::Unicode(single.chars().next().unwrap()),
            _ => anyhow::bail!("unsupported hotkey token: {token}"),
        };
        keys.push(key);
    }
    anyhow::ensure!(!keys.is_empty(), "empty hotkey");
    let mut enigo = Enigo::new(&Settings::default())?;
    let mut pressed = Vec::new();
    for key in keys {
        if let Err(error) = enigo.key(key, Direction::Press) {
            for prior in pressed.into_iter().rev() {
                let _ = enigo.key(prior, Direction::Release);
            }
            return Err(error.into());
        }
        pressed.push(key);
    }
    let mut failed = None;
    for key in pressed.into_iter().rev() {
        if let Err(error) = enigo.key(key, Direction::Release) {
            failed = Some(error);
        }
    }
    if let Some(error) = failed {
        return Err(error.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_pages_require_the_configured_foreground_process() {
        let config = crate::model::Config::default();
        assert_eq!(
            matching_page(&config, &ForegroundProcess::Known("CODE.EXE".into())),
            Some(3)
        );
        assert_eq!(
            matching_page(
                &config,
                &ForegroundProcess::Known("code - insiders.exe".into())
            ),
            Some(3)
        );
        assert_eq!(
            matching_page(&config, &ForegroundProcess::Known("Codex.exe".into())),
            Some(2)
        );
        assert_eq!(
            matching_page(&config, &ForegroundProcess::Known("Spotify.exe".into())),
            Some(4)
        );
        assert_eq!(
            matching_page(&config, &ForegroundProcess::Known("my-codex.exe".into())),
            None
        );
        assert_eq!(matching_page(&config, &ForegroundProcess::Unknown), None);
        assert!(!page_available(
            &config.pages[2],
            &ForegroundProcess::Unknown
        ));
        assert!(!page_available(
            &config.pages[4],
            &ForegroundProcess::Known("Code.exe".into())
        ));
        assert!(page_available(
            &config.pages[0],
            &ForegroundProcess::Unknown
        ));
        assert!(page_available(
            &config.pages[2],
            &ForegroundProcess::Unsupported
        ));
    }
}
