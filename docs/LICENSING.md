<!-- VBWR B

Project: UlanziDeck
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

# Microsoft Store licensing

All thirteen current themes are included in both Store and community builds.
Theme selection works without a purchase, a verified license or a Store connection.

`PREMIUM_ENABLED` in `src/licensing.rs` is currently `false`. This source switch
keeps the existing Store infrastructure available for future premium functions.
When disabled, no Store worker, catalog request, purchase or restore is started,
and the configurator hides the purchase controls. It does not grant or fake a license.

The owner plans to place six future functions under premium. Their scope is not
implemented or advertised in this version. Before enabling the switch, define the
functions and configure the correct Store product. The existing private add-on
identifier is retained as build input; it must not automatically become an offer
for different functionality. Themes must remain included when premium is enabled.

## Retained Store infrastructure

The `store-licensing` Cargo feature still compiles the Windows Store backend.
`Snapshot.owned` remains a real entitlement value for future functions, independent
of theme availability. Purchases require an explicit UI gesture; Store owns the
payment interface and offline license cache. Settings never contain an unlock flag.

The shared `allows`, `require` and `effective` entry points now accept every theme
and preserve the selected theme. No MATRIX fallback or theme-pack suffix is applied.

## Private build configuration

Keep a JSON file **outside the repository**, for example in the owner's private
product records. Use these fields with actual Partner Center values:

```json
{
  "theme_product_id": "YOUR_THEME_ADDON_PRODUCT_ID",
  "package_family_name": "YOUR_PACKAGE_FAMILY_NAME",
  "package_name": "YOUR_PACKAGE_IDENTITY_NAME",
  "display_name": "YOUR_RESERVED_APP_NAME",
  "publisher": "YOUR_PACKAGE_PUBLISHER",
  "publisher_display_name": "YOUR_PUBLISHER_DISPLAY_NAME"
}
```

`display_name` must exactly match a reserved app name in Partner Center. Packaging
uses it for the package, Start entry and startup-task labels.

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
family to the expected family before querying Store services. With premium disabled, starting this EXE
outside the matching MSIX package keeps every theme available without Store queries.

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

The MSIX script supports `-Architecture x64` (the default) and `-Architecture arm64`.
Cross-builds use a host-native build of the same source to generate clean settings;
they do not execute the ARM64 application on an x64 PC. See [Store delivery](STORE-DELIVERY.md).

## Validation boundary

Source compilation and package inspection are separate from software tests,
Store purchases, installation and dock trials. See [Store delivery](STORE-DELIVERY.md)
for the current build results. The premium switch does not establish that future
premium functionality or purchase flows have been tested.

Before enabling a new premium offer, obtain owner authorization for the specific
Store checks, including owned/unowned accounts, cancellation, offline startup,
restore and entitlement removal.

## API references

- [Store licensing and offline cache](https://learn.microsoft.com/en-us/uwp/api/windows.services.store.storecontext.getapplicenseasync)
- [Product ID versus Store ID](https://learn.microsoft.com/en-us/windows/uwp/monetize/in-app-purchases-and-trials#how-to-use-product-ids-for-add-ons-in-your-code)
- [Durable add-on licenses](https://learn.microsoft.com/en-us/uwp/api/windows.services.store.storelicense)
- [Desktop StoreContext window ownership](https://learn.microsoft.com/en-us/uwp/api/windows.services.store.storecontext)
- [Purchase requests](https://learn.microsoft.com/en-us/uwp/api/windows.services.store.storecontext.requestpurchaseasync)
- [Create a durable add-on](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/add-on/what-are-add-ons)
