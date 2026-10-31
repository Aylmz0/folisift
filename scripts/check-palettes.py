#!/usr/bin/env python3
"""Development-only palette contrast and grayscale proof."""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TEXT_ROLES = ("ink", "muted", "soft", "faint", "accent", "secondary")
SURFACE_ROLES = ("paper", "code_bg", "cover_bg", "cream", "zebra", "headerfill")


def luminance(hex_value: str) -> float:
    value = hex_value.lstrip("#")
    if len(value) != 6:
        raise ValueError(hex_value)
    channels = [int(value[index : index + 2], 16) / 255 for index in (0, 2, 4)]
    channels = [
        channel / 12.92 if channel <= 0.03928 else ((channel + 0.055) / 1.055) ** 2.4
        for channel in channels
    ]
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]


def contrast(first: str, second: str) -> float:
    a, b = luminance(first), luminance(second)
    lighter, darker = max(a, b), min(a, b)
    return (lighter + 0.05) / (darker + 0.05)


def main() -> int:
    failed = False
    for path in sorted((ROOT / "styles/palettes").glob("*.json")):
        data = json.loads(path.read_text())
        palette = data["palette"]
        paper = palette["paper"]
        worst_text = min((contrast(palette[role], paper), role) for role in TEXT_ROLES)
        worst_surface = min((contrast(palette["ink"], palette[role]), role) for role in SURFACE_ROLES)
        print(
            f"{data['id']:15} paper Y={luminance(paper):.3f} "
            f"worst-text {worst_text[1]} {worst_text[0]:.2f}:1 "
            f"worst-surface {worst_surface[1]} {worst_surface[0]:.2f}:1"
        )
        if worst_text[0] < 4.5 or worst_surface[0] < 4.5:
            failed = True
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
