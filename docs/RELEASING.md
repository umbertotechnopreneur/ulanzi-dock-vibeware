<!-- Copyright (c) 2026 Umberto Giacobbi | MIT License: see LICENSE -->

# Packages and releases

This guide is for preparing downloads. For everyday use, start with [installation](INSTALLATION.md).

## Package formats

| Platform | Download |
| --- | --- |
| Windows x64 and ARM64 | ZIP with the controller and Windows sign-in launcher. |
| Linux x64 | tar.gz for Ubuntu 24.04 or a compatible system. |
| macOS Apple Silicon | tar.gz built on macOS 15. No Intel package. |

See [platform coverage](PLATFORMS.md) for features and runtime limits. Windows also has a separate [local MSIX installer](INSTALLATION.md#build-your-own-installer); its script currently builds x64 only.

Archives include the README, guides, presets, artwork credits, MIT license, Slint and font licenses, and dependency notices. Python collects notices from already restored Cargo packages. Missing dependencies must be obtained and scanned before packaging; the notice tool does not download them.

## Make an archive

Build the current source for the intended target first. The packaging script reads `target/<target>/release/`; it does not rebuild or use an installed copy. For example, after an authorized Windows x64 target build:

```powershell
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command package -Target x86_64-pc-windows-msvc -Label windows-x64 -Tag v0.2.0-stable
```

Use the intended version and matching platform. Packages go in `dist/`. The script refuses to overwrite an archive or reuse its package folder. It also writes a SHA-256 checksum, which lets someone check that a download arrived unchanged. Unix packages use tar.gz to keep executable permissions.

## Publish only when requested

Publishing needs my explicit request. Source changes follow their normal review process. Documentation-only delivery follows `AGENTS.md`.

Public versions increase by minor number: **0.1.0, 0.2.0, 0.3.0**, and so on. Update `Cargo.toml` and its matching entry in `Cargo.lock`. The manual Release workflow checks the next version and uses `v<version>-stable` unless given an explicit matching tag. Existing tags and releases stay unchanged.

Builds on `main` and pull requests save downloadable build artifacts; they do not publish a release. For an authorized release, run the manual Release workflow with `draft=true` and `prerelease=false`. Review all four platform archives and checksums before publishing the draft.

A built package proves that packaging completed. Real-dock checks are recorded separately in [platform coverage](PLATFORMS.md).

The workflow normally runs hardware-free unit tests. When the owner explicitly
requests builds without tests, dispatch it with `run_tests=false` and record that
limit in the release notes. Do not describe skipped tests as passed.

## WinGet

`UmbertoGiacobbi.UlanziDockVibeWare` is a proposed package ID, not a confirmed listing. For an authorized submission, use the actual release links and checksums for the two Windows ZIPs. Set `InstallerType: zip`, `NestedInstallerType: portable`, executable `ulanzi-dock-vibeware.exe` and command alias `ulanzi-dock-vibeware`.

Keep the launcher and notices beside the executable. Validate manifests with `winget validate --manifest <manifest-directory>` before submitting them to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs). The install command becomes available only after acceptance and indexing. WinGet does not configure sign-in startup.
