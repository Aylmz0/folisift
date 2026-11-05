#!/usr/bin/env python3
"""A-V5 acceptance: source visual candidates, decisions and asset maintenance."""

from __future__ import annotations

import json
import os
import select
import shutil
import subprocess
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
BIN = Path(os.environ.get("FOLISIFT_BIN", REPO / "target/debug/folisift"))
CHECKS: list[dict[str, object]] = []


def record(name: str, ok: bool, detail: str) -> None:
    CHECKS.append({"check": name, "ok": bool(ok), "detail": detail})
    print(("PASS " if ok else "FAIL ") + name + ": " + detail)


def run(args: list[str], root: Path | None = None) -> subprocess.CompletedProcess[str]:
    command = [str(BIN)]
    if root is not None:
        command += ["--root", str(root)]
    command += args
    return subprocess.run(command, text=True, capture_output=True)


def json_stdout(proc: subprocess.CompletedProcess[str]) -> dict | list | None:
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None


def workspace(prefix: str, slug: str, source: Path, config: dict, guide: str) -> Path:
    root = Path(tempfile.mkdtemp(prefix=prefix, dir="/var/tmp"))
    (root / "sources").mkdir()
    (root / "output").mkdir()
    (root / "guides" / slug).mkdir(parents=True)
    (root / "styles").symlink_to(REPO / "styles")
    shutil.copy2(source, root / "sources" / source.name)
    (root / "guides" / slug / "config.json").write_text(json.dumps(config, ensure_ascii=False, indent=2))
    (root / "guides" / slug / "guide.md").write_text(guide)
    return root


def main() -> int:
    # 1. Candidate enumeration and classification on the real data-mining PDF.
    source = REPO / "sources" / "verimadenciliği.pdf"
    args = [
        "visual",
        "candidates",
        "--source",
        "sources/verimadenciliği.pdf",
        "--pages",
        "14,45,71",
        "--min-score",
        "0",
        "--limit",
        "80",
    ]
    first = run(args, REPO)
    second = run(args, REPO)
    data = json_stdout(first)
    record(
        "A-V5-01 candidate report",
        first.returncode == 0
        and isinstance(data, dict)
        and bool(data.get("candidates"))
        and data.get("source_digest")
        and data.get("scorer_version") == 1,
        str({k: data.get(k) for k in ["source", "scorer_version", "counts"]} if isinstance(data, dict) else first.stderr[-200:]),
    )
    record(
        "A-V5-02 deterministic report",
        first.returncode == 0 and second.returncode == 0 and first.stdout == second.stdout,
        "two runs are byte-identical",
    )
    candidates = data.get("candidates", []) if isinstance(data, dict) else []
    page_45 = [c for c in candidates if c.get("page") == 45]
    page_71 = [c for c in candidates if c.get("page") == 71]
    page_14 = [c for c in candidates if c.get("page") == 14]
    record(
        "A-V5-03 D2 concept",
        any(c.get("reconstruction", {}).get("kind") == "d2_concept" for c in page_45),
        str([(c.get("id"), c.get("reconstruction", {}).get("kind")) for c in page_45]),
    )
    record(
        "A-V5-04 quantitative chart",
        any(c.get("reconstruction", {}).get("kind") in {"unique", "d2_schematic"} for c in page_71),
        str([(c.get("id"), c.get("reconstruction", {}).get("kind")) for c in page_71]),
    )
    record(
        "A-V5-05 panel grouping",
        any(c.get("group_members", 1) >= 2 for c in page_14),
        str([(c.get("id"), c.get("group_members")) for c in page_14]),
    )
    top5 = candidates[:5]
    record(
        "A-V5-06 no decorative top-5",
        all(c.get("relevance", {}).get("label") != "decorative" for c in top5),
        str([(c.get("id"), c.get("relevance", {}).get("label")) for c in top5]),
    )
    record(
        "A-V5-07 bounded text-only report",
        all(
            isinstance(c.get("context"), dict)
            and "bbox" in c
            and "score_parts" in c
            and "review_mode" in c
            for c in candidates
        )
        and not any(c.get("image_bytes") for c in candidates),
        "candidate fields present; no image bytes",
    )

    # 2. Native extraction and quality metadata.
    pd3_root = Path(tempfile.mkdtemp(prefix="folisift-v5-pd3-", dir="/var/tmp"))
    try:
        (pd3_root / "sources").mkdir()
        (pd3_root / "output").mkdir()
        (pd3_root / "guides/pd3").mkdir(parents=True)
        (pd3_root / "styles").symlink_to(REPO / "styles")
        shutil.copy2(REPO / "sources/PD3.pdf", pd3_root / "sources/PD3.pdf")
        shutil.copy2(REPO / "guides/pd3/config.json", pd3_root / "guides/pd3/config.json")
        shutil.copy2(REPO / "guides/pd3/guide.md", pd3_root / "guides/pd3/guide.md")
        shutil.copytree(REPO / "guides/pd3/diagrams", pd3_root / "guides/pd3/diagrams")
        extract = run(
            [
                "visual",
                "extract",
                "--guide",
                "pd3",
                "--source",
                "sources/PD3.pdf",
                "--page",
                "76",
                "--image-id",
                "p76_1",
                "--out",
                "data/figures/p76_1.png",
            ],
            pd3_root,
        )
        value = json_stdout(extract)
        record(
            "A-V5-08 native extraction",
            extract.returncode == 0
            and isinstance(value, dict)
            and len(str(value.get("hash", ""))) == 64
            and value.get("local_only") is True
            and (pd3_root / "guides/pd3" / value["asset"]).is_file(),
            str({k: value.get(k) for k in ["asset", "hash", "width", "height", "effective_dpi"]} if isinstance(value, dict) else extract.stderr[-200:]),
        )
        decision = run(
            [
                "visual",
                "decision",
                "--guide",
                "pd3",
                "--source",
                "sources/PD3.pdf",
                "--page",
                "76",
                "--candidate",
                "PD3:p76:img1",
                "--decision",
                "redraw",
                "--reason",
                "Sözdizim grafiği D2 concept diagramına dönüştürüldü.",
                "--asset",
                "diagrams/sozdizim-grafigi.json",
                "--redraw-kind",
                "concept",
                "--caption",
                "Sözdizim grafiği (şematik)",
                "--alt",
                "Sözdizim grafiği; ifade, terim ve faktör düğümlerini gösteren akış",
                "--priority",
                "high",
            ],
            pd3_root,
        )
        outcome = json_stdout(decision)
        audit = json.loads((pd3_root / "guides/pd3/visual-audit.json").read_text())
        record(
            "A-V5-09 decision + audit",
            decision.returncode == 0
            and isinstance(outcome, dict)
            and outcome.get("content_unchanged") is True
            and outcome.get("markdown", "").startswith("::: diagram")
            and audit["source_pages"][0]["decision"] == "redraw"
            and audit["source_pages"][0]["redraw_kind"] == "concept",
            str({k: outcome.get(k) for k in ["decision", "markdown"]} if isinstance(outcome, dict) else decision.stderr[-200:]),
        )
    finally:
        shutil.rmtree(pd3_root, ignore_errors=True)

    # 3. Crop materialization, missing-asset check and restoration.
    config = {
        "schema_version": 3,
        "slug": "veri-smoke",
        "title": "Veri Madenciliği Görsel Smoke",
        "language": "tr",
        "content": "guide.md",
        "stylesheet": None,
        "output": "../../output/veri-smoke.pdf",
        "sources": ["../../sources/verimadenciliği.pdf"],
        "inventory": "data/source.json",
        "coverage": "data/coverage.md",
        "target_pages": "2-4",
        "theme": "reader",
        "style": {
            "aesthetic": "reader",
            "structure": "concept-summary",
            "language": "tr",
            "source_language": "tr",
            "voice": "academic",
            "study_mode": "exam",
            "domain": "cs",
            "citation": "page-ref",
        },
        "style_source": "explicit",
        "style_rationale": "Visual smoke fixture.",
    }
    guide = "# Veri Madenciliği\n\n## Sınıflandırma\n\nModel oluşturma ve geçerleme.\n"
    root = workspace("folisift-v5-crop-", "veri-smoke", source, config, guide)
    try:
        crop = run(
            [
                "visual",
                "decision",
                "--guide",
                "veri-smoke",
                "--source",
                "sources/verimadenciliği.pdf",
                "--page",
                "45",
                "--candidate",
                "verimadenciliği:p45:vec1",
                "--decision",
                "embed_crop",
                "--reason",
                "Sınıflandırma akışı kaynak görselden alındı.",
                "--caption",
                "Sınıflandırma akışı: öğrenme, geçerleme ve sınama",
                "--alt",
                "Veri kümesinden model oluşturma, geçerleme ve sınama aşamalarını gösteren akış",
                "--priority",
                "high",
            ],
            root,
        )
        outcome = json_stdout(crop)
        asset = root / "guides/veri-smoke" / str(outcome.get("asset", "")) if isinstance(outcome, dict) else None
        record(
            "A-V5-10 crop decision",
            crop.returncode == 0
            and isinstance(outcome, dict)
            and outcome.get("markdown", "").startswith("::: figure")
            and asset is not None
            and asset.is_file(),
            str({k: outcome.get(k) for k in ["asset", "hash"]} if isinstance(outcome, dict) else crop.stderr[-200:]),
        )
        check = run(["visual", "assets", "--guide", "veri-smoke", "--action", "check"], root)
        record("A-V5-11 asset check", check.returncode == 0, check.stdout.strip()[-200:])
        if asset is not None:
            asset.unlink()
        missing = run(["visual", "assets", "--guide", "veri-smoke", "--action", "check"], root)
        materialize = run(["visual", "assets", "--guide", "veri-smoke", "--action", "materialize"], root)
        restored = run(["visual", "assets", "--guide", "veri-smoke", "--action", "check"], root)
        record(
            "A-V5-12 materialize restores",
            missing.returncode != 0 and materialize.returncode == 0 and restored.returncode == 0,
            f"missing={missing.returncode} materialize={materialize.returncode} restored={restored.returncode}",
        )
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # 4. Summary mode is bounded.
    summary = run(
        [
            "visual",
            "candidates",
            "--source",
            "sources/verimadenciliği.pdf",
            "--pages",
            "14",
            "--min-score",
            "0",
            "--detail",
            "summary",
        ],
        REPO,
    )
    summary_value = json_stdout(summary)
    record(
        "A-V5-13 summary bounded",
        summary.returncode == 0
        and isinstance(summary_value, dict)
        and len(summary_value.get("candidates", [])) <= 5,
        str(len(summary_value.get("candidates", [])) if isinstance(summary_value, dict) else summary.stderr[-200:]),
    )

    print(json.dumps({"acceptance": all(check["ok"] for check in CHECKS), "checks": CHECKS}))
    return 0 if all(check["ok"] for check in CHECKS) else 1


if __name__ == "__main__":
    raise SystemExit(main())
