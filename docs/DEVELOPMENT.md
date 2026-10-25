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

The app is written in **Rust**. Rust turns source code into a fast, self-contained program. People using the app do not need Rust or Python installed.

**Slint** draws the settings windows and connects their controls to Rust. **Python** prepares pictures, the printable OBS guide and license notices. It is a development tool, not part of the running controller.

## Build on Windows

You need Git, Rust 1.90 or newer with its MSVC toolchain, Visual Studio C++ build tools, the Windows SDK and PowerShell 7. These tools download the source and turn it into an app.

```powershell
git clone https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware.git
Set-Location .\ulanzi-dock-vibeware
cargo build --locked --bins
```

This makes a Debug build in `target/debug/`. Add `--release` for an optimized build in `target/release/`. Neither command needs a dock. Build from the current source, not from an installed app or an older download.

Before introducing a new external library or binary, scan its downloaded archive and extracted files with Microsoft Windows Defender. Keep the source, version or SHA-256, command and clean result. Reuse that evidence for unchanged files. Do not add antivirus scans to builds or tests.

## One menu for the tools

From the repository folder, run:

```powershell
pwsh -NoProfile -File scripts/ulanzi.ps1
```

Choose a task or press Enter to leave. The menu explains which choices build, write files or install the app. It does not publish releases or run tests.

With a command, the menu is skipped:

```powershell
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command build -KeepBuildCache
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command msix -Configuration Debug -PublisherSubject 'CN=Your Local Publisher'
```

`build` keeps the local build script's existing Release behavior: it puts both programs in `artifacts/`, replaces them safely and removes its temporary build folder. `-KeepBuildCache` keeps that folder to speed up the next build. A running workspace copy must be closed first; `-StopRunning` explicitly allows the script to close it.

The existing `build-local.ps1`, `build-msix.ps1` and `package-release.ps1` commands keep their parameters. With no parameters they open the same menu. Use `-?` on `ulanzi.ps1` for help. See [installation](INSTALLATION.md#build-your-own-installer) for signing requirements.

Python 3.10 or newer handles the file tools through `scripts/tools.py`. It also offers a small menu without arguments. Pictures need Pillow; the OBS PDF also needs ReportLab. License collection uses Python's standard library and packages already restored in the Cargo cache. It does not download dependencies. These libraries are external imports and need the first-import scan described above before use.

The `build`, `msix` and `msix-icons` tasks use Windows tools. Other systems can build directly with Cargo; see [platform requirements](PLATFORMS.md).

## Find the right file

| File | What it does |
| --- | --- |
| `src/main.rs` | Reads terminal commands. |
| `src/controller.rs` and `src/dock.rs` | Connect to the dock, draw pages and receive button presses. |
| `src/model.rs` and `src/settings.rs` | Define pages and save preferences. |
| `src/actions.rs` and `src/applications.rs` | Send configured shortcuts and match applications. |
| `ui/dialogs.slint` and `src/gui.rs` | Define the windows and connect their buttons to Rust. |
| `src/paths.rs`, `src/startup.rs` and `src/tray.rs` | Find settings and provide Windows desktop integration. |
| `src/art.rs` | Draws button pictures, including pressed and unavailable states. |

`settings.json` stores pages, shortcuts and runtime preferences. `applications.yaml` maps apps to pages and can provide app commands. Keep preferences such as automatic switching in the JSON file. The app accepts the current formats; it does not convert older ones.

## Work with Codex

Open this repository as a local project in Codex, or run `codex` in its folder if you already use the CLI. Read `AGENTS.md` before changing anything. Tests, diagnostics and dock trials need my explicit permission for each run.

The dock's Codex page sends configured keyboard shortcuts. It does not install Codex or call an AI API.

Keep my name, original logo and [MIT notices](BRANDING.md). See [Windows editor ideas](design/README.md) for the planned WinUI 3 interface; it has not replaced Slint.
