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

# UlanziDock VibeWare version

**Creator:** Umberto Giacobbi · [VibeWare manifesto](https://umbertogiacobbi.biz/vibeware/manifesto)

**Protocol credit:** I built this controller using published D200H protocol research, especially the [OpenActionMirrors D200 project](https://github.com/OpenActionMirrors/com.glmagalhaes.ulanzi.d200). I did not copy its AGPL source code.

<img src="docs/photos/setup-triptych-banner.png" alt="Three enhanced views of my real desk and Ulanzi D200H controller in one panoramic banner" width="100%">

*Three views of my actual desk and dock, combined into an AI-enhanced banner.*

## Why I built it

I bought a Ulanzi D200H to put my own controls and PNG artwork on its buttons. I was frustrated that changing a few images seemed to require almost 800 MB of vendor software. I wanted something focused, so I built my own controller in Rust.

I am sharing my original implementation under the [MIT license](LICENSE). Anyone can use, modify, and share it under that license. I hope it helps someone else make this little console their own without installing a large application.

## My setup

I photographed the controller on my own desk. The banner combines three views; the two larger views below show more detail. They are AI-enhanced from the real photos, so I use the unaltered terminal screenshots below for exact software text and behavior.

<table>
  <tr>
    <td width="50%"><img src="docs/photos/dock-closeup-enhanced.png" alt="Enhanced close view of my real Ulanzi D200H and its illuminated theme buttons" width="100%"></td>
    <td width="50%"><img src="docs/photos/desk-enhanced.png" alt="Enhanced wider view of the Ulanzi D200H on my real computer desk" width="100%"></td>
  </tr>
</table>

## What it does

I use the operating system's HID access to display themed button artwork, listen to the dock's button events, and run the shortcuts I configure. The current source has six pages: **Windows / media, Codex, VS Code, Utility, Themes and Spotify**, with thirteen visual themes. [Take a look at the themes in the PDF guide](docs/ulanzi-dock-vibeware-guide-v2.pdf).

On Windows, I can let Codex, VS Code or Spotify bring its page forward automatically when that app becomes active. If I switch away, that page's app-specific controls become inactive until the app returns; **NEXT PAGE** still works. Windows / media and Utility remain general-purpose pages. I can also turn automatic page switching off.

It does not install a custom driver or require Ulanzi Studio, OpenDeck, or WSL. The local Windows executable I inspected is about **9 MB**; it runs without an installer. That size refers to the EXE, not a download package with documentation.

## Get started

The guided welcome below is included in the **v0.3.0 source**. Look for its Windows package on the [Releases page](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/releases); older packages may behave differently.

For a build with the guided welcome, put the executable in a writable folder, connect the D200H, and run:

```powershell
.\ulanzi-dock-vibeware.exe run
```

On the first run, I show two short screens: **Meet your dock** explains the buttons and pages; **Make yourself at home** shows the selected theme and `settings.json`. Choose **Start my dock** and leave the terminal open while using it. The setup saves your settings when you continue. You can change the theme on the dock, edit shortcuts in `settings.json` while the controller is stopped, or reopen the welcome with `run --oobe`.

<table>
  <tr>
    <th>Controller running</th>
    <th>First-run setup</th>
  </tr>
  <tr>
    <td width="50%"><img src="docs/screenshots/controller-running-0.2.0.png" alt="Unaltered Windows terminal screenshot showing the UlanziDock controller running" width="100%"></td>
    <td width="50%"><img src="docs/screenshots/first-run-setup.png" alt="Unaltered Windows terminal screenshot showing the second step of UlanziDock setup" width="100%"></td>
  </tr>
</table>

These are unaltered local Windows screenshots. The first is from an earlier **0.2.0** run, so its on-screen version and motto predate the current release; the second shows the guided setup. Your theme and configuration may differ. For commands and options, use `ulanzi-dock-vibeware help`.

## What I have checked

I have tested the D200H's button events, clock, page navigation, and a subset of artwork on **Windows x64 hardware**. I have also observed the optional wide status panel on that device. The Windows ARM64 build has not been physically tested on ARM64 hardware; Linux and macOS device behavior has not been physically validated. Offline artwork previews do not establish what the dock displays.

## Source and credits

I wrote this MIT implementation from observed protocol behavior. I used [OpenActionMirrors D200](https://github.com/OpenActionMirrors/com.glmagalhaes.ulanzi.d200) and [independent D200 protocol research](https://github.com/marcelobrake/ulanzi-linux/blob/main/docs/protocol.md) as references; I did not copy AGPL source code. Artwork and font provenance are in [ARTWORK.md](docs/ARTWORK.md). The alternate light [banner illustration](assets/banners/ulanzi-dock-light.png) is a concept, not a device photograph.

Copyright © 2026 Umberto Giacobbi. MIT licensed; see [LICENSE](LICENSE).

---

<a href="https://umbertogiacobbi.biz/vibeware/manifesto">
  <img align="right" src="https://raw.githubusercontent.com/umbertotechnopreneur/VibeWare/main/Branding/vibeware-logo.png" alt="VibeWare floppy logo" width="180">
</a>

### This is VibeWare

VibeWare is a term coined by [Umberto Giacobbi](https://umbertogiacobbi.biz) and an open initiative for developers who build with AI and care about the craft. It challenges the assumption that vibe coding is synonymous with low-quality code. It openly acknowledges the weaknesses and risks of AI-generated software: experienced developers must guide the process, test carefully, check security and take responsibility for the result. A VibeWare footer simply says: this software was developed with AI, and it deserves to be judged by the quality of the work.

[Read the VibeWare manifesto](https://umbertogiacobbi.biz/vibeware/manifesto)

*You are welcome to explore, adapt and reuse the open-source VibeWare materials under the project license — my contribution to the developer community.*
