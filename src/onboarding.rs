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

//! Show the terminal welcome flow and persist completion only after confirmation.
//! Input comes from the attached console, with a numbered fallback for simple terminals.
//! Redirected runs do not wait for input. This flow does not open or test the dock.

use crate::{
    cli_pages::{self, Tone},
    console::{self, MenuKey},
    model::Config,
};
use anyhow::{Context, Result};
use std::{
    io::{self, IsTerminal, Write},
    path::Path,
};

#[derive(Clone, Copy)]
/// A welcome-flow decision, independent of how the terminal encodes input.
enum Choice {
    Next,
    Back,
    Exit,
}

/// One visible menu row and the decision it returns when selected.
struct OptionRow {
    icon: &'static str,
    label: &'static str,
    tone: Tone,
    choice: Choice,
}

/// Restore cursor visibility even when menu drawing or input returns an error.
struct MenuCursor;
impl Drop for MenuCursor {
    /// Restore cursor visibility even when the welcome flow exits early.
    fn drop(&mut self) {
        print!("\x1b[?25h");
        let _ = io::stdout().flush();
    }
}

/// Draw `options`, highlighting zero-based `selected`; `redraw` replaces existing rows.
/// Writes only to stdout and propagates flushing errors.
fn draw_menu(options: &[OptionRow], selected: usize, redraw: bool) -> Result<()> {
    if redraw {
        print!("\x1b[{}A\r", options.len() + 1);
    }
    for (index, option) in options.iter().enumerate() {
        print!("\x1b[2K");
        let label = format!(
            "{} {}",
            cli_pages::setup_icon(option.icon, ">"),
            option.label
        );
        cli_pages::setup_choice(&label, option.tone, index == selected);
    }
    let hint = cli_pages::clipped(
        "↑/↓ Choose · Enter Continue · Esc Exit",
        console::columns().unwrap_or(80).saturating_sub(3),
    );
    println!("\x1b[2K  {}", cli_pages::styled(&hint, Tone::Muted));
    io::stdout().flush()?;
    Ok(())
}

/// Wait for a choice from nonempty `options`, using native keys or numbered input.
/// EOF/cancel returns Exit; console I/O errors are reported to the caller.
fn choose(options: &[OptionRow]) -> Result<Choice> {
    println!();
    let selected = if let Some(mut input) = console::MenuInput::open()? {
        print!("\x1b[?25l");
        let _cursor = MenuCursor;
        let mut selected = 0;
        draw_menu(options, selected, false)?;
        loop {
            let next = match input.read_key()? {
                MenuKey::Up => (selected + options.len() - 1) % options.len(),
                MenuKey::Down => (selected + 1) % options.len(),
                MenuKey::Confirm => break selected,
                MenuKey::Cancel => return Ok(Choice::Exit),
                MenuKey::Other => continue,
            };
            if next != selected {
                selected = next;
                draw_menu(options, selected, true)?;
            }
        }
    } else {
        for (index, option) in options.iter().enumerate() {
            cli_pages::setup_choice(
                &format!("{} · {}", index + 1, option.label),
                option.tone,
                false,
            );
        }
        loop {
            print!(
                "  {} ",
                cli_pages::styled(
                    "Enter to continue · number to choose · q to exit:",
                    Tone::Muted
                )
            );
            io::stdout().flush()?;
            let mut answer = String::new();
            if io::stdin().read_line(&mut answer)? == 0 {
                return Ok(Choice::Exit);
            }
            let answer = answer.trim();
            if answer.eq_ignore_ascii_case("q") {
                return Ok(Choice::Exit);
            }
            if answer.is_empty() {
                break 0;
            }
            if let Ok(number) = answer.parse::<usize>() {
                if (1..=options.len()).contains(&number) {
                    break number - 1;
                }
            }
            cli_pages::setup_text("Choose one of the displayed options.", Tone::Warning);
        }
    };
    cli_pages::setup_text(
        &format!(
            "{} {}",
            cli_pages::setup_icon("✅", "[OK]"),
            options[selected].label
        ),
        options[selected].tone,
    );
    Ok(options[selected].choice)
}

// Complete onboarding only after the owner has seen the essentials and chosen
// to continue. A redirected run never blocks waiting for keyboard input.
/// `config` receives the completion flag after `path` is saved successfully.
/// `show_branding` controls repeated artwork. Return false for cancellation or a
/// noninteractive terminal; propagate input and persistence failures.
pub fn show(config: &mut Config, path: &Path, show_branding: bool) -> Result<bool> {
    cli_pages::print_setup_intro(show_branding);
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        cli_pages::setup_text("Open an interactive terminal and run 'ulanzi-dock-vibeware run --oobe' to finish the welcome.", Tone::Warning);
        return Ok(false);
    }
    let mut step = 0;
    loop {
        let choice = if step == 0 {
            cli_pages::setup_section(
                "🧭",
                "Step 1 of 2 · Meet your dock",
                "Your controls live on the dock. Keep this window open while you use it.",
            );
            cli_pages::setup_text(
                &format!(
                    "{} Ten pages: Windows / media, Utility, Codex, VS Code, Spotify, Word, PowerPoint, Excel, OBS Creator and Themes.",
                    cli_pages::setup_icon("🎛️", ">")
                ),
                Tone::Primary,
            );
            cli_pages::setup_text(
                &format!(
                    "{} Key 4 moves to the next page. Pick your look on the Themes page.",
                    cli_pages::setup_icon("🎨", ">")
                ),
                Tone::Primary,
            );
            cli_pages::setup_text(
                &format!(
                    "{} Codex, VS Code and Spotify select their page when they enter the foreground.",
                    cli_pages::setup_icon("💡", ">")
                ),
                Tone::Info,
            );
            choose(&[
                OptionRow {
                    icon: "👉",
                    label: "Continue to my setup",
                    tone: Tone::Success,
                    choice: Choice::Next,
                },
                OptionRow {
                    icon: "🚪",
                    label: "Exit setup",
                    tone: Tone::Warning,
                    choice: Choice::Exit,
                },
            ])?
        } else {
            cli_pages::setup_section(
                "🚀",
                "Step 2 of 2 · Make yourself at home",
                "Check the essentials, then start your dock.",
            );
            cli_pages::setup_text(
                &format!(
                    "{} Theme · {}",
                    cli_pages::setup_icon("🎨", ">"),
                    config.theme.label()
                ),
                Tone::Accent,
            );
            cli_pages::setup_text("Change it anytime on the Themes page.", Tone::Muted);
            println!();
            cli_pages::setup_text(
                &format!("{} Your settings", cli_pages::setup_icon("📁", ">")),
                Tone::Accent,
            );
            cli_pages::setup_text(&path.display().to_string(), Tone::Info);
            cli_pages::setup_text(
                "Personalize shortcuts in this file while the controller is stopped.",
                Tone::Muted,
            );
            println!();
            cli_pages::setup_text(
                &format!("{} Start with Windows", cli_pages::setup_icon("🖥️", ">")),
                Tone::Accent,
            );
            cli_pages::setup_text(
                "Use 'startup --enable' from the folder containing your settings.",
                Tone::Muted,
            );
            choose(&[
                OptionRow {
                    icon: "🚀",
                    label: "Start my dock",
                    tone: Tone::Success,
                    choice: Choice::Next,
                },
                OptionRow {
                    icon: "🔙",
                    label: "Back",
                    tone: Tone::Muted,
                    choice: Choice::Back,
                },
                OptionRow {
                    icon: "🚪",
                    label: "Exit setup",
                    tone: Tone::Warning,
                    choice: Choice::Exit,
                },
            ])?
        };
        match choice {
            Choice::Next if step == 0 => step = 1,
            Choice::Next => break,
            Choice::Back => step = 0,
            Choice::Exit => {
                cli_pages::setup_text(
                    "Setup paused. Reopen the app or use 'run --oobe' when you are ready.",
                    Tone::Muted,
                );
                return Ok(false);
            }
        }
    }
    if !config.setup_completed {
        crate::settings::SettingsStore::load(path)?
            .save_setup_completed()
            .with_context(|| format!("saving first-run setup in {}", path.display()))?;
        config.setup_completed = true;
    }
    cli_pages::setup_section(
        "🎉",
        "You are ready!",
        "Your setup is saved. Starting UlanziDeck...",
    );
    Ok(true)
}
