/* VBWR B
 *
 * Project: UlanziDeck
 * Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
 * Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
 *
 * VibeWare: Human intent, AI execution, and plenty of tokens
 * Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
 *
 * Modified with AI: OpenAI Codex; explicit Linux shortcut presets; 2026-11-02
 * Human guidance: Umberto Giacobbi; installed Linux application selection
 * Evidence: current source
 *
 * Copyright (c) 2026 Umberto Giacobbi
 * SPDX-License-Identifier: MIT
 * License: MIT - see LICENSE
 * Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
 *
 * VBWR E */

//! Suggested keys applied only on an explicit user request. These do not send input.
//! Keep the existing artwork and both reserved physical controls unchanged.
//! Sources and the unassigned keys are described in docs/PLATFORMS.md.

/// One suggested replacement for a regular physical key.
#[derive(Clone, Copy)]
pub struct Shortcut {
    pub key: usize,
    pub label: &'static str,
    pub action: &'static str,
    pub description: &'static str,
}

const fn shortcut(
    key: usize,
    label: &'static str,
    action: &'static str,
    description: &'static str,
) -> Shortcut {
    Shortcut { key, label, action, description }
}

// Microsoft documents a Linux-specific Format Document binding. VSCodium uses
// the same suggested defaults; user keymaps can override them in the editor.
const CODE: &[Shortcut] = &[
    shortcut(0, "QUICK OPEN", "hotkey:ctrl+p", "Find a file in the focused editor."),
    shortcut(1, "COMMANDS", "hotkey:ctrl+shift+p", "Open the editor command palette."),
    shortcut(2, "TERMINAL", "hotkey:ctrl+grave", "Show or hide the integrated terminal."),
    shortcut(3, "SEARCH", "hotkey:ctrl+shift+f", "Search across the current workspace."),
    shortcut(5, "EXPLORER", "hotkey:ctrl+shift+e", "Show the editor file tree."),
    shortcut(6, "SOURCE CTRL", "hotkey:ctrl+shift+g", "Open source control in the editor."),
    shortcut(7, "DEBUG", "hotkey:ctrl+shift+d", "Open the Run and Debug view."),
    shortcut(8, "EXTENSIONS", "hotkey:ctrl+shift+x", "Open installed and available extensions."),
    shortcut(9, "SAVE", "hotkey:ctrl+s", "Save the current editor document."),
    shortcut(10, "NEW FILE", "hotkey:ctrl+n", "Create an untitled editor document."),
    shortcut(11, "FORMAT", "hotkey:ctrl+shift+i", "Format the document using the Linux editor binding."),
    shortcut(12, "PROBLEMS", "hotkey:ctrl+shift+m", "Open editor errors and warnings."),
];

// Spotify's public shortcut table does not identify Linux bindings. Do not
// present its Windows shortcuts as verified Linux defaults. Media keys follow
// desktop media routing and can reach a player other than Spotify.
const SPOTIFY: &[Shortcut] = &[
    shortcut(0, "PLAY / PAUSE", "media:play-pause", "Send the desktop media play/pause key. The desktop chooses the player."),
    shortcut(1, "PREVIOUS", "media:previous", "Send the desktop previous-track key. The desktop chooses the player."),
    shortcut(2, "NEXT TRACK", "media:next", "Send the desktop next-track key. The desktop chooses the player."),
    shortcut(3, "UNASSIGNED", "none", "Set a shuffle shortcut in settings.json after checking Spotify's shortcut list."),
    shortcut(5, "UNASSIGNED", "none", "Set a repeat shortcut in settings.json after checking Spotify's shortcut list."),
    shortcut(6, "UNASSIGNED", "none", "Set a search shortcut in settings.json after checking Spotify's shortcut list."),
    shortcut(7, "UNASSIGNED", "none", "Set a library shortcut in settings.json after checking Spotify's shortcut list."),
    shortcut(8, "UNASSIGNED", "none", "Set a like-song shortcut in settings.json after checking Spotify's shortcut list."),
    shortcut(9, "UNASSIGNED", "none", "Set a queue shortcut in settings.json after checking Spotify's shortcut list."),
    shortcut(10, "UNASSIGNED", "none", "Set a now-playing shortcut in settings.json after checking Spotify's shortcut list."),
    shortcut(11, "UNASSIGNED", "none", "Set a liked-songs shortcut in settings.json after checking Spotify's shortcut list."),
    shortcut(12, "UNASSIGNED", "none", "Set a home shortcut in settings.json after checking Spotify's shortcut list."),
];

// These are the project's existing OBS preset, not built-in OBS hotkeys.
// Users must assign each chord under OBS Settings > Hotkeys before use.
const OBS: &[Shortcut] = &[
    shortcut(0, "START REC", "hotkey:ctrl+shift+alt+1", "Assign this chord to Start Recording in OBS. No recording state is shown."),
    shortcut(1, "STOP REC", "hotkey:ctrl+shift+alt+2", "Assign this chord to Stop Recording in OBS. Check the recording in OBS."),
    shortcut(2, "PAUSE", "hotkey:ctrl+shift+alt+3", "Assign this chord to Pause Recording in OBS. A separate recording encoder is needed."),
    shortcut(3, "RESUME", "hotkey:ctrl+shift+alt+4", "Assign this chord to Unpause Recording in OBS. No recording state is shown."),
    shortcut(5, "CAMERA", "hotkey:ctrl+shift+alt+5", "Create scene 01 CAMERA and assign this chord in OBS. Keep Studio Mode off."),
    shortcut(6, "SCREEN", "hotkey:ctrl+shift+alt+6", "Create scene 02 SCREEN and assign this chord in OBS. Keep Studio Mode off."),
    shortcut(7, "CAM+SCREEN", "hotkey:ctrl+shift+alt+7", "Create scene 03 CAM+SCREEN and assign this chord in OBS. Keep Studio Mode off."),
    shortcut(8, "BREAK", "hotkey:ctrl+shift+alt+8", "Create scene 04 BREAK and assign this chord in OBS. This does not pause or mute."),
    shortcut(9, "MIC ON", "hotkey:ctrl+shift+alt+9", "Assign this chord to unmute the microphone source in OBS. No microphone state is shown."),
    shortcut(10, "MIC OFF", "hotkey:ctrl+shift+alt+0", "Assign this chord to mute the microphone source in OBS. No microphone state is shown."),
    shortcut(11, "DESKTOP ON", "hotkey:ctrl+shift+alt+f5", "Assign this chord to unmute the desktop audio source in OBS. System volume is unchanged."),
    shortcut(12, "DESKTOP OFF", "hotkey:ctrl+shift+alt+f7", "Assign this chord to mute the desktop audio source in OBS. System volume is unchanged."),
];

// General LibreOffice bindings are shared by all three modules. F12 is not
// Save As in Writer: it toggles an ordered list, so use Ctrl+Shift+S instead.
const WRITER: &[Shortcut] = &[
    shortcut(0, "SAVE", "hotkey:ctrl+s", "Save the Writer document."),
    shortcut(1, "OPEN", "hotkey:ctrl+o", "Choose a document to open in LibreOffice."),
    shortcut(2, "NEW FILE", "hotkey:ctrl+n", "Create a Writer document."),
    shortcut(3, "PRINT", "hotkey:ctrl+p", "Open print options for the Writer document."),
    shortcut(5, "UNDO", "hotkey:ctrl+z", "Undo the last Writer edit."),
    shortcut(6, "REDO", "hotkey:ctrl+y", "Redo the last undone Writer edit."),
    shortcut(7, "FIND", "hotkey:ctrl+f", "Find text in Writer."),
    shortcut(8, "BOLD", "hotkey:ctrl+b", "Toggle bold text in Writer."),
    shortcut(9, "ITALIC", "hotkey:ctrl+i", "Toggle italic text in Writer."),
    shortcut(10, "UNDERLINE", "hotkey:ctrl+u", "Toggle underlined text in Writer."),
    shortcut(11, "SPELLING", "hotkey:f7", "Open Writer spelling tools."),
    shortcut(12, "SAVE AS", "hotkey:ctrl+shift+s", "Choose another name, location or format for the Writer document."),
];

const IMPRESS: &[Shortcut] = &[
    shortcut(0, "SAVE", "hotkey:ctrl+s", "Save the Impress presentation."),
    shortcut(1, "OPEN", "hotkey:ctrl+o", "Choose a document to open in LibreOffice."),
    shortcut(2, "NEW FILE", "hotkey:ctrl+n", "Create an Impress presentation."),
    shortcut(3, "PRINT", "hotkey:ctrl+p", "Open print options for the Impress presentation."),
    shortcut(5, "UNDO", "hotkey:ctrl+z", "Undo the last Impress edit."),
    shortcut(6, "REDO", "hotkey:ctrl+y", "Redo the last undone Impress edit."),
    shortcut(7, "FIND", "hotkey:ctrl+f", "Find text in Impress."),
    shortcut(8, "START SHOW", "hotkey:f5", "Present from the beginning in Impress."),
    shortcut(9, "CURRENT SLIDE", "hotkey:shift+f5", "Present from the current Impress slide."),
    shortcut(10, "NEW SLIDE", "hotkey:ctrl+m", "Insert a slide after the selected Impress slide."),
    shortcut(11, "UNASSIGNED", "none", "Set a duplicate-slide shortcut in settings.json after assigning it in Impress."),
    shortcut(12, "END SHOW", "hotkey:escape", "End an Impress presentation or leave the current editing mode."),
];

const CALC: &[Shortcut] = &[
    shortcut(0, "SAVE", "hotkey:ctrl+s", "Save the Calc spreadsheet."),
    shortcut(1, "OPEN", "hotkey:ctrl+o", "Choose a document to open in LibreOffice."),
    shortcut(2, "NEW FILE", "hotkey:ctrl+n", "Create a Calc spreadsheet."),
    shortcut(3, "PRINT", "hotkey:ctrl+p", "Open print options for the Calc spreadsheet."),
    shortcut(5, "UNDO", "hotkey:ctrl+z", "Undo the last Calc edit."),
    shortcut(6, "REDO", "hotkey:ctrl+y", "Redo the last undone Calc edit."),
    shortcut(7, "FIND", "hotkey:ctrl+f", "Find text or values in Calc."),
    shortcut(8, "UNASSIGNED", "none", "Set an AutoSum shortcut in settings.json after assigning it in Calc."),
    shortcut(9, "FORMAT CELLS", "hotkey:ctrl+1", "Choose formatting for the selected Calc cells."),
    shortcut(10, "INSERT DATE", "hotkey:ctrl+;", "Insert today's date as a fixed value in the Calc cell."),
    shortcut(11, "EDIT CELL", "hotkey:f2", "Edit the current Calc cell."),
    shortcut(12, "FILTER", "hotkey:ctrl+shift+l", "Toggle Calc AutoFilter for the selected range."),
];

/// Return suggested keys for one known Linux profile; unknown IDs have no preset.
pub fn for_profile(id: &str) -> &'static [Shortcut] {
    match id {
        "code" | "code-insiders" | "codium" => CODE,
        "spotify" => SPOTIFY,
        "obs" => OBS,
        "libreoffice-writer" => WRITER,
        "libreoffice-impress" => IMPRESS,
        "libreoffice-calc" => CALC,
        _ => &[],
    }
}
