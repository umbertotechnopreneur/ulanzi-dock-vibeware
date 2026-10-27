/* VBWR B
 *
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Modified with AI: OpenAI Codex; external Store build configuration and readability; 2026-10-27
 * Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
 * Evidence: docs/CODE-REVIEW.md; current working-tree changes
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 *
 * VBWR E */

//! Build-time asset preparation and Slint/native resource compilation.
//! Store builds additionally read an external identity file into generated constants.
//! This script never starts the application, checks licenses, tests hardware, or
//! publishes packages. Generated assets stay under Cargo's OUT_DIR.

use std::{
    env,
    fs::{self, File},
    path::{Path, PathBuf},
};

// Slice one continuous splash into the dock's native display sizes at build time.
// The original ImageGen master stays unchanged and is not decoded during startup.
// directory: Cargo output folder receiving the cropped PNGs and Rust include list.
// Panics on missing artwork or encoding/write failure, which makes Cargo fail clearly.
fn prepare_boot_pictures(directory: &Path) {
    let source = image::open("assets/boot/ulanzi-dock-boot-v1.png")
        .expect("boot image master")
        .to_rgba8();
    let canvas = image::imageops::resize(&source, 988, 592, image::imageops::FilterType::Lanczos3);
    let mut module = String::from("pub const PICTURES: [&[u8]; 14] = [\n");
    for index in 0..14 {
        // The physical rows have two black gaps. Move the outside bands away
        // from those gaps while keeping the centre band anchored in place.
        let row = index / 5;
        let y = match row {
            0 => 36,
            1 => 196,
            _ => 356,
        };
        let (x, width, height) = if index == 13 {
            (588, 400, 200)
        } else {
            ((index % 5) * 196, 196, 196)
        };
        let name = format!("boot-key-{index:02}.png");
        image::imageops::crop_imm(&canvas, x, y, width, height)
            .to_image()
            .save(directory.join(&name))
            .expect("native boot picture");
        module.push_str(&format!(
            "include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{name}\")),\n"
        ));
    }
    module.push_str("];\n");
    fs::write(directory.join("boot-pictures.rs"), module).expect("embedded boot picture list");
}

// Errors: propagated to Cargo for UI compilation, generated artwork, or Windows resources.
fn main() {
    prepare_store_configuration();
    println!("cargo:rerun-if-changed=ui/dialogs.slint");
    println!("cargo:rerun-if-changed=assets/brand/ulanzi-dock-icon.png");
    println!("cargo:rerun-if-changed=assets/brand/ulanzi-dock-logo.png");
    println!("cargo:rerun-if-changed=assets/brand/vibeware-logo.png");
    println!("cargo:rerun-if-changed=assets/boot/ulanzi-dock-boot-v1.png");
    let directory = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo output directory"));
    prepare_boot_pictures(&directory);
    let source = image::open("assets/brand/ulanzi-dock-icon.png")
        .expect("product icon master")
        .to_rgba8();
    for size in [32, 256] {
        image::imageops::resize(&source, size, size, image::imageops::FilterType::Nearest)
            .save(directory.join(format!("ulanzi-dock-icon-{size}.png")))
            .expect("product icon PNG");
    }
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let frames: Vec<_> = [16, 24, 32, 48, 64, 128, 256]
            .into_iter()
            .map(|size| {
                let pixels = image::imageops::resize(
                    &source,
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
        let icon = directory.join("ulanzi-dock.ico");
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

// Store identifiers belong in an external owner-supplied file, not in source control.
// ULANZI_STORE_CONFIG: absolute external JSON path, required only for store-licensing.
// Missing/invalid data is a build failure, never a silently unrestricted Store build.
fn prepare_store_configuration() {
    println!("cargo:rerun-if-env-changed=ULANZI_STORE_CONFIG");
    if env::var_os("CARGO_FEATURE_STORE_LICENSING").is_none() {
        return;
    }
    assert_eq!(
        env::var("CARGO_CFG_TARGET_OS").as_deref(),
        Ok("windows"),
        "Store licensing requires Windows"
    );
    let path = PathBuf::from(
        env::var_os("ULANZI_STORE_CONFIG")
            .expect("Set ULANZI_STORE_CONFIG to your external Store build JSON"),
    );
    let path = fs::canonicalize(path).expect("Store build configuration file");
    let root = fs::canonicalize(env::var_os("CARGO_MANIFEST_DIR").unwrap()).unwrap();
    assert!(
        !path.starts_with(root),
        "Keep Store identifiers outside the repository"
    );
    println!("cargo:rerun-if-changed={}", path.display());
    let document: serde_json::Value =
        serde_json::from_slice(&fs::read(path).expect("read Store build configuration"))
            .expect("Store build JSON");
    let mut generated = String::new();
    for (field, constant) in [
        ("theme_product_id", "THEME_PRODUCT_ID"),
        ("package_family_name", "PACKAGE_FAMILY_NAME"),
    ] {
        let value = document[field]
            .as_str()
            .expect("missing Store configuration field");
        assert!(
            !value.trim().is_empty()
                && value.trim() == value
                && !value.chars().any(char::is_control),
            "invalid Store configuration field"
        );
        generated.push_str(&format!("pub const {constant}: &str = {value:?};\n"));
    }
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("store-config.rs"),
        generated,
    )
    .expect("generated Store configuration");
}
