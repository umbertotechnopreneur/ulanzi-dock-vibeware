/* VBWR B
 *
 * Project: UlanziDeck
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Created with AI: OpenAI Codex; extraction of existing palette drawing code; 2026-10-27
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

//! Define the colors, gradients and background motifs for the thirteen themes.
//! Coordinates passed to gradients are normalized; motif coordinates are native key pixels.
//! These pure drawing rules do not load assets, decide licensing or communicate with a dock.

use crate::model::Theme;

/// Linearly mix RGB colors `a` and `b`; `t` is the contribution of `b` in 0..=1.
pub(super) fn mix(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        (a[0] as f32 * (1.0 - t) + b[0] as f32 * t) as u8,
        (a[1] as f32 * (1.0 - t) + b[1] as f32 * t) as u8,
        (a[2] as f32 * (1.0 - t) + b[2] as f32 * t) as u8,
    ]
}

// The approved ImageGen concept uses one three-color treatment per theme.
// Keep the source pictogram mask sharp while varying color inside its silhouette.
/// Shade `base` for `theme` and `pressed` at normalized icon coordinates `fx`, `fy`.
pub(super) fn icon_gradient(
    theme: Theme,
    pressed: bool,
    base: [u8; 3],
    fx: f32,
    fy: f32,
) -> [u8; 3] {
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

/// Colors for a button face before the inactive overlay is applied.
/// Named colors for a key face, avoiding positional color tuples at call sites.
pub(super) struct KeyPalette {
    pub(super) background_top: [u8; 3],
    pub(super) background_bottom: [u8; 3],
    pub(super) icon: [u8; 3],
    pub(super) label: [u8; 3],
}

/// Select background endpoints, pictogram and label colors for one key state.
pub(super) fn colors(theme: Theme, pressed: bool) -> KeyPalette {
    let (background_top, background_bottom, icon, label) = match (theme, pressed) {
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
    };

    KeyPalette {
        background_top,
        background_bottom,
        icon,
        label,
    }
}

/// Return the theme accent used consistently by glows, swatches and overlays.
pub(super) fn accent_color(theme: Theme) -> [u8; 3] {
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
/// Sample the theme decoration at native pixel coordinates `x`, `y`.
/// Return an RGB tint and opacity, or `None` where the base gradient stays visible.
pub(super) fn motif(theme: Theme, x: u32, y: u32) -> Option<([u8; 3], f32)> {
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
