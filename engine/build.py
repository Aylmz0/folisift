#!/usr/bin/env python3
"""Build a Folisift guide PDF.

A guide is a directory under ``guides/`` with a ``config.json``. Paths in that
config are resolved relative to the guide directory, so a guide can be moved
without changing the engine.

Examples:
    python3 engine/build.py --guide guides/pd0-pd1
    python3 engine/build.py --guide guides/pd0-pd1 --check
    python3 engine/build.py --guide guides/pd0-pd1 --output /tmp/preview.pdf
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_GUIDE = ROOT / "guides/pd0-pd1"


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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--guide", type=Path, default=DEFAULT_GUIDE)
    parser.add_argument("--config", type=Path, help="Varsayılan: <guide>/config.json")
    parser.add_argument("--html", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--check", action="store_true", help="Kapsam denetimini de çalıştır")
    parser.add_argument("--no-optimize", action="store_true")
    args = parser.parse_args()

    guide = args.guide.resolve()
    config_path = (args.config or (guide / "config.json")).resolve()
    if not config_path.exists():
        print(f"HATA: Guide config bulunamadı: {config_path}", file=sys.stderr)
        return 2
    config = json.loads(config_path.read_text(encoding="utf-8"))
    base = config_path.parent

    content = (
        args.html
        or (base / config.get("legacy_content", config.get("content", "guide.html")))
    ).resolve()
    stylesheet = (base / config.get("stylesheet", "../../engine/style.css")).resolve()
    output = (args.output or (base / config.get(
        "output", f"../../output/{config.get('slug', guide.name)}_Calisma_Ozeti.pdf"
    ))).resolve()

    if not content.exists():
        print(f"HATA: İçerik bulunamadı: {content}", file=sys.stderr)
        return 2
    if not stylesheet.exists():
        print(f"HATA: Stil bulunamadı: {stylesheet}", file=sys.stderr)
        return 2

    if args.check:
        inventory = base / config.get("inventory", "data/source.json")
        coverage = base / config.get("coverage", "data/coverage.md")
        checker = ROOT / "engine/tools/check_coverage.py"
        result = subprocess.run(
            [
                sys.executable,
                str(checker),
                "--source-json",
                str(inventory),
                "--html",
                str(content),
                "--out",
                str(coverage),
            ]
        )
        if result.returncode != 0:
            print(
                "Kapsam denetimi geçmedi. data/coverage.md dosyasını inceleyin.",
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
    HTML(filename=str(content), base_url=str(content.parent) + "/").write_pdf(
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
