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

use crate::model::{PageConfig, Theme, CLOCK_KEY};
use anyhow::Result;
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

// page_name: selects a generated source sheet; custom pages borrow Windows icons.
// index: physical key index from 0 through 12.
// Errors: a missing icon in the bundled source atlas.
fn icon_mask(page_name: &str, index: usize) -> Result<GrayImage> {
    let (page_index, centers_x, centers_y, width, height) = source_geometry(page_name);
    let cache = ICON_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(mask) = cache.lock().expect("icon cache lock").get(&(page_index, index)) {
        return Ok(mask.clone());
    }
    let sources = SOURCES.get_or_init(|| {
        [
            image::load_from_memory(include_bytes!("../assets/concepts/windows-light-abstract.png"))
                .expect("bundled Windows icon atlas"),
            image::load_from_memory(include_bytes!("../assets/concepts/codex-light-abstract.png"))
                .expect("bundled Codex icon atlas"),
            image::load_from_memory(include_bytes!("../assets/concepts/vscode-light-abstract.png"))
                .expect("bundled VS Code icon atlas"),
        ]
    });
    let col = (index % 2) * 2;
    let row = index / 2;
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
    anyhow::ensure!(bounds.0 <= bounds.2, "missing icon for key {index}");
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
        .insert((page_index, index), fitted.clone());
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
        (Theme::DarkAbstract, false) => ([42, 19, 61], [7, 17, 44], [255, 177, 116], [255, 226, 208]),
        (Theme::DarkAbstract, true) => ([110, 33, 95], [13, 59, 91], [255, 215, 157], [255, 239, 222]),
        (Theme::LightClassic, false) => ([252, 246, 229], [214, 229, 238], [22, 65, 98], [24, 62, 89]),
        (Theme::LightClassic, true) => ([186, 222, 239], [120, 174, 205], [12, 48, 75], [15, 52, 78]),
        (Theme::LightAbstract, false) => ([248, 247, 236], [209, 237, 227], [16, 91, 105], [20, 76, 88]),
        (Theme::LightAbstract, true) => ([169, 232, 215], [112, 188, 198], [9, 68, 82], [10, 63, 78]),
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
            let glow = (1.0 - (((fx - 0.78).powi(2) + (fy - 0.15).powi(2)).sqrt() / 0.9))
                .max(0.0);
            let accent = match theme {
                Theme::DarkClassic => [0, 113, 54],
                Theme::DarkAbstract => [236, 64, 130],
                Theme::LightClassic => [255, 236, 198],
                Theme::LightAbstract => [195, 245, 232],
            };
            color = mix(color, accent, glow * if pressed { 0.31 } else { 0.16 });
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
fn draw_icon(canvas: &mut RgbaImage, mask: &GrayImage, color: [u8; 3], theme: Theme, pressed: bool) {
    let left = (SIDE - mask.width()) as i32 / 2;
    let top = 27 + (100 - mask.height()) as i32 / 2 + if pressed { 2 } else { 0 };
    if matches!(theme, Theme::DarkClassic | Theme::DarkAbstract) {
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
            blend(canvas, left + x as i32 + 3, top + y as i32 + 4, [0, 0, 0], alpha / 3);
            let tint = mix(color, [255, 255, 255], y as f32 / mask.height() as f32 * 0.12);
            blend(canvas, left + x as i32, top + y as i32, tint, alpha);
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
        let size = if line.len() > 11 { 16.0 } else if line.len() > 9 { 18.0 } else { 20.0 };
        let width: f32 = line
            .chars()
            .map(|character| font.metrics(character, size).advance_width)
            .sum();
        let mut pen = ((SIDE as f32 - width) / 2.0).round() as i32;
        let top = if lines.len() == 1 { 155 } else { 144 + line_number as i32 * 23 };
        for character in line.chars() {
            let (metrics, bitmap) = font.rasterize(character, size);
            for gy in 0..metrics.height {
                for gx in 0..metrics.width {
                    let alpha = bitmap[gy * metrics.width + gx];
                    blend(canvas, pen + gx as i32 + 1, top + gy as i32 + 2, [0, 0, 0], alpha / 4);
                    blend(canvas, pen + gx as i32, top + gy as i32, color, alpha);
                }
            }
            pen += metrics.advance_width.round() as i32;
        }
    }
}

// theme: one of four visual families.
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
    let (top, bottom, icon, label) = colors(theme, pressed);
    let mut canvas = background(theme, pressed, top, bottom);
    draw_icon(&mut canvas, &icon_mask(&page.name, index)?, icon, theme, pressed);
    draw_label(&mut canvas, &page.keys[index].label, label);
    DynamicImage::ImageRgba8(canvas).write_to(&mut output, ImageFormat::Png)?;
    Ok(output.into_inner())
}
