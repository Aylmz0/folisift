# Folisift v0.9 — Guide Quality, Visual Coverage and Style Selection

- **Status:** P0/P1 slice implemented (canonical coverage, visual audit,
  underfilled-page audit, reader and professional packs, TOC links, adaptive
  ranges); production release/run and G generation remain open
- **Date:** 2026-10-29 (implementation update: 2026-10-30)
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
  `docs/authoring-spec.md`, `docs/authoring-evidence.md`,
  `docs/design-system.md`, `docs/agents/opencode-guide.md`,
  ADR-0004, ADR-0007, ADR-0010

## 0. Implementation status (2026-10-30)

Landed in the working tree:

- `SourceInventory.sections`: canonical sections from the source topic list;
  PD2 20/20, PD3 5/5, PD0–PD1 13/13.
- `visual-audit.json` + build gate for every low-text page; manifests exist for
  `pd0-pd1`, `pd2`, `pd3`.
- TOC rows and page numbers are real internal PDF links.
- `reader` style: 10.0 pt body, 0.70 em leading, 17 mm margins, 2.6 mm
  paragraph spacing; PD2/PD3 use it instead of the tighter paper default.
- MCP `style recommend` returns compatible candidates; `journal`, `brief` and
  `accessible` packs exist; lineage/rationale is recorded for pd0-pd1/pd2/pd3.
- `LayoutAudit.underfilled_pages`: PD2 and PD3 were reflowed to zero
  underfilled pages; PD0–PD1 is still clean.
- PD2 D2 diagrams: `dil-soyagaci`, `lisp-listeleri`, `basic-seceresi`.
- PD3 D2 diagrams: `ayristirma-agaci`, `ozellik-akisi`, `sozdizim-grafigi`,
  `anlam-yontemleri`, `islemel-semantik`.
- Manual `parse-tree.svg` and `operational.svg` removed; outputs rebuilt.
- `build --check` and `audit --strict` pass for PD2 (23 pages), PD3 (20 pages)
  and PD0–PD1 (22 pages). No underfilled pages remain.

Still open: specialized `clinical`/`casebook` packs, generated G counts,
short/medium/long genre fixtures, the final OpenCode production report and a
release tag. Core P0/P1 quality gates are in place.

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

### 4.1 The summarizer's admission and proposed patch

The summarizer confirmed the core finding: D2 was installed and working; it
chose a shortcut and hand-wrote `parse-tree.svg` and `operational.svg`. That
admission is correct and useful. Its proposed patch is a good seed but is not
yet a correct or complete plan.

What it gets right:

- It admits the D2 contract violation instead of blaming the engine.
- It keeps the cover motif as a design asset, not a content diagram.
- It proposes D2 specs for the parse tree and operational semantics.
- It proposes a verification loop (`diagram build`, `diagram status`,
  `build --check`, `audit`).

What is wrong or incomplete:

1. **PD2 needs more than one diagram.** The proposed `dil-soyagaci` covers only
   the ALGOL branch. The source also has the LISP list representation (p29) and
   the BASIC lineage (p53). Those are exactly the visual mechanisms the guide
   currently explains only in prose.
2. **The proposed `dil-soyagaci` cannot be a `tree`.** A test of the proposed
   10-node/10-edge spec against the current validator fails:
   `tree depth 5 exceeds 4; split it`. The genealogy is also a DAG (C++ has
   both C and Simula 67 parents), not a strict tree.
3. **The proposed parse tree cannot be a `tree` either.** A test of the
   `X := Y + Z` tree fails: `tree depth 6 exceeds 4; split it`. A parse tree is
   naturally deeper than the current tree limit.
4. **`ozellik-akisi` is not a tree.** Attributes flow in both directions
   (`expected_type` down, `actual_type` up). A tree with two semantic flows
   should use `concept`/`flow` with labeled edges, or a dedicated attribute
   view.
5. **The patch is diagram-only.** It does not fix canonical coverage, the
   visual-audit manifest, underfilled pages, style selection, G evidence, or
   the false “20/20” claim. Adding four diagrams to a guide that still reports
   13/20 coverage is a cosmetic fix.
6. **It does not answer the important questions.** It says why it skipped D2,
   but not whether it actually inspected the low-text pages, why it created two
   guides, why it chose paper/A–G, or why it accepted PD2 pages 16/20 as
   finished.

The correct sequence is: fix the visual/coverage contracts first, then write
the diagrams. Otherwise the next run can add four diagrams and still pass with
unverified coverage and no visual evidence for the remaining pages.

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
| `aesthetic` | `paper`, `reader`, `clean` | paper stable, reader spacious/experimental, clean technical/experimental |
| `structure` | `a-g`, `imrad`, `irac`, `soap`, `formula-sheet`, `literature-matrix`, `lecture-notes`, `executive-brief` | structure packs exist, but domains can make some unusable |
| `language` | `tr`, `en` | localized labels; no `de/fr`/RTL |
| `voice` | `academic`, `technical`, `plain`, `legal`, `clinical`, `business` | validated, not injected into prompt or rendering |
| `study_mode` | `exam`, `deep`, `reference`, `teach` | validated, not used |
| `citation` | `page-ref` | only implemented value; unimplemented styles are rejected |
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

### 6.5 Evidence basis and dynamic adaptation

The normative evidence base is `docs/authoring-evidence.md`. It is deliberately
not a claim that Folisift is “scientifically proven”. It maps principles to
guide components and states the limits.

Key conclusions that must change the product:

- **Retrieval practice and distributed practice are the strongest general
  techniques** in the Dunlosky et al. review. A guide that only condenses text
  is not an optimal study system. Add optional recall prompts and a short review
  schedule.
- **Summarization is rated low utility as a standalone study technique** in that
  review. It can support comprehension when taught with macrorules, but it must
  not be the only learning mechanism. The guide needs examples, comparisons,
  diagrams, self-explanation prompts and retrieval.
- **Self-explanation and interleaving are moderate-utility** techniques. Add
  content-specific “why/how/what-if” prompts and mixed comparison examples,
  not generic study advice.
- **Note-taking has a small encoding effect; review matters more.** The guide
  is an external-storage artifact, so it must be reviewable: clear structure,
  cue boxes, concise summaries, retrieval prompts.
- **Multimedia principles are not “more pictures”.** Coherence, signaling,
  segmenting, spatial contiguity and dual coding support meaningful diagrams;
  decorative visuals add load.
- **Accessibility is a floor.** WCAG 2.2/PDF-UA checks, 12 pt+ for accessible
  styles, contrast, alt text and tagged PDF are required.

The system must adapt to the source, learner and requested output:

```text
source profile  →  learner profile  →  output profile  →  decision rules
     │                    │                   │                  │
  type, units,       study mode,        structure, style,     page_range,
  density, visual    prior knowledge,   visual budget,        component mix,
  load, text layer   time, access       citation              retrieval mode
```

Rules:

- Replace fixed `target_pages` with a computed `page_range`; length is an output
  of canonical sections, concept density, examples, tables and high-value
  visuals, not a target to pad.
- Short outputs use `executive-brief`/`formula-sheet`/`lecture-notes`; long
  outputs use A–G or volume sets. Do not force a 22-page template onto every
  PDF.
- If the source is small, the guide is shorter. If it is large, split volumes
  or switch to reference mode. Never shrink type below the accessibility floor.
- The host LLM may override a default, but must record `style_rationale`,
  `structure_rationale` and the source/learner profile used.
- Add short/medium/long regression fixtures so the engine proves it can adapt,
  not just produce one 22-page shape.

These heuristics are project hypotheses. Calibrate them with expert review and
the user evaluation in `docs/authoring-evidence.md`; do not present them as
universal scientific constants.

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

- `language-genealogy` (p5): `concept`/`flow`, not `tree`; the proposed 10-node
  tree is depth 5 and has multiple parents. Split into an ALGOL branch and a
  wider family map if needed.
- `lisp-lists` (p29): two representations, `concept`/`flow`; explain the
  takeaway.
- `basic-lineage` (p53): `concept`, because Fortran IV and ALGOL 60 are two
  roots; a `tree` would reject it.
- Optional: `cobol-photo` remains decorative; `cartoon` remains decorative.

PD3:

- `parse-tree` (p48–49): replace the manual SVG with a `concept`/layered flow,
  or a dedicated `parse-tree` kind with an explicit depth allowance. The
  proposed `tree` is depth 6 and fails the current limit.
- `operational-semantics` (p84): convert the manual SVG to a `flow` with
  real-world/abstract-machine lanes.
- `attribute-flow` (p73–74): `concept`/`flow` with `inherited ↓` and
  `synthesized ↑` labels; not a tree.
- `syntax-graph` (p76–77): grammar/syntax-graph `concept`.
- `semantics-map` (p80/89): operational/axiomatic/denotational comparison as
  `concept`.
- `factorial-state` (p128–130): only if it clarifies the mechanism; otherwise
  `text` with a reason.

**A5.1 Diagram design review.**

The following table resolves the summarizer's proposed patch against the
current D2 contract.

| Proposed | Verdict | Required design |
|---|---|---|
| PD2 `dil-soyagaci` (`tree`, 10 nodes) | **invalid as written**: depth 5 > 4; DAG with multiple parents | use `concept`/`flow`, or split into an ALGOL branch and a C-family branch; include LISP/COBOL/APL/Prolog families or explicitly label it “selected ALGOL branch” |
| PD2 LISP list representation (p29) | missing | add `lisp-lists` as `concept`/`flow`; show both representations and the takeaway |
| PD2 BASIC lineage (p53) | missing | add `basic-lineage` as `concept` (two roots Fortran IV and ALGOL 60; `tree` would reject two roots) |
| PD3 `ayristirma-agaci` (`tree`) | **invalid as written**: depth 6 > 4 | use `concept`/layered flow, or add a dedicated `parse-tree` kind with an explicit depth allowance; do not silently raise the global tree limit |
| PD3 `ozellik-akisi` (`tree`) | wrong shape: bidirectional attribute flow | use `concept`/`flow` with `inherited ↓` / `synthesized ↑` edge labels; keep source refs |
| PD3 `islemel-semantik` (`flow`) | valid in principle | use two lanes/groups: real world and abstract machine; keep the six-step mapping |
| PD3 syntax graph (p76–77) | missing | add `syntax-graph` as `concept`/`flow`, or document why a text explanation is sufficient |
| PD3 semantics comparison (p80/89) | missing | add `semantics-map` as `concept`; operational/axiomatic/denotational tradeoffs |
| PD3 factorial state (p128–130) | optional | `text` with a source-grounded reason unless it clarifies the mechanism |

Smoke validation result: the same genealogy is accepted as `concept` and `flow`;
the parse tree is accepted as `concept`. Only the `tree` kind fails on depth.
This is a design choice to make explicit, not a reason to abandon D2.

**A6. Manual SVG policy.**

Manual SVG remains allowed only for an annotated source figure that cannot be
expressed as nodes/edges. It requires:

- `assets/<id>.json` with `title`, `desc`, `source`, `caption`, `alt`,
  `sha256`, `reason`;
- sanitizer pass; no script/external refs;
- a `visual-audit.json` exception entry;
- a PDF caption with takeaway and provenance.

D2-compatible content (trees, flows, concept maps) must use D2.

**A7. Tree-depth and DAG policy.**

Do not raise `MAX_DEPTH` blindly. Decide one of:

1. Keep `tree` at depth 4; use `concept`/`flow` for parse trees and DAGs.
2. Add a dedicated `parse-tree` kind with an explicit max depth of 8 and tests.
3. Add an explicit `max_depth` override to the spec, defaulting to 4, with
   validation that the chosen depth still fits the page.

Whichever option is chosen must be documented in `docs/authoring.md`, the
diagram schema resource and the D2 tests. `tree` must reject multiple parents;
the validator currently checks roots and depth, not DAG shape.

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

**E4. Do not accept a diagram-only handoff.**

The summarizer's proposed patch is a diagram-only change. Reject it as the
completion of this plan unless the visual manifest, canonical coverage,
underfilled-page gate and style decision are also addressed. A guide with four
new diagrams and an unverified 13/20 coverage is not production-ready.

### Workstream F — Evidence-based learning design and dynamic adaptation (P0/P1)

**F1. Make the evidence base normative.**

`docs/authoring-evidence.md` is the reference. Update
`docs/authoring-spec.md` to map every guide component to its evidence basis and
to state the limits (summarization is not a standalone high-utility technique;
visuals must be meaningful; note review matters).

**F2. Add source, learner and output profiles.**

`prepare_guide` returns:

```json
{
  "source_profile": {
    "source_type": "slides",
    "units": {"pages": 103, "words": 6448, "sections": 20},
    "low_text_pages": 9,
    "visual_load": "medium",
    "text_layer": "native",
    "structure_clarity": "explicit"
  },
  "learner_profile": {
    "study_mode": "exam",
    "prior_knowledge": "intermediate",
    "time_budget": "standard",
    "accessibility": "standard"
  },
  "output_profile": {
    "structure": "a-g",
    "style": "paper",
    "page_range": [20, 26],
    "visual_budget": "high-priority targets mandatory",
    "component_mix": {
      "worked_examples": "high",
      "diagrams": "high",
      "recall_prompts": "optional"
    }
  }
}
```

The host must confirm or override the output profile and record the rationale.

Source-adaptation rules:

- `text_layer=none`: every page is a visual target. Coverage cannot pass from
  absent text; the host must render/vision-read pages, record the disposition
  and list unreadable pages in G.
- `structure_clarity=none`: build a canonical outline from page ranges or an
  explicit manual map; require `origin: "manual"` and human review. Never invent
  section numbers that the source does not support.
- `source_type=scanned`: use OCR/vision as a recorded method; do not claim text
  extraction.
- `domain_hint` is a hint, not a confident auto-classifier. Default to
  `general` when uncertain and ask the user/host; do not silently force a
  medical, legal or business structure.
- `units` may be slides, pages, sections or code files; do not assume every PDF
  is a slide deck.
- Very large sources split into volumes; very small sources use a short genre
  instead of padding.
- Respect expertise reversal: novices get definitions, worked examples and
  scaffolding; advanced learners get synthesis, edge cases and less redundancy.

**F3. Compute length; do not hard-code it.**

- Replace `target_pages` with `page_range` in new configs; keep reading old
  `target_pages` as an alias.
- Compute the range from canonical sections, source density, examples, tables
  and high-value visuals.
- Allocate section ranges; sum plus front/back matter.
- Fail on padding (underfilled pages) and on omission (coverage), not on a
  fixed page count.
- Add short/medium/long fixtures:
  - `brief`: 4–8 pages, executive structure;
  - `lecture-notes`: 8–16 pages;
  - `course-guide`: 18–30 pages for two sources;
  - `multi-source`: 60–100+ pages or volumes for a full course.
- Keep heuristics in `docs/authoring-evidence.md`; calibrate with evaluation.

**F4. Add evidence-based components.**

Optional, controlled by `study_mode`/`audience`:

- `::: recall` — one specific retrieval question with answer separated
  (retrieval practice).
- `::: review-plan` — short spaced-review schedule (distributed practice).
- `::: compare` / mixed examples — interleaving prompts.
- `::: explain` — “why/how/what-if” self-explanation prompt.
- Worked examples with faded steps for procedural domains.

Do not add generic “study tips”, motivational prose or mandatory quizzes. The
default remains an exam-oriented guide; recall/review components are opt-in or
style/study-mode driven.

**F5. Academic/professional genre mapping.**

- `journal`: IMRAD/literature matrix; APA/IEEE/Chicago style only if
  implemented and metadata exists.
- `brief`: BLUF/executive brief; no academic citation theatre.
- `clinical`: SOAP; evidence/uncertainty/guideline years.
- `casebook`: IRAC; jurisdiction and citations.
- `paper`/`clean`: course reader/technical reader; page-ref citations.
- Do not force IMRAD onto a formula sheet or IRAC onto a clinical note.
- Remove unimplemented citation values (`apa`, `ieee`, `chicago`, `oscola`)
  from accepted config unless a real formatter exists.

**F6. Evaluation and claims.**

- Add an expert rubric and a small user-study protocol to
  `docs/authoring-evidence.md`.
- Do not claim “proven” without a product evaluation.
- Track at least: coverage, visual disposition, underfilled pages, readability,
  time-to-answer and delayed recall in a pilot.
- Use the results to calibrate page ranges, visual density and retrieval
  components.

### Workstream G — Tests, docs and release

**G1. Fixtures.**

- `visual-audit-missing.json` — missing low-text page.
- `visual-audit-unresolved.json` — high-priority page marked `text` with no
  reason.
- `coverage-toc-missing.md` — canonical 20, claimed 13.
- `underfilled-page.pdf` — chapter tail at 10% fill.
- `manual-svg-no-manifest.svg` — manual SVG without exception.
- `style-matrix-law.md` — law/medicine domain must fail or recommend a valid
  pack.
- `style-recommend.md` — candidates returned for cs/humanities/business.

**G2. Regression.**

- Rebuild PD2/PD3 with the new gates.
- Require canonical coverage 20/20 and 5/5.
- Require no unresolved visual targets.
- Require no underfilled pages (or an explicit reason).
- Require at least the planned PD2/PD3 diagrams.
- Treat the old 22/19 page counts as observations, not targets. Recompute the
  page range from the new content budget; review any change.
- Add short/medium/long fixtures so the engine is not tuned to one page count.

**G3. Docs.**

- Update `docs/authoring-spec.md` with the visual manifest, canonical coverage
  and evidence-based component rules.
- Update `docs/authoring.md` with the style recommendation, source/learner
  profile and page-range flow.
- Update `docs/design-system.md` with the style comparison table and
  accessibility targets.
- Update `docs/agents/opencode-guide.md` with mandatory visual steps and the
  production report.
- Link `docs/authoring-evidence.md` as the normative evidence reference.
- Reconcile stale checkboxes in the old plans.
- Add this plan to the roadmap as the quality follow-on.

**G4. Release.**

- `v0.9.0`: style recommendation, visual manifest, canonical coverage,
  density gate, figure stats, source/learner/output profiles and
  prompt/report changes.
- `v0.10.0`: professional packs that do not fit v0.9.0 (clinical/casebook or
  additional languages).
- `v1.0.0`: only after the PD2/PD3 regression, cross-style parity, clean
  install, audit gates, the expert rubric and at least a small user evaluation
  are complete.

## 8. Acceptance criteria

### Visual

- [x] PD2 has at least the genealogy, LISP-list and BASIC-lineage diagrams, or
      a documented source-grounded reason for each omission.
- [x] PD3 has a parse tree, an attribute-flow diagram, a syntax-graph diagram,
      a semantics comparison and an operational-semantics diagram, or
      documented reasons.
- [x] Every `diagram validate` call passes; no proposed diagram fails on tree
      depth, multiple roots or multiple parents.
- [x] `diagram status` reports every spec `ok` with a matching manifest/SVG.
- [x] The manual `parse-tree.svg` and `operational.svg` are either removed or
      registered as explicit exceptions with a manifest.
- [x] Every low-text page has exactly one `visual-audit.json` decision.
- [x] No high-priority visual target is unresolved.
- [x] No manual content SVGs remain in PD2/PD3; a future manual SVG
      exception policy is still open (A6).
- [x] `build_guide` reports figures/diagrams/tables.
- [x] `audit` reports raster `image_pages` separately from
      `figure_captions`/`figure_caption_pages`.

### Coverage

- [x] PD2 canonical total = 20; claimed = 20; gate = 20.
- [x] PD3 canonical total = 5; claimed = 5; gate = 5.
- [x] A guide cannot pass with `gate_total < canonical_total`.
- [ ] G counts are generated from manifests, not hand-written prose.

### Layout

- [x] Underfilled body pages are fixed in PD2 and PD3; no underfilled pages remain.
- [x] `underfilled_pages` is reported by audit and fails in strict mode.
- [x] TOC page numbers and links are machine-checked by
      `scripts/check-toc-links.py` (42/54/24 links, page numbers match).

### Style

- [x] `style recommend` returns compatible candidates with rationale.
- [x] `prepare_guide` reports `style_source` (`explicit` or `default-reader`);
      it no longer fails law/medicine due to a paper-only default.
- [x] law/medicine/business have at least one valid style/domain/structure
      combination (reader).
- [x] `paper`, `reader` and `clean` have previews and compatibility tests.
- [x] Specialized `journal`, `brief` and `accessible` packs exist with
      previews and compatibility tests.
- [x] `style_rationale` is recorded in pd0-pd1/pd2/pd3 configs and returned by
      `build_guide`; future guides should set it.
- [x] `citation` rejects unimplemented styles; `voice`/`study_mode`
      instructions are returned as a prompt fragment.

### Evidence and adaptation

- [x] `docs/authoring-evidence.md` is linked from the authoring spec and the
      plan; every guide component has an evidence basis or is labelled as a
      design convention.
- [x] The plan does not claim “scientifically proven”; product claims are tied
      to the evaluation protocol.
- [x] `prepare_guide` returns source, learner and output profiles (`profiles`).
- [x] `prepare_guide` returns a computed `suggested_page_range`; config
      `target_pages` remains a readable range override, not a fixed target.
- [ ] Short, medium and long fixtures pass: brief, lecture-notes, two-source
      course guide and multi-source volume plan.
- [x] `::: recall`, `::: review-plan` and `::: explain` components exist as
      optional prompts; no generic filler is injected by default.
- [x] Summarization is not presented as a standalone high-utility technique;
      the prompt pairs it with retrieval, examples and self-explanation.
- [x] The style/genre matrix rejects incompatible domain/structure pairs and
      keeps `reader` as the generic fallback.
- [x] An expert rubric and a small user-evaluation protocol are documented.
- [x] No page count is treated as a quality target; underfilled pages and
      coverage are the gates.

### Production

- [ ] The OpenCode run reports source/learner/output profiles, canonical
      coverage, visual decisions, style rationale, diagram status, density and
      audit.
- [x] No “20/20” or “136/136” claim without machine evidence; build reports
      canonical/visual counts.
- [x] 123 tests, clippy, fmt, 22-page reference parity, visual baselines and
      package checksum pass. Dynamic guides may legitimately have different
      page ranges; parity applies to the reference guide.
- [x] A clean install builds PD2/PD3 from the archive.

## 9. Questions for the summarizing LLM

The admission already answers the most important question (“I skipped D2”). Do
not ask an open-ended “why?”; ask for evidence and decisions that change the
plan. A short interview is enough:

1. **Visual inspection evidence.** For each low-text page, did you call
   `render_pages`? If yes, give the output path/hash and the page you inspected.
   If no, why does G say “görsel olarak incelendi”?
2. **`visual_targets`.** Did `prepare_guide` return the targets? Which ones did
   you consider, and why did you mark the rest as text/decorative?
3. **Scope.** Why did you create `pd2` and `pd3` separately instead of one
   guide with two B parts? What did the user ask for?
4. **Style.** Did you consider `clean`, `imrad`, `lecture-notes` or another
   structure? Why `paper`/`a-g`/`academic`? Did you know law/medicine domains
   would fail?
5. **D2 friction.** Which part of `diagram validate → save → shell build →
   status` made the shortcut attractive? What would have made D2 the easier
   path?
6. **Final PDF review.** Did you render/inspect PD2 pages 16 and 20 and PD3
   page 17? If yes, why were the near-empty pages accepted? If no, why did you
   declare the PDF done?
7. **Audit interpretation.** Did you see `image_pages: 0`? What did you think
   it meant for the two SVGs?
8. **Coverage interpretation.** Did you know the gate saw 13/20 and 4/5, while
   G claimed 20/20 and 5/5? If yes, why was that acceptable?

Decision rule: answers 1–3 are **P0 evidence** and must be resolved before the
diagram patch. Answers 4–8 feed the style, prompt and audit workstreams. Do not
accept “the model decided” as a root cause; identify the missing gate or
incentive.

## 10. Out of scope / external blockers

- macOS/Windows archives and signing require runners/toolchains and key
  material; do not fake them.
- GitHub Actions is blocked by the repository account billing/spending limit.
- RTL languages remain deferred; document the font/bidi/table test plan.
- A fully automatic factual fact-checker is out of scope; human review remains
  mandatory for claims, page refs and table cells.
- A hosted multi-user service is out of scope; the MCP remains local/workspace
  scoped.

## 11. Open questions

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
7. Which evaluation is feasible in the next release: expert rubric only, or a
   small pre/post recall study with target students?
8. Which accessibility target is the floor for `paper`/`clean`: 9.5 pt body
   with optional large-print, or 10–11 pt body by default?
