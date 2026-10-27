/* VBWR B
 *
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Created with AI: OpenAI Codex; Store purchase and restore controls; 2026-10-27
 * Modified with AI: OpenAI Codex; Store purchase and restore controls and readability; 2026-10-27
 * Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
 * Evidence: docs/CODE-REVIEW.md; current working-tree changes
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 *
 * VBWR E */

//! Bind Store status and explicit purchase/restore gestures to the Slint configurator.
//! The timer only reads snapshots; Store requests and entitlement decisions live
//! in `licensing`, so this module does not perform network I/O or grant themes.

use super::MainWindow;
use crate::{licensing, model::Theme};
use slint::ComponentHandle;
use std::{cell::RefCell, time::Duration};

/// Attach callbacks to `window` and return the timer its owner must retain.
/// Weak window handles prevent the timer and callbacks from keeping closed UI alive.
pub(super) fn bind(window: &MainWindow) -> slint::Timer {
    window.set_store_edition(cfg!(feature = "store-licensing"));
    let weak = window.as_weak();
    window.on_buy_themes(move || {
        if let Some(window) = weak.upgrade() {
            if let Err(error) = licensing::purchase(window.window()) {
                window.set_message(format!("{error:#}").into());
            }
        }
    });
    window.on_restore_purchases(licensing::refresh);
    apply(window, &licensing::snapshot());
    let weak = window.as_weak();
    let previous = RefCell::new(None);
    let timer = slint::Timer::default();
    timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(250),
        move || {
            let Some(window) = weak.upgrade() else {
                return;
            };
            let snapshot = licensing::snapshot();
            if previous.borrow().as_ref() != Some(&snapshot) {
                apply(&window, &snapshot);
                *previous.borrow_mut() = Some(snapshot);
            }
        },
    );
    timer
}

/// Copy one consistent `snapshot` into `window` without changing an unsaved selection.
fn apply(window: &MainWindow, snapshot: &licensing::Snapshot) {
    window.set_store_owned(snapshot.owned);
    window.set_store_busy(snapshot.busy);
    window.set_store_can_purchase(snapshot.can_purchase);
    window.set_store_price(snapshot.price.clone().into());
    window.set_store_message(snapshot.message.clone().into());
    let themes: Vec<slint::SharedString> = Theme::ALL
        .into_iter()
        .map(|theme| {
            if licensing::is_free(theme) || snapshot.owned {
                theme.label().into()
            } else {
                format!("{} (Theme pack)", theme.label()).into()
            }
        })
        .collect();
    window.set_themes(slint::ModelRc::new(slint::VecModel::from(themes)));
}
