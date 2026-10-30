/* VBWR B
 *
 * Project: UlanziDeck
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Modified with AI: OpenAI Codex; command routing and Store edition boundaries and readability; 2026-10-27
 * Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
 * Evidence: docs/CODE-REVIEW.md; current working-tree changes
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 *
 * VBWR E */

//! Process entry point and command routing. Resolve settings/CLI overrides here;
//! delegate rendering, GUI lifecycle and HID work to their dedicated modules.
//! Informational commands do not connect to the dock. Store builds enforce theme
//! entitlement in the controller and keep artwork export a community developer tool.

mod actions;
mod applications;
mod art;
mod cli_pages;
mod console;
mod controller;
mod dock;
mod gui;
mod licensing;
mod model;
mod onboarding;
mod settings;
mod singleton;
mod startup;
mod tray;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use model::{Config, Theme, CLOCK_KEY, PAGE_INDEX_NAME};
use std::{fs, path::PathBuf, process::ExitCode, sync::Arc};
use ulanzi_dock_vibeware::paths;

#[derive(Parser)]
#[command(
    name = "ulanzi-dock-vibeware",
    version,
    disable_help_subcommand = true,
    about = "Independent Ulanzi D200H controller participating in the VibeWare initiative"
)]
/// Parsed top-level command. Individual variants document their user-visible options.
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
/// Explicit entry points keep device access separate from file-only and UI commands.
enum Command {
    /// Show the command guide and examples.
    Help,
    /// Show the product version and VibeWare identity.
    Version,
    /// Show product and creator information.
    About {
        /// Open the Slint About dialog without connecting to the dock.
        #[arg(long)]
        gui: bool,
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Open the Slint configurator without connecting to the dock.
    Configure {
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Manage launch at Windows sign-in for the current user.
    Startup {
        /// Register this executable to start when you sign in to Windows.
        #[arg(long, conflicts_with = "disable")]
        enable: bool,
        /// Remove this executable's Windows sign-in entry.
        #[arg(long)]
        disable: bool,
    },
    /// Enumerate only the D200H consumer interface; sends no reports.
    Doctor,
    /// Print available visual themes.
    Themes,
    /// Save display brightness (0-100); a running controller applies it without restarting.
    Brightness {
        #[arg(value_parser = clap::value_parser!(u8).range(0..=100))]
        level: u8,
        /// Use this settings file instead of the executable's default settings.
        #[arg(long)]
        config: Option<PathBuf>,
    },
    /// Create a settings file with ten regular pages and a generated page index.
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
        /// Restrict the export to one theme.
        #[arg(long, value_enum)]
        theme: Option<Theme>,
        /// Restrict the export to one page (1-based, including the generated index).
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..))]
        page: Option<u64>,
        /// Preview brightness-bound banners (0-100); omitted exports neutral brightness faces.
        #[arg(long, value_parser = clap::value_parser!(u8).range(0..=100))]
        brightness: Option<u8>,
        /// Preview inactive application keys without querying focus or opening HID.
        #[arg(long)]
        inactive: bool,
        /// Include a generated theme/page/availability panel on the wide key.
        #[arg(long)]
        status_display: bool,
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
        /// Replace the firmware clock with theme, page, and availability status.
        #[arg(long, conflicts_with = "no_clock")]
        status_display: bool,
        /// Seconds between foreground checks (1-60); defaults to settings.json, initially 1.
        #[arg(long, value_parser = clap::value_parser!(u64).range(1..=60))]
        focus_poll_seconds: Option<u64>,
        /// Keep manual page navigation; still disable unavailable application keys.
        #[arg(long)]
        no_auto_page: bool,
        /// Disable foreground detection, inactive application artwork, and page switching.
        #[arg(long)]
        no_app_detection: bool,
        /// Run quietly in the tray without interactive first-run setup (Windows only).
        #[arg(long, conflicts_with = "oobe")]
        background: bool,
        /// Show the guided first-run introduction again.
        #[arg(long)]
        oobe: bool,
    },
}

// output: destination for PNGs; creates a directory tree per theme and page.
// config: validated page definitions.
// selected: optional theme filter; selected_page: optional one-based page filter.
// brightness: preview level override; inactive: render unavailable application keys.
// status_display: replace wide-key clock artwork with the generated status panel.
// Errors: image generation or filesystem failure.
fn export_art(
    output: &PathBuf,
    config: &Config,
    selected: Option<Theme>,
    selected_page: Option<u64>,
    brightness: Option<u8>,
    inactive: bool,
    status_display: bool,
) -> Result<()> {
    anyhow::ensure!(
        !cfg!(feature = "store-licensing"),
        "Artwork export is a development command. Use a community source build for exports."
    );
    anyhow::ensure!(
        selected_page.is_none_or(|page| page <= config.pages.len() as u64),
        "requested page is outside this configuration"
    );
    let mut count = 0;
    for theme in Theme::ALL
        .into_iter()
        .filter(|&theme| selected.is_none_or(|selected| selected == theme))
    {
        let mut themed_config = config.clone();
        themed_config.brightness = brightness.unwrap_or(50);
        themed_config.refresh_theme_selector(theme)?;
        for (page_number, page) in themed_config.pages.iter().enumerate() {
            if selected_page.is_some_and(|selected| selected as usize != page_number + 1) {
                continue;
            }
            let enabled = !inactive || page.executables.is_empty();
            let directory = output
                .join(theme.slug())
                .join(format!("page-{}", page_number + 1));
            fs::create_dir_all(&directory)
                .with_context(|| format!("creating {}", directory.display()))?;
            for index in 0..page.keys.len() {
                for (pressed, state) in [(false, "up"), (true, "down")] {
                    let bytes =
                        if index == CLOCK_KEY && (status_display || page.name == PAGE_INDEX_NAME) {
                            art::render_status(theme, page, enabled)?
                        } else {
                            art::render_available(theme, page, index, pressed, enabled)?
                        };
                    fs::write(directory.join(format!("key-{index:02}-{state}.png")), bytes)?;
                    count += 1;
                }
            }
        }
    }
    println!("Exported {} PNGs to {}", count, output.display());
    Ok(())
}

// Show errors before waiting for a key in a console created for this executable.
// Commands invoked from an existing terminal retain their normal exit behavior.
fn main() -> ExitCode {
    let standalone = console::owns_console();
    let launcher_error_dialog = cfg!(windows)
        && std::env::var_os("ULANZI_DOCK_ERROR_DIALOG").is_some_and(|value| value == "1");
    let control = Arc::new(tray::Control::new());
    let result = execute(Arc::clone(&control)).and_then(|()| {
        if control.restart_requested() {
            tray::restart_current_process()?;
        }
        Ok(())
    });
    let code = match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            if let Some(error) = error.downcast_ref::<clap::Error>() {
                if launcher_error_dialog && error.use_stderr() {
                    tray::show_error(&error.to_string());
                } else {
                    let _ = error.print();
                }
                ExitCode::from(error.exit_code() as u8)
            } else {
                let message = format!("UlanziDeck could not complete this operation:\n{error:#}");
                if launcher_error_dialog || control.is_background() {
                    tray::show_error(&message);
                } else {
                    eprintln!("\n{message}");
                }
                ExitCode::FAILURE
            }
        }
    };
    if standalone && !control.is_background() && !control.stopped_from_tray() {
        if let Err(error) = console::wait_for_close() {
            eprintln!("Could not wait for a key: {error:#}");
        }
    }
    code
}

// control: lifecycle state shared with the Windows tray while the controller runs.
// Errors: command-specific validation, filesystem, HID, rendering, or tray failure.
fn execute(control: Arc<tray::Control>) -> Result<()> {
    tray::wait_for_restart_parent(&control)?;
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() == 1 {
        match args[0].to_str() {
            Some("--help" | "-h") => {
                cli_pages::print_help(&Cli::command().render_long_help().to_string());
                return Ok(());
            }
            Some("--version" | "-V") => {
                cli_pages::print_version();
                return Ok(());
            }
            Some("--about") => {
                cli_pages::print_about();
                return Ok(());
            }
            _ => {}
        }
    }
    let cli = if args.is_empty() {
        cli_pages::print_about();
        let config = paths::default_settings()?;
        println!("\nStarting UlanziDeck.");
        #[cfg(target_os = "windows")]
        println!("The controller will stay in the system tray after setup.");
        println!("Settings: {}", config.display());
        println!("Press Ctrl+C to stop. For the command guide, use 'help'.\n");
        Cli {
            command: Command::Run {
                config,
                theme: None,
                seconds: None,
                no_actions: false,
                no_clock: false,
                status_display: true,
                focus_poll_seconds: None,
                no_auto_page: false,
                no_app_detection: false,
                background: false,
                oobe: false,
            },
        }
    } else {
        Cli::try_parse()?
    };
    match cli.command {
        Command::Help => cli_pages::print_help(&Cli::command().render_long_help().to_string()),
        Command::Version => cli_pages::print_version(),
        Command::About { gui: false, .. } => cli_pages::print_about(),
        Command::About { gui: true, config } => {
            gui::standalone(
                config.map_or_else(paths::default_settings, Ok)?,
                false,
                control,
            )?;
        }
        Command::Configure { config } => {
            gui::standalone(
                config.map_or_else(paths::default_settings, Ok)?,
                true,
                control,
            )?;
        }
        Command::Startup { enable, disable } => startup::manage(enable, disable)?,
        Command::Doctor => {
            let (_, found) = dock::discover()?;
            println!(
                "D200H consumer interface: {}",
                if found { "found" } else { "not found" }
            );
            println!("VID:PID 2207:0019 | usage page 0x0c | usage 1 | no reports sent");
            println!("Foreground process: {:?}", actions::foreground_process());
        }
        Command::Themes => {
            for theme in Theme::ALL {
                println!("{}", theme.slug());
            }
        }
        Command::Brightness { level, config } => {
            let config = config.map_or_else(paths::default_settings, Ok)?;
            settings::SettingsStore::load(&config)?.save_brightness(level)?;
            println!("Saved display brightness: {level}% | {}", config.display());
            println!("A controller using this settings file applies it on its next brightness poll while connected, or on its next connection. No HID reports are sent by this command.");
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
        Command::Render {
            config,
            output,
            theme,
            page,
            brightness,
            inactive,
            status_display,
        } => export_art(
            &output,
            &Config::load(&config)?,
            theme,
            page,
            brightness,
            inactive,
            status_display,
        )?,
        Command::Run {
            config,
            theme,
            seconds,
            no_actions,
            no_clock,
            status_display,
            focus_poll_seconds,
            no_auto_page,
            no_app_detection,
            background,
            oobe,
        } => {
            // Detach before settings, UI and HID initialization. A sign-in launch never
            // waits for console input, including when configuration loading fails.
            let _background_console = if background {
                anyhow::ensure!(cfg!(windows), "--background is available only on Windows");
                control.enter_background();
                Some(console::detach()?)
            } else {
                None
            };
            let _instance = singleton::acquire()?;
            let mut settings = Config::load(&config)?;
            // MSIX launches have redirected handles. Automatic console onboarding
            // returned false there, silently ending the app before its GUI existed.
            let console_welcome =
                oobe || (!cfg!(target_os = "windows") && !settings.setup_completed && !no_actions);
            if !background
                && console_welcome
                && !onboarding::show(&mut settings, &config, !args.is_empty())?
            {
                return Ok(());
            }
            let selected = theme.unwrap_or(settings.theme);
            let focus_poll_seconds =
                focus_poll_seconds.unwrap_or(settings.runtime.focus_poll_seconds);
            let detect_applications = settings.runtime.detect_applications && !no_app_detection;
            let auto_page = detect_applications && settings.runtime.auto_switch && !no_auto_page;
            #[cfg(target_os = "windows")]
            let ui_config = config.clone();
            #[cfg(target_os = "windows")]
            let ui_control = Arc::clone(&control);
            #[cfg(target_os = "windows")]
            let first_run = settings.is_first_run;
            let controller = move || {
                let options = controller::RunOptions {
                    config_path: config,
                    theme: selected,
                    seconds,
                    no_actions,
                    clock: !no_clock,
                    status_display,
                    focus_poll_seconds,
                    auto_page,
                    detect_applications,
                };
                controller::run(settings, options, control)
            };
            #[cfg(target_os = "windows")]
            gui::resident(ui_config, ui_control, first_run, controller)?;
            #[cfg(not(target_os = "windows"))]
            controller()?;
        }
    }
    Ok(())
}
