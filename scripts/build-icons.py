# VBWR B
# Project: UlanziDock VibeWare version
# Creator: Umberto Giacobbi | https://umbertogiacobbi.biz
# VibeWare is Human intent. AI implementation. Accountable human review.
# Manifesto: https://umbertogiacobbi.biz/vibeware/manifesto
# Copyright (c) 2026 Umberto Giacobbi
# SPDX-License-Identifier: MIT
# License: MIT - see LICENSE
# VBWR E

"""Package the unchanged ImageGen icon master for Windows, Linux and macOS."""
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]

# Errors: unreadable master, missing alpha, or filesystem/encoder failures.
def main():
    output = ROOT / "assets" / "icons"
    output.mkdir(parents=True, exist_ok=True)
    with Image.open(ROOT / "assets" / "brand" / "ulanzi-dock-icon.png") as master:
        assert master.mode == "RGBA" and master.getchannel("A").getextrema()[0] == 0
        sizes = [16, 24, 32, 48, 64, 128, 256, 512, 1024]
        frames = {}
        for size in sizes:
            frames[size] = master.resize((size, size), Image.Resampling.NEAREST)
            frames[size].save(output / f"ulanzi-dock-{size}.png")
        frames[256].save(output / "ulanzi-dock.ico", sizes=[(s, s) for s in sizes if s <= 256], append_images=[frames[s] for s in sizes if s < 256])
        frames[1024].save(output / "ulanzi-dock.icns", append_images=[frames[s] for s in [16, 32, 64, 128, 256, 512]])
    print(f"Packaged PNG, ICO and ICNS icons in {output}")

if __name__ == "__main__":
    main()
