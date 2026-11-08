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

The controller shares its Rust code across these systems. The controller retries a missing or disconnected dock on every platform. Windows also has native tray integration and active-application matching.

## What is available

| Feature | Windows x64 / ARM64 | Linux x64 | macOS Apple Silicon |
| --- | --- | --- | --- |
| USB controller, page navigation, themes and brightness | Implemented | Implemented; device permissions needed | Implemented; device access needs checking |
| Shortcut and media-key output | Implemented | MPRIS for playback. PipeWire for audio. Native GNOME desktop commands. Portal keyboard session on Wayland. X11 shortcuts on X11. | Implemented; OS permissions and Mac shortcuts need checking |
| Settings and About windows | Slint windows | Standalone Slint windows in the source | Standalone Slint windows in the source |
| Choose supported installed applications | Shared application catalogue | Linux selector and optional shortcut presets in the source | Shared application catalogue |
| Tray icon and settings while the controller runs | Implemented | StatusNotifierItem on compatible desktops | No tray integration |
| Follow the app in front and dim inactive actions | Implemented | Not implemented | Not implemented |
| Sign-in startup and replacement of a running controller | Implemented | Opt-in installed user service; desktop launcher reuses it | Start and stop manually |
| Automatic reconnect after a missing/disconnected dock | Implemented | Implemented; owner confirmed display restoration after reconnect | Implemented; unverified |
| Download format | ZIP and local x64 and ARM64 MSIX builder | tar.gz and Debian package | tar.gz |

On Linux and macOS, active-application matching reports “unsupported”. Enabled pages remain available, but do not switch automatically. The Linux application selector below does not add active-application matching. The page index always lists enabled pages, including those whose app is closed. Selecting LibreOffice changes the visible Word/Excel/PowerPoint labels to Writer/Calc/Impress while keeping the internal page identifiers. On macOS, edit Windows-specific shortcuts for your system; Ctrl and the Windows key do not automatically become Command shortcuts.

## Linux and macOS use

Extract the whole archive into a writable folder. In a terminal, enter that folder and run:

```sh
./ulanzi-dock-vibeware run
```

Keep the terminal open. Press Ctrl+C to stop the controller. On macOS, run one controller at a time from the extracted folder.

When the Linux controller starts, it sends a desktop notification and displays the UlanziDeck icon on desktops with an indicator host, such as KDE Plasma or GNOME with AppIndicator support. Desktop notification preferences control the banner. Activating the indicator opens settings for the running profile. The icon remains available while waiting for the dock or recovering a connection. Its tooltip reports the connection state. The icon disappears when the controller stops.

On portable Linux, `run` and `configure` use the same default profile beside the executable. To use a different profile, pass the same `--config` path to both commands. Linux allows one controller per user. Stop the current controller before starting another profile. The controller retries missing interfaces and USB access errors with a delay of up to two seconds.

Keep your Linux profile in a folder you control. Use regular files for the settings, application catalogue and session state. Linux rejects symbolic links at those filenames. When the controller creates or replaces settings and backups, it grants access only to the owner and preserves stricter owner permissions.

Open the Debian desktop launcher to open settings and start or reuse the controller service. It can start with a new profile without a terminal. Turn on **Start at sign-in** in Settings or run `ulanzideck startup --enable`. Disabling this option leaves a running controller active. Use `configure` to open settings on its own, or `about --gui` to open About. Theme, brightness and enabled pages update while the controller runs. Restart after changing app mappings or shortcuts.

The Linux build targets Ubuntu 24.04. It needs compatible system libraries and USB/HID permission. The [build workflow](../.github/workflows/build.yml) lists development packages.

On Wayland, the first keyboard shortcut opens the desktop's keyboard permission dialog. Allow keyboard access, return to the app you want to use and press the dock button again. If the session closes, a later press can open a new permission dialog. On X11, shortcuts use a persistent X11 input connection.

For PLAY / PAUSE, NEXT TRACK and PREVIOUS, open a player with MPRIS support in the same desktop session. Playback commands work on Wayland and X11. A playing player takes priority over a paused or stopped player. Ties are resolved by the player's MPRIS bus name in alphabetical order. MUTE and volume use WirePlumber's `wpctl` to control the default PipeWire output. Volume moves by five percentage points up to 100%. The Debian package includes WirePlumber as a dependency.

The macOS build targets macOS 15 for Apple Silicon. Older macOS versions are outside the current target, and there is no Intel package. It is a terminal program, without app signing or notarization. Input permissions and the foreground application's shortcut choices still apply.

## Linux desktop buttons

New Linux profiles use explicit `desktop:` actions on Windows / media and Utility.
Files opens the home folder with GIO. Settings opens GNOME Settings. TASK VIEW
opens the GNOME overview. Show Desktop uses the window manager’s
`_NET_SHOWING_DESKTOP` request when supported. Screenshot requests the interactive
capture UI through the desktop portal. Capture permissions follow desktop policy.
Emoji opens GNOME Characters, where a character can be selected and copied.

Clipboard history uses the installed, enabled Clipboard Indicator extension and
its configured menu shortcut. The extension owns clipboard storage and history.
The controller reads that shortcut and invokes it through the same input connection
used by COPY and PASTE. Generic application shortcuts require the initial
Wayland keyboard authorization. SEARCH opens the GNOME overview, where typing
starts a search. A missing extension or unavailable command is reported
in the controller log. Desktop integration here targets GNOME.

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

Selecting an application keeps the page's shortcuts. **Replace with suggested shortcuts** applies the Linux preset to the twelve action keys. NEXT PAGE, the wide display and the existing artwork stay in place. Apply a preset before customizing the shortcuts.

The current settings format includes a `linux_applications` object, empty when no Linux app is selected. Selections and their keys are saved there in `settings.json`. For example, the Word choice stores its application ID in `linux_applications["Word"].application` and its editable key array in `linux_applications["Word"].keys`. Edit that key array to customize the selected Linux profile. The shared `pages` keys stay unchanged, and Windows ignores these Linux choices. Saves retain the existing `.gui.bak` backup and atomic-write behavior.

Choose **Use shared app mapping** to remove a page's Linux choice and use the shared catalogue and shortcuts again. Page checkboxes still control which pages are enabled. Restart the controller after changing applications or shortcuts.

Detection reads known `.desktop` launchers in the user and system XDG application directories, Flatpak exports and Snap's desktop directory. It also checks known native commands in `PATH` when no known launcher exists. A user-hidden launcher is respected. The lookup uses the [XDG directory rules](https://specifications.freedesktop.org/basedir/latest/), the [desktop entry format](https://specifications.freedesktop.org/desktop-entry/latest/) and [Flatpak export conventions](https://docs.flatpak.org/en/latest/conventions.html#desktop-files).

A detected launcher or command is installation evidence. It does not prove that the application starts or accepts shortcuts. Custom launcher IDs and unexported Flatpak installations may not appear. Discovery never runs launcher commands or reads window titles. LibreOffice modules can share `soffice.bin`; its name cannot identify Writer, Impress or Calc as the active module. Choose those pages manually.

### Suggested shortcuts

On Linux, a hotkey accepts one final key, optionally preceded by distinct modifiers such as Ctrl, Alt, Shift or Super. For example, `hotkey:ctrl+shift+s` and `hotkey:f5` are valid.

The editor preset uses the [VS Code Linux defaults](https://code.visualstudio.com/docs/reference/default-keybindings), including Ctrl+Shift+I for Format Document. Custom editor keymaps can differ. The LibreOffice presets use its [general shortcuts](https://help.libreoffice.org/latest/en-US/text/shared/04/01010000.html), [Writer shortcuts](https://help.libreoffice.org/latest/en-US/text/swriter/04/01020000.html), [Impress shortcuts](https://help.libreoffice.org/latest/en-US/text/simpress/04/01020000.html) and [Calc shortcuts](https://help.libreoffice.org/latest/en-US/text/scalc/04/01020000.html). Writer uses [Ctrl+Shift+S for Save As](https://help.libreoffice.org/latest/en-US/text/shared/01/01070000.html); F12 would toggle an ordered list. Calc's date key inserts a [fixed date](https://help.libreoffice.org/latest/en-US/text/scalc/01/InsertCurrentDate.html), and its filter key uses [Ctrl+Shift+L](https://help.libreoffice.org/latest/en-US/text/scalc/01/12040100.html).

The Linux Spotify shortcut preset assigns the same three MPRIS playback actions. These actions can control another running player. The other Spotify keys show UNASSIGNED because the [public shortcut table](https://support.spotify.com/us/article/keyboard-shortcuts/) does not identify Linux bindings. Check the shortcut list in your installed Spotify app before assigning them. Calc AutoSum and Impress duplicate-slide also remain UNASSIGNED; assign suitable application shortcuts before configuring those keys.

The OBS preset keeps the project's existing shortcut chords. Assign them manually in **OBS Settings → Hotkeys**, following the [OBS Creator guide](../presets/obs-creator/README.md). The buttons do not report recording or audio state.

Choose pages manually on Linux. Desktop shortcut assignments and application keymaps can intercept configured keys on both Wayland and X11.

## Current build

The latest local package is Windows x64 Debug 1.0.9.0, built from source and
signed for local testing. Windows x64 Release 1.0.8.0 is still installed.

I own and use only a D200H on Windows x64. Other models and platforms need owner
reports. [Tell me what you find](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/issues),
including your model, operating system and processor type.

## Where the platform differences live

For contributors: `Cargo.toml` selects Windows, Linux and macOS HID backends. `src/actions.rs` provides Windows foreground detection and delegates URL opening to `src/tray.rs`. `src/linux_apps.rs` discovers supported Linux installations, and `src/linux_apps/shortcuts.rs` defines explicitly applied presets. `src/model.rs` and `src/settings.rs` load and save the independent Linux choices. `src/gui.rs` exposes their Linux selector and standalone Slint windows on all platforms, but its resident event loop and saved monitor placement in `src/gui/placement.rs` are Windows-only. `src/controller.rs`, `src/tray.rs`, `src/startup.rs` and `src/singleton.rs` contain the connection recovery, tray, startup and single-instance rules. The locked `enigo` 0.6.1 dependency uses its default `x11rb` input feature.

The planned WinUI 3 editor is [a separate Windows-only design](design/README.md). It does not replace the portable controller or current Slint windows.
