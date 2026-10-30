#!/usr/bin/env python3
"""Development-only check that TOC rows and printed page numbers link correctly.

Requires PyMuPDF (fitz). The product does not depend on it.
"""
from __future__ import annotations

import sys
from pathlib import Path

import fitz


def check(path: Path) -> int:
    doc = fitz.open(path)
    if doc.page_count < 3:
        print(f"{path}: too few pages", file=sys.stderr)
        return 1
    page = doc[2]
    links = page.get_links()
    words = page.get_text("words")
    errors = []
    for link in links:
        target = link.get("page")
        if target is None:
            continue
        rect = link["from"]
        y = (rect.y0 + rect.y1) / 2
        nearby = [
            word[4]
            for word in words
            if abs((word[1] + word[3]) / 2 - y) < 5 and word[0] > 500
        ]
        if not nearby:
            continue
        printed = nearby[-1]
        if not printed.isdigit():
            continue
        if int(printed) != target + 1:
            errors.append((printed, target + 1))
    if errors:
        print(f"{path}: TOC page-number mismatches: {errors[:5]}", file=sys.stderr)
        return 1
    print(f"{path}: {doc.page_count} pages, {len(links)} TOC links, page numbers match")
    return 0


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: check-toc-links.py <pdf>...", file=sys.stderr)
        return 2
    return max(check(Path(path)) for path in sys.argv[1:])


if __name__ == "__main__":
    raise SystemExit(main())
