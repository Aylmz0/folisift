# Authoring a guide

> **Evidence basis.** Every format rule in this document is grounded in
> [`authoring-evidence.md`](authoring-evidence.md),
> [`revision-format-research.md`](revision-format-research.md) and
> [`document-genres-and-layout-systems.md`](document-genres-and-layout-systems.md).
> The rules are evidence-informed, not claims of proven exam-score gains.

This is the practical runbook. `docs/authoring-spec.md` is the normative
content contract; `docs/authoring-evidence.md` records the learning,
note-taking, summarization and accessibility basis; `docs/architecture.md`
explains the layers.

## Prerequisites

- Rust 1.92+
- `cargo`
- `libpdfium.so` next to the `folisift` binary, or `FOLISIFT_PDFIUM_DIR`
- D2 v0.9.0 only for `folisift diagram build`; set `FOLISIFT_D2` or place
  `d2` next to the binary. Guide builds do not need D2.

No Python, Poppler, WeasyPrint or system Typst installation is needed.

## Guide bundle anatomy (schema 3)

```text
guides/<slug>/
├── guide.md           # Markdown + directives; source order preserved
├── config.json        # schema 3 style profile and paths
├── visual-audit.json  # one decision per low-text source page (host-authored)
├── diagrams/          # optional D2 JSON specs (host-authored)
├── assets/            # guide-specific SVG diagrams and generated assets
└── data/
    ├── source.json    # generated inventory (do not hand-edit)
    ├── source.md      # human-readable inventory
    └── coverage.md    # generated coverage report
```

## First pass: inventory and visual review

```bash
cargo run -p folisift-cli -- analyze \
  --source sources/PD0.pdf --source sources/PD1.pdf \
  --out guides/pd0-pd1/data/source.json \
  --md guides/pd0-pd1/data/source.md
```

For an opt-in LiteParse depth pass (comments/highlights, AcroForm fields,
tagged structure, vector geometry, image refs and per-page complexity):

```bash
cargo run -p folisift-cli -- insights --source sources/PD0.pdf
```

The MCP equivalent is `source_insights`. It is read-only diagnostics: it does
not run OCR, write files, or change coverage. The MCP flow reads source text in
bounded chunks with `read_source` and can search it with `search_source`; the
host LLM writes the prose. For smaller reads use `section_brief` (level:
chapter/section/heading) and `read-section --mode full|summary|claims|tables`;
both accept `--max-tokens` or `--max-chars`. `max_tokens` is a hard payload
budget: the response is truncated with `truncated: true`, `next_offset` and the
full character count instead of silently returning the full section.

Cluster source headings with the bundled local Model2Vec model:

```bash
cargo run -p folisift-cli -- topics --guide pd3 --k 8
cargo run -p folisift-cli -- topics --guide pd3 --k 8 --no-model
```

The default is local embeddings; `--no-model` is the deterministic heading-term
fallback. Neither path sends text to a network.

## Choose a style

Styles are engine-owned packs under `styles/`. `paper` is the frozen parity
layout and is not the default for new guides; `reader` is the spacious density
variant; `clean` is the technical layout; `journal` and `brief` are presets;
`accessible` is a large-print modifier with a high-contrast palette. None of
these is a genre on its own.

Palette and font pairing are separate layers. A style pack declares which
palettes and pairings it accepts; the guide selects them explicitly. The
12-palette and 2-pairing libraries are in `styles/palettes/` and
`styles/fonts/`, and every palette is contrast-tested at load time.

If the reader task is known, select an intent instead of guessing a style:

| Request | Intent | Result |
|---|---|---|
| “Sınav için kısa özet” | `exam-summary` | 2–4 page revision sheet |
| “Sınav için tüm konular” | `exam-revision` | 6–10 page revision sheet |
| “Konuyu anlamak istiyorum” | `concept-summary` | 4–8 page explanation with table/figure |
| “Uzun uzun çalışacağım” | `deep` | 16–28 page reader |
| “Formül sayfası” | `reference` | 2–8 page reference |
| “Pratik yapmak istiyorum” | `practice` | 6–12 page workbook |
| “Tek sayfa” | `one-page` | one-page organiser |

The agent must ask one clarifying question when the request is ambiguous:
“Sınav için mi, konuyu anlamak için mi?” Page count is a budget, not a reason
to shrink type or drop scope.

`notes`, `workbook` and `reference` are full genre layouts for guided notes,
practice sets and formula/reference lookup. They use the `lecture-notes`,
`workbook` and `formula-sheet` structures respectively.

`revision` is the exam-revision layout. It implements two structures:

- `revision-sheet`: map, must-know, comparisons, worked examples, traps,
  recall/answer key, synthesis and review plan.
- `knowledge-organiser`: one page per unit with big questions, key terms,
  timeline, one map/diagram and critical distinctions.

Use revision for exam preparation; use `paper`/`reader` for long-form reading.
The evidence and limits are in
[`revision-format-research.md`](revision-format-research.md) and
[`document-genres-and-layout-systems.md`](document-genres-and-layout-systems.md).

```bash
cargo run -p folisift-cli -- styles list
cargo run -p folisift-cli -- styles info reader --language tr
cargo run -p folisift-cli -- styles preview reader --language tr
cargo run -p folisift-cli -- styles preview revision --language tr
cargo run -p folisift-cli -- styles palettes
cargo run -p folisift-cli -- styles fonts
cargo run -p folisift-cli -- styles preview revision --palette high-contrast --fonts technical
```

The MCP `style` tool also has a `recommend` action. It returns only packs
compatible with the requested domain/structure/language. Domain-specific packs
(`journal`, `brief`) rank first where they fit; `reader` is the spacious
fallback. This is a heuristic; the host must choose a candidate and record
`style_rationale`. When the host omits `style`, `prepare_guide` uses
`revision` for revision-sheet/knowledge-organiser structures and `reader` for
the other structures; the response reports `style_source` (`explicit`,
`default-revision` or `default-reader`).

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
    "aesthetic": "reader",
    "palette": "academic-navy",
    "fonts": "editorial",
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

A style changes presentation and labels; a genre changes the layout logic and
content grammar. Neither changes facts or coverage. Guides never contain raw
Typst or CSS.

For a non-A–G structure, claim source sections with a content comment before
the heading:

```markdown
<!-- covers: PD0:0.1, PD0:0.2 -->
## Introduction
```

The coverage gate reads `covers` first and falls back to `## <number> <title>`
headings. Every source subsection must be claimed exactly once.

Revision sheets and knowledge organisers use a page-scope gate instead. Declare
the source range in frontmatter and cite every page in it at least once:

```yaml
source_scope: "PD2:26-29"
# or a numbered section, resolved from the inventory headings
source_scope: "PD2:section:3.4"
```

```markdown
- **Kod = veri:** program kodu ile veri aynı temsili paylaşır. [PD2, s. 27]
```

A `section:` entry expands to that heading's page through the page before the
next non-descendant heading, or to the source end. A reference outside
`source_scope`, an unknown source, an unknown section or an uncited page fails
the build. This is selective coverage by design, not A–G heading coverage.

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

### Visual audit manifest

Every low-text page needs exactly one decision in
`guides/<slug>/visual-audit.json`:

```json
{
  "schema_version": 1,
  "source_pages": [
    {
      "source": "PD2",
      "page": 5,
      "decision": "redraw",
      "asset": "diagrams/dil-soyagaci.json",
      "reason": "language genealogy; high pedagogical value",
      "priority": "high"
    },
    {
      "source": "PD2",
      "page": 45,
      "decision": "decorative",
      "reason": "historical COBOL photo; no mechanism",
      "priority": "low"
    }
  ]
}
```

Decisions: `native_table`, `redraw_table`, `redraw_figure`, `embed_crop`,
`describe`, `text`, `decorative`, `unreadable`. Legacy `table` is accepted as
`redraw_table` for one release; legacy `redraw` remains a figure redraw.

`embed_crop` entries also carry `asset`, `crop`, `caption`, `alt`, a required
`hash` and `local_only=true`; the crop is local/gitignored under
`data/crops/`. `describe` entries need a reason and must not invent chart
values.

- `native_table` means the source already contains a table.
- `redraw_table` means a figure/matrix must be converted into a table.
- `redraw_figure`/`redraw` requires an existing asset under the guide bundle.
- `text`, `decorative` and `unreadable` require a reason.
- Cover pages and decorative photos are `decorative`; they are not content.
- `build --check` fails when a low-text page is missing from the manifest or
  an entry is unresolved. With `config.audit.visual_gate="required"`, pages
  with figure/table/vector evidence are also required; the default `warn`
  reports them without breaking existing guides.
- This is the evidence trail for the G section.

`build --check` also reports `underfilled_pages` through `folisift audit`.
A page below the body-density threshold must be reflowed or removed from the
layout; do not add filler prose to hide it. `audit --strict` and
`build --check` fail on it.

## Write the guide

Use the headings from the inventory, in source order:

```markdown
---
title: "Programlama Dilleri — Çalışma Kılavuzu"
subtitle: "Kavramlar, diller ve çevrim süreçleri"
course: "Course 101"
running: "Programlama Dilleri · Çalışma Kılavuzu"
cover_kicker: "Course 101 · Study guide"
cover_title: "Çalışma Kılavuzu"
deck: "Bölüm 0: Hazırlık · Bölüm 1: Giriş"
pilot: "PILOT · SAMPLE"
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
| `::: recall label=...` | retrieval question; pair it with a later `::: answer` |
| `::: explain label=...` | optional self-explanation prompt (“why/how/what-if”) |
| `::: review-plan label=...` | short spaced-review schedule; keep it consistent with `review_intervals` |
| `::: must-know` | compact card stack of must-know definitions; revision layout |
| `::: synthesis title=...` | one-page synthesis block; revision layout |
| `::: answer label=...` | separated answer-key block; reader and revision layouts |
| `::: organiser title=... source=...` | one-page knowledge-organiser page; revision layout |
| `::: table widths="5,20,35,40" kind="facts" caption="..." source="..." id="..." caption_number="false" keep_with_next="true" allow_page_break="true" min_space_mm="6"` | table metadata and placement; wrap the Markdown table |
| `::: glossary` | four-column glossary table; group with `###` headings |
| `::: columns` … `::: columnbreak` … `:::` | two-column region and explicit break |
| `::: pagebreak` | start the next block on a new page (rare) |

`chapter` attributes: `part`, `side`, `kicker`, `title`, `toc`, `running`,
`lead`, `range`, `compact`, `subs`, `flow`, `keep_with_next`, `break_before`,
`break_after`, `allow_split`, `full_page_opener`. Use `compact=true` for C–G
chapters; use `subs=false` when a chapter's `##` headings should not appear in
the TOC. `flow=true` pulls the opener into the previous page's remaining space.
The `::: toc` directive accepts `subs`, `toc_depth`, `include` and `exclude`
(comma-separated heading text). `include`/`exclude` match both chapter titles
and `##` headings; a filtered chapter still supplies its `subs` state to its
subsections.

`::: columns` is a two-column prose region. `::: columnbreak` starts the next
column; use it only inside `::: columns`. With `flow=true` the container may
continue on the next page at a block boundary, but a table row, glossary pair
or figure is never split by the break. Use `min_rows_before_break` only with
`0`; non-zero values are rejected rather than ignored.
The first `::: definition` in the front matter may carry
`toc="A · ÇERÇEVE"` so the box gets a TOC entry.

`::: table` attributes: `widths` is a comma-separated percentage list;
`kind="facts"` renders a label/value grid instead of a data table; `caption`
and `source` add a numbered caption and source note. `caption_number="false"`
keeps the caption without a table number. `id` creates a stable Typst label
such as `tab-derleme`, usable in prose as `@tab-derleme`. Widths must be
percentages that sum to 100. `keep_with_next`, `allow_page_break` and
`min_space_mm` control placement.

Glossaries use one `::: glossary` block; each `###` group becomes a balanced
four-column table (`Terim | Tanım | Terim | Tanım`) with repeated headers and
atomic term/definition rows.

### Diagrams

The only diagram engine is **D2 v0.9.0**. It runs in the CLI authoring step;
the MCP server and guide build only read the generated SVG. D2 is discovered
from `FOLISIFT_D2`, `PATH`, the binary's sibling `d2`, or
`~/.local/lib/folisift/d2`; `doctor` prints the resolved path.

A diagram is a typed JSON spec under `guides/<slug>/diagrams/<id>.json`:

```json
{
  "schema_version": 3,
  "id": "pipeline",
  "kind": "flow",
  "title": "Derleme hattı",
  "caption": "Kaynak programdan amaç koda beş aşamalı dönüşüm.",
  "alt": "Kaynak program sözcüksel, sözdizimsel ve anlamsal analizden geçerek amaç koda dönüşür.",
  "source": "PD0, s. 35",
  "direction": "right",
  "layout": "tala",
  "aspect_ratio": 1.8,
  "allow_page_span": false,
  "width": "90%",
  "max_height": "14cm",
  "fit": "contain",
  "sketch": false,
  "nodes": [
    {"id": "src", "label": "Kaynak program"},
    {"id": "lex", "label_lines": ["Sözcüksel", "analiz"]},
    {"id": "code", "label": "Kod üretimi"}
  ],
  "edges": [
    {"from": "src", "to": "lex", "label": "karakter akışı"},
    {"from": "lex", "to": "code"}
  ]
}
```

Reference it from `guide.md` with an empty directive:

```markdown
::: diagram id="pipeline"
:::
```

`aspect_ratio` is a direction hint only when `direction` is absent.
`label_lines` is preferred over escaped `\n`. `class_styles` plus
`node_meta.<id>.class` colour outcome branches; `grid`, `vars`, local `imports`
under `data/d2`, and node/group `width`/`height` are supported. `allow_page_span`
is required when the geometric page share exceeds one; `diagram fit` reports the
share before a build.

`sketch=true` opts into D2 hand-drawn rendering. Keep it off for legal, medical
and other formal guides unless the guide owner explicitly wants the experiment;
the manifest records the flag so stale assets are rebuilt.

Build the assets with the CLI; D2 is not called from MCP or `build`:

```bash
cargo run -p folisift-cli -- diagram build --guide pd0-pd1 --id pipeline
cargo run -p folisift-cli -- diagram status --guide pd0-pd1
cargo run -p folisift-cli -- diagram fit --guide pd0-pd1 --id pipeline --width 75
```

Generated and audited files:

```text
diagrams/<id>.json                 # source of truth
data/d2/<id>.d2                    # generated D2 source
data/diagrams/<id>.json            # manifest: hashes, engine, version
assets/generated/<svg-hash>.svg    # committed, embed-only
```

Supported kinds: `flow`, `concept`, `tree`, `decision`, `sequence`, `class`,
`erd`. Limits: 24 nodes / 40 edges, tree and decision depth 8; no decorative
images. Inline ` ```diagram ` blocks are
rejected; move the JSON to `diagrams/<id>.json` and use the directive. The
only exception is a manually supplied SVG/PNG asset under `assets/`; it still
passes the sanitizer and needs a takeaway caption with provenance.

The figure helper measures the rendered image. Figures use 90% of the content
width by default; the height cap scales with the requested width, so
`width="60%"` and `width="90%"` stay visibly different. A `diagram` directive
accepts `width="50%"`–`width="100%"`, a `max_height` length (`10cm`, `90mm`),
and `fit="contain|width|height"`:
`::: diagram id="flow" width="75%" max_height="9cm" fit="contain"`. A spec can
set the same defaults with `width`, `max_height`, `height` and `fit`; directive
attributes win. `allow_page_span=true` is required when the geometric fit is
taller than one page; `folisift diagram fit` reports the page share before a
build.

### Layout blocks

`folisift analyze`, `scaffold` and MCP `prepare_guide` also write
`guides/<slug>/data/blocks/<source>.json`. The file contains page/block
geometry, heading levels, reading order and table cell boxes from LiteParse.
It is generated data and ignored by default; `inventory.json` keeps only a
compact `layout` summary.

Read a bounded view with:

```bash
folisift read-blocks --guide my-course --source PD0 --pages 26-29
folisift table extract --guide my-course --source sources/PD0.pdf --page 12
```

The MCP equivalent is `read_blocks` (guide, source, pages, max_blocks). Use it
instead of loading a whole block artifact into the context window.

## Build and gate

```bash
cargo run -p folisift-cli -- coverage-manifest --guide pd0-pd1 --action build
cargo run -p folisift-cli -- build --guide pd0-pd1 --check
cargo run -p folisift-cli -- qa --guide pd0-pd1
```

`coverage-manifest` is authoritative in this order:

1. `data/coverage-manifest.json` — adopted combined file; this is what
   `adopt:true` writes from a reviewed manifest;
2. `data/coverage-manifest.review.json` — host-reviewed candidate, used only
   when every row is `covered`, `merged` or `excluded`;
3. `data/coverage-manifest.generated.json` — machine-built skeleton; advisory
   by default. Select it explicitly with `manifest:"generated"` or opt in with
   `"audit": {"manifest_gate": "generated"}`;
4. the legacy heading gate.

`build` writes a generated skeleton and never overwrites a review file. A
review file with `missing` rows is reported but not used as the gate. Generated
rows are body-free candidates: headings first, TOC fallback, with `origin`
(`heading`, `toc`, `inline`, `review`, `manual`). A review may add a manual row
that the source never exposes as a heading; that row stays review-only.

`empty_sections` means `body_chars == 0`; a section with 1–399 characters is
`weak_sections`, not empty. The default weak threshold is 400 characters and is
reported as `weak_section_chars`.

A failed `build --check` writes `output/<slug>_draft.pdf` and returns
`draft_path`; the normal output PDF is not overwritten. Review the draft, then
delete it. `qa` follows the same rule.

The build fails if a numbered source subsection has no matching heading, an
empty section, a missing visual decision, or an underfilled body page. The
coverage report is written to `data/coverage.md`. `qa` runs build, coverage,
audit, visual, cards, claims and diagram counts in one report. A passing gate is
necessary, not sufficient: the G section still needs a human audit of tables,
figures, examples and low-text pages.

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

## Claim spot-check and recall cards

Two deterministic review artifacts sit beside the PDF. Neither changes the
guide or claims learning efficacy.

```bash
folisift claims --guide my-course
folisift cards --guide my-course --review-intervals 1,3,7,14,30 --include-definitions
```

`claims` writes `guides/<slug>/data/claims.md`. It lists high-signal sentences
(numbers, dates, comparisons, causal wording) and the source page they cite,
or marks the reference as missing. A citation is **traceability, not
verification**: open the cited page and check the subject, number/unit, date,
causal wording and qualifiers. Claims inside `::: recall` questions are ignored;
claims inside answers and prose are included.

`cards` pairs `::: recall` and `::: answer` blocks in document order and writes
an Anki-compatible CSV with `front,back,tags,source`. `--include-definitions`
also exports `::: definition`/`::: must-know` blocks. `qa` uses the same policy
and reports `recall`, `definitions` and `total` separately. Set the default
intervals in `config.json`:

```json
"review_intervals": [1, 3, 7]
```

The CLI flag overrides the config. Intervals are 1–365 days, unique, sorted and
limited to ten entries. `last_reviewed` and confidence belong to the review
application, not the PDF; the guide only carries the schedule. Keep the
`::: review-plan` prose consistent with the intervals.

Both artifacts are available over MCP as `claims` and `cards`; output paths
must stay under `guides/`, `output/` or `data/`.

## Semantic section links

Once `data/blocks` exists, the local link helper can propose cross-source
connections:

```bash
folisift links build --guide my-course --no-model   # BM25 baseline
folisift links build --guide my-course              # model2vec if installed
folisift links build --guide my-course --explore    # lower thresholds, wider net
folisift links read --guide my-course --section source:p26:s1
folisift links status --guide my-course
folisift graph build --guide my-course
folisift graph status --guide my-course
```

`graph` writes `data/document-graph.json` with source, page, block, table-cell,
figure and citation nodes plus structural edges. It is deterministic, ignores
unknown citation pages and never gates a build. The default preset is precision-first (`0.75 / 0.55`, top-k 5); `--explore`
uses `0.65 / 0.45`, top-k 8. The builder skips repeated slide titles, adjacent
slides, page-pair duplicates and shared bibliography/boilerplate blocks.
The output is `data/document-links.json` and is review-only. It records source
pages and block ids; `semantically_similar_to` is a similarity proposal, not a
verified prerequisite or causal claim. The MCP `links` tool exposes the same
actions. A release install can use the bundled int8 model; a development
checkout can point `FOLISIFT_MODEL_DIR` at a local model directory.

## New guide

```bash
cargo run -p folisift-cli -- scaffold \
  --slug sample --title "Sample chapter" --course "Course 101" \
  --style reader --structure a-g --language tr --domain cs \
  --palette warm-paper --fonts editorial \
  --source sources/PD2.pdf
```

The MCP `scaffold_guide` tool performs the same operation and generates
headings from the source inventory.

Which tool when:

| Task | CLI | MCP |
|---|---|---|
| Analyze a source and write inventory/blocks | `analyze` | `analyze_source` |
| Create a new guide bundle | `scaffold` | `scaffold_guide` |
| Analyze + scaffold + outline + visual targets in one call | — | `prepare_guide` |
| Build and gate a guide | `build --check` | `build_guide` |
| One release report | `qa` | `qa_guide` |
| Strict PDF audit | `audit --strict` | `audit_guide` |
| Change style without touching content | `style apply` | `style` (`action=restyle`) |
| Inspect or preview styles | `styles` | `style` |
| Validate/build/status D2 | `diagram` | `diagram` (`build` stays CLI-only) |
| Read one source section | `read-section` | `read_section` / `section_brief` |

### IRAC guides

IRAC case bodies use level-3 headings:

```markdown
## Olay 1 — ...

### Sorun

### Kural

### Uygulama

### Sonuç
```

`## Olay/Case/Dava` headings are case rows; source headings belong in
`source-headings.md`, not in `guide.md`. `prepare_guide` writes ten case blocks
for a new IRAC guide, uses the real source id in placeholder citations, writes
`source-headings.md`, and refuses to overwrite an existing IRAC guide without
case headings.

### Source crops

For a chart or table that cannot be faithfully redrawn, crop the source page and
embed the crop with provenance:

```bash
folisift crop-source --guide my-guide --source sources/PD0.pdf \
  --page 45 --crop 0.05,0.10,0.55,0.45
```

The MCP equivalent is `crop_source`. The crop is stored under
`guides/<slug>/data/crops/` (local/gitignored). Reference it with:

```markdown
::: figure src="data/crops/<hash>.png" caption="ROC eğrisi" source="PD0, s. 45" alt="ROC eğrisi; eksenlerde TPR ve FPR"
:::
```

The directive requires `src`, `caption`, `source` and `alt`. The renderer
rejects remote paths, parent-directory escapes and missing assets, and includes
the crop bytes in the build cache key. A crop is not a redrawn or verified
figure. Do not use a crop to claim a data value that is not visible in the
source. If the chart data is available as a
table, prefer a table or a `describe` note; D2 is only for conceptual diagrams,
not XY/ROC plots.

## Figure evidence gate

`visual_evidence` (CLI `visual-evidence`) reports figure, table, vector and
image evidence per page. Set:

```json
"audit": {
  "visual_gate": "required"
}
```

for a chart-sensitive guide. `required` fails the build when an evidence page
has no decision; `warn` (default) reports missing decisions; `off` keeps the
legacy low-text-only gate.

## Design changes

Change `templates/paper.typ` or `templates/revision.typ` for shared geometry;
change a palette or font pairing only in its library file. A guide-specific
visual change belongs in the guide only when it carries meaning; otherwise it
is a design-system change and needs an ADR (ADR-0002, ADR-0007, ADR-0012).

## Checklist

- [ ] All source PDFs are under `sources/` and not committed.
- [ ] Inventory regenerated after a source change.
- [ ] Low-text pages inspected visually.
- [ ] Every numbered subsection has a `##` heading.
- [ ] Source references present; no invented facts.
- [ ] `coverage-manifest --action validate` passes when a manifest exists.
- [ ] `build --check` and `qa` pass.
- [ ] G section records coverage, exclusions and weak spots.
- [ ] PDF page count and Turkish glyphs checked.
