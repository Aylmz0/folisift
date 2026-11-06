#!/usr/bin/env python3
"""A-V4A acceptance harness: the v4AGENT matrix plus MCP guidance.

Run `--scope p0` during 0.22.1 and `--scope all` after 0.23.0.
"""

import argparse
import json
import os
import select
import subprocess
import time
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
BIN = Path(os.environ.get("FOLISIFT_BIN", REPO / "target/debug/folisift"))
checks = []


def record(name, ok, detail=""):
    checks.append({"check": name, "ok": bool(ok), "detail": str(detail)[:500]})
    print(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))


def pending(name, detail):
    checks.append({"check": name, "ok": True, "pending": True, "detail": str(detail)[:500]})
    print(f"PENDING {name}: {detail}")


class Client:
    def __init__(self, root):
        self.root = Path(root)

    def _request(self, method, params):
        payload = [
            {
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-11-25",
                    "capabilities": {},
                    "clientInfo": {"name": "v4-agent-check", "version": "0"},
                },
            },
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
            {"jsonrpc": "2.0", "id": 2, "method": method, "params": params},
        ]
        proc = subprocess.Popen(
            [str(BIN), "mcp", "--root", str(self.root)],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1,
        )
        try:
            proc.stdin.write("".join(json.dumps(item) + "\n" for item in payload))
            proc.stdin.flush()
            deadline = time.monotonic() + 300
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
                if response.get("id") != 2:
                    continue
                return response.get("result", {}), response.get("error")
            stderr = proc.stderr.read()[-500:] if proc.stderr is not None else ""
            return None, stderr or "no MCP response"
        finally:
            proc.stdin.close()
            proc.terminate()
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                proc.kill()

    def call(self, name, arguments):
        result, error = self._request("tools/call", {"name": name, "arguments": arguments})
        if error:
            return None, error
        if result.get("isError"):
            return None, result.get("content", [{}])[0].get("text", "tool error")
        value = result.get("structuredContent")
        if value is not None:
            return value, result
        text = result.get("content", [{}])[0].get("text", "")
        try:
            return json.loads(text), result
        except json.JSONDecodeError:
            return text, result

    def prompt(self, name):
        result, error = self._request("prompts/get", {"name": name})
        if error:
            return None, error
        return result.get("messages", [{}])[0].get("content", {}).get("text", ""), result


def run(args, timeout=600):
    return subprocess.run(
        args,
        cwd=str(REPO),
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
    )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--scope", choices=["p0", "all"], default="p0")
    args = parser.parse_args()
    client = Client(REPO)

    info, _ = client.call("workspace_info", {})
    record("T00 workspace_info", info is not None and info.get("version"), info)

    built, _ = client.call("coverage_manifest", {"action": "build", "guide": "pd0-pd1"})
    record("T07 generated manifest", built is not None and built.get("rows", 0) > 0, built)
    generated_path = REPO / "guides/pd0-pd1/data/coverage-manifest.generated.json"
    rows = json.loads(generated_path.read_text())["rows"] if generated_path.is_file() else []
    clean = all(
        len(row.get("heading", "")) <= 140
        and not row.get("heading", "").lower().startswith(("bu ", "bir ", "örnek "))
        for row in rows
    )
    record("T07 generated keys", bool(rows) and clean, {"rows": len(rows), "clean": clean})
    validate, _ = client.call(
        "coverage_manifest",
        {
            "action": "validate",
            "guide": "pd0-pd1",
            "manifest": "generated",
            "detail": "summary",
            "fields": ["per_source.sections.number"],
            "max_tokens": 50,
        },
    )
    record(
        "T08 nested budget",
        validate is not None
        and validate.get("truncated") is True
        and validate.get("omitted_total", 0) > 0,
        validate,
    )

    reads = []
    for source in ["PD0", "PD0.pdf", "sources/PD0.pdf", "data/blocks/PD0.json"]:
        value, _ = client.call(
            "read_section",
            {"guide": "pd0-pd1", "source": source, "heading": "0.1", "mode": "summary"},
        )
        reads.append(value.get("text", "")[:80] if isinstance(value, dict) else None)
    record("T27 read_section path forms", len(set(reads)) == 1 and reads[0], reads)

    spec = {
        "id": "check-flow",
        "kind": "flow",
        "title": "Check",
        "caption": "A check diagram",
        "alt": "A check diagram with one node",
        "source": "PD0, s. 1",
        "nodes": [{"id": "a", "label": "A"}],
        "edges": [],
    }
    diagram, _ = client.call("diagram", {"action": "validate", "spec": spec})
    record("T28 diagram validate without guide", diagram is not None and diagram.get("ok"), diagram)
    listing, _ = client.call("diagram", {"action": "status", "guide": "guides/pd3"})
    record("T28 diagram guides/<slug>", listing is not None and listing.get("ok") is not None, listing)

    plan, _ = client.call("guide_plan", {"guide": "pd0-pd1", "detail": "full"})
    mapped = [row for row in (plan or {}).get("sections", []) if row.get("source_pages")]
    record("T29 guide_plan source_pages", len(mapped) >= 8, {"mapped": len(mapped)})

    qa, _ = client.call("qa_guide", {"slug": "pd0-pd1", "detail": "summary"})
    visual = (qa or {}).get("visual") or {}
    record(
        "F07 visual gate truth",
        qa is not None
        and visual.get("basis") in {"candidates", "layout-blocks"}
        and visual.get("status") in {"passed", "advisory", "failed"},
        {key: visual.get(key) for key in ["basis", "status", "advisory", "figure_pages"]},
    )
    record(
        "F08 advisory semantics",
        visual.get("status") != "passed" or visual.get("advisory") is True,
        {key: visual.get(key) for key in ["status", "advisory", "missing_figures"]},
    )

    prompt, _ = client.prompt("start-here")
    record("F15 start-here prompt", bool(prompt and "workspace_info" in prompt), prompt)
    instructions, _ = client._request("prompts/list", {})
    record("F15 prompts list", instructions is not None, instructions)

    doctor, _ = client.call("doctor", {})
    ocr = next(
        (check for check in (doctor or {}).get("checks", []) if check.get("name") == "ocr"),
        {},
    )
    if args.scope == "p0":
        pending("T30 OCR availability", ocr)
        pending("F09 excluded", "0.23.0")
        pending("F10 OCR feature build", "0.23.0")
        pending("F11 image refs detail", "0.23.0")
        pending("F12 decorative semantics", "0.23.0")
        pending("F13 cards warning", "0.23.0")
        pending("F14 glossary regression", "0.23.0")
    else:
        doctor_result, _ = client.call("doctor", {})
        ocr_check = next(
            (item for item in (doctor_result or {}).get("checks", []) if item.get("name") == "ocr"),
            {},
        )
        record("F10 OCR doctor", bool(ocr_check), ocr_check)
        cards, _ = client.call("cards", {"slug": "pd2", "include_definitions": True})
        record(
            "F13 cards warning",
            cards is not None and "warning" in cards and "recall" in cards,
            cards,
        )

    payload = {
        "checks": checks,
        "passed": all(check.get("ok") for check in checks),
        "pending": sum(1 for check in checks if check.get("pending")),
        "scope": args.scope,
    }
    print(json.dumps(payload, ensure_ascii=False))
    raise SystemExit(0 if payload["passed"] else 1)


if __name__ == "__main__":
    main()
