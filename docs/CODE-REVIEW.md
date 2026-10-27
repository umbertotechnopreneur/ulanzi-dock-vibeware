<!-- VBWR B

Project: UlanziDeck
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz

VibeWare: Human intent, AI execution, and plenty of tokens
Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto

Created with AI: OpenAI Codex; source review, validation evidence and remaining findings; 2026-10-27
Modified with AI: OpenAI Codex; source review, validation evidence and remaining findings and readability; 2026-10-27
Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
Evidence: docs/CODE-REVIEW.md; current working-tree changes

Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.

VBWR E -->

# Code review — 27 October 2026

This review covers the Rust controller, settings, Windows integration, Slint dialogs,
and selected build and release paths. It is a source review, not a release approval.
Windows source builds and the owner-authorized offline quality regression suite
were run. The pre-licensing suite passed 24 tests (3 library and 21 controller
binary tests). Later licensing work is build-only at the owner's request; those
test results do not cover licensing or the subsequent documentation pass.
No native GUI session, Store purchase, installer run or dock trial was performed.
The scripts and dependencies have not received an exhaustive security audit.

## Changes made

| Priority | Finding | Change in this working tree |
| --- | --- | --- |
| High | Starting a controller terminated every process with the same executable filename. That included standalone configuration windows and copies in unrelated folders. The takeover mutex was released before controller startup. | A Windows session mutex now remains owned for the controller's lifetime. A second controller reports that the existing instance must be closed. No process enumeration or forced termination remains in the application singleton path. |
| High | The HID parser treated every state byte other than `1` as a release. After an accepted press, a malformed report could therefore trigger its action. | Only `0` and `1` produce events. Parsing is a separate function, independent of the device handle. |
| Medium | Unsupported shortcuts, theme targets and artwork could survive configuration validation and fail during button handling or drawing. A bad theme action could stop the controller. | Validate actions, navigation targets, themes and explicit artwork when loading or saving configuration. Share shortcut/media parsing with execution. Reject index artwork outside the ten implemented symbols. |
| Medium | Configure reset its dimensions to 600 × 740 on every presentation. Window size defaults ignored scrollable content, and custom buttons had no text-dependent padding. | Remove the Rust size reset. Derive initial dialog height from content and footer layouts. Set useful initial widths and size buttons from their labels. Keep scrolling for limited screen space. |
| Medium | Dialogs did not remember their monitor or placement. Recovery bypassed the normal presentation path. | All three dialogs use the same presentation path. Windows placement is stored separately, including monitor identity, normal rectangle, DPI and maximized state. |
| Low | The `none` action produced an unsupported-action error. URL launching was duplicated in actions and desktop integration. | Treat `none` as a no-op. Reuse the desktop browser opener, with HTTP(S) and control-character checks before opening configured URLs. |
| Low | `Dock::show` had no callers after transition rendering replaced it. | Remove that unused device-write path. Keep full archive generation under `cfg(test)`, where its remaining callers live. Keep the one-key fallback: it still has a runtime caller. |
| Medium | Two processes could pass settings snapshot checks before either wrote, and window maps could lose other dialogs' updates. | Add canonical-path OS lock sidecars covering the complete read/modify/write transaction. Retain snapshot checks, backups and atomic replacement. |
| Medium | Package settings used an exists-then-copy seed, allowing incomplete or competing first writes. | Stage and flush complete bytes, then atomically publish with a same-volume hard link that cannot replace an existing destination. |
| Medium | More than twelve regular pages silently exceeded page-index capacity. | Reject layouts beyond twelve. Retain the owner-confirmed ten default pages, including OBS Creator before Themes. |
| Low | PID-only temporary names could conflict with abandoned files. | Use create-new staging names with process, time and sequence components. Never delete an unfamiliar temporary file to recover. |
| Medium | Build/release workflow definitions did not execute unit tests. | Add hardware-free library/binary test steps before packaging. No remote workflow was dispatched. |
| Low | `art.rs` mixed compositing with large icon and palette definitions. | Extract `art/icons.rs` and `art/palette.rs`; retain existing art and theme values. |

## Window behavior

On Windows, About, Configuration and Dock disconnected save independent entries in
`settings.windows.json`, beside the selected settings file. Closing a dialog or
shutting down normally saves its state. Forced termination cannot guarantee a save.
The file uses the existing backup and atomic replacement mechanism. It does not
change shortcuts, theme preferences or the remembered dock page.

On reopening, the saved monitor is located by its device interface identity when
available. If it is absent, use the monitor under the mouse; if the pointer cannot
be read, use the Windows primary monitor. Restore the saved size and relative
position, scaled to the target monitor's current DPI and constrained to its work
area. Restore maximized state without reopening minimized. An already visible
window is activated without resetting the user's placement.

This does not force Windows to use an old DPI setting. The saved DPI is a reference
for translating dimensions to the current display scale. Windows placement code is
isolated in `src/gui/placement.rs`. Linux and macOS retain Slint's window placement;
monitor persistence is implemented for Windows only.

Invalid state files are preserved and reported. They do not stop a dialog opening.
The backup can be inspected before removing or repairing a damaged state file.

Implementation references: [Slint layout sizing](https://docs.slint.dev/latest/docs/slint/guide/language/coding/positioning-and-layouts/),
the locally installed Slint 1.13.1 ScrollView source, and Microsoft's
[window placement guidance](https://learn.microsoft.com/en-us/windows/win32/gdi/positioning-objects-on-multiple-display-monitors).

## Persistence limits

Application processes honor the same lock. An external text editor may ignore it;
snapshot comparisons detect observed edits but cannot make a nonparticipating
writer transactional. Keep state on a filesystem supporting hard links and file
locking. The Windows local app-data path normally meets this requirement; portable
paths on other filesystems can return an explicit publication error.

The named controller mutex covers one Windows session. It is not a cross-session
USB ownership mechanism. An older executable that does not participate in this
mutex must be closed before trying the changed controller.

## Maintainability

These were the largest Rust files at the start of the review; line counts include
comments and tests and are not themselves defect counts.

| Module | Lines before changes | Useful boundary |
| --- | ---: | --- |
| `art.rs` | 1,364 | Icon masks and palettes/motifs are now separate modules. Much of the original length was drawing data rather than branching application logic. |
| `tray.rs` | 943 | Separate shared lifecycle state, native tray resources, and shell/restart operations. Keep resource ownership explicit. |
| `dock.rs` | 940 | Separate protocol decoding/archive framing from transition preparation and HID transport. Input decoding is now extracted; the larger split remains. |
| `model.rs` | 710 | Move built-in page definitions away from validation and runtime-derived state. |
| `controller.rs` | 603 | Keep the single event-loop owner. Consider a pure action/navigation decision layer to test transitions without a dock. |
| `gui.rs` | 547 | Native placement is now a separate module; callbacks and configurator state remain here. |

Further duplication exists in boot/shutdown fade loops and similar release build
steps across workflows. Extract those only after behavior has been validated.
Avoid a broad mechanical split of every long file during first-release preparation.

The unused `Dock::show` path was confirmed by repository call-site searches.
No claim is made that every unused function has been found. Cross-platform stubs,
test helpers, embedded assets and documentation images must not be deleted merely
because they have no Windows runtime caller.

## Security observations and validation still needed

The reviewed action path does not evaluate configured shell commands. Configured
URLs are restricted to HTTP(S). Foreground detection reads executable identity,
not window titles. Input dispatch remains tied to accepted dock press/release
pairs. HID selection still requires the D200H vendor/product and consumer interface.
These are source observations, not proof against a malicious USB device or a
compromised dependency. Local settings can intentionally define powerful shortcuts.

The quality regression suite exercised invalid configuration/actions, malformed
HID state parsing, stale editors, isolated cross-process state transactions, and
image/archive generation. It did not open a device or launch the native GUI.
An existing grayscale assertion was corrected to account for the already colored
unavailable overlay while still checking the grayscale base and navigation/clock.

Before considering these changes ready to distribute:

1. Authorize standalone GUI checks with temporary settings: first opening, resize,
   close/reopen, process restart, maximized state and separate dialog positions.
2. Check Windows displays at 100%, 150% and 200%, including a monitor left of the
   primary, a taskbar on another edge, a DPI change and a disconnected saved monitor.
   Verify that controls and footer buttons remain reachable on a small work area.
3. Follow up the dependency findings below before claiming release security readiness.
4. Arrange a separate watched D200H trial for input handling and controller startup.
   Standalone GUI validation does not authorize opening HID or prove dock behavior.

## Dependency advisory review

On 27 October 2026, the 591 registry packages in Cargo.lock were queried by public
name and version against the OSV API. No private product identity was sent. Ten
packages matched twelve RustSec advisories. This is an advisory lookup with a
source reachability review, not proof of absence of vulnerabilities.

| Locked package | Finding and current relevance |
| --- | --- |
| `zip` 2.4.2 | [0367](https://rustsec.org/advisories/RUSTSEC-2026-0367.html), [0368](https://rustsec.org/advisories/RUSTSEC-2026-0368.html), [0369](https://rustsec.org/advisories/RUSTSEC-2026-0369.html): unsafe extraction behavior around permissions/symlinks. The reviewed app writes archives and reads its own generated bytes in memory; no extraction API or external ZIP input path was found. Fixed versions require a major-version migration. |
| `tokio` 1.53.2 | [0386](https://rustsec.org/advisories/RUSTSEC-2026-0386.html): Unix AsyncFd unsoundness. Not in the resolved Windows x64 dependency tree. Linux target inspection could not complete offline because a cached dependency was missing; no Linux validation is claimed. |
| `bincode` 2.0.1 | [0141](https://rustsec.org/advisories/RUSTSEC-2025-0141.html): maintenance status. |
| `paste` 1.0.15 | [0436](https://rustsec.org/advisories/RUSTSEC-2024-0436.html): maintenance status. |
| `rustybuzz` 0.20.1 | [0206](https://rustsec.org/advisories/RUSTSEC-2026-0206.html): maintenance status. |
| `ttf-parser` 0.25.1 | [0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html): maintenance status. |
| `unsafe-libyaml` 0.2.11 | [0449](https://rustsec.org/advisories/RUSTSEC-2024-0449.html): maintenance status; current YAML support needs a migration assessment. |
| `zbus_names` 4.3.4, `zvariant` 5.15.0, `zvariant_derive` 5.15.0 | [0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html), [0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html), [0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html): maintenance/consolidation into a newer upstream release. |

No dependency version was upgraded as part of this review. Store support promotes
already locked/cached Windows bindings to optional direct dependencies; no new
package archive was downloaded. No first-import scan or historical clean-dependency
certification is claimed for existing packages.

## Store feature and readable source

The owner subsequently requested Store licensing: three permanent free themes,
ten unlocked by a durable add-on, and no trial. See [licensing](LICENSING.md) for
the implementation, external build configuration and outstanding live checks.
Only source compilation was authorized for this feature, not another test run.
Final Windows Debug builds completed without warnings for both variants:

```text
cargo build --locked --offline --bins
cargo build --locked --offline --bins --features store-licensing
```

The Store build read its configuration from the owner's external Obsidian file.
Both commands compiled application binaries from the current source. No package
was reconstructed from an installed executable or an older release.

The owner also requested a complete readability pass. Module summaries, method
contracts, parameter explanations and comments on non-obvious branches were added.
VibeWare headers follow the owner's `SOURCE-CODE-HEADER.md` template, recording
known creation/modification scope and human direction without inventing prior
review or release evidence. The README uses the matching footer template.

The existing `open-code-here.bat` editor helper contains a YAi! AGPL notice.
That notice was preserved, not relabeled MIT. The helper is separate from the
Rust application; its inclusion in any source redistribution needs separate
license consideration. No third-party notices were replaced by VibeWare headers.

The owner subsequently requested a Store MSIX. Release x64 package version 1.0.0.0
was generated from current source with licensing enabled, and MakeAppx validation
succeeded. Static package inspection confirmed the matching identity, clean
defaults, included notices and compiled feature. This does not validate Store
account behavior, purchase/restore UI or dock operation.

No commit, push, installation, Store submission or published release was performed.

## Startup follow-up

The owner reported that the installed Debug app did not open. Source inspection
found that the MSIX launcher redirects standard handles, while automatic terminal
onboarding returned false for redirected input and ended startup successfully
before any GUI/tray was created. Windows now uses a persistent `is_first_run`
graphical About welcome. Its first presentation is centered on the primary work
area, and completion is saved only after native placement succeeds.

The tray's left-click/keyboard selection now opens About directly. The obsolete
native information message and its always-false modal guard were removed. The
right-click context menu remains available.

The authorized About-only check exposed a second startup issue: before winit has
created the native window, Slint 1.13.1 can report `HandleError::NotSupported`
through its adapter fallback, rather than `Unavailable`. The bounded startup
retry now covers both results. The owner took over further GUI verification.
The large VibeWare information panels were removed from About and Configuration
at the owner's request; the small initiative badge and legal links remain.
