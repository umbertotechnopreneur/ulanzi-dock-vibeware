# VBWR B
# Project: UlanziDock VibeWare version
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
# VibeWare is Human intent, AI execution, and plenty of tokens
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
# AI Tooling: May include OpenAI Codex, GitHub Copilot and AI-assisted CI/CD pipelines.
# AI Versions: Tools and models may vary by contributor and execution environment.
# AI Traceability: Refer to Git history and CI/CD logs for recorded provenance.
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
# Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
# VBWR E

"""Build the three-page OBS guide and a physical-layout artwork preview.

Requires Pillow and ReportLab. Input is the controller's offline render output,
not a hardware screenshot. See presets/obs-creator/README.md for regeneration.
"""

import argparse
import json
from pathlib import Path

import reportlab
from PIL import Image, ImageDraw, ImageFont
from reportlab.lib import colors
from reportlab.lib.pagesizes import A4, landscape
from reportlab.lib.utils import ImageReader, simpleSplit
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.pdfgen import canvas

ROOT = Path(__file__).resolve().parents[1]
PACK = ROOT / "presets" / "obs-creator"
OUTPUT = ROOT / "docs" / "obs-creator-quick-start.pdf"
W, H = landscape(A4)
INK = colors.HexColor("#18263a")
MUTED = colors.HexColor("#526477")
BLUE = colors.HexColor("#266fbb")
RULE = colors.HexColor("#dce3ec")
PANEL = colors.HexColor("#eff4f9")
GOLD = colors.HexColor("#f2b65b")
FONTS = Path(reportlab.__file__).parent / "fonts"
pdfmetrics.registerFont(TTFont("Body", str(FONTS / "Vera.ttf")))
pdfmetrics.registerFont(TTFont("Bold", str(ROOT / "assets/fonts/VeraBd.ttf")))
pdfmetrics.registerFont(pdfmetrics.Font("Mono", "Courier", "WinAnsiEncoding"))


def text(pdf, value, x, y, size=10, font="Body", color=INK):
    pdf.setFont(font, size)
    pdf.setFillColor(color)
    pdf.drawString(x, H - y, value)


def paragraph(pdf, value, x, y, width, size=10, leading=15, color=MUTED):
    lines = simpleSplit(value, "Body", size, width)
    for line in lines:
        text(pdf, line, x, y, size=size, color=color)
        y += leading
    if y > 534:
        raise ValueError(f"Body overflows footer: {value[:70]}")
    return y


def box(pdf, x, y, width, height, fill=PANEL):
    pdf.setFillColor(fill)
    pdf.roundRect(x, H - y - height, width, height, 9, fill=1, stroke=0)


def rule(pdf, y):
    pdf.setStrokeColor(RULE)
    pdf.setLineWidth(0.7)
    pdf.line(36, H - y, W - 36, H - y)


def frame(pdf, number, section, title, subtitle):
    text(pdf, "ULANZIDOCK / OBS CREATOR", 36, 30, 8.5, "Bold", BLUE)
    pdf.setFont("Mono", 8)
    pdf.setFillColor(MUTED)
    pdf.drawRightString(W - 36, H - 30, "14 OCT 2026 / v1")
    text(pdf, title, 36, 69, 26, "Bold")
    text(pdf, subtitle, 36, 94, 10.5, color=MUTED)
    rule(pdf, 110)
    rule(pdf, 543)
    text(pdf, "Umberto Giacobbi  /  MIT", 36, 562, 8)
    pdf.linkURL("https://umbertogiacobbi.biz", (36, H - 565, 200, H - 551))
    text(pdf, "VibeWare - Human intent, AI execution, and plenty of tokens", 36, 578, 7.5, color=MUTED)
    logo = ROOT / "assets/brand/vibeware-logo.png"
    pdf.drawImage(str(logo), W - 177, H - 579, 60, 29, preserveAspectRatio=True, mask="auto", anchor="c")
    pdf.linkURL("https://umbertogiacobbi.biz/vibeware/manifesto", (W - 180, H - 583, W - 112, H - 546))
    text(pdf, f"{section}  /  {number:02}", W - 104, 572, 8, "Mono", BLUE)


def preview(artwork):
    """Assemble original rendered tiles; the wide clock is clearly illustrative."""
    side, gap, margin, top = 196, 14, 24, 90
    sheet = Image.new("RGB", (1084, 754), "#111b2b")
    draw = ImageDraw.Draw(sheet)
    bold = ROOT / "assets/fonts/VeraBd.ttf"
    heading = ImageFont.truetype(str(bold), 28)
    small = ImageFont.truetype(str(bold), 15)
    draw.text((margin, 22), "OBS CREATOR", fill="#f5ba6e", font=heading)
    draw.text((margin, 59), "D200H  /  BLADE RUNNER  /  RELEASED BUTTON ARTWORK", fill="#b8c7da", font=small)
    for index in range(13):
        path = artwork / f"key-{index:02}-up.png"
        with Image.open(path) as tile:
            if tile.size != (side, side):
                raise ValueError(f"Unexpected tile size: {path}")
            x, y = margin + index % 5 * (side + gap), top + index // 5 * (side + gap)
            sheet.paste(tile.convert("RGB"), (x, y))
    x, y = margin + 3 * (side + gap), top + 2 * (side + gap)
    draw.rounded_rectangle((x, y, x + 2 * side + gap, y + side), radius=12, fill="#1d2d44")
    draw.text((x + 54, y + 59), "FIRMWARE CLOCK", fill="#c7d5e7", font=heading)
    draw.text((x + 58, y + 112), "ILLUSTRATION / NO OBS STATUS", fill="#93a9c3", font=small)
    draw.text((margin, 731), "Original rendered controls. Wide clock illustrated; no physical device capture.", fill="#9aadc4", font=small)
    path = PACK / "preview.png"
    sheet.save(path, optimize=True)
    return path


def cover(pdf, image_path):
    frame(pdf, 1, "THE DECK", "Your desk. Your little studio.",
          "An optional OBS Studio recording page for the Ulanzi D200H. Twelve clear actions, one take at a time.")
    pdf.drawImage(str(image_path), 36, H - 505, width=533, height=371, preserveAspectRatio=True)
    x, width = 593, 212
    text(pdf, "I built this for solo videos.", x, 146, 12, "Bold")
    paragraph(pdf, "Tutorials, screen demos and talking-head videos, without reaching for OBS every time.", x, 169, width)
    text(pdf, "01 / Control the take", x, 231, 11, "Bold", BLUE)
    paragraph(pdf, "Start, stop, pause and resume have separate buttons. A second START does not become STOP.", x, 253, width)
    text(pdf, "02 / Tell the story", x, 314, 11, "Bold", BLUE)
    paragraph(pdf, "CAMERA, SCREEN, CAM+SCREEN and BREAK switch to four scenes you create in OBS.", x, 336, width)
    text(pdf, "03 / Own the sound", x, 397, 11, "Bold", BLUE)
    paragraph(pdf, "MIC and DESKTOP each have ON/OFF buttons. Desktop controls affect OBS audio, not Windows volume.", x, 419, width)
    text(pdf, "Always check the OBS meters.", x, 486, 10, "Bold")
    paragraph(pdf, "The dock sends hotkeys; it receives no recording, mute or scene status from OBS.", x, 505, width, size=8.5, leading=12)
    text(pdf, "NEXT PAGE stays in place. The wide display keeps your configured clock or status panel.", 36, 524, 8.5, color=MUTED)
    pdf.showPage()


def setup(pdf):
    frame(pdf, 2, "SETUP", "Set it up once.",
          "Windows workflow. Use a build that includes the obs icon family; older releases reject this preset.")
    x, right, width = 36, 432, 373
    text(pdf, "1  Add the dock page", x, 145, 14, "Bold")
    paragraph(pdf, "Close UlanziDock from its tray menu. From the repository root, run this PowerShell 7 command, replacing the settings path with the file used by your controller:", x, 171, width)
    box(pdf, x, 218, width, 76)
    for i, line in enumerate([
        "pwsh -NoProfile -File",
        "  .\\presets\\obs-creator\\install-preset.ps1",
        "  -SettingsPath 'C:\\UlanziDock\\settings.json'",
        "Type as ONE line; the path above is an example.",
    ]):
        text(pdf, line, x + 12, 235 + i * 15, 7.7, "Mono")
    paragraph(pdf, "The installer adds OBS Creator before Themes and creates a backup. It keeps your other pages, shortcuts and theme. Start the updated controller with the same settings file.", x, 316, width)
    text(pdf, "2  Keep OBS controls within reach", x, 385, 14, "Bold")
    paragraph(pdf, "In UlanziDock Configure, turn automatic page switching OFF, then Save + Restart. Application detection may stay ON. Press NEXT PAGE until OBS Creator appears.", x, 411, width)
    paragraph(pdf, "In OBS Settings > Advanced > Hotkeys, choose Never disable hotkeys. Keep Studio Mode OFF so scene keys go directly to the recorded output.", x, 476, width, size=9.4, leading=14)
    text(pdf, "3  Build four OBS scenes", right, 145, 14, "Bold")
    scenes = [
        ("01 CAMERA", "Video Capture Device, filling the frame."),
        ("02 SCREEN", "Window Capture for your demo, or Display Capture when needed."),
        ("03 CAM+SCREEN", "Screen plus the same camera source in a corner. Use Add Existing."),
        ("04 BREAK", "A plain Color Source and Text: Back in a moment."),
    ]
    y = 165
    for title, detail in scenes:
        box(pdf, right, y, width, 64)
        text(pdf, title, right + 13, y + 21, 10.5, "Bold", BLUE)
        paragraph(pdf, detail, right + 13, y + 40, width - 26, size=9, leading=13)
        y += 72
    paragraph(pdf, "Select one microphone and one desktop-audio device in Settings > Audio. Reuse them across scenes; avoid capturing the same device twice.", right, 470, width, size=9.4, leading=14)
    text(pdf, "BREAK changes the picture. Press MIC OFF separately.", right, 524, 9, "Bold")
    pdf.showPage()


def bindings(pdf, page):
    frame(pdf, 3, "FIRST TAKE", "Bind the keys. Then press REC.",
          "In OBS Settings > Hotkeys, assign these combinations manually. They are custom bindings, not OBS defaults.")
    x, width, right = 36, 442, 505
    box(pdf, x, 128, width, 33, INK)
    text(pdf, "Every shortcut starts with CTRL + ALT + SHIFT", x + 12, 149, 10, "Bold", colors.white)
    labels = {
        0: "Start Recording", 1: "Stop Recording", 2: "Pause Recording", 3: "Unpause Recording",
        5: "01 CAMERA > Switch to scene", 6: "02 SCREEN > Switch to scene",
        7: "03 CAM+SCREEN > Switch to scene", 8: "04 BREAK > Switch to scene",
        9: "Mic/Aux > Unmute", 10: "Mic/Aux > Mute",
        11: "Desktop Audio > Unmute", 12: "Desktop Audio > Mute",
    }
    text(pdf, "BUTTON", x + 9, 182, 8, "Bold", BLUE)
    text(pdf, "KEY", x + 132, 182, 8, "Bold", BLUE)
    text(pdf, "OBS FIELD", x + 178, 182, 8, "Bold", BLUE)
    y = 193
    for row, (index, obs_field) in enumerate(labels.items()):
        key = page["keys"][index]
        if row % 2 == 0:
            box(pdf, x, y, width, 24)
        text(pdf, key["label"], x + 9, y + 16, 8.5, "Bold")
        text(pdf, key["action"].split("+")[-1].upper(), x + 132, y + 16, 9, "Mono")
        text(pdf, obs_field, x + 178, y + 16, 8.3)
        y += 25
    paragraph(pdf, "Use your actual audio-source names. Remove conflicting assignments. Keep start/stop and mute/unmute on separate hotkeys.", x, 510, width, size=8.6, leading=12)
    rw = W - 36 - right
    text(pdf, "A practical starting point", right, 145, 12, "Bold")
    paragraph(pdf, "Run Tools > Auto-Configuration Wizard for recording. For tutorials, try 1920 x 1080 at 30 fps, Simple output, High Quality and hardware encoding if available. Adjust for your computer.", right, 168, rw, size=9, leading=13)
    paragraph(pdf, "Record as MKV. When you need MP4, use File > Remux Recordings. For PAUSE, avoid Same as stream (or Use stream encoder in Advanced output).", right, 248, rw, size=9, leading=13)
    text(pdf, "Your first 60 seconds", right, 313, 12, "Bold")
    paragraph(pdf, "MIC ON > CAMERA > START REC. Speak, switch to SCREEN, then CAM+SCREEN. Try PAUSE and RESUME. Press STOP REC and wait for OBS to finish. Play the file back and check picture and sound.", right, 336, rw, size=9, leading=13)
    paragraph(pdf, "If a key does nothing: check its OBS binding, hotkey focus setting and dock page. Keep both apps at the same privilege level. Read recording and mute status in OBS.", right, 409, rw, size=8.6, leading=12)
    text(pdf, "Official references / checked 14 Oct 2026", right, 474, 8.5, "Bold", BLUE)
    links = [
        ("Quick start", "https://obsproject.com/kb/quick-start-guide"),
        ("Recording presets", "https://obsproject.com/kb/recording-encoder-presets-guide"),
        ("Recording / remux", "https://obsproject.com/kb/standard-recording-output-guide"),
        ("Scene workflow", "https://obsproject.com/kb/OBS-Studio-Overview"),
        ("Hotkey labels", "https://github.com/obsproject/obs-studio/blob/master/frontend/data/locale/en-US.ini"),
    ]
    for i, (label, url) in enumerate(links):
        lx = right + (i % 2) * 151
        ly = 491 + (i // 2) * 14
        text(pdf, label, lx, ly, 8, color=BLUE)
        pdf.linkURL(url, (lx, H - ly - 3, lx + 143, H - ly + 10))
    pdf.showPage()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artwork", type=Path, default=ROOT / "tmp/obs-creator/artwork/dark-abstract/page-9")
    args = parser.parse_args()
    page = json.loads((PACK / "page.json").read_text(encoding="utf-8-sig"))
    image_path = preview(args.artwork)
    pdf = canvas.Canvas(str(OUTPUT), pagesize=landscape(A4), pageCompression=1, invariant=1)
    pdf.setTitle("OBS Creator - UlanziDock quick start")
    pdf.setAuthor("Umberto Giacobbi")
    pdf.setSubject("Optional D200H recording preset: installation, OBS scenes and hotkeys")
    cover(pdf, image_path)
    setup(pdf)
    bindings(pdf, page)
    pdf.save()
    print(OUTPUT)
    print(image_path)


if __name__ == "__main__":
    main()
