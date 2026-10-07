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

"""Assemble the exact 196 x 196 production sprites into reviewable atlases."""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont


ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / "assets" / "buttons"
OUTPUT = ROOT / "assets" / "production"
THEMES = (
    "dark-classic",
    "dark-abstract",
    "light-classic",
    "light-abstract",
    "manga-ink",
    "steampunk-brass",
    "cyberpunk-neon",
    "moire",
    "cubism",
    "art-deco",
    "ukiyo-e",
    "solarpunk",
    "memphis",
)
PAGES = (("windows", 1), ("codex", 2), ("vscode", 3), ("utility", 4), ("themes", 5))
SIDE = 196
GAP = 12
MARGIN = 24
HEADER = 68


# theme: one of the thirteen production palettes.
# page_name: English page label for the sheet title.
# page_number: one-based page index in the sprite export.
# Errors: missing or invalid source PNG raises a Pillow or filesystem exception.
def build(theme: str, page_name: str, page_number: int) -> Path:
    dark = theme in {
        "dark-classic",
        "dark-abstract",
        "steampunk-brass",
        "cyberpunk-neon",
        "art-deco",
        "ukiyo-e",
    }
    background = "#101a2a" if dark else "#ede9de"
    foreground = "#ecf2f8" if dark else "#14304a"
    width = MARGIN * 2 + SIDE * 4 + GAP * 3
    height = MARGIN * 2 + HEADER + SIDE * 7 + GAP * 6
    sheet = Image.new("RGB", (width, height), background)
    draw = ImageDraw.Draw(sheet)
    font_path = ROOT / "assets" / "fonts" / "VeraBd.ttf"
    font = ImageFont.truetype(str(font_path), 26) if font_path.exists() else ImageFont.load_default()
    draw.text((MARGIN, MARGIN), f"{page_name.upper()}  /  {theme.upper()}", font=font, fill=foreground)
    for index in range(14):
        for pressed, state in ((False, "up"), (True, "down")):
            col = index % 2 * 2 + int(pressed)
            row = index // 2
            path = SOURCE / theme / f"page-{page_number}" / f"key-{index:02}-{state}.png"
            with Image.open(path) as sprite:
                assert sprite.size == (SIDE, SIDE), path
                x = MARGIN + col * (SIDE + GAP)
                y = MARGIN + HEADER + row * (SIDE + GAP)
                sheet.paste(sprite.convert("RGB"), (x, y))
    OUTPUT.mkdir(parents=True, exist_ok=True)
    path = OUTPUT / f"{page_name}-{theme}.png"
    sheet.save(path, optimize=True)
    return path


if __name__ == "__main__":
    for selected_theme in THEMES:
        for selected_page_name, selected_page_number in PAGES:
            print(build(selected_theme, selected_page_name, selected_page_number))
