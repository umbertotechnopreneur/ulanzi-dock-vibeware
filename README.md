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

# UlanziDock VibeWare version

[![Build status](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/actions/workflows/build.yml/badge.svg?branch=main)](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/actions/workflows/build.yml)
![Platform: Windows, Linux and macOS](https://img.shields.io/badge/platform-Windows_%C2%B7_Linux_%C2%B7_macOS-0078D4)
[![License: MIT](https://img.shields.io/badge/license-MIT-2EA44F)](LICENSE)

Your shortcuts, music and favourite apps, one button away.

## Quick-start PDFs

- **[Quick Start PDF](docs/ulanzi-dock-vibeware-guide-v2.pdf)** — the visual guide to UlanziDock.
- **[OBS Creator Quick Start PDF](docs/obs-creator-quick-start.pdf)** — set up the OBS page and its shortcuts.

## Why I built it

I am [Umberto Giacobbi](https://umbertogiacobbi.biz). I made this as a weekend project for my own desk.

I bought a **Ulanzi D200H** and wanted to change a few button pictures without running a huge app. So I built a small controller in Rust, a language that compiles into a fast program without a browser runtime. Pick a theme, set your shortcuts and get on with your day.

This is an independent, unofficial project. I do not work for Ulanzi, and I am not affiliated with or sponsored by Ulanzi. Ulanzi does not endorse this project. All trademarks and registered trademarks belong to their respective owners.

<img src="docs/photos/setup-triptych-banner.png" alt="My desk and Ulanzi D200H" width="100%">

*These desk photos were enhanced with AI.*

## What it does

- Ten pages: Windows / media, Utility, Codex, VS Code, Spotify, Word, PowerPoint, Excel, OBS Creator and Themes.
- Thirteen themes, with different colors and button styles.
- On Windows, optional page switching to follow the app you are using.
- Optional dimming of buttons when their app is not active. Turn pages off without losing their shortcuts.
- A clock or a large page-and-theme status display.
- No Node.js or browser runtime. No vendor app or custom driver required.

The code is free to use, change and share under the [MIT License](LICENSE).

<table>
  <tr><th>Windows / media</th><th>Spotify</th></tr>
  <tr>
    <td width="50%"><img src="assets/production/windows-dark-classic.png" alt="Windows and media button artwork, released and pressed" width="100%"></td>
    <td width="50%"><img src="assets/production/spotify-dark-classic.png" alt="Spotify button artwork, released and pressed" width="100%"></td>
  </tr>
</table>

*Button artwork in the MATRIX theme.*

## Get it running

Download the right package from [Releases](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/releases). Extract the whole archive into a writable folder. On Windows, open `ulanzi-dock-vibeware.exe`. Keep the launcher beside it for startup at sign-in.

Linux and macOS Apple Silicon use `./ulanzi-dock-vibeware run`. Features differ by system; see [Windows, Linux and macOS](docs/PLATFORMS.md).

Want an installer and a Start-menu entry? See [installation and settings](docs/INSTALLATION.md). Want to build or change the app? See [development and the tools menu](docs/DEVELOPMENT.md). [All guides](docs/README.md) lists the other topics.

## Change your setup

On Windows, right-click the tray icon near the Windows clock and choose **About → Configure**. Pick a theme, enable pages and choose whether the dock follows the active app. Click **Save + Restart**.

The settings buttons open the files for shortcuts and app assignments. See [settings and backups](docs/INSTALLATION.md#change-and-back-up-settings) for their locations and what to edit.

To show the page and theme instead of the clock:

```powershell
.\ulanzi-dock-vibeware.exe run --status-display
```

Run `ulanzi-dock-vibeware help` for more commands.

### Brightness

Use **BRIGHT +** and **BRIGHT -** on the Utility page, or save a level from 0 to 100:

```powershell
.\ulanzi-dock-vibeware.exe brightness 50
```

The controller picks up the saved level while running. If it is closed, the level is used next time. The default is 100%. If you run with a custom settings file, pass the same `--config` path to this command.

At the minimum or maximum, the matching button shows UNAVAILABLE.

### Jump to a page

At the first dock connection, the UlanziDock and VibeWare picture appears immediately and stays fully visible for at least two seconds. It then fades to black over one more second, and the last enabled page and saved theme return. The artwork compensates for the two physical gaps between the three display rows. If the saved page was removed or disabled, the normal home page opens instead. The page index itself is never remembered.

On page changes, buttons fade in from left to right, top to bottom. The animation targets 100 ms in total; USB and device processing can add time. Button pictures are kept in a bounded memory cache for reuse, and overdue animation updates are combined. The wide panel returns after the buttons. Presses during a transition are discarded.

When the app closes normally, the current dock screen fades to black over one second. This does not change the saved brightness. A forced termination, crash or USB disconnection cannot guarantee the closing animation.

Press the wide bottom display to open the page index, then choose a page. Disabled pages cannot be selected. Automatic switching pauses while this menu is open. NEXT PAGE still cycles through the regular pages.

### OBS recording page

New settings include OBS Creator before Themes. The [OBS Creator guide](presets/obs-creator/README.md) explains how to add it to existing settings and how to configure the matching OBS hotkeys. Its recording, scene and audio buttons use all thirteen themes. There is also a [quick-start PDF](docs/obs-creator-quick-start.pdf).

<img src="presets/obs-creator/preview.png" alt="OBS Creator button layout in the Blade Runner theme" width="700">

*Artwork preview, not a dock photo. Buttons do not show live OBS status.*

## A future Windows editor

I am planning a WinUI 3 app for editing keys, pages, apps, themes, presets and device settings. It includes simple Hue and Saturation sliders.

[See the twelve dark/light mockups](docs/design/README.md). They are pictures of an idea, not a working interface. The current dialogs use Slint.

## Which hardware works?

> [!IMPORTANT]
> **Windows on ARM testers wanted — volunteer guinea pigs welcome! 🐹**
>
> This is my hobby project, built for my own **Ulanzi D200H on Windows x64**. If you can try the **Windows ARM64**, **Linux x64** or **macOS Apple Silicon** build, I'd love your help checking USB connection, button images and presses, page switching, brightness and shortcuts. I'll give you all the support I can.
>
> Own another Ulanzi model? You're welcome too. The controller currently detects the D200H specifically, so other models may need code changes before they can connect. You can [build the source](docs/DEVELOPMENT.md) and help me find out what needs changing.
>
> [Open an issue](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/issues) and tell me your dock model, operating system, CPU architecture, and what worked or failed.

## Credits

Protocol references: [OpenActionMirrors D200](https://github.com/OpenActionMirrors/com.glmagalhaes.ulanzi.d200) and [independent D200 research](https://github.com/marcelobrake/ulanzi-linux/blob/main/docs/protocol.md). I wrote this as an independent MIT project. See [artwork notes](docs/ARTWORK.md) and [third-party licenses](docs/THIRD-PARTY.md).

Copyright © 2026 Umberto Giacobbi. [MIT License](LICENSE).

## A note on security

Before adding an external library or binary, I require a targeted Microsoft Windows Defender scan of the downloaded archive and extracted files. New imports stay aside until the scan finishes without detections; I record the source and result. See [development](docs/DEVELOPMENT.md) for the first-import rule.

---

### This is VibeWare · [Manifesto](https://umbertogiacobbi.biz/vibeware/manifesto)

I built UlanziDock with AI help. I guide the work and am responsible for the result.

I started VibeWare for people making useful software with AI. Want to know more? [Read my manifesto](https://umbertogiacobbi.biz/vibeware/manifesto).

[UlanziDock on GitHub](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware) · [VibeWare on GitHub](https://github.com/umbertotechnopreneur/VibeWare) · [umbertogiacobbi.biz](https://umbertogiacobbi.biz)

[Terms](https://umbertogiacobbi.biz/terms/) · [Privacy](https://umbertogiacobbi.biz/privacy/)

<p align="right">
  <a href="https://umbertogiacobbi.biz/vibeware/manifesto">
    <img src="assets/brand/vibeware-logo.png" alt="VibeWare floppy logo" width="200">
  </a><br>
  <em>Human Intent, AI Execution, Plenty of Token</em>
</p>
