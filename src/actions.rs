/* VBWR B
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 * VibeWare is Human intent, AI, and plenty of tokens ;-)
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
