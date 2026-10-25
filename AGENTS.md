# AGENTS.md

Folisift is a domain-agnostic study-guide production pipeline. It turns lecture
PDFs into paper-like, exam-oriented guides. The engine is shared; each course
lives in its own guide bundle under `guides/`.

## Commands

Run from the repository root unless stated otherwise.

```bash
# Build the current guide
python3 engine/build.py --guide guides/pd0-pd1

# Build + coverage gate
python3 engine/build.py --guide guides/pd0-pd1 --check

# Source inventory
cd guides/pd0-pd1
python3 ../../engine/tools/analyze_sources.py \
  ../../sources/PD0.pdf ../../sources/PD1.pdf \
  --out data/source.json --md data/source.md

# Low-text page contact sheets
python3 ../../engine/tools/prepare_pages.py ../../sources/PD0.pdf \
  --source-json data/source.json --filename PD0.pdf \
  --out data/pages/pd0 --cols 2 --rows 2

# New guide bundle
python3 engine/tools/scaffold.py --slug pd2 --title "..." \
  --sources sources/PD2.pdf

# Rust v2 workspace (in progress; Python still builds the current guide)
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo run -p folisift-cli -- workspace-info
cargo run -p folisift-cli -- guide-status pd0-pd1
cargo run -p folisift-cli -- analyze --source sources/PD0.pdf \
  --out guides/pd0-pd1/data/source.json --md guides/pd0-pd1/data/source.md
cargo run -p folisift-cli -- render-pages --source sources/PD0.pdf \
  --pages 7 --out guides/pd0-pd1/data/pages
cargo run -p folisift-cli -- build --guide pd0-pd1 \
  --output output/PD0-PD1_v2.pdf --check
```

The Rust binary loads `libpdfium.so` (or the platform equivalent) from its own
directory. Set `FOLISIFT_PDFIUM_DIR` for a development copy. Do not add runtime
extraction. The v2 plan and Phase 0 evidence live in `docs/plans/`. Do not
remove the Python engine until the Rust path reaches coverage/PDF parity.

## Architecture

- `engine/` — reusable code, templates, CSS and SVG brand assets.
- `crates/` — Rust v2 workspace: `folisift-core` (config/schema/path safety),
  `folisift-parse` (liteparse/inventory/screenshots), `folisift-render`
  (Markdown/Typst), `folisift-cli`.
- `guides/<slug>/` — one self-contained course/material bundle:
  `guide.md` (v2) or `guide.html` (legacy), `config.json`, `assets/`, `data/`.
- `sources/` — personal/course PDFs; **not tracked by git**.
- `output/` — generated PDFs, tracked for review.
- `docs/` — architecture, authoring, design system, ADRs and agent config.

Guide config paths are relative to the guide directory. Do not hard-code a
course name, a page count or a source path in the engine.

## Invariants

- `sources/*.pdf` must never be committed. They are local inputs.
- Never claim full coverage without `engine/tools/check_coverage.py` passing.
- Keep the paper design system in `engine/style.css`; guide-specific styling
  belongs in the guide bundle only when it carries meaning.
- Do not invent facts, formulas, dates or page references.
- Label added clarifications with **Açıklayıcı not** in the guide.
- One guide bundle must be usable without changing another bundle.

## Agent skills

### Issue tracker

Issues live in GitHub Issues (`Aylmz0/folisift`). See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical labels are used unchanged. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context repository: read `CONTEXT.md` and `docs/adr/` before changing
architecture. See `docs/agents/domain.md`.
