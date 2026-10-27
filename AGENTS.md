<!-- VBWR B

Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz

VibeWare: Human intent, AI execution, and plenty of tokens
Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto

Modified with AI: OpenAI Codex; owner-confirmed ten default pages and readability; 2026-10-27
Human guidance: Umberto Giacobbi; AGENTS.md and owner-directed implementation/documentation
Evidence: docs/CODE-REVIEW.md; current working-tree changes

Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.

VBWR E -->

# Working on this project

## Keep it simple

- Talk to me in Italian. Write repository docs in English.
- Use short sentences and normal words. Explain what someone needs to do. Skip repeated warnings, formal labels and image hashes in everyday docs.
- Support only the current settings and startup formats. Do not add migrations or fallback behavior for older versions. Keep backups, atomic writes and device safety checks.
- Write public copy as me: Umberto Giacobbi. Use "I", not "we". This is my weekend project. VibeWare is my initiative, not the app's author or publisher.
- Keep my name, the original logo and MIT notices. See [branding](docs/BRANDING.md).
- Do not copy code or art from the AGPL D200 project. Credit protocol references.

## Ask before tests and releases

- Do not run software tests, diagnostics or dock trials unless I explicitly authorize that specific check. Earlier permission is not permission for a new run.
- Do not start vendor software, change drivers or flash firmware without asking and arranging for me to watch the device.
- Publish a GitHub release only when I ask for that release. A commit, push or tag is not permission. Keep release runs manual and drafts for my review.
- Say what was actually checked. A build is not a device test; a USB write is not proof that the screen changed.
- I own and have tested only a D200H. Other models are unverified. Invite their owners to try the source and report the result. Some features remain untested even on my D200H.

## Keep the controls working

- Keep ten pages in this order: Windows / media, Utility, Codex, VS Code, Spotify, Word, PowerPoint, Excel, OBS Creator, Themes. Keep thirteen themes and both pressed/released images.
- NEXT PAGE stays at physical index 4. It skips disabled pages and the page index.
- The wide display at index 13 opens the page index from every page, even an inactive one. Pause automatic page switching while the index is open. Index choices open enabled pages directly.
- Themes stays last and offers the twelve themes other than the active one.
- At brightness 0/100, disable the corresponding minus/plus key and show UNAVAILABLE.
- Build inactive images with the existing renderer. Do not add separate disabled PNG sets.
- With app detection on, use foreground executable names to select pages and disable inactive actions. Do not read window titles or global keyboard input.
- Page checkboxes save each page's enabled flag without deleting shortcuts. With detection off, enabled pages work without inactive overlays or automatic switching.
- Inject only configured shortcuts, after a D200H consumer-interface button release. Never capture global keyboard input.
- Keep the Rust controller portable. Windows x64 and ARM64 are the first distribution targets; report builds and physical checks separately.
