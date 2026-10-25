<!-- Copyright (c) 2026 Umberto Giacobbi | MIT License: see LICENSE -->

# Windows, Linux and macOS

The controller shares its Rust code across these systems. The Windows desktop integration has more features. Building a package and using it on a real dock are separate checks.

## What is available

| Feature | Windows x64 / ARM64 | Linux x64 | macOS Apple Silicon |
| --- | --- | --- | --- |
| USB controller, page navigation, themes and brightness | Implemented | Implemented; device permissions needed | Implemented; device access needs checking |
| Shortcut and media-key output | Implemented | X11 backend; native Wayland support is not enabled | Implemented; OS permissions and Mac shortcuts need checking |
| Settings and About windows | Slint windows | Standalone Slint windows in the source | Standalone Slint windows in the source |
| Tray icon and settings while the controller runs | Implemented | No tray integration | No tray integration |
| Follow the app in front and dim inactive actions | Implemented | Not implemented | Not implemented |
| Sign-in startup and replacement of a running controller | Implemented | Start and stop manually | Start and stop manually |
| Automatic reconnect after a missing/disconnected dock | Implemented | Controller returns an error | Controller returns an error |
| Download format | ZIP; local x64 MSIX builder | tar.gz | tar.gz; no signed app bundle |

On Linux and macOS, application matching reports “unsupported”. Pages remain available, but do not switch automatically. Edit Windows-specific shortcuts for your system. In particular, Ctrl and the Windows key do not automatically become macOS Command shortcuts.

## Linux and macOS use

Extract the whole archive into a writable folder. In a terminal, enter that folder and run:

```sh
./ulanzi-dock-vibeware run
```

Keep the terminal open. Press Ctrl+C to stop the controller. Settings use the executable folder unless you pass a different `--config` path. Run one controller at a time; the Windows process-replacement behavior is not implemented here.

The standalone settings window can be opened with `configure`; `about --gui` opens About. They are built on all three platforms, but Linux/macOS interaction has not been confirmed on a real desktop. Save settings, then restart the controller manually to apply them.

The Linux archive is built on Ubuntu 24.04. It needs a compatible glibc system, USB/HID permission and desktop libraries. The [build workflow](../.github/workflows/build.yml) lists development packages. The locked keyboard library uses its default X11 backend; a working GUI on Wayland would not establish that shortcuts reach native Wayland apps. Native Wayland input needs separate work and validation.

The macOS archive is built on macOS 15 for Apple Silicon. Older macOS versions are not validated, and there is no Intel package. It is a terminal program, without app signing or notarization. Input permissions and the foreground application's shortcut choices still apply.

## Build evidence

Checked on 25 October 2026: the latest completed [Build run, 21 October](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/actions/runs/39552455782) built and packaged all four targets successfully: Windows x64, Windows ARM64, Linux x64 and macOS ARM64. It used commit `0a3373a2459031a1696e15a3bd8e67455d6e968d`. The committed changes after that run, up to `7a4986575942b1a431ca31745076727da1391295`, only changed the README.

That CI run does not validate later working-folder changes, including the 100 ms page fade, picture cache, startup picture and remembered-page changes, or prove device behavior. On 25 October 2026, the current working source completed a local optimized Windows x64 build. No software test or dock trial was run, and the other platforms were not rebuilt from these changes.

I own and have physically tested only a D200H on Windows x64. Some features still need a physical check there, too. Windows ARM64, Linux, macOS and other dock models need owner reports. Other models may also need changes to device detection. [Tell me what you find](https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware/issues), including your model, operating system and processor type.

## Where the platform differences live

For contributors: `Cargo.toml` selects Windows, Linux and macOS HID backends. `src/actions.rs` provides Windows foreground detection and platform URL openers. `src/gui.rs` exposes standalone Slint windows on all platforms, but its resident event loop is Windows-only. `src/controller.rs`, `src/tray.rs`, `src/startup.rs` and `src/singleton.rs` contain the reconnect, tray, startup and process-replacement differences. The `enigo` 0.6.1 manifest in the local Cargo cache confirms the default `x11rb` input feature.

The planned WinUI 3 editor is [a separate Windows-only design](design/README.md). It does not replace the portable controller or current Slint windows.
