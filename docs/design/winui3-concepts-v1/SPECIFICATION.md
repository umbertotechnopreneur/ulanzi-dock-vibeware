<!-- VBWR B
Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
VibeWare is Human intent, AI execution, and plenty of tokens
Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
VBWR E -->

# WinUI 3 design and implementation brief

Status: visual concept baseline requested on 16 October 2026. I plan a native Windows-only editor over the existing portable Rust controller/CLI. The twelve PNGs describe the proposed product; they do not implement its controls or prove device state.

## Six workspaces

| Workspace | Intended behavior |
| --- | --- |
| Key editor | Select a key on the dock preview; edit its label, shortcut, supported media action, URL and icon; compare released/pressed artwork. |
| Pages | Create, duplicate, enable and organize pages with a preview of the selected layout. Preserve reserved navigation and clock positions. |
| Applications | Associate executable names with pages; configure availability detection and automatic page switching. |
| Themes | Browse the existing theme families, preview a full layout, adjust Hue/Saturation and reset colors. |
| Presets | Preview, import and export configuration presets, and add their pages while preserving existing settings. |
| Device | Show actual connection state and expose brightness, sign-in startup, background operation and default-page controls. |

## Hue and Saturation

The owner requested simple slider-based color customization. The concepts expose one Hue slider and one Saturation slider, alongside a before/after preview and Reset colors.

- Proposed Hue range: -180 to +180 degrees, default 0.
- Proposed Saturation range: 0 to 200 percent, default 100 percent.
- Scope: the selected key, the selected page, or all pages.
- Reset restores the base theme colors for the chosen scope.
- App appearance (System / Light / Dark) is independent of the dock's artwork and color adjustments.
- Preserve label contrast and reserved-control visibility. Derive inactive artwork through the existing grayscale/unavailable treatment rather than creating another set of disabled PNGs.
- Slider previews must render locally. Selecting a key in the editor must not execute its shortcut.

The ranges above are proposed implementation defaults, not existing configuration properties. Configuration inheritance, serialization and live application to the controller still need implementation decisions.

## Visual direction

Windows 11 Fluent / WinUI 3: restrained Mica surfaces, Segoe UI Variable, clear hierarchy and dark/light counterparts. Use expressive colors for keys and thumbnails. Primary actions use violet; teal and amber provide meaningful secondary accents. Keep the VibeWare identity and original artwork bytes intact.

The physical preview has five square keys in row one, five in row two, and three in row three followed by one double-width clock/status area: thirteen ordinary keys and one wide display. NEXT PAGE is physical index 4 (the fifth key); the clock/status position is index 13. Preserve the existing nine regular pages, generated index and thirteen themes. Themes remains the final regular page; NEXT PAGE skips the index.

## Architecture and evidence boundaries

- Build the Windows editor separately from the portable controller. The current Rust GUI uses Slint; these concepts do not migrate it to WinUI 3.
- Reuse the actual Rust artwork renderer for previews and device output, avoiding a second implementation of theme/color behavior.
- Add a defined communication channel for editor commands, controller status and configuration changes; do not infer live application support from the mockups.
- Custom PNG import, arbitrary executable launch, multi-step macros and new presets require explicit engine/configuration support where absent today.
- The OBS preset uses configured keyboard shortcuts. Recording, scene and mute feedback would require a separate OBS integration. The clock is not an OBS recording indicator.
- Connected badges, clock values, preset thumbnails and extra page examples in the PNGs are illustrative. They are not USB, firmware, provider or Store validation.
- Build, installer, Store publication and owner-authorized interactive/device checks remain separate delivery steps.

## Deliverables

The twelve final PNGs, [gallery](index.html), [prompts](prompts.md), [portable manifest](manifest.json) and this brief are versioned together. The archive at [../ulanzi-winui3-concepts-v1.zip](../ulanzi-winui3-concepts-v1.zip) contains the complete concept folder. Original generation-cache paths are not required to use these artifacts.
