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

use crate::model::{PageConfig, Theme, CLOCK_KEY, NEXT_KEY};
use anyhow::{Context, Result};
use fontdue::{Font, FontSettings};
use image::{imageops, DynamicImage, GrayImage, ImageFormat, Luma, Rgba, RgbaImage};
use std::{
    collections::HashMap,
    io::Cursor,
    sync::{Mutex, OnceLock},
};

const SIDE: u32 = 196;
static SOURCES: OnceLock<[DynamicImage; 3]> = OnceLock::new();
static ICON_CACHE: OnceLock<Mutex<HashMap<(usize, usize), GrayImage>>> = OnceLock::new();
static UNAVAILABLE_OVERLAY: OnceLock<RgbaImage> = OnceLock::new();
static LABEL_FONT: OnceLock<Font> = OnceLock::new();
static SPOTIFY_SYMBOLS: OnceLock<HashMap<usize, GrayImage>> = OnceLock::new();
static OFFICE_SYMBOLS: OnceLock<HashMap<(String, usize), GrayImage>> = OnceLock::new();

// page_name: Word, PowerPoint, or Excel; used for application-specific command symbols.
// index: physical command key, excluding navigation and the clock.
// Returns original native artwork, with no Microsoft logo or third-party raster dependency.
fn office_symbol(page_name: &str, index: usize) -> GrayImage {
    const SCALE: u32 = 4;
    let mut canvas = RgbaImage::new(118 * SCALE, 99 * SCALE);
    for y in 0..canvas.height() {
        for x in 0..canvas.width() {
            let px = x as f32 / SCALE as f32;
            let py = y as f32 / SCALE as f32;
            let rect = |left: f32, top: f32, right: f32, bottom: f32| {
                px >= left && px <= right && py >= top && py <= bottom
            };
            let line = |ax: f32, ay: f32, bx: f32, by: f32| {
                let dx = bx - ax;
                let dy = by - ay;
                let t = (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0., 1.);
                (px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2) <= 3.0_f32.powi(2)
            };
            let document = rect(29., 10., 90., 87.) && !rect(35., 16., 84., 81.);
            let slide = rect(15., 14., 104., 69.) && !rect(21., 20., 98., 63.)
                || line(59., 69., 59., 84.)
                || line(40., 85., 78., 85.);
            let filled = match index {
                0 => {
                    rect(24., 10., 94., 87.)
                        && !rect(32., 45., 86., 80.)
                        && !rect(39., 10., 77., 33.)
                }
                1 => {
                    line(17., 32., 17., 83.)
                        || line(17., 83., 101., 83.)
                        || line(101., 83., 101., 36.)
                        || line(17., 32., 48., 32.)
                        || line(48., 32., 58., 42.)
                        || line(58., 42., 101., 42.)
                }
                2 => document || line(59., 34., 59., 65.) || line(44., 49., 74., 49.),
                3 => {
                    (rect(16., 36., 102., 70.) && !rect(22., 42., 96., 64.))
                        || (rect(31., 12., 87., 87.) && !rect(37., 18., 81., 81.))
                        || line(42., 72., 76., 72.)
                }
                5 | 6 => {
                    let qx = if index == 6 { 118. - px } else { px };
                    let radius = ((qx - 63.).powi(2) + (py - 54.).powi(2)).sqrt();
                    (radius > 24. && radius < 31. && (qx > 63. || py < 54.))
                        || (qx > 24. && qx < 48. && (py - 31.).abs() < (48. - qx) * 0.65)
                }
                7 => {
                    let radius = ((px - 50.).powi(2) + (py - 40.).powi(2)).sqrt();
                    (radius > 23. && radius < 29.) || line(70., 60., 95., 85.)
                }
                11 if page_name == "PowerPoint" => {
                    (rect(17., 15., 86., 65.) && !rect(23., 21., 80., 59.))
                        || (rect(36., 34., 105., 84.) && !rect(42., 40., 99., 78.))
                }
                8..=12 if page_name == "PowerPoint" => {
                    slide
                        || match index {
                            8 => px > 47. && px < 76. && (py - 42.).abs() < (76. - px) * 0.6,
                            9 => line(43., 32., 68., 42.) || line(68., 42., 43., 52.),
                            10 => line(59., 28., 59., 56.) || line(45., 42., 73., 42.),
                            _ => rect(46., 29., 72., 55.),
                        }
                }
                8 if page_name == "Excel" => {
                    line(36., 14., 88., 14.)
                        || line(36., 14., 65., 48.)
                        || line(65., 48., 36., 82.)
                        || line(36., 82., 88., 82.)
                }
                9 if page_name == "Word" => {
                    line(67., 15., 50., 81.) || line(45., 15., 83., 15.) || line(33., 81., 71., 81.)
                }
                10 if page_name == "Word" => line(33., 86., 88., 86.),
                9 if page_name == "Excel" => {
                    document
                        || line(29., 36., 90., 36.)
                        || line(29., 61., 90., 61.)
                        || line(49., 10., 49., 87.)
                        || line(69., 10., 69., 87.)
                }
                10 if page_name == "Excel" => {
                    document
                        || line(29., 32., 90., 32.)
                        || line(44., 5., 44., 21.)
                        || line(74., 5., 74., 21.)
                        || rect(43., 46., 51., 54.)
                        || rect(64., 46., 72., 54.)
                }
                11 | 12
                    if index == 11 && page_name == "Excel"
                        || index == 12 && page_name == "Word" =>
                {
                    document
                        || line(49., 67., 88., 28.)
                        || line(55., 73., 94., 34.)
                        || line(49., 67., 55., 73.)
                }
                12 if page_name == "Excel" => {
                    line(19., 17., 99., 17.)
                        || line(19., 17., 49., 53.)
                        || line(99., 17., 69., 53.)
                        || line(49., 53., 49., 82.)
                        || line(69., 53., 69., 74.)
                        || line(49., 82., 69., 74.)
                }
                _ => false,
            };
            if filled {
                canvas.put_pixel(x, y, Rgba([255, 255, 255, 255]));
            }
        }
    }
    let glyph = match (page_name, index) {
        ("Word", 8) => "B",
        ("Word", 10) => "U",
        ("Word", 11) => "Aa",
        _ => "",
    };
    if !glyph.is_empty() {
        draw_panel_text(
            &mut canvas,
            glyph,
            31 * SCALE as i32,
            75 * SCALE as i32,
            62. * SCALE as f32,
            [255; 3],
        );
    }
    let small = imageops::resize(&canvas, 118, 99, imageops::FilterType::Lanczos3);
    GrayImage::from_fn(118, 99, |x, y| Luma([small.get_pixel(x, y)[3]]))
}

// Extend the native pictogram system; themes supply the approved internal gradients.
fn spotify_symbols() -> &'static HashMap<usize, GrayImage> {
    SPOTIFY_SYMBOLS.get_or_init(|| {
        let mut symbols = HashMap::new();
        for index in [3, 5, 7, 8, 9, 10, 11, 12] {
            let mut mask = GrayImage::new(118 * 4, 99 * 4);
            for y in 0..mask.height() {
                for x in 0..mask.width() {
                    let px = x as f32 / 4.0;
                    let py = y as f32 / 4.0;
                    let line = |ax: f32, ay: f32, bx: f32, by: f32| {
                        let dx = bx - ax;
                        let dy = by - ay;
                        let t = (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy))
                            .clamp(0.0, 1.0);
                        (px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2) < 3.7_f32.powi(2)
                    };
                    let filled = match index {
                        3 => {
                            line(23., 25., 92., 74.)
                                || line(23., 74., 92., 25.)
                                || line(78., 13., 94., 25.)
                                || line(94., 25., 78., 37.)
                                || line(78., 62., 94., 74.)
                                || line(94., 74., 78., 86.)
                        }
                        5 => {
                            line(23., 29., 94., 29.)
                                || line(94., 29., 94., 58.)
                                || line(94., 71., 23., 71.)
                                || line(23., 71., 23., 42.)
                                || line(79., 17., 94., 29.)
                                || line(94., 29., 79., 41.)
                                || line(38., 59., 23., 71.)
                                || line(23., 71., 38., 83.)
                        }
                        7 => {
                            line(25., 23., 25., 79.)
                                || line(45., 23., 45., 79.)
                                || line(65., 23., 65., 79.)
                                || line(84., 24., 98., 77.)
                                || line(25., 32., 45., 32.)
                                || line(25., 70., 45., 70.)
                        }
                        8 | 11 => {
                            let hx = (px - 59.) / 27.;
                            let hy = (53. - py) / 25.;
                            let heart =
                                (hx * hx + hy * hy - 1.).powi(3) - hx * hx * hy.powi(3) <= 0.;
                            heart
                                || index == 11
                                    && (line(93., 13., 93., 33.) || line(83., 23., 103., 23.))
                        }
                        9 => [27., 49., 71.].into_iter().any(|row| {
                            line(42., row, 96., row)
                                || (px - 25.).powi(2) + (py - row).powi(2) < 5.5_f32.powi(2)
                        }),
                        10 => {
                            (24.0..94.0).contains(&px)
                                && [29., 48., 67.]
                                    .into_iter()
                                    .any(|row| (py - row - (px - 59.).powi(2) * 0.004).abs() < 3.8)
                        }
                        12 => {
                            line(20., 47., 59., 17.)
                                || line(59., 17., 98., 47.)
                                || line(31., 43., 31., 80.)
                                || line(31., 80., 87., 80.)
                                || line(87., 80., 87., 43.)
                                || line(50., 80., 50., 59.)
                                || line(50., 59., 68., 59.)
                                || line(68., 59., 68., 80.)
                        }
                        _ => false,
                    };
                    if filled {
                        mask.put_pixel(x, y, Luma([255]));
                    }
                }
            }
            symbols.insert(
                index,
                imageops::resize(&mask, 118, 99, imageops::FilterType::Lanczos3),
            );
        }
        symbols
    })
}

fn configured_icon(page: &PageConfig, index: usize) -> Result<GrayImage> {
    let asset = &page.keys[index].asset;
    if asset.is_empty() || index == NEXT_KEY {
        return icon_mask(&page.name, index);
    }
    let (family, source_index) = crate::applications::validate_asset(asset)?;
    let source_page = match family {
        "codex" => "Codex",
        "vscode" => "VS Code",
        "utility" => "Utility",
        "spotify" => "Spotify",
        "word" => "Word",
        "powerpoint" => "PowerPoint",
        "excel" => "Excel",
        _ => "Windows / media",
    };
    icon_mask(source_page, source_index)
}

// page_name: name of one of the three default pages.
// Returns the source atlas index and measured card geometry.
fn source_geometry(page_name: &str) -> (usize, [u32; 4], [u32; 7], u32, u32) {
    match page_name {
        "Codex" => (
            1,
            [125, 339, 609, 826],
            [117, 343, 568, 793, 1018, 1244, 1470],
            194,
            196,
        ),
        "VS Code" => (
            2,
            [135, 356, 595, 817],
            [129, 345, 561, 777, 993, 1209, 1425],
            190,
            184,
        ),
        _ => (
            0,
            [324, 484, 678, 840],
            [188, 343, 498, 653, 808, 963, 1119],
            146,
            138,
        ),
    }
}

// index: Utility key needing a symbol absent from the generated source sheets.
// Returns a smooth emoji or undo pictogram at the renderer's icon size.
fn utility_symbol(index: usize) -> GrayImage {
    const SCALE: u32 = 4;
    let mut large = GrayImage::new(118 * SCALE, 99 * SCALE);
    for y in 0..large.height() {
        for x in 0..large.width() {
            let px = x as f32 / SCALE as f32;
            let py = y as f32 / SCALE as f32;
            let filled = if index == 2 {
                let face_radius = ((px - 59.0).powi(2) + (py - 49.0).powi(2)).sqrt();
                let left_eye = ((px - 45.0).powi(2) + (py - 39.0).powi(2)).sqrt() < 3.5;
                let right_eye = ((px - 73.0).powi(2) + (py - 39.0).powi(2)).sqrt() < 3.5;
                let smile_y = 68.0 - (px - 59.0).powi(2) * 0.015;
                (face_radius > 36.0 && face_radius < 42.0)
                    || left_eye
                    || right_eye
                    || ((px - 59.0).abs() < 24.0 && (py - smile_y).abs() < 3.0)
            } else {
                let shaft = (38.0..85.0).contains(&px) && (42.0..51.0).contains(&py);
                let arrow = (18.0..48.0).contains(&px) && (py - 46.5).abs() < (48.0 - px) * 0.70;
                shaft || arrow
            };
            if filled {
                large.put_pixel(x, y, Luma([255]));
            }
        }
    }
    imageops::resize(&large, 118, 99, imageops::FilterType::Lanczos3)
}

// page_name: selects a generated source sheet; Utility borrows matching pictograms.
// index: physical key index from 0 through 12.
// Errors: a missing icon in the bundled source atlas.
fn icon_mask(page_name: &str, index: usize) -> Result<GrayImage> {
    if matches!(page_name, "Word" | "PowerPoint" | "Excel") {
        if index == NEXT_KEY {
            return icon_mask("Windows / media", NEXT_KEY);
        }
        let symbols = OFFICE_SYMBOLS.get_or_init(|| {
            let mut symbols = HashMap::new();
            for page in ["Word", "PowerPoint", "Excel"] {
                for key in (0..CLOCK_KEY).filter(|&key| key != NEXT_KEY) {
                    symbols.insert((page.to_owned(), key), office_symbol(page, key));
                }
            }
            symbols
        });
        return symbols
            .get(&(page_name.to_owned(), index))
            .cloned()
            .context("missing native Office icon");
    }
    if page_name == "Spotify" {
        if let Some(mask) = spotify_symbols().get(&index) {
            return Ok(mask.clone());
        }
        return match index {
            6 => icon_mask("VS Code", 3),
            _ => icon_mask("Windows / media", index),
        };
    }
    if page_name == "Utility" && matches!(index, 2 | 12) {
        return Ok(utility_symbol(index));
    }
    let (icon_page, icon_index) = if page_name == "Utility" {
        match index {
            0 => ("Codex", 11),
            1 => ("VS Code", 3),
            2 => ("Codex", 2),
            3 => ("Windows / media", 10),
            4 => ("Windows / media", 4),
            5 => ("Windows / media", 7),
            6 => ("Windows / media", 8),
            7 => ("Windows / media", 11),
            8 => ("Windows / media", 9),
            9 => ("Windows / media", 12),
            10 => ("Codex", 10),
            11 => ("Codex", 11),
            _ => ("Windows / media", 1),
        }
    } else if page_name == "Themes" {
        ("Windows / media", NEXT_KEY)
    } else {
        (page_name, index)
    };
    let (page_index, centers_x, centers_y, width, height) = source_geometry(icon_page);
    let cache = ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(mask) = cache
        .lock()
        .expect("icon cache lock")
        .get(&(page_index, icon_index))
    {
        return Ok(mask.clone());
    }
    let sources = SOURCES.get_or_init(|| {
        [
            image::load_from_memory(include_bytes!(
                "../assets/concepts/windows-light-abstract.png"
            ))
            .expect("bundled Windows icon atlas"),
            image::load_from_memory(include_bytes!(
                "../assets/concepts/codex-light-abstract.png"
            ))
            .expect("bundled Codex icon atlas"),
            image::load_from_memory(include_bytes!(
                "../assets/concepts/vscode-light-abstract.png"
            ))
            .expect("bundled VS Code icon atlas"),
        ]
    });
    let col = (icon_index % 2) * 2;
    let row = icon_index / 2;
    let x = centers_x[col] - width / 2;
    let y = centers_y[row] - height / 2;
    let tile = sources[page_index].crop_imm(x, y, width, height).to_rgb8();
    let mut mask = GrayImage::new(width, height);
    let mut bounds = (width, height, 0, 0);
    // Sample only the pictogram zone, keeping card edges and generated labels out.
    let left_limit = if page_index == 2 { 24 } else { 18 };
    for py in height * 12 / 100..height * 72 / 100 {
        for px in width * left_limit / 100..width * 82 / 100 {
            let pixel = tile.get_pixel(px, py);
            let brightness =
                (u16::from(pixel[0]) * 30 + u16::from(pixel[1]) * 59 + u16::from(pixel[2]) * 11)
                    / 100;
            let alpha = (190u16.saturating_sub(brightness) * 255 / 115).min(255) as u8;
            if alpha > 55 {
                mask.put_pixel(px, py, Luma([alpha]));
                bounds.0 = bounds.0.min(px);
                bounds.1 = bounds.1.min(py);
                bounds.2 = bounds.2.max(px);
                bounds.3 = bounds.3.max(py);
            }
        }
    }
    anyhow::ensure!(bounds.0 <= bounds.2, "missing icon for key {icon_index}");
    let icon = imageops::crop_imm(
        &mask,
        bounds.0,
        bounds.1,
        bounds.2 - bounds.0 + 1,
        bounds.3 - bounds.1 + 1,
    )
    .to_image();
    let scale = (118.0 / icon.width() as f32).min(99.0 / icon.height() as f32);
    let fitted = imageops::resize(
        &icon,
        (icon.width() as f32 * scale).round() as u32,
        (icon.height() as f32 * scale).round() as u32,
        imageops::FilterType::Lanczos3,
    );
    cache
        .lock()
        .expect("icon cache lock")
        .insert((page_index, icon_index), fitted.clone());
    Ok(fitted)
}

// a: color at the start of the gradient.
// b: color at the end of the gradient.
// t: interpolation factor from 0 to 1.
fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 * (1.0 - t) + b[0] as f32 * t) as u8,
        (a[1] as f32 * (1.0 - t) + b[1] as f32 * t) as u8,
        (a[2] as f32 * (1.0 - t) + b[2] as f32 * t) as u8,
    ]
}

// The approved ImageGen concept uses one three-color treatment per theme.
// Keep the source pictogram mask sharp while varying color inside its silhouette.
fn icon_gradient(theme: Theme, pressed: bool, base: [u8; 3], fx: f32, fy: f32) -> [u8; 3] {
    let (stops, t): ([[u8; 3]; 3], f32) = match theme {
        Theme::DarkClassic => (
            [[0, 187, 91], [57, 242, 151], [234, 255, 214]],
            0.76 * fx + 0.24 * (1.0 - fy),
        ),
        Theme::DarkAbstract => (
            [[103, 58, 255], [242, 65, 193], [255, 171, 98]],
            0.68 * fx + 0.32 * fy,
        ),
        Theme::LightClassic => (
            [[151, 103, 28], [201, 159, 68], [50, 105, 82]],
            0.58 * fx + 0.42 * fy,
        ),
        Theme::LightAbstract => (
            [[0, 117, 151], [113, 86, 190], [189, 78, 159]],
            0.88 * fx + 0.12 * fy,
        ),
        Theme::MangaInk => (
            [[20, 24, 32], [75, 74, 68], [203, 51, 43]],
            (0.78 * fx + 0.22 * fy).powf(1.3),
        ),
        Theme::SteampunkBrass => (
            [[255, 219, 107], [200, 130, 45], [137, 69, 32]],
            (0.65 * fx + 0.35 * fy).sqrt(),
        ),
        Theme::CyberpunkNeon => (
            [[7, 232, 248], [109, 67, 247], [255, 69, 190]],
            0.78 * fx + 0.22 * fy,
        ),
        Theme::Moire => (
            [[14, 103, 204], [8, 174, 171], [225, 249, 242]],
            0.74 * fx + 0.26 * (1.0 - fy),
        ),
        Theme::Cubism => (
            [[183, 68, 42], [235, 145, 96], [62, 100, 140]],
            0.65 * fx + 0.35 * fy,
        ),
        Theme::ArtDeco => (
            [[255, 221, 144], [185, 209, 125], [42, 206, 159]],
            0.69 * fx + 0.31 * fy,
        ),
        Theme::UkiyoE => (
            [[28, 58, 122], [235, 69, 57], [255, 240, 197]],
            0.62 * fx + 0.38 * fy,
        ),
        Theme::Solarpunk => (
            [[35, 129, 57], [139, 183, 42], [220, 166, 42]],
            0.80 * fx + 0.20 * (1.0 - fy),
        ),
        Theme::Memphis => (
            [[220, 71, 94], [145, 96, 199], [206, 163, 32]],
            0.67 * fx + 0.33 * fy,
        ),
    };
    let t = t.clamp(0.0, 1.0);
    let tint = if t < 0.5 {
        mix(stops[0], stops[1], t * 2.0)
    } else {
        mix(stops[1], stops[2], (t - 0.5) * 2.0)
    };
    mix(tint, base, if pressed { 0.18 } else { 0.04 })
}

// theme: visual family selected by the user.
// pressed: whether the physical key is down.
// Returns full-bleed gradient colors and icon/text colors, with no card border.
fn colors(theme: Theme, pressed: bool) -> ([u8; 3], [u8; 3], [u8; 3], [u8; 3]) {
    match (theme, pressed) {
        (Theme::DarkClassic, false) => ([3, 32, 19], [0, 9, 10], [122, 255, 160], [209, 255, 215]),
        (Theme::DarkClassic, true) => ([0, 92, 47], [1, 31, 32], [218, 255, 235], [236, 255, 240]),
        (Theme::DarkAbstract, false) => {
            ([42, 19, 61], [7, 17, 44], [255, 177, 116], [255, 226, 208])
        }
        (Theme::DarkAbstract, true) => (
            [110, 33, 95],
            [13, 59, 91],
            [255, 215, 157],
            [255, 239, 222],
        ),
        (Theme::LightClassic, false) => {
            ([252, 246, 229], [214, 229, 238], [22, 65, 98], [24, 62, 89])
        }
        (Theme::LightClassic, true) => {
            ([186, 222, 239], [120, 174, 205], [12, 48, 75], [15, 52, 78])
        }
        (Theme::LightAbstract, false) => (
            [248, 247, 236],
            [209, 237, 227],
            [16, 91, 105],
            [20, 76, 88],
        ),
        (Theme::LightAbstract, true) => {
            ([169, 232, 215], [112, 188, 198], [9, 68, 82], [10, 63, 78])
        }
        (Theme::MangaInk, false) => ([250, 248, 239], [217, 229, 229], [21, 27, 42], [25, 35, 49]),
        (Theme::MangaInk, true) => (
            [255, 226, 218],
            [211, 226, 228],
            [117, 28, 48],
            [52, 28, 45],
        ),
        (Theme::SteampunkBrass, false) => {
            ([75, 39, 29], [22, 31, 33], [240, 187, 101], [255, 225, 175])
        }
        (Theme::SteampunkBrass, true) => (
            [133, 72, 38],
            [49, 48, 37],
            [255, 219, 135],
            [255, 237, 196],
        ),
        (Theme::CyberpunkNeon, false) => {
            ([14, 17, 65], [34, 10, 65], [58, 250, 237], [245, 226, 255])
        }
        (Theme::CyberpunkNeon, true) => (
            [58, 23, 119],
            [14, 67, 105],
            [255, 79, 217],
            [255, 235, 252],
        ),
        (Theme::Moire, false) => ([234, 239, 237], [179, 207, 211], [23, 55, 75], [28, 54, 65]),
        (Theme::Moire, true) => ([157, 215, 219], [104, 161, 181], [8, 52, 77], [14, 46, 71]),
        (Theme::Cubism, false) => ([245, 204, 164], [208, 152, 137], [32, 59, 88], [33, 55, 75]),
        (Theme::Cubism, true) => ([245, 176, 117], [166, 144, 167], [29, 58, 91], [30, 51, 74]),
        (Theme::ArtDeco, false) => ([7, 67, 63], [8, 28, 48], [242, 203, 123], [251, 231, 186]),
        (Theme::ArtDeco, true) => (
            [16, 111, 89],
            [17, 51, 73],
            [255, 226, 158],
            [255, 240, 205],
        ),
        (Theme::UkiyoE, false) => (
            [29, 62, 105],
            [17, 37, 75],
            [244, 227, 190],
            [249, 235, 212],
        ),
        (Theme::UkiyoE, true) => (
            [52, 112, 141],
            [34, 75, 115],
            [255, 238, 206],
            [255, 245, 225],
        ),
        (Theme::Solarpunk, false) => ([249, 236, 170], [197, 231, 180], [28, 90, 64], [32, 77, 60]),
        (Theme::Solarpunk, true) => ([255, 219, 133], [145, 208, 151], [18, 76, 57], [28, 67, 52]),
        (Theme::Memphis, false) => ([255, 194, 163], [239, 176, 211], [27, 46, 89], [28, 43, 74]),
        (Theme::Memphis, true) => ([255, 174, 124], [178, 184, 243], [25, 40, 89], [25, 39, 74]),
    }
}

fn accent_color(theme: Theme) -> [u8; 3] {
    match theme {
        Theme::DarkClassic => [0, 113, 54],
        Theme::DarkAbstract => [236, 64, 130],
        Theme::LightClassic => [255, 236, 198],
        Theme::LightAbstract => [195, 245, 232],
        Theme::MangaInk => [255, 143, 144],
        Theme::SteampunkBrass => [176, 102, 49],
        Theme::CyberpunkNeon => [28, 154, 220],
        Theme::Moire => [207, 242, 236],
        Theme::Cubism => [255, 226, 161],
        Theme::ArtDeco => [11, 149, 124],
        Theme::UkiyoE => [210, 83, 67],
        Theme::Solarpunk => [255, 246, 183],
        Theme::Memphis => [255, 238, 153],
    }
}

// theme: visual family that owns the decorative motif.
// x: horizontal pixel coordinate.
// y: vertical pixel coordinate.
// Returns an accent tint and a deliberately subtle amount behind the pictogram.
fn motif(theme: Theme, x: u32, y: u32) -> Option<([u8; 3], f32)> {
    let xf = x as f32;
    let yf = y as f32;
    match theme {
        Theme::MangaInk => {
            let dx = (x % 18) as i32 - 9;
            let dy = (y % 18) as i32 - 9;
            (dx * dx + dy * dy < 9).then_some(([28, 43, 61], 0.13))
        }
        Theme::SteampunkBrass => {
            let radius = ((xf - 168.0).powi(2) + (yf - 25.0).powi(2)).sqrt();
            (radius % 27.0 < 2.4).then_some(([237, 183, 94], 0.16))
        }
        Theme::CyberpunkNeon => {
            if (x + 2 * y) % 53 < 2 {
                Some(([251, 73, 193], 0.20))
            } else if x % 43 == 0 {
                Some(([55, 241, 229], 0.11))
            } else {
                None
            }
        }
        Theme::Moire => {
            let interference = (xf * 0.125 + yf * 0.083).sin() * (xf * 0.123 - yf * 0.079).sin();
            Some(([39, 103, 124], (interference.abs() * 0.18).min(0.18)))
        }
        Theme::Cubism => {
            let cell = (x / 53 + y / 53) % 3;
            let diagonal = (x % 53) + (y % 53) > 53;
            if diagonal {
                Some((
                    if cell == 0 {
                        [47, 93, 123]
                    } else {
                        [242, 222, 181]
                    },
                    0.17,
                ))
            } else {
                None
            }
        }
        Theme::ArtDeco => {
            let angle = (xf - 98.0).atan2(212.0 - yf);
            let spoke = (angle * 13.0).sin().abs();
            (spoke > 0.966).then_some(([230, 187, 94], 0.24))
        }
        Theme::UkiyoE => {
            let wave = (yf * 0.16 + (xf * 0.055).sin() * 2.2).sin();
            (wave > 0.84).then_some(([233, 242, 230], 0.16))
        }
        Theme::Solarpunk => {
            let angle = (yf - 4.0).atan2(xf - 174.0);
            let ray = (angle * 13.0).sin().abs();
            (ray > 0.95).then_some(([255, 250, 193], 0.23))
        }
        Theme::Memphis => {
            let cell_x = x / 37;
            let cell_y = y / 37;
            let dx = x % 37;
            let dy = y % 37;
            let seed = (cell_x * 7 + cell_y * 11) % 3;
            if (dx as i32 - 18).pow(2) + (dy as i32 - 18).pow(2) < 36 {
                Some((
                    if seed == 0 {
                        [42, 111, 163]
                    } else {
                        [255, 239, 136]
                    },
                    0.24,
                ))
            } else if seed == 2 && dx + dy > 55 && dx + dy < 59 {
                Some(([43, 111, 163], 0.20))
            } else {
                None
            }
        }
        _ => None,
    }
}

// theme: selected full-bleed appearance.
// pressed: active state selects a stronger gradient.
// top: upper gradient color.
// bottom: lower gradient color.
fn background(theme: Theme, pressed: bool, top: [u8; 3], bottom: [u8; 3]) -> RgbaImage {
    let mut canvas = RgbaImage::new(SIDE, SIDE);
    for y in 0..SIDE {
        for x in 0..SIDE {
            let fy = y as f32 / (SIDE - 1) as f32;
            let fx = x as f32 / (SIDE - 1) as f32;
            let mut color = mix(top, bottom, fy * 0.75 + fx * 0.25);
            let glow = (1.0 - (((fx - 0.78).powi(2) + (fy - 0.15).powi(2)).sqrt() / 0.9)).max(0.0);
            let accent = accent_color(theme);
            color = mix(color, accent, glow * if pressed { 0.31 } else { 0.16 });
            if let Some((tint, amount)) = motif(theme, x, y) {
                color = mix(color, tint, amount);
            }
            canvas.put_pixel(x, y, Rgba([color[0], color[1], color[2], 255]));
        }
    }
    canvas
}

// canvas: target 196 x 196 image.
// x: horizontal pixel coordinate.
// y: vertical pixel coordinate.
// color: foreground RGB color.
// alpha: foreground coverage from 0 to 255.
fn blend(canvas: &mut RgbaImage, x: i32, y: i32, color: [u8; 3], alpha: u8) {
    if x < 0 || y < 0 || x >= canvas.width() as i32 || y >= canvas.height() as i32 || alpha == 0 {
        return;
    }
    let old = canvas.get_pixel(x as u32, y as u32).0;
    let t = alpha as f32 / 255.0;
    let rgb = mix([old[0], old[1], old[2]], color, t);
    canvas.put_pixel(x as u32, y as u32, Rgba([rgb[0], rgb[1], rgb[2], 255]));
}

// Tint the single bundled ribbon with the current theme while keeping its
// cream lettering and edge line legible over the grayscale key artwork.
fn overlay_unavailable(canvas: &mut RgbaImage, theme: Theme) {
    let overlay = UNAVAILABLE_OVERLAY.get_or_init(|| {
        let image = image::load_from_memory(include_bytes!("../assets/overlay/not-available.png"))
            .expect("bundled unavailable overlay");
        image
            .resize_exact(SIDE, SIDE, imageops::FilterType::Lanczos3)
            .to_rgba8()
    });
    let accent = accent_color(theme);
    for (x, y, pixel) in overlay.enumerate_pixels() {
        let [red, green, blue, alpha] = pixel.0;
        if alpha == 0 {
            continue;
        }
        let cream = red > 145
            && green > 140
            && blue > 125
            && red.abs_diff(green) < 70
            && green.abs_diff(blue) < 80;
        let color = if cream {
            [255, 246, 220]
        } else {
            let luminance =
                (77 * red as u32 + 150 * green as u32 + 29 * blue as u32) as f32 / (255.0 * 256.0);
            mix([8, 18, 24], accent, 0.32 + 0.40 * luminance)
        };
        blend(canvas, x as i32, y as i32, color, alpha);
    }
}

// canvas: gradient face being rendered.
// mask: cropped and resized source pictogram alpha.
// color: main pictogram color.
// theme: selects colored glow on dark themes.
// pressed: moves the icon slightly down while active.
fn draw_icon(
    canvas: &mut RgbaImage,
    mask: &GrayImage,
    color: [u8; 3],
    theme: Theme,
    pressed: bool,
) {
    let left = (SIDE - mask.width()) as i32 / 2;
    let top = 27 + (100 - mask.height()) as i32 / 2 + if pressed { 2 } else { 0 };
    if matches!(
        theme,
        Theme::DarkClassic
            | Theme::DarkAbstract
            | Theme::SteampunkBrass
            | Theme::CyberpunkNeon
            | Theme::ArtDeco
            | Theme::UkiyoE
    ) {
        let blurred = imageops::blur(mask, 3.8);
        for y in 0..blurred.height() {
            for x in 0..blurred.width() {
                let alpha = (blurred.get_pixel(x, y)[0] as f32 * 0.36) as u8;
                blend(canvas, left + x as i32, top + y as i32, color, alpha);
            }
        }
    }
    for y in 0..mask.height() {
        for x in 0..mask.width() {
            let alpha = mask.get_pixel(x, y)[0];
            blend(
                canvas,
                left + x as i32 + 3,
                top + y as i32 + 4,
                [0, 0, 0],
                alpha / 3,
            );
            let fx = x as f32 / mask.width().saturating_sub(1).max(1) as f32;
            let fy = y as f32 / mask.height().saturating_sub(1).max(1) as f32;
            let tint = icon_gradient(theme, pressed, color, fx, fy);
            blend(canvas, left + x as i32, top + y as i32, tint, alpha);
        }
    }
}

// canvas: destination 196 x 196 theme button.
// theme: appearance represented by the color sample.
// pressed: moves the sample slightly down with the physical key.
fn draw_swatch(canvas: &mut RgbaImage, theme: Theme, pressed: bool) {
    let (top, bottom, icon, label) = colors(theme, pressed);
    let shift: i32 = if pressed { 3 } else { 0 };
    let discs: [(i32, i32, i32, [u8; 3]); 3] = [
        (76, 78 + shift, 37, icon),
        (116, 72 + shift, 29, mix(top, label, 0.55)),
        (105, 101 + shift, 22, mix(bottom, icon, 0.46)),
    ];
    for (cx, cy, radius, tint) in discs {
        for y in (cy - radius - 2)..=(cy + radius + 2) {
            for x in (cx - radius - 2)..=(cx + radius + 2) {
                let distance = (((x - cx).pow(2) + (y - cy).pow(2)) as f32).sqrt();
                let coverage = (radius as f32 + 0.5 - distance).clamp(0.0, 1.0);
                blend(canvas, x + 3, y + 4, [0, 0, 0], (coverage * 58.0) as u8);
                let fx = (x - cx + radius) as f32 / (radius * 2) as f32;
                let fy = (y - cy + radius) as f32 / (radius * 2) as f32;
                let gradient = icon_gradient(theme, pressed, tint, fx, fy);
                blend(canvas, x, y, gradient, (coverage * 236.0) as u8);
            }
        }
    }
}

// label: configured English command title.
// Returns one or two short lines without cutting a word.
fn wrap(label: &str) -> Vec<String> {
    let mut lines = vec![String::new()];
    for word in label.to_uppercase().split_whitespace() {
        let last = lines.last_mut().expect("one line exists");
        if last.len() + word.len() + usize::from(!last.is_empty()) > 11 && !last.is_empty() {
            lines.push(word.to_string());
        } else {
            if !last.is_empty() {
                last.push(' ');
            }
            last.push_str(word);
        }
    }
    lines.truncate(2);
    lines
}

// canvas: destination image.
// label: key title from the page configuration.
// color: readable foreground RGB color.
fn draw_label(canvas: &mut RgbaImage, label: &str, color: [u8; 3]) {
    let font = LABEL_FONT.get_or_init(|| {
        Font::from_bytes(
            include_bytes!("../assets/fonts/VeraBd.ttf").as_slice(),
            FontSettings::default(),
        )
        .expect("bundled Bitstream Vera Bold font")
    });
    let lines = wrap(label);
    for (line_number, line) in lines.iter().enumerate() {
        let size = if line.len() > 11 {
            16.0
        } else if line.len() > 9 {
            18.0
        } else {
            20.0
        };
        let width: f32 = line
            .chars()
            .map(|character| font.metrics(character, size).advance_width)
            .sum();
        let mut pen = ((SIDE as f32 - width) / 2.0).round() as i32;
        let top = if lines.len() == 1 {
            155
        } else {
            144 + line_number as i32 * 23
        };
        for character in line.chars() {
            let (metrics, bitmap) = font.rasterize(character, size);
            for gy in 0..metrics.height {
                for gx in 0..metrics.width {
                    let alpha = bitmap[gy * metrics.width + gx];
                    blend(
                        canvas,
                        pen + gx as i32 + 1,
                        top + gy as i32 + 2,
                        [0, 0, 0],
                        alpha / 4,
                    );
                    blend(canvas, pen + gx as i32, top + gy as i32, color, alpha);
                }
            }
            pen += metrics.advance_width.round() as i32;
        }
    }
}

// theme: one of thirteen visual families.
// page: page title, labels, and actions.
// index: physical key index from 0 through 13.
// pressed: true uses the active state.
// Errors: invalid index, missing source icon, or failed PNG encoding.
// Disabled faces are derived from the rendered theme on demand, not stored assets.
// Navigation and the reserved clock position retain their normal appearance.
pub fn render_available(
    theme: Theme,
    page: &PageConfig,
    index: usize,
    pressed: bool,
    enabled: bool,
) -> Result<Vec<u8>> {
    anyhow::ensure!(index < page.keys.len(), "key index out of range");
    let mut output = Cursor::new(Vec::new());
    if index == CLOCK_KEY {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(SIDE, SIDE, Rgba([0, 0, 0, 255])))
            .write_to(&mut output, ImageFormat::Png)?;
        return Ok(output.into_inner());
    }
    let disabled = !enabled && index != NEXT_KEY;
    let pressed = pressed && !disabled;
    let selected = page.keys[index]
        .action
        .strip_prefix("theme:")
        .map(|slug| Theme::from_slug(slug).with_context(|| format!("unknown theme '{slug}'")))
        .transpose()?;
    let display_theme = selected.unwrap_or(theme);
    let (top, bottom, icon, label) = colors(display_theme, pressed);
    let mut canvas = background(display_theme, pressed, top, bottom);
    if selected.is_some() {
        draw_swatch(&mut canvas, display_theme, pressed);
    } else {
        draw_icon(
            &mut canvas,
            &configured_icon(page, index)?,
            icon,
            display_theme,
            pressed,
        );
    }
    draw_label(&mut canvas, &page.keys[index].label, label);
    if disabled {
        for pixel in canvas.pixels_mut() {
            // Integer luminance preserves layout, contrast, and alpha.
            let gray = ((77 * pixel[0] as u32 + 150 * pixel[1] as u32 + 29 * pixel[2] as u32 + 128)
                >> 8) as u8;
            pixel[0] = gray;
            pixel[1] = gray;
            pixel[2] = gray;
        }
        overlay_unavailable(&mut canvas, theme);
    }
    DynamicImage::ImageRgba8(canvas).write_to(&mut output, ImageFormat::Png)?;
    Ok(output.into_inner())
}

// Render the optional wide information panel from the same theme palette.
// Its 400x200 geometry follows the observed vendor double-key artwork format.
pub fn render_status(theme: Theme, page: &PageConfig, enabled: bool) -> Result<Vec<u8>> {
    let (top, bottom, _, foreground) = colors(theme, false);
    let mut canvas = imageops::resize(
        &background(theme, false, top, bottom),
        400,
        200,
        imageops::FilterType::CatmullRom,
    );
    draw_panel_text(&mut canvas, "VIBEWARE", 24, 30, 15.0, foreground);
    draw_panel_text(&mut canvas, &page.name, 24, 79, 32.0, foreground);
    draw_panel_text(
        &mut canvas,
        &format!("THEME: {}", theme.label()),
        24,
        119,
        20.0,
        foreground,
    );
    let marker = if enabled {
        [36, 196, 111]
    } else {
        [148, 148, 148]
    };
    for y in 149..166 {
        for x in 25..42 {
            if (x as i32 - 33).pow(2) + (y as i32 - 157).pow(2) <= 64 {
                canvas.put_pixel(x, y, Rgba([marker[0], marker[1], marker[2], 255]));
            }
        }
    }
    draw_panel_text(
        &mut canvas,
        if enabled { "ACTIVE" } else { "INACTIVE" },
        54,
        166,
        25.0,
        foreground,
    );
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(canvas).write_to(&mut output, ImageFormat::Png)?;
    Ok(output.into_inner())
}

fn draw_panel_text(
    canvas: &mut RgbaImage,
    text: &str,
    left: i32,
    baseline: i32,
    mut size: f32,
    color: [u8; 3],
) {
    let font = LABEL_FONT.get_or_init(|| {
        Font::from_bytes(
            include_bytes!("../assets/fonts/VeraBd.ttf").as_slice(),
            FontSettings::default(),
        )
        .expect("bundled Bitstream Vera Bold font")
    });
    let text = if text.chars().count() > 48 {
        format!("{}...", text.chars().take(45).collect::<String>())
    } else {
        text.to_string()
    };
    while text
        .chars()
        .map(|c| font.metrics(c, size).advance_width)
        .sum::<f32>()
        > (canvas.width() as i32 - left - 24) as f32
        && size > 8.0
    {
        size -= 1.0;
    }
    let mut pen = left as f32;
    for character in text.chars() {
        let (metrics, bitmap) = font.rasterize(character, size);
        for gy in 0..metrics.height {
            for gx in 0..metrics.width {
                blend(
                    canvas,
                    pen.round() as i32 + metrics.xmin + gx as i32,
                    baseline - metrics.height as i32 - metrics.ymin + gy as i32,
                    color,
                    bitmap[gy * metrics.width + gx],
                );
            }
        }
        pen += metrics.advance_width;
    }
}
