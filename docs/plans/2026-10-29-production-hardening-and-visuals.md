# Folisift v0.6→v1.0 — Production Hardening and Visuals Plan

- **Status:** Draft for implementation
- **Date:** 2026-10-29
- **Current release:** `v0.5.0` (12 MCP tools, paper/clean, stdio/HTTP)
- **Related:** `docs/plans/2026-10-28-roadmap.md`,
  `docs/plans/2026-10-28-visual-parity-and-production.md`,
  `docs/plans/2026-10-28-style-language-profiles.md`,
  `docs/architecture.md`, `docs/authoring.md`, ADR-0005, ADR-0007
- **Goal:** close every known production gap, and make tables, diagrams and
  SVG figures a first-class, reproducible part of the guide pipeline.

## 1. Why this plan exists

The engine is no longer a prototype: `paper` reproduces the 22-page reference,
`clean` renders the same content with the same content hash, coverage and layout
audits pass, and the MCP surface is stable. But the product is not production
ready yet:

- style packs are mostly metadata; `tokens.json` does not drive the PDF;
- structure and domain profiles are not enforced end to end;
- `covers` does not prove “exactly once”;
- layout audit is page-level, not content-box/split/TOC-level;
- visual regression, PDF/UA and real CI are incomplete;
- tables cannot be extracted or structurally validated;
- diagrams/SVG are consumed, never generated or validated;
- builds and previews always re-render;
- macOS/Windows packaging and signing are absent.

Students value visuals. Dual coding and multimedia-learning research shows that
meaningful diagrams plus verbal explanation improve comprehension and recall;
decorative images add cognitive load. The product should make **meaningful
visuals easy for the host LLM to request and deterministic for the engine to
render**, not ask an LLM to invent fragile Mermaid or raw SVG and hope it works.

## 2. Current gap inventory

Priority: **P0** blocks production; **P1** blocks a stable v1.0; **P2** polish.

| # | Gap | Priority | Evidence / current state |
|---|---|---|---|
| G1 | Style tokens are not applied | P1 | `styles/*/tokens.json` is read/validated but `paper.typ`/`clean.typ` use hard-coded colours |
| G2 | Structures are metadata only | P1 | `structure` validates, but scaffold still seeds A–G chapters |
| G3 | Domain profiles are not injected | P1 | `domains/*.json` exist; prompts do not consume them |
| G4 | `covers` is not exactly-once | P0 | duplicate claims overwrite silently; audit section not required |
| G5 | Layout audit misses content-box, orphan, split, TOC | P0 | only page bounds/empty/glyph/tagged/bookmark/min-font |
| G6 | Visual regression is manual | P1 | page map + audit; no SSIM/pixel baselines |
| G7 | Build cache absent | P0 | every `build_guide` recompiles Typst |
| G8 | Table extraction absent | P0 | host transcribes tables manually |
| G9 | Table structure validation absent | P1 | widths/header exist; no split/column/caption checks |
| G10 | SVG/diagram generation absent | P0 | only consumes committed SVG assets |
| G11 | Source figure extraction absent | P1 | `render_pages` returns PNG screenshots only |
| G12 | LLM SVG quality unguarded | P0 | host can write any SVG; no sanitizer |
| G13 | Accessibility is partial | P1 | tagged/bookmarks/min font/contrast; no PDF/UA, alt text, reading order |
| G14 | Localization lint is partial | P2 | English labels are in packs; guide content not linted |
| G15 | Raw Typst/CSS in guide not rejected | P1 | treated as plain text |
| G16 | Audit section can be removed | P0 | no build gate |
| G17 | Missing style families | P1 | journal/brief/clinical/casebook/accessible advertised but no packs |
| G18 | Platform release gaps | P1 | Linux archive only; unsigned; CI blocked by billing |
| G19 | HTTP MCP has no auth | P1 | loopback only by documentation, not enforced |
| G20 | Known small correctness issues | P2 | chars/bytes, low-text list cap, dev sidecar, counts, cache test |
| G21 | Visual selection quality is not enforced | P0 | no rubric for “useful vs decorative”, no visual budget |
| G22 | Figure/table numbering and cross-refs are manual | P1 | captions parse numbers from text; no `Tablo 1` counters |
| G23 | Professional caption/alt-text standards are not gated | P1 | no title/description/source/alt contract |

### 2.1 Evidence: what actually helps learners

The visual system must be evidence-led, not “more pictures = better”. The
research is consistent on a few points:

| Finding | Evidence | Consequence for Folisift |
|---|---|---|
| Concept/knowledge maps improve retention | Meta-analysis: overall `g = 0.58`; constructing maps `g = 0.72`; science achievement `g = 0.776` | Add a first-class `concept` diagram kind with labeled edges |
| Graphic organizers help memory and comprehension | 2025 meta-analysis: instructor-provided `g = 0.70` memory, `g = 0.53` comprehension; learner-generated `g = 0.59`/`0.63` | Prioritize matrices, flowcharts and hierarchies over prose lists |
| Text + relevant diagrams beats text alone | Multimedia principle `g ≈ 0.39` | Pair every major mechanism with one explanatory visual |
| Signaling/cueing helps | `g ≈ 0.38` | Use direct labels, arrows and callouts; avoid legends when possible |
| Segmenting helps | `g ≈ 0.32–0.36` | One visual = one idea; split dense diagrams |
| Seductive details hurt | negative effects around `−0.37` to `−0.41`; removing them has a large positive effect | No decorative photos, no “fun” icons, no irrelevant backdrops |
| Drawing/sketching improves recall | drawing group retained 50–80% more in one biology study; drawing effect literature | Support annotated/redrawn diagrams and student-style sketches |
| Sketchnoting combines text + visuals | qualitative and experimental support; dual coding | Allow labeled callouts, arrows and simple icons, not art projects |

User signals from study communities match the research:

- Students ask for **flowcharts for processes** (photosynthesis, cellular
  respiration) and say diagrams with minimal words help.
- Students say the **same headings, tables and flowcharts as the textbook**
  improve exam answers.
- Mind-map users value **connections and big-picture structure**; they warn
  against spending more time making it pretty than learning.
- Note-taking threads ask for **tables, diagrams and hierarchy** and for tools
  that reduce the time to produce them.
- Reddit’s recurring complaints are: too much decoration, too much text,
  diagrams that cannot be read at a glance, and notes that take longer to make
  than to study.

What users actually say (paraphrased, with links):

| User signal | Product implication |
|---|---|
| “Drawing diagrams really helped me, especially flowcharts for photosynthesis and cellular respiration.” | Process diagrams are a top request for biology/science. |
| “The same headings, diagrams, tables and flowcharts from the textbook give much higher marks.” | Exam answers reward matching structure, not just prose. |
| “Tables are good for comparisons; replace big blocks of text with tables.” | Comparison tables are a core study artifact. |
| “Mind maps help with connections and the big picture; include page numbers.” | Concept maps need labeled relationships and source refs. |
| “Creating each mind map takes too much time.” | The engine should make the first draft cheap; the host edits. |
| “Some flowcharts are impossible to understand at a glance.” | Enforce node/edge limits and direct labels. |

Links:
- <https://www.reddit.com/r/biology/comments/qwvuy7/study_tips_for_general_biology_college_level>
- <https://www.reddit.com/r/GetStudying/comments/1o6hnvj/how_do_i_use_spaced_repetition_for_descriptive>
- <https://www.reddit.com/r/IWantToLearn/comments/5fxjkw/iwtl_how_to_make_a_study_guide>
- <https://www.reddit.com/r/GetStudying/comments/1ldlokz/are_mind_maps_actually_good_for_studying>
- <https://www.reddit.com/r/studytips/comments/1maqd9l/mindmap_generator_mindgrasp_chat_gpt>
- <https://www.reddit.com/r/StudentNurse/comments/grg6c3/tip_concept_mapping_as_a_visual_thinking_strategy>

### 2.2 Professional and academic visual standards

Professional readers judge a document by its figures and captions. The plan
adopts these conventions:

- **Figure captions below figures; table captions above tables.**
- Caption anatomy:
  1. number and short title (what the reader should learn);
  2. explanation of elements, units, sample/conditions, abbreviations;
  3. source and whether the figure is original, redrawn or extracted.
- Caption text is selectable text, never baked into the image.
- Tables: units in headers, consistent decimals, no vertical rules, header
  repeated on continuation, source note below.
- Figures: labeled axes with units, no background gridlines, no shadows, no
  decorative icons, no 3D, colorblind-safe palette, no color-only meaning.
- Minimum 5 pt in figure artwork; Folisift targets 7.5 pt and enforces 6.5 pt.
- Alt text describes the takeaway, not “image of …”; the caption does not
  replace alt text.
- Every extracted or redrawn figure records provenance and source page.

References:
- Caltech figure-caption guide:
  <https://writing.caltech.edu/documents/27629/HWC-FigureCaptionHandout.1-2024.pdf>
- Bucknell figure/table guidelines:
  <https://www.eg.bucknell.edu/~dkelley/eceg351/lab/eceg351FigTabCaptionGuidelines.pdf>
- Nature figure specifications:
  <https://research-figure-guide.nature.com/figures/preparing-figures-our-specifications/>
- ACS Energy Letters caption tips:
  <https://pubs.acs.org/doi/10.1021/acsenergylett.9b00253>
- Alt text vs caption:
  <https://openoregon.pressbooks.pub/dothework/chapter/3-2-strategies-and-examples>

### 2.3 What “professional satisfaction” means here

A professional/academic user should be able to say:

- “I can find the answer in seconds; the TOC and index are trustworthy.”
- “Every figure teaches something; none is decoration.”
- “I can cite the source page and see whether the figure was redrawn.”
- “Tables are readable, units are clear, and the header repeats.”
- “The PDF is accessible, tagged, bookmarked and printable.”
- “The same content produces the same output tomorrow.”

That becomes the acceptance lens for the phases below.

## 3. Product invariants (unchanged)

1. **MCP is hands; host LLM is brain.** No LLM inside the server.
2. **No shell, network or arbitrary code inside MCP.**
3. **One semantic `guide.md`; styles never change facts or coverage.**
4. **Content hash is stable across styles.**
5. **Every source subsection claimed exactly once.**
6. **Audit section is mandatory.**
7. **Deterministic rendering: same content + style + engine → same PDF.**
8. **No silent truncation or silent repair.**
9. **Visuals are content: captioned, sourced, explained and accessible.**
10. **External renderers are optional CLI tools, never MCP runtime deps.**

### 3.1 Non-goals

- No text-to-image or decorative image generation. Diagrams are structured
  explanations, not illustrations.
- No automatic fact generation or source invention.
- No OCR by default; low-text pages are inspected visually.
- No arbitrary Typst/CSS/plugin execution from a guide.
- No browser, Node, Graphviz or network call inside MCP.
- No claim of full PDF/UA certification from machine checks alone.
- No RTL or multi-user authentication in the v1.0 scope; separate plans.
- No replacement of the host LLM’s judgement about what matters.

## 4. Target pipeline

```text
source PDFs
  │
  ├─ text/layout/table extraction ──────────────┐
  │                                             │
  └─ page render + vector figure extraction ────┤
                                                ▼
host LLM: selects content, writes prose, table drafts, diagram specs
                                                │
                                                ▼
guide.md + assets/*.svg + diagrams/*.json
                                                │
                    ┌───────────────────────────┴───────────────────────┐
                    ▼                                                   ▼
           deterministic diagram compiler                    deterministic PDF renderer
           (JSON → SVG, theme-aware)                          (Typst, style pack)
                    └───────────────────────────┬───────────────────────┘
                                                ▼
                                     build cache + content hash
                                                │
                                                ▼
                          coverage + structure + layout + visual + a11y gates
```

The host LLM never writes raw Typst. It may write SVG only through a validated
asset path; the preferred path is a JSON diagram spec that the engine renders.

## 5. Workstream A — correctness gates (P0)

### A1. `covers` exactly-once and audit gate

- Parse `<!-- covers: PD0:0.1, PD0:0.2 -->` into a claim map keyed by
  `source:number`.
- Reject:
  - duplicate claims (same source section claimed twice);
  - unknown source id or section number;
  - sections claimed by a style/structure that does not list the required
    section;
  - missing audit marker.
- Require exactly one of:
  - a `::: chapter` with `audit=true`, or
  - `<!-- audit -->` immediately before the audit chapter, or
  - the localized audit heading from the style string pack.
- `check_coverage` returns `duplicate_claims`, `unknown_claims`,
  `audit_missing` and a machine-readable `issues[]` list.
- `guide_status` reports the same fields.
- Tests: duplicate claim, unknown claim, missing audit, A–G fallback,
  IMRAD with covers, multi-source collision (`PD0:0.1` vs `PD1:0.1`).

### A2. Raw Typst/CSS rejection

- Reject ` ```typst `, `#let `, `#set `, `#show `, `<style>`, `@page`,
  `class=` in guide bodies unless inside a fenced `text` block explicitly
  marked as a quotation.
- Add a lint result to `build_guide`; no silent pass.

### A3. Known small bugs

- `read_source` counts characters consistently (not bytes).
- `search_source` caps low-text page lists and reports `truncated`.
- `workspace_info` reports sidecar path/version without lying in dev builds.
- `guide_status` separates source-section count from all-section count.
- `list_guides.content_words` counts prose, not Markdown syntax.
- `PDFIUM_VERSION` generated from the same input as the sidecar.
- Add a full cache invalidation test (source → parser → cache → build).

## 6. Workstream B — tables as first-class content (P0)

### B1. Table directive v2

Extend `::: table` attributes:

```markdown
::: table id="t-compare" caption="Derleme ve yorumlama" source="PD0, s. 106–111"
kind="data" widths="20,40,40" header=true align="left,left,left"
summary="Derleme makine kodu üretir; yorumlama deyimi çalışma sırasında çevirir."
| Ölçüt | Derleme | Yorumlama |
|---|---|---|
...
:::
```

- `caption`, `id`, `source`, `summary`, `header`, `align`, `widths`.
- Auto-number tables (`Tablo 1`, localized); `id` gives stable cross-references.
- Table captions and source refs styled by the style pack; no raw Typst.
- Caption goes **above** the table; source note below.
- Table conventions: units in headers, consistent decimals, no vertical rules,
  header repeated on continuation, no colour-only status cells.
- Table caption anatomy: number + short title, then conditions/abbreviations,
  then source; no “Table showing …” filler.
- Complex merged cells use a JSON table block:

```markdown
```table-json
{"columns":[{"width":25},{"width":75}],
 "rows":[[{"text":"A","rowspan":2},{"text":"B"}],[{"text":"C"}]]}
```
```

The first release can reject `rowspan`/`colspan` with a clear error; add after
the simple path is stable.

### B2. Table extraction from source PDFs

Add a `TableExtractor` seam in `folisift-parse`:

```rust
pub trait TableExtractor {
    fn extract(&self, path: &Path, page: usize) -> Result<Vec<ExtractedTable>>;
}
```

- First adapter: **PDFium drawing rectangles + text alignment** (pure Rust,
  no extra runtime). Rect-based tables first; heuristic alignment second.
- Evaluate `pdf-inspector` (Rust) and `glyphrush-core` as optional adapters
  behind a feature flag; do not add an ML/ONNX stack to the default build.
- Every extracted table carries `confidence`, `warnings`, `cells`, `bbox` and
  a `draft` flag. Never treat extraction as verified content.
- CLI: `folisift table extract --source ... --page N --out data/tables/...`.
- MCP: no new tool. `analyze_source` may report `tables: {detected, uncertain}`
  counts; host reads the JSON file and verifies cells.

### B3. Table validation

- Column count consistency, empty header, duplicate header, missing caption
  if more than N rows, width sum ≈ 100, numeric column alignment.
- Table split: detect repeated header on continuation or a table block across
  pages; fail or warn by configured threshold.
- Table overflow: cell text outside its column/content box; fail.
- Table/caption separation: caption must stay with table.
- `build_guide` returns `tables: {count, warnings, failures}`.

## 7. Workstream C — diagram and SVG generation (P0/P1)

### C1. Principle

Do **not** make Mermaid or raw SVG the primary generation path. LLM-generated
Mermaid is brittle, and Mermaid CLI needs Node/Chromium; both conflict with the
“no shell/browser inside MCP” invariant. See:

- Mermaid syntax failures and repair loops:
  <https://microsoft.github.io/genaiscript/blog/mermaids>
- Mermaid CLI needs a browser for PNG:
  <https://github.com/mermaid-js/mermaid-cli>
- D2 has a native CLI and direct SVG export:
  <https://d2lang.com/tour/exports>
- Graphviz emits SVG from DOT; pin the layout engine/version because layouts
  are not guaranteed deterministic:
  <https://graphviz.org/docs/outputs/svg>

The engine gets a **small, typed diagram DSL** for the common study visuals.
Mermaid/D2/DOT are optional importers for authors, never MCP runtime deps.

### C2. Diagram spec (content format)

```markdown
```diagram
{
  "kind": "flow",
  "title": "Derleme hattı",
  "alt": "Kaynak programdan amaç koda uzanan altı aşamalı derleme hattı.",
  "source": "PD0, s. 35",
  "direction": "right",
  "groups": [
    {"id": "front", "label": "Ön uç", "nodes": ["lex", "parse", "sem"]}
  ],
  "nodes": [
    {"id": "src", "label": "Kaynak program"},
    {"id": "lex", "label": "Sözcüksel analiz"},
    {"id": "parse", "label": "Sözdizim analizi"},
    {"id": "sem", "label": "Anlam analizi"},
    {"id": "opt", "label": "Eniyileme"},
    {"id": "code", "label": "Kod üretimi"}
  ],
  "edges": [
    {"from": "src", "to": "lex", "label": "karakter akışı"},
    {"from": "lex", "to": "parse", "label": "token"},
    {"from": "parse", "to": "sem", "label": "AST"},
    {"from": "sem", "to": "opt", "label": "ara kod"},
    {"from": "opt", "to": "code"}
  ]
}
```
```

Kinds for the first release:

| Kind | Priority | Use | Layout |
|---|---|---|---|
| `concept` | high | definitions, relationships, big-picture maps | node–link graph with labeled edges |
| `flow` | high | pipelines, processes, SDLC | layered DAG, orthogonal edges |
| `tree` | high | parse trees, taxonomies, hierarchies | tidy tree |
| `matrix` | high | comparisons, 2×2, pros/cons | fixed grid |
| `annotated` | high | source figure with callouts | validated image/SVG + numbered labels |
| `sequence` | medium | fetch–execute, protocols | lifelines, ordered arrows |
| `timeline` | medium | history, evolution | horizontal time axis |
| `cycle` | medium | loops, PDCA, fetch–execute | circular layout |

Later: `plot` (bar/line), `venn`, `state`, `class`, `er`. Do not add a kind
without a fixture and a visual gate.

### C3. Deterministic SVG compiler

- `crates/folisift-diagram` (new crate) or `folisift-render::diagram`.
- Input: validated JSON spec + style tokens + font metrics.
- Output: standalone SVG + `title`/`desc` + stable hash.
- Layout:
  - deterministic topological layering;
  - no randomness;
  - text measurement with embedded Inter/Noto metrics (no overlap);
  - edge routing with orthogonal polylines and arrow markers;
  - group boxes drawn behind nodes;
  - minimum node width from text;
  - page-width-aware scaling but no shrinking below 6.5 pt.
- Output path: `guides/<slug>/assets/generated/<sha>.svg`, referenced by the
  figure block. Source spec lives in `guides/<slug>/diagrams/<id>.json` for
  reproducibility.
- Hash includes spec + tokens + engine version; unchanged spec reuses SVG.

### C4. SVG accessibility and safety

Every generated SVG:

```xml
<svg role="img" aria-labelledby="title desc" viewBox="...">
  <title id="title">...</title>
  <desc id="desc">...</desc>
  ...
</svg>
```

- No `<script>`, `<foreignObject>`, external `href`, CSS `url()`, event
  handlers, embedded fonts, or remote images.
- Text must not rely on colour alone; labels are visible.
- Contrast of text/fill checked against theme tokens.
- Minimum text size 6.5 pt in the final PDF; target 7.5 pt.
- `title` = short name; `desc` = one-sentence meaning and takeaway; caption
  below the figure does not replace `desc`.
- SVG sanitizer for host-authored SVG:
  - XML parse;
  - tag/attribute allowlist;
  - reject external references and scripts;
  - require `viewBox`;
  - require `title`/`desc`;
  - reject raster-only SVG without text description;
  - warn on text smaller than 6 pt or viewBox wider than 2× page width.

Research basis: MDN SVG accessibility, W3C SVG AAM, EU data-visualisation
guide:
- <https://developer.mozilla.org/en-US/docs/Web/SVG/Guides/SVG_in_HTML>
- <https://www.w3.org/TR/2023/WD-svg-aam-1.0/>
- <https://data.europa.eu/apps/data-visualisation-guide/accessible-svg-and-aria>

### C5. LLM workflow: specs, not drawings

- `prepare_guide` returns `visual_targets` derived deterministically from the
  inventory:
  - low-text pages;
  - pages with many rectangles/lines (potential diagrams);
  - pages with many table-like rows;
  - sections with no figure and > N chars;
  - suggested kinds (`flow`, `tree`, `sequence`, `timeline`, `matrix`) with
    page hints.
- It does **not** write diagrams.
- The host LLM chooses 1–3 high-value visuals per chapter and writes a JSON
  spec or a validated SVG.
- Prompt fragment in `prepare_guide`/`write-study-guide`:
  - use visuals only for mechanisms, processes, hierarchies, comparisons;
  - every figure needs caption + alt + source;
  - no decorative images, no “seductive details”;
  - prefer a diagram over a long paragraph when it shows structure.
- `build_guide` validates specs, renders SVGs, and returns a short failures
  list; the host fixes only the failing spec.

### C6. Optional importers (CLI only)

```bash
folisift diagram import --from mermaid diagram.mmd --out assets/x.svg
folisift diagram import --from d2 diagram.d2 --out assets/x.svg
folisift diagram import --from dot diagram.dot --out assets/x.svg
```

- Uses `mmdc`/`d2`/`dot` only if installed; no automatic install; no network.
- Sanitizes output; requires title/desc; applies style tokens where possible.
- Records the source and tool version in `diagrams/manifest.json`.
- Not exposed through MCP. The server never shells out.

This gives authors escape hatches without making the MCP server depend on
Node, Chromium or Graphviz.

### C7. Visual selection rules (hard product rules)

The engine cannot decide whether a visual is pedagogically useful, but it can
enforce the contract and make the host choose deliberately.

**Use a visual when the content is:**

- a process or pipeline (`flow`);
- a hierarchy, grammar or taxonomy (`tree`/`concept`);
- a comparison with 3+ criteria (`matrix`/table);
- a cycle or feedback loop (`cycle`);
- a sequence of events over time (`timeline`/`sequence`);
- spatial structure or anatomy (`annotated`);
- a big-picture relationship map (`concept`).

**Do not use a visual when the content is:**

- a single fact, date or definition;
- a list of unrelated items;
- decorative flavour, a stock photo, an AI-generated illustration, or a “fun”
  icon;
- a dense diagram that needs a second diagram to explain it.

**Visual budget:**

- 0–3 diagrams per chapter;
- 1 diagram per major mechanism;
- 0–4 tables per chapter;
- no page filled with decorative visuals;
- no visual without a one-sentence takeaway in the caption.

**Cognitive-load limits:**

- 12 nodes / 20 edges per `concept` or `flow` diagram;
- 4 levels deep for `tree`;
- 7 labels per row/column in `matrix`;
- split anything larger into “overview” + “detail” figures.

**Caption contract:**

```text
Şekil N. Short takeaway title
What each element/arrow means; units; conditions; source.
[PD0, s. 35] · redrawn / extracted / original
```

**Alt-text contract:**

```text
“Diagram showing X causing Y through Z; the key takeaway is …”
```

not “image of a diagram”.

**Color contract:**

- color is never the only carrier of meaning;
- colorblind-safe palette (Okabe–Ito or theme equivalent);
- direct labels preferred over legends;
- selected nodes may be highlighted with shape + color, not color alone.

## 8. Workstream D — source figure extraction (P1)

### D1. Vector figure extraction

CLI:

```bash
folisift figure extract \
  --source sources/PD0.pdf --page 35 \
  --box 30,180,560,520 \
  --out guides/pd0-pd1/assets/pipeline.svg
```

- First choice: PDFium path objects + text inside the box; reconstruct a clean
  SVG with a style-neutral palette.
- Fallback: Poppler `pdftocairo -svg` crop (development/CLI only; document the
  optional dependency).
- If the region is raster-only: render at 300 DPI and crop PNG; mark it
  `raster=true` and warn in the guide audit.

Reference: `pdf-extract-svg` demonstrates region-based PDF→SVG extraction with
Poppler:
<https://github.com/mbrukman/pdf-extract-svg>

### D2. `render_pages` crop support

Extend the existing `render_pages` tool with an optional crop box and a
`reason` field; no new tool. Returns path + DPI + page box. This helps the host
inspect a diagram before choosing to extract or redraw it.

### D3. Figure source rules

- A figure copied from the source must keep its source page ref.
- A redrawn figure is labelled as redrawn; do not pretend it is the original.
- A figure extracted from a PDF must be checked for clipped labels and
  missing text; the audit reports raster/vector and text count.
- Prefer a simple redraw over a low-quality vector extraction.

## 9. Workstream E — visual and layout QA (P0/P1)

### E1. Structural layout audit

Extend `audit_pdf` beyond page-level checks:

| Check | Method | Gate |
|---|---|---|
| Content-box overflow | PDFium char boxes + configured margins | fail |
| Header/footer collision | text bbox vs header/footer bands | fail |
| Orphan heading | heading font/size + bottom-band + no following body | fail |
| Widow line | last line only on next page | warn/fail threshold |
| Table split | repeated header + table block page span | fail small tables |
| Table caption separated | caption bbox page vs table bbox | fail |
| Figure/caption split | image bbox page vs caption bbox | fail |
| Box split | definition/critical/example/note page span | fail |
| TOC page resolve | TOC entries vs headings/pages | fail |
| Bookmark tree | depth/order vs TOC depth | fail |
| Code overflow | char bbox vs content width | fail |
| Page count | style target ± declared range | fail/warn |

Start with PDFium text boxes + page images; add a Typst frame-introspection
spike if needed for split detection. Do not use an LLM.

### E2. Visual regression

- Render selected pages at 150 DPI with PDFium.
- Store baselines per style in `tests/visual/<style>/`.
- Compare with SSIM + pixel mismatch:
  - SSIM ≥ 0.985;
  - mismatched pixels ≤ 0.5%;
  - mask nothing by default; allow per-page masks for anti-aliasing.
- Selected pages: cover, front, TOC, first chapter opener, a table page, a
  figure page, C glossary, audit page.
- A change requires an explicit baseline update commit.
- Reference: pixel + SSIM separation:
  <https://github.com/DemchaAV/GraphCompose/blob/main/docs/operations/visual-regression-testing.md>
- `paper` parity to the old PDF remains a **structural/page-map** gate; old-PDF
  pixel equality is not required because fonts and Typst layout differ.

### E3. PDF/UA and accessibility

- Internal:
  - tagged PDF;
  - bookmarks;
  - title/language metadata;
  - alt text per figure/table;
  - no text-only colour meaning;
  - contrast >= 4.5:1;
  - min text >= 6 pt (target 7.5 pt);
  - reading order from Typst headings/blocks.
- External CI (optional): `verapdf --flavour ua1` on generated PDFs. veraPDF is
  a reference validator, not a runtime dependency:
  <https://docs.verapdf.org/cli/validation>
- PDF/UA machine checks do not replace human review; report warnings.
- `clean` and `accessible` styles must pass; `paper` passes the practical
  subset.

### E4. Seeded-defect fixtures

Create fixtures that intentionally contain:

- orphan heading;
- table split with repeated header;
- figure/caption split;
- box split;
- TOC page mismatch;
- empty page;
- text outside content box;
- 5 pt text;
- low-contrast label;
- missing alt text.

Every check must fail on its fixture and pass on the real guide.

## 10. Workstream F — tokens, structures, domains, languages (P0/P1)

### F1. Apply style tokens

- Refactor `paper.typ`/`clean.typ` to read a `theme` object:
  `palette`, `typography`, `spacing`, `cover`, `tables`, `boxes`, `density`.
- Pass `tokens.json` into Typst as a JSON string or generated constants.
- Delete duplicated hard-coded colour/font blocks.
- `style preview` must visibly change when a token changes.
- Token schema versioned (`tokens.schema.json`); unknown keys warn.
- Validate contrast/body size at load (already started).
- Layout family still owns geometry; tokens own colour/type scale/spacing.

### F2. Structures

- Add `structures/<id>.json` with required sections, part labels and audit
  section metadata for all 8 structures.
- `prepare_guide` and `scaffold` seed the requested structure when
  `structure != a-g`.
- Coverage requires every required section and every source claim.
- A structure can be `stable` only after a fixture build passes coverage,
  layout and visual gates.
- Initial stable: `a-g`, `lecture-notes`; others experimental.

### F3. Domain profiles

- Inject domain prompt fragment into `prepare_guide` / `write-study-guide`
  responses (server returns text; host consumes; no LLM call).
- Add domain checklists to `build_guide` response (`checks`, not content).
- Domain profiles select a default structure but never override an explicit
  user choice without a warning.
- Add `domains/*.schema.json` and a lint for required keys.

### F4. Languages

- `tr`, `en` stable; add `de`/`fr` only after font/hyphenation tests.
- Localization lint over the actual PDF text:
  - English build must not contain Turkish UI labels;
  - Turkish build must not contain English UI labels from style packs;
  - guide content is allowed to quote source-language terms.
- Number/date formatting per locale.
- RTL deferred; document the font/bidi/table test plan.

## 11. Workstream G — build cache and performance (P0)

### G1. Build cache

- Key: `content_hash + style_pack_hash + tokens_hash + engine_version +
  template_hash + font_hash + format`.
- Store under `data/cache/build/<key>/`:
  - `output.pdf`
  - `audit.json`
  - `coverage.json`
  - `content_hash.txt`
- Atomic write; no partial files.
- `build_guide` returns `cache_hit` and the same hash/page count.
- Source cache and build cache are separate; style changes reuse source cache.
- A changed template/font/style invalidates the build.

### G2. Preview cache

- `style preview` caches by style/tokens/language/sample hash.
- `diagram` SVG cache by spec hash/theme hash.

### G3. Bounded responses

- `build_guide` returns only counts, paths, hashes and failures; no full
  document dump.
- `audit` returns only failing pages and rules.
- `prepare_guide` returns `visual_targets` bounded to N pages.
- `read_source`/`search_source` report `truncated` and `next`.

## 12. Workstream H — platform, security, release (P1)

### H1. CI

- CI is defined in `.github/workflows/ci.yml`; GitHub Actions is currently
  blocked by account billing. Fixing billing is an external prerequisite.
- Add:
  - visual snapshot job;
  - veraPDF job (installed in CI, not shipped);
  - style/token fixture job;
  - diagram fixtures job.
- Cross-platform compile/test matrix stays.

### H2. Packaging and signing

- Linux archive exists; add macOS (`tar.gz`) and Windows (`zip`) archives with
  the matching PDFium sidecar.
- Signing requires key material; do not fake it. Add a release checklist gate:
  - unsigned release is allowed only with a visible `UNSIGNED` marker and
    SHA-256;
  - signed release requires `minisign`/`cosign`/platform certificate.
- `doctor` reports signature status if a signature file is present.

### H3. HTTP MCP security

- Default bind is loopback only.
- Refuse non-loopback bind unless `--allow-remote` and a bearer token
  (`FOLISIFT_MCP_TOKEN`) are set.
- Cap request size, sessions and concurrent render jobs.
- No auth in scope for v0.6; document it as a hard constraint.

## 13. Data formats

### 13.1 Diagram spec

JSON with `schema_version`, `kind`, `title`, `alt`, `source`, `direction`,
`nodes`, `edges`, `groups`, `notes`. Validate IDs, duplicate edges, unknown
node refs, cycles for `tree`, empty labels, max 12 nodes / 20 edges, max text
length.

### 13.2 Table spec

JSON with `columns` (width, align, header), `rows` (cells with optional
rowspan/colspan), `caption`, `source`, `summary`. Markdown tables remain the
default; JSON is only for complex tables.

### 13.3 Figure manifest

`guides/<slug>/diagrams/manifest.json`:

```json
{
  "schema_version": 1,
  "figures": [
    {
      "id": "pipeline",
      "spec": "diagrams/pipeline.json",
      "svg": "assets/generated/<hash>.svg",
      "hash": "...",
      "kind": "flow",
      "engine": "0.6.0"
    }
  ]
}
```

### 13.4 Layout audit report

`data/audit.json` with page-level metrics and a flat `issues[]` list:
`code`, `page`, `severity`, `message`, `bbox`.

## 14. Tool and CLI surface

No new MCP tool in v0.6. The 12-tool cap stays.

| Operation | Surface |
|---|---|
| diagram spec validation/render | CLI `diagram build`, `build_guide` |
| diagram import (Mermaid/D2/DOT) | CLI only |
| figure extraction | CLI `figure extract` |
| crop inspection | `render_pages` optional `box` |
| table extraction | CLI `table extract`; inventory counts |
| visual audit | `build_guide` + `audit` |
| style preview | `style` tool |
| structure/domain validation | `prepare_guide`, `build_guide` |

If a future tool is unavoidable, add one `visual` tool with
`build|extract|validate` and remove/merge a low-value tool; do not exceed 12.

## 15. Phases and sequencing

Durations assume one focused engineer; multiply for review/iteration.

### Phase 1 — Gates and cache (3–4 days)

- `covers` exactly-once + audit marker.
- Raw Typst/CSS rejection.
- Build cache + `cache_hit`.
- Small bug fixes.
- Exit: seeded fixtures fail; cache hit proven; `paper` still 22 pages.

### Phase 2 — Table system (4–6 days)

- Table directive v2 (caption/id/source/summary/align).
- Table validation and split/caption checks.
- `table extract` adapter (rect-based first).
- Exit: simple table fixture extracted/reviewed; table audit catches splits;
  `paper` table pages unchanged.

### Phase 3 — Diagram compiler (5–7 days)

- New diagram crate/module; `concept`, `flow`, `tree`, `sequence`.
- Style-aware SVG; accessibility title/desc; deterministic hash.
- `diagram` fenced block preprocessing; cache.
- SVG sanitizer for host-authored assets.
- Exit: fixtures for four kinds; SVG validation; no overlap; PDF embedding.

### Phase 4 — Visual workflow and source extraction (4–5 days)

- `prepare_guide.visual_targets`.
- `render_pages` crop box.
- `figure extract` vector/raster path.
- Prompt fragments and docs.
- Optional Mermaid/D2/DOT import CLI.
- Exit: host can choose a low-text page, extract/redraw, and build without
  manual SVG editing.

### Phase 5 — Visual/layout/accessibility gates (5–7 days)

- Content-box/orphan/split/TOC/bookmark audit.
- SSIM/pixel baselines per style.
- PDF/UA internal checks; optional veraPDF CI.
- Seeded defect fixtures.
- Exit: every seeded defect caught; paper/clean baselines stable.

### Phase 6 — Tokens/structures/domains (4–6 days)

- Token-driven templates.
- Structure packs and section validation.
- Domain prompt/checklist integration.
- Localization lint.
- Exit: token change visibly changes preview; IMRAD/IRAC/SOAP fixtures build
  with covers; English build has no Turkish UI labels.

### Phase 7 — Platform/release (4–6 days)

- macOS/Windows archives; CI matrix green (billing permitting).
- Signing policy and optional signature verification.
- HTTP auth/loopback enforcement.
- Exit: clean install on three platforms; release gate documented.

## 16. Acceptance criteria

### Correctness

- [x] Duplicate/unknown `covers` fail with the exact source/section.
- [x] Missing audit section fails.
- [x] Raw Typst/CSS is rejected with line numbers.
- [x] Build cache returns the same PDF/hash and `cache_hit=true`.
- [x] Every PDF audit issue is page/rule specific.

### Tables

- [x] Table caption/id/source survive all styles.
- [x] Simple table extraction fixture is recovered with cells and confidence.
- [x] Table split, caption split, column mismatch and overflow fail. Non-breakable table blocks prevent caption/table splits; width sum, column count and unsafe ids are rejected.
- [x] `paper` 24 tables and `clean` 24 tables still pass. The 24-table
  stress fixture renders and audits in both styles.

### Diagrams/SVG

- [x] `concept`, `flow`, `tree`, `sequence` fixtures render deterministically.
- [x] A diagram with >12 nodes is rejected with a split recommendation.
- [x] A decorative/stock image is rejected by the visual lint.
- [x] Every figure has a takeaway caption, provenance and alt text.
- [x] Table captions are above tables; figure captions are below figures.
- [x] Every generated SVG has `role`, `title`, `desc`, `viewBox`, no scripts
      and no external refs.
- [x] Diagram text is >= 6.5 pt and does not overlap. Width is capped at 900
      units so the 12 pt SVG font stays above 6.5 pt at page width.
- [x] Same spec + theme + engine → same SVG hash.
- [x] Host-authored SVG with script/external ref is rejected.
- [x] A low-text source page can be inspected/extracted/redrawn and embedded. `render-pages --crop` produces a normalized crop for a figure asset.

### Visual/accessibility

- [x] Orphan heading fixture fails. `tests/fixtures/pdf/orphan-heading.pdf` is
  checked in and `seeded_orphan_heading_fails` proves the audit catches it.
- [x] Table/figure/box split fixtures fail. Non-breakable table blocks prevent
  table/caption splits; `box-overflow.pdf` and `image-overflow.pdf` are checked
  in and negative tests prove content-box and image/figure overflow detection.
- [x] TOC/bookmark mismatch fails. Bookmark destinations are matched against extracted page text.
- [x] Empty/overflow/missing-glyph pages fail.
- [x] PDF/UA internal checks pass for paper/clean; veraPDF optional CI. Tagged text, alt text, bookmarks, minimum font size and destination checks are enforced. veraPDF remains an external optional gate.
- [x] SSIM/pixel baselines exist for selected pages. Pixel baselines cover the paper guide and the clean preview.

### Styles/structures

- [x] Token change changes preview without changing content.
- [x] IMRAD/IRAC/SOAP/formula/literature/brief fixtures build with coverage. Every pack renders in both languages; coverage remains source-dependent.
- [x] Domain profile appears in prepare/build output.
- [x] English guide has no Turkish UI labels.
- [x] `same content hash across styles` test still passes.

### Platform

- [ ] Linux/macOS/Windows archives documented and built. Linux is built by
  `make package`; macOS/Windows remain source builds until the required
  toolchains and CI runners are provisioned. This is an external blocker, not
  an engine gap.
- [x] Signature policy is explicit; unsigned archives are marked.
- [x] HTTP non-loopback requires a token and is documented as unsupported
      otherwise.

## 17. Test fixtures

```text
tests/fixtures/
├── style-sample.md                 # cross-style content fixture
├── covers-duplicate.md             # negative coverage fixture
├── audit-missing.md                # negative coverage fixture
├── raw-typst.md                    # negative render fixture
├── diagrams/
│   ├── concept.json
│   ├── flow.json
│   ├── tree.json
│   └── sequence.json
├── svg/
│   ├── invalid-script.svg
│   └── valid-accessible.svg
└── pdf/
    ├── generate.py                 # byte-for-byte fixture generator
    ├── orphan-heading.pdf
    ├── box-overflow.pdf
    ├── image-overflow.pdf
    └── bookmark-mismatch.pdf
```

`paper` and `clean` build every stable fixture; content hash and coverage are
compared; visual snapshots are committed only for selected pages. The
24-table stress fixture is generated in `crates/folisift-cli/tests/table_parity.rs`.

## 18. Research notes

### Layout backend decision

The layout and SVG engine evaluation is in
`docs/diagram-engine-evaluation.md`. Final decision: **D2 v0.9.0 is the single
external diagram engine**, pinned and invoked only by the CLI. It covers flow,
concept, tree, sequence and container diagrams with TALA. The typed JSON spec,
D2 source generation, SVG sanitisation, accessibility post-processing and Typst
integration stay in Rust. There is no Graphviz fallback and no second external
backend. MCP and guide builds embed only pre-generated, sanitised SVG. ELK-rs
is rejected as too young (9 stars, created 2026); Eclipse ELK's Java/JS runtime
is not a fit. Full in-house layout is the expensive, low-differentiation part.

### Diagrams and LLM reliability

- LLM-generated Mermaid often has syntax errors and needs a repair loop:
  <https://microsoft.github.io/genaiscript/blog/mermaids>
- Mermaid CLI needs a browser for PNG rendering:
  <https://github.com/mermaid-js/mermaid-cli>
- D2 renders SVG natively and has deterministic hash prefixes:
  <https://d2lang.com/tour/exports>
- Graphviz has direct SVG output from DOT; layout determinism depends on the
  engine/version:
  <https://graphviz.org/docs/outputs/svg>
- Typst/CeTZ can draw scientific diagrams, but introduces Typst package
  dependencies; use only if the typed DSL is insufficient:
  <https://typst.app/universe/package/cetz>
- DiagramEval argues diagrams should be evaluated as graphs (nodes/edges/path
  alignment), not pixel similarity:
  <https://arxiv.org/abs/2510.25761>
- FigureCode treats figures as editable HTML/SVG code and reports accessibility
  gains:
  <https://aclanthology.org/2026.findings-emnlp.2060.pdf>

### SVG accessibility

- MDN SVG in HTML: `role="img"`, `<title>`, `<desc>`, `aria-labelledby`:
  <https://developer.mozilla.org/en-US/docs/Web/SVG/Guides/SVG_in_HTML>
- EU data-visualisation guide:
  <https://data.europa.eu/apps/data-visualisation-guide/accessible-svg-and-aria>
- SVG AAM:
  <https://www.w3.org/TR/2023/WD-svg-aam-1.0/>

### Table extraction

- `pdf-inspector` reports rect-based + text-alignment table detection:
  <https://github.com/firecrawl/pdf-inspector>
- Glyphrush exposes structured table blocks and uncertainty flags:
  <https://docs.rs/crate/glyphrush-core/0.1.0>
- Table extraction is not a solved problem; benchmarks show large
  differences and failure on merged cells/cross-page tables:
  <https://arxiv.org/html/2603.18652v1>
  <https://arxiv.org/html/2511.16134v1>

### Visual regression and PDF/UA

- Separate semantic layout snapshots from pixel regression:
  <https://github.com/DemchaAV/GraphCompose/blob/main/docs/operations/visual-regression-testing.md>
- Pixelmatch vs SSIM; thresholds and anti-aliasing false positives:
  <https://vitest.dev/guide/browser/visual-regression-testing>
- veraPDF PDF/UA validation:
  <https://docs.verapdf.org/cli/validation>
- PDF/UA machine checks do not replace human review:
  <https://pdfa.org/resource/the-matterhorn-protocol/>

### Learning and visuals

- Dual coding / multimedia principle: words + relevant pictures beat words
  alone; avoid seductive details:
  <https://multimedia.ucsd.edu/best-practices/multimedia-learning.html>
- Six evidence-based study strategies:
  <https://pmc.ncbi.nlm.nih.gov/articles/PMC5780548/>
- Concept maps meta-analysis (science achievement `g = 0.776`):
  <https://link.springer.com/article/10.1007/s10648-024-09877-y>
- Studying and constructing concept maps meta-analysis (`g = 0.58` overall,
  construction `g = 0.72`):
  <https://doi.org/10.1007/s10648-017-9403-9>
- Graphic organizers meta-analysis (instructor-provided memory `g = 0.70`):
  <https://eric.ed.gov/?id=EJ1496202>
- Drawing-to-learn study (50–80% retention advantage in one biology sample):
  <https://pmc.ncbi.nlm.nih.gov/articles/PMC5459246>
- Sketchnoting/visual note-taking:
  <https://files.eric.ed.gov/fulltext/EJ1341649.pdf>
- Multimedia principles meta-analysis (multimedia, signaling, segmenting,
  seductive details):
  <https://www.sciencedirect.com/science/article/pii/S1747938X25000673>
- Seductive details meta-analysis:
  <https://link.springer.com/article/10.1007/s10648-020-09522-4>
- Visual explanations reduce transient information load:
  <https://www.tes.com/magazine/archive/why-every-teacher-should-be-using-dual-coding>

### User signals

- Study guide thread (tables/diagrams, “making the guide is studying”):
  <https://www.reddit.com/r/IWantToLearn/comments/5fxjkw/iwtl_how_to_make_a_study_guide>
- Biology flowcharts and minimal-word diagrams:
  <https://www.reddit.com/r/biology/comments/qwvuy7/study_tips_for_general_biology_college_level>
- Descriptive exams: textbook headings/tables/flowcharts:
  <https://www.reddit.com/r/GetStudying/comments/1o6hnvj/how_do_i_use_spaced_repetition_for_descriptive>
- Mind maps: connections vs decoration:
  <https://www.reddit.com/r/GetStudying/comments/1ldlokz/are_mind_maps_actually_good_for_studying>
- Mind-map generation time complaint:
  <https://www.reddit.com/r/studytips/comments/1maqd9l/mindmap_generator_mindgrasp_chat_gpt>
- Nursing concept mapping:
  <https://www.reddit.com/r/StudentNurse/comments/grg6c3/tip_concept_mapping_as_a_visual_thinking_strategy>
- Diagram note-taking tools:
  <https://www.reddit.com/r/productivity/comments/uotd8g/note_taking_and_diagrams>

### Professional visual standards

- Caltech figure captions:
  <https://writing.caltech.edu/documents/27629/HWC-FigureCaptionHandout.1-2024.pdf>
- Bucknell figure/table caption rules:
  <https://www.eg.bucknell.edu/~dkelley/eceg351/lab/eceg351FigTabCaptionGuidelines.pdf>
- Nature figure specifications (units, accessibility, no decoration):
  <https://research-figure-guide.nature.com/figures/preparing-figures-our-specifications/>
- ACS caption tips:
  <https://pubs.acs.org/doi/10.1021/acsenergylett.9b00253>
- Alt text vs caption:
  <https://openoregon.pressbooks.pub/dothework/chapter/3-2-strategies-and-examples>

### MCP and tool design

- Tools should be intent-level, bounded and idempotent; do not mirror every
  endpoint:
  <https://www.anthropic.com/engineering/writing-tools-for-agents>
- Cache read-heavy operations and cap responses:
  <https://aws.amazon.com/blogs/machine-learning/mcp-tool-design-practical-approaches-and-tradeoffs>
- Idempotency and retries:
  <https://www.firecrawl.dev/blog/agent-tools>

## 19. Risks

| Risk | Mitigation |
|---|---|
| Diagram DSL becomes a second layout engine | small kind set, engine-owned geometry, fixtures, no arbitrary code |
| LLM writes bad SVG | validator + sanitizer; prefer JSON spec |
| Table extraction silently wrong | confidence + draft flag + host review; never auto-publish |
| Visual regression brittle | semantic audit first; SSIM threshold; explicit baseline updates |
| PDF/UA claims overreach | machine checks only; human review documented |
| More styles multiply work | layout families + tokens; stable/experimental gate |
| CI blocked by billing | local gates remain; release cannot claim CI-green until fixed |
| Signing keys unavailable | unsigned release marked; checksum only |
| Tool count creep | 12-tool cap; CLI for visuals; one visual tool only by ADR |
| External renderers add runtime deps | CLI opt-in only; MCP never shells out |

## 20. Open decisions (recommended answers)

1. **Diagram DSL vs Mermaid first?** DSL first; Mermaid/D2/DOT optional CLI.
2. **New MCP tool for visuals?** No; extend `build_guide`/`prepare_guide`.
3. **Table extraction default?** Rect-based draft + confidence; no ML default.
4. **PDF/UA gate?** Internal hard checks; veraPDF optional CI, not runtime.
5. **Visual regression scope?** Selected semantic pages per stable style.
6. **New styles now?** Add `accessible` next; journal/brief/clinical/casebook
   only with fixtures and gates.
7. **Signing?** Explicit unsigned release policy until keys exist.
8. **RTL?** Deferred; separate font/bidi/table plan.

## 21. Definition of done

A user can ask:

> “Bu bölümü sınav için hazırla; derleme hattını şema olarak göster,
> tabloları çıkar, İngilizce clean düzende üret.”

and Folisift:

1. extracts source text and table drafts;
2. suggests visual targets from low-text/diagram-heavy pages;
3. lets the host write a diagram spec or validated SVG;
4. renders deterministic, accessible SVG;
5. builds the guide in the chosen style/language;
6. enforces coverage exactly once, audit section, table/figure/box splits,
   content-box overflow, TOC/bookmark and accessibility gates;
7. returns a cached PDF and a short, machine-readable report;
8. never calls an LLM, shell or network from MCP.
