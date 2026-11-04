#!/usr/bin/env python3
"""Final feedback v3 acceptance checks (A-V4-01..13, A-V4-17)."""
import json
import os
import select
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
EXTERNAL = Path(os.environ.get("FOLISIFT_V4_EXTERNAL", "/home/yilmaz/Documents/DersNot"))
_installed = Path.home() / ".local/bin/folisift"
BIN = Path(
    os.environ.get(
        "FOLISIFT_BIN",
        str(_installed if _installed.exists() else REPO / "target/debug/folisift"),
    )
)
checks = []


def record(name, ok, detail=""):
    checks.append({"check": name, "ok": bool(ok), "detail": str(detail)[:500]})
    print(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))


def skip(name, detail):
    checks.append({"check": name, "ok": True, "skipped": True, "detail": detail})
    print(f"SKIP {name}: {detail}")


class Client:
    def __init__(self, root, binary=None):
        self.root = Path(root)
        self.binary = Path(binary or BIN)
        self.messages = [
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-11-25",
                    "capabilities": {},
                    "clientInfo": {"name": "v4-check", "version": "0"},
                },
            },
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
        ]
        self.next_id = 2

    def call(self, name, arguments):
        request_id = self.next_id
        self.next_id += 1
        payload = self.messages + [
            {
                "jsonrpc": "2.0",
                "id": request_id,
                "method": "tools/call",
                "params": {"name": name, "arguments": arguments},
            }
        ]
        proc = subprocess.Popen(
            [str(self.binary), "mcp", "--root", str(self.root)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        try:
            assert proc.stdin is not None and proc.stdout is not None
            proc.stdin.write("".join(json.dumps(message) + "\n" for message in payload))
            proc.stdin.flush()
            deadline = time.monotonic() + 900
            while time.monotonic() < deadline:
                ready, _, _ = select.select([proc.stdout], [], [], 1.0)
                if not ready:
                    if proc.poll() is not None:
                        break
                    continue
                line = proc.stdout.readline()
                if not line:
                    break
                response = json.loads(line)
                if response.get("id") != request_id:
                    continue
                result = response.get("result", {})
                if result.get("isError"):
                    return None, result
                structured = result.get("structuredContent")
                if structured is not None:
                    return structured, result
                content = result.get("content", [{}])[0].get("text", "")
                try:
                    return json.loads(content), result
                except json.JSONDecodeError:
                    return content, result
            stderr = proc.stderr.read()[-500:] if proc.stderr is not None else ""
            return None, stderr or "no MCP response"
        finally:
            if proc.stdin is not None:
                proc.stdin.close()
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()


def run(args, cwd=REPO, timeout=900):
    return subprocess.run(
        args,
        cwd=str(cwd),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )


def json_stdout(proc):
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None


def temp_guide(source: Path):
    root = Path(tempfile.mkdtemp(prefix="folisift-v4-guide-", dir="/var/tmp"))
    (root / "sources").mkdir()
    (root / "guides/is-hukuku").mkdir(parents=True)
    (root / "output").mkdir()
    (root / "styles").symlink_to(REPO / "styles")
    src = root / "sources/is-hukuku.pdf"
    shutil.copy2(source, src)
    for name in ["config.json", "guide.md"]:
        shutil.copy2(EXTERNAL / "guides/is-hukuku" / name, root / "guides/is-hukuku" / name)
    return root


def cli(root, *args):
    return run([str(BIN), "--root", str(root), *args])


if EXTERNAL.is_dir():
    client = Client(EXTERNAL)
    qa_main, _ = client.call("qa_guide", {"slug": "is-hukuku"})
    qa_digital, _ = client.call("qa_guide", {"slug": "dijital-donusum-temelleri"})
    source = EXTERNAL / "sources/is-hukuku.pdf"
    if not source.is_file():
        skip("A-V4-01 generated rows", "is-hukuku source missing")
    else:
        root = temp_guide(source)
        try:
            built = cli(root, "coverage-manifest", "--guide", "is-hukuku", "--action", "build")
            generated_path = root / "guides/is-hukuku/data/coverage-manifest.generated.json"
            if built.returncode != 0 or not generated_path.is_file():
                record("A-V4-01 generated rows", False, (built.stderr or built.stdout)[-300:])
            else:
                data = json.loads(generated_path.read_text())
                rows = data.get("rows", [])
                headings = [row.get("heading", "") for row in rows]
                body_like = [heading for heading in headings if len(heading) > 120]
                has_444 = any(item.startswith("4.4.4") for item in headings)
                has_102 = any(item.startswith("10.2") for item in headings)
                record(
                    "A-V4-01 generated rows",
                    len(rows) == 138 and not body_like and has_444 and not has_102,
                    f"rows={len(rows)} body_like={len(body_like)} 4.4.4={has_444} 10.2={has_102}",
                )
                validated = cli(
                    root,
                    "coverage-manifest",
                    "--guide",
                    "is-hukuku",
                    "--action",
                    "validate",
                    "--manifest",
                    "generated",
                )
                value = json_stdout(validated)
                record(
                    "A-V4-02 generated validate",
                    validated.returncode == 0
                    and value is not None
                    and value.get("passed") is True
                    and value.get("claimed") == value.get("total")
                    and not value.get("missing"),
                    str({k: value.get(k) for k in ["total", "claimed", "missing", "passed"]})
                    if value
                    else (validated.stderr or validated.stdout)[-300:],
                )
                advisory = cli(
                    root, "coverage-manifest", "--guide", "is-hukuku", "--action", "validate"
                )
                record(
                    "A-V4-03 generated advisory",
                    advisory.returncode != 0
                    and "advisory" in (advisory.stderr + advisory.stdout),
                    (advisory.stderr or advisory.stdout)[-180:],
                )
                record(
                    "A-V4-03 combined precedence",
                    qa_main is not None
                    and qa_main.get("coverage_source") == "combined"
                    and (qa_main.get("coverage") or {}).get("total") == 141
                    and qa_digital is not None
                    and qa_digital.get("coverage_source") == "combined"
                    and (qa_digital.get("coverage") or {}).get("total") == 41,
                    f"main={qa_main and qa_main.get('coverage_source')} digital={qa_digital and qa_digital.get('coverage_source')}",
                )
        finally:
            shutil.rmtree(root, ignore_errors=True)

    chapter, _ = client.call(
        "chapter_brief",
        {"guide": "is-hukuku", "source": "is-hukuku", "heading": "4", "max_tokens": 400},
    )
    record(
        "A-V4-04 chapter brief",
        chapter is not None
        and chapter.get("level") == "chapter"
        and chapter.get("chars", 0) > 100
        and len(chapter.get("sections", [])) >= 3
        and 0 < chapter.get("estimated_tokens", 0) <= 400
        and chapter.get("summary", "") != chapter.get("heading", ""),
        str({k: chapter.get(k) for k in ["chars", "estimated_tokens", "sections"]} if chapter else None),
    )
    section, _ = client.call(
        "section_brief",
        {"guide": "is-hukuku", "source": "is-hukuku", "heading": "4.4.3", "max_tokens": 50},
    )
    heading, _ = client.call(
        "heading_brief",
        {"guide": "is-hukuku", "source": "is-hukuku", "heading": "4.4.3", "max_tokens": 20},
    )
    record(
        "A-V4-05 brief budgets",
        section is not None
        and heading is not None
        and section.get("estimated_tokens", 999) <= 50
        and heading.get("estimated_tokens", 999) <= 20
        and bool(section.get("summary"))
        and bool(heading.get("summary"))
        and section.get("next_offset", 0) > 0
        and heading.get("next_offset", 0) > 0,
        f"{section and section.get('estimated_tokens')}/{heading and heading.get('estimated_tokens')}",
    )
    validate, _ = client.call(
        "coverage_manifest",
        {
            "action": "validate",
            "guide": "is-hukuku",
            "manifest": "combined",
            "detail": "full",
            "max_tokens": 80,
            "fields": ["total", "claimed", "missing", "per_source.sections.number"],
        },
    )
    record(
        "A-V4-06 validate envelope",
        validate is not None
        and "total" in validate
        and "claimed" in validate
        and "missing" in validate
        and validate.get("truncated") is True
        and validate.get("max_tokens") == 80,
        str({k: validate.get(k) for k in ["total", "claimed", "truncated", "max_tokens"]} if validate else None),
    )
    hint, _ = client.call(
        "coverage_manifest",
        {
            "action": "hint",
            "guide": "dijital-donusum-temelleri",
            "detail": "summary",
            "max_tokens": 500,
            "fields": ["total", "covered", "likely_missing", "merged_heading_hits", "hints.heading", "hints.status", "hints.score"],
        },
    )
    hint_rows = (hint or {}).get("hints", [])
    merged_row = next(
        (row for row in hint_rows if "2.2 " in str(row.get("heading", ""))), {}
    )
    record(
        "A-V4-07 hint envelope",
        hint is not None
        and hint.get("merged_heading_hits", 0) >= 1
        and hint.get("likely_missing") == 0
        and merged_row.get("status") == "covered"
        and merged_row.get("score") == 1.0,
        str({k: hint.get(k) for k in ["total", "covered", "likely_missing", "merged_heading_hits"]} if hint else None),
    )
    brief, _ = client.call(
        "source_brief",
        {"guide": "is-hukuku", "source": "is-hukuku", "max_headings": 200},
    )
    rows = (brief or {}).get("headings", [])
    record(
        "A-V4-10 source brief counters",
        brief is not None
        and brief.get("headings_returned") == len(rows)
        and brief.get("headings_total") == 141
        and brief.get("heading_count") == 141
        and any("4.4.4" in row.get("heading", "") for row in rows)
        and any(row.get("heading", "").startswith("10.2") for row in rows),
        str({k: brief.get(k) for k in ["heading_count", "headings_total", "headings_returned", "review_added"]} if brief else None),
    )
    narrow, _ = client.call(
        "source_brief",
        {"guide": "is-hukuku", "source": "is-hukuku", "max_headings": 30},
    )
    record(
        "A-V4-10 source brief pagination",
        narrow is not None
        and narrow.get("headings_returned") == 30
        and narrow.get("truncated") is True
        and narrow.get("next") == 30,
        str({k: narrow.get(k) for k in ["headings_returned", "truncated", "next"]} if narrow else None),
    )
    record(
        "A-V4-08 excluded status",
        run(
            ["cargo", "test", "-q", "-p", "folisift-parse", "--lib", "excluded_rows"]
        ).returncode
        == 0,
        "excluded/merged report unit test",
    )
    record(
        "A-V4-09 merged heading parser",
        hint is not None and merged_row.get("score") == 1.0,
        str(merged_row),
    )
    citations, _ = client.call(
        "verify_citations", {"guide": "is-hukuku", "max_claims": 200, "detail": "full"}
    )
    multi = next(
        (claim for claim in (citations or {}).get("claims", []) if claim.get("reference_count", 0) >= 2),
        None,
    )
    record(
        "A-V4-12 multi-reference citations",
        multi is not None and len(multi.get("references", [])) >= 2,
        str({k: multi.get(k) for k in ["line", "reference_count"]} if multi else None),
    )
    coverage = (qa_main or {}).get("coverage") or {}
    record(
        "A-V4-13 weak sections",
        coverage.get("weak_section_chars") == 400 and len(coverage.get("weak_sections", [])) >= 1,
        f"weak={len(coverage.get('weak_sections', []))} threshold={coverage.get('weak_section_chars')}",
    )
    doc = (REPO / "docs/cookbook.md").read_text()
    record(
        "A-V4-17 workflow docs",
        "coverage-manifest.json" in doc
        and "adopt" in doc
        and "_draft.pdf" in doc
        and "weak_section_chars" in doc,
        "cookbook precedence/adopt/draft/weak",
    )
    scaffold = run(
        ["cargo", "test", "-q", "-p", "folisift-mcp", "--lib", "irac_skeleton"]
    )
    record(
        "A-V4-11 IRAC scaffold",
        scaffold.returncode == 0,
        "source id and ten-case skeleton unit test",
    )
    palettes = run([str(BIN), "--root", str(REPO), "styles", "palettes"])
    try:
        palette_rows = json.loads(palettes.stdout)
    except json.JSONDecodeError:
        palette_rows = []
    palette_ok = (
        len(palette_rows) >= 19
        and all(row.get("use") for row in palette_rows)
    )
    palette_root = Path(tempfile.mkdtemp(prefix="folisift-v4-palette-", dir="/var/tmp"))
    try:
        (palette_root / "sources").mkdir()
        (palette_root / "guides/pd3").mkdir(parents=True)
        (palette_root / "output").mkdir()
        (palette_root / "styles").symlink_to(REPO / "styles")
        shutil.copy2(REPO / "sources/PD3.pdf", palette_root / "sources/PD3.pdf")
        for name in ["config.json", "guide.md"]:
            shutil.copy2(REPO / "guides/pd3" / name, palette_root / "guides/pd3" / name)
        restyle = cli(
            palette_root,
            "styles",
            "restyle",
            "--guide",
            "pd3",
            "--palette",
            "forest-sage",
        )
        restyle_value = json_stdout(restyle)
        config = json.loads((palette_root / "guides/pd3/config.json").read_text())
        palette_ok = (
            palette_ok
            and restyle.returncode == 0
            and restyle_value is not None
            and restyle_value.get("content_unchanged") is True
            and restyle_value.get("palette") == "forest-sage"
            and config["style"]["palette"] == "forest-sage"
            and config["style"]["aesthetic"] == "reader"
        )
        record(
            "A-V4-18 palette switch",
            palette_ok,
            f"palettes={len(palette_rows)} restyle={restyle_value and restyle_value.get('palette')}",
        )
    finally:
        shutil.rmtree(palette_root, ignore_errors=True)
else:
    for name in [
        "A-V4-01 generated rows",
        "A-V4-02 generated validate",
        "A-V4-03 generated advisory",
        "A-V4-03 combined precedence",
        "A-V4-04 chapter brief",
        "A-V4-05 brief budgets",
        "A-V4-06 validate envelope",
        "A-V4-07 hint envelope",
        "A-V4-08 excluded status",
        "A-V4-09 merged heading parser",
        "A-V4-10 source brief counters",
        "A-V4-10 source brief pagination",
        "A-V4-11 IRAC scaffold",
        "A-V4-12 multi-reference citations",
        "A-V4-13 weak sections",
        "A-V4-17 workflow docs",
    ]:
        skip(name, f"{EXTERNAL} not found")

ok = all(item["ok"] for item in checks)
print(json.dumps({"acceptance": ok, "checks": checks}, ensure_ascii=False))
sys.exit(0 if ok else 1)
