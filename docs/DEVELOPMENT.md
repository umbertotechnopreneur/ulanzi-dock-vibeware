<!-- VBWR B
Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
VibeWare is Human intent, AI execution, and plenty of tokens
Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
VBWR E -->

# Build and change UlanziDock

I made this as a weekend project. You can use and change the code under the [MIT License](../LICENSE).

## Build on Windows

Install Git, Rust 1.90 or newer with the MSVC toolchain, Visual Studio C++ build tools, Windows SDK and PowerShell 7.

```powershell
git clone https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware.git
Set-Location .\ulanzi-dock-vibeware
cargo build --release --bins
```

The programs are in `target/release/`. You do not need a dock to build them.

You can also use `pwsh -NoProfile -File scripts/build-local.ps1` to put the programs in `artifacts/`. See [installation](INSTALLATION.md) to make an MSIX.

## Work with Codex

Add this repository folder as a local project in Codex. Make it the primary folder and start a chat about your change. See the [official project guide](https://learn.chatgpt.com/docs/projects).

If you already use Codex CLI, open a terminal in the repository folder and run:

```powershell
codex
```

Read `AGENTS.md` first. Do not run tests or try the dock unless I ask for that specific check.

The dock's **Codex** page is different: it sends keyboard shortcuts to the desktop app. It does not install Codex or connect to the OpenAI API. Edit shortcuts in `settings.json` and app names in `applications.yaml`.

## Where the code is

- `main.rs`: reads commands.
- `controller.rs`: connects to the dock and handles its buttons.
- `settings.rs`: saves preferences, including finished setup.
- `paths.rs`: finds the settings folder for the app and launcher.
- `gui.rs`: opens windows and connects their buttons to these operations.

Preferences live in `settings.json`. Application shortcuts live in `applications.yaml`. Old formats are rejected, not converted.

## Future interface

The current dialogs use Slint. I am planning a Windows-only WinUI 3 editor; [the pictures and notes](design/README.md) show the idea. Hue/Saturation and other new editor controls still need to be built. Keep the Rust controller and command line usable on other platforms.

Keep my name, the original VibeWare logo and the [short branding notes](BRANDING.md). I have only tested the D200H; other model owners are welcome to try the code and [report what happens](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/issues).
