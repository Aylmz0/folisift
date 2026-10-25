#!/usr/bin/env python3
"""Render low-text PDF pages into labelled contact sheets for visual review.

The study-guide workflow uses this for slides whose text layer is missing or
insufficient: diagrams, code screenshots, tables and scanned figures.

Examples:
    python3 tools/prepare_pages.py ../sources/PD0.pdf --pages 7,17,21-23 \
        --out data/pages/pd0 --cols 2 --rows 2

    python3 tools/prepare_pages.py ../sources/PD1.pdf \
        --source-json data/pd0-pd1.source.json --filename PD1.pdf \
        --out data/pages/pd1
"""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from PIL import Image, ImageDraw


def parse_pages(spec: str) -> list[int]:
    pages: list[int] = []
    for part in spec.split(","):
        part = part.strip()
        if not part:
            continue
        if "-" in part:
            start, end = part.split("-", 1)
            pages.extend(range(int(start), int(end) + 1))
        else:
            pages.append(int(part))
    return sorted(set(pages))


def pages_from_json(path: Path, filename: str | None) -> list[int]:
    data = json.loads(path.read_text(encoding="utf-8"))
    for source in data["sources"]:
        if filename is None or source["filename"] == filename:
            return list(source["low_text_pages"])
    raise SystemExit(f"Kaynak bulunamadı: {filename or path}")


def render_page(pdf: Path, page: int, dpi: int, out_dir: Path) -> Path:
    prefix = out_dir / f"page_{page:04d}"
    target = prefix.with_suffix(".png")
    if target.exists():
        return target
    subprocess.run(
        [
            "pdftoppm",
            "-f",
            str(page),
            "-l",
            str(page),
            "-r",
            str(dpi),
            "-png",
            "-singlefile",
            str(pdf),
            str(prefix),
        ],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    return target


def make_sheets(
    images: list[tuple[int, Path]], out_dir: Path, cols: int, rows: int
) -> list[tuple[Path, list[int]]]:
    sheets = []
    per_sheet = cols * rows
    for index in range(0, len(images), per_sheet):
        batch = images[index : index + per_sheet]
        loaded = [Image.open(path).convert("RGB") for _, path in batch]
        width = max(image.width for image in loaded)
        height = max(image.height for image in loaded)
        sheet = Image.new("RGB", (width * cols, height * rows), "white")
        draw = ImageDraw.Draw(sheet)
        for slot, ((page, _), image) in enumerate(zip(batch, loaded)):
            x = (slot % cols) * width
            y = (slot // cols) * height
            sheet.paste(image, (x, y))
            draw.rectangle([x, y, x + 74, y + 24], fill="white")
            draw.text((x + 6, y + 6), f"s.{page}", fill="#8A3E2F")
        sheet_path = out_dir / f"sheet_{index // per_sheet + 1:02d}.png"
        sheet.save(sheet_path, optimize=True)
        sheets.append((sheet_path, [page for page, _ in batch]))
    return sheets


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pdf", type=Path)
    parser.add_argument("--pages", help="Örn. 7,17,21-23")
    parser.add_argument("--source-json", type=Path)
    parser.add_argument("--filename", help="JSON içindeki PDF dosya adı")
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--dpi", type=int, default=110)
    parser.add_argument("--cols", type=int, default=2)
    parser.add_argument("--rows", type=int, default=2)
    args = parser.parse_args()

    if args.pages:
        pages = parse_pages(args.pages)
    elif args.source_json:
        pages = pages_from_json(args.source_json, args.filename)
    else:
        parser.error("--pages veya --source-json gerekli")

    args.out.mkdir(parents=True, exist_ok=True)
    images = []
    for page in pages:
        print(f"Render: s.{page}")
        images.append((page, render_page(args.pdf, page, args.dpi, args.out)))

    sheets = make_sheets(images, args.out, args.cols, args.rows)
    lines = [
        f"# Görsel İnceleme Sayfaları — {args.pdf.name}",
        "",
        f"Toplam sayfa: **{len(pages)}**",
        "",
        "| Sayfa | Dosya |",
        "|---:|---|",
    ]
    for sheet, sheet_pages in sheets:
        lines.append(f"| {', '.join(map(str, sheet_pages))} | {sheet.name} |")
    (args.out / "index.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(f"Yazıldı: {args.out} ({len(sheets)} kontakt sayfası)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
