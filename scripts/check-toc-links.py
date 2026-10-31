#!/usr/bin/env python3
"""Development-only check that TOC rows and printed page numbers link correctly.

Requires PyMuPDF (fitz). The product does not depend on it.
"""
from __future__ import annotations

import sys
from pathlib import Path

import fitz


def toc_pages(doc: fitz.Document) -> list[tuple[int, list[dict]]]:
    found = []
    for index, page in enumerate(doc):
        links = [link for link in page.get_links() if link.get("page") is not None]
        if len(links) < 2:
            continue
        words = page.get_text("words")
        numeric = 0
        for link in links:
            rect = link["from"]
            y = (rect.y0 + rect.y1) / 2
            nearby = [
                word[4]
                for word in words
                if abs((word[1] + word[3]) / 2 - y) < 5 and word[0] > 500
            ]
            if nearby and nearby[-1].isdigit():
                numeric += 1
        if numeric >= 2:
            found.append((index, links))
    return found


def check(path: Path) -> int:
    doc = fitz.open(path)
    if doc.page_count < 3:
        print(f"{path}: too few pages", file=sys.stderr)
        return 1
    pages = toc_pages(doc)
    if not pages:
        print(f"{path}: no TOC page with numeric internal links", file=sys.stderr)
        return 1
    errors = []
    total_links = 0
    for index, links in pages:
        total_links += len(links)
        words = doc[index].get_text("words")
        for link in links:
            target = link.get("page")
            if target is None:
                continue
            if not (0 <= target < doc.page_count):
                errors.append(("out-of-range", target))
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
    if total_links < 3:
        print(f"{path}: only {total_links} TOC links", file=sys.stderr)
        return 1
    if errors:
        print(f"{path}: TOC page-number mismatches: {errors[:5]}", file=sys.stderr)
        return 1
    print(
        f"{path}: {doc.page_count} pages, {len(pages)} TOC page(s), "
        f"{total_links} links, page numbers match"
    )
    return 0


def main() -> int:
    if len(sys.argv) < 2:
        print("usage: check-toc-links.py <pdf>...", file=sys.stderr)
        return 2
    return max(check(Path(path)) for path in sys.argv[1:])


if __name__ == "__main__":
    raise SystemExit(main())
