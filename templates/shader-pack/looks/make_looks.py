#!/usr/bin/env python3
"""Writes the template's looks as 17x17x17 .cube files.

Each look is a function from a display colour to a display colour, both sRGB-encoded from 0 to 1
as in a screenshot. A LUT exported from DaVinci Resolve, Photoshop or any other grading tool
works the same way; this script only shows that a look can be made in code too.
"""

from pathlib import Path

SIZE = 17


def luma(r, g, b):
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def saturate(rgb, amount):
    l = luma(*rgb)
    return [l + amount * (c - l) for c in rgb]


def contrast(rgb, amount, pivot=0.45):
    return [pivot + (c - pivot) * amount for c in rgb]


def monochrome(rgb):
    rgb = saturate(rgb, 0.15)
    rgb = contrast(rgb, 1.05)
    return [c * 0.9 for c in rgb]


def overcast(rgb):
    rgb = saturate(rgb, 0.55)
    rgb = [0.03 + c * 0.88 for c in rgb]
    return [rgb[0] * 0.98, rgb[1], rgb[2] * 1.03]


def noir(rgb):
    rgb = saturate(rgb, 0.0)
    rgb = contrast(rgb, 1.25)
    return [c * 0.95 for c in rgb]


def write(name, title, look):
    lines = [f'TITLE "{title}"', f"LUT_3D_SIZE {SIZE}"]
    for b in range(SIZE):
        for g in range(SIZE):
            for r in range(SIZE):
                rgb = look([r / (SIZE - 1), g / (SIZE - 1), b / (SIZE - 1)])
                lines.append(" ".join(f"{min(max(c, 0.0), 1.0):.6f}" for c in rgb))
    Path(__file__).with_name(f"{name}.cube").write_text("\n".join(lines) + "\n")


write("monochrome", "Monochrome", monochrome)
write("overcast", "Overcast", overcast)
write("noir", "Noir", noir)
