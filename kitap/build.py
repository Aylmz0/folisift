#!/usr/bin/env python3
"""Generic PDF builder for study-guide projects.

Examples:
    python3 build.py
    python3 build.py --config config/pd0-pd1.json
    python3 build.py --html pilot.html --output ../out.pdf
    python3 build.py --config config/pd0-pd1.json --check
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def page_count(pdf: Path) -> int | None:
    try:
        out = subprocess.run(
            ["pdfinfo", str(pdf)],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL,
        ).stdout.decode("utf-8", errors="replace")
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None
    for line in out.splitlines():
        if line.startswith("Pages:"):
            return int(line.split(":", 1)[1].strip())
    return None


def resolve(base: Path, value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else (base / path).resolve()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--config", type=Path, default=ROOT / "config/pd0-pd1.json")
    parser.add_argument("--html", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--css", type=Path)
    parser.add_argument("--check", action="store_true", help="Kapsam denetimini de çalıştır")
    parser.add_argument("--no-optimize", action="store_true")
    args = parser.parse_args()

    config = {}
    if args.config.exists():
        config = json.loads(args.config.read_text(encoding="utf-8"))
        base = args.config.resolve().parent.parent
    else:
        base = ROOT

    html_path = args.html or resolve(base, config.get("html", "pilot.html"))
    output = args.output or resolve(
        base, config.get("output", "../output/PD0-PD1_Calisma_Ozeti.pdf")
    )
    css_path = args.css or resolve(base, config.get("css", "style.css"))

    if not html_path.exists():
        print(f"HATA: HTML bulunamadı: {html_path}", file=sys.stderr)
        return 2
    if not css_path.exists():
        print(f"HATA: CSS bulunamadı: {css_path}", file=sys.stderr)
        return 2

    if args.check:
        source_json = resolve(base, config.get("source_json", "data/source.json"))
        coverage_md = resolve(base, config.get("coverage_md", "data/coverage.md"))
        checker = ROOT / "tools/check_coverage.py"
        result = subprocess.run(
            [
                sys.executable,
                str(checker),
                "--source-json",
                str(source_json),
                "--html",
                str(html_path),
                "--out",
                str(coverage_md),
            ]
        )
        if result.returncode != 0:
            print(
                "Kapsam denetimi geçmedi. Önce data/coverage.md dosyasını inceleyin.",
                file=sys.stderr,
            )
            return result.returncode

    try:
        from weasyprint import HTML
    except ImportError:
        print(
            "HATA: WeasyPrint kurulu değil.\n"
            "Kurulum: python3 -m pip install --user weasyprint",
            file=sys.stderr,
        )
        return 3

    output.parent.mkdir(parents=True, exist_ok=True)
    HTML(filename=str(html_path), base_url=str(html_path.parent) + "/").write_pdf(
        str(output),
        optimize_images=not args.no_optimize,
    )
    pages = page_count(output)
    print(f"Yazıldı: {output}")
    if pages:
        print(f"Sayfa: {pages}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
