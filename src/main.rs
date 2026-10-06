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

mod actions;
mod art;
mod dock;
mod model;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use model::{Config, Theme, CLOCK_KEY, NEXT_KEY};
use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(
    name = "ulanzi-dock-vibeware",
    version,
    about = "VibeWare standalone controller for the Ulanzi D200H"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Enumerate only the D200H consumer interface; sends no reports.
    Doctor,
    /// Print available visual themes.
    Themes,
    /// Create a settings file with all three pages and editable actions.
    Init {
        #[arg(long, default_value = "settings.json")]
        config: PathBuf,
    },
    /// Export every up/down key image for visual inspection; no device access.
    Render {
        #[arg(long, default_value = "settings.json")]
        config: PathBuf,
        #[arg(long, default_value = "dist/artwork")]
        output: PathBuf,
    },
    /// Display a page, listen to this device, and dispatch configured actions.
    Run {
        #[arg(long, default_value = "settings.json")]
        config: PathBuf,
        #[arg(long, value_enum)]
        theme: Option<Theme>,
        /// Stop after this many seconds (1-300); omit to run until Ctrl+C.
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..=300))]
        seconds: Option<u64>,
        /// Display and read the dock without dispatching desktop actions.
        #[arg(long)]
        no_actions: bool,
        /// Disable the clock on the wide bottom display.
        #[arg(long)]
        no_clock: bool,
    },
}

// output: destination for PNGs; creates a directory tree per theme and page.
// config: validated page definitions.
// Errors: image generation or filesystem failure.
fn export_art(output: &PathBuf, config: &Config) -> Result<()> {
    for theme in Theme::ALL {
        for (page_number, page) in config.pages.iter().enumerate() {
            let directory = output
                .join(theme.slug())
                .join(format!("page-{}", page_number + 1));
            fs::create_dir_all(&directory)
                .with_context(|| format!("creating {}", directory.display()))?;
            for index in 0..page.keys.len() {
                for (pressed, state) in [(false, "up"), (true, "down")] {
                    let bytes = art::render(theme, page, index, pressed)?;
                    fs::write(directory.join(format!("key-{index:02}-{state}.png")), bytes)?;
                }
            }
        }
    }
    println!(
        "Exported {} PNGs to {}",
        Theme::ALL.len() * config.pages.len() * 28,
        output.display()
    );
    Ok(())
}

// config: validated page and action definitions.
// theme: selected visual family.
// seconds: optional bounded diagnostic duration.
// no_actions: disable shortcut and media dispatch.
// clock: periodically update the wide bottom clock window.
// Errors: device access, display update, or clock failure.
fn run(
    config: Config,
    theme: Theme,
    seconds: Option<u64>,
    no_actions: bool,
    clock: bool,
) -> Result<()> {
    let (api, found) = dock::discover()?;
    anyhow::ensure!(
        found,
        "D200H consumer HID interface not found; run 'doctor'"
    );
    let dock = dock::Dock::open(&api)?;
    let running = Arc::new(AtomicBool::new(true));
    let stop = Arc::clone(&running);
    ctrlc::set_handler(move || stop.store(false, Ordering::SeqCst))?;
    let mut page = 0usize;
    let mut held = None;
    let started = Instant::now();
    dock.show(&config.pages[page], theme, None)?;
    if clock {
        dock.clock()?;
    }
    let mut last_clock = Instant::now();
    println!(
        "D200H active | page: {} | theme: {} | Ctrl+C to stop",
        config.pages[page].name,
        theme.slug()
    );
    while running.load(Ordering::SeqCst)
        && seconds.is_none_or(|limit| started.elapsed() < Duration::from_secs(limit))
    {
        if clock && last_clock.elapsed() >= Duration::from_secs(1) {
            dock.clock()?;
            last_clock = Instant::now();
        }
        if let Some(event) = dock.read_button(Duration::from_millis(100))? {
            if event.index == CLOCK_KEY {
                continue;
            }
            if event.pressed {
                if held != Some(event.index) {
                    held = Some(event.index);
                    dock.show(&config.pages[page], theme, held)?;
                }
                continue;
            }
            if held != Some(event.index) {
                continue;
            }
            held = None;
            if event.index == NEXT_KEY {
                page = (page + 1) % config.pages.len();
                println!("Page: {}", config.pages[page].name);
            } else if !no_actions {
                let key = &config.pages[page].keys[event.index];
                if let Err(error) = actions::execute(&key.action) {
                    eprintln!("Action '{}' failed: {error:#}", key.label);
                }
            }
            dock.show(&config.pages[page], theme, None)?;
        }
    }
    println!("Stopped. The dock may clear its display when the program exits.");
    Ok(())
}

// Errors: command-specific validation, filesystem, HID, or rendering failure.
fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Doctor => {
            let (_, found) = dock::discover()?;
            println!(
                "D200H consumer interface: {}",
                if found { "found" } else { "not found" }
            );
            println!("VID:PID 2207:0019 | usage page 0x0c | usage 1 | no reports sent");
        }
        Command::Themes => {
            for theme in Theme::ALL {
                println!("{}", theme.slug());
            }
        }
        Command::Init { config } => {
            anyhow::ensure!(
                !config.exists(),
                "{} already exists; preserving it",
                config.display()
            );
            Config::default().save_new(&config)?;
            println!("Created {}", config.display());
        }
        Command::Render { config, output } => export_art(&output, &Config::load(&config)?)?,
        Command::Run {
            config,
            theme,
            seconds,
            no_actions,
            no_clock,
        } => {
            let settings = Config::load(&config)?;
            let selected = theme.unwrap_or(settings.theme);
            run(settings, selected, seconds, no_actions, !no_clock)?;
        }
    }
    Ok(())
}
