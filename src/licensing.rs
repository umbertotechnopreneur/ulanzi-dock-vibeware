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
//! Community builds retain the MIT source's complete theme collection. Store builds
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

/// Return whether `theme` belongs to the permanent, three-theme free selection.
pub fn is_free(theme: Theme) -> bool {
    matches!(
        theme,
        Theme::DarkClassic | Theme::LightClassic | Theme::LightAbstract
    )
}

/// Start the Store worker once if needed and return its current state immediately.
/// The first Store snapshot grants only the free themes until a license is verified.
pub fn snapshot() -> Snapshot {
    #[cfg(feature = "store-licensing")]
    {
        store::service().snapshot()
    }
    #[cfg(not(feature = "store-licensing"))]
    {
        Snapshot {
            owned: true,
            busy: false,
            can_purchase: false,
            price: String::new(),
            message: "Community build: all themes are included.".into(),
        }
    }
}

/// Check `theme` against current entitlement without trusting any saved preference.
pub fn allows(theme: Theme) -> bool {
    is_free(theme) || snapshot().owned
}

/// Reject an explicit attempt to select an unowned premium `theme` with UI-ready text.
pub fn require(theme: Theme) -> anyhow::Result<()> {
    anyhow::ensure!(
        allows(theme),
        "This theme needs the theme pack. Buy it or restore your purchase in Configuration."
    );
    Ok(())
}

/// Resolve a saved/CLI `theme` for rendering. A blocked preference stays saved,
/// while MATRIX is used on the dock until entitlement becomes available again.
pub fn effective(theme: Theme) -> Theme {
    if allows(theme) {
        theme
    } else {
        Theme::DarkClassic
    }
}

/// Queue a license/catalog refresh; repeated clicks while busy are coalesced.
/// No purchase UI is shown and no account-specific data is saved by this app.
pub fn refresh() {
    #[cfg(feature = "store-licensing")]
    store::service().refresh();
}

/// Begin a purchase only after a UI click. `window` owns the native Store dialog.
/// Return setup/queue errors immediately; the snapshot receives the eventual result.
pub fn purchase(window: &slint::Window) -> anyhow::Result<()> {
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
        anyhow::bail!("This build already includes all themes.")
    }
}
