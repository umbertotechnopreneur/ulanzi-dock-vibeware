<!-- Copyright (c) 2026 Umberto Giacobbi | MIT License: see LICENSE -->

# Packages and releases

This guide is for preparing downloads. For everyday use, start with [installation](INSTALLATION.md).

## Package formats

| Platform | Download |
| --- | --- |
| Windows x64 and ARM64 | ZIP with the controller and Windows sign-in launcher. |
| Linux x64 | tar.gz and a Debian `.deb` installer, built on Ubuntu 24.04 in CI. |
| macOS Apple Silicon | tar.gz built on macOS 15. No Intel package. |

See [platform coverage](PLATFORMS.md) for features and runtime limits. Windows also has a separate [local MSIX installer](INSTALLATION.md#build-your-own-installer) for x64 and ARM64.

Archives include the README, guides, presets, artwork credits, MIT license, Slint and font licenses, and dependency notices. Python collects notices from already restored Cargo packages. Missing dependencies must be obtained and scanned before packaging; the notice tool does not download them.

## Make an archive

Build the current source for the intended target first. The packaging script reads `target/<target>/release/`; it does not rebuild or use an installed copy. For example, after an authorized Windows x64 target build:

```powershell
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command package -Target x86_64-pc-windows-msvc -Label windows-x64 -Tag v0.2.0-stable
```

Use the intended version and matching platform. Packages go in `dist/`. The script refuses to overwrite an archive or reuse its package folder. It also writes a SHA-256 checksum, which lets someone check that a download arrived unchanged. Unix packages use tar.gz to keep executable permissions.

## Make a Debian package

Use the shared console after building the native Linux release:

```sh
python3 scripts/ulanzi.py build
python3 scripts/ulanzi.py deb
```

The package and its checksum go to `~/.cache/ulanzideck/dist` by default
(or `$XDG_CACHE_HOME/ulanzideck/dist`). Build material remains on the selected
portable cache disk. The command never installs the package or starts the app.
Existing output names are preserved. Use `--output` to select another destination
and `--binary` to select an existing native executable:

```sh
python3 scripts/ulanzi.py deb --binary target/x86_64-unknown-linux-gnu/release/ulanzi-dock-vibeware --output dist
```

Debian packaging requires `dpkg-dev`, included in Linux setup. The package carries
the controller, configurator launcher, original icon, project license, artwork
credits, font licenses and locked dependency notices. Shared-library dependencies
come from `dpkg-shlibdeps`; dynamically loaded desktop libraries are listed too.
A local package targets the libraries on its build host. Building on Ubuntu 26.04
does not establish Ubuntu 24.04 compatibility. CI uses the Ubuntu 24.04 baseline.
The command also accepts native ARM64 hosts, but the CI Debian artifact is x64.
ARM64 Linux and other dock models remain unverified.

Install the requested package with APT:

```sh
sudo apt install /absolute/path/ulanzi-dock-vibeware_0.4.0_amd64.deb
```

Reconnect the D200H after installation so its restricted `uaccess` rule applies.
Only the active local desktop session receives access. The package does not grant
world-writable HID access or run the controller as root. Open UlanziDeck from the
application menu for settings, or run `ulanzideck` in a terminal for the controller.
Use `ulanzideck configure` to open settings directly. Keep that terminal open
while using the controller. Settings are stored in
`$XDG_CONFIG_HOME/ulanzideck` (normally `~/.config/ulanzideck`). An explicit
`--config` selects another profile. Removing the package leaves user settings.
The installed launcher records controller and configurator errors in
`$XDG_STATE_HOME/ulanzideck/logs` (normally `~/.local/state/ulanzideck/logs`),
including panic backtraces. Each launch gets its own private log file.
The configurator edits settings without opening the dock; `ulanzideck run`
starts the controller, waits for the D200H and reconnects after USB errors.

Build CI adds the `.deb` and checksum to its Linux artifact. The manual Release
workflow includes them alongside the four platform archives. No release is
published merely by creating a local package or running Build CI.

## Publish only when requested

Publishing needs my explicit request. Source changes follow their normal review process. Documentation-only delivery follows `AGENTS.md`.

Public versions increase by minor number: **0.1.0, 0.2.0, 0.3.0**, and so on. Update `Cargo.toml` and its matching entry in `Cargo.lock`. The manual Release workflow checks the next version and uses `v<version>-stable` unless given an explicit matching tag. Existing tags and releases stay unchanged.

Builds on `main` and pull requests save downloadable build artifacts; they do not publish a release. For an authorized release, run the manual Release workflow with `draft=true` and `prerelease=false`. Review all four platform archives, the Debian package and their checksums before publishing the draft.

See [platform coverage](PLATFORMS.md) for supported features and device limitations.

The workflow normally runs hardware-free unit tests. When the owner explicitly
requests builds without tests, dispatch it with `run_tests=false` and record that
limit in the release notes. Do not describe skipped tests as passed.

## WinGet

`UmbertoGiacobbi.UlanziDeck` is a proposed package ID, not a confirmed listing. For an authorized submission, use the actual release links and checksums for the two Windows ZIPs. Set `InstallerType: zip`, `NestedInstallerType: portable`, executable `ulanzi-dock-vibeware.exe` and command alias `ulanzi-dock-vibeware`.

Keep the launcher and notices beside the executable. Validate manifests with `winget validate --manifest <manifest-directory>` before submitting them to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs). The install command becomes available only after acceptance and indexing. WinGet does not configure sign-in startup.
