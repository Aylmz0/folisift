#!/usr/bin/env python3
"""Development-only typography gate.

Reads the `audit` JSON for built PDFs and checks the role metrics introduced in
v0.11: body/code size, code/body ratio, line-length percentiles and long lines.
Paper is the labelled legacy exception; new styles must meet the thresholds.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/folisift"

CASES = [
    ("output/PD0-PD1_Calisma_Ozeti.pdf", "pd0-pd1", 22, 0.0, 0),
    ("output/PD2_Calisma_Ozeti.pdf", "pd2", 24, 0.90, 0),
    ("output/PD3_Calisma_Ozeti.pdf", "pd3", 22, 0.90, 0),
    ("output/PD2_Revizyon.pdf", "pd2-rev", 4, 0.90, 82),
    ("output/PD2_LISP_Bilgi_Organizatoru.pdf", "pd2-ko", 1, 0.0, 90),
]


def audit(pdf: Path, guide: str, pages: int) -> dict:
    result = subprocess.run(
        [
            str(BINARY),
            "audit",
            "--pdf",
            str(pdf),
            "--guide",
            guide,
            "--expected-pages",
            str(pages),
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    if result.returncode not in (0, 1):
        print(result.stderr, file=sys.stderr)
        raise SystemExit(result.returncode)
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as error:
        print(f"{pdf}: invalid audit JSON: {error}", file=sys.stderr)
        print(result.stdout, file=sys.stderr)
        raise SystemExit(1) from error


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--pdf", action="append", default=[])
    parser.add_argument("--guide", action="append", default=[])
    args = parser.parse_args()
    cases = CASES
    if args.pdf:
        if len(args.pdf) != len(args.guide):
            parser.error("--pdf and --guide must be paired")
        cases = [
            (pdf, guide, None, 0.0, 0)
            for pdf, guide in zip(args.pdf, args.guide)
        ]
    if not BINARY.is_file():
        print(f"missing {BINARY}; run cargo build -p folisift-cli", file=sys.stderr)
        return 2
    failed = False
    for pdf, guide, pages, min_ratio, max_line in cases:
        path = ROOT / pdf
        if not path.is_file():
            print(f"{pdf}: missing", file=sys.stderr)
            failed = True
            continue
        expected = pages if pages is not None else audit(path, guide, 1)["pages"]
        report = audit(path, guide, expected)
        ratio = report.get("code_body_ratio")
        p90 = report.get("line_chars_p90", 0)
        long_pages = report.get("long_line_pages", [])
        histogram = ", ".join(
            f"{item['size']:g}pt×{item['chars']}" for item in report["font_sizes"][-5:]
        )
        print(
            f"{pdf}: body {report['body_font_size']:g} pt, "
            f"code {report.get('code_font_size') or '—'}, "
            f"ratio {ratio if ratio is None else round(ratio, 3)}, "
            f"p50 {report['line_chars_p50']}, p90 {p90}, "
            f"long {long_pages or '—'}"
        )
        print(f"  largest roles: {histogram}")
        if not report.get("passed"):
            print(f"  audit issues: {report.get('issues')}", file=sys.stderr)
            failed = True
        if min_ratio and (ratio is None or ratio < min_ratio):
            print(
                f"  code/body ratio {ratio} is below {min_ratio}",
                file=sys.stderr,
            )
            failed = True
        if max_line and p90 > max_line:
            print(f"  p90 {p90} exceeds {max_line}", file=sys.stderr)
            failed = True
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
