<!-- VBWR B

Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz

VibeWare: Human intent, AI execution, and plenty of tokens
Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto

Modified with AI: OpenAI Codex; documentation and VibeWare attribution; 2026-10-27
Human guidance: Umberto Giacobbi; AGENTS.md; readability and VibeWare template request
Evidence: docs/CODE-REVIEW.md and current working-tree changes

Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE

VBWR E -->

# Install and set up UlanziDock

## Use the Windows installer

Download an MSIX from [my releases](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/releases), when available. MSIX is Windows' app package format.

1. Open the MSIX and choose **Install**.
2. Open **UlanziDock VibeWare** from Start.
3. Follow the first-run setup. Right-click the tray icon, then choose **About → Configure** to change your choices later.

The tray icon is the small app icon near the Windows clock. Starting the controller can update a connected dock. If startup fails, an error window explains why.

After the first launch, Windows lists the app in **Startup apps**. Turn it off there if you do not want it to start at sign-in. Terminal users can use `ulanzi-dock-vibeware.exe` without finding the WindowsApps folder.

A locally signed installer needs a trusted certificate from its maker. Check who made it before trusting that certificate.

## Run without installing

Extract the whole Windows ZIP into a writable folder. Open `ulanzi-dock-vibeware.exe`. Keep `ulanzi-dock-launcher.exe` beside it for startup at sign-in.

Linux and macOS use a terminal. See [platform instructions](PLATFORMS.md).

## Change and back up settings

On Windows, open **About → Configure**. Pick a theme, enable pages and choose whether the dock follows the app you are using. **Save + Restart** applies the changes.

The settings buttons open two text files:

- `settings.json`: pages, button shortcuts, brightness and automatic switching preferences.
- `applications.yaml`: application names, executable names and optional app commands.

The controller also keeps the last regular page in `settings.session.json`, beside `settings.json`. The selected theme stays in `settings.json`. On the next run, the boot picture fades out and that page and theme return. A removed or disabled page falls back to the normal home page, and the temporary page index is not saved. With a custom settings path such as `my-dock.json`, the session file is `my-dock.session.json`.

Office shortcuts start with US English defaults. Adjust them for your keyboard and application language.

The MSIX saves these files under `%LOCALAPPDATA%\Packages\<package-family>\LocalState\`. The portable app uses its executable folder. Local builds in `artifacts/` use the repository's settings. Files inside WindowsApps are not your editable settings.

Updates keep existing preferences. Back up the two settings files, the application catalogue and the session file before uninstalling or resetting. If a file uses an unsupported format, keep it as a reference and create a separate current file with `init --config new-settings.json`. Copy the shortcuts you want into that file; use the current application catalogue too.

### Add OBS Creator to existing settings

Fresh settings already contain OBS Creator. An update does not insert it into a settings file you already use. Close the app and follow [Add the page](../presets/obs-creator/README.md#add-the-page), using your actual settings path. The preset installer makes a backup.

With automatic switching on, OBS must be the app in front to select its page. Having it open in the background is not enough.

## Build your own installer

First follow [development setup](DEVELOPMENT.md). You also need Python 3.10 or newer and the Windows SDK tools `makeappx.exe` and `signtool.exe`.

The script uses your existing signing certificate. Its private key must be in `Cert:\CurrentUser\My`, its public certificate trusted in `Cert:\CurrentUser\Root`, and its expiry at least 30 days away. The script does not create or trust certificates.

```powershell
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command msix -Configuration Debug -PublisherSubject 'CN=Your Local Publisher'
```

It builds both programs from current source, creates fresh package settings, includes licenses and signs an x64 MSIX in `artifacts/msix/`. The version comes from `Cargo.toml`. `-Configuration Release` makes an optimized build; the original `build-msix.ps1` command still defaults to Release when given parameters.

Add `-Install` to install the package on this PC. This can close a running copy. Local signing does not publish the app to Microsoft Store.

See [platform coverage](PLATFORMS.md) for what has been checked on real hardware.
