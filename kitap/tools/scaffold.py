#!/usr/bin/env python3
"""Scaffold a new study-guide project from the shared paper template.

Example:
    python3 tools/scaffold.py \
        --slug pd2 \
        --title "Bölüm 2: Önemli Programlama Dillerinin Gelişimi" \
        --subtitle "Tarihsel gelişim" \
        --sources ../sources/PD2.pdf

Creates:
    guides/pd2/content.html
    guides/pd2/config.json
    guides/pd2/data/pd2.source.json
    guides/pd2/data/pd2.source.md

The content file is a skeleton; the agent fills B/C/D/E/F/G sections.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TEMPLATE = ROOT / "templates/guide_skeleton.html"


def load_analysis(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--slug", required=True)
    parser.add_argument("--title", required=True)
    parser.add_argument("--subtitle", default="Çalışma kılavuzu")
    parser.add_argument("--course", default="BMÜ325 Programlama Dilleri")
    parser.add_argument("--sources", nargs="*", type=Path, default=[])
    parser.add_argument("--out", type=Path)
    parser.add_argument("--version-label", default="Çalışma sürümü")
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args()

    out = (args.out or (ROOT / "guides" / args.slug)).resolve()
    content_path = out / "content.html"
    config_path = out / "config.json"
    data_dir = out / "data"
    if (content_path.exists() or config_path.exists()) and not args.force:
        print(f"HATA: {out} zaten var. --force kullanın.", file=sys.stderr)
        return 2
    out.mkdir(parents=True, exist_ok=True)
    data_dir.mkdir(parents=True, exist_ok=True)

    analysis = None
    source_names = "Kaynak PDF"
    total_pages = "—"
    source_rows = "<tr><td>—</td><td>—</td><td>—</td><td>—</td></tr>"
    if args.sources:
        source_json = data_dir / f"{args.slug}.source.json"
        source_md = data_dir / f"{args.slug}.source.md"
        cmd = [
            sys.executable,
            str(ROOT / "tools/analyze_sources.py"),
            *[str(p.resolve()) for p in args.sources],
            "--out",
            str(source_json),
            "--md",
            str(source_md),
        ]
        print("Kaynak envanteri çıkarılıyor...")
        subprocess.run(cmd, check=True)
        analysis = load_analysis(source_json)
        source_names = " · ".join(
            f"{s['filename'].replace('.pdf', '')} ({s['pages']} slayt)"
            for s in analysis["sources"]
        )
        total_pages = str(analysis["totals"]["pages"])
        rows = []
        for s in analysis["sources"]:
            sections = [
                h["heading"]
                for h in s["headings"]
                if h["heading"][:1].isdigit()
            ]
            first = sections[0] if sections else "—"
            rows.append(
                f"<tr><td>{s['filename'].replace('.pdf', '')}</td>"
                f"<td>{s['pages']}</td><td>{first} …</td>"
                f"<td>Doldurulacak</td></tr>"
            )
        source_rows = "\n      ".join(rows)

    html = TEMPLATE.read_text(encoding="utf-8")
    rel_css = os.path.relpath(ROOT / "style.css", out)
    rel_cover = os.path.relpath(ROOT / "assets/cover-motif.svg", out)
    html = html.replace("../../style.css", rel_css)
    html = html.replace("../../assets/cover-motif.svg", rel_cover)
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
        "title": args.title,
        "slug": args.slug,
        "html": os.path.relpath(content_path, ROOT),
        "css": "style.css",
        "output": f"../output/{args.slug}_Calisma_Ozeti.pdf",
        "sources": [str(p.resolve()) for p in args.sources],
        "source_json": os.path.relpath(data_dir / f"{args.slug}.source.json", ROOT),
        "coverage_md": os.path.relpath(data_dir / f"{args.slug}.coverage.md", ROOT),
    }
    config_path.write_text(
        json.dumps(config, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
    )

    print(f"Oluşturuldu: {out}")
    print(f"  {content_path.name}")
    print(f"  {config_path.name}")
    if args.sources:
        print(f"  data/{args.slug}.source.json")
    print("\nSonraki adımlar:")
    print(f"  1. {content_path} içindeki iskeleti kaynak sırasına göre doldurun.")
    print(f"  2. python3 build.py --config {config_path} --check")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
