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
static LABEL_FONT: OnceLock<Font> = OnceLock::new();

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
            let accent = match theme {
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
            };
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
    if x < 0 || y < 0 || x >= SIDE as i32 || y >= SIDE as i32 || alpha == 0 {
        return;
    }
    let old = canvas.get_pixel(x as u32, y as u32).0;
    let t = alpha as f32 / 255.0;
    let rgb = mix([old[0], old[1], old[2]], color, t);
    canvas.put_pixel(x as u32, y as u32, Rgba([rgb[0], rgb[1], rgb[2], 255]));
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
            let tint = mix(
                color,
                [255, 255, 255],
                y as f32 / mask.height() as f32 * 0.12,
            );
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
                blend(canvas, x, y, tint, (coverage * 236.0) as u8);
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
pub fn render(theme: Theme, page: &PageConfig, index: usize, pressed: bool) -> Result<Vec<u8>> {
    anyhow::ensure!(index < page.keys.len(), "key index out of range");
    let mut output = Cursor::new(Vec::new());
    if index == CLOCK_KEY {
        DynamicImage::ImageRgba8(RgbaImage::from_pixel(SIDE, SIDE, Rgba([0, 0, 0, 255])))
            .write_to(&mut output, ImageFormat::Png)?;
        return Ok(output.into_inner());
    }
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
            &icon_mask(&page.name, index)?,
            icon,
            display_theme,
            pressed,
        );
    }
    draw_label(&mut canvas, &page.keys[index].label, label);
    DynamicImage::ImageRgba8(canvas).write_to(&mut output, ImageFormat::Png)?;
    Ok(output.into_inner())
}
