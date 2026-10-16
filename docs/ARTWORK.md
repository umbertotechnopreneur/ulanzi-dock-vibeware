<!-- VBWR B
Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
VibeWare is Human intent, AI execution, and plenty of tokens
Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
AI Tooling: May include OpenAI Codex, GitHub Copilot and AI-assisted CI/CD pipelines.
AI Versions: Tools and models may vary by contributor and execution environment.
AI Traceability: Refer to Git history and CI/CD logs for recorded provenance.
Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
VBWR E -->

# Pictures and icons

## Where the pictures are

- `assets/brand/`: the app icon, wordmark and original VibeWare floppy.
- `assets/buttons/`: one PNG for each button and state.
- `assets/production/`: sheets that collect a page's buttons in one picture. These are previews, not the only source of button images.
- `assets/concepts/` and `assets/banners/`: AI-made design pictures.
- `docs/design/`: ideas for the future Windows interface.

Most button images are 196 × 196 pixels. The page-index status panel is 400 × 200. Each action has a released and pressed image for every theme. Disabled buttons are made from these images at runtime, not saved as another set.

The regular pages are Windows / media, Utility, Codex, VS Code, Spotify, Word, PowerPoint, Excel and Themes. Artwork exports also include the page index.

## App icon and VibeWare logo

The app icon is 8-bit pixel art. The MSIX uses a simpler drawing below 64 pixels and a more detailed one at larger sizes. See `scripts/render-msix-icons.ps1` and the two `ulanzi-dock-msix-icon` PNGs in `assets/brand/`.

The earlier icon and wordmark are `ulanzi-dock-icon.png` and `ulanzi-dock-logo.png`. They are my product art, not official Ulanzi branding. `scripts/build-icons.py` makes the PNG, ICO and ICNS files in `assets/icons/`.

Use the original [VibeWare floppy](../assets/brand/vibeware-logo.png) unchanged. It is separate from the app icon. See [branding](BRANDING.md).

## Button artwork

Some early symbols came from AI-made sheets. The Rust renderer in `src/art.rs` adds labels, colors and original symbols for Utility, Office, Themes and OBS. The Office and OBS symbols are not official logos.

`assets/overlay/not-available.png` is the ribbon used on disabled keys. The app colors it for the current theme.

There are thirteen themes. The settings use these names:

`dark-classic`, `dark-abstract`, `light-classic`, `light-abstract`, `manga-ink`, `steampunk-brass`, `cyberpunk-neon`, `moire`, `cubism`, `art-deco`, `ukiyo-e`, `solarpunk`, `memphis`.

Labels use Bitstream Vera Bold. Keep its [font license](../assets/fonts/bitstream-vera-license.txt).

To export buttons and rebuild the preview sheets:

```powershell
ulanzi-dock-vibeware render --output assets/buttons
python docs/build_production_atlases.py
```

The [OBS guide](../presets/obs-creator/README.md) has its own export steps. The older PDF catalogue does not include the newer Office pages.

## What has been checked

I saw the earlier light-abstract Windows/media, Codex and VS Code artwork working on my D200H, including NEXT PAGE. The later styles and pages still need a physical check.

The desk photos in the README were enhanced with AI. Other previews and mockups are not photos of working hardware. Linux desktop icons, macOS app installation and the updated About window still need an interactive check.
