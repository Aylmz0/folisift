# Folisift v0.9 — Guide Quality, Visual Coverage and Style Selection

- **Status:** Draft for implementation
- **Date:** 2026-10-29
- **Baseline:** `v0.8.0` (107 tests, 13 MCP tools, MCP schema 3)
- **Trigger:** the 16-minute PD2/PD3 production run
- **Artifacts under review:**
  `guides/pd2/`, `guides/pd3/`,
  `output/PD2_Calisma_Ozeti.pdf`, `output/PD3_Calisma_Ozeti.pdf`
- **Related:** `docs/plans/2026-10-26-mcp-v0.2-plan.md`,
  `docs/plans/2026-10-26-rust-mcp-typst.md`,
  `docs/plans/2026-10-28-roadmap.md`,
  `docs/plans/2026-10-28-style-language-profiles.md`,
  `docs/plans/2026-10-28-visual-parity-and-production.md`,
  `docs/plans/2026-10-29-production-hardening-and-visuals.md`,
  `docs/plans/2026-10-29-d2-pipeline-and-opencode-automation.md`,
  `docs/authoring-spec.md`, `docs/design-system.md`,
  `docs/agents/opencode-guide.md`, ADR-0004, ADR-0007, ADR-0010

## 1. Executive summary

The pipeline works mechanically: both PDFs exist, page counts match, fonts are
embedded, bookmarks and TOC are present, coverage returns PASS, and the layout
audit reports no empty/overflow/glyph failures.

The **quality contract** is not yet met:

1. PD2 has **zero content figures** and **zero D2 diagrams**; PD3 has **two
   hand-written SVGs** and **zero D2 diagrams**.
2. The G sections claim that all low-text slides were visually inspected, but
   no persisted render/visual-decision evidence exists under either guide.
3. Coverage is a false positive: PD2 reports **13 sections**, PD3 **4**, while
   the guides claim **20/20** and **5/5**. The gate cannot see the sections the
   inventory missed.
4. Both PDFs contain near-empty section-tail pages (PD2 pages 16 and 20; PD3
   page 17) that the audit does not flag.
5. Style selection is silent: both configs use `paper`/`a-g`/`academic`/`exam`
   by default; there is no recommender, no style rationale, and only two
   layout packs (`paper`, `clean`) exist.
6. Several style/profile axes are metadata only: `voice`, `study_mode`,
   `citation` and domain defaults are validated but not injected into prompts
   or rendering.
7. The two PDFs became **two separate guides**. If the request was one guide
   over two sources, that is a scope error; the prompt did not make the
   single-vs-per-source decision explicit.

The 16-minute duration is not itself a defect. The source text is small
(PD2 6,448 words; PD3 6,574 words), so writing 5–6k words quickly is plausible.
The defect is that speed came from skipping visual extraction, not from a
faster visual workflow.

## 2. Evidence

### 2.1 Output metrics

| Metric | PD2 | PD3 |
|---|---:|---:|
| Pages | 22 | 19 |
| PDF bookmarks | 82 | 58 |
| Content words | 8,184 | 6,910 |
| Prose words | 6,272 | 5,272 |
| Source slides | 103 | 136 |
| Low-text source pages | 9 | 18 |
| Tables | 6 | 6 |
| Figures / images in Markdown | 0 | 2 |
| D2 `::: diagram` directives | 0 | 0 |
| `diagrams/` specs | none | none |
| Raster images in PDF (`audit`) | 0 | 0 |
| Coverage sections seen by gate | 13 | 4 |
| Coverage sections claimed in G | 20/20 | 5/5 |
| Audit result | pass | pass |

Commands used:

```bash
folisift guide-status pd2
folisift guide-status pd3
folisift audit --pdf output/PD2_Calisma_Ozeti.pdf --expected-pages 22
folisift audit --pdf output/PD3_Calisma_Ozeti.pdf --expected-pages 19
```

### 2.2 Page density

Body character counts from `pdftotext`:

| PDF | Normal body pages | Suspicious pages |
|---|---|---|
| PD2 | ~2,700–3,400 chars | p16: 310, p20: 320 |
| PD3 | ~1,700–3,000 chars | p17: 450 |

PD2 page 16 contains only the last three bullets of “Bölüm 2’nin sentezi”.
PD2 page 20 contains only two source-consistency bullets. PD3 page 17 contains
three end-of-section bullets. All three are followed by a new `::: chapter`
opener, so the page break is intentional but the page is professionally
underfilled.

### 2.3 Visual assets

```text
guides/pd2/assets/cover-motif.svg        # decorative cover only
guides/pd2/diagrams/                     # absent
guides/pd3/assets/cover-motif.svg        # decorative cover
guides/pd3/assets/parse-tree.svg         # manual SVG, no D2 spec
guides/pd3/assets/operational.svg        # manual SVG, no D2 spec
guides/pd3/diagrams/                     # absent
```

No `data/pages/` or other rendered source-page evidence exists under either
guide. The guide text nevertheless says “Metin içermeyen eğitsel slaytlar
görsel olarak incelenmiştir.” This is an unauditable claim, not proof of
fabrication: the host may have inspected images without keeping evidence, but
the repository cannot verify it.

### 2.4 Coverage mismatch

Inventory heading extraction found:

- PD2: 18 headings, 13 numbered sections (`2.1, 2.3, 2.4, 2.7, 2.8, 2.10,
  2.12, 2.14, 2.16–2.20`). The guide has all 20 sections (`2.1–2.20`).
- PD3: 12 headings, 4 numbered sections (`3.1, 3.3, 3.4, 3.5`). The guide has
  all 5 sections (`3.1–3.5`).

The missing headings are present in the source topic-list pages:

- PD2 pages 2–4 list 20 topics.
- PD3 page 2 lists 5 topics.

`prepare_guide` uses only detected headings, so the coverage gate never learns
that sections 2.2, 2.5, 2.6, 2.9, 2.11, 2.13, 2.15 and 3.2 exist. The guide
author added them manually, but the gate did not verify them. The G claim is
therefore self-attested.

## 3. Findings

### P0-1 — Visuals are optional in the contract

Evidence:

- `prepare_guide.visual_rules` says “Use 0-3 meaningful diagrams per chapter”.
  Zero is valid, so a text-only guide passes.
- `write-study-guide` step 8 says “For each mechanism that needs a visual …”.
  The model can decide that no mechanism needs one.
- `build_guide` and `check_coverage` never require a figure, a diagram or a
  visual decision.
- `audit` reports `image_pages: 0` even for PD3, because the two SVGs are
  vector drawings, not raster images.

Root cause: the engine exposes a visual capability but no visual **contract**.
The host agent is rewarded for covering headings, not for covering the visual
structure of the source.

### P0-2 — Coverage can pass without covering the source

Evidence: PD2 gate sees 13 of 20 sections; PD3 sees 4 of 5; both pass.

Root cause: the inventory is built from font-size/heading heuristics, not from
the source’s own topic list. The gate compares guide headings with that
incomplete inventory. A section the extractor misses is invisible to the gate.

This is the most dangerous defect: the product reports “coverage PASS” and the
guide reports “20/20” while the machine has verified only 13/20.

### P0-3 — Underfilled pages are invisible to the layout audit

Evidence: PD2 p16/p20 and PD3 p17 contain 10–20% of the median body text, but
`audit` only checks `chars < 5` for empty pages and does not measure vertical
fill or chapter-tail density.

Root cause: `PageAudit` already records `chars` and `text_bottom`, but no rule
uses them. The chapter opener uses `pagebreak(weak: true)`; the preceding
section can end at the top of a page, leaving a nearly blank page.

### P0-4 — G sections are prose, not evidence

Evidence: G claims all low-text pages were inspected; no `visual-audit.json`,
render manifest, image hash or per-page decision exists. Some G descriptions
are plausible but not traceable. PD2 G calls the D.3 monospace text tree a
“şema”; it is not a rendered figure. PD3 G says pages 73/74, 76/77 and 111 were
handled “metin”/“kavramsal açıklama”, but those are exactly the pages where a
diagram would carry the mechanism.

Root cause: G is written by the host as free prose. There is no generated
source-of-truth for visual coverage, so the host can write a confident summary
without a corresponding artifact.

### P1-1 — Vector figures are not counted or validated

Evidence: `audit_pdf` increments `image_pages` from `page.image_bounds()`,
which sees raster image XObjects. PD3’s SVG figures are vector, so the audit
says `image_pages: 0`. Manual SVGs are not checked for `<title>`, `<desc>`,
`role`, source manifest, or D2 provenance.

Root cause: the audit measures PDF image objects, not the guide’s figure
contract. `build_guide` knows the Markdown assets but does not return figure
statistics or enforce a manifest.

### P1-2 — Style selection is a silent default

Evidence: both configs contain:

```json
{
  "aesthetic": "paper",
  "structure": "a-g",
  "voice": "academic",
  "study_mode": "exam",
  "domain": "cs",
  "citation": "page-ref"
}
```

`build_style` defaults to `paper`/`a-g`; `prepare_guide` has no
`style_candidates` or `recommend` action. The host did not choose a style; it
inherited one. `voice`, `study_mode` and `citation` are validated but not used
by the renderer or the prompt. `domain_profile.structure_default` is returned
but not applied: humanities still gets `a-g`, math still gets `a-g`.

Root cause: the style registry is a compatibility layer, not a decision layer.
There is no mapping from domain/source/audience to a recommended style and
structure.

### P1-3 — Only two style packs exist, and some domains cannot be used

Evidence:

- Built-in packs: `paper` (stable), `clean` (experimental).
- `paper` supports domains `general, cs, math, humanities`.
- `clean` supports domains `general, cs, math, business`.
- The style plan advertises `journal`, `brief`, `clinical`, `casebook` and
  `accessible`; none has a pack or layout family.
- `prepare_guide(domain="law")` and `domain="medicine"` fail with
  `style "paper": domain ... is not supported`, even though `irac`/`soap`
  structures and law/medicine domain profiles exist.
- `domain="business"` fails on paper and only works if the host explicitly
  selects `clean`.

Root cause: the domain matrix, structure matrix and style packs were designed
independently. The advertised breadth is not implemented.

### P1-4 — `paper` and `clean` are skins, not full professional styles

Evidence:

- Both use 9.5 pt body type; paper uses Noto Serif/1.33, clean uses Inter/1.38.
- They differ mainly in palette, body font, cover, density and accent colour.
- Structure, voice and study mode are separate axes; neither pack supplies a
  narrative template beyond labels.
- `templates/clean.typ` still begins with the comment “Folisift paper
  template”, a copy-paste leftover.

Root cause: layout family differences were limited to tokens and a few
component choices. The engine has no per-style typographic scale, measure,
margin system, heading hierarchy, citation treatment or narrative contract.

### P1-5 — Manual SVGs bypass the D2 contract

Evidence: PD3’s `parse-tree.svg` and `operational.svg` are hand-written SVGs.
They have no `diagrams/<id>.json`, no manifest, no D2 source and no deterministic
spec hash. The parse-tree figure has a `program` text label above a `<program>`
node and floating `:=`/`+` labels without connecting lines; it was not validated
by the D2 spec rules.

Root cause: the authoring contract says manual SVG is an exception for
annotated source figures, but the build accepts it without an exception record.
The host can bypass D2 entirely and still pass.

### P1-6 — Single-guide vs per-source-guide decision is not explicit

Evidence: the run produced `guides/pd2` and `guides/pd3` separately. The
authoring model supports one guide with multiple sources and one B part per PDF.
If the user intended one combined guide, the output is wrong; if the user
intended two booklets, it is right. The prompt did not force the decision.

Root cause: `prepare_guide` accepts multiple sources but does not return a scope
recommendation or ask the host to state the choice.

### P2-1 — Figure/table numbering and captions are manual

Evidence: PD3’s two figures use hand-written “Şekil 3.1/3.2” captions; PD2’s
D.3 is a monospace text tree, not a numbered figure. The renderer supports
figure captions but does not allocate figure numbers or cross-reference them.

Root cause: the figure contract stops at Markdown image syntax; counters and
cross-references are left to the host.

### P2-2 — Audit does not verify TOC page numbers

Evidence: `audit` checks bookmark destinations, not the printed TOC entries.
The current TOCs happen to match, but a TOC regression would pass.

Root cause: the audit reads the outline, not the TOC page text.

### P2-3 — Stale plan checkboxes hide completed work and open work

Examples:

- `2026-10-28-style-language-profiles.md` has unchecked “No raw Typst/CSS” and
  “No style can remove the audit section”; both are covered by v0.6 tests.
- `2026-10-28-visual-parity-and-production.md` has unchecked visual diff, MCP
  smoke, `prepare_guide` and checksum items; all are now verified.
- `2026-10-29-production-hardening-and-visuals.md` still lists G3 (domain
  injection), G17 (missing style families), G21 (visual selection quality) and
  G23 (caption/alt standards) as open, correctly, but the plan status says
  “Draft” and the new gaps are not listed.

## 4. Why the run produced no diagrams

This is a chain of incentives, not a single bug:

1. The user asked for a summary; the prompt did not explicitly require
   diagrams.
2. `prepare_guide` returned `visual_targets`, but they were advisory. It also
   included page 1 (cover), page 45 (COBOL photo) and page 101 (cartoon),
   diluting the meaningful targets.
3. `write-study-guide` made visuals conditional (“for each mechanism that
   needs a visual”).
4. `build_guide`/`check_coverage` only measured headings, so a text-only guide
   passed.
5. `audit` only measured layout of what existed; zero figures was not a
   failure.
6. The host optimized for the visible gate: coverage PASS and a 22-page PDF.
7. D2 existed, but using it required extra host steps (`diagram validate`,
   `diagram save`, shell `diagram build`, `diagram status`). The model took the
   cheaper path.

The PD2 source makes the miss obvious: page 5 is a language genealogy chart,
page 29 is a two-representation LISP list diagram, and page 53 is a BASIC
lineage chart. PD3 pages 73/74, 76/77 and 111 are attribute-flow, syntax-graph
and parse-tree material. These are not decorative; they are the mechanism.

## 5. Prior plan reconciliation

| Plan | Claim | Audit result |
|---|---|---|
| `2026-10-26-mcp-v0.2-plan.md` | OpenCode smoke test with real prompt/model still open | Functional smoke passed on 2026-10-29: MCP connected, PD2/PD3 produced. **Quality smoke failed**; keep open until this plan’s gates pass. |
| `2026-10-26-rust-mcp-typst.md` | Phase 0–6 complete; CI pending | Complete. CI remains an external billing blocker; do not call it green. |
| `2026-10-28-style-language-profiles.md` | S0–S7 landed; paper/clean | S0–S7 landed for paper/clean. The advertised `journal`, `brief`, `clinical`, `casebook`, `accessible` packs are **not implemented**. Raw Typst/audit-section checkboxes are stale and covered by v0.6 tests. Domain injection, domain defaults and style recommendation are missing. |
| `2026-10-28-visual-parity-and-production.md` | Phase 0–3; four unchecked acceptance items | 22-page parity, visual baselines, MCP smoke, one-call `prepare_guide` and checksum are now verified; checkboxes are stale. The new visual contract is not covered by this plan. |
| `2026-10-29-production-hardening-and-visuals.md` | Draft; G1–G23 | G1, G4–G16, G19 are largely landed. **G3 (domain prompts), G17 (style families), G21 (visual selection quality), G23 (caption/alt contract) remain open.** G18 (macOS/Windows/signing/CI) is external. New gaps: canonical TOC coverage, visual evidence manifests, underfilled pages, vector-figure counting. |
| `2026-10-29-d2-pipeline-and-opencode-automation.md` | D2-only pipeline released | Pipeline is implemented and tested, but **unused in PD2/PD3**. No `diagrams/` specs or manifests exist in either guide. |
| `2026-10-28-roadmap.md` | Sequence through v0.7 | Sequence delivered through v0.8.0; roadmap “current release v0.5.0” and some follow-on links are stale. Add this plan as the quality follow-on. |

## 6. Style analysis

### 6.1 What exists today

| Axis | Current values | Notes |
|---|---|---|
| `aesthetic` | `paper`, `clean` | paper stable, clean experimental |
| `structure` | `a-g`, `imrad`, `irac`, `soap`, `formula-sheet`, `literature-matrix`, `lecture-notes`, `executive-brief` | structure packs exist, but domains can make some unusable |
| `language` | `tr`, `en` | localized labels; no `de/fr`/RTL |
| `voice` | `academic`, `technical`, `plain`, `legal`, `clinical`, `business` | validated, not injected into prompt or rendering |
| `study_mode` | `exam`, `deep`, `reference`, `teach` | validated, not used |
| `citation` | `page-ref`, `apa`, `ieee`, `chicago`, `oscola` | page-ref only in practice; no citation formatter |
| `domain` | general, cs, math, humanities, business, medicine, law, language | profiles exist, but paper/clean support only some |

The practical result: **two visual skins, eight heading structures, and no
style recommendation**. `paper` and `clean` do not by themselves change the
narrative voice; the host writes the same A–G prose in both.

### 6.2 Current visual differences

| Dimension | `paper` | `clean` |
|---|---|---|
| Paper | warm cream `#FAF6EC` | white `#FFFFFF` |
| Body | Noto Serif 9.5 pt / 1.33 | Inter 9.5 pt / 1.38 |
| Headings | Inter | Inter |
| Accent | brick red / sage / gold | blue / teal / amber |
| Cover | double frame, motif | single rule, no motif |
| Density | comfortable | compact |
| Boxes | editorial fill + left rule | square technical boxes |
| Best for | long-form academic/exam reading | CS/technical documentation |
| Weakness | dense, low visual signal, small type | cold, still A–G, no code-first template |

### 6.3 Planned professional styles (not implemented)

These are the families promised by
`docs/plans/2026-10-28-style-language-profiles.md`. They are **not** available
today.

| Style | Aesthetic / typography | Layout | Narrative / structure | Best for |
|---|---|---|---|---|
| `journal` | serif body 10.5–11 pt, 1.35–1.45, 20–25 mm margins, restrained palette | single column or two-column optional; abstract, numbered figures/tables, footnotes | IMRAD / literature matrix; claims + evidence + method + discussion | humanities, social science, research |
| `brief` | sans or serif/sans mix 10.5–12 pt, 1.25–1.4, 18–22 mm margins | one-page/short, strong summary band, recommendation box, data tables | BLUF: context → findings → implications → recommendation → risks | business, policy, management |
| `clinical` | high-contrast sans/serif 10.5–11 pt, 1.35–1.45 | structured SOAP blocks, red-flag boxes, tables, wide margins for notes | SOAP; evidence, uncertainty, guideline years, no diagnosis without evidence | medicine, health sciences |
| `casebook` | serif 11–12 pt, 1.4–1.5, wide margins, numbered paragraphs | IRAC blocks, case tables, citation columns | Issue → Rule → Application → Conclusion; jurisdiction explicit | law |
| `accessible` | sans 12–14 pt, 1.5–1.6, high contrast, no colour-only meaning | single column, generous margins, large tables, alt text | plain language, short sentences, explicit structure | accessibility, low vision, print disabilities |
| `clean` | sans 9.5–10.5 pt, compact | code/table-first, square boxes | technical A–G / lecture notes | CS, engineering, math |
| `paper` | serif 9.5–10 pt, warm editorial | long-form A–G, boxes, tables | academic exam reader | general, CS theory, humanities |

These are starting points, not a substitute for a named style guide (APA, IEEE,
Chicago, OSCOLA, IMRAD, IRAC, SOAP). Each implemented pack must state which
conventions it follows and which it does not.

### 6.4 Does the LLM choose the style?

**Currently no.** The host can pass `style`, `structure`, `voice`, `domain`,
`study_mode` and `citation` to `prepare_guide`, but:

- `build_style` fills omitted axes with `paper`, `a-g`, `academic`, `exam`,
  `general`, `page-ref`;
- no candidate list or rationale is returned;
- no config field records why a style was chosen;
- `domain_profile.structure_default` is not applied;
- law/medicine/business combinations fail or require a non-obvious `clean`
  selection.

The plan is to make the choice explicit:

1. Add `style` action `recommend` (no new MCP tool; keep 13 tools).
2. Return 2–3 candidates with `why`, `tradeoffs`, `best_for`, `not_for` and a
   preview path.
3. `prepare_guide` includes `style_candidates` and `structure_candidates`.
4. The host must select one and write `style_rationale` (and optionally
   `style_source`) into the guide config/frontmatter.
5. Do not silently fall back to `paper` when the domain/style matrix is
   ambiguous; fail with the candidates and the exact compatibility error.
6. Apply `domain_profile.structure_default` only when the user did not choose a
   structure; warn when the domain default conflicts with an explicit choice.

## 7. New plan

### Workstream A — Visual coverage contract (P0)

**A1. Add a visual audit manifest.**

Per guide: `guides/<slug>/visual-audit.json` (hand-authored source of truth).

```json
{
  "schema_version": 1,
  "source_pages": [
    {
      "source": "PD2",
      "page": 5,
      "page_kind": "diagram",
      "decision": "redraw",
      "asset": "diagrams/language-genealogy.json",
      "reason": "language genealogy; high pedagogical value",
      "priority": "high"
    },
    {
      "source": "PD2",
      "page": 45,
      "page_kind": "photo",
      "decision": "decorative",
      "reason": "historical COBOL photo; no mechanism",
      "priority": "low"
    }
  ]
}
```

Allowed decisions: `redraw`, `table`, `text`, `decorative`, `unreadable`.
Every low-text page must have exactly one entry. `redraw` requires an existing
diagram/diagram spec; `table`/`text` requires a section reference; `unreadable`
requires a reason and must appear in G.

**A2. Seed and prioritize targets.**

`prepare_guide` writes/returns the manifest template for every low-text page,
with:

- source, page, page kind, nearby heading, text snippet, suggested kinds;
- `priority: high` for pages with many vector/line/rectangle objects, diagram
  captions, or a section with no existing figure;
- `priority: low` for cover/TOC/decorative photos/cartoons;
- do not include page 1/chapter covers as visual targets.

**A3. Enforce at build.**

`build_guide` reads `visual-audit.json` and fails when:

- a low-text page has no entry;
- a `high` priority page is `text`/`decorative` without a reason;
- a `redraw` entry has no existing asset or diagram manifest;
- a manual SVG has no exception record.

`build_guide` returns:

```json
{
  "visual": {
    "low_text_pages": 9,
    "redrawn": 3,
    "tables": 2,
    "text": 2,
    "decorative": 1,
    "unreadable": 1,
    "unresolved": []
  }
}
```

**A4. Add figure/diagram counts to the build/audit.**

`build_guide` parses guide Markdown and returns `figures`, `diagrams`, `tables`,
`boxes`, `figure_pages`. `audit` reports `raster_image_pages` separately from
`vector_figure_pages` (or `figure_captions`), so a vector-only guide is not
reported as “0 images”.

**A5. Convert PD2/PD3 visual targets.**

PD2:

- `language-genealogy` (p5): curated tree, max 12 nodes; split into 2 diagrams
  if needed.
- `lisp-lists` (p29): two representations, group/flow; explain the takeaway.
- `basic-lineage` (p53): Fortran IV/ALGOL 60 → BASIC → QuickBASIC/Visual BASIC.
- Optional: `cobol-photo` remains decorative; `cartoon` remains decorative.

PD3:

- `parse-tree` (p48–49): replace the manual SVG with a D2 tree spec.
- `operational-semantics` (p84): convert the manual SVG to D2 or register it as
  an annotated-source exception with a manifest.
- `attribute-flow` (p73–74): synthesized/inherited attribute flow.
- `syntax-graph` (p76–77): grammar/syntax-graph concept.
- `semantics-map` (p80/89): operational/axiomatic/denotational comparison.
- `factorial-state` (p128–130): only if it clarifies the mechanism; otherwise
  `text` with a reason.

**A6. Manual SVG policy.**

Manual SVG remains allowed only for an annotated source figure that cannot be
expressed as nodes/edges. It requires:

- `assets/<id>.json` with `title`, `desc`, `source`, `caption`, `alt`,
  `sha256`, `reason`;
- sanitizer pass; no script/external refs;
- a `visual-audit.json` exception entry;
- a PDF caption with takeaway and provenance.

D2-compatible content (trees, flows, concept maps) must use D2.

### Workstream B — Canonical outline and honest coverage (P0)

**B1. Extract the source topic list.**

Parse the first 4–6 pages (topic list/TOC) for numbered sections:
`1. …`, `2. …`, or `2.1 …`, including wrapped lines. Merge with heading
detection; use the source topic list as the canonical section set.

**B2. Add `canonical_sections` to inventory.**

```json
{
  "source": "PD2",
  "sections": [
    {"number": "2.1", "title": "…", "pages": "7–10", "origin": "toc"},
    {"number": "2.2", "title": "…", "pages": "11–14", "origin": "toc"}
  ]
}
```

`prepare_guide` uses this list for `outline` and for generated `covers` hints.
The host may add a manual section only with an explicit `origin: "manual"` and
a source page range.

**B3. Make coverage compare canonical vs claimed.**

`check_coverage`/`build_guide check=true` report:

```json
{
  "canonical_total": 20,
  "claimed": 20,
  "gate_total": 13,
  "missing_from_gate": ["2.2", "2.5", ...],
  "passed": true
}
```

It must fail if `claimed != canonical_total`, if a canonical section has no
claim, or if a claim is unknown. “13/13 PASS” is not acceptable when the
canonical total is 20.

**B4. Generate G counts from the canonical/visual manifests.**

G must not be free prose for counts. The host writes the explanation; the
counts come from `sections.json` + `visual-audit.json` + build stats. A
mismatch fails the build.

**B5. Tests.**

- PD2 fixture: topic list has 20 sections, heading extractor finds 13; gate
  fails until all 20 are claimed.
- PD3 fixture: topic list has 5, extractor finds 4; gate fails until 3.2 is
  claimed.
- Unknown/duplicate claims fail.
- A guide that claims 20/20 but only has 13 matched headings fails.

### Workstream C — Page density and chapter tails (P0)

**C1. Add `underfilled_pages` to the audit.**

For each page, record `body_chars`, `text_bottom`, `fill_ratio`, `page_kind`.
Flag a page when:

- it is not cover/TOC/chapter-opener;
- `body_chars < 0.35 × median_body_chars` or
  `text_bottom < 0.55 × page_height`.

Report `underfilled_pages` and `density_warnings` with page numbers and
fill ratios. `build --strict`/`audit --strict` fails; normal build warns.

**C2. Fix chapter tails.**

Options, in order:

1. Move the last section content so the tail page has enough material.
2. Use `flow=true` for compact C–G chapter openers when the previous page is
   underfilled.
3. Add a meaningful summary table/figure to the tail page.
4. Explicitly allow an underfilled page with a documented reason (rare).

Do not shrink type or add filler prose.

**C3. Tests.**

- PD2 p16/p20 and PD3 p17 reproduce as underfilled.
- A normal body page is not flagged.
- A chapter opener at the top of a page is not flagged.
- `flow=true` removes the tail without breaking TOC/bookmarks.

### Workstream D — Style selection and professional packs (P1)

**D1. Add style metadata.**

Extend `styles/<id>/style.json` with:

```json
{
  "best_for": ["humanities", "social science"],
  "not_for": ["clinical"],
  "narrative": "IMRAD / evidence-first",
  "typography": "serif 10.5–11 pt, 1.4",
  "layout": "single column; 22 mm margins",
  "differences": ["numbered citations", "abstract", "figure/table counters"]
}
```

**D2. Add `style` action `recommend`.**

Input: domain, structure, language, study_mode, source density, low-text count,
target pages, visual load. Output: 2–3 candidates with `why`, `tradeoffs`,
`best_for`, `not_for`, `preview`, `compatibility`.

No new MCP tool; the 13-tool budget stays.

**D3. Make `prepare_guide` return candidates and require a choice.**

- `style_candidates`, `structure_candidates`, `domain_defaults`.
- If `style` is omitted, do not silently choose paper when the domain is
  ambiguous; return candidates or fail with a clear message.
- If `structure` is omitted, apply `domain_profile.structure_default` only if
  valid for the chosen style; otherwise return candidates.
- Store `style_rationale` and `style_source` in the guide config/frontmatter.

**D4. Implement the missing packs.**

Priority order:

1. `journal` (humanities/research; IMRAD/literature matrix).
2. `brief` (business/policy; executive brief; BLUF).
3. `accessible` (all domains; 12 pt+, high contrast).
4. `clinical` (medicine; SOAP; red flags).
5. `casebook` (law; IRAC; case tables).

Each pack needs:

- `style.json`, `tokens.json`, `strings/tr.json`, `strings/en.json`;
- a layout family (grid, margins, type scale, headings, boxes, tables, figures,
  citations);
- a preview and visual baseline;
- domain/structure compatibility tests;
- a prompt fragment describing the narrative and typographic contract.

If a pack cannot be implemented in the release, remove it from the advertised
list rather than leaving a dead option.

**D5. Fix the compatibility matrix.**

- `paper`: general, cs, math, humanities.
- `clean`: general, cs, math, business.
- `journal`: humanities, general.
- `brief`: business, general.
- `clinical`: medicine.
- `casebook`: law.
- `accessible`: all.
- `language`: choose a default (clean or accessible) and document it.
- Every `structure` must have at least one valid style/domain pair, or the
  combination is rejected with an exact explanation.

**D6. Wire voice/study_mode/citation.**

- Inject a prompt fragment into `prepare_guide`/`write-study-guide` for
  `voice` and `study_mode`.
- Implement at least page-ref citations; if `apa/ieee/chicago/oscola` are not
  implemented, remove them from the accepted values or return a clear
  “not implemented” error. Do not claim citation formatting that does not
  exist.
- Add renderer/prompt tests for each voice/study mode.

### Workstream E — Prompt and production report (P0)

**E1. Update `write-study-guide`.**

Replace the conditional visual step with a mandatory workflow:

1. `prepare_guide` returns `visual_targets` and `visual-audit.json` template.
2. For every target, record `redraw`/`table`/`text`/`decorative`/`unreadable`.
3. For every high-priority target, create a D2 spec or give a source-grounded
   reason why not.
4. Run `diagram validate`/`save`, shell `diagram build`, `diagram status`.
5. `build_guide check=true` must pass visual, canonical coverage and density
   gates.
6. Run `audit --strict`.
7. Report the counts from the manifests, not from memory.

Also require a scope statement: one guide with multiple sources or one guide
per source.

**E2. Add a production report.**

`build_guide` returns:

```json
{
  "scope": "single-guide | per-source",
  "sources": 2,
  "canonical_sections": 25,
  "claimed_sections": 25,
  "gate_sections": 25,
  "figures": 5,
  "diagrams": 4,
  "tables": 12,
  "underfilled_pages": [],
  "style": {"id": "paper", "rationale": "..."},
  "visual": {"low_text_pages": 27, "unresolved": []}
}
```

The host must include this report in its completion message; a prose “done” is
not sufficient.

**E3. Update MCP schema.**

Adding response fields/actions is a MINOR change. Bump
`MCP_TOOL_SCHEMA_VERSION` to 4 and SemVer to `v0.9.0`; keep 13 tools.

### Workstream F — Tests, docs and release

**F1. Fixtures.**

- `visual-audit-missing.json` — missing low-text page.
- `visual-audit-unresolved.json` — high-priority page marked `text` with no
  reason.
- `coverage-toc-missing.md` — canonical 20, claimed 13.
- `underfilled-page.pdf` — chapter tail at 10% fill.
- `manual-svg-no-manifest.svg` — manual SVG without exception.
- `style-matrix-law.md` — law/medicine domain must fail or recommend a valid
  pack.
- `style-recommend.md` — candidates returned for cs/humanities/business.

**F2. Regression.**

- Rebuild PD2/PD3 with the new gates.
- Require canonical coverage 20/20 and 5/5.
- Require no unresolved visual targets.
- Require no underfilled pages (or an explicit reason).
- Require at least the planned PD2/PD3 diagrams.
- Keep 22/19 page targets; if visuals change page count, update the target only
  after review.

**F3. Docs.**

- Update `docs/authoring-spec.md` with the visual manifest and canonical
  coverage.
- Update `docs/authoring.md` with the style recommendation flow.
- Update `docs/design-system.md` with the style comparison table and
  accessibility targets.
- Update `docs/agents/opencode-guide.md` with mandatory visual steps and the
  production report.
- Reconcile stale checkboxes in the old plans.
- Add this plan to the roadmap as the quality follow-on.

**F4. Release.**

- `v0.9.0`: style recommendation, visual manifest, canonical coverage,
  density gate, figure stats, prompt/report changes.
- `v0.10.0`: professional packs that do not fit v0.9.0 (clinical/casebook or
  additional languages).
- `v1.0.0`: only after the PD2/PD3 regression, cross-style parity, clean
  install, and audit gates pass.

## 8. Acceptance criteria

### Visual

- [ ] PD2 has at least the genealogy, LISP-list and BASIC-lineage diagrams, or
      a documented source-grounded reason for each omission.
- [ ] PD3 has a D2 parse tree, an attribute-flow diagram, a syntax-graph
      diagram and a semantics comparison, or documented reasons.
- [ ] Every low-text page has exactly one `visual-audit.json` decision.
- [ ] No high-priority visual target is unresolved.
- [ ] Manual SVGs have a manifest/exception record and title/desc/alt/source.
- [ ] `build_guide` reports figures/diagrams/tables and `audit` distinguishes
      raster images from vector figures.

### Coverage

- [ ] PD2 canonical total = 20; claimed = 20; gate = 20.
- [ ] PD3 canonical total = 5; claimed = 5; gate = 5.
- [ ] A guide cannot pass with `gate_total < canonical_total`.
- [ ] G counts are generated from manifests, not hand-written prose.

### Layout

- [ ] PD2 p16/p20 and PD3 p17 are either fixed or explicitly justified.
- [ ] `underfilled_pages` is reported by audit and fails in strict mode.
- [ ] TOC page numbers and bookmarks still match after reflow.

### Style

- [ ] `style recommend` returns 2–3 valid candidates with rationale.
- [ ] `prepare_guide` does not silently default when the domain is ambiguous.
- [ ] law/medicine/business have at least one valid style/domain/structure
      combination.
- [ ] `paper`, `clean` and at least `journal`/`brief`/`accessible` have
      previews and compatibility tests.
- [ ] `style_rationale` is recorded in the guide.
- [ ] `voice`/`study_mode`/`citation` are either implemented or rejected with
      a clear “not implemented” error.

### Production

- [ ] The OpenCode run reports canonical coverage, visual decisions, style
      rationale, diagram status, density and audit.
- [ ] No “20/20” or “136/136” claim without machine evidence.
- [ ] 107+ tests, clippy, fmt, 22-page parity, visual baselines and package
      checksum pass.
- [ ] A clean install builds PD2/PD3 from the archive.

## 9. Out of scope / external blockers

- macOS/Windows archives and signing require runners/toolchains and key
  material; do not fake them.
- GitHub Actions is blocked by the repository account billing/spending limit.
- RTL languages remain deferred; document the font/bidi/table test plan.
- A fully automatic factual fact-checker is out of scope; human review remains
  mandatory for claims, page refs and table cells.
- A hosted multi-user service is out of scope; the MCP remains local/workspace
  scoped.

## 10. Open questions

1. Should the default product be one guide per source or one guide per course?
   The plan should make the default explicit and require the host to state it.
2. Which style is the default for CS: `paper`, `clean`, or a new `technical`
   pack? Do not default silently.
3. Should `journal` be single-column first (safer) or two-column optional
   (more authentic)?
4. Are `clinical` and `casebook` in v0.9.0, or v0.10.0? They need domain
   review, not just layout work.
5. Should manual SVG be allowed at all after D2, or only as a registered
   exception with a human review gate?
6. Which citation styles are actually required? If only page-ref is
   implemented, remove the others from the accepted values.
