/* VBWR B
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 * VibeWare is Human intent. AI implementation. Accountable human review.
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * VBWR E */

// A small pixel-art derivative of the approved floppy, never a replacement master.
// The same 32-pixel drawing supplies the EXE, tray, and dialog icons.
pub fn floppy() -> image::RgbaImage {
    let mut image = image::RgbaImage::new(32, 32);
    let ink = [35, 42, 47, 255];
    let blue = [58, 106, 150, 255];
    let cream = [255, 244, 220, 255];
    let metal = [193, 193, 180, 255];
    let mut rect = |x: u32, y: u32, w: u32, h: u32, color| {
        for row in y..y + h {
            for column in x..x + w {
                image.put_pixel(column, row, image::Rgba(color));
            }
        }
    };
    rect(3, 1, 23, 30, ink);
    rect(1, 3, 30, 26, ink);
    rect(26, 4, 2, 26, ink);
    rect(28, 6, 2, 23, ink);
    rect(3, 3, 21, 26, blue);
    rect(24, 5, 2, 24, blue);
    rect(26, 7, 3, 20, blue);
    rect(7, 2, 16, 11, ink);
    rect(9, 2, 12, 9, metal);
    rect(16, 3, 3, 7, blue);
    rect(6, 15, 20, 13, cream);
    rect(8, 16, 16, 1, blue);
    rect(8, 26, 16, 1, blue);
    // Stepped W, drawn on the integer grid with no gradients or antialiasing.
    for (x, y, w, h) in [
        (8, 18, 2, 3),
        (10, 20, 2, 3),
        (12, 22, 2, 3),
        (14, 20, 2, 3),
        (16, 22, 2, 3),
        (18, 20, 2, 3),
        (20, 18, 2, 3),
    ] {
        rect(x, y, w, h, blue);
    }
    rect(3, 25, 2, 3, ink);
    rect(27, 25, 2, 3, ink);
    rect(3, 26, 1, 1, cream);
    rect(28, 26, 1, 1, cream);
    image
}
