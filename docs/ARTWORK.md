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

The two wide README illustrations in `assets/banners/` were generated with ImageGen on 7 October 2026. `ulanzi-dock-dark.png` uses a dark blue and cyan terminal palette; `ulanzi-dock-light.png` uses ivory and cobalt. Both depict a conceptual control dock, leave space for separately added text, and contain no generated VibeWare logo. They are illustrative artwork, not photographs of the D200H or evidence of device output. The original VibeWare logo remains unchanged in `assets/brand/vibeware-logo.png`; the CLI and Slint About dialog embed that file.

The About layout follows the owner's VoiceDucker reference: product/version, creator, project links, and a Close button. The owner authorized reuse of his assets on 8 October 2026. VoiceDucker's VibeWare PNG and this repository's existing PNG have identical SHA-256 `84321fe9f77ccc083813e02c02ff3716ad50a45ed000b8663c5a2d416e2ba5af`; no duplicate copy or VoiceDucker-specific banner is needed. The final identity panel says **This is VibeWare**; its left side carries the unchanged motto and **What is VibeWare?** manifesto link, with the original floppy on the right.

On 8 October 2026, the owner requested a separate ImageGen product logo and desktop icon for this open dock driver. `assets/brand/ulanzi-dock-icon.png` is the transparent 1280×1280 icon master: an 8-bit control pad with blue keys, cream case, status strip and USB cable. `assets/brand/ulanzi-dock-logo.png` is the transparent 2172×724 wordmark, generated with that icon as its reference and the exact labels **ULANZIDOCK** and **OPEN DOCK DRIVER**. They are original product artwork, not the official Ulanzi logo, a hardware photograph, or replacements for the approved VibeWare floppy.

[build.rs](../build.rs) embeds the product icon in the Windows EXE and tray; Slint embeds the master for both window icons and the product wordmark above the dialog content. [build-icons.py](../scripts/build-icons.py) packages PNG sizes 16–1024, a multi-size Windows ICO and a macOS ICNS in `assets/icons/`, using nearest-neighbor scaling and preserving transparency. Linux can use the PNG variants. Icon files have been decoded locally, but Linux desktop integration and macOS app-bundle installation have not been checked. Dialog frames use square borders, a cream/navy palette, and monospace text, without neon effects.

The transparent `assets/overlay/not-available.png` is the fourth, green ribbon chosen from four ImageGen proposals on 7 October 2026. It is the only new overlay asset. The controller resizes and tints this PNG at runtime for the current theme, then composites it over grayscale inactive application keys. The artwork is a source asset, not a captured image from the device.

Three ImageGen reference sheets remain in `assets/concepts/`, one for each of the original Windows/media, Codex, and VS Code pages. They provide smooth pictogram silhouettes. Their card frames and lettering are excluded from the production sprites. The original ImageGen prompt requested a 7-row by 4-column comparison of released/up and pressed/down button concepts with English labels and clear spacing. Utility, Spotify, Word, PowerPoint and Excel add native symbols; the Themes page uses native palette swatches. The Office symbols are original geometry drawn in `src/art.rs`, not Microsoft logos or copied icon sets. The nine new palettes and motifs are deterministic renderer output, not additional source-sheet crops. Earlier pixel-art concepts were superseded and are not part of the distributable artwork.

`docs/icon-gradient-concept.png` is the ImageGen comparison sheet approved on 7 October 2026. It repeats one Play glyph across all thirteen themes to show each theme's three-color treatment. The renderer uses those palette directions to interpolate color inside the existing sharp pictogram masks and the native theme swatches; the sheet is a design reference, not a runtime texture or a device photograph. The existing VibeWare logo is not modified.

The 117 current full-color atlases in `assets/production/` are assembled from the exact 196×196 PNGs in `assets/buttons/`. Each sheet covers one of nine pages and one of thirteen themes. `docs/build_production_atlases.py` recreates them after `ulanzi-dock-vibeware render --output assets/buttons`.

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

Current page order: Windows / media, Utility, Codex, VS Code, Spotify, Word, PowerPoint, Excel, Themes.
The existing PDF catalogue retains the earlier sequence and omits Office. Page action lists
(physical key index order):

- **Windows / media:** Play / pause, Previous, Next track, Mute, Next page, Volume down, Volume up, File Explorer, Desktop, Settings, Screenshot, Task view, Browser, Clock.
- **Codex:** New chat, Projects, Attention, Side chat, Next page, Review, Terminal, Files, Commands, Copy link, Copy, Paste, Find, Clock.
- **VS Code:** Quick open, Commands, Terminal, Search, Next page, Explorer, Source control, Debug, Extensions, Save, New file, Format, Problems, Clock.
- **Utility:** Clipboard, Search, Emoji, Screenshot, Next page, Explorer, Desktop, Task view, Settings, Project, Copy, Paste, Undo, Clock.
- **Themes:** Twelve choices showing all themes except the active one, Next page at index 4, and Clock at index 13. Each choice uses the target theme as its own preview and has up/down art.
- **Spotify:** Play / pause, Previous, Next track, Shuffle, Next page, Repeat, Search, Library, Like, Queue, Now playing, Liked songs, Home, Clock.
- **Word:** Save, Open, New file, Print, Next page, Undo, Redo, Find, Bold, Italic, Underline, Spelling, Save as, Clock.
- **PowerPoint:** Save, Open, New file, Print, Next page, Undo, Redo, Find, Start show, Current slide, New slide, Duplicate slide, End show, Clock.
- **Excel:** Save, Open, New file, Print, Next page, Undo, Redo, Find, AutoSum, Format cells, Insert date, Edit cell, Filter, Clock.

Office defaults follow Microsoft's Windows/US English shortcut documentation for [Word](https://support.microsoft.com/en-us/accessibility/word/keyboard-shortcuts-in-word), [PowerPoint editing](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-create-powerpoint-presentations), [PowerPoint presenting](https://support.microsoft.com/en-us/accessibility/powerpoint/use-keyboard-shortcuts-to-deliver-powerpoint-presentations), and [Excel](https://support.microsoft.com/en-us/accessibility/excel/keyboard-shortcuts-in-excel). Labels and actions remain editable. Other language/keyboard layouts can differ; this change has not been tested against running Office applications or physical dock buttons.

The renderer preserves each icon's proportions, uses antialiased Bitstream Vera Bold labels, and fills every pixel of the button face without drawing a bezel. Each of the 13 action keys has an up and down state in every theme and page. Index 13 exports a blank placeholder; the firmware clock owns that display. The font is bundled under the license in `assets/fonts/bitstream-vera-license.txt`. The gradient sprites were exported offline and assembled into new atlases and the PDF catalogue; their physical display and ZIP packaging have not been checked. Earlier, on 7 October 2026, the owner observed the light-abstract Windows/media, Codex, and VS Code pages on the physical D200H: their previous icon treatment and labels were intact and readable, and NEXT PAGE worked. The timed display later cleared; resident mode restored it. The nine new styles and the Utility and Themes pages have not yet been physically verified.
