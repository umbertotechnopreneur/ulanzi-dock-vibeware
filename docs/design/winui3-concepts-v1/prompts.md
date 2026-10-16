# UlanziDock WinUI 3 concept prompts

Generated with the built-in image_gen tool. These are proposed interfaces, not screenshots of a running WinUI implementation. Twelve screens: six functions in dark and light appearance. Hue and saturation affect dock artwork independently of app appearance. The reference is existing project artwork, not a live device capture.

## Shared design specification

Use case: ui-mockup.
Asset type: one high-fidelity desktop application screenshot concept, landscape 16:10, generous resolution and readable text.
Product: UlanziDock, a Windows desktop editor for a D200H macro-control dock, by VibeWare. Deliver a single full app window, front-on and edge-to-edge, with the Windows title bar fully visible. No surrounding desk, no perspective, no browser, no collage, no marketing presentation.
Design direction: exceptionally beautiful, polished, sophisticated Windows 11 WinUI 3 / Fluent 2 app. Segoe UI Variable typography, restrained Mica tint on title bar and left NavigationView, precise 8px spacing rhythm, softly rounded content cards, fine separators, tasteful subtle shadows and substantial whitespace. Functional, credible, implementation-ready controls. Calm premium creative-tool feel. Color is expressive in the dock buttons, theme thumbnails and controls; the app shell stays elegant and quiet. Primary actions violet, secondary preview actions teal, import accents amber, destructive actions coral; never make every button blue.
Common shell on ALL twelve screens: compact pixel-art four-key dock emblem and exact title "UlanziDock" at top left; a small subtle "Concept" chip; standard minimize/maximize/close at top right. Fixed left navigation approximately 208 px wide with these exact English labels in this order: "Key editor", "Pages", "Applications", "Themes", "Presets", "Device". Small footer text "VibeWare" and Settings gear. Main panel uses a crisp large page title, one short helpful subtitle, and a compact command bar.
Dock geometry, whenever displayed: exactly five equal square keys in row one, five in row two, three in row three followed by ONE wide clock/status display spanning the final TWO columns. Thus 13 square keys plus one double-width status display; NEVER sixteen keys or a 3x3 deck. Button artwork has clear colored pictograms and short readable labels, inspired by the referenced source art, with controlled violet/coral/teal/amber/green treatments instead of all-blue buttons. Physical key 5 (last key in first row) is always "NEXT PAGE"; clock stays separate. Key editor is a top-down digital preview, not a photorealistic device photograph.
Hue/Saturation feature: hue is a single simple rainbow slider labelled "Hue" with degree value; saturation is a second simple slider labelled "Saturation" with percent value. Add a compact scope choice "Key", "Page", "All pages" where appropriate and "Reset colors". Colors are adjustable separately from the app's Dark/Light appearance. Preserve text legibility and contrast in the preview.
All UI text in clear natural English. Large enough labels to be legible, short copy, no lorem ipsum, no fake charts, no pricing, no AI chat widget, no stock photos, no watermark. The screen is a design proposal, not evidence of live device state.

## 01-key-editor-dark

Appearance: dark.

Three-column workspace beyond the nav: slim page list, large central dock preview, right inspector about 290 px. Header "Key editor"; subtitle "Make every key yours." A "Windows / media" page is selected. Header command bar has "Undo", "Preview", and a violet "Apply changes". Central dock uses PLAY / PAUSE, PREVIOUS, NEXT TRACK, MUTE, NEXT PAGE, VOL DOWN, VOL UP, FILE EXPLORER, DESKTOP, SETTINGS, SCREENSHOT, TASK VIEW, BROWSER and wide clock "14:32". PLAY / PAUSE is selected with a fine violet border, no giant glow. Right inspector "Play / Pause" with tabs "Action" and "Appearance"; Appearance selected. A normal/pressed mini preview, a concise "Label" input "PLAY / PAUSE", an "Icon" selector, prominent rainbow "Hue" slider "+24°", "Saturation" slider "110%", scope "Key" selected and "Reset colors". At bottom a small unobtrusive unsaved-changes status. This is the visual anchor for the entire 12-screen suite.

## 02-key-editor-light

Appearance: light.

Match the key-editor anchor's entire geometry, navigation, deck, labels, and right-hand inspector, transformed to a luminous warm-neutral LIGHT appearance. White elevated cards, soft silver Mica sidebar, ink text, violet and teal accents. Keep the dock's colored key art attractive in light mode. Page "Windows / media", PLAY / PAUSE selected, Appearance tab selected with "Hue" "+24°", "Saturation" "110%", scope "Key", "Reset colors". Command bar "Undo", "Preview", "Apply changes". Normal and pressed previews side by side. Make this an exact coherent light-mode counterpart of the dark editor, not a redesign.

## 03-pages-dark

Appearance: dark.

Navigation "Pages" selected. Title "Pages", subtitle "Build a layout for every workflow." Toolbar "New page" (violet), "Duplicate", search. Main content a beautifully spaced sortable page list on the left with drag handles, colored small page icons, enabled switches, names and small key counts: "Windows / media", "Utility", "Codex", "VS Code", "Spotify", "Word", "PowerPoint", "Excel", "Themes". Windows / media selected. Right broad card is a real 5/5/3+wide miniature dock preview for the selected page. Below it two elegant property cards: "Page name" input, "Include in navigation" enabled, and a "Navigation" card explaining "NEXT PAGE skips disabled pages." Show a small lock badge on NEXT PAGE and the clock, with the caption "Reserved controls". Useful page management rather than a theme gallery.

## 04-pages-light

Appearance: light.

Navigation "Pages" selected. Exact same layout and content as the dark Pages screen, using luminous neutral LIGHT Fluent surfaces, white cards, silver tinted sidebar and ink typography. Header "Pages", subtitle "Build a layout for every workflow.", commands "New page", "Duplicate", search. Sortable pages list with nine named pages, toggles, drag handles; Windows / media selected. Selected-page deck preview with proper 5/5/3+wide geometry, page-name field, "Include in navigation" switch, compact "Reserved controls" indication for NEXT PAGE and the clock. Violet primary action, teal enabled controls, individually colorful thumbnails. Clear premium information hierarchy.

## 05-applications-dark

Appearance: dark.

Navigation "Applications" selected. Title "Applications", subtitle "The right page, at the right time." Header small "Automatic switching" switch enabled. Main area two columns: beautiful compact rule cards for "Codex", "VS Code", "Spotify", "Word", "PowerPoint", "Excel" with distinct colored application glyphs, an executable filename, and destination-page tag. VS Code rule selected. Right inspector card "VS Code", input "Executable" showing "Code.exe", chip "Code - Insiders.exe", dropdown "Switch to page" = "VS Code", enabled "Detect availability" and "Switch automatically", dropdown "When unavailable" = "Show disabled keys". A small accurate caption "Matches process names, not window titles." Bottom a graceful thin connection/routing diagram from Code.exe pill to VS Code page thumbnail, and "Add application" action. No fake live monitoring or process spy charts.

## 06-applications-light

Appearance: light.

Exact same Applications workspace as its dark-mode counterpart, in an exceptionally polished LIGHT WinUI 3 appearance. Title "Applications", subtitle "The right page, at the right time.", Automatic switching on. Two-column layout: color-coded app rule cards Codex, VS Code selected, Spotify, Word, PowerPoint, Excel; inspector for Code.exe and Code - Insiders.exe, target page "VS Code", toggles "Detect availability" and "Switch automatically", unavailable dropdown "Show disabled keys", short caption "Matches process names, not window titles." Violet selection edge, teal toggles, bright white cards, warm gray canvas, extremely crisp readable text, small routing preview, "Add application".

## 07-themes-dark

Appearance: dark.

Navigation "Themes" selected. Title "Themes", subtitle "Change the mood. Keep your layout." This is the showcase COLOR customization screen. Main left area elegant thumbnail grid of dock-style themes with visible names: "Dark classic", "Dark abstract" selected, "Light classic", "Light abstract", "Cyberpunk neon", "Manga ink", "Solarpunk", "Art deco", "Memphis". Thumbnails must look genuinely different, not nine purple clones. Right spacious "Color studio" panel with a medium 5/5/3+wide dock preview showing the selected colorful theme. Prominent single rainbow Hue slider "+42°", Saturation slider "125%", small before/after key swatches, scope segmented control "Key / Page / All pages" with "All pages" selected, "Reset colors", "Apply theme" violet. A small separate top-right app appearance control "System / Light / Dark" with Dark selected. Make hues vivid and sophisticated, not neon UI chrome.

## 08-themes-light

Appearance: light.

Navigation "Themes" selected, exact structural counterpart to the dark Themes screen. Premium LIGHT Windows Fluent appearance: creamy light-gray canvas, white panels, subtle separators, violet selection, colorful thumbnail artwork. Title "Themes", subtitle "Change the mood. Keep your layout." Gallery named Dark classic, Dark abstract selected, Light classic, Light abstract, Cyberpunk neon, Manga ink, Solarpunk, Art deco, Memphis, all distinctly styled. Right "Color studio" panel with correct 5/5/3+wide deck, rainbow "Hue" "+42°" slider, "Saturation" "125%" slider, before/after tiles, scope "All pages" selected, "Reset colors", "Apply theme". Separate top-right app appearance choice "System / Light / Dark" with Light selected. The dock artwork may remain dark while the application shell is light.

## 09-presets-dark

Appearance: dark.

Navigation "Presets" selected. Title "Presets", subtitle "Start with a workflow. Make it yours." Header controls "Import preset" amber-outline and "Export current". Under a search bar and filter chips "All", "Productivity", "Creator", "My presets", a curated two-column gallery of four generous cards: "Everyday essentials", "OBS Creator", "Developer", "Office focus". Each includes a colorful correct-geometry miniature dock screenshot, a short readable description, page-count tag and a "Preview" action. OBS Creator selected with inspector panel "OBS Creator", description "Scenes, recording and audio shortcuts.", badges "1 page" and "13 themes". Clearly label "Requires OBS hotkeys", then short entries "Recording controls", "Scene shortcuts", "Microphone and desktop audio". Bottom action "Add to my pages" violet and a small option "Keep existing shortcuts" on. This is a local preset library; no marketplace, pricing, downloads counters, or community fiction.

## 10-presets-light

Appearance: light.

Same Presets design as the dark counterpart, in very beautiful LIGHT WinUI 3 surfaces. Title "Presets", subtitle "Start with a workflow. Make it yours.", "Import preset" warm amber-outline, "Export current". Filter chips All, Productivity, Creator, My presets. Four excellent preview cards Everyday essentials, OBS Creator selected, Developer, Office focus, each using a different colored dock miniature. Right selected preset details: "OBS Creator", "Scenes, recording and audio shortcuts.", "1 page", "13 themes", "Requires OBS hotkeys", recording/scene/audio feature list, "Keep existing shortcuts" enabled, violet "Add to my pages". Precise spacing and white cards over gray Mica, no marketplace or prices.

## 11-device-dark

Appearance: dark.

Navigation "Device" selected. Title "Device", subtitle "Your dock, ready when you are." Broad hero card with an elegant front-on matte-black D200H silhouette showing the correct 13 square keys plus wide clock, with vividly colored key art and simple caption "Ulanzi D200H". Illustrative green "Connected" status pill, "USB" label. Below and right, restrained WinUI settings cards: "Brightness" slider "80%", "Start with Windows" toggle on, "Run in the background" on, "Default page" dropdown "Windows / media", "App appearance" segmented System/Light/Dark with Dark selected. Footer region "Connection" with short "Reconnect automatically" on, buttons "Restart controller" secondary and "Open logs" subtle. A low-key settings summary, no made-up firmware upgrades, no fake performance graphs, no CPU metrics, no buttons to flash firmware. Device screenshot is explicitly a concept via common title chip.

## 12-device-light

Appearance: light.

Exact coherent LIGHT counterpart of Device screen. Navigation Device selected, title "Device", subtitle "Your dock, ready when you are." Beautiful white hero card containing front-on dark D200H with colorful 13 square keys and a wide clock, green illustrative "Connected" badge and USB label. White Fluent settings cards for Brightness "80%" slider, Start with Windows on, Run in the background on, Default page "Windows / media", App appearance System/Light/Dark with Light selected. Connection section Reconnect automatically on, Restart controller secondary, Open logs subtle. Airy warm-neutral canvas and soft silver Mica navigation. No firmware updater, no fake diagnostics charts, no extra devices.
