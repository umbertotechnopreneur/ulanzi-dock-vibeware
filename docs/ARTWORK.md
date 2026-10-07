<!-- VBWR B
Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
VibeWare is Human intent, AI, and plenty of tokens ;-)
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

Three ImageGen reference sheets remain in `assets/concepts/`, one for each page. They provide the 13 smooth pictogram silhouettes. Their card frames and lettering are excluded from the production sprites. The original ImageGen prompt requested a 7-row by 4-column comparison of released/up and pressed/down button concepts with English labels and clear spacing. Earlier pixel-art concepts were superseded and are not part of the distributable artwork.

The 12 current full-color atlases in `assets/production/` are assembled from the exact 196×196 PNGs in `assets/buttons/`. Each sheet covers one page and one theme. `docs/build_production_atlases.py` recreates them after `ulanzi-dock-vibeware render --output assets/buttons`.

Theme variations:

- `dark-classic`: Matrix-inspired deep green gradient, mint glow, bright active state.
- `dark-abstract`: Blade Runner-inspired violet/blue gradient, warm amber icons, neon active state.
- `light-classic`: ivory-to-blue gradient, navy icons and a soft shadow.
- `light-abstract`: ivory-to-mint gradient, teal icons and a soft shadow.

Page action lists (index order):

- **Windows / media:** Play / pause, Previous, Next track, Mute, Next page, Volume down, Volume up, File Explorer, Desktop, Settings, Screenshot, Task view, Browser, Clock.
- **Codex:** New chat, Projects, Attention, Side chat, Next page, Review, Terminal, Files, Commands, Copy link, Copy, Paste, Find, Clock.
- **VS Code:** Quick open, Commands, Terminal, Search, Next page, Explorer, Source control, Debug, Extensions, Save, New file, Format, Problems, Clock.

The renderer preserves each icon's proportions, uses antialiased Bitstream Vera Bold labels, and fills every pixel of the button face without drawing a bezel. Each of the 13 action keys has an up and down state in every theme and page. Index 13 exports a blank placeholder; the firmware clock owns that display. The font is bundled under the license in `assets/fonts/bitstream-vera-license.txt`. The current production sprites passed offline export and ZIP packaging. On 7 October 2026 the owner observed the final light-abstract Windows/media set on the physical D200H during a 20-second, no-actions run: all icons and labels were intact and readable. The set remained visible briefly after the process exited, then cleared. A resident no-actions run restored the icons and clock. Two NEXT PAGE presses displayed Codex and then VS Code, both with intact, readable artwork. The other three themes remain physically unverified.
