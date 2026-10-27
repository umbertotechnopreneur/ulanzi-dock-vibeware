/* VBWR B
 *
 * Project: UlanziDock VibeWare version
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Created with AI: OpenAI Codex; Microsoft Store worker and purchase lifecycle; 2026-10-27
 * Modified with AI: OpenAI Codex; Microsoft Store worker and purchase lifecycle and readability; 2026-10-27
 * Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
 * Evidence: docs/CODE-REVIEW.md; current working-tree changes
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 *
 * VBWR E */

//! Microsoft Store adapter for packaged Windows builds. Owns one MTA worker,
//! catalog queries, cached-license reads, and explicit purchase requests.
//! It never stores an entitlement boolean on disk or collects payment details.

use super::Snapshot;
use anyhow::{ensure, Context, Result};
use std::{
    sync::{mpsc, Arc, Mutex, OnceLock},
    time::Duration,
};
use windows::{
    core::{Interface, HSTRING},
    Foundation::TypedEventHandler,
    Services::Store::{StoreContext, StoreProduct, StorePurchaseStatus},
    Win32::{
        Foundation::HWND,
        System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED},
        UI::Shell::IInitializeWithWindow,
    },
};

include!(concat!(env!("OUT_DIR"), "/store-config.rs"));

/// Work queued by UI gestures or Store license-change notifications.
enum Command {
    Refresh,
    Purchase(isize),
}

/// Process-lifetime coordinator. The worker alone touches Store COM objects;
/// the UI and controller exchange plain snapshots under a short-lived mutex.
pub struct Service {
    state: Arc<Mutex<Snapshot>>,
    commands: mpsc::Sender<Command>,
}

/// Lazily start the worker; startup failure leaves free themes usable with an error.
pub fn service() -> &'static Service {
    static INSTANCE: OnceLock<Service> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let (commands, incoming) = mpsc::channel();
        let state = Arc::new(Mutex::new(Snapshot {
            owned: false,
            busy: true,
            can_purchase: false,
            price: String::new(),
            message: "Checking your Microsoft Store license…".into(),
        }));
        let worker_state = Arc::clone(&state);
        let events = commands.clone();
        let worker_started = std::thread::Builder::new()
            .name("ulanzi-store".into())
            .spawn(move || {
                if let Err(error) = worker(&worker_state, incoming, events) {
                    let message = format!(
                        "Microsoft Store is unavailable: {error:#}. \
                         The three free themes remain available."
                    );
                    update(&worker_state, |snapshot| {
                        snapshot.busy = false;
                        snapshot.can_purchase = false;
                        snapshot.message = message;
                    });
                }
            });
        if let Err(error) = worker_started {
            update(&state, |snapshot| {
                snapshot.busy = false;
                snapshot.message = format!("Could not start licensing: {error}");
            });
        }
        Service { state, commands }
    })
}

/// Apply `change` to shared `state`. Callers must not perform I/O in the closure.
fn update(state: &Mutex<Snapshot>, change: impl FnOnce(&mut Snapshot)) {
    change(&mut state.lock().unwrap_or_else(|poison| poison.into_inner()));
}

impl Service {
    /// Clone the last result without exposing a lock guard to callers.
    pub fn snapshot(&self) -> Snapshot {
        self.state.lock().unwrap_or_else(|p| p.into_inner()).clone()
    }

    /// Mark `command` busy before queuing so duplicate UI clicks cannot start purchases.
    /// A stopped worker yields an error and disables the purchase button.
    fn send(&self, command: Command) -> Result<()> {
        let mut state = self.state.lock().unwrap_or_else(|p| p.into_inner());
        ensure!(
            !state.busy,
            "A Microsoft Store request is already in progress."
        );
        state.busy = true;
        state.message = "Waiting for Microsoft Store…".into();
        if self.commands.send(command).is_err() {
            state.busy = false;
            state.can_purchase = false;
            state.message = "Microsoft Store is unavailable. Restart the app to retry.".into();
            anyhow::bail!("Microsoft Store worker is unavailable.");
        }
        Ok(())
    }

    /// Request a license reread; an existing request already serves repeated clicks.
    pub fn refresh(&self) {
        let _ = self.send(Command::Refresh);
    }

    /// Queue an explicit purchase owned by the live Win32 `owner` HWND.
    /// Reject missing owners, owned products, unavailable listings, and concurrent work.
    pub fn purchase(&self, owner: isize) -> Result<()> {
        let state = self.snapshot();
        ensure!(
            !state.owned && state.can_purchase,
            "The theme pack is not currently available for purchase."
        );
        ensure!(owner != 0, "The purchase window is not ready.");
        self.send(Command::Purchase(owner))
    }
}

/// Balances successful RoInitialize even when a later Store call returns an error.
struct Apartment;
impl Drop for Apartment {
    fn drop(&mut self) {
        unsafe { RoUninitialize() };
    }
}

/// Initialize Store services after verifying package identity, then service requests.
/// `incoming` carries UI commands; `events` routes Store changes through the same
/// serial queue. Five-minute polling also picks up purchases in another process.
fn worker(
    state: &Mutex<Snapshot>,
    incoming: mpsc::Receiver<Command>,
    events: mpsc::Sender<Command>,
) -> Result<()> {
    verify_package()?;
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)?;
    }
    let _apartment = Apartment;
    let context = StoreContext::GetDefault()?;
    // This subscription lives with the context. No disk flag or app-managed receipt cache.
    let event = context.OfflineLicensesChanged(&TypedEventHandler::new(move |_, _| {
        let _ = events.send(Command::Refresh);
        Ok(())
    }))?;
    refresh(&context, state);
    update(state, |snapshot| snapshot.busy = false);
    loop {
        let command = match incoming.recv_timeout(Duration::from_secs(300)) {
            Ok(command) => command,
            Err(mpsc::RecvTimeoutError::Timeout) => Command::Refresh,
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        };
        update(state, |s| s.busy = true);
        match command {
            Command::Refresh => refresh(&context, state),
            Command::Purchase(owner) => {
                let outcome = purchase(&context, owner);
                // A successful purchase result alone does not grant themes: reload entitlement.
                refresh(&context, state);
                update(state, |snapshot| {
                    snapshot.message = match outcome {
                        Ok(Purchase::Completed) if snapshot.owned => {
                            "Theme pack purchased. All 13 themes are available.".into()
                        }
                        Ok(Purchase::Completed) => {
                            "Purchase received. License confirmation is pending; \
                             use Restore purchases to retry."
                                .into()
                        }
                        Ok(Purchase::Cancelled) => {
                            "Purchase cancelled. No themes were changed.".into()
                        }
                        Err(error) => format!("Purchase could not be completed: {error:#}"),
                    };
                });
            }
        }
        update(state, |s| s.busy = false);
    }
    context.RemoveOfflineLicensesChanged(event)?;
    Ok(())
}

/// Read entitlement before the catalog. Catalog failure must not revoke a purchase.
/// `state` receives errors as text; verified absence revokes, transient errors retain
/// the last verified in-process entitlement. Windows owns the offline license cache.
fn refresh(context: &StoreContext, state: &Mutex<Snapshot>) {
    match owned(context) {
        Ok(owned) => update(state, |s| {
            s.owned = owned;
            s.message = if owned {
                "Theme pack owned. All 13 themes are available."
            } else {
                "3 free themes. Unlock the other 10 with a one-time purchase."
            }
            .into();
        }),
        Err(error) => update(state, |s| {
            // Retain the last verified entitlement in this process on a transport failure.
            // An authoritative license result without this add-on revokes it above.
            s.message = format!(
                "Could not refresh the Store license: {error:#}. Retry with Restore purchases."
            );
        }),
    }
    match product(context) {
        Ok(product) => match product.Price().and_then(|price| price.FormattedPrice()) {
            Ok(price) => update(state, |s| {
                s.price = price.to_string();
                s.can_purchase = !s.owned;
            }),
            Err(_) => update(state, |s| {
                s.can_purchase = false;
                s.price.clear();
            }),
        },
        Err(error) => update(state, |s| {
            s.can_purchase = false;
            s.price.clear();
            if !s.owned {
                s.message
                    .push_str(&format!(" Purchase unavailable: {error:#}"));
            }
        }),
    }
}

/// Query valid durable licenses for `context` and match the configured offer token.
/// Return false only after a successful response, never by converting an API error.
fn owned(context: &StoreContext) -> Result<bool> {
    // Windows returns its cached licenses offline, including their validity.
    let app = context.GetAppLicenseAsync()?.get()?;
    if !app.IsActive()? {
        return Ok(false);
    }
    for entry in app.AddOnLicenses()? {
        let license = entry.Value()?;
        if license.InAppOfferToken()? == THEME_PRODUCT_ID {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Resolve the owner's Product ID to the catalog's StoreProduct and Store ID.
/// Missing regional availability or unpublished listings are actionable errors.
fn product(context: &StoreContext) -> Result<StoreProduct> {
    let kinds: windows_collections::IIterable<HSTRING> = vec![HSTRING::from("Durable")].into();
    let result = context.GetAssociatedStoreProductsAsync(&kinds)?.get()?;
    result.ExtendedError()?.ok()?;
    for entry in result.Products()? {
        let product = entry.Value()?;
        if product.InAppOfferToken()? == THEME_PRODUCT_ID {
            return Ok(product);
        }
    }
    anyhow::bail!("The theme pack is not published or is unavailable for this account/region")
}

/// Purchase feedback is separate from entitlement; only a license reread unlocks.
enum Purchase {
    Completed,
    Cancelled,
}

/// Associate native Store UI with `owner`, obtain the current catalog product, and
/// wait on this worker for Microsoft's result. Payment details remain in Store UI.
fn purchase(context: &StoreContext, owner: isize) -> Result<Purchase> {
    // The user may have closed the configurator while the request was queued.
    ensure!(
        unsafe { windows_sys::Win32::UI::WindowsAndMessaging::IsWindow(owner as *mut _) } != 0,
        "The purchase window was closed. Open Configuration and try again."
    );
    // Use a separate context so the owner belongs to this explicit user gesture.
    let purchase_context = StoreContext::GetDefault()?;
    let desktop: IInitializeWithWindow = purchase_context.cast()?;
    unsafe {
        desktop.Initialize(HWND(owner as *mut _))?;
    }
    let store_id = product(context)?.StoreId()?;
    let result = purchase_context.RequestPurchaseAsync(&store_id)?.get()?;
    match result.Status()? {
        StorePurchaseStatus::Succeeded | StorePurchaseStatus::AlreadyPurchased => {
            Ok(Purchase::Completed)
        }
        StorePurchaseStatus::NotPurchased => Ok(Purchase::Cancelled),
        _ => {
            result.ExtendedError()?.ok()?;
            anyhow::bail!("Microsoft Store returned a network or server error; try again")
        }
    }
}

/// Compare OS-reported package identity to build-time configuration before any query.
/// Portable or mismatched packages remain on the three free themes with an explanation.
fn verify_package() -> Result<()> {
    use windows_sys::Win32::{
        Foundation::ERROR_INSUFFICIENT_BUFFER,
        Storage::Packaging::Appx::GetCurrentPackageFamilyName,
    };
    let mut length = 0;
    ensure!(
        unsafe { GetCurrentPackageFamilyName(&mut length, std::ptr::null_mut()) }
            == ERROR_INSUFFICIENT_BUFFER,
        "Install the Microsoft Store edition to use purchases"
    );
    let mut name = vec![0u16; length as usize];
    ensure!(
        unsafe { GetCurrentPackageFamilyName(&mut length, name.as_mut_ptr()) } == 0,
        "Could not read package identity"
    );
    let end = name.iter().position(|&c| c == 0).unwrap_or(name.len());
    ensure!(
        String::from_utf16(&name[..end]).context("package identity")? == PACKAGE_FAMILY_NAME,
        "This package does not match the configured Microsoft Store product"
    );
    Ok(())
}
