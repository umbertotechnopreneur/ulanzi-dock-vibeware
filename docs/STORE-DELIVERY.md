<!-- Copyright (c) 2026 Umberto Giacobbi. SPDX-License-Identifier: MIT.
Created with AI: OpenAI Codex; owner-requested Store package and artwork; 2026-10-27. -->

# Microsoft Store delivery

The delivery folder is `artifacts/store-upload/1.0.3.0`. The archive
`UlanziDeck-Store-1.0.3.0-x64-arm64-kit.zip` beside it contains the same files.
Both Windows packages are built from this repository's source with
Store licensing enabled and the owner's existing Partner Center identity.

The reserved public name is **UlanziDeck**. Version 1.0.3.0 corrects the
package DisplayName and application/startup labels that caused the 1.0.2.0
upload rejection. The private Store configuration supplies this name explicitly.

## Upload files

| Partner Center field | File |
| --- | --- |
| Packages — x64 | `ulanzi-dock-vibeware-1.0.3.0-x64.unsigned.msix` |
| Packages — ARM64 | `ulanzi-dock-vibeware-1.0.3.0-arm64.unsigned.msix` |
| App tile icon | `listing/app-tile-300x300.png` |
| Super hero art | `listing/super-hero-3840x2160.png` or the 1920 x 1080 version |
| Desktop screenshots | The four files in `screenshots/` |

The MSIX files are unsigned for Store submission. Microsoft signs accepted packages.
Upload both MSIX files in Packages, rather than the whole delivery ZIP. They use
the same identity and version, with native executables for each architecture.
Remove the rejected 1.0.2.0 uploads and select Windows 10/11 Desktop in device
family availability. Partner Center acceptance still needs to complete.

The two titled banners are additional promotional artwork. Use the **text-free**
super hero in the Super hero art field. Square promotional art is also included.
Xbox and Holographic-specific fields do not apply to this Desktop package.

`windows-icons/` contains the 77 PNG assets embedded in the MSIX, including scale
variants and target-size variants with unplated forms. No separate upload is needed
for them. `brand/` contains the original brand masters and a seven-size Windows ICO.
The original VibeWare logo is preserved.

## Screenshot captions

1. Configure themes, application detection and enabled shortcut pages.
2. The CLI controller running on my D200H (version 0.2.0).
3. A close view of the D200H theme buttons and wide display.
4. The D200H on a clean desk with an ergonomic mouse and mousepad.

The first screenshot was captured from the current Release configurator, without
connecting to the dock. It shows the Store-unavailable message because the EXE
was launched outside an installed Store package. The CLI screenshot is the
owner-selected 0.2.0 capture, unchanged. The two product photos were retouched
with AI at the owner's request. Source photos remain unchanged in `docs/photos/`.

## What was checked

Release compilation ran in offline mode. Three locked ARM64 dependency packages
were downloaded and scanned before use; see [the import record](ARM64-DEPENDENCY-IMPORT.md).
MakeAppx completed its package validation. Static
inspection checks identity, version, architecture, executable contents, default
settings, included artwork and notices. See the delivery's `verification.json`
and `inventory.json` for results and file dimensions.

No installation, software test suite, Windows App Certification Kit run, purchase,
restore, dock trial or Store submission was performed. Screenshot capture only
opened the standalone configurator with a separate temporary settings file.
ARM64 was cross-compiled on this x64 PC; it was not run on ARM64 hardware.

## Rebuild

Use the private Store JSON outside this repository. Keep the same identity and
choose the next submission version in Partner Center.

```powershell
$env:CARGO_NET_OFFLINE = 'true'
pwsh -NoProfile -File scripts/build-msix.ps1 -Architecture x64 -Configuration Release -PackageVersion 1.0.3.0 -StoreConfig 'C:\Private\Product\store-build.json' -KeepStaging
pwsh -NoProfile -File scripts/build-msix.ps1 -Architecture arm64 -Configuration Release -PackageVersion 1.0.3.0 -StoreConfig 'C:\Private\Product\store-build.json' -KeepStaging
pwsh -NoProfile -File scripts/prepare-store-artwork.ps1
```

The artwork command creates deterministic icons and promotional graphics from the
existing pixel-art masters. It preserves separately captured screenshots and photos.
Before introducing a new external dependency, scan its archive and extracted files
once with Windows Defender and record the source, version, command and clean result.

Artwork dimensions follow [Microsoft's Store image guidance](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images).
