# Releasing UlanziDock VibeWare

The first stable public version is **0.1.0**, tagged **v0.1.0-stable**.
Earlier `v0.1.0`, `v0.2.0`, and `v0.3.0` releases remain historical previews.
The stable tag deliberately uses a separate name; never move existing tags.

## Packages

Both **Build** and the manual **Release** workflow compile actual release
binaries for Windows x64/ARM64, Linux x64, and macOS Intel/Apple Silicon.
Windows uses ZIP; Unix uses tar.gz to retain executable permissions. The Windows
package includes the native sign-in launcher alongside the controller.
Each archive includes README, MIT license, artwork notes, the older PDF guide,
Slint notices and license text, Bitstream Vera attribution, and dependency
notices copied from restored Cargo sources. The PDF remains a historical
catalogue and does not include the newer Office pages.

Linux packages target Ubuntu 24.04 or a compatible glibc desktop. They depend on
system libraries, including udev, fontconfig, xkbcommon, Wayland and X11 support;
install the runtime equivalents of the libraries in the build workflow.
They are not static binaries for every Linux distribution. HID device permissions
must be configured for the intended user. Do not assume root access is needed.
macOS packages target macOS 15 and later; they are terminal distributions,
not signed or notarized app bundles. macOS privacy permissions may be needed
for configured shortcut injection.

Windows tray, foreground application detection and sign-in registration are
Windows features. Compile success is not physical validation on another OS.
Packaging/building does not launch the controller, write to HID, register
startup, or run software or hardware tests.

## Publication

Open a pull request and review source changes. The manual **Release** workflow
accepts `tag=v0.1.0-stable`, `draft=true`, and `prerelease=false`. It creates a
new draft only when all five archives and their checksums exist. Publish after
owner review. Existing releases and tags are never overwritten.

To package an already built target locally:

```powershell
pwsh -NoProfile -File scripts/package-release.ps1 -Target x86_64-pc-windows-msvc -Label windows-x64 -Tag v0.1.0-stable
```

The executable must be under `target/<target>/release/`. Packaging refuses to
reuse a staging directory or replace an archive.

## WinGet

Proposed identifier: `UmbertoGiacobbi.UlanziDockVibeWare`; version: `0.1.0`.
Only the two Windows ZIPs belong in WinGet. Set `InstallerType: zip`,
`NestedInstallerType: portable`, and point `NestedInstallerFiles` at
`ulanzi-dock-vibeware.exe`, with command alias `ulanzi-dock-vibeware`.
The ZIP extraction also retains `ulanzi-dock-launcher.exe` and notices.
WinGet installation does not register sign-in startup or connect to the dock.

Use exact final release asset URLs and SHA-256 hashes. Validate the three
manifests with `winget validate --manifest <manifest-directory>`, then submit
under `manifests/u/UmbertoGiacobbi/UlanziDockVibeWare/0.1.0/` in
[microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs).
The install command becomes available after Microsoft merges and indexes it.
Publishing ZIPs on GitHub does not guarantee a same-day WinGet listing.

References: [submission](https://learn.microsoft.com/en-us/windows/package-manager/package/repository)
and [archive manifest schema](https://github.com/microsoft/winget-pkgs/blob/master/doc/manifest/schema/1.9.0/installer.md).
