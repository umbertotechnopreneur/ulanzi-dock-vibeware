<!-- VBWR B
Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
VibeWare is Human intent, AI execution, and plenty of tokens
Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
VBWR E -->

# Install on Windows

## Use the installer

1. Double-click the MSIX and choose **Install**.
2. Open **UlanziDock VibeWare** from Start.
3. Right-click its tray icon and choose **About → Configure**.

Opening the app can send commands to a connected dock.

If startup fails, the launcher and controller show a Windows error dialog with the cause, even when no terminal is open.

After the first launch, the package registers **UlanziDock VibeWare** in Windows
Startup apps. Windows controls that setting; turn it off there if you do not want
the hidden tray controller at sign-in. In a terminal, the MSIX also provides the
`ulanzi-dock-vibeware.exe` app-execution alias, so scripts never need the changing
`WindowsApps` installation path.

Use packages from [my releases](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/releases), when available, or build your own below. A package built on your PC is not a Microsoft Store release.

If Windows rejects a local package's certificate, check who made it before trusting it. Do not disable Windows security to install it.

## Run without installing

Download a ZIP from my releases and extract the whole folder somewhere you can write files. Open `ulanzi-dock-vibeware.exe`. Keep `ulanzi-dock-launcher.exe` beside it if you want startup at sign-in.

For Linux and macOS, see [package notes](RELEASING.md).

## Build your own MSIX

First follow [the build guide](DEVELOPMENT.md). You also need Python 3, PowerShell 7 and the Windows SDK tools `makeappx.exe` and `signtool.exe`.

You need your own code-signing certificate. It must have a private key in `Cert:\CurrentUser\My`, be trusted in `Cert:\CurrentUser\Root` and remain valid for at least 30 days. The script does not create or trust certificates.

Run this from the repository folder, using your certificate's exact subject:

```powershell
pwsh -NoProfile -File .\scripts\build-msix.ps1 -PublisherSubject 'CN=Your Local Publisher'
```

The signed x64 package is saved in `artifacts/msix/`, with its license, notices and icon sizes. It gets fresh settings, not your personal file. Its version comes from `Cargo.toml`.

For a local Debug build, add `-Configuration Debug`. The script builds both programs from the current source with the locked dependencies. The default configuration is Release.

Add `-Install` to install it too. This can close a running copy of the app.

## Settings and updates

The MSIX keeps settings in `%LOCALAPPDATA%\Packages\<package-family>\LocalState\`. Do not edit files in `WindowsApps`. Back up your settings before uninstalling or resetting.

Old settings are not converted. Create a current file with `init --config new-settings.json` and copy the shortcuts you want to keep. Use the current `applications.yaml`; app preferences now belong in the JSON file.

### OBS Creator is missing

Updating the app keeps your existing settings. It does not add the OBS Creator page to them. The installed MSIX uses its own `LocalState/settings.json`; editing the repository's settings does not change that file.

Close UlanziDock and follow [Add the page](../presets/obs-creator/README.md#add-the-page), passing the installed app's settings path. Keep app detection and automatic switching on if you want the dock to switch when OBS Studio is in front. An OBS window open in the background does not select its page.

I have tested only my D200H on Windows x64. Building or installing the app does not prove that every feature works on a dock. The WinUI 3 pictures show a [planned interface](design/README.md), not the current app.
