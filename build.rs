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

#[path = "src/branding.rs"]
mod branding;

use std::{env, fs::File, path::PathBuf};

// Errors: propagated to Cargo for UI compilation, generated artwork, or Windows resources.
fn main() {
    println!("cargo:rerun-if-changed=ui/dialogs.slint");
    println!("cargo:rerun-if-changed=src/branding.rs");
    println!("cargo:rerun-if-changed=assets/brand/vibeware-logo.png");
    let directory = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    let floppy = branding::floppy();
    floppy
        .save(directory.join("vibeware-pixel.png"))
        .expect("pixel icon PNG");
    image::imageops::resize(&floppy, 256, 256, image::imageops::FilterType::Nearest)
        .save(directory.join("vibeware-pixel-preview.png"))
        .expect("pixel icon preview");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let frames: Vec<_> = [16, 24, 32, 48, 64, 128, 256]
            .into_iter()
            .map(|size| {
                let pixels = image::imageops::resize(
                    &floppy,
                    size,
                    size,
                    image::imageops::FilterType::Nearest,
                );
                image::codecs::ico::IcoFrame::as_png(
                    pixels.as_raw(),
                    size,
                    size,
                    image::ExtendedColorType::Rgba8,
                )
                .expect("Windows icon frame")
            })
            .collect();
        let icon = directory.join("vibeware-pixel.ico");
        image::codecs::ico::IcoEncoder::new(File::create(&icon).expect("Windows icon file"))
            .encode_images(&frames)
            .expect("Windows icon encoding");
        winresource::WindowsResource::new()
            .set_icon(icon.to_str().expect("Windows icon path"))
            .set("ProductName", "UlanziDock VibeWare")
            .set("CompanyName", "Umberto Giacobbi")
            .set("LegalCopyright", "Copyright © 2026 Umberto Giacobbi")
            .compile()
            .expect("Windows executable resources");
    }
    slint_build::compile_with_config(
        "ui/dialogs.slint",
        slint_build::CompilerConfiguration::new()
            .with_style("fluent".into())
            .embed_resources(slint_build::EmbedResourcesKind::EmbedFiles),
    )
    .expect("Slint dialog compilation");
}
