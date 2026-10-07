# VBWR B
# Project: UlanziDock VibeWare version
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
# VibeWare is Human intent. AI implementation. Accountable human review.
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
# AI Tooling: May include OpenAI Codex, GitHub Copilot and AI-assisted CI/CD pipelines.
# AI Versions: Tools and models may vary by contributor and execution environment.
# AI Traceability: Refer to Git history and CI/CD logs for recorded provenance.
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
# Required by the MIT License: retain the copyright and permission notice in all copies or substantial portions of the Software.
# VBWR E

"""Package exported Spotify artwork and update the existing PDF without HID access."""

from io import BytesIO
from pathlib import Path
import json

from PIL import Image, ImageDraw, ImageFont
from pypdf import PdfReader, PdfWriter
from reportlab.lib.colors import HexColor
from reportlab.lib.pagesizes import A4, landscape
from reportlab.pdfgen import canvas
import yaml

ROOT = Path(__file__).resolve().parents[1]
PDF = ROOT / "docs/ulanzi-dock-vibeware-guide-v2.pdf"
THEMES = [
    ("dark-classic", "Matrix"), ("dark-abstract", "Blade Runner"),
    ("light-classic", "Ivory Blue"), ("light-abstract", "Mint Teal"),
    ("manga-ink", "Manga"), ("steampunk-brass", "Steampunk"),
    ("cyberpunk-neon", "Cyberpunk"), ("moire", "Moire"), ("cubism", "Cubism"),
    ("art-deco", "Art Deco"), ("ukiyo-e", "Ukiyo-e"), ("solarpunk", "Solarpunk"),
    ("memphis", "Memphis"),
]
WIDTH, HEIGHT = landscape(A4)
INK = HexColor("#172638")
BLUE = HexColor("#3379ad")
MUTED = HexColor("#5f7286")


def header(pdf, category, title, subtitle, number):
    pdf.setFillColor(INK)
    pdf.setFont("Helvetica-Bold", 11)
    pdf.drawString(28, HEIGHT - 28, "ULANZIDOCK / " + category)
    pdf.setFillColor(MUTED)
    pdf.setFont("Helvetica", 9)
    pdf.drawRightString(WIDTH - 28, HEIGHT - 28, "08 OCT 2026 / v2")
    pdf.setStrokeColor(BLUE)
    pdf.line(28, HEIGHT - 41, WIDTH - 28, HEIGHT - 41)
    pdf.setFillColor(INK)
    pdf.setFont("Helvetica-Bold", 24)
    pdf.drawString(28, HEIGHT - 75, title)
    pdf.setFillColor(MUTED)
    pdf.setFont("Helvetica", 10)
    pdf.drawString(28, HEIGHT - 95, subtitle)
    pdf.setFont("Helvetica", 8)
    pdf.drawString(28, 28, "Umberto Giacobbi | VibeWare | Copyright 2026 - MIT")
    pdf.drawRightString(WIDTH - 28, 28, f"{number:02d}")


def lines(pdf, text, x, y, size=12, leading=20, color=INK):
    pdf.setFillColor(color)
    pdf.setFont("Helvetica", size)
    for line in text:
        pdf.drawString(x, y, line)
        y -= leading
    return y


def link(pdf, label, url, x, y):
    pdf.setFillColor(BLUE)
    pdf.setFont("Helvetica", 10)
    pdf.drawString(x, y, label)
    pdf.linkURL(url, (x, y - 2, x + pdf.stringWidth(label, "Helvetica", 10), y + 12), relative=0)


def front_pages():
    stream = BytesIO()
    pdf = canvas.Canvas(stream, pagesize=landscape(A4))
    header(pdf, "OVERVIEW", "UlanziDock VibeWare", "Six pages, thirteen themes, shortcuts one key away.", 1)
    lines(pdf, [
        "Windows / media - Codex - VS Code - Utility - Themes - Spotify",
        "Automatic application pages on Windows, checked every one second by default.",
        "Manual page choices remain in place until the foreground application changes.",
        "Inactive application keys are gray with the themed NOT AVAILABLE ribbon.",
    ], 28, 450)
    pdf.setFont("Helvetica-Bold", 12)
    pdf.setFillColor(BLUE)
    pdf.drawString(28, 338, "YOUR LOCAL CONFIGURATION")
    lines(pdf, [
        "applications.yaml: process names, commands, descriptions, icon IDs and actions.",
        "Runtime: auto_switch and focus_poll_seconds (1-60 seconds).",
        "CLI: run --no-auto-page or run --focus-poll-seconds N.",
        "settings.json retains themes and local settings. Restart after configuration edits.",
    ], 28, 314, size=11)
    pdf.setFillColor(BLUE)
    pdf.setFont("Helvetica-Bold", 12)
    pdf.drawString(28, 213, "ARTWORK AND EVIDENCE")
    lines(pdf, [
        "78 theme/page atlases and 2,184 normal/pressed PNGs, including clock placeholders.",
        "Earlier atlas pages are retained; the Spotify set follows them in thirteen themes.",
        "The current Windows x64 build compiles. New switching and Spotify actions still",
        "require observation on the physical dock; no new hardware trial is claimed.",
        "A side project provided as-is, without compatibility or behavior guarantees.",
    ], 28, 189, size=11, leading=18)
    link(pdf, "Project source and guide", "https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware", 28, 78)
    link(pdf, "VibeWare manifesto", "https://umbertogiacobbi.biz/vibeware/manifesto", 235, 78)
    pdf.showPage()
    header(pdf, "KEY MAP", "One fixed navigation key", "Key 04 is top right; key 13 is the wide clock/status display.", 2)
    gap, cell_w, cell_h = 10, 96, 51
    origin_x, origin_y = 28, 437
    for row, indices in enumerate(([0, 1, 2, 3, 4], [5, 6, 7, 8, 9], [10, 11, 12, 13])):
        for column, key in enumerate(indices):
            x, y = origin_x + column * (cell_w + gap), origin_y - row * (cell_h + gap)
            width = cell_w * 2 + gap if key == 13 else cell_w
            pdf.setFillColor(HexColor("#e9f0f6"))
            pdf.roundRect(x, y, width, cell_h, 5, stroke=0, fill=1)
            pdf.setFillColor(BLUE if key == 4 else INK)
            pdf.setFont("Helvetica-Bold", 12)
            pdf.drawString(x + 10, y + 21, f"{key:02d}" + (" NEXT PAGE" if key == 4 else " CLOCK / STATUS" if key == 13 else ""))
    lines(pdf, [
        "Windows / media -> Codex -> VS Code -> Utility -> Themes -> Spotify -> Windows / media",
        "Select a theme on page five. Theme selection returns to Windows / media.",
        "Automatic switching chooses Codex, VS Code or Spotify when its process takes focus.",
        "A page switch cancels held keys; a release cannot trigger the new page's command.",
        "Unrecognized apps leave the selected page in place; unavailable keys remain disabled.",
    ], 28, 257, size=10.5, leading=21)
    link(pdf, "Spotify desktop shortcut reference", "https://support.spotify.com/mt/article/keyboard-shortcuts/", 28, 115)
    lines(pdf, [
        "Previous/next use Windows media-session routing. Other Spotify shortcuts depend",
        "on the focused control and installed version. This integration needs no account link.",
    ], 28, 88, size=10, leading=16, color=MUTED)
    pdf.showPage()
    pdf.save()
    stream.seek(0)
    return PdfReader(stream)


def main():
    font = ImageFont.truetype("C:/Windows/Fonts/segoeuib.ttf", 30)
    catalog = yaml.safe_load((ROOT / "applications.yaml").read_text(encoding="utf-8-sig"))
    spotify = next(app for app in catalog["applications"] if app["page"] == "Spotify")
    production = ROOT / "assets/production"
    for slug, _ in THEMES:
        atlas = Image.new("RGB", (868, 1560), "#0e1929")
        ImageDraw.Draw(atlas).text((24, 22), "SPOTIFY / " + slug.upper(), font=font, fill="#e8eef3")
        for key in range(14):
            row, pair = divmod(key, 2)
            for state_index, state in enumerate(("up", "down")):
                source = ROOT / f"assets/buttons/{slug}/page-6/key-{key:02d}-{state}.png"
                with Image.open(source) as tile:
                    atlas.paste(tile.convert("RGB"), (24 + (pair * 2 + state_index) * 208, 92 + row * 208))
        atlas.save(production / f"spotify-{slug}.png")

    old = PdfReader(PDF)
    # Replace only the introductory pages; retain the previously approved artwork pages.
    retained = [page for page in old.pages[2:] if "ULANZIDOCK / SPOTIFY ATLAS" not in (page.extract_text() or "")]
    base_count = 2 + len(retained)
    stream = BytesIO()
    pdf = canvas.Canvas(stream, pagesize=landscape(A4))
    for theme_index, (slug, label) in enumerate(THEMES):
        header(pdf, "SPOTIFY ATLAS", label + " / Spotify", "Released/up and pressed/down artwork; theme gradients remain inside each pictogram.", base_count + theme_index + 1)
        pdf.setFillColor(BLUE)
        pdf.setFont("Helvetica-Bold", 11)
        pdf.drawString(28, 450, "SPOTIFY.EXE / FOREGROUND CONTROLS")
        y = 423
        for command in spotify["commands"]:
            pdf.setFont("Helvetica", 10)
            pdf.setFillColor(MUTED)
            pdf.drawString(28, y, f"{command['key']:02d}")
            pdf.setFillColor(INK)
            pdf.setFont("Helvetica-Bold", 10)
            pdf.drawString(60, y, command["name"])
            pdf.setFont("Helvetica", 9)
            pdf.drawString(213, y, command.get("action", "settings.json"))
            y -= 23
        lines(pdf, ["04  NEXT PAGE", "13  CLOCK / STATUS - no physical-press action"], 28, 130, size=10, leading=20, color=MUTED)
        lines(pdf, ["Inactive: grayscale plus the accent-colored NOT AVAILABLE ribbon.",
                    "Previous/next follow Windows media-session routing."], 28, 79, size=9, leading=16, color=MUTED)
        pdf.drawImage(str(production / f"spotify-{slug}.png"), 572, 53, width=245, height=440, preserveAspectRatio=True)
        pdf.showPage()
    pdf.save()
    stream.seek(0)
    writer = PdfWriter()
    for page in front_pages().pages:
        writer.add_page(page)
    for page in retained:
        writer.add_page(page)
    for page in PdfReader(stream).pages:
        writer.add_page(page)
    writer.add_metadata({"/Title": "UlanziDock VibeWare - Six-page artwork guide", "/Author": "Umberto Giacobbi", "/Subject": "Application switching, YAML commands and Spotify artwork"})
    pending = PDF.with_suffix(".new.pdf")
    with pending.open("wb") as file:
        writer.write(file)
    pending.replace(PDF)
    print(json.dumps({"spotify_atlases": 13, "pdf_pages": len(writer.pages), "pdf": str(PDF)}))


if __name__ == "__main__":
    main()
