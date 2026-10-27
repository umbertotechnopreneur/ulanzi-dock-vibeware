/* VBWR B
 *
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Created with AI: OpenAI Codex; extraction of existing icon drawing code; 2026-10-27
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

//! Build monochrome pictogram masks from bundled artwork and small drawing primitives.
//! Caches avoid repeatedly decoding embedded assets. Masks describe shape only; the parent
//! renderer supplies theme colors, text, pressed states and unavailable overlays.

use super::draw_panel_text;
use crate::model::{PageConfig, CLOCK_KEY, NEXT_KEY};
use anyhow::{Context, Result};
use image::{imageops, DynamicImage, GrayImage, Luma, Rgba, RgbaImage};
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};
static SOURCES: OnceLock<[DynamicImage; 3]> = OnceLock::new();
static ICON_CACHE: OnceLock<Mutex<HashMap<(usize, usize), GrayImage>>> = OnceLock::new();
static SPOTIFY_SYMBOLS: OnceLock<HashMap<usize, GrayImage>> = OnceLock::new();
static OFFICE_SYMBOLS: OnceLock<HashMap<(String, usize), GrayImage>> = OnceLock::new();
static OBS_SYMBOLS: OnceLock<HashMap<usize, GrayImage>> = OnceLock::new();

// Original OBS Creator controls. The shared renderer supplies every theme and state;
// these neutral masks do not claim to reflect OBS recording or audio status.
/// Lazily draw OBS-specific masks, keyed by physical index; no app state is queried.
fn obs_symbols() -> &'static HashMap<usize, GrayImage> {
    OBS_SYMBOLS.get_or_init(|| {
        let mut symbols = HashMap::new();
        const SCALE: u32 = 4;
        for index in (0..CLOCK_KEY).filter(|&key| key != NEXT_KEY) {
            let mut mask = GrayImage::new(118 * SCALE, 99 * SCALE);
            for y in 0..mask.height() {
                for x in 0..mask.width() {
                    let px = x as f32 / SCALE as f32;
                    let py = y as f32 / SCALE as f32;
                    let rect = |left: f32, top: f32, right: f32, bottom: f32| {
                        px >= left && px <= right && py >= top && py <= bottom
                    };
                    let line = |ax: f32, ay: f32, bx: f32, by: f32, width: f32| {
                        let dx = bx - ax;
                        let dy = by - ay;
                        let t =
                            (((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy)).clamp(0., 1.);
                        (px - ax - t * dx).powi(2) + (py - ay - t * dy).powi(2)
                            <= (width / 2.).powi(2)
                    };
                    let ring = |cx: f32, cy: f32, radius: f32, width: f32| {
                        (((px - cx).powi(2) + (py - cy).powi(2)).sqrt() - radius).abs()
                            <= width / 2.
                    };
                    let monitor = (rect(13., 17., 105., 72.) && !rect(19., 23., 99., 66.))
                        || line(59., 72., 59., 86., 6.)
                        || line(40., 87., 78., 87., 6.);
                    let filled = match index {
                        0 => (px - 59.).powi(2) + (py - 49.).powi(2) <= 30_f32.powi(2),
                        1 => rect(31., 21., 87., 77.),
                        2 => rect(30., 19., 48., 79.) || rect(70., 19., 88., 79.),
                        3 => px >= 37. && px <= 90. && (py - 49.).abs() <= (90. - px) * 0.65,
                        5 => {
                            (rect(17., 26., 76., 75.) && !rect(23., 32., 70., 69.))
                                || (px >= 76.
                                    && px <= 103.
                                    && (py - 50.5).abs() <= (px - 60.) * 0.56)
                        }
                        6 => monitor,
                        7 => {
                            let inset = rect(67., 48., 108., 82.);
                            (monitor && !rect(62., 43., 113., 87.))
                                || (inset && !rect(72., 53., 103., 77.))
                                || (px - 87.5).powi(2) + (py - 60.).powi(2) <= 4.5_f32.powi(2)
                                || (py >= 68. && py <= 75. && (px - 87.5).abs() <= 10.)
                        }
                        8 => {
                            (rect(28., 36., 79., 74.) && !rect(34., 36., 73., 68.))
                                || (ring(82., 52., 14., 6.) && px >= 79.)
                                || line(23., 82., 86., 82., 6.)
                                || line(42., 24., 46., 13., 5.)
                                || line(60., 24., 64., 13., 5.)
                        }
                        9 | 10 => {
                            let capsule = rect(46., 27., 72., 50.)
                                || (px - 59.).powi(2) + (py - 27.).powi(2) <= 13_f32.powi(2)
                                || (px - 59.).powi(2) + (py - 50.).powi(2) <= 13_f32.powi(2);
                            let mic = capsule
                                || (ring(59., 49., 25., 6.) && py >= 49.)
                                || line(34., 39., 34., 49., 6.)
                                || line(84., 39., 84., 49., 6.)
                                || line(59., 74., 59., 85., 6.)
                                || line(45., 86., 73., 86., 6.);
                            if index == 10 {
                                (mic && !line(23., 15., 95., 87., 14.))
                                    || line(23., 15., 95., 87., 6.)
                            } else {
                                mic
                            }
                        }
                        11 | 12 => {
                            let speaker = rect(20., 38., 38., 62.)
                                || (px >= 38. && px <= 62. && (py - 50.).abs() <= px - 24.)
                                || (px >= 75.
                                    && (ring(57., 50., 25., 6.) || ring(57., 50., 39., 6.)));
                            if index == 12 {
                                (speaker && !line(23., 15., 97., 87., 14.))
                                    || line(23., 15., 97., 87., 6.)
                            } else {
                                speaker
                            }
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

// page_name: Word, PowerPoint, or Excel; used for application-specific command symbols.
// index: physical command key, excluding navigation and the clock.
// Returns original native artwork, with no Microsoft logo or third-party raster dependency.
/// Draw the Office pictogram for `page_name` and physical `index` in a 118 x 99 mask.
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
/// Lazily build Spotify masks shared by all theme and press-state renders.
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

/// Resolve `page.keys[index].asset`, falling back to that page's built-in geometry.
/// `index` must address a configured key. Errors report unsupported asset references.
pub(super) fn configured_icon(page: &PageConfig, index: usize) -> Result<GrayImage> {
    let asset = &page.keys[index].asset;
    if asset.is_empty() || index == NEXT_KEY {
        return icon_mask(&page.name, index);
    }
    let (family, source_index) = crate::applications::validate_asset(asset)?;
    if family == "index" {
        static SYMBOLS: OnceLock<Vec<GrayImage>> = OnceLock::new();
        return SYMBOLS
            .get_or_init(|| (0..10).map(page_index_symbol).collect())
            .get(source_index)
            .cloned()
            .context("missing native page index icon");
    }
    let source_page = match family {
        "codex" => "Codex",
        "vscode" => "VS Code",
        "utility" => "Utility",
        "spotify" => "Spotify",
        "word" => "Word",
        "powerpoint" => "PowerPoint",
        "excel" => "Excel",
        "obs" => "OBS Creator",
        _ => "Windows / media",
    };
    icon_mask(source_page, source_index)
}

// page_name: name of one of the three default pages.
// Returns the source atlas index and measured card geometry.
/// Describe the source sheet for `page_name`: sheet index, grid positions and crop size.
/// These coordinates refer to embedded concept sheets, not physical display pixels.
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

// Original category symbols for the generated page index, drawn in the same palette.
/// Draw one of the ten supported index pictograms selected by `kind`.
fn page_index_symbol(kind: usize) -> GrayImage {
    const SCALE: u32 = 4;
    let mut mask = GrayImage::new(118 * SCALE, 99 * SCALE);
    for y in 0..mask.height() {
        for x in 0..mask.width() {
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
            let radius = ((px - 59.).powi(2) + (py - 49.).powi(2)).sqrt();
            let filled = match kind {
                0 => {
                    (rect(16., 14., 102., 72.) && !rect(22., 20., 96., 66.))
                        || line(59., 72., 59., 86.)
                        || line(38., 86., 80., 86.)
                }
                1 => {
                    (radius > 21. && radius < 34.)
                        || ((px - 59.).abs() < 7. && radius > 27. && radius < 43.)
                        || ((py - 49.).abs() < 7. && radius > 27. && radius < 43.)
                        || (((px - 59.).abs() - (py - 49.).abs()).abs() < 8.
                            && radius > 27.
                            && radius < 43.)
                }
                2 => {
                    (rect(18., 15., 100., 68.) && !rect(24., 21., 94., 62.))
                        || line(35., 68., 35., 85.)
                        || line(35., 85., 56., 68.)
                        || line(35., 35., 83., 35.)
                        || line(35., 49., 68., 49.)
                }
                3 => {
                    line(40., 20., 17., 49.)
                        || line(17., 49., 40., 78.)
                        || line(78., 20., 101., 49.)
                        || line(101., 49., 78., 78.)
                        || line(66., 16., 51., 83.)
                }
                4 => {
                    line(48., 20., 48., 74.)
                        || line(87., 15., 87., 66.)
                        || line(48., 20., 87., 15.)
                        || ((px - 37.).powi(2) / 14.0_f32.powi(2)
                            + (py - 76.).powi(2) / 10.0_f32.powi(2)
                            <= 1.)
                        || ((px - 76.).powi(2) / 14.0_f32.powi(2)
                            + (py - 68.).powi(2) / 10.0_f32.powi(2)
                            <= 1.)
                }
                5..=7 => rect(24., 10., 94., 88.) && !rect(30., 16., 88., 82.),
                8 => {
                    (radius > 32. && radius < 39.)
                        || [(43., 28.), (70., 27.), (83., 49.), (69., 71.)].iter().any(
                            |&(cx, cy)| (px - cx).powi(2) + (py - cy).powi(2) <= 7.0_f32.powi(2),
                        )
                }
                _ => (0..3).any(|row| {
                    (0..3).any(|column| {
                        let left = 24. + column as f32 * 25.;
                        let top = 15. + row as f32 * 25.;
                        rect(left, top, left + 17., top + 17.)
                    })
                }),
            };
            if filled {
                mask.put_pixel(x, y, Luma([255]));
            }
        }
    }
    if matches!(kind, 5..=7) {
        let mut canvas = DynamicImage::ImageLuma8(mask).to_rgba8();
        draw_panel_text(
            &mut canvas,
            ["W", "P", "X"][kind - 5],
            39 * SCALE as i32,
            67 * SCALE as i32,
            36. * SCALE as f32,
            [255, 255, 255],
        );
        mask = DynamicImage::ImageRgba8(canvas).to_luma8();
    }
    imageops::resize(&mask, 118, 99, imageops::FilterType::Lanczos3)
}

// index: Utility key needing a symbol absent from the generated source sheets.
// Returns a native sun with plus/minus, or emoji pictogram, at the renderer's icon size.
/// Draw a utility-page mask for physical `index`; navigation keeps its shared symbol.
fn utility_symbol(index: usize) -> GrayImage {
    const SCALE: u32 = 4;
    let mut large = GrayImage::new(118 * SCALE, 99 * SCALE);
    for y in 0..large.height() {
        for x in 0..large.width() {
            let px = x as f32 / SCALE as f32;
            let py = y as f32 / SCALE as f32;
            let filled = if index <= 1 {
                let distance = ((px - 43.0).powi(2) + (py - 49.0).powi(2)).sqrt();
                let ring = (14.0..20.0).contains(&distance);
                let rays = (0..8).any(|ray| {
                    let angle = ray as f32 * std::f32::consts::FRAC_PI_4;
                    let (dy, dx) = angle.sin_cos();
                    let along = (px - 43.0) * dx + (py - 49.0) * dy;
                    let across = (py - 49.0) * dx - (px - 43.0) * dy;
                    (26.0..36.0).contains(&along) && across.abs() < 3.0
                });
                let minus = (80.0..108.0).contains(&px) && (py - 49.0).abs() < 3.0;
                let plus = index == 0 && (35.0..63.0).contains(&py) && (px - 94.0).abs() < 3.0;
                ring || rays || minus || plus
            } else {
                let face_radius = ((px - 59.0).powi(2) + (py - 49.0).powi(2)).sqrt();
                let left_eye = ((px - 45.0).powi(2) + (py - 39.0).powi(2)).sqrt() < 3.5;
                let right_eye = ((px - 73.0).powi(2) + (py - 39.0).powi(2)).sqrt() < 3.5;
                let smile_y = 68.0 - (px - 59.0).powi(2) * 0.015;
                (face_radius > 36.0 && face_radius < 42.0)
                    || left_eye
                    || right_eye
                    || ((px - 59.0).abs() < 24.0 && (py - smile_y).abs() < 3.0)
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
/// Crop and normalize the requested page/key pictogram, reusing cached masks.
/// `page_name` selects the source family; `index` is its physical key position.
/// Embedded decoding and unsupported-source errors propagate to the renderer.
fn icon_mask(page_name: &str, index: usize) -> Result<GrayImage> {
    if page_name == "OBS Creator" {
        if index == NEXT_KEY {
            return icon_mask("Windows / media", NEXT_KEY);
        }
        return obs_symbols()
            .get(&index)
            .cloned()
            .context("missing native OBS Creator icon");
    }
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
    if page_name == "Utility" && index <= 2 {
        static SYMBOLS: OnceLock<[GrayImage; 3]> = OnceLock::new();
        return Ok(SYMBOLS
            .get_or_init(|| [utility_symbol(0), utility_symbol(1), utility_symbol(2)])[index]
            .clone());
    }
    let (icon_page, icon_index) = if page_name == "Utility" {
        match index {
            3 => ("Windows / media", 10),
            4 => ("Windows / media", 4),
            5 => ("Windows / media", 7),
            6 => ("Windows / media", 8),
            7 => ("Windows / media", 11),
            8 => ("Windows / media", 9),
            9 => ("Codex", 11),
            10 => ("Codex", 10),
            11 => ("Codex", 11),
            12 => ("VS Code", 3),
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
                "../../assets/concepts/windows-light-abstract.png"
            ))
            .expect("bundled Windows icon atlas"),
            image::load_from_memory(include_bytes!(
                "../../assets/concepts/codex-light-abstract.png"
            ))
            .expect("bundled Codex icon atlas"),
            image::load_from_memory(include_bytes!(
                "../../assets/concepts/vscode-light-abstract.png"
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
