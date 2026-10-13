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

Your shortcuts, music and favourite apps, one button away.

## Why I built it

I built this because I can't stand twenty Node.js processes running just to change something once a week. I wanted to choose the pictures, tell each button what to do, and get on with my day.

I bought a **Ulanzi D200H**, then discovered that changing a few tiny pictures seemed to need almost **800 MB** of vendor software. That is a rather large moving truck for a handful of buttons. So I built a small controller in Rust.

I am [Umberto Giacobbi](https://umbertogiacobbi.biz), and this is my MIT-licensed controller, made as part of VibeWare. Bring your dock, pick a theme, and make the buttons yours.

<img src="docs/photos/setup-triptych-banner.png" alt="Three views of my desk and Ulanzi D200H controller" width="100%">

*These desk photos were enhanced with AI.*

## What you get

- **Nine pages:** Windows / media, Utility, Codex, VS Code, Spotify, Microsoft Office (Word, PowerPoint and Excel), and Themes. Windows and Utility cover everyday shortcuts, media controls and shell tools.
- **Automatic app switching on Windows:** enable application detection and automatic switching, and the dock follows the app you're using. Recognised apps get their own page; other apps return you to Windows / media.
- **Controls that know when they're useful:** with detection on, app-specific buttons are dimmed and inactive when their app isn't in focus. You can also turn individual application pages on or off without losing your shortcuts.
- **13 themes:** choose one on the dock or in the configuration window.
- **A big status display:** optionally replace the clock with the current application page, theme and active/inactive status.
- **A lightweight native Rust app:** designed to keep memory and CPU use low, with no Node.js or browser runtime. No vendor software or custom driver required.
- **Free and open source:** use it, change it, share it, or build your own version under the MIT license. Just keep the license notice.

<table>
  <tr><th>Windows / media</th><th>Spotify</th></tr>
  <tr>
    <td width="50%"><img src="assets/production/windows-dark-classic.png" alt="Windows and media button artwork, released and pressed" width="100%"></td>
    <td width="50%"><img src="assets/production/spotify-dark-classic.png" alt="Spotify button artwork, released and pressed" width="100%"></td>
  </tr>
</table>

*Button artwork previews in the MATRIX theme.*

## Download

Look for **Releases** on the right and choose the package for your computer. Extract the whole archive into a writable folder.

On Windows, double-click `ulanzi-dock-vibeware.exe` and follow the welcome screens. Keep `ulanzi-dock-launcher.exe` beside it if you want automatic startup at sign-in. Linux and macOS packages run from a terminal with `./ulanzi-dock-vibeware run`.

## Make it yours

On Windows, right-click the tray icon, choose **About**, then **Configure**. Pick a theme, enable the application pages you want, and choose whether the dock should follow the active app. **Save + Restart** applies your changes.

Edit shortcuts through the `settings.json` and `applications.yaml` buttons. Office shortcuts default to a US English keyboard layout; adjust them for your language if needed.

For the large status display, run:

```powershell
.\ulanzi-dock-vibeware.exe run --status-display
```

For more commands, run `ulanzi-dock-vibeware help`. To build from source on Windows, use `pwsh -NoProfile -File scripts/build-local.ps1`.

## Compatibility and credits

Button events, clock, navigation and the status display have been checked on Windows x64 hardware. Office pages and automatic switching still need a physical dock check. The tray and app detection are Windows features; ARM64, Linux and macOS device behavior has not been physically verified. See [package notes](docs/RELEASING.md) for platform requirements.

Protocol references: [OpenActionMirrors D200](https://github.com/OpenActionMirrors/com.glmagalhaes.ulanzi.d200) and [independent D200 research](https://github.com/marcelobrake/ulanzi-linux/blob/main/docs/protocol.md). This is an independent MIT implementation. See [artwork credits](docs/ARTWORK.md) and [third-party notices](docs/THIRD-PARTY.md).

Copyright © 2026 Umberto Giacobbi. MIT licensed; see [LICENSE](LICENSE).

---

<p align="right">
  <a href="https://umbertogiacobbi.biz/vibeware/manifesto">
    <img src="https://raw.githubusercontent.com/umbertotechnopreneur/VibeWare/main/Branding/vibeware-logo.png" alt="VibeWare floppy logo" width="160">
  </a><br>
  <strong>VibeWare</strong><br>
  <em>Human intent, AI execution, and plenty of tokens</em>
</p>

### What is VibeWare?

I coined the term [VibeWare](https://umbertogiacobbi.biz) and started an open initiative for developers who build with AI and care about the craft. I challenge the assumption that vibe coding is synonymous with low-quality code, while openly acknowledging the weaknesses and risks of AI-generated software: experienced developers must guide the process, test carefully, check security and take responsibility for the result. With this footer, I acknowledge that I developed this software with AI and ask you to judge the quality of the work.

[**Read the VibeWare manifesto**](https://umbertogiacobbi.biz/vibeware/manifesto)

*You are welcome to explore, adapt and reuse the open-source VibeWare materials under the project license — my contribution to the developer community.*
