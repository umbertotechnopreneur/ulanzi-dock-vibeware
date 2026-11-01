<!-- VBWR B

Project: UlanziDeck
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz

VibeWare: Human intent, AI execution, and plenty of tokens
Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto


Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.

VBWR E -->

<!-- Copyright (c) 2026 Umberto Giacobbi | MIT License: see LICENSE -->

# Windows, Linux and macOS

The controller shares its Rust code across these systems. Windows has tray integration, automatic reconnect and active-application matching.

## What is available

| Feature | Windows x64 / ARM64 | Linux x64 | macOS Apple Silicon |
| --- | --- | --- | --- |
| USB controller, page navigation, themes and brightness | Implemented | Implemented; device permissions needed | Implemented; device access needs checking |
| Shortcut and media-key output | Implemented | X11 backend; native Wayland support is not enabled | Implemented; OS permissions and Mac shortcuts need checking |
| Settings and About windows | Slint windows | Standalone Slint windows in the source | Standalone Slint windows in the source |
| Choose supported installed applications | Shared application catalogue | Linux selector and optional shortcut presets in the source | Shared application catalogue |
| Tray icon and settings while the controller runs | Implemented | No tray integration | No tray integration |
| Follow the app in front and dim inactive actions | Implemented | Not implemented | Not implemented |
| Sign-in startup and replacement of a running controller | Implemented | Start and stop manually | Start and stop manually |
| Automatic reconnect after a missing/disconnected dock | Implemented | Controller returns an error | Controller returns an error |
| Download format | ZIP; local x64 and ARM64 MSIX builder | tar.gz | tar.gz; no signed app bundle |

On Linux and macOS, active-application matching reports “unsupported”. Enabled pages remain available, but do not switch automatically. The Linux application selector below does not add active-application matching. On macOS, edit Windows-specific shortcuts for your system; Ctrl and the Windows key do not automatically become Command shortcuts.

## Linux and macOS use

Extract the whole archive into a writable folder. In a terminal, enter that folder and run:

```sh
./ulanzi-dock-vibeware run
```

Keep the terminal open. Press Ctrl+C to stop the controller. Settings use the executable folder unless you pass a different `--config` path. Run one controller at a time; the Windows single-instance behavior is not implemented here.

The standalone settings window can be opened with `configure`; `about --gui` opens About. Standalone Slint windows are available in the source for all three platforms. Save settings, then restart the controller manually to apply them.

The Linux build targets Ubuntu 24.04. It needs a compatible glibc system, USB/HID permission and desktop libraries. The [build workflow](../.github/workflows/build.yml) lists development packages. The locked keyboard library uses its default X11 backend; a working GUI on Wayland would not establish that shortcuts reach native Wayland apps. Native Wayland input needs separate work and validation.

The macOS build targets macOS 15 for Apple Silicon. Older macOS versions are outside the current target, and there is no Intel package. It is a terminal program, without app signing or notarization. Input permissions and the foreground application's shortcut choices still apply.

## Linux application choices

Linux configuration includes a selector for supported installed applications. Open it without connecting to the dock:

```sh
./ulanzi-dock-vibeware configure
```

Under **Installed Linux applications**, choose an application for each page. Use **Refresh** after installing or removing a program. The selector covers these supported applications:

| Existing page | Linux choices |
| --- | --- |
| VS Code | VS Code, VS Code Insiders, VSCodium |
| Spotify | Spotify |
| Word | LibreOffice Writer |
| PowerPoint | LibreOffice Impress |
| Excel | LibreOffice Calc |
| OBS Creator | OBS Studio |

Selecting an application keeps the page's effective shortcuts. **Use suggested shortcuts** replaces its twelve action keys with the selected Linux preset. It keeps NEXT PAGE at index 4, the wide display at index 13 and the existing artwork. This is an explicit replacement: apply a preset before making your own shortcut edits.

The current settings format includes a `linux_applications` object, empty when no Linux app is selected. Selections and their keys are saved there in `settings.json`. For example, the Word choice stores its application ID in `linux_applications["Word"].application` and its editable key array in `linux_applications["Word"].keys`. Edit that key array to customize the selected Linux profile. The shared `pages` keys stay unchanged, and Windows ignores these Linux choices. Saves retain the existing `.gui.bak` backup and atomic-write behavior.

Choose **Use applications.yaml mapping** to remove a page's Linux choice and use the shared catalogue and shortcuts again. Page checkboxes still control which pages are enabled. Finding or selecting an application does not enable a page, launch the application or install anything. All ten regular pages and thirteen themes remain in place. Restart the controller after changing applications or shortcuts.

Detection reads known `.desktop` launchers in the user and system XDG application directories, Flatpak exports and Snap's desktop directory. It also checks known native commands in `PATH` when no known launcher exists. A user-hidden launcher is respected. The lookup uses the [XDG directory rules](https://specifications.freedesktop.org/basedir/latest/), the [desktop entry format](https://specifications.freedesktop.org/desktop-entry/latest/) and [Flatpak export conventions](https://docs.flatpak.org/en/latest/conventions.html#desktop-files).

A detected launcher or command is installation evidence. It does not prove that the application starts or accepts shortcuts. Custom launcher IDs and unexported Flatpak installations may not appear. Discovery never runs launcher commands or reads window titles. LibreOffice modules can share `soffice.bin`; its name cannot identify Writer, Impress or Calc as the active module. Choose those pages manually.

### Suggested shortcuts

The editor preset uses the [VS Code Linux defaults](https://code.visualstudio.com/docs/reference/default-keybindings), including Ctrl+Shift+I for Format Document. Custom editor keymaps can differ. The LibreOffice presets use its [general shortcuts](https://help.libreoffice.org/latest/en-US/text/shared/04/01010000.html), [Writer shortcuts](https://help.libreoffice.org/latest/en-US/text/swriter/04/01020000.html), [Impress shortcuts](https://help.libreoffice.org/latest/en-US/text/simpress/04/01020000.html) and [Calc shortcuts](https://help.libreoffice.org/latest/en-US/text/scalc/04/01020000.html). Writer uses [Ctrl+Shift+S for Save As](https://help.libreoffice.org/latest/en-US/text/shared/01/01070000.html); F12 would toggle an ordered list. Calc's date key inserts a [fixed date](https://help.libreoffice.org/latest/en-US/text/scalc/01/InsertCurrentDate.html), and its filter key uses [Ctrl+Shift+L](https://help.libreoffice.org/latest/en-US/text/scalc/01/12040100.html).

The Spotify preset assigns only play/pause, previous and next as generic media keys. Desktop media routing can send them to another player. The other Spotify keys show UNASSIGNED because the [public shortcut table](https://support.spotify.com/us/article/keyboard-shortcuts/) does not identify Linux bindings. Check the shortcut list in your installed Spotify app before assigning them. Calc AutoSum and Impress duplicate-slide also remain UNASSIGNED; assign suitable application shortcuts before configuring those keys.

The OBS preset keeps the project's existing shortcut chords. Assign them manually in **OBS Settings → Hotkeys**, following the [OBS Creator guide](../presets/obs-creator/README.md). The buttons do not report recording or audio state.

These selections do not change the input backend. Shortcut output still uses X11. Native Wayland input, active-application matching, automatic page switching and dimming inactive actions remain unimplemented on Linux. Desktop shortcut assignments and application keymaps can also intercept configured keys.

## Current support

Linux support is in development. The table above describes the implemented features and current desktop limitations.

My hardware is a D200H on Windows x64. Other dock models are unverified and may need changes to device detection. Owners of Windows ARM64, Linux, macOS or another model are welcome to [report their experience](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/issues), including the dock model, operating system and processor type.

## Current build

The latest local package is Windows x64 Release 1.0.8.0, built from source and installed.
ARM64, Linux and macOS were not rebuilt for this package. No dock trial was run.

I own and use only a D200H on Windows x64. Other models and platforms need owner
reports. [Tell me what you find](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/issues),
including your model, operating system and processor type.

## Where the platform differences live

For contributors: `Cargo.toml` selects Windows, Linux and macOS HID backends. `src/actions.rs` provides Windows foreground detection and delegates URL opening to `src/tray.rs`. `src/linux_apps.rs` discovers supported Linux installations, and `src/linux_apps/shortcuts.rs` defines explicitly applied presets. `src/model.rs` and `src/settings.rs` load and save the independent Linux choices. `src/gui.rs` exposes their Linux selector and standalone Slint windows on all platforms, but its resident event loop and saved monitor placement in `src/gui/placement.rs` are Windows-only. `src/controller.rs`, `src/tray.rs`, `src/startup.rs` and `src/singleton.rs` contain the reconnect, tray, startup and single-instance differences. The locked `enigo` 0.6.1 dependency uses its default `x11rb` input feature.

The planned WinUI 3 editor is [a separate Windows-only design](design/README.md). It does not replace the portable controller or current Slint windows.
