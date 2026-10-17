<!-- VBWR B
Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
VibeWare is Human intent, AI execution, and plenty of tokens
Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
VBWR E -->

# What the Windows app should do

I want a WinUI 3 editor for Windows, with dark and light modes. The twelve pictures show the idea. The current app still uses Slint.

## Six screens

- **Key editor:** choose a key, change its name, action and picture, and preview it pressed or released.
- **Pages:** add, copy, enable and arrange pages.
- **Applications:** choose which app opens which page, using its executable name.
- **Themes:** pick a theme and change its colors.
- **Presets:** preview, import and export setups without losing existing shortcuts.
- **Device:** show the real connection state, brightness, startup and default page.

## Two color sliders

**Hue** changes the colors. **Saturation** changes their strength. Show a before/after preview and a Reset button.

Start with Hue from -180 to +180, and Saturation from 0% to 200%. Normal is Hue 0 and Saturation 100%. These are planned values, not settings supported today.

Let people change one key, one page or all pages. Keep labels readable. App light/dark mode must not change the dock's colors. Clicking a preview key must not run its action.

## Look and layout

Use normal Windows controls, clear text and enough space. Give buttons different colors: violet, teal, amber and coral, not all blue.

The dock has five square keys on the first row, five on the second, then three and a wide clock. Keep NEXT PAGE on the fifth key and the clock at the bottom right.

Keep the nine regular pages and thirteen themes. Themes stays last. NEXT PAGE skips the page index; pressing the wide display opens it.

Follow the [branding notes](../../BRANDING.md) for my name, logo, About and footer.

## Build it in small pieces

Keep the Windows editor separate from the portable Rust controller. Reuse the existing artwork code so the preview and dock match. Add a way for the editor to send settings to the controller and receive its status.

New image imports, custom apps, macros and presets need controller support too. They are not finished just because they appear in a picture.

The OBS preset sends hotkeys; live recording or mute status would need an OBS connection. The pictures' clocks and Connected labels are examples.

Keep the existing [MIT license](../../../LICENSE). See [building](../../DEVELOPMENT.md) and [installation](../../INSTALLATION.md).

[Gallery](index.html) · [Original image prompts](prompts.md) · [ZIP](../ulanzi-winui3-concepts-v1.zip)
