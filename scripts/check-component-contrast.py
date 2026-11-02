#!/usr/bin/env python3
"""Development-only component-level contrast check.

Requires PyMuPDF (fitz) and NumPy. The product does not depend on them.

Palette validation proves role pairs on paper. This script measures the actual
rendered component: for every text span it finds the dominant colour in the
span box (table fill, callout, code background, diagram node) and compares it
with the declared text colour. Run with ``--strict`` to fail a release.
"""
from __future__ import annotations

import sys
from collections import Counter
from pathlib import Path

import fitz
import numpy as np

SCALE = 2
LARGE_TEXT = 18.0
LARGE_BOLD = 14.0


def luminance(color: tuple[float, float, float]) -> float:
    channels = [
        value / 12.92 if value <= 0.03928 else ((value + 0.055) / 1.055) ** 2.4
        for value in color
    ]
    return 0.2126 * channels[0] + 0.7152 * channels[1] + 0.0722 * channels[2]


def contrast(first: tuple[float, float, float], second: tuple[float, float, float]) -> float:
    a, b = luminance(first), luminance(second)
    return (max(a, b) + 0.05) / (min(a, b) + 0.05)


def span_color(value: int) -> tuple[float, float, float]:
    return (
        ((value >> 16) & 255) / 255,
        ((value >> 8) & 255) / 255,
        (value & 255) / 255,
    )


def background(pixels: np.ndarray, box: fitz.Rect) -> tuple[float, float, float] | None:
    height, width, _ = pixels.shape
    x0 = max(0, int(box.x0 * SCALE))
    y0 = max(0, int(box.y0 * SCALE))
    x1 = min(width, int(box.x1 * SCALE) + 1)
    y1 = min(height, int(box.y1 * SCALE) + 1)
    if x1 - x0 < 2 or y1 - y0 < 2:
        return None
    block = pixels[y0:y1:2, x0:x1:2, :3]
    quantized = (block >> 4).reshape(-1, 3)
    colors, counts = np.unique(quantized, axis=0, return_counts=True)
    dominant = colors[counts.argmax()]
    return tuple((int(value) * 16 + 8) / 255 for value in dominant)


def check(path: Path, strict: bool) -> int:
    doc = fitz.open(path)
    failures: list[tuple[float, int, str, tuple, tuple]] = []
    spans = 0
    for page_number, page in enumerate(doc, start=1):
        pixmap = page.get_pixmap(matrix=fitz.Matrix(SCALE, SCALE), alpha=False)
        pixels = np.frombuffer(pixmap.samples, dtype=np.uint8).reshape(
            pixmap.height, pixmap.width, pixmap.n
        )
        for block in page.get_text("dict")["blocks"]:
            if block["type"] != 0:
                continue
            for line in block["lines"]:
                for span in line["spans"]:
                    if not span["text"].strip() or span["alpha"] == 0:
                        continue
                    fill = background(pixels, fitz.Rect(span["bbox"]))
                    if fill is None:
                        continue
                    spans += 1
                    foreground = span_color(span["color"])
                    ratio = contrast(foreground, fill)
                    large = span["size"] >= LARGE_TEXT or (
                        span["size"] >= LARGE_BOLD and span["flags"] & 16
                    )
                    required = 3.0 if large else 4.5
                    if ratio + 1e-6 < required:
                        failures.append(
                            (
                                ratio,
                                page_number,
                                span["text"].strip()[:40],
                                foreground,
                                fill,
                            )
                        )

    failures.sort()
    print(f"{path}: {spans} text spans, {len(failures)} below threshold")
    grouped: Counter[tuple[tuple, tuple]] = Counter(
        (foreground, fill) for _, _, _, foreground, fill in failures
    )
    for (foreground, fill), count in grouped.most_common(8):
        example = next(
            text
            for _, _, text, foreground_key, fill_key in failures
            if foreground_key == foreground and fill_key == fill
        )
        print(
            f"  {count:3} span(s) fg=#{''.join(f'{int(value * 255):02X}' for value in foreground)} "
            f"bg=#{''.join(f'{int(value * 255):02X}' for value in fill)} "
            f"ratio={contrast(foreground, fill):.2f}:1 example={example!r}"
        )
    if strict and failures:
        return 1
    return 0


def main() -> int:
    paths = [Path(argument) for argument in sys.argv[1:] if argument != "--strict"]
    strict = "--strict" in sys.argv[1:]
    if not paths:
        print("usage: check-component-contrast.py [--strict] <pdf>...", file=sys.stderr)
        return 2
    return max(check(path, strict) for path in paths)


if __name__ == "__main__":
    raise SystemExit(main())
