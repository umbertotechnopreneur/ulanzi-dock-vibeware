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

//! Validate configured actions and dispatch approved shortcuts or browser links.
//! Foreground matching uses executable names only. This module never hooks global input;
//! the controller decides whether a physical dock release is allowed to execute an action.

use anyhow::Result;
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

#[derive(Clone, Debug, PartialEq, Eq)]
/// Distinguish a known executable from a failed lookup or an unsupported platform.
pub enum ForegroundProcess {
    Known(String),
    Unknown,
    Unsupported,
}

/// `page` is usable when it has no executable restriction or matches `foreground`.
/// Unknown foreground identity does not authorize an application-specific page.
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

/// Find an enabled application page for `foreground`; return no automatic target
/// for an unknown process or for pages without an executable mapping.
pub fn matching_page(
    config: &crate::model::Config,
    foreground: &ForegroundProcess,
) -> Option<usize> {
    if !matches!(foreground, ForegroundProcess::Known(_)) {
        return None;
    }
    config.pages.iter().position(|page| {
        page.enabled && !page.executables.is_empty() && page_available(page, foreground)
    })
}

// Inspect only the owner of the foreground window; never read titles or keyboard input.
#[cfg(target_os = "windows")]
/// Read the active executable, or explicitly report that this platform has no implementation.
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
/// Read the active executable, or explicitly report that this platform has no implementation.
pub fn foreground_process() -> ForegroundProcess {
    ForegroundProcess::Unsupported
}

// action: one configured action, excluding page navigation.
// Errors: invalid action or failed OS input/open operation.
pub fn execute(action: &str) -> Result<()> {
    if action == "none" {
        return Ok(());
    }
    if let Some(keys) = action.strip_prefix("hotkey:") {
        return hotkey(keys);
    }
    if let Some(media) = action.strip_prefix("media:") {
        let key = media_key(media)?;
        return Enigo::new(&Settings::default())?
            .key(key, Direction::Click)
            .map_err(Into::into);
    }
    if let Some(url) = action.strip_prefix("open:") {
        validate_url(url)?;
        return crate::tray::open_browser(url);
    }
    anyhow::bail!("unsupported action: {action}")
}

/// Translate a configured media token to an input-library key without sending it.
fn media_key(media: &str) -> Result<Key> {
    Ok(match media {
        "play-pause" => Key::MediaPlayPause,
        "previous" => Key::MediaPrevTrack,
        "next" => Key::MediaNextTrack,
        "mute" => Key::VolumeMute,
        "volume-down" => Key::VolumeDown,
        "volume-up" => Key::VolumeUp,
        _ => anyhow::bail!("unsupported media action: {media}"),
    })
}

/// Accept only nonempty HTTP(S) targets without whitespace/control characters.
/// This is a launch-policy check, not a network request or a destination trust check.
fn validate_url(url: &str) -> Result<()> {
    let address = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"));
    anyhow::ensure!(
        address.is_some_and(|address| !address.is_empty() && !address.starts_with('/')),
        "only non-empty HTTP(S) URLs are supported"
    );
    anyhow::ensure!(
        !url.chars().any(|c| c.is_control() || c.is_whitespace()),
        "URL contains whitespace or control characters"
    );
    Ok(())
}

/// Validate without constructing an input injector or launching an application.
pub fn validate(action: &str) -> Result<()> {
    if action == "none" {
        return Ok(());
    }
    if let Some(chord) = action.strip_prefix("hotkey:") {
        hotkey_keys(chord)?;
        return Ok(());
    }
    if let Some(media) = action.strip_prefix("media:") {
        media_key(media)?;
        return Ok(());
    }
    if let Some(url) = action.strip_prefix("open:") {
        return validate_url(url);
    }
    anyhow::bail!("unsupported action: {action}")
}

// chord: plus-separated modifiers and final key, e.g. ctrl+shift+p.
// Errors: unsupported key name. Parsing never injects input.
/// Parse `chord` into ordered keys before any injection begins.
/// Reject unknown tokens so malformed shortcuts cannot leave a partial chord pressed.
fn hotkey_keys(chord: &str) -> Result<Vec<Key>> {
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
    Ok(keys)
}

/// Inject validated `chord` in order, then release held keys in reverse order.
/// Propagate input-library errors and attempt cleanup after partial injection.
fn hotkey(chord: &str) -> Result<()> {
    let keys = hotkey_keys(chord)?;
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
    fn invalid_actions_are_rejected_without_os_side_effects() {
        for action in [
            "hotkey:",
            "hotkey:ctrl+",
            "hotkey:ctrl+unknown",
            "media:invalid",
            "open:file:///settings.json",
            "open:https://",
            "open:https://example.com\0.exe",
        ] {
            assert!(validate(action).is_err(), "accepted {action:?}");
        }
        for action in [
            "none",
            "hotkey:ctrl+shift+f12",
            "media:play-pause",
            "open:https://example.com/",
        ] {
            assert!(validate(action).is_ok(), "rejected {action:?}");
        }
    }

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
