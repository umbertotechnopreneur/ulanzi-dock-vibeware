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

//! Validate configured actions and dispatch approved shortcuts or browser links.
//! Foreground matching uses executable names only. This module never hooks global input;
//! the controller decides whether a physical dock release is allowed to execute an action.

#[cfg(target_os = "linux")]
mod mpris;
#[cfg(target_os = "linux")]
mod audio;
#[cfg(target_os = "linux")]
mod wayland;
#[cfg(target_os = "linux")]
pub(crate) mod desktop;

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

/// Validated desktop action. Parsing a profile has no desktop side effects.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ActionPlan<'a> {
    None,
    Hotkey(Vec<Key>),
    MediaKey(Key),
    #[cfg(target_os = "linux")]
    Mpris(&'static str),
    #[cfg(target_os = "linux")]
    ToggleMute,
    #[cfg(target_os = "linux")]
    Volume(bool),
    #[cfg(target_os = "linux")]
    Desktop(desktop::DesktopAction),
    OpenUrl(&'a str),
}

pub(crate) fn plan(action: &str) -> Result<ActionPlan<'_>> {
    if action == "none" {
        return Ok(ActionPlan::None);
    }
    #[cfg(target_os = "linux")]
    if let Some(token) = action.strip_prefix("desktop:") {
        return Ok(ActionPlan::Desktop(desktop::DesktopAction::parse(token)?));
    }
    if let Some(chord) = action.strip_prefix("hotkey:") {
        return Ok(ActionPlan::Hotkey(hotkey_keys(chord)?));
    }
    if let Some(media) = action.strip_prefix("media:") {
        #[cfg(target_os = "linux")]
        match media {
            "play-pause" => return Ok(ActionPlan::Mpris("PlayPause")),
            "next" => return Ok(ActionPlan::Mpris("Next")),
            "previous" => return Ok(ActionPlan::Mpris("Previous")),
            "mute" => return Ok(ActionPlan::ToggleMute),
            "volume-down" => return Ok(ActionPlan::Volume(false)),
            "volume-up" => return Ok(ActionPlan::Volume(true)),
            _ => {}
        }
        return Ok(ActionPlan::MediaKey(media_key(media)?));
    }
    if let Some(url) = action.strip_prefix("open:") {
        validate_url(url)?;
        return Ok(ActionPlan::OpenUrl(url));
    }
    anyhow::bail!("unsupported action: {action}")
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Output {
    Key(Key, Direction),
    #[cfg(target_os = "linux")]
    Mpris(&'static str),
    #[cfg(target_os = "linux")]
    ToggleMute,
    #[cfg(target_os = "linux")]
    Volume(bool),
    #[cfg(target_os = "linux")]
    Desktop(desktop::DesktopAction),
    OpenUrl(String),
}

/// Send the plan to the selected output. If a press fails, release keys already held.
/// Attempt every release even if an earlier release reports an error.
fn dispatch(plan: ActionPlan<'_>, mut send: impl FnMut(Output) -> Result<()>) -> Result<()> {
    match plan {
        ActionPlan::None => Ok(()),
        ActionPlan::OpenUrl(url) => send(Output::OpenUrl(url.to_owned())),
        ActionPlan::MediaKey(key) => send(Output::Key(key, Direction::Click)),
        #[cfg(target_os = "linux")]
        ActionPlan::Mpris(method) => send(Output::Mpris(method)),
        #[cfg(target_os = "linux")]
        ActionPlan::ToggleMute => send(Output::ToggleMute),
        #[cfg(target_os = "linux")]
        ActionPlan::Volume(up) => send(Output::Volume(up)),
        #[cfg(target_os = "linux")]
        ActionPlan::Desktop(action) => send(Output::Desktop(action)),
        ActionPlan::Hotkey(keys) => {
            for (index, key) in keys.iter().enumerate() {
                if let Err(error) = send(Output::Key(*key, Direction::Press)) {
                    for held in keys[..index].iter().rev() {
                        let _ = send(Output::Key(*held, Direction::Release));
                    }
                    return Err(error);
                }
            }
            let mut result = Ok(());
            for key in keys.into_iter().rev() {
                if let Err(error) = send(Output::Key(key, Direction::Release)) {
                    result = Err(error);
                }
            }
            result
        }
    }
}

thread_local! {
    // Keep one input connection for the controller thread and its portal authorization.
    #[cfg(target_os = "linux")]
    static WAYLAND_KEYBOARD: std::cell::RefCell<Option<wayland::Keyboard>> = const { std::cell::RefCell::new(None) };
    static KEYBOARD: std::cell::RefCell<Option<Enigo>> = const { std::cell::RefCell::new(None) };
}

/// Keep input teardown inside the controller stack, before thread-local destruction.
pub(crate) struct InputSession;

impl Drop for InputSession {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        let _ = WAYLAND_KEYBOARD.try_with(|slot| {
            let input = slot.borrow_mut().take();
            drop(input);
        });
        let _ = KEYBOARD.try_with(|slot| {
            let input = slot.borrow_mut().take();
            drop(input);
        });
    }
}

pub fn execute(action: &str) -> Result<()> {
    dispatch(plan(action)?, |event| match event {
        Output::Key(key, direction) => {
            #[cfg(target_os = "linux")]
            if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                return WAYLAND_KEYBOARD.with(|keyboard| {
                    let mut keyboard = keyboard.borrow_mut();
                    let input = match &mut *keyboard {
                        Some(input) => input,
                        slot @ None => slot.insert(wayland::Keyboard::new()?),
                    };
                    input.key(key, direction)
                });
            }
            KEYBOARD.with(|keyboard| {
                let mut keyboard = keyboard.borrow_mut();
                let input = match &mut *keyboard {
                    Some(input) => input,
                    slot @ None => slot.insert(Enigo::new(&Settings::default())?),
                };
                Ok(input.key(key, direction)?)
            })
        },
        #[cfg(target_os = "linux")]
        Output::Mpris(method) => mpris::dispatch(method),
        #[cfg(target_os = "linux")]
        Output::ToggleMute => audio::toggle_mute(),
        #[cfg(target_os = "linux")]
        Output::Volume(up) => audio::volume(up),
        #[cfg(target_os = "linux")]
        Output::Desktop(action) => desktop::execute(action),
        Output::OpenUrl(url) => crate::tray::open_browser(&url),
    })
}

#[cfg(test)]
pub(crate) use self::Output as RecordedOutput;

/// Exercise the same parsing and dispatch as a live action using an in-memory sink.
#[cfg(test)]
pub(crate) fn simulate(action: &str) -> Result<Vec<RecordedOutput>> {
    let mut events = Vec::new();
    dispatch(plan(action)?, |event| {
        events.push(event);
        Ok(())
    })?;
    Ok(events)
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
    plan(action).map(|_| ())
}

// chord: plus-separated modifiers and final key, e.g. ctrl+shift+p.
// Errors: unsupported key name. Parsing never injects input.
/// Parse `chord` into ordered keys before any injection begins.
/// Reject unknown tokens so malformed shortcuts cannot leave a partial chord pressed.
fn hotkey_keys(chord: &str) -> Result<Vec<Key>> {
    #[cfg(target_os = "linux")]
    anyhow::ensure!(chord.len() <= 64, "hotkey is too long");
    let mut keys = Vec::new();
    #[cfg(target_os = "linux")]
    let mut has_final_key = false;
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
            "f9" => Key::F9,
            "f12" => Key::F12,
            single if single.chars().count() == 1 => Key::Unicode(single.chars().next().unwrap()),
            _ => anyhow::bail!("unsupported hotkey token: {token}"),
        };
        #[cfg(target_os = "linux")]
        {
            anyhow::ensure!(!has_final_key, "only modifiers may precede the final key");
            if matches!(key, Key::Control | Key::Alt | Key::Shift | Key::Meta) {
                anyhow::ensure!(!keys.contains(&key), "duplicate hotkey modifier: {token}");
            } else {
                anyhow::ensure!(
                    !matches!(key, Key::Unicode(c) if c.is_control()),
                    "use a named hotkey key instead of a control character"
                );
                has_final_key = true;
            }
        }
        keys.push(key);
    }
    #[cfg(target_os = "linux")]
    anyhow::ensure!(has_final_key, "hotkey requires a final key");
    anyhow::ensure!(!keys.is_empty(), "empty hotkey");
    Ok(keys)
}

#[cfg(test)]
mod offline_actions {
    use super::*;
    use Direction::{Click, Press, Release};

    #[test]
    fn desktop_routes_preserve_payloads_and_chord_order() {
        let chord = simulate("hotkey:super+shift+s").unwrap();
        assert_eq!(chord, [
            Output::Key(Key::Meta, Press),
            Output::Key(Key::Shift, Press),
            Output::Key(Key::Unicode('s'), Press),
            Output::Key(Key::Unicode('s'), Release),
            Output::Key(Key::Shift, Release),
            Output::Key(Key::Meta, Release),
        ]);
        let url = "https://example.com/search?q=rust&lang=fr";
        assert_eq!(simulate(&format!("open:{url}")).unwrap(), [Output::OpenUrl(url.into())]);
        #[cfg(not(target_os = "linux"))]
        for (action, key) in [
            #[cfg(not(target_os = "linux"))]
            ("media:mute", Key::VolumeMute),
            #[cfg(not(target_os = "linux"))]
            ("media:volume-down", Key::VolumeDown),
            #[cfg(not(target_os = "linux"))]
            ("media:volume-up", Key::VolumeUp),
        ] {
            assert_eq!(simulate(action).unwrap(), [Output::Key(key, Click)], "{action}");
        }
        #[cfg(target_os = "linux")]
        assert_eq!(simulate("media:mute").unwrap(), [Output::ToggleMute]);
        assert!(simulate("none").unwrap().is_empty());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn playback_sends_one_mpris_command_without_keyboard_events() {
        for (action, method) in [
            ("media:play-pause", "PlayPause"),
            ("media:next", "Next"),
            ("media:previous", "Previous"),
        ] {
            assert_eq!(simulate(action).unwrap(), [Output::Mpris(method)], "{action}");
        }
    }

    #[test]
    fn failed_final_press_releases_only_held_modifiers() {
        let mut events = Vec::new();
        let result = dispatch(plan("hotkey:ctrl+shift+p").unwrap(), |event| {
            let should_fail = event == Output::Key(Key::Unicode('p'), Press);
            events.push(event);
            anyhow::ensure!(!should_fail, "press failed");
            Ok(())
        });
        assert_eq!(result.unwrap_err().to_string(), "press failed");
        assert_eq!(events, [
            Output::Key(Key::Control, Press), Output::Key(Key::Shift, Press),
            Output::Key(Key::Unicode('p'), Press), Output::Key(Key::Shift, Release),
            Output::Key(Key::Control, Release),
        ]);
    }

    #[test]
    fn failed_final_key_release_still_releases_modifiers() {
        let mut events = Vec::new();
        let result = dispatch(plan("hotkey:alt+f5").unwrap(), |event| {
            let should_fail = event == Output::Key(Key::F5, Release);
            events.push(event);
            anyhow::ensure!(!should_fail, "release failed");
            Ok(())
        });
        assert_eq!(result.unwrap_err().to_string(), "release failed");
        assert_eq!(events, [
            Output::Key(Key::Alt, Press), Output::Key(Key::F5, Press),
            Output::Key(Key::F5, Release), Output::Key(Key::Alt, Release),
        ]);
    }
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
    #[cfg(target_os = "linux")]
    fn linux_hotkeys_accept_one_final_key_after_distinct_modifiers() {
        for chord in [
            "a+b", "a+ctrl", "ctrl", "ctrl+ALT", "ctrl+control+s", "meta+win+p", "\n",
        ] {
            assert!(hotkey_keys(chord).is_err(), "accepted {chord:?}");
        }
        for chord in [
            "space", "ctrl+;", "shift+alt+f", "CONTROL+alt+shift+meta+enter",
        ] {
            assert!(hotkey_keys(chord).is_ok(), "rejected {chord:?}");
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
