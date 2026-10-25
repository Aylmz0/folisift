#!/usr/bin/env python3
"""Check that every top-level source section is represented in the guide.

This is a guardrail, not a substitute for reading. It catches the common
failure mode where the first half of a deck is covered well and later sections
are silently skipped.

Usage:
    python3 tools/check_coverage.py \
        --source-json data/pd0-pd1.source.json \
        --html pilot.html \
        --out data/pd0-pd1.coverage.md
"""

from __future__ import annotations

import argparse
import difflib
import html
import json
import re
from pathlib import Path

TAG_RE = re.compile(r"<[^>]+>")


def normalize(text: str) -> str:
    text = html.unescape(text).casefold()
    text = text.replace("ı", "i").replace("İ", "i")
    text = re.sub(r"[^\w\s]", " ", text, flags=re.UNICODE)
    return re.sub(r"\s+", " ", text).strip()


def html_text(path: Path) -> tuple[str, list[str]]:
    raw = path.read_text(encoding="utf-8")
    raw = re.sub(r"<(script|style)\b.*?</\1>", " ", raw, flags=re.S | re.I)
    raw = re.sub(r'<table class="toc".*?</table>', " ", raw, flags=re.S | re.I)
    raw = re.sub(r"</?(?:span|strong|em|code|a|b|i)\b[^>]*>", " ", raw, flags=re.I)
    text = html.unescape(TAG_RE.sub("\n", raw))
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    return normalize(text), lines


def top_level_sections(source: dict) -> list[dict]:
    """Keep numbered subsections such as 0.1 / 1.7, drop slide bullets."""
    result = []
    seen = set()
    for item in source["headings"]:
        heading = item["heading"].strip()
        match = re.match(r"^(\d+\.\d+)\s+(.+)$", heading)
        if not match:
            continue
        number, title = match.group(1), match.group(2).strip()
        key = number.casefold()
        if key in seen:
            continue
        seen.add(key)
        result.append({"number": number, "title": title, "page": item["page"]})
    return result


def best_line(number: str, title: str, lines: list[str]) -> tuple[float, str]:
    number_re = re.compile(rf"(?<![\d.]){re.escape(number)}(?![\d.])")
    candidates = [line for line in lines if number_re.search(line)]
    if not candidates:
        return 0.0, ""
    target = normalize(f"{number} {title}")
    best_score = 0.0
    best_text = ""
    for line in candidates:
        score = difflib.SequenceMatcher(None, target, normalize(line)).ratio()
        if score > best_score:
            best_score = score
            best_text = line
    return best_score, best_text


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-json", type=Path, required=True)
    parser.add_argument("--html", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--threshold", type=float, default=0.55)
    args = parser.parse_args()

    data = json.loads(args.source_json.read_text(encoding="utf-8"))
    _, lines = html_text(args.html)

    report = [
        "# Kapsam Denetimi",
        "",
        f"Kaynak envanteri: `{args.source_json}`",
        f"Kılavuz: `{args.html}`",
        "",
    ]
    missing = 0
    total = 0

    for source in data["sources"]:
        sections = top_level_sections(source)
        report += [
            f"## {source['filename']} — {source['pages']} slayt",
            "",
            "| Bölüm | Kaynak sayfa | Durum | Eşleşen kılavuz satırı |",
            "|---|---:|---|---|",
        ]
        for section in sections:
            total += 1
            score, matched = best_line(
                section["number"], section["title"], lines
            )
            ok = score >= args.threshold
            if not ok:
                missing += 1
            status = "OK" if ok else "EKSİK"
            safe_match = matched.replace("|", "\\|")[:150]
            report.append(
                f"| {section['number']} {section['title']} | "
                f"{section['page']} | {status} | {safe_match} |"
            )
        report += [
            "",
            f"Alt bölüm: **{len(sections)}** · "
            f"Düşük metinli slayt: **{len(source['low_text_pages'])}**",
            "",
        ]

    report += [
        "## Sonuç",
        "",
        f"- Denetlenen alt bölüm: **{total}**",
        f"- Eksik/weak match: **{missing}**",
        f"- Sonuç: **{'GEÇTİ' if missing == 0 else 'KONTROL GEREKLİ'}**",
        "",
        "Not: Bu otomatik denetim başlık düzeyindedir. Tablo, şekil, formül ve "
        "örnek kapsamı ayrıca G bölümünde elle doğrulanmalıdır.",
        "",
    ]

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text("\n".join(report), encoding="utf-8")
    print(
        f"Yazıldı: {args.out}\n"
        f"Alt bölüm: {total}, eksik: {missing}"
    )
    return 0 if missing == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
