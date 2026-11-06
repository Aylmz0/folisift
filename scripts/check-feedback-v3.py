#!/usr/bin/env python3
"""Visual evidence and chart-policy acceptance checks (A-V3-01..12)."""
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
EXTERNAL = Path(os.environ.get("FOLISIFT_V3_EXTERNAL", "/home/yilmaz/Documents/DersNot"))
DATA_MINING = REPO / "sources/verimadenciliği.pdf"
_installed = Path.home() / ".local/bin/folisift"
BIN = Path(
    os.environ.get(
        "FOLISIFT_BIN",
        str(_installed if _installed.exists() else REPO / "target/debug/folisift"),
    )
)
INSTALLED = BIN == _installed or os.environ.get("FOLISIFT_BIN") == str(_installed)
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
                    "clientInfo": {"name": "v3-check", "version": "0"},
                },
            },
            {"jsonrpc": "2.0", "method": "notifications/initialized"},
        ]
        self.next_id = 2

    def call(self, name, arguments):
        request_id = self.next_id
        self.next_id += 1
        messages = self.messages + [
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
            proc.stdin.write("".join(json.dumps(message) + "\n" for message in messages))
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
        args, cwd=str(cwd), text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout
    )


def json_stdout(proc):
    try:
        return json.loads(proc.stdout)
    except json.JSONDecodeError:
        return None


def extract_pages(source, pages, target):
    try:
        import fitz  # type: ignore
    except ImportError:
        fitz = None
    target.parent.mkdir(parents=True, exist_ok=True)
    if fitz is not None:
        document = fitz.open(str(source))
        excerpt = fitz.open()
        for page in pages:
            excerpt.insert_pdf(document, from_page=page - 1, to_page=page - 1)
        excerpt.save(str(target))
        excerpt.close()
        document.close()
        return True
    if not shutil.which("pdfseparate") or not shutil.which("pdfunite"):
        return False
    temp = target.parent / "pages"
    temp.mkdir(exist_ok=True)
    parts = []
    for page in pages:
        part = temp / f"page-{page}.pdf"
        proc = run(["pdfseparate", "-f", str(page), "-l", str(page), str(source), str(part)])
        if proc.returncode != 0:
            return False
        parts.append(str(part))
    proc = run(["pdfunite", *parts, str(target)])
    return proc.returncode == 0


def pdf_text(path):
    try:
        import fitz  # type: ignore
    except ImportError:
        fitz = None
    if fitz is not None:
        document = fitz.open(str(path))
        text = "\n".join(page.get_text() for page in document)
        document.close()
        return text
    if shutil.which("pdftotext"):
        proc = run(["pdftotext", str(path), "-"])
        if proc.returncode == 0:
            return proc.stdout
    return ""


def setup_smoke_workspace(source):
    root = Path(tempfile.mkdtemp(prefix="folisift-v3-smoke-", dir="/var/tmp"))
    (root / "sources").mkdir()
    (root / "guides/veri-smoke").mkdir(parents=True)
    (root / "output").mkdir()
    (root / "styles").symlink_to(REPO / "styles")
    excerpt = root / "sources/veri-smoke.pdf"
    if not extract_pages(source, [14, 45], excerpt):
        shutil.rmtree(root, ignore_errors=True)
        return None
    guide = root / "guides/veri-smoke"
    config = {
        "schema_version": 3,
        "slug": "veri-smoke",
        "title": "Veri Madenciliği Görsel Smoke",
        "language": "tr",
        "content": "guide.md",
        "stylesheet": None,
        "output": "../../output/veri-smoke.pdf",
        "sources": ["../../sources/veri-smoke.pdf"],
        "inventory": "data/source.json",
        "coverage": "data/coverage.md",
        "target_pages": "2-4",
        "review_intervals": [1, 3, 7, 14],
        "theme": "reader",
        "style": {
            "aesthetic": "reader",
            "structure": "concept-summary",
            "language": "tr",
            "source_language": "tr",
            "voice": "plain",
            "study_mode": "exam",
            "domain": "cs",
            "citation": "page-ref",
        },
        "style_source": "explicit",
        "style_rationale": "Görsel kanıt smoke kabul testi.",
        "audit": {"visual_gate": "required"},
    }
    (guide / "config.json").write_text(json.dumps(config, ensure_ascii=False, indent=2) + "\n")
    (guide / "guide.md").write_text(
        '''---
title: "Veri Madenciliği Görsel Smoke"
subtitle: "Görsel kanıt kabul testi"
course: "Kabul testi"
language: tr
sources: [veri-smoke]
source_scope: "veri-smoke:1-2"
running: "Görsel Smoke"
---

## Büyük resim

Veri madenciliğinde dağılım ve ROC grafikleri, bir ölçümün yayılımını ve bir
sınıflandırıcının eşik davranışını okumak için kullanılır. Bu sayfa, kaynak
sayfalardaki görselleri yeniden çizmeden; eksenleri, eğilimi ve sınav tuzağını
açıklar. Amaç grafiği süslemek değil, grafikten hangi sınırlı sonucun
çıkarılabileceğini göstermektir. [veri-smoke, s. 1]

## Ana kavramlar

Dağılım grafiği tek değişkenli bir ölçümün yoğunluğunu gösterir. Yatay eksen
ölçüm değerini, dikey eksen yoğunluğu taşır. Eğrinin tek tepeli ve sağa çarpık
olması, merkez ile kuyruğun aynı yerde olmadığını anlatır. ROC eğrisi ise
sınıflandırma eşiği değiştikçe yanlış pozitif oranı ile doğru pozitif oranı
arasındaki ilişkiyi gösterir. Eşik yükseldikçe eğri üzerinde farklı bir noktaya
geçilir; tek bir nokta modelin tamamını temsil etmez. [veri-smoke, s. 2]

::: table caption="Dağılım grafiğinin okunması" source="veri-smoke, s. 1"
| Öğe | Anlam | Sınav notu |
|---|---|---|
| Yatay eksen | Ölçüm değeri | Birim kaynakta belirtilir |
| Dikey eksen | Yoğunluk | Frekans değil |
| Kuyruk | Sağa çarpıklık | Ortalama medyanın sağında |
:::

## Mekanizmalar

Bir dağılım grafiğinde tepe noktası en sık görülen değer aralığını, kuyruk ise
uç değerlerin yönünü anlatır. ROC eğrisinde eşik düştükçe daha fazla örnek
pozitif sınıfa atanır; bu, TPR ile FPR arasındaki dengeyi değiştirir. Grafiğin
şekli, sınıflandırıcının hangi eşik aralığında daha dengeli davrandığını
gösterir; sayısal değerler kaynakta yazmıyorsa uydurulmaz. [veri-smoke, s. 2]

## Örnekler ve karşılaştırmalar

::: example label="Eksen okuma"
Yatay eksen FPR, dikey eksen TPR ise eşik yükseldikçe eğri boyunca hareket
edilir; tek bir nokta tüm modeli temsil etmez. [veri-smoke, s. 2]
:::

## Bağlantılar

Dağılım grafiği ölçümün yayılımını, ROC eğrisi ise sınıflandırma eşiğinin
maliyetini gösterir. İkisi de kaynak sayfadaki görselden okunur; sayısal veri
uydurulmaz. Dağılımın kuyruğu aykırı değerleri, ROC eğrisinin dirseği ise
dengeli eşik aralığını hatırlatır. [veri-smoke, s. 2]

## Kendini sına

::: recall
Dağılım grafiğinde yatay ve dikey eksen neyi gösterir?
:::

::: answer
Yatay eksen ölçüm değerini, dikey eksen yoğunluğu gösterir. [veri-smoke, s. 1]
:::

::: critical label="SINAV TUZAĞI"
ROC eğrisini D2 ile yeniden çizme. D2 kavramsal akış içindir; eksenli grafik
kaynak kırpması veya metin açıklamasıyla verilir. [veri-smoke, s. 2]
:::
''',
        encoding="utf-8",
    )
    return root


if not DATA_MINING.is_file():
    skip("A-V3-01 visual evidence", f"{DATA_MINING} not found")
    skip("A-V3-02 bounded evidence", "data-mining source not found")
    skip("A-V3-03 required gate", "data-mining source not found")
    skip("A-V3-04 6-up evidence", "data-mining source not found")
    skip("A-V3-09 table fallback", "data-mining source not found")
    skip("A-V3-12 smoke guide", "data-mining source not found")
else:
    client = Client(REPO)
    evidence, _ = client.call(
        "visual_evidence",
        {"source": "sources/verimadenciliği.pdf", "pages": "14,45,71"},
    )
    visual = (evidence or {}).get("visual_evidence") or {}
    pages = {item.get("page"): item for item in visual.get("items", [])}
    required_pages = [14, 45, 71]
    ok = all(page in pages for page in required_pages) and all(
        pages[page].get("table_likely") and pages[page].get("chart_likely")
        for page in required_pages
    )
    record("A-V3-01 visual evidence", ok, str({k: visual.get(k) for k in ["figure_pages", "table_pages", "chart_pages"]}))
    summary = visual.get("summary") or {}
    ok = (
        visual.get("parsed_pages") == 3
        and len(visual.get("items", [])) == 3
        and summary.get("figure_pages") == 3
        and summary.get("table_pages") == 3
        and summary.get("chart_pages") == 3
        and "blocks" not in json.dumps(visual)
    )
    record("A-V3-02 bounded evidence", ok, str(summary))
    six_up = pages.get(14, {})
    ok = (
        six_up.get("figures", 0) >= 1
        and six_up.get("vector_graphics", 0) > 100
        and six_up.get("table_likely") is True
        and "slide" not in json.dumps(visual).lower()
    )
    record("A-V3-04 6-up evidence", ok, str({k: six_up.get(k) for k in ["figures", "image_refs", "vector_graphics", "table_likely"]}))

    table, _ = client.call("table_extract", {"source": "sources/verimadenciliği.pdf", "page": 45})
    ok = (
        table is not None
        and table.get("mode") == "native-draft"
        and table.get("table_likely") is True
        and bool(table.get("reason"))
        and bool(table.get("text_preview"))
        and bool(table.get("manual_action"))
        and table.get("source_region", {}).get("page") == 45
    )
    record("A-V3-09 table fallback", ok, str({k: table.get(k) for k in ["mode", "table_likely", "reason"]} if table else None))

    root = setup_smoke_workspace(DATA_MINING)
    if root is None:
        skip("A-V3-03 required gate", "could not extract data-mining pages")
        skip("A-V3-05 chart describe", "could not extract data-mining pages")
        skip("A-V3-06 crop build", "could not extract data-mining pages")
        skip("A-V3-07 crop provenance", "could not extract data-mining pages")
        skip("A-V3-08 crop policy", "could not extract data-mining pages")
        skip("A-V3-12 smoke guide", "could not extract data-mining pages")
    else:
        try:
            guide = root / "guides/veri-smoke"
            analyze = run(
                [
                    str(BIN),
                    "--root",
                    str(root),
                    "analyze",
                    "--source",
                    "sources/veri-smoke.pdf",
                    "--out",
                    "guides/veri-smoke/data/source.json",
                    "--md",
                    "guides/veri-smoke/data/source.md",
                ]
            )
            if analyze.returncode != 0:
                raise RuntimeError(f"analyze failed: {analyze.stderr[-500:]}")

            smoke_client = Client(root)
            brief, _ = smoke_client.call(
                "visual_brief", {"guide": "veri-smoke", "source": "veri-smoke", "page": 2}
            )
            layout_table, _ = smoke_client.call(
                "table_extract",
                {"source": "sources/veri-smoke.pdf", "page": 1, "guide": "veri-smoke"},
            )
            record(
                "A-V3-09 layout mode",
                layout_table is not None
                and layout_table.get("mode") == "layout"
                and layout_table.get("table_likely") is True
                and (bool(layout_table.get("reason")) or bool(layout_table.get("tables")))
                and bool(layout_table.get("text_preview"))
                and layout_table.get("source_region", {}).get("page") == 1,
                str({k: layout_table.get(k) for k in ["mode", "table_likely", "reason"]} if layout_table else None),
            )

            audit_path = guide / "visual-audit.json"
            audit = {
                "schema_version": 1,
                "source_pages": [
                    {
                        "source": "veri-smoke",
                        "page": 1,
                        "decision": "describe",
                        "reason": "Dağılım grafiği; yatay eksen ölçüm değeri, dikey eksen yoğunluk; tek tepeli ve sağa çarpık. Kaynakta okunmayan sayısal değer yok.",
                        "priority": "high",
                    }
                ],
            }

            blocks_path = guide / "data/blocks/veri-smoke.json"
            crop = "0.0,0.0,1.0,1.0"
            try:
                layout = json.loads(blocks_path.read_text())
                page_two = next(page for page in layout["pages"] if page["page"] == 2)
                bbox = next(
                    block["bbox"]
                    for block in page_two["blocks"]
                    if block["kind"] == "figure" and block.get("bbox")
                )
                width = page_two["width"]
                height = page_two["height"]
                left = max(0.0, (bbox["x"] - 8) / width)
                top = max(0.0, (bbox["y"] - 8) / height)
                crop_width = min(1.0 - left, (bbox["width"] + 16) / width)
                crop_height = min(1.0 - top, (bbox["height"] + 16) / height)
                crop = f"{left:.4f},{top:.4f},{crop_width:.4f},{crop_height:.4f}"
            except (KeyError, StopIteration, ValueError, json.JSONDecodeError):
                crop = "0.0,0.0,1.0,1.0"

            cropped = run(
                [
                    str(BIN),
                    "--root",
                    str(root),
                    "crop-source",
                    "--guide",
                    "veri-smoke",
                    "--source",
                    "sources/veri-smoke.pdf",
                    "--page",
                    "2",
                    "--crop",
                    crop,
                ]
            )
            crop_value = json_stdout(cropped)
            record(
                "A-V3-06 crop build",
                cropped.returncode == 0
                and crop_value
                and crop_value.get("asset", "").startswith("data/crops/")
                and len(crop_value.get("hash", "")) == 64
                and crop_value.get("local_only") is True
                and crop_value.get("width", 0) > 0
                and crop_value.get("height", 0) > 0,
                str({k: crop_value.get(k) for k in ["asset", "hash", "width", "height", "local_only"]} if crop_value else cropped.stderr[-200:]),
            )
            if crop_value:
                guide_md = (guide / "guide.md").read_text()
                figure = (
                    '::: figure src="'
                    + crop_value["asset"]
                    + '" caption="ROC eğrisi ve eşik noktaları" source="veri-smoke, s. 2" '
                    'alt="ROC eğrisi; yatay eksen yanlış pozitif oranı, dikey eksen doğru pozitif oranı" width="85%"\n:::\n\n'
                )
                guide_md = guide_md.replace("## Bağlantılar", figure + "## Bağlantılar", 1)
                (guide / "guide.md").write_text(guide_md)

            no_audit = run(
                [str(BIN), "--root", str(root), "build", "--guide", "veri-smoke", "--check"]
            )
            record(
                "A-V3-03 required gate",
                no_audit.returncode != 0
                and "visual-audit.json is missing" in (no_audit.stderr + no_audit.stdout),
                (no_audit.stderr or no_audit.stdout)[-180:],
            )

            audit_path.write_text(json.dumps(audit, ensure_ascii=False, indent=2) + "\n")
            config_path = guide / "config.json"
            config = json.loads(config_path.read_text())
            config["audit"]["visual_gate"] = "warn"
            config_path.write_text(json.dumps(config, ensure_ascii=False, indent=2) + "\n")
            warned = run(
                [str(BIN), "--root", str(root), "build", "--guide", "veri-smoke", "--check"]
            )
            warned_value = json_stdout(warned)
            warned_visual = (warned_value or {}).get("visual") or {}
            brief_ok = (
                brief is not None
                and brief.get("chart_likely") is True
                and brief.get("table_likely") is True
                and brief.get("evidence_kind") == "figure+table"
            )
            record(
                "A-V3-05 chart describe",
                warned.returncode == 0
                and "veri-smoke:2" in warned_visual.get("missing", [])
                and warned_visual.get("visual_gate") == "warn"
                and brief_ok,
                str({k: warned_visual.get(k) for k in ["visual_gate", "missing", "figure_pages"]})
                if warned_visual
                else (warned.stderr + warned.stdout)[-300:],
            )

            if crop_value:
                audit["source_pages"].append(
                    {
                        "source": "veri-smoke",
                        "page": 2,
                        "decision": "embed_crop",
                        "reason": "ROC eğrisi ve eşik noktaları; yatay eksen FPR, dikey eksen TPR.",
                        "asset": crop_value["asset"],
                        "crop": crop,
                        "caption": "ROC eğrisi ve eşik noktaları",
                        "alt": "ROC eğrisi; yatay eksen yanlış pozitif oranı, dikey eksen doğru pozitif oranı",
                        "hash": crop_value["hash"],
                        "local_only": True,
                        "priority": "high",
                    }
                )
                audit_path.write_text(json.dumps(audit, ensure_ascii=False, indent=2) + "\n")
                guide_md = (guide / "guide.md").read_text()
                figure = (
                    '::: figure src="'
                    + crop_value["asset"]
                    + '" caption="ROC eğrisi ve eşik noktaları" source="veri-smoke, s. 2" '
                    'alt="ROC eğrisi; yatay eksen yanlış pozitif oranı, dikey eksen doğru pozitif oranı" width="85%"\n:::\n\n'
                )
                guide_md = guide_md.replace("## Bağlantılar", figure + "## Bağlantılar", 1)
                (guide / "guide.md").write_text(guide_md)

                visual_manifest = run(
                    [
                        str(BIN),
                        "--root",
                        str(root),
                        "visual",
                        "candidates",
                        "--guide",
                        "veri-smoke",
                        "--source",
                        "sources/veri-smoke.pdf",
                        "--write",
                        "--detail",
                        "summary",
                    ]
                )
                config["audit"]["visual_gate"] = "required"
                config_path.write_text(json.dumps(config, ensure_ascii=False, indent=2) + "\n")
                built = run(
                    [str(BIN), "--root", str(root), "build", "--guide", "veri-smoke", "--check"]
                )
                built_value = json_stdout(built)
                output = root / "output/veri-smoke.pdf"
                text = pdf_text(output) if output.is_file() else ""
                pdf_bytes = output.read_bytes() if output.is_file() else b""
                record(
                    "A-V3-07 crop provenance",
                    built.returncode == 0
                    and built_value
                    and built_value.get("passed") is True
                    and output.is_file()
                    and "ROC eğrisi ve eşik noktaları" in text
                    and "veri-smoke, s. 2" in text
                    and b"/Alt" in pdf_bytes
                    and 'alt="ROC eğrisi' in (guide / "guide.md").read_text(),
                    str((built_value or {}).get("pages")) if built.returncode == 0 else (built.stderr + built.stdout)[-300:],
                )
                record(
                    "A-V3-08 crop policy",
                    "guides/*/data/crops/" in (REPO / ".gitignore").read_text()
                    and crop_value.get("local_only") is True
                    and (guide / crop_value["asset"]).is_file()
                    and "embed_crop must be local_only" not in (built.stderr + built.stdout),
                    "local-only crop asset under data/crops/, gitignored",
                )
                record(
                    "A-V3-12 smoke guide",
                    warned_value is not None
                    and warned_visual.get("missing") == ["veri-smoke:2"]
                    and built.returncode == 0
                    and built_value.get("passed") is True
                    and "| Öğe |" in (guide / "guide.md").read_text()
                    and (guide / "data/coverage.md").is_file(),
                    "warn names veri-smoke:2; required build passes with table + describe + crop",
                )
        finally:
            if not os.environ.get("FOLISIFT_V3_KEEP"):
                shutil.rmtree(root, ignore_errors=True)

# A-V3-10: installed MCP from an external workspace.
if EXTERNAL.is_dir():
    external_client = Client(EXTERNAL, BIN)
    doctor, _ = external_client.call("doctor", {})
    model_check = next(
        (check for check in (doctor or {}).get("checks", []) if check.get("name") == "semantic-model"),
        {},
    )
    topics, _ = external_client.call(
        "source_topics", {"guide": "is-hukuku", "semantic": True, "k": 5}
    )
    ok = (
        model_check.get("ok") is True
        and topics is not None
        and topics.get("method") == "model2vec"
    )
    record(
        "A-V3-10 external model",
        ok,
        f"{topics and topics.get('method')}; installed={INSTALLED}; {model_check.get('detail')}",
    )
else:
    skip("A-V3-10 external model", f"{EXTERNAL} not found")

# A-V3-11: digital review remains a labeled legacy fallback.
if EXTERNAL.is_dir() and (EXTERNAL / "guides/dijital-donusum-temelleri").is_dir():
    external_client = Client(EXTERNAL, BIN)
    review, _ = external_client.call(
        "coverage_manifest",
        {"action": "review", "guide": "dijital-donusum-temelleri"},
    )
    qa, _ = external_client.call("qa_guide", {"slug": "dijital-donusum-temelleri"})
    review_rows = (review or {}).get("rows")
    review_missing = (review or {}).get("missing")
    review_ok = (
        review is not None
        and (
            (review_rows == 41 and review_missing == 0)
            or (review_rows == 62 and review_missing == 60)
        )
        and (review.get("review_required") is (review_missing > 0))
    )
    qa_source = (qa or {}).get("coverage_source")
    qa_ok = (
        qa is not None
        and qa_source in ("combined", "review", "legacy")
        and (qa_source != "legacy" or bool(qa.get("coverage_warning")))
        and (qa.get("coverage") or {}).get("total") == 41
        and qa.get("passed") is True
    )
    record(
        "A-V3-11 digital review",
        review_ok and qa_ok,
        f"review {review_rows} rows/{review_missing} missing; qa {qa_source} {qa and (qa.get('coverage') or {}).get('total')}",
    )
else:
    skip("A-V3-11 digital review", "digital guide not found")

ok = all(item["ok"] for item in checks)
print(json.dumps({"acceptance": ok, "checks": checks}, ensure_ascii=False))
sys.exit(0 if ok else 1)
