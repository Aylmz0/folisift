# Authoring a guide

This is the practical runbook. `docs/authoring-spec.md` is the normative
content contract; `docs/architecture.md` explains the layers.

## Prerequisites

For the Python reference:

- Python 3.10+
- Poppler: `pdfinfo`, `pdftotext`, `pdftoppm`
- Pillow and WeasyPrint

For the Rust v2 path:

- Rust 1.92+
- `cargo`
- `libpdfium.so` next to the `folisift` binary, or `FOLISIFT_PDFIUM_DIR`

## Guide bundle anatomy (schema 2)

```text
guides/<slug>/
├── guide.md         # Markdown + directives; source order preserved
├── config.json      # paths relative to this directory
├── assets/          # guide-specific SVG diagrams
└── data/
    ├── source.json  # generated inventory (do not hand-edit)
    ├── source.md    # human-readable inventory
    └── coverage.md  # generated coverage report
```

Schema 1 bundles use `guide.html` and the Python engine. Keep them readable
during the migration; do not create new schema-1 bundles.

## First pass: inventory and visual review

```bash
# Rust
cargo run -p folisift-cli -- analyze \
  --source sources/PD0.pdf --source sources/PD1.pdf \
  --out guides/pd0-pd1/data/source.json \
  --md guides/pd0-pd1/data/source.md

# Python reference
cd guides/pd0-pd1
python3 ../../engine/tools/analyze_sources.py \
  ../../sources/PD0.pdf ../../sources/PD1.pdf \
  --out data/source.json --md data/source.md
```

Low-text pages need a visual pass:

```bash
cargo run -p folisift-cli -- render-pages \
  --source sources/PD0.pdf --pages 7,9,17 \
  --out guides/pd0-pd1/data/pages
```

Read the images. Do not invent content for a page you cannot see. Record
inaccessible or ambiguous content in the guide’s G section.

## Write the guide

Use the headings from the inventory, in source order:

```markdown
---
title: "Programlama Dilleri — Çalışma Kılavuzu"
language: tr
sources: [PD0, PD1]
---

# Bölüm 0 — Hazırlık

## 0.1 Giriş ve amaçlar

Metin... [PD0, s. 3]

::: definition
**Programlama dili:** ...
:::

::: critical
**Sözdizimi / anlambilim:** ...
:::
```

Rules:

- Use `## <section number> <title>` for every inventory subsection.
- Cite the source after claims: `[PD0, s. 3]`.
- Label added explanation with **Açıklayıcı not** inside `::: note`.
- Keep Turkish terminology; give the English term in parentheses on first use.
- Tables use Markdown tables; figures use `![caption](assets/x.svg)`.
- Do not write raw HTML, CSS or Typst.

## Build and gate

```bash
cargo run -p folisift-cli -- build --guide pd0-pd1 \
  --output output/PD0-PD1_v2.pdf --check
```

The command fails if a numbered source subsection has no matching heading. The
coverage report is written to `data/coverage.md`. A passing gate is necessary,
not sufficient: the G section still needs a human audit of tables, figures,
examples and low-text pages.

Python reference:

```bash
python3 engine/build.py --guide guides/pd0-pd1 --check
```

## New guide

```bash
cargo run -p folisift-cli -- scaffold \
  --slug pd2 --title "Bölüm 2" --course "BMÜ325" \
  --source sources/PD2.pdf
```

The MCP `scaffold_guide` tool performs the same operation and generates
headings from the source inventory.

## Design changes

Change `templates/paper.typ` for the shared design. A guide-specific visual
change belongs in the guide only when it carries meaning; otherwise it is a
design-system change and needs an ADR (ADR-0002, ADR-0007).

## Checklist

- [ ] All source PDFs are under `sources/` and not committed.
- [ ] Inventory regenerated after a source change.
- [ ] Low-text pages inspected visually.
- [ ] Every numbered subsection has a `##` heading.
- [ ] Source references present; no invented facts.
- [ ] `build --check` passes.
- [ ] G section records coverage, exclusions and weak spots.
- [ ] PDF page count and Turkish glyphs checked.
