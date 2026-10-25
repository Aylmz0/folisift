#!/usr/bin/env python3
"""Create a new guide bundle under guides/<slug>/.

A guide bundle is intentionally self-contained:

    guides/<slug>/
    ├── guide.html      # content (from engine/templates/guide.html)
    ├── config.json     # paths + metadata
    ├── assets/         # guide-specific figures
    └── data/           # generated inventory + coverage report

Example (run from repo root):
    python3 engine/tools/scaffold.py \
        --slug pd2 \
        --title "Bölüm 2: Önemli Programlama Dillerinin Gelişimi" \
        --sources sources/PD2.pdf
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TEMPLATE = ROOT / "engine/templates/guide.html"


def load_analysis(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--slug", required=True)
    parser.add_argument("--title", required=True)
    parser.add_argument("--subtitle", default="Çalışma kılavuzu")
    parser.add_argument("--course", default="Ders")
    parser.add_argument("--sources", nargs="*", type=Path, default=[])
    parser.add_argument("--out", type=Path)
    parser.add_argument("--version-label", default="Çalışma sürümü")
    parser.add_argument("--target-pages", default="18-22")
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args()

    out = (args.out or (ROOT / "guides" / args.slug)).resolve()
    content_path = out / "guide.html"
    config_path = out / "config.json"
    data_dir = out / "data"
    if (content_path.exists() or config_path.exists()) and not args.force:
        print(f"HATA: {out} zaten var. --force kullanın.", file=sys.stderr)
        return 2
    out.mkdir(parents=True, exist_ok=True)
    data_dir.mkdir(parents=True, exist_ok=True)
    (out / "assets").mkdir(exist_ok=True)

    source_names = "Kaynak PDF"
    total_pages = "—"
    source_rows = "<tr><td>—</td><td>—</td><td>—</td><td>—</td></tr>"
    source_paths: list[str] = []

    if args.sources:
        source_json = data_dir / "source.json"
        source_md = data_dir / "source.md"
        source_paths = [
            os.path.relpath(source.resolve(), out) for source in args.sources
        ]
        cmd = [
            sys.executable,
            str(ROOT / "engine/tools/analyze_sources.py"),
            *source_paths,
            "--out",
            "data/source.json",
            "--md",
            "data/source.md",
        ]
        print("Kaynak envanteri çıkarılıyor...")
        subprocess.run(cmd, check=True, cwd=out)
        analysis = load_analysis(source_json)
        source_names = " · ".join(
            f"{s['filename'].replace('.pdf', '')} ({s['pages']} slayt)"
            for s in analysis["sources"]
        )
        total_pages = str(analysis["totals"]["pages"])
        rows = []
        for source in analysis["sources"]:
            first = next(
                (
                    h["heading"]
                    for h in source["headings"]
                    if h["heading"][:1].isdigit()
                ),
                "—",
            )
            rows.append(
                f"<tr><td>{source['filename'].replace('.pdf', '')}</td>"
                f"<td>{source['pages']}</td><td>{first} …</td>"
                f"<td>Doldurulacak</td></tr>"
            )
        source_rows = "\n      ".join(rows)

    html = TEMPLATE.read_text(encoding="utf-8")
    html = html.replace(
        "{{STYLESHEET}}", os.path.relpath(ROOT / "engine/style.css", out)
    )
    html = html.replace(
        "{{COVER_IMAGE}}", os.path.relpath(ROOT / "engine/assets/cover-motif.svg", out)
    )
    html = html.replace("{{TITLE}}", args.title)
    html = html.replace("{{SUBTITLE}}", args.subtitle)
    html = html.replace("{{COURSE}}", args.course)
    html = html.replace("{{VERSION_LABEL}}", args.version_label)
    html = html.replace("{{SOURCE_NAMES}}", source_names)
    html = html.replace("{{TOTAL_PAGES}}", total_pages)
    html = html.replace("{{SOURCE_ROWS}}", source_rows)
    html = html.replace(
        "{{LEAD}}",
        "Bu kılavuz, kaynak slaytların sınav odaklı bir özetidir; "
        "kavramlar, mekanizmalar ve kritik ayrımlar tek bir metinde toplanır.",
    )
    html = html.replace(
        "{{OVERVIEW_BOX}}",
        "Kaynakların ana konularını, birbirine bağlandığı noktaları ve "
        "kılavuzun kapsamını burada özetleyin.",
    )
    html = html.replace(
        "{{ABBREVIATIONS}}",
        "Kılavuzda kullanılan kısaltmaları burada tanımlayın.",
    )
    content_path.write_text(html, encoding="utf-8")

    config = {
        "slug": args.slug,
        "title": args.title,
        "language": "tr",
        "content": "guide.html",
        "stylesheet": os.path.relpath(ROOT / "engine/style.css", out),
        "output": os.path.relpath(
            ROOT / "output" / f"{args.slug}_Calisma_Ozeti.pdf", out
        ),
        "sources": source_paths,
        "inventory": "data/source.json",
        "coverage": "data/coverage.md",
        "target_pages": args.target_pages,
    }
    config_path.write_text(
        json.dumps(config, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    print(f"Oluşturuldu: {out}")
    print(f"  {content_path.name}")
    print(f"  {config_path.name}")
    if args.sources:
        print("  data/source.json")
    print("\nSonraki adımlar:")
    print(f"  1. {content_path} içindeki iskeleti kaynak sırasına göre doldurun.")
    print(f"  2. python3 engine/build.py --guide {out} --check")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
