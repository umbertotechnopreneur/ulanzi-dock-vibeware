<!-- VBWR B

Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz

VibeWare: Human intent, AI execution, and plenty of tokens
Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto

Modified with AI: OpenAI Codex; documentation and VibeWare attribution; 2026-10-27
Human guidance: Umberto Giacobbi; AGENTS.md; readability and VibeWare template request
Evidence: docs/CODE-REVIEW.md and current working-tree changes

Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.

VBWR E -->

# Pictures and icons

## Find the pictures

| Folder | Contents |
| --- | --- |
| `assets/brand/` | App icon, wordmark and original VibeWare floppy. |
| `assets/boot/` | Startup picture master and its ImageGen prompt. |
| `assets/buttons/` | Exported pictures for each key and its pressed/released states. |
| `assets/production/` | Preview sheets that collect a page's buttons in one picture. |
| `assets/concepts/` and `assets/banners/` | Design pictures and banners. |
| `docs/photos/` | My desk photos, enhanced with AI. |
| `docs/design/` | Ideas for the future Windows editor. |

Most button pictures are 196 × 196 pixels. The page-index status panel is 400 × 200. The Rust drawing code in `src/art.rs` creates unavailable buttons from the normal pictures and a ribbon; there is no separate disabled-picture set.

The startup master is not a ready-made set of key files. During a source build, `build.rs` resizes and slices it for the thirteen square keys and wide panel. The controller joins those slices into one picture across the dock.

Current defaults include Windows / media, Utility, Codex, VS Code, Spotify, Word, PowerPoint, Excel, OBS Creator and Themes, plus the page index. The thirteen theme names are:

`dark-classic`, `dark-abstract`, `light-classic`, `light-abstract`, `manga-ink`, `steampunk-brass`, `cyberpunk-neon`, `moire`, `cubism`, `art-deco`, `ukiyo-e`, `solarpunk`, `memphis`.

## Rebuild preview sheets

First export button pictures with a program built from the current source. Use a separate settings file so your personal setup stays untouched. Then pass that same settings file to the picture tool:

```powershell
.\target\debug\ulanzi-dock-vibeware.exe render --config tmp/preview-settings.json --output tmp/button-export
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command atlases -Settings tmp/preview-settings.json -Source tmp/button-export
```

Create `tmp/preview-settings.json` with the app's `init` command first if it does not exist. The picture tool reads page names and order from that file instead of assuming page numbers. It expects a complete export and writes sheets to `assets/production/`; add `-Output` for a different folder. Neither command sends pictures to the dock.

The [OBS guide](../presets/obs-creator/README.md#rebuild-the-pictures-and-pdf) explains its separate preview and printable PDF.

## App icon and logo

The app icon is my pixel art, separate from official Ulanzi branding. The Windows installer uses a simpler drawing at small sizes. Rebuild its two masters with:

```powershell
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command msix-icons
```

To package the existing `assets/brand/ulanzi-dock-icon.png` master as PNG, ICO and ICNS files in `assets/icons/`:

```powershell
python scripts/tools.py icons
```

Keep the original [VibeWare floppy](../assets/brand/vibeware-logo.png) unchanged. See [branding](BRANDING.md).

Some early symbols came from AI-made sheets. The drawing code adds labels, theme colors and original Utility, Office, Themes and OBS symbols. Office and OBS symbols are not official logos. Labels use Bitstream Vera Bold; keep its [font license](../assets/fonts/bitstream-vera-license.txt).

Preview sheets show the artwork, not a live device. The [platform guide](PLATFORMS.md) separates software builds from physical checks.
