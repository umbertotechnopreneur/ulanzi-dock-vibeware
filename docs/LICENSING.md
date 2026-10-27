<!-- VBWR B

Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz

VibeWare: Human intent, AI execution, and plenty of tokens
Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto

Created with AI: OpenAI Codex; Store theme policy, private build inputs and validation limits; 2026-10-27
Modified with AI: OpenAI Codex; Store theme policy, private build inputs and validation limits and readability; 2026-10-27
Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
Evidence: docs/CODE-REVIEW.md; current working-tree changes

Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.

VBWR E -->

# Microsoft Store theme licensing

The Store edition includes three permanent free themes: **MATRIX**, **IVORY BLUE**
and **MINT TEAL**. One durable add-on unlocks the other ten. There is no trial,
subscription, account system or payment processor operated by this application.
Configure the add-on's Product lifetime as **Forever** in Partner Center.

The ordinary MIT community source build retains all thirteen themes. Store builds
must enable `store-licensing`; the Store packaging command below does this itself.
Source availability means a modified community build can remove restrictions.
This is Store entitlement enforcement, not an anti-tampering guarantee.

## Runtime responsibilities

| File | Responsibility |
| --- | --- |
| `src/licensing.rs` | Three-theme free policy and effective theme selection. |
| `src/licensing/store.rs` | Windows Store worker, license changes, catalog and purchases. |
| `src/gui/store_ui.rs` | Purchase/restore buttons, local price and status messages. |
| `src/controller.rs` | Apply entitlement to JSON/CLI preferences and dock selections. |
| `src/settings.rs` | Check new premium selections before saving preferences. |
| `build.rs` | Require external identifiers when compiling the Store edition. |

`GetAppLicenseAsync` supplies valid durable licenses, matched by `InAppOfferToken`
against the configured Product ID. Product discovery resolves that ID to the Store
ID used for purchase. The UI uses the Store's formatted price, never a fixed dollar
amount. No product IDs or publisher identities are checked into source control.

The worker initializes WinRT on its own thread. UI and dock threads only read a
small shared snapshot. Purchases require an explicit desktop button click and
associate native Store UI with that window's HWND. A locked theme button on the
dock only opens Configuration. Success/already-owned purchase responses trigger a
license reread; the payment result alone never grants themes. Cancellation and
network/server errors leave preferences intact.

Windows supplies cached licenses offline. The app does not save an editable
`premium=true` flag or implement its own receipt cache. A failed query retains the
last verified entitlement **within that process** and displays a retry message.
A successful query without the add-on revokes premium access. At a fresh startup,
only free themes are available until Windows returns a valid license.

License-change notifications and a five-minute refresh catch external purchases
and revocations. Restore purchases requests an immediate refresh. Entitlement
changes reach the running controller on its next idle poll, without restart.
When a saved premium theme is unavailable, MATRIX is displayed and the saved
preference is retained. A later valid license restores that preferred theme.
The developer artwork-export command is unavailable in Store builds.

## Private build configuration

Keep a JSON file **outside the repository**, for example in the owner's private
product records. Use these fields with actual Partner Center values:

```json
{
  "theme_product_id": "YOUR_THEME_ADDON_PRODUCT_ID",
  "package_family_name": "YOUR_PACKAGE_FAMILY_NAME",
  "package_name": "YOUR_PACKAGE_IDENTITY_NAME",
  "publisher": "YOUR_PACKAGE_PUBLISHER",
  "publisher_display_name": "YOUR_PUBLISHER_DISPLAY_NAME"
}
```

The Product ID is the name entered when creating the add-on, not its generated
Store ID. Match spelling/case exactly. The owner keeps the real file in Obsidian;
do not paste those values into this document, a public workflow or source code.
Identifiers necessarily appear in generated binaries/manifests, which remain
ignored build output. They are identifiers, not credentials.

For a source build only:

```powershell
$env:ULANZI_STORE_CONFIG = 'C:\Private\Product\store-build.json'
cargo build --locked --offline --bins --features store-licensing
```

`build.rs` fails for a missing/invalid configuration, a path inside the repository,
or a non-Windows Store build. The runtime also compares the OS-reported package
family to the expected family before querying Store services. Starting this EXE
outside the matching MSIX package leaves the three free themes available and
explains why purchases cannot be used.

To prepare an unsigned x64 Store submission from source when packaging is authorized:

```powershell
pwsh -NoProfile -File scripts/build-msix.ps1 -Configuration Release -PackageVersion 1.0.0.0 -StoreConfig 'C:\Private\Product\store-build.json'
```

This applies private package identity to the generated manifest and enables Store
licensing in the source build. It produces `*.unsigned.msix` for Partner Center;
it does not install, purchase, submit or publish. `-Install` is rejected in this mode.
Without `-StoreConfig`, the script keeps its local signing/install behavior.

For an owner-authorized local Debug installation using the same Store identity:

```powershell
pwsh -NoProfile -File scripts/build-msix.ps1 -Configuration Debug -PackageVersion 1.0.0.1 -StoreConfig 'C:\Private\Product\store-build.json' -SignStoreTestPackage
```

This requires an existing, trusted code-signing certificate matching the private
Store publisher; the script never creates or trusts certificates. It produces a
separate `-debug.msix` so Release submission output remains intact. Installation
does not create a Store entitlement or validate purchases.

`-PackageVersion` sets the four-part MSIX version separately from the source crate
version. Store packages require the first section to be at least 1 and the fourth
section to be 0. If the crate is still at 0.x, supply this parameter explicitly.

The current MSIX script packages x64. ARM64 packaging remains separate work; this
change does not establish an ARM64 Store deliverable.

## Validation boundary

The Windows x64 Store source build is checked locally. On 27 October 2026, the
owner requested a Store submission package: the Release x64 MSIX version 1.0.0.0
was built from current source with `store-licensing`, the private identity file,
and clean first-run settings. MakeAppx completed package validation. Static package
inspection confirmed identity, version, architecture, matching Release binaries,
feature selection, generated Store constants, and dependency notices.

The owner authorized builds and packaging, not licensing unit tests, GUI sessions,
Store purchases or device trials. The package has not been installed or submitted.

Before distribution, use the Microsoft Store testing flow with the matching app
identity and published add-on. Check free/owned accounts, an already-owned purchase,
cancellation, offline launch, restore after reinstall, and entitlement removal.
A successful Rust build does not validate any of those account or Store behaviors.

## API references

- [Store licensing and offline cache](https://learn.microsoft.com/en-us/uwp/api/windows.services.store.storecontext.getapplicenseasync)
- [Product ID versus Store ID](https://learn.microsoft.com/en-us/windows/uwp/monetize/in-app-purchases-and-trials#how-to-use-product-ids-for-add-ons-in-your-code)
- [Durable add-on licenses](https://learn.microsoft.com/en-us/uwp/api/windows.services.store.storelicense)
- [Desktop StoreContext window ownership](https://learn.microsoft.com/en-us/uwp/api/windows.services.store.storecontext)
- [Purchase requests](https://learn.microsoft.com/en-us/uwp/api/windows.services.store.storecontext.requestpurchaseasync)
- [Create a durable add-on](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/add-on/what-are-add-ons)
