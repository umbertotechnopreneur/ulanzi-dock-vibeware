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

# Build and change UlanziDeck

The app is written in **Rust**. Rust turns source code into a fast, self-contained program. People using the app do not need Rust or Python installed.

**Slint** draws the settings windows and connects their controls to Rust. **Python** prepares pictures, the printable OBS guide and license notices. It is a development tool, not part of the running controller.

## Prepare new dependencies

Get the locked versions and native packages from their official sources.
Stage each new archive and its extracted files outside the repository. Check
the source and version, then scan the staging files with Microsoft Windows
Defender. For Linux imports, use a Windows-accessible staging drive and scan
the Linux files there before moving them to the build host. Use the files once
the targeted scan has finished without detections. Reuse the same unchanged
staging files during that session. Keep scan results in the current session.

An unavailable or failed scan stops the import. Builds and tests use prepared
dependencies offline. Remove temporary staging files and intermediate build
output after the task.

## Build on Windows

Use Git, Rust 1.90 or newer with its MSVC toolchain, Visual Studio C++ build
tools, the Windows SDK and PowerShell 7. Prepare missing dependencies before
compiling. The following builds the current source with the prepared Cargo cache.

```powershell
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command build
```

The controller and launcher go to `artifacts/`. For a Debug build from source,
use `cargo build --frozen --bins` with the same cache.

## Build on Linux

Prepare Rust 1.90 or newer, a C compiler, C library headers, `pkg-config` and
the native development libraries listed in the [build workflow](../.github/workflows/build.yml).
Use `dpkg-dev` for Debian packaging and Pillow and ReportLab for artwork tools.

If you need fresh Cargo dependencies, stage the locked dependencies for your
native architecture with the shared console.

```sh
python3 scripts/ulanzi.py setup /path/to/empty-staging
```

This fetches dependencies into an empty directory outside the repository. It
uses the installed toolchain and does not compile or install anything. Stage
and scan the new `.crate` archives and extracted sources before using the cache.
If Cargo has only downloaded an archive, extract a copy for the scan. Include
any fetched Git dependency checkouts. The same first-import steps apply to any
new native packages you prepare separately.

Point `CARGO_HOME` to your prepared Cargo cache. Use a writable directory for
build output. On a live USB, choose a mounted disk with enough space.

```sh
export CARGO_HOME="/path/to/prepared-cargo-cache"
export CARGO_TARGET_DIR="/path/to/build-output"
bash scripts/build-linux.sh
```

The script runs `cargo build --frozen --release --bins`. An incomplete cache
stops the build. The controller is in `$CARGO_TARGET_DIR/release/`. To open
settings, use `"$CARGO_TARGET_DIR/release/ulanzi-dock-vibeware" configure`.

See [Linux application choices](PLATFORMS.md#linux-application-choices) for
profiles, shortcut editing and desktop limitations.

## One console for the tools

Use Python 3.11 or newer. Open the shared menu from the repository folder:

```sh
python3 scripts/ulanzi.py
```

On Windows, use `python` instead of `python3`. The PowerShell entry
`pwsh -NoProfile -File scripts/ulanzi.ps1` opens the same menu. The shared console
provides the shared menu. Linux build and packaging run directly from it. The
console calls the Windows and artwork scripts for their commands.

```sh
python3 scripts/ulanzi.py preflight
python3 scripts/ulanzi.py build
python3 scripts/ulanzi.py configure
```

When ready to use the connected dock, select `run` in the menu or execute
`python3 scripts/ulanzi.py run`. This opens the device and dispatches configured
shortcuts. It is a separate explicit action, never part of setup or build.

`preflight` checks installed build tools. `setup` stages new Linux Cargo
dependencies for a separate first-import scan. `build` uses the prepared cache
offline. `configure` opens settings without accessing the dock.

On Linux, build output and temporary files default to `$XDG_CACHE_HOME/ulanzideck`
or `~/.cache/ulanzideck`. Set `CARGO_TARGET_DIR` and `TMPDIR` to use other
directories. Builds use up to four parallel jobs by default. Set
`CARGO_BUILD_JOBS` to change that.

Use `python3 scripts/ulanzi.py deb` to package a native Linux build.
See [Debian packages](RELEASING.md#make-a-debian-package) for options and installation.
The console also exposes icons, preview sheets, the OBS guide, dependency notices,
Windows installers and packaging. On other systems, Python artwork tasks need separately prepared
and scanned Pillow or ReportLab dependencies. Windows packaging tasks require
PowerShell 7 and retain their existing options:

```powershell
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command build
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command msix -Configuration Debug -PublisherSubject 'CN=Your Local Publisher'
```

The Windows build stages both executables before replacing the output pair.
MSIX packaging uses a separate build directory. Both use prepared dependencies.
Packaging and publication are separate commands.

## Check Linux actions without a dock

When an action test run is authorized, use `python3 scripts/ulanzi.py test-actions`.
The reference layout covers 87 distinct actions across the 140 key positions
on the ten enabled pages with the MATRIX theme. Each simulated press and release
uses the controller's routing code. Desktop output goes to memory. The tests
compare key order, media methods, URLs and internal commands against the expected
effect of each position. They also check the index, disabled controls and
brightness limits. The test process can run alongside the controller.

Add `--config /absolute/path/settings.json` to compare an existing profile,
including its catalogue and Linux overrides, with this reference layout.
Intentional differences appear as mismatches. Confirm application behavior
separately by observing the desktop when you release the physical button.

## Find the right file

| File | What it does |
| --- | --- |
| `src/main.rs` | Reads terminal commands. |
| `src/controller.rs` and `src/dock.rs` | Connect to the dock, draw pages and receive button presses. |
| `src/model.rs` and `src/settings.rs` | Define pages and save preferences. |
| `src/persistence.rs` | Lock, back up and atomically publish state across processes. |
| `src/actions.rs` and `src/applications.rs` | Send configured shortcuts and match applications. |
| `ui/dialogs.slint`, `src/gui.rs` and `src/gui/placement.rs` | Draw the model, send control requests and remember Windows display placement. |
| `src/paths.rs`, `src/startup.rs` and `src/tray.rs` | Find settings and provide Windows desktop integration. |
| `src/art.rs`, `src/art/icons.rs`, `src/art/palette.rs` | Compose button pictures, build icon masks and choose theme colors/motifs. |
| `src/licensing.rs`, `src/licensing/store.rs`, `src/gui/store_ui.rs` | Apply the Store theme policy, read licenses and connect purchase controls. |

`settings.json` stores pages, shortcuts and preferences. `applications.yaml` maps
apps to pages and can provide app commands. The controller applies those mappings
when it starts. Live preference changes retain the foreground options passed to `run`.
The settings editor sends a single change at a time and draws the confirmed
model after saving.

Windows dialogs save their size, position, monitor and DPI in `settings.windows.json` beside the selected settings file. If a saved monitor is disconnected, the dialog uses the monitor under the mouse, or the primary monitor if the pointer is unavailable. A second interactive launch in the same Windows session brings the existing Configuration window forward if it is open, or shows About otherwise. It preserves the current window size and does not force maximization. The second process exits successfully without a dialog or console prompt. A duplicate background launch exits silently without opening a window.

`is_first_run` starts true for a profile that has not seen the graphical welcome.
The first Windows launch opens About on the primary display and clears the flag
only after native presentation succeeds. Later left-clicks on the tray icon open
About using its remembered placement; right-clicks open the tray menu. Windows
startup never depends on the terminal welcome, which remains available through
the explicit `run --oobe` command.

Each writable state file has a persistent `.lock` sidecar. Its OS handle serializes
application transactions; the empty sidecar is not a stale lock after a crash.
Do not remove it while a process may be saving. Backups remain in `.gui.bak` files.
Initial file publication requires filesystem hard-link support (for example NTFS).
External editors do not honor the application lock; stale-snapshot checks detect
observed changes, but cannot eliminate every race with an unrelated editor.

See [Store licensing](LICENSING.md) for the optional Windows Store build and private
configuration. Ordinary community builds include all themes. No real product
identity belongs in this repository.

## Work with Codex

Open this repository as a local project in Codex, or run `codex` in its folder if you already use the CLI. Read `AGENTS.md` before changing anything. Tests, diagnostics and dock trials need my explicit permission for each run.

The dock's Codex page sends configured keyboard shortcuts. It does not install Codex or call an AI API.

Keep my name, original logo and [MIT notices](BRANDING.md). See [Windows editor ideas](design/README.md) for the planned WinUI 3 interface; it has not replaced Slint.
