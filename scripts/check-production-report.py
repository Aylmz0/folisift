#!/usr/bin/env python3
"""Development-only MCP production-report smoke test.

Starts the local Folisift MCP, calls prepare_guide and build_guide for pd2,
and verifies that the report contains the fields the OpenCode loop consumes.
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BINARY = ROOT / "target/debug/folisift"


def main() -> int:
    if not BINARY.is_file():
        print(f"missing {BINARY}; run cargo build -p folisift-cli", file=sys.stderr)
        return 2
    proc = subprocess.Popen(
        [str(BINARY), "mcp", "--root", str(ROOT)],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )

    def send(value: dict) -> None:
        assert proc.stdin is not None
        proc.stdin.write(json.dumps(value) + "\n")
        proc.stdin.flush()

    def recv() -> dict:
        assert proc.stdout is not None
        return json.loads(proc.stdout.readline())

    send(
        {
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-25",
                "capabilities": {},
                "clientInfo": {"name": "production-report", "version": "0"},
            },
        }
    )
    recv()
    send({"jsonrpc": "2.0", "method": "notifications/initialized"})
    guides = [
        {
            "slug": "pd2",
            "structure": "a-g",
            "style": "reader",
            "voice": "academic",
        },
        {
            "slug": "pd2-rev",
            "structure": "revision-sheet",
            "style": "revision",
            "voice": "plain",
        },
        {
            "slug": "pd2-ko",
            "structure": "knowledge-organiser",
            "style": "revision",
            "voice": "plain",
        },
    ]
    reports = []
    for index, guide in enumerate(guides, start=2):
        send(
            {
                "jsonrpc": "2.0",
                "id": index,
                "method": "tools/call",
                "params": {
                    "name": "prepare_guide",
                    "arguments": {
                        "slug": guide["slug"],
                        "title": "Programlama Dilleri — Bölüm 2",
                        "course": "BMÜ325",
                        "sources": ["sources/PD2.pdf"],
                        "domain": "cs",
                        "language": "tr",
                        "structure": guide["structure"],
                        "style": guide["style"],
                        "voice": guide["voice"],
                    },
                },
            }
        )
        prepared = recv()
        if prepared.get("result", {}).get("isError"):
            print(f"prepare_guide {guide['slug']} failed: {prepared}", file=sys.stderr)
            return 1
        prepared = prepared["result"]["structuredContent"]
        required_prepare = ["profiles", "prompt_fragment", "style_source", "visual_audit"]
        missing = [key for key in required_prepare if key not in prepared]
        if missing:
            print(f"prepare_guide {guide['slug']} missing: {missing}", file=sys.stderr)
            return 1
        send(
            {
                "jsonrpc": "2.0",
                "id": index + 100,
                "method": "tools/call",
                "params": {
                    "name": "build_guide",
                    "arguments": {"slug": guide["slug"], "check": True},
                },
            }
        )
        built = recv()
        if built.get("result", {}).get("isError"):
            print(f"build_guide {guide['slug']} failed: {built}", file=sys.stderr)
            return 1
        built = built["result"]["structuredContent"]
        required_build = [
            "coverage",
            "visual",
            "stats",
            "pages",
            "style_rationale",
            "passed",
        ]
        missing = [key for key in required_build if key not in built]
        if missing:
            print(f"build_guide {guide['slug']} missing: {missing}", file=sys.stderr)
            return 1
        if not built["passed"]:
            print(f"build_guide {guide['slug']} did not pass its gates", file=sys.stderr)
            return 1
        if not str(built.get("style_rationale") or "").strip():
            print(f"build_guide {guide['slug']} has no style_rationale", file=sys.stderr)
            return 1
        reports.append(
            {
                "slug": guide["slug"],
                "coverage": built["coverage"],
                "visual": built["visual"],
                "stats": built["stats"],
                "pages": built["pages"],
                "style": built["style"],
                "style_rationale": built["style_rationale"],
            }
        )
    print(json.dumps(reports, ensure_ascii=False, indent=2))
    proc.stdin.close()
    proc.wait(timeout=60)
    return proc.returncode


if __name__ == "__main__":
    raise SystemExit(main())
