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

## Build on Windows

You need Git, Rust 1.90 or newer with its MSVC toolchain, Visual Studio C++ build tools, the Windows SDK and PowerShell 7. These tools download the source and turn it into an app.

```powershell
git clone https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware.git
Set-Location .\ulanzi-dock-vibeware
cargo build --locked --bins
```

This makes a Debug build in `target/debug/`. Add `--release` for an optimized build in `target/release/`. Neither command needs a dock. Build from the current source, not from an installed app or an older download.

Before introducing a new external library or binary, scan its downloaded archive and extracted files with Microsoft Windows Defender. Keep the source, version or SHA-256, command and clean result. Reuse that evidence for unchanged files. Do not add antivirus scans to builds or tests.

## Build on Linux

Prepare Rust 1.90 or newer, a C compiler and linker, C library headers,
`pkg-config` and the xkbcommon development files. On Ubuntu, the baseline native
packages are `gcc`, `libc6-dev`, `pkg-config` and `libxkbcommon-dev`.
The [build workflow](../.github/workflows/build.yml) installs a broader library
set; that list is not a confirmed minimum for the current enabled features.
Prepare and scan missing toolchain or system packages before installing them.
The build script does not install prerequisites.

Keep downloaded dependencies outside the repository. Reuse an existing scanned
Cargo cache where possible. On a live USB with persistence, use another mounted
disk for the cache, temporary files and build output. Check its mount, free space
and filesystem first. The settings writer needs hard-link support. Avoid making
a second copy of a cache that is already available.

Choose absolute paths appropriate for your machine. For example, set these to
writable directories on the selected disk:

```sh
export CARGO_HOME="/path/to/cargo-cache"
export CARGO_TARGET_DIR="/path/to/build-output"
export TMPDIR="/path/to/temporary-files"
```

Prepare dependencies separately from compilation. From the repository folder,
fetch the locked Linux x64 dependencies into that external cache:

```sh
cargo fetch --locked --target x86_64-unknown-linux-gnu
```

[`cargo fetch`](https://doc.rust-lang.org/cargo/commands/cargo-fetch.html)
downloads dependency material without compiling it or executing dependency
`build.rs` scripts. It can extract crate sources into the cache. Before any
build, scan the downloaded `.crate` archives in `$CARGO_HOME/registry/cache` with
ClamAV using current signatures, then scan the extracted sources in
`$CARGO_HOME/registry/src`. Include any Git dependency checkouts under
`$CARGO_HOME/git` and any separately downloaded tools. Preserve the source,
versions and clean scan results. Scan errors, skipped files or scan limits must
be resolved before proceeding; they are not a clean result. Do not execute
installers or dependency build scripts before this scan is complete.

Once the toolchain, native libraries and cached dependencies are ready and
scanned, build from the repository folder:

```sh
bash scripts/build-linux.sh
```

The script calls the shared console, checks prerequisites and the dependency
scan receipt, then runs `cargo build --frozen --release --bins`. It inherits the
cache, output and temporary directories above. It also prevents rustup from
automatically installing a missing toolchain. Cargo's
[`--frozen`](https://doc.rust-lang.org/cargo/commands/cargo-build.html#manifest-options)
combines `--locked` and `--offline`: the build keeps `Cargo.lock` unchanged and
Cargo does not download missing dependencies. An incomplete cache causes an
error. Fetch and scan the missing material separately before trying again.

The script builds optimized binaries without running the app, software tests or
dock checks. It leaves the cache and build output available for reuse. With the
example output directory above, the controller is at
`$CARGO_TARGET_DIR/release/ulanzi-dock-vibeware`. To open its configurator without
connecting to the dock:

```sh
"$CARGO_TARGET_DIR/release/ulanzi-dock-vibeware" configure
```

The configurator lists supported installed Linux apps and saves explicit choices.
See [Linux application choices](PLATFORMS.md#linux-application-choices) for
profiles, shortcut editing and current desktop limitations. Start the controller
with `run` only when ready for an authorized dock trial. A successful build does
not prove that HID access or desktop shortcuts work.

## One console for the tools

Use Python 3.11 or newer. Open the shared menu from the repository folder:

```sh
python3 scripts/ulanzi.py
```

On Windows, use `python` instead of `python3`. The PowerShell entry
`pwsh -NoProfile -File scripts/ulanzi.ps1` opens the same menu. The shared console
owns navigation and native Linux build/setup/packaging. Windows packaging and
artwork implementations remain separate modules called by that console; they
are not copies of a second menu. Commands can run
directly without prompts:

```sh
python3 scripts/ulanzi.py preflight
python3 scripts/ulanzi.py setup
python3 scripts/ulanzi.py build
python3 scripts/ulanzi.py configure
```

When ready to use the connected dock, select `run` in the menu or execute
`python3 scripts/ulanzi.py run`. This opens the device and dispatches configured
shortcuts. It is a separate explicit action, never part of setup or build.

`preflight` checks installed build prerequisites. It does not connect to the dock.
`setup` installs missing build packages on Debian/Ubuntu through APT, using sudo
when needed. Setup also installs ClamAV and its updater from authenticated distribution
repositories when missing. This scanner bootstrap is the sole exception to the
scan-before-install rule: APT verifies repository signatures and package hashes.
Setup updates missing or stale signatures automatically. If the system updater
is running, setup stops it during the update and restores it afterward.
Signatures must be under 48 hours old. Signature updates use
[FreshClam](https://docs.clamav.net/manual/Usage/SignatureManagement.html).
The scanner accepts individual files up to 1 GiB and scan data up to 2 GiB;
exceeding either limit stops setup. It scans downloaded DEBs
and their extracted contents before installation. APT package scripts run only
after those scans pass. The distribution must provide Rust 1.90 or newer.
Pillow and ReportLab are included in the Linux package prerequisites.
Other operating systems still need the manual prerequisites described above.
No vendor software, drivers or firmware are installed by setup.

Setup fetches locked Cargo dependencies, scans archives, extracted sources and
Git checkouts, and records the scan in the external cache. Errors and scan-limit
alerts stop setup. Builds compare cached dependency contents with the scan
receipt before compiling; they do not run antivirus scans. Run setup again if the
cache changes. Keep the cache private to your user.

Linux build output, Cargo material and temporary files default to
`$XDG_CACHE_HOME/ulanzideck` (or `~/.cache/ulanzideck`). Explicit `CARGO_HOME`,
`CARGO_TARGET_DIR` and `TMPDIR` override those locations. Select a suitable mounted
disk when using a live USB. Builds use up to four parallel jobs by default;
`CARGO_BUILD_JOBS` can override this. `scripts/build-linux.sh` forwards to the same build
command. Compilation uses `--frozen --release --bins` and never downloads missing
dependencies. `configure` opens settings without starting a dock trial.

Use `python3 scripts/ulanzi.py deb` to package a native Linux build.
See [Debian packages](RELEASING.md#make-a-debian-package) for options and installation.
The console also exposes icons, preview sheets, the OBS guide, dependency notices,
Windows installers and packaging. On other systems, Python artwork tasks need separately prepared
and scanned Pillow or ReportLab dependencies. Windows packaging tasks require
PowerShell 7 and retain their existing options:

```powershell
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command build -KeepBuildCache
pwsh -NoProfile -File scripts/ulanzi.ps1 -Command msix -Configuration Debug -PublisherSubject 'CN=Your Local Publisher'
```

The Windows build preserves staged replacement, backups and explicit process-stop
options. Its existing dependency preparation remains manual. Packaging never
publishes a GitHub release. The console does not run software tests, diagnostics
or dock trials automatically. Running preflight is an explicit inspection command.

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
| `ui/dialogs.slint`, `src/gui.rs` and `src/gui/placement.rs` | Define windows, connect their buttons and remember Windows display placement. |
| `src/paths.rs`, `src/startup.rs` and `src/tray.rs` | Find settings and provide Windows desktop integration. |
| `src/art.rs`, `src/art/icons.rs`, `src/art/palette.rs` | Compose button pictures, build icon masks and choose theme colors/motifs. |
| `src/licensing.rs`, `src/licensing/store.rs`, `src/gui/store_ui.rs` | Apply the Store theme policy, read licenses and connect purchase controls. |

`settings.json` stores pages, shortcuts and runtime preferences. `applications.yaml` maps apps to pages and can provide app commands. Keep preferences such as automatic switching in the JSON file. The app accepts the current formats; it does not convert older ones.

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
