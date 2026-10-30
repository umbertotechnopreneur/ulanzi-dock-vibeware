/* VBWR B
 *
 * Project: UlanziDeck
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Created with AI: OpenAI Codex; central theme entitlement policy; 2026-10-27
 * Modified with AI: OpenAI Codex; central theme entitlement policy and readability; 2026-10-27
 * Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
 * Evidence: docs/CODE-REVIEW.md; current working-tree changes
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 *
 * VBWR E */

//! One entitlement policy for the settings UI, controller and command line.
//! Only Microsoft Store supplies entitlement; settings never contain an unlock flag.
//! All builds include every theme. Future premium builds
//! delegate WinRT and networking to a worker; this module never blocks the UI on I/O.
use crate::model::Theme;

#[cfg(feature = "store-licensing")]
mod store;

#[derive(Clone, PartialEq, Eq)]
/// UI-safe copy of the last Store response. `busy` disables duplicate requests;
/// `owned` is entitlement, while `can_purchase` only describes catalog availability.
pub struct Snapshot {
    pub owned: bool,
    pub busy: bool,
    pub can_purchase: bool,
    pub price: String,
    pub message: String,
}

/// Reserved for future premium functionality. Themes are always included.
/// Enable only when the new features and their Store product are ready.
pub const PREMIUM_ENABLED: bool = false;

pub fn premium_enabled() -> bool {
    cfg!(feature = "store-licensing") && PREMIUM_ENABLED
}

/// Every current theme is free, independently of Store entitlement.
pub fn is_free(_theme: Theme) -> bool {
    true
}

/// Return the real Store entitlement only when premium functionality is enabled.
/// Disabling premium does not fabricate an owned license or start a Store worker.
pub fn snapshot() -> Snapshot {
    #[cfg(feature = "store-licensing")]
    if premium_enabled() {
        return store::service().snapshot();
    }
    Snapshot {
        owned: false,
        busy: false,
        can_purchase: false,
        price: String::new(),
        message: "All 13 themes are included. No purchase is required.".into(),
    }
}

/// Theme selection never depends on the premium switch or Store state.
pub fn allows(theme: Theme) -> bool {
    is_free(theme)
}

/// Retain the shared theme-selection entry point for settings and CLI callers.
pub fn require(_theme: Theme) -> anyhow::Result<()> {
    Ok(())
}

/// Keep the user's selected theme, including when offline or without a license.
pub fn effective(theme: Theme) -> Theme {
    theme
}

/// Queue a license/catalog refresh; repeated clicks while busy are coalesced.
/// No purchase UI is shown and no account-specific data is saved by this app.
pub fn refresh() {
    if !premium_enabled() {
        return;
    }
    #[cfg(feature = "store-licensing")]
    store::service().refresh();
}

/// Begin a purchase only after a UI click. `window` owns the native Store dialog.
/// Return setup/queue errors immediately; the snapshot receives the eventual result.
pub fn purchase(window: &slint::Window) -> anyhow::Result<()> {
    anyhow::ensure!(premium_enabled(), "Premium features are not available in this version.");
    #[cfg(feature = "store-licensing")]
    {
        use raw_window_handle::{HasWindowHandle, RawWindowHandle};
        let owner = window.window_handle();
        let handle = owner.window_handle()?;
        let RawWindowHandle::Win32(handle) = handle.as_raw() else {
            anyhow::bail!("The purchase window is not ready.");
        };
        store::service().purchase(handle.hwnd.get())
    }
    #[cfg(not(feature = "store-licensing"))]
    {
        let _ = window;
        anyhow::bail!("Premium purchases are unavailable in this build.")
    }
}
