# Authoring a guide

This is the practical runbook. `docs/authoring-spec.md` is the normative
content contract; `docs/architecture.md` explains the layers.

## Prerequisites

- Rust 1.92+
- `cargo`
- `libpdfium.so` next to the `folisift` binary, or `FOLISIFT_PDFIUM_DIR`

No Python, Poppler, WeasyPrint or system Typst installation is needed.

## Guide bundle anatomy (schema 3)

```text
guides/<slug>/
├── guide.md         # Markdown + directives; source order preserved
├── config.json      # schema 3 style profile and paths
├── assets/          # guide-specific SVG diagrams
└── data/
    ├── source.json  # generated inventory (do not hand-edit)
    ├── source.md    # human-readable inventory
    └── coverage.md  # generated coverage report
```

## First pass: inventory and visual review

```bash
cargo run -p folisift-cli -- analyze \
  --source sources/PD0.pdf --source sources/PD1.pdf \
  --out guides/pd0-pd1/data/source.json \
  --md guides/pd0-pd1/data/source.md
```

The MCP flow reads source text in bounded chunks with `read_source` and can
search it with `search_source`; the host LLM writes the prose.

## Choose a style

Styles are engine-owned packs under `styles/`. `paper` is the stable parity
style; `clean` is the experimental technical style.

```bash
cargo run -p folisift-cli -- styles list
cargo run -p folisift-cli -- styles info clean --language en
cargo run -p folisift-cli -- styles preview clean --language en
```

A schema-3 guide config carries the independent axes:

```json
{
  "schema_version": 3,
  "slug": "pd0-pd1",
  "title": "Programlama Dilleri — Çalışma Kılavuzu",
  "language": "tr",
  "content": "guide.md",
  "output": "../../output/PD0-PD1_Calisma_Ozeti.pdf",
  "sources": ["../../sources/PD0.pdf", "../../sources/PD1.pdf"],
  "style": {
    "aesthetic": "paper",
    "structure": "a-g",
    "language": "tr",
    "source_language": "tr",
    "voice": "academic",
    "study_mode": "exam",
    "domain": "cs",
    "citation": "page-ref"
  }
}
```

A style changes presentation and labels, never facts or coverage. Guides
never contain raw Typst or CSS.

For a non-A–G structure, claim source sections with a content comment before
the heading:

```markdown
<!-- covers: PD0:0.1, PD0:0.2 -->
## Introduction
```

The coverage gate reads `covers` first and falls back to `## <number> <title>`
headings. Every source subsection must be claimed exactly once.

Low-text pages need a visual pass. Cropping a source figure (normalized
fractions `left,top,width,height`) keeps the extract small and focused:

```bash
cargo run -p folisift-cli -- render-pages \
  --source sources/PD0.pdf --pages 7 \
  --out guides/pd0-pd1/assets/source --crop 0.12,0.08,0.76,0.40
```

Then reference the PNG from `guide.md`; the caption must state the takeaway and
provenance (for example `[PD0, s. 7]`). A cropped screenshot is `extracted`,
not `original`.

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
subtitle: "Kavramlar, diller ve çevrim süreçleri"
course: "BMÜ325 Programlama Dilleri"
running: "Programlama Dilleri · Çalışma Kılavuzu"
cover_kicker: "BMÜ325 · Programlama Dilleri"
cover_title: "Çalışma Kılavuzu"
deck: "Bölüm 0: Hazırlık · Bölüm 1: Giriş"
pilot: "PİLOT SÜRÜM · PD0–PD1"
cover_motif: "assets/cover-motif.svg"
cover_notes:
  - "Ders slaytlarından derlenmiştir: PD0 (123 slayt) · PD1 (137 slayt)"
  - "Metin içermeyen eğitsel slaytlar görsel olarak incelenmiştir."
  - "Ek açıklamalar “Açıklayıcı not” etiketiyle işaretlenmiştir."
language: tr
sources: [PD0, PD1]
---

::: front kicker="Ön kısım · Okuma notu" title="Bu kitapçık hakkında" lead="..."
|  |  |
|---|---|
| Kaynaklar | PD0 ve PD1. |
:::

::: toc title="Kılavuzun haritası" note="Sayfa numaraları bu PDF’e aittir."
:::

::: chapter part="B · AYRINTILI İNCELEME" side="KAYNAK: PD0 · 123 SLAYT" kicker="BÖLÜM 0" title="Hazırlık" toc="Bölüm 0 — Hazırlık" running="Bölüm 0 · Hazırlık" lead="..." range="Kaynak aralığı: PD0, s. 1–123"
:::

## 0.1 Giriş ve amaçlar

Metin... [PD0, s. 3]

::: definition label="Tanım"
**Programlama dili:** ...
:::

::: critical label="Kritik ayrım"
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

### Layout directives

The template owns pagination and typography; content uses semantic blocks.

| Directive | Purpose |
|---|---|
| `::: front kicker=... title=... lead=...` | front-matter page before the TOC |
| `::: toc title=... note=...` | custom part-aware table of contents |
| `::: chapter ...` | chapter opener with part/source/kicker/title/lead |
| `::: definition label=...` | cream definition box |
| `::: critical label=...` | sage critical-distinction box |
| `::: example label=...` | rule-top/bottom example box |
| `::: note label=...` | gold note box (use for **Açıklayıcı not**) |
| `::: table widths="5,20,35,40" kind="facts" caption="..." source="..." id="..."` | table metadata; wrap the Markdown table |
| `::: columns` … `::: columnbreak` … `:::` | two-column region and explicit break |
| `::: pagebreak` | start the next block on a new page (rare) |

`chapter` attributes: `part`, `side`, `kicker`, `title`, `toc`, `running`,
`lead`, `range`, `compact`, `subs`. Use `compact=true` for C–G chapters; use
`subs=false` when a chapter's `##` headings should not appear in the TOC.
The first `::: definition` in the front matter may carry
`toc="A · ÇERÇEVE"` so the box gets a TOC entry.

`::: table` attributes: `widths` is a comma-separated percentage list;
`kind="facts"` renders a label/value grid instead of a data table; `caption`
and `source` add a numbered caption and source note. `id` creates a stable
Typst label such as `tab-derleme`, usable in prose as `@tab-derleme`. Widths
must be percentages that sum to 100.

### Diagram blocks

The preferred visual format is a typed JSON spec, not raw SVG:

````markdown
```diagram
{
  "kind": "flow",
  "title": "Derleme hattı",
  "alt": "Kaynak programdan amaç koda altı aşamalı hat.",
  "source": "PD0, s. 35",
  "nodes": [
    {"id": "src", "label": "Kaynak program"},
    {"id": "lex", "label": "Sözcüksel analiz"},
    {"id": "code", "label": "Kod üretimi"}
  ],
  "edges": [
    {"from": "src", "to": "lex", "label": "karakter akışı"},
    {"from": "lex", "to": "code"}
  ]
}
```
````

Supported kinds: `flow`, `concept`, `tree`, `sequence`. The engine writes a
hash-named SVG under `assets/generated/` and embeds it as a figure. Limits:
12 nodes / 20 edges; no decorative images. Raw SVG remains allowed only as an
asset and is not generated by the engine yet.

## Build and gate

```bash
cargo run -p folisift-cli -- build --guide pd0-pd1 --check
```

The command fails if a numbered source subsection has no matching heading. The
coverage report is written to `data/coverage.md`. A passing gate is necessary,
not sufficient: the G section still needs a human audit of tables, figures,
examples and low-text pages.

### Visual regression

```bash
make visual          # compare selected pages against the baseline
make visual-capture  # accept the current PDF as the new baseline
```

`visual compare` renders both PDFs with PDFium, writes `diff-NNN.png` images
under `data/visual/`, and fails when either the mean pixel difference or the
changed-pixel ratio exceeds the limit. The baseline is a PDF, not a set of
screenshots, so page size and rendering DPI stay under engine control.

Review the diff images before running `visual-capture`; the tool proves that
pixels changed, not that the new pixels are better.

## New guide

```bash
cargo run -p folisift-cli -- scaffold \
  --slug pd2 --title "Bölüm 2" --course "BMÜ325" \
  --style paper --structure a-g --language tr --domain cs \
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
