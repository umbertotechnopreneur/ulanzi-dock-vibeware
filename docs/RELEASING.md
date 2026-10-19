# Make a release

Only publish a release when I ask. Use a pull request for source changes, then let me review the release draft. Do not overwrite existing releases or move tags, including `v0.1.0-stable` and the earlier preview tags.

## Version numbers

I number public releases by increasing the minor version: **0.1.0, 0.2.0, 0.3.0**, and so on. Before each release, update the package version in `Cargo.toml` and its matching entry in `Cargo.lock`.

The manual Release workflow checks that this is the next minor version after the latest published release. It uses `v<version>-stable` as the tag unless an explicit matching tag is supplied. Titles and archive names use the same package version. Earlier preview tags stay unchanged.

Pushing source changes does not publish a release or create a tag. Builds on `main` and pull requests save packages as GitHub Actions artifacts.

## Packages

The Build and manual Release workflows target:

- Windows x64 and ARM64: ZIP, with the controller and sign-in launcher.
- Linux x64: tar.gz, for Ubuntu 24.04 or a compatible glibc system.
- macOS Intel and Apple Silicon: tar.gz, for macOS 15 or later.

Include the README, MIT license, artwork credits, Slint and font licenses, and dependency notices. The bundled older PDF is not a guide to the newer Office pages.

Linux needs udev and the desktop libraries listed in the build workflow, plus permission to use the HID device. Do not assume it must run as root. macOS packages are terminal programs, not signed or notarized apps; shortcut injection may need permission.

The tray, app detection and sign-in setup are Windows features. I have tested only my D200H on Windows x64. A successful build is not a dock test.

## Windows installer

See [MSIX installation and building](INSTALLATION.md). Use **Umberto Giacobbi** as the visible publisher and include the licenses. Local signing is not Microsoft Store approval. See [branding](BRANDING.md) for the logo and credits.

## Create the draft

Run the manual Release workflow with `draft=true` and `prerelease=false`. Leave the tag empty to use the package version automatically. It needs all five archives and their checksums. Publish only after my review.

To package an already built Windows x64 target locally:

```powershell
pwsh -NoProfile -File scripts/package-release.ps1 -Target x86_64-pc-windows-msvc -Label windows-x64 -Tag v0.2.0-stable
```

Use the intended tag. The executable must be in `target/<target>/release/`. The script will not overwrite an archive or reuse its staging folder. Building and packaging do not run the app or test the dock.

## WinGet

The proposed package ID is `UmbertoGiacobbi.UlanziDockVibeWare`. This is not a claim that it is listed.

For the two Windows ZIPs, use:

- `InstallerType: zip`
- `NestedInstallerType: portable`
- Nested executable: `ulanzi-dock-vibeware.exe`
- Command alias: `ulanzi-dock-vibeware`

Keep the launcher and notices in the extracted folder. Use the actual release version, download links and SHA-256 values.

Check the manifests with `winget validate --manifest <manifest-directory>`, then submit to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs). The install command works only after Microsoft accepts and indexes it. WinGet does not set up sign-in startup.

[Microsoft's submission guide](https://learn.microsoft.com/en-us/windows/package-manager/package/repository)
