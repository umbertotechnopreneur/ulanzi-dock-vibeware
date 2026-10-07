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

# UlanziDock VibeWare version

**Creator:** Umberto Giacobbi · [VibeWare manifesto](https://umbertogiacobbi.biz/vibeware/manifesto)

A standalone, resident CLI controller for the Ulanzi D200H, **hardware tested on Windows x64**. It displays themed button artwork, reads the dock's own button events, and dispatches configured shortcuts or media commands on release. It uses the operating system's HID access and does not require a custom kernel driver, Ulanzi Studio, OpenDeck, or WSL.

The source is designed for Windows, Linux, and macOS. Binary distribution currently targets Windows x64 and Windows ARM64. The controller must keep running while the dock is in use.

## Tested on Windows

Physical D200H testing on **Windows x64**, most recently on **7 October 2026**, confirmed:

- Device button press/release events and the firmware clock.
- Complete, readable light-abstract artwork on the Windows/media, Codex, and VS Code pages.
- NEXT PAGE navigation between those pages.
- The other keys remain visually stable during the observed single-key press/release trial after the redraw fix.

The current implementation passes all six software tests. Native Windows x64 and ARM64 release builds have been produced. **ARM64 hardware operation has not been tested.** Linux and macOS device access and shortcut behavior have not been physically validated.

All five pages and thirteen themes have passed offline artwork generation and ZIP packaging. Physical checks cover a subset of those combinations; end-to-end verification of every configured shortcut and media action is still incomplete.

## Get started on Windows

1. Download the artifact for your architecture from GitHub Actions or the matching release package. Extract it into a writable directory. The EXE contains the artwork, so no external image files are required at runtime.
2. Open PowerShell in that directory and run `./ulanzi-dock-vibeware.exe doctor`. It only checks whether the D200H consumer interface is present.
3. Run `./ulanzi-dock-vibeware.exe init` to create `settings.json`.
4. Run `./ulanzi-dock-vibeware.exe run`. Leave this console open while using the dock; press Ctrl+C to stop.

The current source sends a complete layout on startup and when the page or theme changes. Ordinary button presses and releases update only that key's artwork, keeping the other keys stable. This improvement is included in commits from `6cf8e2d` onward; the published v0.2.0 release still sends complete layouts for button state changes. The wide bottom display is reserved for the firmware clock, updated about once per second by default. Use `--no-clock` only for a display diagnostic. Display content may disappear when the program exits. Only one controller should operate this device at a time.

For a bounded display-only diagnostic, use `./ulanzi-dock-vibeware.exe run --seconds 20 --no-actions`. It sends the layout and reads only dock events, but does not dispatch shortcuts or media keys. `--seconds` accepts 1-300.

## Pages and navigation

The default pages loop in this order: **Windows / media → Codex → VS Code → Utility → Themes → Windows / media**. Physical key **4**, at the top right, is **NEXT PAGE** on every page. The wide bottom position, index **13**, displays only the clock and dispatches no action.

```text
top:      00  01  02  03  04 NEXT PAGE
middle:   05  06  07  08  09
bottom:   10  11  12  [ 13 CLOCK ]
```

The 13 action keys have separate **up** and **down** PNG art in thirteen themes. The 65 production atlases are in `assets/production/`, one per page and theme, and the [68-page PDF catalogue](docs/ulanzi-dock-vibeware-guide-v2.pdf) shows the full set. The executable extracts complete pictograms from three ImageGen reference sheets and adds native Utility symbols and theme swatches. It draws smooth gradients, decorative motifs, and readable labels on a full-bleed 196×196 face, and sends no graphic for the reserved clock position. `render` exports 1,820 individual PNGs, including blank clock placeholders, without touching the device:

```powershell
./ulanzi-dock-vibeware.exe render --output artwork
```

### Suggested actions

| Position | Windows / media | Codex | VS Code |
| --- | --- | --- | --- |
| 00 | Play / pause | New chat | Quick open |
| 01 | Previous track | Projects | Commands |
| 02 | Next track | Attention | Terminal |
| 03 | Mute | Side chat | Search |
| 04 | Next page | Next page | Next page |
| 05 | Volume down | Review | Explorer |
| 06 | Volume up | Terminal | Source control |
| 07 | File Explorer | Files | Debug |
| 08 | Desktop | Commands | Extensions |
| 09 | Settings | Copy link | Save |
| 10 | Screenshot | Copy | New file |
| 11 | Task view | Paste | Format |
| 12 | Browser | Find | Problems |
| 13 | Clock | Clock | Clock |

Codex and VS Code shortcuts act on the **focused application**. A shortcut can change between application versions and OSes. Review the generated `settings.json` and customize it to your actual apps and keyboard layout.

Utility provides Clipboard, Search, Emoji, Screenshot, Explorer, Desktop, Task View, Settings, the project website, Copy, Paste, and Undo. These are editable Windows-oriented defaults. The fifth page is the Themes selector. Its twelve available keys show every theme except the one currently active; choosing a theme immediately returns to Windows/media in the new appearance and saves it to `settings.json`. NEXT PAGE and the clock retain their fixed positions. An existing three-page settings file is upgraded in memory when loaded; choosing a theme writes the five-page version and saves the previous file as `settings.json.bak`.

## Change theme and actions

Run `./ulanzi-dock-vibeware.exe themes` for all thirteen IDs. Select a theme on page five or set the `theme` field in `settings.json` and restart `run`. `--theme` overrides the saved startup choice for that launch:

The original themes are `dark-classic` (Matrix-inspired green), `dark-abstract` (Blade Runner-inspired violet and amber), `light-classic` (ivory/blue), and `light-abstract` (mint/teal). The nine additions are `manga-ink`, `steampunk-brass`, `cyberpunk-neon`, `moire`, `cubism`, `art-deco`, `ukiyo-e`, `solarpunk`, and `memphis`. Every theme has its own palette and motif, plus a stronger down state. None of the production sprites draws a button bezel.

```powershell
./ulanzi-dock-vibeware.exe run --theme light-abstract
```

Edit the first four pages' action key `label` and `action` fields in `settings.json`. Supported actions are `hotkey:ctrl+shift+p`, `media:play-pause`, `media:previous`, `media:next`, `media:mute`, `media:volume-down`, `media:volume-up`, and `open:https://example.com`. The final hotkey key can be a single character, `grave`, `tab`, `enter`, or `escape`. `ctrl`, `alt`, `shift`, and `super` are modifiers. Key 4 remains `page:next`; key 13 remains `none` for the clock. The Themes page is generated from the current selection. Changed labels use the built-in text renderer so the art matches the new label.

`init` refuses to replace an existing settings file. Keep a backup before editing yours. No shell commands or arbitrary executable paths are accepted in settings.

## Build from source

Rust 1.85 or newer is required. On Windows, install the corresponding MSVC toolchain and run:

```powershell
cargo build --release --target x86_64-pc-windows-msvc
```

For Windows ARM64 use target `aarch64-pc-windows-msvc`, preferably on an ARM64 Windows runner. GitHub Actions builds both targets on native runners and uploads an EXE plus README and LICENSE for each architecture. Linux and macOS compilation is part of source portability work; device access and shortcut behavior there are not yet physically validated.

## Protocol and provenance

This is an original MIT implementation informed by observed D200H protocol behavior. The separate [OpenActionMirrors D200 project](https://github.com/OpenActionMirrors/com.glmagalhaes.ulanzi.d200) is AGPL-3.0 and was used as a protocol reference; its source code is not copied into this repository. The dock has VID `2207`, PID `0019`; the consumer interface uses usage page `0x0c`, usage `1`. On Windows, an output report consists of Report ID `0` plus 1024 protocol bytes. Full layouts use command `0x0001`, individual key artwork updates use `0x000D`, and clock data uses `0x0006`. The additive update command is also documented in the independent [D200 protocol research](https://github.com/marcelobrake/ulanzi-linux/blob/main/docs/protocol.md). Input is parsed only from this interface. A successful HID write is not a firmware acknowledgment or proof of visible rendering.

The approved VibeWare logo is copied unchanged into `assets/brand/`. The production atlases are offline previews, not photos or proof of hardware output. The ImageGen source provenance and font license are documented in `docs/ARTWORK.md`.

Copyright © 2026 Umberto Giacobbi. MIT licensed; see [LICENSE](LICENSE).
