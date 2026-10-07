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

use image::imageops::FilterType;
use std::io::{self, IsTerminal};

const LOGO: &[u8] = include_bytes!("../assets/brand/vibeware-logo.png");
const NAME: &str = "ULANZI DOCK";
const LOGO_COLUMNS: usize = 20;

fn color_enabled() -> bool {
    io::stdout().is_terminal()
        && std::env::var_os("NO_COLOR").is_none()
        && std::env::var("TERM").map_or(true, |term| term != "dumb")
}

fn glyph(letter: char) -> [&'static str; 7] {
    match letter {
        'U' => [
            "#   #", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
        ],
        'L' => [
            "#    ", "#    ", "#    ", "#    ", "#    ", "#    ", "#####",
        ],
        'A' => [
            " ### ", "#   #", "#   #", "#####", "#   #", "#   #", "#   #",
        ],
        'N' => [
            "#   #", "##  #", "# # #", "#  ##", "#   #", "#   #", "#   #",
        ],
        'Z' => [
            "#####", "    #", "   # ", "  #  ", " #   ", "#    ", "#####",
        ],
        'I' => [
            "#####", "  #  ", "  #  ", "  #  ", "  #  ", "  #  ", "#####",
        ],
        'D' => [
            "#### ", "#   #", "#   #", "#   #", "#   #", "#   #", "#### ",
        ],
        'O' => [
            " ### ", "#   #", "#   #", "#   #", "#   #", "#   #", " ### ",
        ],
        'C' => [
            " ####", "#    ", "#    ", "#    ", "#    ", "#    ", " ####",
        ],
        'K' => [
            "#   #", "#  # ", "# #  ", "##   ", "# #  ", "#  # ", "#   #",
        ],
        _ => ["     "; 7],
    }
}

fn banner_rows() -> Vec<(String, usize)> {
    if !color_enabled() {
        let name = "UlanziDock VibeWare";
        return vec![(name.to_owned(), name.chars().count())];
    }
    let banner_width = NAME.chars().count() * 6 - 1;
    if crate::console::columns().is_some_and(|columns| columns <= banner_width) {
        return vec![
            (format!("\x1b[1;38;2;80;153;213m{NAME}\x1b[0m"), NAME.len()),
            ("VibeWare".to_owned(), 8),
        ];
    }
    let palette = [
        (80, 153, 213),
        (80, 153, 213),
        (91, 184, 219),
        (91, 184, 219),
        (173, 210, 226),
        (232, 215, 181),
        (232, 215, 181),
    ];
    let mut rows = Vec::new();
    for (row, (red, green, blue)) in palette.into_iter().enumerate() {
        let line = NAME
            .chars()
            .map(|letter| glyph(letter)[row])
            .collect::<Vec<_>>()
            .join(" ");
        rows.push((
            format!(
                "\x1b[1;38;2;{red};{green};{blue}m{}\x1b[0m",
                line.replace('#', "█")
            ),
            banner_width,
        ));
    }
    let tagline = "VibeWare  ·  Human intent. AI implementation. Accountable human review.";
    rows.push((
        format!("\x1b[38;2;135;171;199m{tagline}\x1b[0m"),
        tagline.chars().count(),
    ));
    rows
}

fn print_banner() {
    println!();
    for (line, _) in banner_rows() {
        println!("{line}");
    }
}

fn blended(pixel: image::Rgba<u8>) -> (u8, u8, u8) {
    let alpha = u16::from(pixel[3]);
    let blend = |channel: u8, background: u8| {
        ((u16::from(channel) * alpha + u16::from(background) * (255 - alpha)) / 255) as u8
    };
    (
        blend(pixel[0], 13),
        blend(pixel[1], 25),
        blend(pixel[2], 40),
    )
}

fn logo_rows() -> Option<Vec<String>> {
    if !color_enabled() {
        return None;
    }
    let image = image::load_from_memory(LOGO).ok()?;
    let width = image.width();
    let height = image.height();
    let symbol = image.crop_imm(
        width * 23 / 100,
        height * 15 / 100,
        width * 54 / 100,
        height * 54 / 100,
    );
    let pixels = symbol.resize_exact(20, 20, FilterType::Triangle).to_rgba8();
    let mut rows = Vec::new();
    for y in (0..20).step_by(2) {
        let mut line = String::new();
        for x in 0..20 {
            let (r1, g1, b1) = blended(*pixels.get_pixel(x, y));
            let (r2, g2, b2) = blended(*pixels.get_pixel(x, y + 1));
            line.push_str(&format!(
                "\x1b[38;2;{r1};{g1};{b1}m\x1b[48;2;{r2};{g2};{b2}m▀"
            ));
        }
        line.push_str("\x1b[0m");
        rows.push(line);
    }
    Some(rows)
}

fn print_identity() {
    let banner = banner_rows();
    let logo = logo_rows();
    let columns = crate::console::columns().unwrap_or(80);
    let banner_width = banner.iter().map(|(_, width)| *width).max().unwrap_or(0);
    println!();
    if let Some(logo) = logo.filter(|_| columns > LOGO_COLUMNS) {
        // Leave one column unused to avoid a terminal's automatic line wrap.
        let logo_left = columns - LOGO_COLUMNS - 1;
        if columns >= banner_width + LOGO_COLUMNS + 5 {
            for row in 0..banner.len().max(logo.len()) {
                let (left, width) = banner
                    .get(row)
                    .map(|(text, width)| (text.as_str(), *width))
                    .unwrap_or(("", 0));
                let right = logo.get(row).map(String::as_str).unwrap_or("");
                println!("{left}{}{right}", " ".repeat(logo_left - width));
            }
        } else {
            for (line, _) in banner {
                println!("{line}");
            }
            for line in logo {
                println!("{}{line}", " ".repeat(logo_left));
            }
        }
    } else {
        for (line, _) in banner {
            println!("{line}");
        }
    }
}

pub fn print_help(help: &str) {
    print_banner();
    println!("\nControl your Ulanzi D200H from the terminal.\n");
    print!("{help}");
    println!("\nExamples:");
    println!("  ulanzi-dock-vibeware init");
    println!("  ulanzi-dock-vibeware --about");
    println!("  ulanzi-dock-vibeware run --status-display");
    println!("  ulanzi-dock-vibeware run --oobe");
    println!("  ulanzi-dock-vibeware run --focus-poll-seconds 1");
    println!("  ulanzi-dock-vibeware run --no-auto-page");
    println!("  ulanzi-dock-vibeware startup --enable");
    println!("  ulanzi-dock-vibeware startup --disable");
}

pub fn print_version() {
    print_identity();
    println!("\nVersion {}", env!("CARGO_PKG_VERSION"));
    println!("Creator  Umberto Giacobbi");
    println!("License  MIT");
    println!("Manifesto https://umbertogiacobbi.biz/vibeware/manifesto");
}

pub fn print_about() {
    print_identity();
    println!("\nA resident CLI controller for the Ulanzi D200H.");
    println!(
        "It displays themed button art and handles actions from the dock's own button events."
    );
    println!("\nCreator      Umberto Giacobbi");
    println!("Version      {}", env!("CARGO_PKG_VERSION"));
    println!("License      MIT");
    println!("Source       https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware");
    println!("Manifesto    https://umbertogiacobbi.biz/vibeware/manifesto");
}

pub fn print_setup_intro(show_branding: bool) {
    if show_branding {
        print_identity();
    }
    println!();
    setup_text(
        &format!("{} Welcome to UlanziDock VibeWare", setup_icon("✨", "*")),
        Tone::Accent,
    );
    setup_text(
        "Your shortcuts, one key away. Give your Ulanzi D200H six pages of themed controls.",
        Tone::Primary,
    );
    setup_text("Two short steps, then you can get started.", Tone::Muted);
    println!();
    setup_text("A side project by Umberto Giacobbi. Provided as-is, without guarantees of compatibility or behavior.", Tone::Warning);
    println!();
    setup_link(
        "🔗",
        "Source & guide",
        "https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware",
    );
    setup_link(
        "💙",
        "VibeWare manifesto",
        "https://umbertogiacobbi.biz/vibeware/manifesto",
    );
    setup_link(
        "❤️",
        "Made with love by Umberto",
        "https://umbertogiacobbi.biz",
    );
}

// Semantic colors and emoji-led sections follow PromptMeUp's first-run view,
// using the VibeWare palette and copy rather than its product-specific screens.
#[derive(Clone, Copy)]
pub(crate) enum Tone {
    Accent,
    Primary,
    Muted,
    Info,
    Success,
    Warning,
}

pub(crate) fn styled(text: &str, tone: Tone) -> String {
    if !color_enabled() {
        return text.to_owned();
    }
    let (r, g, b, bold) = match tone {
        Tone::Accent => (80, 153, 213, true),
        Tone::Primary => (232, 238, 243, false),
        Tone::Muted => (135, 155, 173, false),
        Tone::Info => (91, 184, 219, false),
        Tone::Success => (113, 214, 144, true),
        Tone::Warning => (232, 181, 96, false),
    };
    format!(
        "\x1b[{};38;2;{r};{g};{b}m{text}\x1b[0m",
        if bold { 1 } else { 22 }
    )
}

pub(crate) fn setup_icon<'a>(emoji: &'a str, fallback: &'a str) -> &'a str {
    if io::stdout().is_terminal() && std::env::var("TERM").map_or(true, |term| term != "dumb") {
        emoji
    } else {
        fallback
    }
}

fn character_width(character: char) -> usize {
    match character as u32 {
        0xfe0e | 0xfe0f | 0x200d => 0,
        0x2600..=0x27ff
        | 0x1f000..=0x1faff
        | 0x2e80..=0xa4cf
        | 0xac00..=0xd7a3
        | 0xf900..=0xfaff
        | 0xff01..=0xff60 => 2,
        _ => 1,
    }
}

fn text_width(text: &str) -> usize {
    text.chars().map(character_width).sum()
}

pub(crate) fn clipped(text: &str, width: usize) -> String {
    let mut used = 0;
    text.chars()
        .take_while(|&character| {
            used += character_width(character);
            used <= width
        })
        .collect()
}

pub(crate) fn setup_text(text: &str, tone: Tone) {
    let width = crate::console::columns()
        .unwrap_or(80)
        .saturating_sub(3)
        .clamp(1, 104);
    let mut line = String::new();
    for word in text.split_whitespace() {
        if !line.is_empty() && text_width(&line) + 1 + text_width(word) > width {
            println!("  {}", styled(&line, tone));
            line.clear();
        }
        if !line.is_empty() {
            line.push(' ');
        }
        for character in word.chars() {
            if !line.is_empty() && text_width(&line) + character_width(character) > width {
                println!("  {}", styled(&line, tone));
                line.clear();
            }
            line.push(character);
        }
    }
    println!("  {}", styled(&line, tone));
}

pub(crate) fn setup_section(emoji: &str, title: &str, hint: &str) {
    println!();
    setup_text(&format!("{} {title}", setup_icon(emoji, ">")), Tone::Accent);
    let width = crate::console::columns()
        .unwrap_or(80)
        .saturating_sub(3)
        .min(88);
    println!(
        "  {}",
        styled(
            &if color_enabled() {
                "─".repeat(width)
            } else {
                "-".repeat(width)
            },
            Tone::Accent
        )
    );
    setup_text(hint, Tone::Muted);
    println!();
}

fn setup_link(emoji: &str, label: &str, url: &str) {
    let label = format!("{} {label}", setup_icon(emoji, ">"));
    if color_enabled()
        && text_width(&label) < crate::console::columns().unwrap_or(80).saturating_sub(2)
    {
        println!(
            "  \x1b]8;;{url}\x1b\\\x1b[4m{}\x1b]8;;\x1b\\",
            styled(&label, Tone::Info)
        );
    } else {
        setup_text(&label, Tone::Info);
    }
    setup_text(url, Tone::Muted);
}

pub(crate) fn setup_choice(label: &str, tone: Tone, selected: bool) {
    let width = crate::console::columns().unwrap_or(80).saturating_sub(5);
    let label = clipped(label, width);
    let row = format!("  {} {label}", if selected { ">" } else { " " });
    if selected && color_enabled() {
        print!("\x1b[48;2;22;43;63m");
        // styled() restores the background at the end of this row.
        println!("{}", styled(&row, tone));
    } else {
        println!("{}", styled(&row, tone));
    }
}
