# VBWR B
# Project: UlanziDock VibeWare version
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
# VibeWare is Human intent, AI, and plenty of tokens ;-)
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
# AI Tooling: May include OpenAI Codex, GitHub Copilot and AI-assisted CI/CD pipelines.
# AI Versions: Tools and models may vary by contributor and execution environment.
# AI Traceability: Refer to Git history and CI/CD logs for recorded provenance.
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
# Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
# VBWR E

"""Build the public A4 landscape controller atlas using Umberto's PDF layout spec."""

from pathlib import Path
from reportlab.lib import colors
from reportlab.lib.pagesizes import A4, landscape
from reportlab.lib.utils import ImageReader, simpleSplit
from reportlab.pdfbase import pdfmetrics
from reportlab.pdfbase.ttfonts import TTFont
from reportlab.pdfgen import canvas

ROOT = Path(__file__).resolve().parents[1]
OUTPUT = ROOT / "docs" / "ulanzi-dock-vibeware-guide-v1.pdf"
FONT_DIR = Path(r"C:\Windows\Fonts")
MONO_DIR = Path(r"C:\Users\umber\AppData\Local\Microsoft\Windows\Fonts")
W, H = landscape(A4)

INK = colors.HexColor("#142737")
MUTED = colors.HexColor("#536776")
BLUE = colors.HexColor("#359FE7")
RULE = colors.HexColor("#D6E1E8")
PANEL = colors.HexColor("#F1F6F9")
NAVY = colors.HexColor("#203040")

pdfmetrics.registerFont(TTFont("Segoe", str(FONT_DIR / "segoeui.ttf")))
pdfmetrics.registerFont(TTFont("SegoeBold", str(FONT_DIR / "segoeuib.ttf")))
pdfmetrics.registerFont(TTFont("JetMono", str(MONO_DIR / "JetBrainsMonoNerdFontMono-Regular.ttf")))
pdfmetrics.registerFont(TTFont("JetMonoBold", str(MONO_DIR / "JetBrainsMonoNerdFontMono-Bold.ttf")))


def baseline(pdf, text, x, y, font="Segoe", size=10.5, color=INK):
    """Draw one baseline with top-origin coordinates."""
    pdf.setFont(font, size)
    pdf.setFillColor(color)
    pdf.drawString(x, H - y, text)


def paragraph(pdf, text, x, y, width, size=10.5, leading=15, color=MUTED, font="Segoe"):
    """Draw wrapped copy and return the next top-origin y position."""
    for line in simpleSplit(text, font, size, width):
        baseline(pdf, line, x, y, font, size, color)
        y += leading
    return y


def box(pdf, x, y, width, height, fill=PANEL, radius=8):
    """Draw a filled rounded panel using top-origin coordinates."""
    pdf.setFillColor(fill)
    pdf.roundRect(x, H - y - height, width, height, radius, fill=1, stroke=0)


def rule(pdf, x1, y, x2, color=RULE, width=0.65):
    """Draw a horizontal rule using top-origin coordinates."""
    pdf.setStrokeColor(color)
    pdf.setLineWidth(width)
    pdf.line(x1, H - y, x2, H - y)


def frame(pdf, number, section, title, subtitle, source, status="DESIGN / IMPLEMENTATION"):
    """Apply the owner's A4 landscape document frame to one page."""
    baseline(pdf, "ULANZIDOCK / " + section, 36, 30, "JetMonoBold", 8.5, BLUE)
    pdf.setFont("JetMono", 8)
    pdf.setFillColor(MUTED)
    pdf.drawRightString(W - 36, H - 30, "07 OCT 2026 / v1")
    baseline(pdf, title, 36, 66, "SegoeBold", 25, INK)
    baseline(pdf, subtitle, 36, 89, "Segoe", 10.5, MUTED)
    rule(pdf, 36, 104, W - 36)
    baseline(pdf, "SOURCE / " + source, 36, 466, "JetMono", 7.1, MUTED)
    baseline(pdf, "NOTES / QUESTIONS", 36, 491, "JetMonoBold", 8, BLUE)
    for y in (507, 525, 543):
        rule(pdf, 36, y, W - 36)
    rule(pdf, 36, 551, W - 36)
    baseline(pdf, "Umberto Giacobbi | hello@umbertogiacobbi.biz", 36, 566, "Segoe", 8.7, MUTED)
    pdf.linkURL("mailto:hello@umbertogiacobbi.biz", (36, H - 569, 280, H - 555), relative=0)
    baseline(pdf, "Copyright 2026 Umberto Giacobbi - MIT", 36, 581, "JetMono", 7.3, MUTED)
    pdf.setFont("JetMono", 7.3)
    pdf.setFillColor(MUTED)
    pdf.drawRightString(W - 62, H - 581, status)
    pdf.setFont("JetMonoBold", 9)
    pdf.setFillColor(BLUE)
    pdf.drawRightString(W - 36, H - 581, f"{number:02}")


def image_fit(pdf, path, x, y, width, height):
    """Place a PNG inside a top-origin box without changing its aspect ratio."""
    image = ImageReader(str(path))
    iw, ih = image.getSize()
    scale = min(width / iw, height / ih)
    dw, dh = iw * scale, ih * scale
    pdf.drawImage(image, x + (width - dw) / 2, H - y - (height + dh) / 2, dw, dh, mask="auto")


def label_value(pdf, y, label, value):
    """Draw one compact key-value row."""
    box(pdf, 36, y, 470, 35, PANEL)
    baseline(pdf, label, 48, y + 22, "SegoeBold", 10, INK)
    baseline(pdf, value, 170, y + 22, "Segoe", 9.6, MUTED)


def cover(pdf):
    """Present scope, delivery and evidence status."""
    frame(pdf, 1, "OVERVIEW", "UlanziDock VibeWare version", "Standalone D200H controller - button artwork, pages and shortcuts", "Project source, generated atlases and owner-supplied Cody PDF layout specification")
    logo = ROOT / "assets" / "brand" / "vibeware-logo.png"
    image_fit(pdf, logo, 548, 133, 220, 220)
    paragraph(pdf, "A lightweight resident CLI for 13 display keys plus the wide bottom key. The program draws a complete page, receives input from the D200H consumer HID interface and dispatches a configured action on release.", 36, 139, 473, 13, 19, INK)
    label_value(pdf, 221, "PAGES", "Windows / media - Codex - VS Code")
    label_value(pdf, 264, "THEMES", "Two dark and two light; up/down per key")
    label_value(pdf, 307, "DELIVERY", "Source portable; Windows x64 and ARM64 binaries")
    box(pdf, 36, 365, 730, 67, PANEL)
    baseline(pdf, "EVIDENCE STATUS", 50, 387, "JetMonoBold", 8, BLUE)
    paragraph(pdf, "Windows x64 build and layout packaging tests passed. The owner confirmed the clock and NEXT PAGE on the D200H with earlier artwork. The new borderless 196 x 196 faces are verified offline and await a physical display check. ARM64 device behavior is unverified.", 50, 406, 700, 9.6, 14, MUTED)
    pdf.showPage()


def physical_map(pdf):
    """Explain the exact dock positions and proposed page commands."""
    frame(pdf, 2, "KEY MAP", "One fixed navigation key", "Key 04 is top right on hardware; the atlas lists key pairs in numeric order.", "Default settings.json layout and D200H key indices")
    positions = [(i % 5, i // 5, i) for i in range(13)]
    for col, row, index in positions:
        x, y = 36 + col * 63, 131 + row * 74
        fill = BLUE if index == 4 else PANEL
        box(pdf, x, y, 54, 60, fill, 6)
        baseline(pdf, f"{index:02}", x + 13, y + 34, "JetMonoBold", 15, colors.white if index == 4 else INK)
    box(pdf, 225, 279, 118, 60, NAVY, 6)
    baseline(pdf, "13 CLOCK", 236, 314, "JetMonoBold", 11, colors.white)
    baseline(pdf, "NEXT PAGE", 273, 125, "JetMonoBold", 7.5, BLUE)
    paragraph(pdf, "Cycles Windows / media -> Codex -> VS Code -> Windows / media. A release selects the next page once. The wide bottom display shows the clock only.", 36, 372, 323, 10, 15, MUTED)
    x = 392
    rows = [
        ("00-03", "Media transport and mute", "Chat navigation", "Open/search/tools"),
        ("04", "Next page", "Next page", "Next page"),
        ("05-09", "Volume and windows", "Review and work", "Editor panels"),
        ("10-12", "Capture and browser", "Copy/paste/find", "Files and problems"),
        ("13", "Clock", "Clock", "Clock"),
    ]
    for i, (keys, win, codex, vscode) in enumerate(rows):
        y = 132 + i * 59
        box(pdf, x, y, 376, 52, PANEL if i % 2 == 0 else colors.white, 5)
        baseline(pdf, keys, x + 10, y + 18, "JetMonoBold", 9, BLUE)
        baseline(pdf, win, x + 70, y + 18, "Segoe", 9.4, INK)
        baseline(pdf, codex + " / " + vscode, x + 70, y + 37, "Segoe", 8.2, MUTED)
    pdf.showPage()


def theme_page(pdf, number, slug, theme_title, page_slug, page_number, page_title):
    """Show every production up/down sprite for one page and one theme."""
    frame(pdf, number, "PRODUCTION ATLAS", f"{theme_title} / {page_title}", "Each pair shows released/up and pressed/down on a full 196 x 196 display face.", "Built-in renderer and assets/buttons; no button bezel is drawn", "OFFLINE SPRITES / 13 ACTION KEYS + CLOCK")
    for row in range(7):
        panel = 0 if row < 4 else 1
        display_row = row if panel == 0 else row - 4
        for pair_col in range(2):
            index = row * 2 + pair_col
            for pressed, state in ((False, "up"), (True, "down")):
                column = pair_col * 2 + int(pressed)
                x = 36 + panel * 410 + column * 87
                y = 119 + display_row * 79
                if index == 13:
                    box(pdf, x, y, 78, 78, NAVY, 3)
                    baseline(pdf, "CLOCK", x + 13, y + 43, "JetMonoBold", 9, colors.white)
                else:
                    sprite = ROOT / "assets" / "buttons" / slug / f"page-{page_number}" / f"key-{index:02}-{state}.png"
                    image_fit(pdf, sprite, x, y, 78, 78)
    pdf.showPage()


def operation(pdf):
    """Provide operating commands and evidence boundaries."""
    frame(pdf, 15, "OPERATE", "Run, customize, verify", "A single executable embeds the artwork; settings.json changes themes and actions.", "README.md, CLI implementation and Windows x64 offline build")
    commands = [
        ("01 / CHECK DEVICE", ".\\ulanzi-dock-vibeware.exe doctor"),
        ("02 / CREATE SETTINGS", ".\\ulanzi-dock-vibeware.exe init"),
        ("03 / START", ".\\ulanzi-dock-vibeware.exe run"),
        ("04 / OVERRIDE THEME", ".\\ulanzi-dock-vibeware.exe run --theme light-abstract"),
        ("05 / EXPORT ART", ".\\ulanzi-dock-vibeware.exe render --output artwork"),
    ]
    for i, (heading, command) in enumerate(commands):
        y = 126 + i * 52
        box(pdf, 36, y, 770, 44, PANEL, 7)
        baseline(pdf, heading, 49, y + 17, "JetMonoBold", 7.9, BLUE)
        baseline(pdf, command, 205, y + 27, "JetMono", 10, INK)
    box(pdf, 36, 403, 770, 44, colors.HexColor("#E8F3F9"), 7)
    paragraph(pdf, "Shortcuts target the focused app. The wide bottom display shows the clock by default and has no assigned action. USB write, visible output and button input are separate observations. Stop with Ctrl+C.", 49, 420, 738, 9.2, 13, INK)
    pdf.showPage()


def main():
    """Write the fifteen-page PDF with complete production atlases."""
    pdf = canvas.Canvas(str(OUTPUT), pagesize=(W, H), pageCompression=1)
    pdf.setTitle("UlanziDock VibeWare version - Controller Atlas and Guide")
    pdf.setAuthor("Umberto Giacobbi | hello@umbertogiacobbi.biz")
    pdf.setSubject("Ulanzi D200H pages, four themes, up/down artwork, CLI use and verification status")
    pdf.setKeywords("Ulanzi D200H, VibeWare, Umberto Giacobbi, Codex, VS Code, media controller")
    cover(pdf)
    physical_map(pdf)
    number = 3
    for slug, theme_title in (("dark-classic", "Matrix"), ("dark-abstract", "Blade Runner"), ("light-classic", "Ivory / blue"), ("light-abstract", "Mint / teal")):
        for page_slug, page_number, page_title in (("windows", 1, "Windows / media"), ("codex", 2, "Codex"), ("vscode", 3, "VS Code")):
            theme_page(pdf, number, slug, theme_title, page_slug, page_number, page_title)
            number += 1
    operation(pdf)
    pdf.save()
    print(OUTPUT)


if __name__ == "__main__":
    main()
