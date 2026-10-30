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

//! Compose key faces and the wide status panel into PNG bytes.
//! Icon geometry lives in `icons`; theme colors and motifs live in `palette`. Rendering
//! is in memory and does not choose pages, execute actions, save files or access HID.

use crate::model::{PageConfig, Theme, CLOCK_KEY, NEXT_KEY, PAGE_INDEX_NAME};
use anyhow::{Context, Result};
use fontdue::{Font, FontSettings};
use image::{imageops, DynamicImage, GrayImage, ImageFormat, Rgba, RgbaImage};
use std::{io::Cursor, sync::OnceLock};

mod icons;
mod palette;
use icons::configured_icon;
use palette::{accent_color, colors, icon_gradient, mix, motif};

const SIDE: u32 = 196;
static UNAVAILABLE_OVERLAY: OnceLock<RgbaImage> = OnceLock::new();
static LABEL_FONT: OnceLock<Font> = OnceLock::new();

/// Fill a native key face using `theme`, press-dependent glow and two RGB endpoints.
/// This is the shared base for normal keys and generated unavailable artwork.
fn background(theme: Theme, pressed: bool, top: [u8; 3], bottom: [u8; 3]) -> RgbaImage {
    let mut canvas = RgbaImage::new(SIDE, SIDE);
    for y in 0..SIDE {
        for x in 0..SIDE {
            let fy = y as f32 / (SIDE - 1) as f32;
            let fx = x as f32 / (SIDE - 1) as f32;
            let diagonal_position = fy * 0.75 + fx * 0.25;
            let mut color = mix(top, bottom, diagonal_position);

            // Keep the glow near the upper-right corner, independent of key text.
            let distance_from_glow = ((fx - 0.78).powi(2) + (fy - 0.15).powi(2)).sqrt();
            let glow = (1.0 - distance_from_glow / 0.9).max(0.0);
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
    let palette = colors(theme, pressed);
    let shift: i32 = if pressed { 3 } else { 0 };
    let discs: [(i32, i32, i32, [u8; 3]); 3] = [
        (76, 78 + shift, 37, palette.icon),
        (
            116,
            72 + shift,
            29,
            mix(palette.background_top, palette.label, 0.55),
        ),
        (
            105,
            101 + shift,
            22,
            mix(palette.background_bottom, palette.icon, 0.46),
        ),
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
/// Split `label` into short display lines; return owned strings for font layout.
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

/// Decode the bundled font once and share it across all key renders.
fn label_font() -> &'static Font {
    LABEL_FONT.get_or_init(|| {
        Font::from_bytes(
            include_bytes!("../assets/fonts/VeraBd.ttf").as_slice(),
            FontSettings::default(),
        )
        .expect("bundled Bitstream Vera Bold font")
    })
}

// canvas: destination image.
// label: key title from the page configuration.
// color: readable foreground RGB color.
fn draw_label(canvas: &mut RgbaImage, label: &str, color: [u8; 3]) {
    let font = label_font();
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
    let disabled = (!enabled || !page.key_available(index)) && index != NEXT_KEY;
    let pressed = pressed && !disabled;
    let selected = page.keys[index]
        .action
        .strip_prefix("theme:")
        .map(|slug| Theme::from_slug(slug).with_context(|| format!("unknown theme '{slug}'")))
        .transpose()?;
    let display_theme = selected.unwrap_or(theme);
    let palette = colors(display_theme, pressed);
    let mut canvas = background(
        display_theme,
        pressed,
        palette.background_top,
        palette.background_bottom,
    );
    if page.name == PAGE_INDEX_NAME && page.keys[index].action == "none" {
        DynamicImage::ImageRgba8(canvas).write_to(&mut output, ImageFormat::Png)?;
        return Ok(output.into_inner());
    }
    if selected.is_some() {
        draw_swatch(&mut canvas, display_theme, pressed);
    } else {
        draw_icon(
            &mut canvas,
            &configured_icon(page, index)?,
            palette.icon,
            display_theme,
            pressed,
        );
    }
    draw_label(&mut canvas, &page.keys[index].label, palette.label);
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
    let palette = colors(theme, false);
    let foreground = palette.label;
    let mut canvas = imageops::resize(
        &background(
            theme,
            false,
            palette.background_top,
            palette.background_bottom,
        ),
        400,
        200,
        imageops::FilterType::CatmullRom,
    );
    draw_panel_text(&mut canvas, "VIBEWARE", 24, 30, 15.0, foreground);
    draw_panel_text(&mut canvas, "PAGES", 249, 30, 15.0, foreground);
    for row in 0..3 {
        for column in 0..3 {
            for y in 14 + row * 14..24 + row * 14 {
                for x in 334 + column * 14..344 + column * 14 {
                    canvas.put_pixel(
                        x,
                        y,
                        Rgba([foreground[0], foreground[1], foreground[2], 255]),
                    );
                }
            }
        }
    }
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

/// Draw `text` into `canvas`, reducing `size` to fit the remaining panel width.
/// `left` and `baseline` are pixel coordinates; `color` is foreground RGB.
/// Long text is truncated before rasterization; this helper does not encode the PNG.
fn draw_panel_text(
    canvas: &mut RgbaImage,
    text: &str,
    left: i32,
    baseline: i32,
    mut size: f32,
    color: [u8; 3],
) {
    let font = label_font();
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
