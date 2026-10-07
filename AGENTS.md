<!-- VBWR B
Project: UlanziDock VibeWare version
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
VibeWare is Human intent. AI implementation. Accountable human review.
Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
AI Tooling: May include OpenAI Codex, GitHub Copilot and AI-assisted CI/CD pipelines.
AI Versions: Tools and models may vary by contributor and execution environment.
AI Traceability: Refer to Git history and CI/CD logs for recorded provenance.
Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
VBWR E -->

# Agent guidelines

- Keep user discussion in Italian and repository documentation in English.
- Preserve Umberto Giacobbi's VibeWare identity, original logo bytes, and the MIT source header on source/configuration files that support comments.
- Do not copy code or artwork from the AGPL upstream D200 project. Record protocol observations and credit references in documentation.
- Distinguish USB write completion, device acceptance, visible display output, and input events.
- Do not automatically launch software tests, hardware trials, or diagnostic test sessions. Run them only when the owner explicitly requests the specific test or authorizes the proposed run. Ask for authorization before any otherwise unrequested test; approval for an earlier test is not blanket approval for later tests.
- Prepare or publish a GitHub release only on the owner's explicit request for that release. Commits, pushes, and tags do not authorize release publication. Keep the release workflow manual and default to a draft for review.
- Never read global keyboard input. The CLI may inject only the configured shortcuts after a D200H consumer-interface button release.
- Do not automatically launch vendor processes, alter drivers, flash firmware, or run hardware tests without coordinating the physical observation with the owner.
- Keep all six pages, thirteen themes, and released/pressed artwork in sync. Navigation stays on physical key 4; key 13 shows the firmware clock by default or the optional runtime theme/page/availability status panel, with no physical-press action. Page five selects among the twelve themes other than the active one.
- Derive inactive application faces from the existing renderer at runtime; do not distribute additional disabled icon sets. On Windows, Codex, VS Code, and Spotify pages use foreground process metadata and suppress inactive actions. NEXT PAGE remains usable. Keep global keyboard input and window titles out of foreground detection.
- Windows x64 and ARM64 are the initial distribution targets. Keep source portable and state what platforms were actually built and physically tested.
