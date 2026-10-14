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

mod actions;
mod applications;
mod art;
mod cli_pages;
mod console;
mod dock;
mod gui;
mod gui_settings;
mod model;
mod onboarding;
mod singleton;
mod startup;
mod tray;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use model::{Config, Theme, CLOCK_KEY, NEXT_KEY, PAGE_INDEX_NAME};
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(
    name = "ulanzi-dock-vibeware",
    version,
    disable_help_subcommand = true,
    about = "VibeWare standalone controller for the Ulanzi D200H"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
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
    /// Create a settings file with nine regular pages and a generated page index.
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
        /// Seconds between foreground checks (1-60); defaults to the YAML setting, initially 1.
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

// Refresh only when the current page gains or loses its foreground application.
// Losing focus cancels a held app-specific key, even if focus returns before release.
fn refresh_availability(
    dock: &dock::Dock,
    page: &model::PageConfig,
    theme: Theme,
    enabled: &mut bool,
    held: &mut Option<usize>,
    status_display: bool,
    foreground: &actions::ForegroundProcess,
) -> Result<()> {
    let next = actions::page_available(page, foreground);
    if next != *enabled {
        if !next && *held != Some(NEXT_KEY) && *held != Some(CLOCK_KEY) {
            *held = None;
        }
        dock.refresh_availability(page, theme, *held, next, status_display)?;
        *enabled = next;
        println!(
            "Page availability: {} | {}",
            page.name,
            if next { "active" } else { "inactive" }
        );
    }
    Ok(())
}

// control: shared stop/restart state, checked during the retry delay.
// started: original run start, including all time spent disconnected.
// seconds: optional total run limit.
#[cfg(target_os = "windows")]
fn wait_for_connection_retry(control: &tray::Control, started: Instant, seconds: Option<u64>) {
    // Keep Exit/Restart responsive while waiting two seconds between connection attempts.
    for _ in 0..20 {
        if !control.is_running()
            || seconds.is_some_and(|limit| started.elapsed() >= Duration::from_secs(limit))
        {
            return;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

// config: validated page and action definitions.
// config_path: settings file used to persist a theme selected on the final page.
// theme: initial visual family.
// seconds: optional bounded diagnostic duration.
// no_actions: disable shortcut and media dispatch.
// clock: periodically update the wide bottom clock window.
// focus_poll_seconds: interval between foreground application checks.
// status_display: replace the firmware clock with the optional status panel.
// auto_page: select an application page when its process gains foreground focus.
// detect_applications: query foreground metadata and gate unavailable application keys.
// control: shared lifecycle state for Ctrl+C and the Windows tray menu.
// Errors: non-transport display/configuration failures; device failures on non-Windows.
fn run(
    mut config: Config,
    config_path: &Path,
    mut theme: Theme,
    seconds: Option<u64>,
    no_actions: bool,
    clock: bool,
    status_display: bool,
    focus_poll_seconds: u64,
    auto_page: bool,
    detect_applications: bool,
    control: Arc<tray::Control>,
) -> Result<()> {
    config.refresh_theme_selector(theme)?;
    let stop = Arc::clone(&control);
    ctrlc::set_handler(move || stop.stop())?;
    let foreground = if detect_applications {
        actions::foreground_process()
    } else {
        actions::ForegroundProcess::Unsupported
    };
    let mut last_known_process = if matches!(foreground, actions::ForegroundProcess::Known(_)) {
        Some(foreground.clone())
    } else {
        None
    };
    let mut page = if auto_page {
        actions::matching_page(&config, &foreground).unwrap_or_else(|| config.home_page())
    } else {
        config.home_page()
    };
    let mut held = None;
    let mut enabled = actions::page_available(&config.pages[page], &foreground);
    let started = Instant::now();
    #[cfg(target_os = "windows")]
    let mut last_connection_error = String::new();
    // Preserve page/theme in this process across USB sessions. The original run deadline
    // and tray lifecycle remain in effect while the dock is absent or initializing.
    loop {
        let dock = loop {
            if !control.is_running()
                || seconds.is_some_and(|limit| started.elapsed() >= Duration::from_secs(limit))
            {
                return Ok(());
            }
            let attempt = (|| -> Result<dock::Dock> {
                let (api, _) = dock::discover()?;
                let dock = dock::Dock::open(&api)?;
                config.brightness = Config::load_brightness(config_path)?;
                config.refresh_key_availability();
                let foreground = if detect_applications {
                    actions::foreground_process()
                } else {
                    actions::ForegroundProcess::Unsupported
                };
                if matches!(foreground, actions::ForegroundProcess::Known(_)) {
                    if auto_page && config.pages[page].name != PAGE_INDEX_NAME {
                        page = actions::matching_page(&config, &foreground)
                            .unwrap_or_else(|| config.home_page());
                    }
                    last_known_process = Some(foreground.clone());
                }
                enabled = actions::page_available(&config.pages[page], &foreground);
                dock.show(&config.pages[page], theme, None, enabled, status_display)?;
                dock.set_brightness(config.brightness)?;
                if clock && !status_display && config.pages[page].name != PAGE_INDEX_NAME {
                    dock.clock()?;
                }
                dock.discard_input()?;
                Ok(dock)
            })();
            match attempt {
                Ok(dock) => {
                    #[cfg(target_os = "windows")]
                    last_connection_error.clear();
                    break dock;
                }
                Err(error) => {
                    if !dock::is_connection_error(&error) {
                        return Err(error);
                    }
                    #[cfg(not(target_os = "windows"))]
                    return Err(error);
                    #[cfg(target_os = "windows")]
                    {
                        let message = format!("{error:#}");
                        if message != last_connection_error {
                            eprintln!("Waiting for D200H: {message}");
                            last_connection_error = message;
                        }
                        wait_for_connection_retry(&control, started, seconds);
                    }
                }
            }
        };
        let session = (|| -> Result<()> {
            let mut last_clock = Instant::now();
            let mut last_focus = Instant::now();
            let mut last_brightness = Instant::now();
            let mut brightness_error = String::new();
            println!(
                "D200H active | page: {} | theme: {} | Ctrl+C to stop",
                config.pages[page].name,
                theme.slug()
            );
            println!(
                "Foreground polling: {focus_poll_seconds}s | automatic pages: {}",
                if auto_page { "on" } else { "off" }
            );
            while control.is_running()
                && seconds.is_none_or(|limit| started.elapsed() < Duration::from_secs(limit))
            {
                if last_brightness.elapsed() >= Duration::from_millis(500) {
                    match Config::load_brightness(config_path) {
                        Ok(level) => {
                            brightness_error.clear();
                            if level != config.brightness {
                                dock.set_brightness(level)?;
                                config.brightness = level;
                                config.refresh_key_availability();
                                if held
                                    .is_some_and(|index| !config.pages[page].key_available(index))
                                {
                                    held = None;
                                }
                                dock.refresh_brightness_keys(
                                    &config.pages[page],
                                    theme,
                                    held,
                                    enabled,
                                )?;
                                println!("Brightness USB write completed: {level}%");
                            }
                        }
                        Err(error) => {
                            let message = format!("{error:#}");
                            if message != brightness_error {
                                eprintln!("Keeping current brightness: {message}");
                                brightness_error = message;
                            }
                        }
                    }
                    last_brightness = Instant::now();
                }
                if detect_applications
                    && last_focus.elapsed() >= Duration::from_secs(focus_poll_seconds)
                {
                    let foreground = actions::foreground_process();
                    if matches!(foreground, actions::ForegroundProcess::Known(_))
                        && last_known_process.as_ref() != Some(&foreground)
                    {
                        if auto_page && config.pages[page].name != PAGE_INDEX_NAME {
                            // Known applications without a dedicated page return to Windows / media.
                            let next_page = actions::matching_page(&config, &foreground)
                                .unwrap_or_else(|| config.home_page());
                            if next_page != page {
                                // A release from the previous page must never dispatch a new page's action.
                                held = None;
                                page = next_page;
                                enabled = actions::page_available(&config.pages[page], &foreground);
                                dock.show(
                                    &config.pages[page],
                                    theme,
                                    None,
                                    enabled,
                                    status_display,
                                )?;
                                println!("Automatic page: {}", config.pages[page].name);
                            }
                        }
                        // Unknown focus does not reset the manual choice or cause transient bouncing.
                        last_known_process = Some(foreground.clone());
                    }
                    refresh_availability(
                        &dock,
                        &config.pages[page],
                        theme,
                        &mut enabled,
                        &mut held,
                        status_display,
                        &foreground,
                    )?;
                    last_focus = Instant::now();
                }
                if (clock || status_display || config.pages[page].name == PAGE_INDEX_NAME)
                    && last_clock.elapsed() >= Duration::from_secs(1)
                {
                    if status_display || config.pages[page].name == PAGE_INDEX_NAME {
                        dock.background()?;
                    } else {
                        dock.clock()?;
                    }
                    last_clock = Instant::now();
                }
                if let Some(event) = dock.read_button(Duration::from_millis(100))? {
                    if event.index == CLOCK_KEY {
                        if event.pressed {
                            held = Some(CLOCK_KEY);
                        } else if held == Some(CLOCK_KEY) {
                            held = None;
                            page = config.page_index_position();
                            enabled = true;
                            dock.show(&config.pages[page], theme, None, enabled, status_display)?;
                            println!("Page: {}", config.pages[page].name);
                            last_focus = Instant::now();
                        }
                        continue;
                    }
                    refresh_availability(
                        &dock,
                        &config.pages[page],
                        theme,
                        &mut enabled,
                        &mut held,
                        status_display,
                        &if detect_applications {
                            actions::foreground_process()
                        } else {
                            actions::ForegroundProcess::Unsupported
                        },
                    )?;
                    if event.pressed {
                        if (!enabled && event.index != NEXT_KEY)
                            || !config.pages[page].key_available(event.index)
                        {
                            continue;
                        }
                        if held != Some(event.index) {
                            held = Some(event.index);
                            dock.show_key(&config.pages[page], theme, event.index, true, enabled)?;
                        }
                        continue;
                    }
                    if held != Some(event.index) {
                        continue;
                    }
                    held = None;
                    if !config.pages[page].key_available(event.index) {
                        continue;
                    }
                    let mut page_changed = false;
                    if event.index == NEXT_KEY {
                        page = config.next_enabled_page(page);
                        println!("Page: {}", config.pages[page].name);
                        page_changed = true;
                    } else if let Some(target) = config.pages[page].keys[event.index]
                        .action
                        .strip_prefix("page:go:")
                    {
                        if let Some(destination) = config
                            .pages
                            .iter()
                            .position(|page| page.enabled && page.name == target)
                        {
                            page = destination;
                            page_changed = true;
                            println!("Page: {}", config.pages[page].name);
                        }
                    } else if let Some(slug) = config.pages[page].keys[event.index]
                        .action
                        .strip_prefix("theme:")
                    {
                        let selected = Theme::from_slug(slug)
                            .with_context(|| format!("unknown selected theme '{slug}'"))?;
                        theme = selected;
                        config.theme = selected;
                        config.refresh_theme_selector(selected)?;
                        println!("Theme: {}", selected.slug());
                        page = config.home_page();
                        println!("Page: {}", config.pages[page].name);
                        page_changed = true;
                        if let Err(error) = control
                            .lock_settings()
                            .and_then(|_lock| config.save_selected_theme(config_path))
                        {
                            eprintln!("Could not save theme selection: {error:#}");
                        }
                    } else if !no_actions
                        && enabled
                        && matches!(
                            config.pages[page].keys[event.index].action.as_str(),
                            "brightness:up" | "brightness:down"
                        )
                    {
                        let increase =
                            config.pages[page].keys[event.index].action == "brightness:up";
                        let changed = (|| -> Result<u8> {
                            let _lock = control.lock_settings()?;
                            let mut settings = gui_settings::Settings::load(config_path)?;
                            // Read the latest saved value, including a recent CLI change.
                            let current = settings.config.brightness;
                            let level = if increase {
                                current.saturating_add(10).min(100)
                            } else {
                                current.saturating_sub(10)
                            };
                            if level != current {
                                settings.save_brightness(level)?;
                            }
                            dock.set_brightness(level)?;
                            Ok(level)
                        })();
                        match changed {
                            Ok(level) => {
                                config.brightness = level;
                                config.refresh_key_availability();
                                dock.refresh_brightness_keys(
                                    &config.pages[page],
                                    theme,
                                    None,
                                    enabled,
                                )?;
                                println!("Brightness USB write completed: {level}%");
                            }
                            Err(error) if dock::is_connection_error(&error) => return Err(error),
                            Err(error) => eprintln!("Could not change brightness: {error:#}"),
                        }
                    } else if !no_actions
                        && enabled
                        && (!detect_applications
                            || actions::page_available(
                                &config.pages[page],
                                &actions::foreground_process(),
                            ))
                    {
                        let key = &config.pages[page].keys[event.index];
                        if let Err(error) = actions::execute(&key.action) {
                            eprintln!("Action '{}' failed: {error:#}", key.label);
                        }
                    }
                    if page_changed {
                        enabled = !detect_applications
                            || actions::page_available(
                                &config.pages[page],
                                &actions::foreground_process(),
                            );
                        dock.show(&config.pages[page], theme, None, enabled, status_display)?;
                        if clock && !status_display && config.pages[page].name != PAGE_INDEX_NAME {
                            dock.clock()?;
                        }
                        last_focus = Instant::now();
                    } else {
                        dock.show_key(&config.pages[page], theme, event.index, false, enabled)?;
                    }
                }
            }
            Ok(())
        })();
        // Drop the failed HID handle before discovery opens a new device interface.
        drop(dock);
        match session {
            Ok(()) => break,
            Err(error) => {
                if !dock::is_connection_error(&error) {
                    return Err(error);
                }
                #[cfg(not(target_os = "windows"))]
                return Err(error);
                #[cfg(target_os = "windows")]
                {
                    // Never reuse a held press or replay an action from the interrupted session.
                    held = None;
                    eprintln!("D200H disconnected; waiting to reconnect: {error:#}");
                    wait_for_connection_retry(&control, started, seconds);
                }
            }
        }
    }
    println!("Stopped. The dock may clear its display when the program exits.");
    Ok(())
}

// Show errors before waiting for a key in a console created for this executable.
// Commands invoked from an existing terminal retain their normal exit behavior.
fn main() -> ExitCode {
    let standalone = console::owns_console();
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
                let _ = error.print();
                ExitCode::from(error.exit_code() as u8)
            } else {
                let message = format!("UlanziDock could not complete this operation:\n{error:#}");
                if control.is_background() {
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

// An Explorer launch uses the EXE folder; the local artifacts EXE shares the
// repository's settings.json with the Windows sign-in entry.
fn launch_config() -> Result<PathBuf> {
    let executable = std::env::current_exe().context("locating the executable")?;
    let directory = executable
        .parent()
        .context("locating the executable folder")?;
    let directory = if directory
        .file_name()
        .is_some_and(|name| name == "artifacts")
    {
        directory
            .parent()
            .filter(|parent| parent.join("Cargo.toml").is_file())
            .unwrap_or(directory)
    } else {
        directory
    };
    Ok(directory.join("settings.json"))
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
        let config = launch_config()?;
        println!("\nStarting UlanziDock.");
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
            gui::standalone(config.map_or_else(launch_config, Ok)?, false, control)?;
        }
        Command::Configure { config } => {
            gui::standalone(config.map_or_else(launch_config, Ok)?, true, control)?;
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
            let config = config.map_or_else(launch_config, Ok)?;
            gui_settings::Settings::load(&config)?.save_brightness(level)?;
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
            let mut settings = Config::load(&config)?;
            if !background
                && (oobe || !settings.setup_completed && !no_actions)
                && !onboarding::show(&mut settings, &config, !args.is_empty())?
            {
                return Ok(());
            }
            let selected = theme.unwrap_or(settings.theme);
            let focus_poll_seconds =
                focus_poll_seconds.unwrap_or(settings.runtime.focus_poll_seconds);
            let detect_applications = settings.runtime.detect_applications && !no_app_detection;
            let auto_page = detect_applications && settings.runtime.auto_switch && !no_auto_page;
            singleton::replace_existing()?;
            #[cfg(target_os = "windows")]
            let ui_config = config.clone();
            #[cfg(target_os = "windows")]
            let ui_control = Arc::clone(&control);
            let controller = move || {
                run(
                    settings,
                    &config,
                    selected,
                    seconds,
                    no_actions,
                    !no_clock,
                    status_display,
                    focus_poll_seconds,
                    auto_page,
                    detect_applications,
                    control,
                )
            };
            #[cfg(target_os = "windows")]
            gui::resident(ui_config, ui_control, controller)?;
            #[cfg(not(target_os = "windows"))]
            controller()?;
        }
    }
    Ok(())
}
