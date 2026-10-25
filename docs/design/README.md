<!-- VBWR B
Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
VibeWare is Human intent, AI execution, and plenty of tokens
Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
VBWR E -->

# Ideas for a future Windows editor

These twelve AI-made pictures show an idea I want to build. The current settings windows use **Slint**, a toolkit that draws desktop controls. The proposed editor would use **WinUI 3**, Microsoft's Windows interface toolkit. It would be Windows-only; the Rust controller would stay portable.

[Open the gallery](winui3-concepts-v1/index.html) to compare six screens in dark and light mode. Click a picture to enlarge it. The gallery works offline.

## What the screens would do

| Screen | Intended use |
| --- | --- |
| Key editor | Choose a key, edit its label, action and picture, and preview both button states. |
| Pages | Add, copy, enable and arrange pages. |
| Applications | Match executable names to pages. |
| Themes | Choose a theme and adjust its colors. |
| Presets | Preview, import and export setups while preserving existing shortcuts. |
| Device | Show real connection state, brightness, startup and default-page settings. |

**Hue** changes colors. **Saturation** makes them softer or stronger. The plan is Hue from -180 to +180 and Saturation from 0% to 200%, with a before/after preview and Reset. These sliders do not exist in the current app. People could apply changes to one key, one page or all pages. App light/dark mode would be separate from dock colors.

## Keep the existing behavior

Keep the dock's five/five/three square-key layout and wide bottom display. NEXT PAGE stays on the fifth key. Themes stays last, and the wide display opens the page index. Follow the current built-in pages rather than treating the pictures' page lists as authoritative.

Use clear Windows controls, readable labels and violet, teal, amber and coral accents. Previewing a key must not run its action. Follow [branding](../BRANDING.md) for my name, logo, About and footer.

Build the editor separately from the controller. Reuse the existing picture renderer so previews match the dock. Imports, custom apps, macros and presets need real controller support before they can work. The OBS page currently sends shortcuts; live recording or mute status would need an OBS connection.

The pictures' clocks and Connected labels are illustrations. The [original image prompts](winui3-concepts-v1/prompts.md) and `manifest.json` record how the art was made; they are not current app instructions.
