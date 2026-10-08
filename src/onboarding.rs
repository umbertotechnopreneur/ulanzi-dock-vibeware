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
enum Choice {
    Next,
    Back,
    Exit,
}

struct OptionRow {
    icon: &'static str,
    label: &'static str,
    tone: Tone,
    choice: Choice,
}

struct MenuCursor;
impl Drop for MenuCursor {
    fn drop(&mut self) {
        print!("\x1b[?25h");
        let _ = io::stdout().flush();
    }
}

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
                    "{} Nine pages: Windows / media, Utility, Codex, VS Code, Spotify, Word, PowerPoint, Excel and Themes.",
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
        config.setup_completed = true;
        config
            .save_selected_theme(path)
            .with_context(|| format!("saving first-run setup in {}", path.display()))?;
    }
    cli_pages::setup_section(
        "🎉",
        "You are ready!",
        "Your setup is saved. Starting UlanziDock...",
    );
    Ok(true)
}
