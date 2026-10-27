<!-- VBWR B

Project: UlanziDeck
Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
Creator: Umberto Giacobbi | https://umbertogiacobbi.biz

VibeWare: Human intent, AI execution, and plenty of tokens
Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto

Modified with AI: OpenAI Codex; documentation and VibeWare attribution; 2026-10-27
Human guidance: Umberto Giacobbi; AGENTS.md; readability and VibeWare template request
Evidence: docs/CODE-REVIEW.md and current working-tree changes

Copyright (c) 2026 Umberto Giacobbi
SPDX-License-Identifier: MIT
License: MIT - see LICENSE
Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.

VBWR E -->

# OBS Creator

A page for recording with OBS: start, stop, pause, resume, four scenes, and microphone/desktop audio buttons. New settings include it by default. It works with all thirteen themes.

[Quick-start PDF](../../docs/obs-creator-quick-start.pdf)

![OBS Creator buttons in the Blade Runner theme](preview.png)

*Artwork preview, not a dock photo. The clock is illustrated.*

This page sends hotkeys, not commands over an OBS connection. It shows no live recording or mute status. I have not yet tried it with the dock and OBS.

## Add the page

Existing settings need the page added once. Close UlanziDeck and find the `settings.json` it uses. Start the app once first if the file does not exist.
2. From the repository folder, run this in PowerShell 7 with your actual settings path:

   ```powershell
   .\presets\obs-creator\install-preset.ps1 -SettingsPath 'C:\path\to\UlanziDeck\settings.json'
   ```

   The script makes a backup and prints its path. It keeps your other pages and shortcuts, and adds OBS Creator before Themes. It stops on invalid settings or duplicate page names. To replace an existing OBS Creator page, add `-Replace`; this also replaces that page's custom shortcuts.

3. Start the updated app. Open **About → Configure**, keep app detection on and turn automatic page switching off. Choose **Save + Restart**. This keeps your manual page choice while the app checks which program is in front.
4. Press **NEXT PAGE** until OBS Creator appears. Its actions show UNAVAILABLE unless OBS Studio (`obs64.exe`) is in front. With app detection off, the actions stay available.

To undo this, close the app and restore the printed backup. Automatic switching is saved in `settings.json` under `runtime`; check it in Configure after restoring the file.

`page.json` contains one page. Do not use it to replace your whole settings file. The installer does not start the app or change `applications.yaml`.

## Set up OBS

Create these scenes:

| Name | What to show |
| --- | --- |
| `01 CAMERA` | Your webcam. |
| `02 SCREEN` | The app or screen you want to record. |
| `03 CAM+SCREEN` | Screen plus a small webcam view. |
| `04 BREAK` | A "Back in a moment" screen. |

Use **Add Existing** when reusing a camera or screen source. In **Settings → Audio**, choose your microphone and desktop output once. Capturing the same audio twice can cause an echo.

Set **Settings → Advanced → Hotkeys → Hotkey Focus Behavior** to **Never disable hotkeys**. Leave **Studio Mode off** for direct scene changes. Pause needs a separate recording encoder, not **Same as stream** or **Use stream encoder**.

In **Settings → Hotkeys**, assign these keys. Every shortcut starts with **Ctrl + Shift + Alt**. They are my choices, not OBS defaults; remove conflicts.

| Dock button | Last key | OBS action |
| --- | --- | --- |
| START REC | `1` | Start Recording |
| STOP REC | `2` | Stop Recording |
| PAUSE | `3` | Pause Recording |
| RESUME | `4` | Unpause Recording |
| CAMERA | `5` | Switch to 01 CAMERA |
| SCREEN | `6` | Switch to 02 SCREEN |
| CAM+SCREEN | `7` | Switch to 03 CAM+SCREEN |
| BREAK | `8` | Switch to 04 BREAK |
| MIC ON | `9` | Microphone source: Unmute |
| MIC OFF | `0` | Microphone source: Mute |
| DESKTOP ON | `F5` | Desktop audio source: Unmute |
| DESKTOP OFF | `F7` | Desktop audio source: Mute |

Assign mute/unmute to the audio sources you actually use. NEXT PAGE and the clock need no OBS hotkeys.

BREAK only changes the scene. It does not pause or mute anything. For a private break, also pause or mute, and check OBS.

Before a full recording, make a short clip and play it back. Check the picture, microphone, scene changes, pause and mute.

[OBS quick start](https://obsproject.com/kb/quick-start-guide) · [Recording setup](https://obsproject.com/kb/standard-recording-output-guide)

## Rebuild the pictures and PDF

Build the current source as described in [development](../../docs/DEVELOPMENT.md). Use that executable for the export.

To recreate the pictures, use a fresh temporary settings file. These commands create files without opening the dock or sending shortcuts. You also need Python, Pillow and ReportLab.

```powershell
New-Item -ItemType Directory -Force tmp/obs-creator | Out-Null
.\target\debug\ulanzi-dock-vibeware.exe init --config tmp/obs-creator/settings.json
.\target\release\ulanzi-dock-vibeware.exe render --config tmp/obs-creator/settings.json --output tmp/obs-creator/artwork --page 9
python scripts/tools.py obs-guide
```

`init` will not overwrite an existing file. Fresh settings already include OBS Creator, so do not run the preset installer on that file. OBS Creator is page 9 and Themes is page 10. The export makes both button states in all thirteen themes. The builder writes `preview.png` here and the PDF in `docs/`.

See [platform coverage](../../docs/PLATFORMS.md) for build and physical-check evidence. These pictures do not show live OBS status.

I made this as [Umberto Giacobbi](https://umbertogiacobbi.biz). **VibeWare is Human intent, AI execution, and plenty of tokens.** [Manifesto](https://umbertogiacobbi.biz/vibeware/manifesto).
