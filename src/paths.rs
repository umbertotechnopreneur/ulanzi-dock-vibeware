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
 *
 * VBWR E */

//! Resolve portable and MSIX settings locations and seed a new package profile safely.
//! Existing user files are preserved. These helpers do not migrate old formats, install
//! packages or load the application UI; the launcher and controller share them.

use anyhow::{Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Create a separate local log for one controller launch. No device access.
/// Store logs beside the writable profile, never inside the installed package.
pub fn controller_log(settings: &Path) -> Result<fs::File> {
    use std::io::Write;
    let directory = settings.parent().context("locating the log folder")?.join("logs");
    fs::create_dir_all(&directory).context("creating the controller log folder")?;
    let name = format!("controller-{}-{}.log", chrono::Utc::now().format("%Y%m%dT%H%M%S%.3fZ"), std::process::id());
    let mut file = fs::OpenOptions::new().create_new(true).append(true)
        .open(directory.join(name)).context("opening the controller log")?;
    writeln!(file, "Controller launch UTC={} executable={} settings={}",
        chrono::Utc::now().to_rfc3339(), std::env::current_exe()?.display(), settings.display())?;
    Ok(file)
}

/// Resolve the writable settings file for a normal launch.
pub fn default_settings() -> Result<PathBuf> {
    let executable = std::env::current_exe().context("locating the executable")?;
    if let Some(settings) = package_settings(&executable)? {
        return Ok(settings);
    }

    let directory = executable
        .parent()
        .context("locating the executable folder")?;
    // Local builds in artifacts/ share the repository settings rather than another copy.
    let repository = directory.parent().filter(|parent| {
        directory
            .file_name()
            .is_some_and(|name| name == "artifacts")
            && parent.join("Cargo.toml").is_file()
    });
    Ok(repository.unwrap_or(directory).join("settings.json"))
}

/// Seed package settings once; portable executables have no package identity.
pub fn package_settings(executable: &Path) -> Result<Option<PathBuf>> {
    let Some(local_state) = package_local_state()? else {
        return Ok(None);
    };

    fs::create_dir_all(&local_state)
        .with_context(|| format!("creating {}", local_state.display()))?;

    let settings = local_state.join("settings.json");
    let transaction = crate::persistence::Transaction::begin(&settings)?;
    if transaction.read()?.is_none() {
        let source = executable
            .parent()
            .context("locating the MSIX installation folder")?
            .join("settings.json");
        let bytes = fs::read(&source).with_context(|| format!("reading {}", source.display()))?;
        transaction.create(&bytes)?;
    }

    Ok(Some(settings))
}

/// Report whether this process has MSIX package identity without touching settings.
pub fn is_packaged() -> Result<bool> {
    Ok(package_local_state()?.is_some())
}

#[cfg(target_os = "windows")]
/// Resolve the current package's per-user LocalState folder without creating it.
/// No package returns `None`; Windows identity/environment failures are errors.
fn package_local_state() -> Result<Option<PathBuf>> {
    use anyhow::ensure;
    use std::{ffi::OsString, os::windows::ffi::OsStringExt, ptr};
    use windows_sys::Win32::{
        Foundation::{APPMODEL_ERROR_NO_PACKAGE, ERROR_INSUFFICIENT_BUFFER},
        Storage::Packaging::Appx::GetCurrentPackageFamilyName,
    };

    let mut length = 0;
    // Windows first returns the required UTF-16 buffer length, including its terminator.
    let result = unsafe { GetCurrentPackageFamilyName(&mut length, ptr::null_mut()) };
    if result == APPMODEL_ERROR_NO_PACKAGE {
        return Ok(None);
    }
    ensure!(
        result == ERROR_INSUFFICIENT_BUFFER && length > 0,
        "reading the MSIX package identity failed with Windows error {result}"
    );

    let mut family_name = vec![0_u16; length as usize];
    let result = unsafe { GetCurrentPackageFamilyName(&mut length, family_name.as_mut_ptr()) };
    ensure!(
        result == 0,
        "reading the MSIX package identity failed with Windows error {result}"
    );

    let characters = family_name
        .split(|character| *character == 0)
        .next()
        .unwrap_or_default();
    let family_name = OsString::from_wide(characters);
    let local_app_data = std::env::var_os("LOCALAPPDATA")
        .context("LOCALAPPDATA is unavailable for the MSIX package")?;

    Ok(Some(
        PathBuf::from(local_app_data)
            .join("Packages")
            .join(family_name)
            .join("LocalState"),
    ))
}

#[cfg(not(target_os = "windows"))]
/// Resolve the current package's per-user LocalState folder without creating it.
/// No package returns `None`; Windows identity/environment failures are errors.
fn package_local_state() -> Result<Option<PathBuf>> {
    Ok(None)
}
