<!-- Copyright (c) 2026 Umberto Giacobbi. SPDX-License-Identifier: MIT. -->

# Microsoft Store delivery

Use **UlanziDeck** as the public name. All thirteen themes are included;
premium purchases are disabled. Keep the Store identity in the private JSON
outside the repository. See [Store configuration](LICENSING.md).

## Prepare packages and artwork

Choose the submission version in Partner Center. Build both architectures from
the current source with the same identity and version. For example:

```powershell
$packageVersion = '1.0.0.0' # Replace with the intended submission version.
$storeConfig = 'C:\Private\Product\store-build.json'
$env:CARGO_NET_OFFLINE = 'true'
pwsh -NoProfile -File scripts/build-msix.ps1 -Architecture x64 -Configuration Release -PackageVersion $packageVersion -StoreConfig $storeConfig -KeepStaging
pwsh -NoProfile -File scripts/build-msix.ps1 -Architecture arm64 -Configuration Release -PackageVersion $packageVersion -StoreConfig $storeConfig -KeepStaging
pwsh -NoProfile -File scripts/prepare-store-artwork.ps1 -OutputDirectory "artifacts/store-upload/$packageVersion"
```

Packages go in `artifacts/msix/`. The artwork command writes listing images,
Windows icons and brand files to the selected output folder. It preserves
separately captured screenshots and photos. Use screenshots of the current app.

Before importing a new dependency, follow the
[first-import steps](DEVELOPMENT.md#prepare-new-dependencies).

## Upload files

| Partner Center field | File |
| --- | --- |
| Packages | The x64 and ARM64 `*.unsigned.msix` files for the chosen version |
| App tile icon | `listing/app-tile-300x300.png` |
| Store display images | `windows-icons/Square150x150Logo.png` and `Square71x71Logo.png` |
| Poster art | `listing/poster-art-1440x2160.png` |
| Box art | `listing/box-art-2160x2160.png` |
| Super hero art | `listing/super-hero-3840x2160.png` or the 1920 x 1080 version |
| Desktop screenshots | Current images from `screenshots/` |
| English listing text | [Store listing copy](STORE-LISTING.en-US.md) |

Microsoft signs accepted Store packages. Upload the unsigned MSIX files,
not a delivery ZIP or a locally signed installer. Select Windows 10/11 Desktop
in device availability. Xbox and Holographic fields do not apply.

Use the text-free image for Super hero art. Poster and box art are optional for
this Desktop app. Windows icon variants are embedded in the package and do not
need separate uploads. Keep the original VibeWare logo unchanged.

Check identity, version, architecture, executable contents, clean default settings
and license notices before submission. Publishing requires my explicit request.
See [Microsoft's image guidance](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images)
for dimensions and [packaging](RELEASING.md) for other download formats.
