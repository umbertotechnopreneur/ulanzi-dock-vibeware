<!-- VBWR B
Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
VibeWare is Human intent. AI implementation. Accountable human review.
Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
AI Tooling: May include OpenAI Codex, GitHub Copilot and AI-assisted CI/CD pipelines.
AI Versions: Tools and models may vary by contributor and execution environment.
AI Traceability: Refer to Git history and CI/CD logs for recorded provenance.
Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
VBWR E -->

# Artwork sources and production atlases

The two wide README illustrations in `assets/banners/` were generated with ImageGen on 7 October 2026. `ulanzi-dock-dark.png` uses a dark blue and cyan terminal palette; `ulanzi-dock-light.png` uses ivory and cobalt. Both depict a conceptual control dock, leave space for separately added text, and contain no generated VibeWare logo. They are illustrative artwork, not photographs of the D200H or evidence of device output. The original VibeWare logo remains unchanged in `assets/brand/vibeware-logo.png`; the CLI embeds that file for its version and about pages.

The transparent `assets/overlay/not-available.png` is the fourth, green ribbon chosen from four ImageGen proposals on 7 October 2026. It is the only new overlay asset. The controller resizes and tints this PNG at runtime for the current theme, then composites it over grayscale inactive application keys. The artwork is a source asset, not a captured image from the device.

Three ImageGen reference sheets remain in `assets/concepts/`, one for each of the original Windows/media, Codex, and VS Code pages. They provide smooth pictogram silhouettes. Their card frames and lettering are excluded from the production sprites. The original ImageGen prompt requested a 7-row by 4-column comparison of released/up and pressed/down button concepts with English labels and clear spacing. Utility and Spotify add native symbols; the Themes page uses native palette swatches. The nine new palettes and motifs are deterministic renderer output, not additional source-sheet crops. Earlier pixel-art concepts were superseded and are not part of the distributable artwork.

`docs/icon-gradient-concept.png` is the ImageGen comparison sheet approved on 7 October 2026. It repeats one Play glyph across all thirteen themes to show each theme's three-color treatment. The renderer uses those palette directions to interpolate color inside the existing sharp pictogram masks and the native theme swatches; the sheet is a design reference, not a runtime texture or a device photograph. The existing VibeWare logo is not modified.

The 78 current full-color atlases in `assets/production/` are assembled from the exact 196×196 PNGs in `assets/buttons/`. Each sheet covers one of six pages and one of thirteen themes. `docs/build_production_atlases.py` recreates them after `ulanzi-dock-vibeware render --output assets/buttons`.

Theme variations:

- `dark-classic`: Matrix-inspired deep green gradient, mint glow, bright active state.
- `dark-abstract`: Blade Runner-inspired violet/blue gradient, warm amber icons, neon active state.
- `light-classic`: ivory-to-blue gradient, navy icons and a soft shadow.
- `light-abstract`: ivory-to-mint gradient, teal icons and a soft shadow.
- `manga-ink`: paper, ink, red active accent, and halftone dots.
- `steampunk-brass`: copper and brass over a dark mechanical ring motif.
- `cyberpunk-neon`: violet and electric cyan with magenta light traces.
- `moire`: pale blue-green interference waves at deliberately low contrast.
- `cubism`: terracotta and blue geometric planes.
- `art-deco`: emerald and gold fan rays.
- `ukiyo-e`: indigo, cream, and stylized waves.
- `solarpunk`: sunlit yellow and leaf green rays.
- `memphis`: coral, lavender, lemon, playful dots, and diagonal marks.

Page action lists (index order):

- **Windows / media:** Play / pause, Previous, Next track, Mute, Next page, Volume down, Volume up, File Explorer, Desktop, Settings, Screenshot, Task view, Browser, Clock.
- **Codex:** New chat, Projects, Attention, Side chat, Next page, Review, Terminal, Files, Commands, Copy link, Copy, Paste, Find, Clock.
- **VS Code:** Quick open, Commands, Terminal, Search, Next page, Explorer, Source control, Debug, Extensions, Save, New file, Format, Problems, Clock.
- **Utility:** Clipboard, Search, Emoji, Screenshot, Next page, Explorer, Desktop, Task view, Settings, Project, Copy, Paste, Undo, Clock.
- **Themes:** Twelve choices showing all themes except the active one, Next page at index 4, and Clock at index 13. Each choice uses the target theme as its own preview and has up/down art.
- **Spotify:** Play / pause, Previous, Next track, Shuffle, Next page, Repeat, Search, Library, Like, Queue, Now playing, Liked songs, Home, Clock.

The renderer preserves each icon's proportions, uses antialiased Bitstream Vera Bold labels, and fills every pixel of the button face without drawing a bezel. Each of the 13 action keys has an up and down state in every theme and page. Index 13 exports a blank placeholder; the firmware clock owns that display. The font is bundled under the license in `assets/fonts/bitstream-vera-license.txt`. The gradient sprites were exported offline and assembled into new atlases and the PDF catalogue; their physical display and ZIP packaging have not been checked. Earlier, on 7 October 2026, the owner observed the light-abstract Windows/media, Codex, and VS Code pages on the physical D200H: their previous icon treatment and labels were intact and readable, and NEXT PAGE worked. The timed display later cleared; resident mode restored it. The nine new styles and the Utility and Themes pages have not yet been physically verified.
