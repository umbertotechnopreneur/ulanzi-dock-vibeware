# VBWR B
#
# Project: UlanziDock VibeWare version
# Repository: https://github.com/umbertotechnopreneur/ulanzi-dock-vibeware
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
#
# VibeWare: Human intent, AI execution, and plenty of tokens
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
#
# Modified with AI: OpenAI Codex; comments and readability; 2026-10-27
# Human guidance: Umberto Giacobbi; AGENTS.md; readability/documentation request
# Evidence: docs/CODE-REVIEW.md and current working-tree changes
#
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
#
# VBWR E

"""Prepare artwork, the OBS guide, or notices through explicit developer commands.

Files are written only by the selected command. Existing rendered pictures and
restored Cargo sources are inputs; this tool never controls HID, launches the
controller, downloads dependencies or publishes a release.
"""

import argparse
import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def icons():
    """Write PNG/ICO/ICNS derivatives of the original transparent logo.

    Reject a nontransparent source; Pillow and filesystem errors propagate.
    """
    from PIL import Image

    output = ROOT / "assets/icons"
    output.mkdir(parents=True, exist_ok=True)
    with Image.open(ROOT / "assets/brand/ulanzi-dock-icon.png") as master:
        if master.mode != "RGBA" or master.getchannel("A").getextrema()[0] != 0:
            raise ValueError("The icon master must have a transparent background.")
        sizes = [16, 24, 32, 48, 64, 128, 256, 512, 1024]
        frames = {size: master.resize((size, size), Image.Resampling.NEAREST) for size in sizes}
        for size, frame in frames.items():
            frame.save(output / f"ulanzi-dock-{size}.png")
        frames[256].save(output / "ulanzi-dock.ico", sizes=[(s, s) for s in sizes if s <= 256],
                         append_images=[frames[s] for s in sizes if s < 256])
        frames[1024].save(output / "ulanzi-dock.icns",
                          append_images=[frames[s] for s in [16, 32, 64, 128, 256, 512]])
    print(f"Packaged PNG, ICO and ICNS icons in {output}")


def atlases(source: Path, settings: Path, output: Path):
    """Build preview sheets from a complete offline artwork export.

    source: directory of exported theme/page/key PNGs.
    settings: the JSON used by the render command, preserving page-number meaning.
    output: destination directory for one sheet per theme/page.
    Validate completeness and dimensions before replacing preview sheets.
    """
    from PIL import Image, ImageDraw, ImageFont

    pages = json.loads(settings.read_text(encoding="utf-8-sig"))["pages"]
    if not any(page["name"] == "Page index" for page in pages):
        pages.append({"name": "Page index"})
    names = {"Windows / media": "windows", "VS Code": "vscode",
             "OBS Creator": "obs", "Page index": "index"}
    slugs = []
    for page in pages:
        page_name = page["name"]
        default_slug = re.sub(r"[^a-z0-9]+", "-", page_name.lower()).strip("-")
        slugs.append(names.get(page_name, default_slug))
    if not all(slugs) or len(set(slugs)) != len(slugs):
        raise ValueError("Page names must produce distinct, non-empty picture filenames.")
    themes = sorted(path for path in source.iterdir() if path.is_dir())
    if not themes:
        raise ValueError(f"No exported themes found in {source}")
    # Check for a complete export before replacing any preview sheets.
    for theme in themes:
        for number in range(1, len(pages) + 1):
            for key in range(14):
                for state in ("up", "down"):
                    path = theme / f"page-{number}" / f"key-{key:02}-{state}.png"
                    if not path.is_file():
                        raise FileNotFoundError(f"Incomplete export: {path}")
    font = ImageFont.truetype(str(ROOT / "assets/fonts/VeraBd.ttf"), 26)
    side, gap, margin, header = 196, 12, 24, 68
    output.mkdir(parents=True, exist_ok=True)
    dark_themes = {"dark-classic", "dark-abstract", "steampunk-brass", "cyberpunk-neon", "art-deco", "ukiyo-e"}
    for theme in themes:
        dark = theme.name in dark_themes
        for number, slug in enumerate(slugs, 1):
            with Image.new("RGB", (margin * 2 + side * 4 + gap * 3,
                                    margin * 2 + header + side * 7 + gap * 6),
                           "#101a2a" if dark else "#ede9de") as sheet:
                ImageDraw.Draw(sheet).text((margin, margin), f"{slug.upper()}  /  {theme.name.upper()}",
                                           font=font, fill="#ecf2f8" if dark else "#14304a")
                for key in range(14):
                    for pressed, state in enumerate(("up", "down")):
                        path = theme / f"page-{number}" / f"key-{key:02}-{state}.png"
                        with Image.open(path) as sprite:
                            if sprite.size != (side, side) and not (key == 13 and sprite.size == (400, 200)):
                                raise ValueError(f"Unexpected button size: {path}")
                            tile = sprite.convert("RGB")
                            tile.thumbnail((side, side), Image.Resampling.LANCZOS)
                            # Two physical keys per row, each with released/pressed tiles.
                            column = key % 2 * 2 + pressed
                            row = key // 2
                            x = margin + column * (side + gap)
                            y = margin + header + row * (side + gap)
                            sheet.paste(tile, (x + (side - tile.width) // 2, y + (side - tile.height) // 2))
                path = output / f"{slug}-{theme.name}.png"
                sheet.save(path, optimize=True)
                print(path)


def notices(output: Path, target: str | None, features: str | None = None):
    """Copy dependency notices from locally restored, locked Cargo packages.

    output: destination for per-package files and an index.
    target: optional Rust platform filter, not a build request.
    features: optional Cargo feature selection matching the packaged binaries.
    Cargo runs offline. Metadata, copy and file errors abort the command.
    """
    command = ["cargo", "metadata", "--locked", "--offline", "--format-version", "1"]
    if target:
        command += ["--filter-platform", target]
    if features:
        command += ["--features", features]
    metadata = json.loads(subprocess.check_output(command, cwd=ROOT, text=True, encoding="utf-8"))
    output.mkdir(parents=True, exist_ok=True)
    index = ["# Dependency notices", "",
             "Generated from locked, restored packages. This list can include build tools, too.", "",
             "Project code remains MIT licensed. Dependencies retain their own licenses.", "",
             "| Dependency | Declared license | Source |", "| --- | --- | --- |"]
    packages = sorted((p for p in metadata["packages"] if p["source"] is not None),
                      key=lambda p: (p["name"], p["version"]))
    for package in packages:
        name, version = package["name"], package["version"]
        license_name = package.get("license") or "See supplied license files"
        directory = Path(package["manifest_path"]).parent
        destination = output / f"{name}-{version}"
        candidates = []
        for item in directory.iterdir():
            if item.is_file() and re.match(r"(?i)^(licen[sc]e|copying|copyright|notice|third.?party)", item.name):
                candidates.append(item)
            elif item.is_dir() and item.name.lower() in {"licenses", "licences"}:
                candidates += [file for file in item.rglob("*") if file.is_file()]
        declared = directory / (package.get("license_file") or license_name)
        if declared.is_file() and declared not in candidates:
            candidates.append(declared)
        for item in candidates:
            path = destination / item.relative_to(directory)
            path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(item, path)
        url = package.get("repository") or ""
        index.append(f'| {name} {version} | {license_name.replace("|", "/")} | {url} |')
    (output / "README.md").write_text("\n".join(index) + "\n", encoding="utf-8")
    print(f"Collected notices for {len(packages)} locked dependencies in {output}")


def main():
    """Parse arguments or an interactive menu and run exactly one selected task.

    Noninteractive use requires a command; invalid or missing input is an error.
    """
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("icons", help="Prepare app icons for Windows, Linux and macOS")
    sheets = commands.add_parser("atlases", help="Collect exported buttons into preview sheets")
    sheets.add_argument("--source", type=Path, default=ROOT / "assets/buttons")
    sheets.add_argument("--settings", type=Path, required=True, help="The settings used for the render export")
    sheets.add_argument("--output", type=Path, default=ROOT / "assets/production")
    guide = commands.add_parser("obs-guide", help="Rebuild the OBS preview and printable guide")
    guide.add_argument("--artwork", type=Path, default=ROOT / "tmp/obs-creator/artwork/dark-abstract/page-9")
    licenses = commands.add_parser("notices", help="Copy licenses from already restored Cargo packages")
    licenses.add_argument("--mode", choices=["cargo"], default="cargo")
    licenses.add_argument("--features", help="Cargo feature selection used by the packaged build")
    licenses.add_argument("--output", type=Path, required=True)
    licenses.add_argument("--target")
    if len(sys.argv) == 1:
        if not sys.stdin.isatty():
            parser.error("Choose a command; run with --help to see the choices.")
        print("\nUlanziDock file tools\n1. App icons\n2. Button preview sheets\n3. OBS guide\n4. Dependency notices\n0. Exit")
        choice = input("Choose: ").strip()
        if choice in {"", "0"}:
            return
        arguments = {"1": ["icons"], "2": ["atlases"], "3": ["obs-guide"], "4": ["notices"]}.get(choice)
        if arguments is None:
            parser.error("Choose a number from 0 to 4.")
        if choice == "2":
            arguments += ["--settings", input("Settings file used for the export: ").strip()]
        elif choice == "3":
            artwork = input("OBS artwork folder (Enter for tmp/obs-creator/artwork/dark-abstract/page-9): ").strip()
            if artwork:
                arguments += ["--artwork", artwork]
        elif choice == "4":
            arguments += ["--output", input("Notices output folder: ").strip()]
        if any(not argument for argument in arguments):
            parser.error("A required path was left empty.")
        args = parser.parse_args(arguments)
    else:
        args = parser.parse_args()
    if args.command == "icons":
        icons()
    elif args.command == "atlases":
        atlases(args.source, args.settings, args.output)
    elif args.command == "obs-guide":
        from lib.obs_guide import main as obs_guide
        obs_guide(args.artwork)
    elif args.command == "notices":
        notices(args.output, args.target, args.features)


if __name__ == "__main__":
    main()
