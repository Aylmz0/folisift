#!/usr/bin/env python3
"""Build a deterministic source inventory for a study-guide project.

This script does the repetitive part of the workflow:
  * page count / title / word count
  * one-line map of every page
  * heading-like lines and their page numbers
  * pages with little or no text layer (figures, code screenshots, tables)

It deliberately uses only poppler utilities (pdfinfo, pdftotext), so it works
without Python PDF libraries.

Usage:
    python3 tools/analyze_sources.py ../sources/PD0.pdf ../sources/PD1.pdf \
        --out data/pd0-pd1.source.json \
        --md data/pd0-pd1.source.md
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

HEADING_RE = re.compile(
    r"^(?P<num>\d+(?:\.\d+){0,3})\.?\s+(?P<title>[A-ZÇĞİÖŞÜ][^\n]{2,150})$"
)
CHAPTER_RE = re.compile(r"^(?:BÖLÜM|Bölüm)\s+\d+\b.*$")
PAGE_NO_RE = re.compile(r"^\s*\d{1,4}\s*$")
BULLET_RE = re.compile(r"^[\s•●▪◦\-–—*]+")


def run(cmd: list[str]) -> str:
    try:
        return subprocess.run(
            cmd, check=True, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL
        ).stdout.decode("utf-8", errors="replace")
    except (subprocess.CalledProcessError, FileNotFoundError) as exc:
        raise RuntimeError(f"Komut başarısız: {' '.join(cmd)}") from exc


def clean_text(text: str) -> str:
    text = text.replace("\f", "\n")
    text = text.replace("\u00a0", " ")
    lines = []
    for line in text.splitlines():
        line = line.rstrip()
        if line.strip():
            lines.append(line.strip())
    return "\n".join(lines)


def meaningful_lines(text: str) -> list[str]:
    out = []
    for line in clean_text(text).splitlines():
        stripped = line.strip()
        if not stripped or PAGE_NO_RE.match(stripped):
            continue
        if stripped.startswith(("http://", "https://")):
            continue
        out.append(stripped)
    return out


def heading_of(lines: list[str]) -> str | None:
    for line in lines[:8]:
        candidate = line.strip()
        if CHAPTER_RE.match(candidate):
            return candidate
        match = HEADING_RE.match(candidate)
        if match:
            title = match.group("title").strip()
            # Ignore sentence-like lines and slide bullet fragments.
            if len(title.split()) <= 14 and not title.endswith((".", ":", ";")):
                return f"{match.group('num')} {title}"
    return None


def first_line(lines: list[str]) -> str:
    for line in lines[:6]:
        if len(line) >= 3:
            return line[:120]
    return ""


def page_text(pdf: Path, page: int) -> str:
    return run(
        [
            "pdftotext",
            "-f",
            str(page),
            "-l",
            str(page),
            "-layout",
            str(pdf),
            "-",
        ]
    )


def pdf_info(pdf: Path) -> tuple[int, str]:
    info = run(["pdfinfo", str(pdf)])
    pages = 0
    title = ""
    for line in info.splitlines():
        if line.startswith("Pages:"):
            pages = int(line.split(":", 1)[1].strip())
        elif line.startswith("Title:"):
            title = line.split(":", 1)[1].strip()
    if not pages:
        raise RuntimeError(f"Sayfa sayısı okunamadı: {pdf}")
    return pages, title


def analyze(pdf: Path, low_threshold: int) -> dict:
    pages, title = pdf_info(pdf)
    page_rows = []
    headings = []
    low_pages = []
    seen_headings = set()
    total_chars = 0
    total_words = 0

    for page in range(1, pages + 1):
        raw = page_text(pdf, page)
        text = clean_text(raw)
        chars = len(re.sub(r"\s+", "", text))
        total_chars += chars
        total_words += len(re.findall(r"\S+", text))
        lines = meaningful_lines(raw)
        first = first_line(lines)
        heading = heading_of(lines)
        page_rows.append(
            {
                "page": page,
                "first_line": first,
                "chars": chars,
                "low_text": chars < low_threshold,
            }
        )
        if chars < low_threshold:
            low_pages.append(page)
        if heading:
            key = re.sub(r"\s+", " ", heading).casefold()
            if key not in seen_headings:
                seen_headings.add(key)
                headings.append({"heading": heading, "page": page})

    return {
        "path": str(pdf),
        "filename": pdf.name,
        "title": title,
        "pages": pages,
        "chars": total_chars,
        "words": total_words,
        "low_text_threshold": low_threshold,
        "low_text_pages": low_pages,
        "headings": headings,
        "page_map": page_rows,
    }


def markdown_report(data: dict) -> str:
    lines = [
        "# Kaynak Envanteri",
        "",
        f"Üretim: {data['generated_at']}",
        "",
        "| PDF | Sayfa | Kelime | Düşük metinli sayfa |",
        "|---|---:|---:|---:|",
    ]
    for src in data["sources"]:
        lines.append(
            f"| {src['filename']} | {src['pages']} | {src['words']} | "
            f"{len(src['low_text_pages'])} |"
        )
    lines += [
        "",
        f"**Toplam:** {data['totals']['pages']} slayt, "
        f"{data['totals']['words']} kelime, "
        f"{data['totals']['low_text_pages']} düşük metinli slayt.",
        "",
    ]

    for src in data["sources"]:
        lines += [
            f"## {src['filename']} — {src['title'] or 'Başlıksız'}",
            "",
            f"- Sayfa: **{src['pages']}**",
            f"- Kelime: **{src['words']}**",
            f"- Düşük metinli sayfalar: "
            f"**{', '.join(map(str, src['low_text_pages'])) or 'yok'}**",
            "",
            "### Başlıklar",
            "",
            "| Sayfa | Başlık |",
            "|---:|---|",
        ]
        for heading in src["headings"]:
            lines.append(f"| {heading['page']} | {heading['heading']} |")
        if not src["headings"]:
            lines.append("| — | Başlık çıkarılamadı; sayfa haritasına bakın. |")
        lines += [
            "",
            "### Sayfa Haritası",
            "",
            "| Sayfa | İlk anlamlı satır | Düşük metin |",
            "|---:|---|---|",
        ]
        for row in src["page_map"]:
            flag = "evet" if row["low_text"] else ""
            first = row["first_line"].replace("|", "\\|")
            lines.append(f"| {row['page']} | {first} | {flag} |")
        lines.append("")
    return "\n".join(lines)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pdfs", nargs="+", type=Path)
    parser.add_argument("--out", type=Path, default=Path("data/source_map.json"))
    parser.add_argument("--md", type=Path, default=Path("data/source_map.md"))
    parser.add_argument("--low-threshold", type=int, default=120)
    args = parser.parse_args()

    sources = []
    for pdf in args.pdfs:
        if not pdf.exists():
            print(f"HATA: PDF bulunamadı: {pdf}", file=sys.stderr)
            return 2
        print(f"Analiz ediliyor: {pdf}")
        sources.append(analyze(pdf, args.low_threshold))

    data = {
        "generated_at": datetime.now(timezone.utc).isoformat(timespec="seconds"),
        "low_text_threshold": args.low_threshold,
        "sources": sources,
        "totals": {
            "pages": sum(s["pages"] for s in sources),
            "words": sum(s["words"] for s in sources),
            "low_text_pages": sum(len(s["low_text_pages"]) for s in sources),
        },
    }

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(data, ensure_ascii=False, indent=2), encoding="utf-8")
    args.md.parent.mkdir(parents=True, exist_ok=True)
    args.md.write_text(markdown_report(data), encoding="utf-8")

    print(
        f"Yazıldı: {args.out}\n"
        f"Yazıldı: {args.md}\n"
        f"Toplam: {data['totals']['pages']} slayt, "
        f"{data['totals']['words']} kelime, "
        f"{data['totals']['low_text_pages']} düşük metinli slayt."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
