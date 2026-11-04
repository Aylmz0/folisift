#!/usr/bin/env python3
"""External feedback-v2 acceptance checks (A-V2-01..18)."""
import json
import os
import select
import subprocess
import sys
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
ROOT = Path(os.environ.get("FOLISIFT_V2_WORKSPACE", "/home/yilmaz/Documents/DersNot"))
_installed = Path.home() / ".local/bin/folisift"
BIN = os.environ.get(
    "FOLISIFT_BIN",
    str(_installed if _installed.exists() else Path(__file__).resolve().parents[1] / "target/debug/folisift"),
)
checks = []


def record(name, ok, detail=""):
    checks.append({"check": name, "ok": bool(ok), "detail": detail})
    print(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))


def client():
    messages = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "v2-check", "version": "0"}}},
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
    ]
    next_id = 2

    def call(name, arguments):
        nonlocal next_id
        request_id = next_id
        next_id += 1
        payload = messages + [{"jsonrpc": "2.0", "id": request_id, "method": "tools/call", "params": {"name": name, "arguments": arguments}}]
        proc = subprocess.Popen(
            [BIN, "mcp", "--root", str(ROOT)],
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
            deadline = time.monotonic() + 600
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

    call.initialize = lambda: (messages, next_id)
    return call


call = client()
if not ROOT.is_dir():
    print(f"skip: {ROOT} not found")
    sys.exit(0)

# A01/A02/A17
main = "is-hukuku"
value, _ = call("coverage_manifest", {"action": "validate", "guide": main, "manifest": "review"})
record("A-V2-01 review 141/141", value and value.get("passed") and value.get("total") == 141 and value.get("claimed") == 141 and not value.get("missing") and not value.get("empty_sections"), str(value)[:300])
qa_main, _ = call("qa_guide", {"slug": main})
record("A-V2-02 qa coverage 141", qa_main and (qa_main.get("coverage") or {}).get("total") == 141 and qa_main.get("coverage_source") == "review", str((qa_main or {}).get("coverage_source")))
brief, _ = call("source_brief", {"guide": main, "source": main, "max_headings": 200})
record("A-V2-17 source_brief 141", brief and brief.get("headings_total") == 141 and brief.get("review_added") == 2, str(brief.get("headings_total") if brief else None))

# A03
digital = "dijital-donusum-temelleri"
qa_digital, _ = call("qa_guide", {"slug": digital})
record("A-V2-03 digital canonical 41", qa_digital and (qa_digital.get("coverage") or {}).get("total") == 41 and qa_digital.get("passed"), str((qa_digital or {}).get("coverage_source")))

# A04/A05
section, _ = call("section_brief", {"guide": main, "source": main, "heading": "4.4.3", "max_tokens": 50})
heading, _ = call("heading_brief", {"guide": main, "source": main, "heading": "4.4.3", "max_tokens": 20})
record("A-V2-04 brief budgets", section and heading and section.get("truncated") and heading.get("truncated") and section.get("estimated_tokens", 999) < 200, f"{section.get('estimated_tokens')}/{heading.get('estimated_tokens')}")
chapter, _ = call("chapter_brief", {"guide": main, "source": main, "heading": "4", "max_tokens": 200})
record("A-V2-05 chapter exact", chapter and str(chapter.get("heading", "")).startswith("4.") and "4857" in str(chapter.get("heading")), str((chapter or {}).get("heading")))

# A06
doctor, _ = call("doctor", {})
model_check = next((check for check in (doctor or {}).get("checks", []) if check.get("name") == "semantic-model"), {})
topics, _ = call("source_topics", {"guide": main, "semantic": True, "k": 5})
topics_ok = topics and (
    topics.get("method") == "model2vec"
    or (not model_check.get("ok") and topics.get("fallback") and topics.get("warning"))
)
record("A-V2-06 semantic topics", topics_ok, str((topics or {}).get("method")))

# A07/A08
claims, _ = call("claims", {"slug": main, "detail": "summary", "max_tokens": 1200})
record("A-V2-07 claims budget", claims and len(json.dumps(claims)) // 4 <= 1200, str(len(json.dumps(claims)) // 4 if claims else None))
graph, _ = call("graph", {"action": "query", "guide": main, "query": "fesih", "max_tokens": 800})
record("A-V2-08 graph budget", graph and len(json.dumps(graph)) // 4 <= 800 and all("bbox" not in node for node in graph.get("nodes", [])), str(len(json.dumps(graph)) // 4 if graph else None))

# A09
irac = "is-hukuku-irac"
status, _ = call("guide_status", {"slug": irac})
listing, _ = call("list_guides", {})
row = next((item for item in (listing or {}).get("guides", []) if item.get("slug") == irac), {})
record("A-V2-09 IRAC metrics", status and status.get("case_count") == 10 and status.get("section_count") == 14 and row.get("case_count") == 10 and row.get("section_count") == 14, f"{status and (status.get('case_count'), status.get('section_count'))}")

# A10
citations, _ = call("verify_citations", {"guide": main, "max_claims": 20})
rows = (citations or {}).get("claims", [])
bad = [row for row in rows if row.get("verified") and not row.get("snippet")]
header = [row for row in rows if row.get("snippet") and "Ders Notları" in row.get("snippet", "")]
record("A-V2-10 citation quality", citations is not None and not bad and not header, f"{len(rows)} checked")

# A11
doctor, _ = call("doctor", {})
d2 = next((check for check in (doctor or {}).get("checks", []) if check.get("name") == "d2"), {})
record("A-V2-11 D2 discovery", doctor and doctor.get("ok") and d2.get("ok"), str(d2.get("detail")))

# A12
table, _ = call("table_extract", {"source": "sources/Ders_Notu_1_VİZE.pdf", "page": 26, "guide": digital})
visual, _ = call("visual_brief", {"guide": digital, "source": "Ders_Notu_1_VİZE", "page": 26})
record("A-V2-12 visual taxonomy", table and table.get("reason") and visual and visual.get("role") == "visual-evidence-required", f"{table and table.get('mode')}/{visual and visual.get('role')}")

# A13
style, _ = call("style", {"action": "estimate", "guide": irac, "style": "accessible", "target_pages": "10-12"})
record("A-V2-13 style over_budget", style and style.get("over_budget") is True, str((style or {}).get("estimated_pages")))

# A14
cards = (qa_main or {}).get("cards", {})
record("A-V2-14 cards parity", cards.get("total") == 9 and cards.get("definitions") == 1, str(cards))

# A15: restyle a disposable copy; content must not change.
import shutil
import tempfile

tmp = Path(tempfile.mkdtemp(prefix="folisift-v2-restyle-"))
try:
    (tmp / "sources").mkdir()
    (tmp / "sources/PD3.pdf").write_bytes(b"%PDF-1.4\n")
    (tmp / "guides/pd3").mkdir(parents=True)
    shutil.copy2(REPO / "guides/pd3/config.json", tmp / "guides/pd3/config.json")
    shutil.copy2(REPO / "guides/pd3/guide.md", tmp / "guides/pd3/guide.md")
    (tmp / "styles").symlink_to(REPO / "styles")
    restyle = subprocess.run(
        [BIN, "--root", str(tmp), "styles", "restyle", "--guide", "pd3",
         "--style", "reader", "--palette", "warm-paper", "--fonts", "editorial"],
        stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True,
    )
    try:
        restyle_value = json.loads(restyle.stdout)
    except json.JSONDecodeError:
        restyle_value = {}
    record("A-V2-15 restyle hash", restyle.returncode == 0 and restyle_value.get("content_unchanged") is True, str(restyle_value.get("content_unchanged")))
finally:
    shutil.rmtree(tmp, ignore_errors=True)

# A16
scaffold_test = subprocess.run(
    ["cargo", "test", "-q", "-p", "folisift-mcp", "--lib", "irac_skeleton"],
    cwd=str(Path(__file__).resolve().parents[1]),
    stdout=subprocess.DEVNULL,
    stderr=subprocess.DEVNULL,
)
record("A-V2-16 IRAC scaffold", scaffold_test.returncode == 0, "skeleton unit test")
# A15 is an explicit manual check because it rewrites guide config.

repo = Path(__file__).resolve().parents[1]
repo_passed = True
for slug in ["pd0-pd1", "pd2", "pd3", "pd2-rev", "pd2-ko"]:
    run = subprocess.run(
        [BIN, "--root", str(repo), "qa", "--guide", slug],
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    try:
        repo_passed = repo_passed and json.loads(run.stdout).get("passed") is True
    except json.JSONDecodeError:
        repo_passed = False
record("A-V2-18 full regression", qa_main and qa_digital and repo_passed, "external + repo guides")

ok = all(item["ok"] for item in checks)
print(json.dumps({"acceptance": ok, "checks": checks}, ensure_ascii=False))
sys.exit(0 if ok else 1)
