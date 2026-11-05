# Source visual pipeline — detailed plan

- **Status:** Completed — v0.22.0, MCP schema 17, 41 tools; A-V5 green
- **Date:** 2026-11-05
- **Target:** Folisift 0.22.0, MCP tool schema 17, 41 tools
- **Builds on:** [`2026-11-04-visual-evidence-and-chart-policy.md`](2026-11-04-visual-evidence-and-chart-policy.md)
  (A+B policy) and the completed v3 remediation plan
- **Calibration:** [`2026-11-05-source-visual-calibration.md`](2026-11-05-source-visual-calibration.md)
- **Research basis:** [`../learning-science-and-genre-review-2026-11-02.md`](../learning-science-and-genre-review-2026-11-02.md#6-visual-learning-evidence-2026-11-05-update)
- **Purpose:** make source visuals (figures, charts, image-based tables) first-class,
  reproducible guide assets instead of a manual crop escape hatch
- **One-sentence goal:** every material visual in a source can be discovered,
  classified, ranked, extracted at the best available fidelity, embedded with
  provenance, and validated by a gate — without decorative spam, without a hard
  quota, and without forcing the host LLM to look at every image.

## 0. Implementation notes

The implementation follows this plan with the following concrete choices:

- Candidate discovery also includes a **vector-cluster fallback** and rejects
  page-spanning figure/table blocks as containers. A page with many fragmented
  figure pieces (PD3 page 76) collapses into one group candidate with
  `group_members`.
- `visual_decision` and `visual_assets` are the one-call workflow. The
  low-level `crop-source` command remains available for a manual normalized
  crop; candidate-based cropping is handled by `visual_decision`.
- `visual_brief` remains the page-level brief; `visual_candidates` is the
  per-visual report. This avoids parsing the source twice in the same call.
- `review_mode` currently emits `text-only` and
  `visual-check-recommended`; `visual-check-required` is reserved for a future
  confidence upgrade.
- The optional Model2Vec semantic-relevance feature and the `semantic` flag are
  **deferred**; the core filter is lexical/structural and deterministic.
- The real-crop build acceptance is covered by the existing A-V3 crop build
  (`check-feedback-v3.py`), which builds a smoke guide with `embed_crop` and
  `visual_gate=required`.
- Calibration is provisional, not a claim of perfect pedagogical importance:
  the top candidates for the calibration pages were rendered and inspected;
  the report records the observed ranking and known gaps.

## 0.1 Plain-language answer to the originating question

Today the direct-use path exists but is only a **manual, page-level escape
hatch**:

- `crop_source` renders one page, crops a normalized box, writes a hash-named
  PNG under `guides/<slug>/data/crops/`, and returns `local_only: true`.
- `::: figure src=… caption=… source=… alt=…` embeds it with required
  provenance.
- `visual-audit` `embed_crop` validates the asset path, hash, crop box,
  `local_only`, caption and alt text.
- A-V3-06/07/08 prove the path end to end on a smoke guide.

What does **not** exist:

- a candidate list with bounding boxes;
- any notion of “this visual is worth reusing”;
- native image extraction (the original embedded pixels);
- candidate-based cropping (the host still types fractions by hand);
- deduplication of logos/backgrounds/repeated images;
- per-candidate audit decisions;
- blank/low-resolution quality checks;
- a decision on “D2 can rebuild this” versus “this is unique and must be
  embedded/described”;
- one-call materialization and a reproducible regeneration recipe;
- any real guide that embeds a source crop.

The result: the mechanism is correct but the host must find every visual by
reading pages and guessing crop fractions. The plan below closes that gap.

## 1. Current evidence (what the code actually does)

| Capability | Current state | Evidence |
|---|---|---|
| Page-level visual evidence | `figures`, `image_refs`, `vector_graphics`, `table_likely`, `chart_likely` per page | `crates/folisift-parse/src/insights.rs` |
| Per-page brief | `evidence_kind`, `role`, recommendation, existing decision | `crates/folisift-mcp/src/lib.rs` (`visual_brief`) |
| Source crop | normalized crop, 200 DPI default, hash-named PNG, local-only | `crop_source`, `render.rs` |
| Figure embed | required src/caption/alt/source; tagged PDF figure | `markdown.rs` |
| Audit gate | `embed_crop` requires asset, crop, hash, `local_only`, caption, alt, reason | `visual_audit.rs` |
| Figure-page gate | `visual_gate = off｜warn｜required`; default `warn` | `folisift-build/src/lib.rs` |
| Native image extraction | available in liteparse 2.17 (`ParseResult.images`) but **not used** | `liteparse` dependency |
| Guide usage | **zero** `embed_crop` entries; one orphan crop file | `guides/*/visual-audit.json` |

Important limitation: the MCP server and the host may not be able to see
pixels. Any “importance” signal must come from PDF structure, text, geometry
and image metadata — not from visual semantics. The plan is explicit about
that boundary.

## 2. Manual calibration set v0 (inspected, not guessed)

On 2026-11-05 `verimadenciliği` pages 14/45/71, `PD3` page 76 and `PD2`
page 53 were rendered at 140 DPI and inspected visually. The remaining
PD2/PD3 labels were cross-checked against the existing D2 assets and
`visual-audit.json` reasons. The labels below are the ground truth for scorer
calibration and for the reconstruction classifier. They are deliberately stored
before the scorer exists, so the scorer cannot “define its own truth”.

| Source:page | Region | Content | Material | Reconstruction | Expected action | Quality |
|---|---|---|---|---|---|---|
| `verimadenciliği:14` | slide 13 | text bullets | 0 | text | skip | — |
| `verimadenciliği:14` | slide 14 | formulas + text | 0 | text | skip | — |
| `verimadenciliği:14` | slide 15 | three skewness curves (`mean/median/mode`) | 1 | `unique` (smooth curves) | `describe` or `embed_crop` | low |
| `verimadenciliği:14` | slide 16 | box plot + quartile text | 1 | `d2_schematic` or `table` | `redraw` (schematic) or `describe` | low |
| `verimadenciliği:14` | slide 17 | quantile plot | 2 | `unique` (exact data) | `embed_crop` or `describe` | low |
| `verimadenciliği:14` | slide 18 | scatter plot | 2 | `unique` (exact pattern) | `embed_crop` or `describe` | low |
| `verimadenciliği:45` | slide 49 | example table (`P(+\|A)`, class) | 2 | table | `native_table` | high |
| `verimadenciliği:45` | slide 50 | ROC curve + confusion matrix | 2 | `unique` (exact thresholds) + `table` | `native_table` + `embed_crop` | low |
| `verimadenciliği:45` | slide 52 | classification workflow (boxes/arrows) | 2 | `d2_concept` | `redraw` (D2) | high |
| `verimadenciliği:45` | slides 51/53/54 | text/formulas | 0 | text | skip | — |
| `verimadenciliği:71` | slide 31 | four-panel clustering scatter | 2 | `d2_schematic` or `unique` | `redraw` (schematic) or `embed_crop` | low |
| `PD2:5` | slide | language family tree | 1 | `d2_concept` | `redraw` (D2) | high |
| `PD2:29` | slide | two LISP list representations | 2 | `d2_concept` | `redraw` (D2) | high |
| `PD2:53` | slide | BASIC genealogy (nodes/arrows) | 1 | `d2_concept` | `redraw` (D2) | high |
| `PD2:45` | slide | historical COBOL photo | 0 | photo | skip | — |
| `PD2:101` | slide | decorative cartoon | 0 | photo | skip | — |
| `PD3:73/74` | slides | attribute-flow diagrams | 2 | `d2_concept` | `redraw` (D2) | high |
| `PD3:76/77` | slides | syntax graphs (boxes/arrows) | 2 | `d2_concept` | `redraw` (D2) | high |
| `PD3:80` | slide | three meaning-definition methods | 2 | `d2_concept` | `redraw` (D2) | high |
| `PD3:5/26/60` | slides | definition/error/EBNF tables | 2 | `table` | `native_table` | high |
| `PD0:1`, `PD1:1`, `PD3:1` | cover | cover motif | 0 | `decoration` | skip | — |

**Relevance labels.** `Material 2` maps to `core`, `1` to `supporting`, and
`0` to `decorative`. A separate `seductive` label is reserved for visuals that
are interesting but irrelevant to the section; the research basis below
explains why those are skipped even when they increase liking.

### 1.1 What the inspection proved

1. **Page-level evidence is too coarse.** `verimadenciliği:14` contains four
   different visuals across six slides; one page decision cannot describe
   them. The candidate unit must be a bbox, not a page.
2. **A page can mix table + chart + diagram.** `verimadenciliği:45` has a
   native table, a ROC chart and a D2-rebuildable workflow on the same page.
   Per-candidate decisions are required.
3. **“Native image” is not automatically better.** The real plot images in the
   6-up data-mining pages are small (`207×155`, `251×106`, `249×142` at
   ~150 ppi). Extracting them natively and placing them at 90% guide width
   would be blurry (~45 DPI). The quality gate must compute effective DPI at
   the **natural bbox width** and recommend a smaller width or `describe`.
4. **Decorative images are abundant.** The same 6-up pages contain 1-pixel
   title strips (`142×1`, `158×1`, `3441×1`) and 30×23 logos. These must be
   filtered by aspect/area/repeat rules before scoring.
5. **D2 is appropriate for concept diagrams and qualitative schematics.**
   PD2 genealogy trees and PD3 syntax graphs are boxes/arrows/text; D2
   reproduces them crisply and consistently with the guide palette. D2 can
   also sketch qualitative relations such as cluster blobs, cluster separation
   or a box-plot schematic, when the caption says “şematik” and no data values
   are asserted. D2 is **not a plotting library**: exact XY coordinates,
   thresholds and values must come from a source crop, table or description.
6. **A page-level `chart_likely` flag has false positives.** The old report
   flags pages 14/45/71 as chart pages, but page 45 also has a table and a
   diagram. Candidate-level classification is the fix.
7. **Layout blocks alone are not enough on 6-up pages.** The stored blocks show
   a page-spanning `table` on page 45 (480×639 pt, no text) and no `figure`
   blocks on page 71, where the clustering scatter is 3,452 vector objects. A
   vector-cluster fallback and a container rejection rule are required.

### 1.2 Calibration protocol

After the scorer is implemented:

1. Run `visual_candidates` on the labels above.
2. Render the top candidate crops and inspect them visually (the host can see
   images in this environment; the MCP server does not need to).
3. Compare:
   - material visuals in top 10: **recall ≥ 0.90**;
   - top 5 precision (material/unique): **≥ 0.80**;
   - decorative images in top 5: **0**;
   - representation agreement (`d2_concept｜d2_schematic｜table｜unique`) with
     the table: **≥ 0.85**;
   - action agreement (`embed_image｜embed_crop｜native_table｜redraw｜describe｜skip`):
     **≥ 0.80**;
   - no exact chart classified as high-confidence `d2_schematic` without a
     “şematik” caption and a source/table fallback.
4. Tune weights/thresholds only on a calibration split; keep PD0/PD1 as the
   out-of-sample check.
5. Record the confusion matrix and every disagreement in
   `docs/plans/2026-11-05-source-visual-calibration.md`.
6. Freeze `scorer_version=1` only after the manual review; a weight change
   bumps the version.

This is the manual part the host agent owns. The scorer is not allowed to
claim “important” without this review.

## 3. Scope

### In scope

- Enumerate visual candidates with stable IDs and bounding boxes.
- Score **salience** deterministically and explain the score.
- Classify **relevance**: `core｜supporting｜decorative｜seductive`.
- Classify **reconstruction**: `d2_concept｜d2_schematic｜table｜text｜unique`
  with confidence.
- Extract native raster images losslessly when they are the best rendition.
- Crop vector/mixed figures at adaptive print quality when a crop is the best
  rendition.
- Prefer native table extraction/redraw over blurry table screenshots.
- Deduplicate repeated images and detect decorative placements.
- Record one audit decision per **candidate** while keeping legacy per-page
  entries valid.
- Validate asset quality (blank, resolution, aspect, size, caption/alt).
- Materialize missing assets from a stored recipe; verify hashes.
- Add MCP/CLI one-call workflows and a strict preflight.
- Add a cost-aware, text-first host loop.
- Enforce research-based caption, placement and exactness rules.
- Keep the A+B chart policy and D2 boundary unchanged.

### Out of scope (explicit)

- No pixel-semantic “is this chart important?” model; no local/remote vision
  model inside the MCP server.
- No **quantitative** chart reconstruction from pixels; no chart renderer (C remains deferred). A D2 schematic may express a qualitative relation only.
- No OCR by default; no full-PDF OCR.
- No PDF-specific 6-up/slide-grid segmentation. Candidate granularity is the
  layout/image bbox; the host may refine a crop manually.
- No automatic editing of `guide.md` (the tool returns an exact Markdown
  snippet; content placement stays with the host).
- No hard quota on the number of visuals. A soft budget may warn; it does not
  fail by default.
- No D2 quota and no automatic D2/source-crop pairing. D2 remains a
  concept-diagram and qualitative-schematic engine; source crops are embedded
  through `::: figure`. `schematic_of` is an explicit host choice when exact
  data matters, not an automatic coupling. D2 is not a plotting library.

## 3.5 Tool and repository evaluation

The existing stack is sufficient for v1. No new repository or runtime
dependency is required.

| Tool/repo | What it already gives us | What it cannot do | Verdict |
|---|---|---|---|
| `liteparse` 2.17 | layout blocks, figure/table blocks, `image_refs`, native image bytes (`extract_images`), `duplicate_of`, vector graphics, structure tree, complexity | does not judge materiality or D2 suitability | **use as the primary evidence source** |
| `pdfium` via `liteparse-pdfium` | page rendering and image-object rendering | no semantic understanding | **use for crop rendering** |
| `image` + `blake3` | decode, blank/entropy checks, crop, hash | no content understanding | **use for quality and integrity** |
| `D2` | crisp concept diagrams and qualitative schematics (flow, tree, sequence, class, ERD, cluster blobs) | exact/quantitative charts, photos, screenshots, arbitrary XY plots | **use only for `d2_concept`/`d2_schematic`; exact charts stay `unique`** |
| `Model2Vec` | local semantic similarity for headings/context; topic clusters | cannot see pixels; cannot classify a chart | **optional relevance feature, not a hard dependency** |
| Host LLM | final judgment, caption/alt writing, D2 spec | may be expensive or unable to see images | **final decision maker; receives text-only triage** |

Conclusion: liteparse answers “what is on the page”; the scorer answers “how
salient is it”; the classifier answers “can D2 rebuild it”; the host answers
“should this guide include it”. A vision model is not required for the filter,
and adding one would need a separate ADR because the MCP server must stay
LLM-free.

## 3.6 Research basis and design implications

The full review lives in
[`docs/learning-science-and-genre-review-2026-11-02.md`](../learning-science-and-genre-review-2026-11-02.md#6-visual-learning-evidence-2026-11-05-update).
Claims must say “evidence-informed”, not “proven”; effect sizes are averages,
not guarantees. Never label a reader as a “visual learner”. The
design-relevant conclusions are:

| Principle | Evidence | Pipeline consequence |
|---|---|---|
| **Multimedia principle** | Words plus relevant explanatory pictures generally beat words alone; decorative pictures do not (Mayer, 2021; Fletcher & Tobias, 2005; Butcher, 2014). | Embed/redraw when the visual explains a relation, mechanism or data pattern. |
| **Coherence / seductive details** | Interesting but irrelevant details can reduce retention and transfer; irrelevant static images on paper can be especially harmful (Harp & Mayer, 1998; Rey, 2012; Sundararajan & Adesope, 2020). Graphics can improve liking without improving learning (Sung & Mayer, 2012). | Skip `decorative` and `seductive` candidates. “Less boring” is not a sufficient reason to embed. |
| **Dual coding / picture superiority** | Pictures are often remembered better than words (Paivio & Csapo, 1973; Paivio, 1986), but recall is not transfer. | Pair every visual with a takeaway caption; do not treat the picture as self-explanatory. |
| **Graph comprehension** | Graph reading depends on visual features, graph knowledge and content knowledge (Shah & Hoeffner, 2002; Cleveland & McGill, 1984; Heer & Bostock, 2010). | Exact charts keep their source pixels or a table; D2 schematics must not distort quantitative relations. |
| **Data visualization literacy** | Reading and producing visualizations is a measurable domain competence (Börner, Bueckle & Ginda, 2019). | In data-mining/statistics, charts are **core content**, not decoration. |
| **Multiple representations** | Multiple representations help when learners are supported in connecting them; not automatically (Ainsworth, 2006; Rau, 2017). | Pair text/table/visual deliberately; captions state the connection; avoid three redundant copies. |
| **Emotional design** | Warm colour and rounded shapes can improve affect/comprehension; transfer is mixed (Um et al., 2012; Plass et al., 2014). | Use palette/typography for affect; do not add cartoons as “emotional design”. |
| **Generative drawing** | Learner-generated drawings can support comprehension/transfer with training (Van Meter & Garner, 2005; Fiorella & Mayer, 2016). | Optional self-explanation/sketch prompt in a caption; not a substitute for a clear provided visual. |
| **Cognitive load** | Segmenting, signalling, spatial contiguity and coherence reduce extraneous load (Sweller, 1988; Mayer, 2021). | Place the figure near its paragraph; label key parts; one concept per figure. |
| **Learning styles** | No adequate evidence for matching instruction to “visual/auditory” styles (Pashler et al., 2008). | Never claim “visual learners”. Use visuals because the content has structure worth showing. |

### 3.6.1 The exactness rule

The research supports visuals, but it does **not** say any visual can be
replaced by any other visual. The pipeline therefore asks two questions:

1. **Is the visual relevant to the section?** (`core`, `supporting`,
   `decorative`, `seductive`)
2. **Does the learner need exact values/positions, or a qualitative
   relation?**

| Learner need | Representation | Example |
|---|---|---|
| Exact values, thresholds, coordinates, trends | `unique` (source crop), `table`, or `describe` | ROC thresholds, quantile values, a confusion matrix, a scatter pattern |
| Qualitative relation | `d2_schematic` | “These clusters are separated”; “these groups overlap”; “this box plot has a long upper whisker” |
| Structure, flow, hierarchy, sequence | `d2_concept` | language tree, classification workflow, syntax graph, attribute flow |
| Text only is sufficient | `text` | a bullet list with no relational content |
| Decorative or seductive | `skip` | cover motif, logo, cartoon, unrelated photo |

**D2 schematic constraints:**

- The caption must contain “şematik” (or “schematic” in English guides).
- No numeric values may be asserted unless they are also in the source text
  or a table.
- Axes may be drawn as schematic axes, but point positions are illustrative,
  not data.
- If exact data matters, the source crop or a table must accompany the
  schematic. A schematic alone is not evidence of a data pattern.
- D2 cannot place arbitrary XY points and cannot draw arbitrary smooth curves;
  it can draw circles, ovals, boxes, labels and connections. That is enough
  for qualitative cluster/box/flow schematics, not for a faithful scatter,
  ROC or quantile plot.

### 3.6.2 Caption and placement rules

- Caption states the **takeaway**, not just “Şekil 3”.
- Caption includes the source/page citation.
- Alt text describes the visual for a reader who cannot see it; it is not a
  copy of the caption.
- The figure sits next to the paragraph it explains; no figure dump at the end.
- One concept per figure; multi-panel figures get a caption that explains the
  comparison.
- No colour-only meaning; labels or patterns carry the distinction too.
- Do not repeat the same sentence in body, caption and alt text.
- Do not recolor a source chart to match the guide palette; colour may encode
  data. D2 schematics use the guide palette; source crops keep their original
  colours and get a neutral caption/border.

### 3.6.3 What this means for the host

The filter is **pro-visual, not anti-visual**. It exists because:

- relevant visuals can improve comprehension and recall;
- a visual can reduce boredom and increase engagement, but engagement is not
  the same as learning; graphics can improve liking without improving
  learning (Sung & Mayer, 2012);
- decorative/seductive visuals can increase interest while reducing learning
  (Sundararajan & Adesope, 2020);
- data-mining charts are domain content and must be preserved when exact;
- D2 schematics are useful for qualitative relations but must not pretend to
  be data;
- the host LLM should not pay to look at every image, especially when it
  cannot see images at all.

The design target is therefore **relevant visual variety**, not “more
pictures”. A guide that alternates prose, tables and meaningful figures is
easier to sustain than a wall of text; a guide that adds decorative pictures is
not automatically better and may be worse.

## 4. Vocabulary

| Term | Meaning |
|---|---|
| **Candidate** | One visual unit on a source page: a layout `figure` block, a table block, or an image object not covered by a figure block. |
| **Salience** | Deterministic estimate of how likely a candidate is material to the section. Not a truth claim and not “aesthetic quality”. |
| **Relevance** | `core` (domain content), `supporting`, `decorative` (no content) or `seductive` (interesting but irrelevant). |
| **Reconstruction** | Whether the content is better rebuilt (`d2_concept`, `d2_schematic`, `table`, `text`) or kept as a source visual (`unique`). |
| **D2 schematic** | A qualitative D2 drawing (cluster blobs, cluster separation, box-plot sketch) that must be labelled “şematik” and must not assert data values. |
| **Native asset** | The embedded raster image extracted from the PDF at its original pixels (JPEG passthrough or PNG). |
| **Crop asset** | A rendered region of the page, used for vector figures, mixed raster+vector figures, or figures that cannot be extracted natively. |
| **Materialized** | The asset exists on disk and its bytes match the audit hash. |
| **Recipe** | The data needed to regenerate an asset: source, page, `image_id` or `crop` + `dpi` + padding + renderer. |

## 5. Architecture

```text
sources/<file>.pdf
  │
  ├─ liteparse parse (blocks, image_refs, struct_nodes, vector graphics)
  │
  ├─ candidate builder
  │    ├─ figure block  → candidate (bbox, figure_id, format)
  │    ├─ table block   → candidate only with rows/header or table text
  │    ├─ image object  → candidate when no figure block covers it
  │    ├─ vector cluster → candidate when no usable figure block exists
  │    └─ container check → reject page-spanning figure/table blocks
  │
  ├─ reconstruction classifier (d2_concept / d2_schematic / table / text / unique)
  │
  ├─ salience scorer (deterministic, explainable)
  │
  ├─ visual_candidates report (ranked + page context, text-only)
  │
  ├─ host/LLM decision (reads bounded context, not the whole PDF)
  │    ├─ embed_image   → native extraction
  │    ├─ embed_crop    → adaptive-DPI page crop
  │    ├─ native_table  → existing table extractor
  │    ├─ redraw_*      → existing D2/table path
  │    ├─ describe      → text/table description
  │    └─ text/decorative/unreadable
  │
  ├─ visual_decision
  │    ├─ materializes the asset
  │    ├─ validates quality
  │    ├─ upserts visual-audit.json
  │    └─ returns the exact `::: figure` Markdown snippet
  │
  ├─ visual assets --check [--strict]
  │
  └─ build --check (existing visual_gate)
```

New modules:

| Module | Responsibility |
|---|---|
| `crates/folisift-parse/src/visual_candidates.rs` | Candidate discovery, stable IDs, context, ranking |
| `crates/folisift-parse/src/visual_score.rs` | Feature extraction, weights, thresholds, score explanation |
| `crates/folisift-parse/src/visual_classify.rs` | D2 concept/schematic, table, text and unique reconstruction classifier |
| `crates/folisift-parse/src/visual_assets.rs` | Native extraction, crop recipe, materialization, hash verification |
| `crates/folisift-parse/src/visual_quality.rs` | Blank/resolution/aspect/size checks |
| `crates/folisift-parse/src/visual_audit.rs` | Extended decisions and validation (existing module) |

No new runtime dependency is expected: `liteparse`, `image`, `blake3`,
`serde`, `serde_json` and `regex` are already available. The optional semantic
relevance feature uses the existing Model2Vec stack.

## 6. Candidate model

### 6.1 Stable ID

```text
{source}:p{page}:{kind}{ordinal}
```

- `kind`: `fig` (figure block), `img` (image object), `tbl` (table block).
- `ordinal` is assigned after sorting by `bbox.y`, then `bbox.x`, then
  `figure_id`; never by liteparse block index.
- Examples: `PD3:p76:fig1`, `verimadenciliği:p14:img3`,
  `PD2:p31:tbl1`.

Native image IDs keep the liteparse identifier (`p76_1`) because the
extraction API is keyed by it. The candidate carries both:

```json
{
  "id": "verimadenciliği:p14:img3",
  "kind": "image",
  "image_id": "p14_12",
  "bbox": {"x": 72.0, "y": 140.0, "width": 180.0, "height": 100.0},
  "page_width": 595.0,
  "page_height": 842.0,
  "area_ratio": 0.036,
  "figure_format": "png"
}
```

### 6.2 Candidate fields

| Field | Description |
|---|---|
| `id`, `source`, `page` | Stable identity |
| `kind` | `figure｜image｜table｜vector` |
| `bbox` | Viewport points, top-left origin |
| `area_ratio` | `bbox area / page area` |
| `native` | Whether a single image object can be extracted |
| `image_id` | liteparse image id when native |
| `pixel_width`, `pixel_height`, `format` | Native metadata |
| `vector_primitives` | Strokes/fills intersecting the bbox |
| `text_inside_chars` | Text characters whose bbox intersects the bbox |
| `heading` | Nearest heading above, within 360 pt |
| `caption_seed` | Tagged alt text, detected caption line, or nearest heading (marked as seed; host rewrites it as a takeaway) |
| `context` | Bounded `before`/`after` text from the same page region/column (≤ 800 chars total) |
| `repeat_key` | Fingerprint for repeated decorations |
| `group_id` | Groups multi-panel visuals on the same page (e.g., skewness curves) |
| `duplicate_of` | Native duplicate id when extraction knows it |
| `content_class` | `chart｜diagram｜table｜screenshot｜photo｜decoration｜text` |
| `relevance` | `core｜supporting｜decorative｜seductive` + confidence |
| `reconstruction` | `d2_concept｜d2_schematic｜table｜text｜unique` + confidence + reasons |
| `score`, `score_parts`, `signals`, `penalties` | Explainable salience |
| `suggested` | `embed_image｜embed_crop｜native_table｜redraw｜describe｜skip` |
| `recommended_width_percent` | Print width that preserves quality |
| `effective_dpi` | DPI at the natural bbox width |
| `review_mode` | `text-only｜visual-check-recommended｜visual-check-required` |
| `reason` | One-sentence host-facing explanation |

### 6.3 Candidate discovery rules

1. Every layout `Figure` block is a candidate **unless** it covers > 50% of
   the page; such a block is a `container`, not a single visual.
2. A layout table block is a candidate only when it has a non-empty header,
   rows, or a plausible table text preview. A text-empty, page-spanning table
   block is a `container` and is split into image/vector candidates.
3. An image object becomes an `img` candidate only when no figure block covers
   at least 60% of its area.
4. **Vector-cluster fallback:** cluster vector primitives by spatial proximity
   and create a `vector` candidate when the cluster is dense enough and no
   figure/image candidate already covers it. This covers charts drawn as paths
   (e.g., page 71, 3,452 vector objects) that the layout classifier may label
   as table-like.
5. **Sub-candidate refinement:** when a figure block covers ≥ 200 text chars
   and contains an image object covering 20–80% of its area, emit both:
   - a `panel` candidate with `suggested=skip` or `describe` (the slide body);
   - an `img` candidate for the image bbox (the actual visual).
   This is the generic 6-up fix; no filename or slide-grid heuristic.
6. Candidate boxes that overlap by IoU ≥ 0.8 are merged; a table candidate
   with extractable cells wins over a figure candidate; an image candidate
   wins over a page-spanning container.
7. Bounding boxes are clamped to `content_bounds` when available, then to the
   page.
8. `text_inside_chars` counts text blocks intersecting the candidate; this
   catches 6-up slides where a whole slide is one image and the text is
   already extracted. Context is scoped to the same y-band/column so text from
   a neighbouring slide does not leak into the caption seed.
9. Images with height ≤ 2 pt, width ≤ 24 pt, or aspect ratio > 20 are marked
   `decoration` (title strips, rules, bullets).
10. **Grouping:** multi-panel visuals on the same page that share a heading or
    are spatially adjacent get a shared `group_id` (e.g., the three skewness
    curves). The host may embed the group as one comparison or select one
    panel; the scorer does not force a split.
11. **Series/continuation:** when adjacent pages share a heading and contain
    candidates of the same `content_class`, tag them `series` and
    `series_index`. The host decides whether to use one representative or both;
    v1 does not stitch images automatically.

## 7. Salience scoring

### 7.1 Features and weights

The score is `0..100`; penalties are subtracted after the weighted sum.

| Feature | Source | Mapping | Weight |
|---|---|---|---|
| `area` | bbox / page | piecewise below | 0.20 |
| `heading` | nearest heading above | 1.0 ≤72 pt, 0.75 ≤180 pt, 0.45 ≤360 pt, else 0.2 | 0.12 |
| `caption` | tagged alt or caption regex | tagged alt 1.0; caption line 1.0; short adjacent line 0.55; none 0.1 | 0.18 |
| `complexity` | image refs + vector primitives inside bbox | single image base 0.6; vectors `+min(count/40, 0.4)`; both cap 1.0 | 0.18 |
| `quality` | native effective DPI or adaptive crop DPI | ≥200:1.0, 150:0.85, 120:0.65, 96:0.4, <96:0.2; vector at target DPI:1.0 | 0.14 |
| `low_text` | inventory page chars | 1.0 when below the source threshold, else 0 | 0.08 |
| `structure` | tagged structure node | Figure/Formula with alt 1.0; without alt 0.5; none 0 | 0.10 |

Area mapping:

```text
< 0.01         → 0.05
0.01 – 0.04    → 0.45
0.04 – 0.50    → 1.00
0.50 – 0.75    → 0.55
> 0.75         → 0.25
```

Caption regex (Turkish and English):

```text
^(Şekil|Sekil|Figure|Tablo|Table|Grafik|Chart|Diyagram|Diagram|Algoritma|Algorithm|Görsel|Gorsel)\b
```

### 7.1.1 Relevance gate

Relevance is a gate plus a small bonus. It prevents a visually attractive but
irrelevant image from climbing the ranking, and it lets domain-core charts rank
above decorative images even when the chart has no caption.

| Relevance | Signals | Effect |
|---|---|---|
| `core` | domain content (chart/table/diagram), low-text page, caption/heading, structure role | **+15 bonus**, no cap |
| `supporting` | nearby example, secondary illustration, context match | **+5 bonus**, no cap; score usually 40–69 |
| `decorative` | cover/logo/band/repeat/photo with no section link, `text_inside_chars` high, no caption | cap score at 20; `suggested=skip` |
| `seductive` | engaging/interesting image with no relation to the section context, cartoon/photo/meme | cap score at 20; `suggested=skip`; reason names the irrelevance |

The bonus is applied after the weighted sum and before the final `0..100` cap.
It is deliberately small: it breaks ties in favour of core content, it does not
rescue a low-quality or unreadable asset.

The host may override a `decorative`/`seductive` label, but the override is
recorded in the audit with a reason. The research basis is explicit: decorative
visuals can increase liking while reducing learning (Sung & Mayer, 2012;
Sundararajan & Adesope, 2020).

### 7.2 Penalties

| Signal | Penalty | Rationale |
|---|---|---|
| `text_inside_chars > 200` | `-0.15 × min(1, chars/1000)` | The “figure” is mostly a text container or a whole slide |
| table block overlaps ≥ 60% | `-0.18` | Prefer `native_table`/`redraw_table`; crops of tables are a last resort |
| repeated fingerprint on ≥ 3 pages | `-0.25` for repeats, 0 for the first | Logos, backgrounds, recurring chrome |
| `duplicate_of` present | `-0.25` | Exact native duplicate |
| center in top/bottom 6% band | `-0.20` | Header/footer decoration |
| `area_ratio < 0.01` | `-0.20` | Tiny icon |
| aspect ratio > 8 or < 1/8 | `-0.10` | Banner/line, unlikely to be a figure |
| height ≤ 2 pt or aspect > 20 | `-0.30` | Title strips and rules |

### 7.3 Suggested action

| Condition | Suggested |
|---|---|
| table block with rows/header | `native_table` |
| table block, image-only | `embed_crop` |
| `reconstruction=d2_concept`, score ≥ 50 | `redraw` |
| `reconstruction=d2_schematic`, qualitative, exact data not required | `redraw` (caption says “şematik”) |
| `reconstruction=unique` (chart/photo/screenshot), score ≥ 70 | `embed_image` or `embed_crop` |
| `reconstruction=unique`, score 50–69 | `describe`, or `embed_crop` with `quality_exception` |
| single native image covers ≥ 80% of the figure bbox, vector count < 10, `text_inside_chars < 80`, score ≥ 50 | `embed_image` |
| score ≥ 70, low-text page | `embed_image` or `embed_crop` |
| 30–49 | `describe` |
| < 30 | `skip` |
| `relevance=decorative｜seductive` | `skip` (score capped at 20) |
| repeated/decorative penalty leaves score < 20 | `skip` |

The host may always override. `reason` states why the suggestion was made.

### 7.4 Explainability example

```json
{
  "id": "verimadenciliği:p14:img3",
  "score": 58,
  "score_parts": {
    "area": 9.0,
    "heading": 12.0,
    "caption": 0.0,
    "complexity": 16.2,
    "quality": 5.6,
    "low_text": 0.0,
    "structure": 0.0,
    "relevance_bonus": 15.0
  },
  "penalties": [],
  "signals": ["multi-panel", "image-only", "chart-keywords", "core-domain"],
  "relevance": {"label": "core", "confidence": "high"},
  "reconstruction": {"kind": "unique", "confidence": "high", "reasons": ["axes", "data points"]},
  "suggested": "describe",
  "confidence": "medium",
  "reason": "core multi-panel chart; effective DPI falls below 96 at 35% width"
}
```

### 7.5 Determinism

- No timestamps in the report.
- Sort by `score desc`, then `page asc`, then `bbox.y`, then `bbox.x`.
- Include `source_digest` and `scorer_version`.
- Two runs on the same source must produce byte-identical JSON.

## 8. D2, schematic, table or unique: reconstruction classifier

This is the decision the user asked for: can D2 rebuild the visual, or is it
unique content that must be kept? The answer is not binary. The classifier
distinguishes **concept diagrams**, **qualitative schematics**, **tables**,
**text** and **unique source visuals**.

### 8.1 Signals

| Signal | How it is measured |
|---|---|
| `raster_coverage` | image object area / candidate area |
| `vector_shape_count` | rectangles/ellipses/paths inside the bbox |
| `line_count` | line segments inside the bbox |
| `vector_cluster_density` | vector primitives per 1,000 pt² in a cluster |
| `text_inside_chars` | labels inside the visual |
| `color_count` | sampled unique colors (raster images) |
| `axis_likelihood` | long horizontal/vertical lines + tick-like short lines |
| `data_point_likelihood` | many small dots/symbols spread across the bbox |
| `node_edge_likelihood` | rectangles/ellipses connected by arrows |
| `keyword_class` | title/context words (below) |
| `exactness` | whether the section asserts values, thresholds or coordinates |
| `container_span` | block area / page area; > 0.5 means a container, not a visual |
| `structure_role` | tagged `Figure`, `Formula`, `Table` |

Keyword classes:

```text
chart:   grafik, plot, scatter, saçılım, serpilme, dağılım, histogram,
         roc, eğri, quantile, kutu grafiği, box plot, regresyon,
         bar chart, pasta grafiği, çizgi grafiği
diagram: akış, süreç, model, şema, diyagram, ağaç, soyağacı, mimari,
         sınıflandırma, döngü, durum, ilişki, erd, uml, adım, katman
table:   tablo, table, matris, karşılaştırma, oran, sınıf
photo:   fotoğraf, photo, karikatür, cover, kapak, portre
```

### 8.2 The exactness question

Ask: **does the learner need to read exact values/positions, or only a
qualitative relation?**

| Learner need | Classification | Suggested |
|---|---|---|
| Exact values, thresholds, coordinates, trends | `unique` + `table` where possible | `embed_crop`/`embed_image` + `native_table` |
| Qualitative relation (separation, direction, order, overlap) | `d2_schematic` | `redraw` with a “şematik” caption |
| Structure, flow, hierarchy, sequence | `d2_concept` | `redraw` |
| Ruled/text table with extractable cells | `table` | `native_table` or `redraw_table` |
| Text only | `text` | `skip` |
| Decorative/seductive | `decoration` | `skip` |

Examples:

- **Cluster blobs / circle clusters:** `d2_schematic` is acceptable when the
  point is “these groups are separated/overlapping”; exact point positions
  are not a claim. D2 draws circles/ovals, not a point cloud.
- **Smooth curves (skewness, ROC, quantile):** `unique` or `describe`; D2
  cannot draw arbitrary curves faithfully.
- **K-means vs DBSCAN comparison:** if the point is the *shape* of the
  clusters and the algorithm behavior, `d2_schematic` can show it; if the
  exact assignment of points matters, `unique`.
- **ROC curve with thresholds:** `unique`; D2 must not invent the curve.
- **Box plot:** `d2_schematic` is acceptable for the quartile idea; exact
  values need the source crop or a table.
- **Syntax graph / flow chart / genealogy:** `d2_concept`.

### 8.3 Decision rules

| Condition | Classification | Suggested |
|---|---|---|
| table block or `table_likely` with rows/header | `table` | `native_table` or `redraw_table` |
| exact chart: axes + data points/curves + values/thresholds | `unique` | `embed_crop`/`embed_image` or `describe` |
| qualitative chart: cluster blobs, cluster separation, quartile schematic, no exact values asserted | `d2_schematic` | `redraw` with “şematik” caption |
| diagram keywords AND node/edge structure AND low color count | `d2_concept` | `redraw` |
| many small nodes + arrows + text labels, no axes | `d2_concept` | `redraw` |
| raster photo/screenshot, high color count, no axes/diagram structure | `unique` (photo) | `embed_image`/`embed_crop` or `skip` |
| repeated/logo/cover/band | `decoration` | `skip` |
| no visual primitives and text only | `text` | `skip` |
| ambiguous | `unique` with low confidence | `describe`; host decides |

**Domain prior:** in a data-mining, statistics, geometry or CS source, a
chart/table/diagram with domain keywords is `core` by default. Do not require a
low-text page or a caption to rank it. Within the page budget, prefer
embed/describe over skip: dropping the only clustering, ROC or quantile example
is worse than one extra relevant visual. This prior does **not** apply to
logos, covers, cartoons or unrelated photos.

**Safety rule:** D2 is not a plotting library. It can draw circles, blobs,
boxes, labels and connections; it cannot place arbitrary XY points or smooth
curves. A `d2_schematic` must:

1. be labelled “şematik” in the caption;
2. assert no numeric values that are not in the source text/table;
3. be accompanied by the source crop or a table when exact data matters;
4. never be high-confidence when the chart has axes, thresholds or data
   points that the learner must read.

If those conditions cannot be met, classify `unique`. A wrong redraw is worse
than a source crop.

### 8.4 Calibration expectations

The labels in §2 exercise the classifier:

- PD2:5/29/53 and PD3:73/74/76/77/80 → `d2_concept`.
- veri:45/52 → `d2_concept`.
- veri:14/16 and veri:71/31 → `d2_schematic` or `unique`, depending on
  whether exact values are asserted (the calibration label records both).
- veri:14/15 → `unique`; D2 cannot draw smooth curves faithfully.
- veri:14/17, veri:14/18, veri:45/50 → `unique` (exact data).
- veri:45/49, veri:45/50 matrix, PD3:5/26/60 → `table`.
- PD2:45/101, PD0:1, PD1:1, PD3:1 → `decoration`.

A disagreement with these labels is a calibration failure, not a reason to
loosen the rule.

## 9. Native image versus crop, and effective DPI

| Situation | Action | Why |
|---|---|---|
| One image object covers ≥ 80% of the figure, no significant vector overlay | `embed_image` | Exact original pixels; JPEG passthrough |
| Vector-only chart/diagram | `embed_crop` | Only pixels exist; adaptive crop |
| Raster + vector labels/overlay | `embed_crop` | Crop preserves the composition |
| Single raster but effective DPI < 96 | `embed_image` only with `quality_exception`; otherwise `describe` | Do not silently print blur |
| Repeated image (`duplicate_of` or repeat key) | Reuse the first asset, `reuse_of` | No duplicate files |
| Table with extractable text | `native_table` | Searchable, crisp |
| Table image with no text layer | `embed_crop` | Last resort; caption says “kaynak görsel” |
| Concept diagram or qualitative schematic better expressed as D2 | `redraw` (host choice) | D2 concept/schematic path; label schematic and keep exact source data |

### 9.1 Natural-width DPI

Do not evaluate quality at an assumed 90% guide width. A small 6-up panel at
90% width is upscaled and looks worse than the same panel at 45% width.

```text
natural_width_cm = bbox.width / 72 * 2.54
effective_dpi    = pixel_width / (natural_width_cm / 2.54)
```

`recommended_width_percent` is the largest width (35–90%) that keeps at least
150 DPI:

```text
raw_pct = pixel_width / (150 * 6.3in) * 100
recommended_width_percent = clamp(round(raw_pct), 35, 90)
```

If even 35% is below 96 DPI, mark `quality=low` and require `describe` or a
`quality_exception`.

**Raster cap:** for a crop of a raster image, effective DPI is capped by the
source image's native pixel dimensions. Rendering the page at 500 DPI does not
add detail to a 251 px image; the gate must use the native pixels, not the
upscaled crop. Vector content has no such cap: a 500 DPI render produces real
vector detail.

### 9.2 Adaptive crop DPI

A fixed 300 DPI is not enough for a 4.5 cm panel that will be shown at 14 cm.
Choose the DPI from the bbox width and the target print width:

```text
target_px = 150 * 6.3in * (recommended_width_percent / 100)
dpi = clamp(ceil(target_px / (bbox.width / 72)), 200, 500)
```

- 4.5 cm panel at 90% width → ~480 DPI.
- 14 cm chart at 90% width → ~154 DPI (300 DPI is comfortable).
- Cap at 500 DPI and 40 MP per rendered page.
- If the required DPI exceeds 500, flag low quality rather than silently
  upscaling.

This is the fix for the data-mining 6-up pages: the source has 150 ppi panels
that are only ~250 px wide; a 300 DPI crop of a 4.5 cm region is still only
531 px, but a 480–500 DPI crop reaches ~850 px and is acceptable at 90% width.

## 10. Asset extraction and storage

### 10.1 Native extraction

```text
liteparse parse with:
  extract_images = true
  target_pages = "<page>"
  extract_blocks = true
  output_format = Json
```

- Find the image by `image_id` in `ParseResult.images`.
- If `duplicate_of` is set, reuse the canonical asset instead of writing a
  second file.
- Verify the image bbox overlaps the candidate bbox (IoU ≥ 0.5).
- Write to `guides/<slug>/data/figures/<blake3>.<jpg|png>`.
- Return `hash`, `pixel_width`, `pixel_height`, `format`, `effective_dpi`,
  `duplicate_of`.

### 10.2 Crop extraction

- `bbox` + `padding` (default `4 pt`) → normalized crop.
- Clamp to `[0,1]`; reject zero width/height.
- Render one page at the adaptive `dpi` (200–500) with a 40 MP guard.
- Crop after render; write `guides/<slug>/data/crops/<blake3>.png`.
- Return `crop`, `dpi`, `pixel_width`, `pixel_height`, `hash`, `effective_dpi`.

### 10.3 Storage and security

- `guides/*/data/crops/` and `guides/*/data/figures/` are gitignored.
- Assets are local-only; the PDF output may embed them.
- Paths are guide-relative, canonicalized, and must stay inside the guide.
- Absolute paths, `..`, symlink escapes and non-asset extensions are rejected.
- Source PDFs stay read-only.

## 11. Quality gates

Every `embed_image`/`embed_crop` asset is checked before it is written to the
audit and again at build time.

| Check | Threshold | Result |
|---|---|---|
| Blank | luminance range < 8 or < 2 unique sampled colors, after compositing alpha on white | **fail** |
| Minimum short edge | < 320 px | **fail** |
| Effective DPI | < 96 | **fail** unless `quality_exception` |
| Effective DPI | 96–149 | warn |
| Aspect ratio | > 8 or < 1/8 | warn |
| File size | > 8 MB | warn |
| Caption length | < 8 or > 200 chars | **fail** |
| Alt length | < 12 or > 300 chars | **fail** |
| Caption == alt | exact match | **fail** |
| Label-only caption | `^(Şekil\|Figure\|Tablo\|Table)\s*\d+[.:]?$` | **fail** |
| Schematic caption | `redraw_kind=schematic` without “şematik”/“schematic” | **fail** |
| Decorative/seductive embed | no host override reason | **fail** |
| Placeholder text | `TODO`, `...`, `lorem` | **fail** |
| Source citation | must include source id and page number | **fail** |
| Asset hash | mismatch | **fail** |
| Duplicate asset | same hash under two different candidates | warn |
| Quality exception | only with a non-empty reason | **fail** otherwise |

`quality_exception` is for genuinely low-resolution source images that still
carry material information; it must state why the visual is necessary and
whether the host accepted a smaller print width.

## 12. Audit and provenance

### 12.1 Schema compatibility

`visual-audit` schema stays **1**. New fields are optional; existing entries
remain valid. `DECISIONS` gains `embed_image`; all old decision values remain
accepted.

New/optional entry fields:

```json
{
  "source": "<source>",
  "page": 14,
  "candidate_id": "<source>:p14:fig1",
  "decision": "embed_crop",
  "reason": "Vector chart; exact values matter, so the source crop was used.",
  "priority": "high",
  "relevance": "core",
  "reconstruction": "unique",
  "asset": "data/crops/<hash>.png",
  "asset_kind": "crop",
  "crop": "0.12,0.14,0.60,0.28",
  "bbox": "72,120,360,240",
  "dpi": 300,
  "hash": "<blake3>",
  "local_only": true,
  "pixel_width": 1500,
  "pixel_height": 1000,
  "effective_dpi": 300,
  "caption": "ROC eğrisi (kaynak görsel)",
  "alt": "ROC eğrisi; yatay eksende yanlış pozitif oranı, dikey eksende doğru pozitif oranı",
  "source_digest": "<source blake3>",
  "renderer": "pdfium <version>",
  "quality": {"blank": false, "short_edge": 1000, "effective_dpi": 300}
}
```

A redraw entry is shorter:

```json
{
  "source": "PD3",
  "page": 76,
  "candidate_id": "PD3:p76:fig1",
  "decision": "redraw",
  "reason": "Sözdizim grafiği D2 concept diagramına dönüştürüldü.",
  "priority": "high",
  "relevance": "core",
  "reconstruction": "d2_concept",
  "redraw_kind": "concept",
  "schematic_of": null,
  "asset": "diagrams/sozdizim-grafigi.json",
  "caption": "Sözdizim grafiği (şematik)",
  "alt": "Sözdizim grafiği; ifade, terim ve faktör düğümlerini gösteren akış",
  "source_digest": "<source blake3>"
}
```

Crop entries additionally carry `crop`, `bbox`, `dpi`, `padding`. A
`redraw_kind=schematic` caption must contain “şematik” and must not assert
values that are not in the source text or a table. `schematic_of` references
the source crop asset or candidate id when exact data matters; validation
requires that reference to resolve to an existing asset/decision.

### 12.2 Multiple decisions per page

- Existing key: `(source, page)`.
- New key: `(source, page, candidate_id)`.
- A page is covered when at least one entry exists.
- Legacy entries without `candidate_id` keep working.
- `visual_decision` upserts by the new key; it never silently replaces a
  different candidate’s decision.

### 12.3 Unknown fields

Add `#[serde(flatten)] extra` to audit and entry structs so host-authored
notes survive a read/write cycle.

### 12.4 Materialization

```bash
folisift visual assets --guide pd3 --check
folisift visual assets --guide pd3 --materialize
folisift visual assets --guide pd3 --materialize --refresh
```

- `--check` reports missing assets, hash mismatches, source-digest mismatches
  and unreachable recipes; exit non-zero.
- `--materialize` regenerates missing or mismatched assets from the recipe.
- `--refresh` explicitly accepts the newly rendered hash and updates the audit
  (for PDFium/platform differences).
- Build never mutates assets; it reports the command to run.

## 13. CLI and MCP

### 13.1 CLI

Add under the existing `visual` subcommand and keep `crop-source` as a
low-level alias:

```bash
folisift visual candidates --guide pd3 --source PD3 --min-score 40 --limit 20
folisift visual extract --guide pd3 --source sources/PD3.pdf --page 76 --image-id p76_1
folisift visual decision --guide pd3 --source PD3 --page 76 \
  --candidate PD3:p76:fig1 --decision embed_image \
  --caption "Sözdizim grafiği" \
  --alt "Sözdizim grafiği; giriş sembolünden ifade ve terim düğümlerine giden oklar" \
  --reason "Bölümün ana şeması; kaynak görsel doğrudan kullanıldı."
folisift visual assets --guide pd3 --check
```

`crop-source` gains:

```text
--candidate <id>       compute the crop from the candidate bbox
--bbox x,y,w,h         explicit viewport points
--padding 4            points around the bbox
--dpi auto|200..500    adaptive DPI when auto
```

`visual candidates` flags:

```text
--summary              counts + top 5 only (default)
--detail full          all fields for the bounded top N
--min-score 30         filter
--limit 20             max 100
--semantic             optional Model2Vec relevance feature
--max-context 600      characters per candidate
```

`visual decision` returns:

```json
{
  "guide": "pd3",
  "candidate_id": "PD3:p76:fig1",
  "decision": "embed_image",
  "asset": "data/figures/<hash>.png",
  "recommended_width_percent": 90,
  "markdown": "::: figure src=\"data/figures/<hash>.png\" caption=\"...\" source=\"PD3, s. 76\" alt=\"...\" width=\"90%\"\n:::",
  "audit": "guides/pd3/visual-audit.json",
  "content_unchanged": true
}
```

It does **not** edit `guide.md`; the host inserts the snippet.

### 13.2 MCP tools

| Tool | Purpose | Writes |
|---|---|---|
| `visual_candidates` | Ranked, bounded candidate report with context | no |
| `visual_extract` | Extract one native image or candidate crop | yes, under `data/` |
| `visual_decision` | Materialize, validate, upsert audit, return Markdown | yes, under `data/` + audit |
| `visual_assets` | `check｜materialize｜refresh` | yes, under `data/` |
| `crop_source` | low-level crop (existing, extended) | yes, under `data/crops/` |

`visual_candidates` defaults: `detail=summary`, `limit=20`, `max=100`,
`min_score=30`, `context_chars=600`. `detail=full` returns every field for the
bounded top N. It never returns the whole PDF and never returns image bytes.

Schema bump: **16 → 17**. Tool count: **37 → 41**.

### 13.3 `visual_candidates` response (shape)

```json
{
  "guide": "veri-smoke",
  "source": "verimadenciliği",
  "source_digest": "<blake3>",
  "scorer_version": 1,
  "counts": {"total": 42, "returned": 12, "embed": 7, "describe": 9, "skip": 26},
  "candidates": [
    {
      "id": "verimadenciliği:p14:img3",
      "kind": "image",
      "page": 14,
      "bbox": {"x": 72.0, "y": 140.0, "width": 180.0, "height": 100.0},
      "area_ratio": 0.036,
      "native": true,
      "image_id": "p14_12",
      "pixel_width": 251,
      "pixel_height": 106,
      "content_class": "chart",
      "relevance": {"label": "core", "confidence": "high"},
      "reconstruction": {"kind": "unique", "confidence": "high", "reasons": ["chart keywords", "axis + data points", "exact values matter"]},
      "heading": "Quantile Plot",
      "caption_seed": "",
      "context": {"before": "...", "after": "..."},
      "score": 58,
      "score_parts": {"area": 9.0, "heading": 12.0, "caption": 0.0, "complexity": 16.2, "quality": 5.6, "low_text": 0.0, "structure": 0.0, "relevance_bonus": 15.0},
      "signals": ["multi-panel", "image-only", "chart-keywords", "core-domain"],
      "penalties": [],
      "suggested": "describe",
      "confidence": "medium",
      "recommended_width_percent": 35,
      "effective_dpi": 100,
      "review_mode": "visual-check-recommended",
      "reason": "multi-panel chart; effective DPI falls below 96 at 35% width"
    }
  ],
  "next": null,
  "policy": "salience is advisory; the host decides"
}
```

## 14. Cost-aware host workflow

The filter exists to save the host LLM money and to support hosts that cannot
see images.

1. `visual_candidates` with `detail="summary"`, `limit=20`, `min_score=40`.
   This is **text only**; no image payloads, no image tokens.
2. Use `relevance` + `reconstruction` + `review_mode`:
   - `core` + `d2_concept`: text-only; decide from the context and redraw.
   - `core` + `table`: text-only; run `native_table`/`redraw_table`.
   - `core` + `unique` chart/photo: `visual-check-recommended`; if the host
     has vision, inspect the candidate crop. If not, prefer `describe` or a
     small crop with a quality exception; never invent values.
   - `core` + `d2_schematic`: text-only if the section only needs the
     qualitative relation; require a “şematik” caption and a source/table
     fallback when exact values matter.
   - `decorative`/`seductive`: skip without reading an image.
3. `visual_decision` writes the asset/audit and returns the Markdown snippet.
   The host does not need to see the image to call it.
4. Insert the snippet next to the paragraph it explains. Caption states the
   takeaway and includes the source/page. For a D2 schematic, the caption says
   “şematik” and the body keeps the exact table/crop if values matter.
5. `visual assets --check --strict` before building.
6. `build --guide <slug> --check`.

`--strict` adds one rule: every candidate with `score ≥ 70` must have a
decision other than `text`/`decorative` unless its reason says why it is not
material. This is a warning by default; `--strict` makes it an error. It is
the “do not silently drop the important figure” guard.

### 14.1 Model2Vec role (optional)

Model2Vec is useful for **semantic relevance**, not visual classification:

- Compare candidate `heading + caption + context` with **guide section
  headings** (parsed from `guide.md`) and source headings. This answers “is
  this visual relevant to a section the guide actually has?”, not “is this
  visual important?”.
- Add a small `semantic_relevance` feature when `--semantic` is set and the
  bundled model is available. It can lift a `supporting` candidate whose source
  heading is generic, but it must not override the relevance gate or the
  reconstruction classifier.
- Fall back to BM25/lexical overlap; never fail the command if the model is
  missing.
- Cache embeddings by source/guide digest if needed; do not persist stale
  candidate reports.
- It cannot decide whether D2 can rebuild a chart. That is the classifier in
  §8.

## 15. Build gate changes

Existing `visual_gate` semantics stay:

- `off`: no figure-page gate; low-text pages are still hard-gated by the
  existing rule.
- `warn` (default): report missing decisions and quality issues.
- `required`: figure/table pages need a decision; embedded assets must be
  materialized and pass hard quality checks. Pages that only have a
  vector-cluster candidate (e.g., data-mining page 71) are covered by
  `visual assets --check --strict`, not by the build gate, so the build does
  not over-gate every vector-heavy page.

New build messages:

| Code | Meaning | Default |
|---|---|---|
| `VISUAL_ASSET_MISSING` | Audit entry exists but the asset is missing | fail under `required`, warn under `warn` |
| `VISUAL_ASSET_HASH` | Asset hash mismatch | fail under `required`, warn under `warn` |
| `VISUAL_QUALITY` | Blank/tiny/low-DPI asset without exception | fail |
| `VISUAL_HIGH_SCORE_UNDECIDED` | Score ≥ 70 candidate has no decision | warn |
| `VISUAL_BUDGET` | Soft budget exceeded | warn |
| `VISUAL_SOURCE_CHANGED` | Source digest differs from the audit | warn |

### Soft budget

Optional config:

```json
{
  "audit": {
    "visual_gate": "required",
    "visual_budget": {
      "mode": "warn",
      "max_per_source": 16,
      "max_per_guide": 30,
      "max_visual_page_share_percent": 40
    }
  }
}
```

- Default: absent → no cap, but counts are always reported.
- `mode="warn"` is the recommended default when a budget is set.
- `mode="required"` is available but not the default; the product principle is
  “as many as the content needs, not a hard quota”.
- `max_visual_page_share_percent` is an integer so `AuditPolicy` can keep its
  `Eq` derive; avoid introducing floats into `folisift-core` config types.
- `visual_report` always reports `embedded`, `described`, `redrawn`,
  `skipped`, `estimated_visual_pages` and `budget_status`.

## 16. Work packages

| WP | Work | Main files | Exit criteria | Estimate |
|---|---|---|---|---|
| **WP0** | Calibration set, research note and fixtures | `docs/plans/2026-11-05-source-visual-calibration.md`, `docs/learning-science-and-genre-review-2026-11-02.md`, test fixture generator | §2 labels frozen; visual-learning evidence and citations linked; synthetic PDF with photo, vector chart, table, logo; data-mining pages 14/45/71 available | 1.5 d |
| **WP1** | Candidate model | `visual_candidates.rs`, `layout.rs`, `insights.rs` | Stable IDs, bboxes, context, panel/image sub-candidates, table candidates, vector-cluster fallback, container rejection, `group_id`; deterministic JSON | 2.5 d |
| **WP2** | Scorer and classifier | `visual_score.rs`, `visual_classify.rs` | Unit tests for area/caption/heading/repeat/table/text penalties and relevance gate; `d2_concept｜d2_schematic｜table｜unique` labels match §2 | 2.5 d |
| **WP3** | Native extraction and adaptive crop | `visual_assets.rs`, `render.rs`, `insights.rs` | JPEG passthrough and PNG extraction; `duplicate_of` reuse; natural-width DPI; adaptive 200–500 DPI; 40 MP guard | 2 d |
| **WP4** | Audit, quality, materialize | `visual_audit.rs`, `visual_quality.rs` | Optional fields, per-candidate key, blank/resolution checks, materialize/check/refresh | 2 d |
| **WP5** | CLI and MCP | `main.rs`, MCP `lib.rs`, `version.rs` | 4 tools + CLI; schema 17; tool count 41; all old commands still work | 1.5 d |
| **WP6** | Docs and ADR | `docs/adr/0015-source-visual-assets.md`, `architecture.md`, `authoring.md`, `authoring-spec.md`, `cookbook.md`, `CONTEXT.md`, `VERSIONS.md`, `learning-science-and-genre-review-2026-11-02.md` | Host workflow, calibration protocol, D2 schematic/concept/unique boundary, caption and relevance rules, regeneration recipe, research citations | 1.5 d |
| **WP7** | Acceptance and real sources | `scripts/check-visuals-v5.py`, `Makefile`, `acceptance.sh`, baselines | A-V5-01..30; `make acceptance`; real crop build; no visual regression | 2 d |

Total ~15.5 engineering days. WP1–WP2 can run in parallel with WP3–WP4 after the
candidate schema is fixed.

## 17. Acceptance

| ID | Check | Pass condition |
|---|---|---|
| A-V5-01 | Candidate enumeration | PD3:76 yields a figure candidate with bbox and `image_id`; data-mining 14/45/71 yield per-visual candidates, not one page blob |
| A-V5-02 | Determinism | Two runs produce byte-identical candidate JSON (no timestamp) |
| A-V5-03 | Ranking | A known material figure outranks a repeated logo/background on the fixture |
| A-V5-04 | Native extraction | Extracted bytes hash equals the source image bytes; JPEG stays JPEG |
| A-V5-05 | Native duplicate | `duplicate_of` reuses one asset; no second file |
| A-V5-06 | Crop recipe | Candidate crop + padding produces a valid PNG with stored recipe |
| A-V5-07 | `embed_image` audit | Missing `image_id` or wrong asset dir fails validation |
| A-V5-08 | `embed_crop` audit | Existing contract still passes |
| A-V5-09 | Blank gate | All-white crop is rejected |
| A-V5-10 | Low-res gate | <96 DPI fails without `quality_exception`; passes with a reason |
| A-V5-11 | Per-candidate decisions | Two candidates on one page can both have entries; legacy one-per-page still passes |
| A-V5-12 | Materialize | Delete a crop, `visual assets --materialize`, hash matches, build passes |
| A-V5-13 | Source change | Changed source digest warns; `--refresh` resolves |
| A-V5-14 | Decision output | `visual_decision` writes audit + returns Markdown; `guide.md` unchanged |
| A-V5-15 | Required gate | Missing decision fails; after decision + materialize it passes |
| A-V5-16 | High-score undecided | `--strict` fails when a score ≥70 candidate is skipped without a reason |
| A-V5-17 | Budget | Soft budget warns; `mode=required` fails only when configured |
| A-V5-18 | Decorative control | Top 5 on data-mining 14/45/71 and PD0–PD3 contains no repeated logo/background/title strip |
| A-V5-19 | Real render | Smoke guide with a real crop builds; figure page is present; page count changes by at most 1 |
| A-V5-20 | Regression | `make acceptance` green; existing guides unchanged; MCP schema 17 / 41 tools |
| A-V5-21 | Calibration precision/recall | Top-5 precision ≥0.80; top-10 material recall ≥0.90 on §2 |
| A-V5-22 | Representation agreement | ≥0.85 agreement with §2 labels (`d2_concept｜d2_schematic｜table｜unique`); no exact chart classified `d2_schematic` with high confidence |
| A-V5-23 | Natural-width DPI | A 250 px panel does not recommend 90% width; it recommends a smaller width or `describe` |
| A-V5-24 | Adaptive crop | A 4.5 cm panel at 90% width reaches ≥150 DPI at 480–500 DPI or flags low quality |
| A-V5-25 | Cost mode | `visual_candidates` response contains no image bytes; `detail=summary` is bounded; `review_mode` present |
| A-V5-26 | Relevance gate | Decorative/seductive candidates are capped and skipped; a high-salience caption cannot rescue them |
| A-V5-27 | D2 schematic contract | Caption contains “şematik”; no invented values; `schematic_of` resolves to a source crop/table decision when exact data matters |
| A-V5-28 | Caption quality | Caption states a takeaway and source/page; not a label-only “Şekil 3”; alt differs from caption |
| A-V5-29 | Research traceability | Plan and authoring docs link to the visual-learning review; the review cites the sources in §6 |
| A-V5-30 | 6-up container handling | Page 45 page-spanning table is rejected as a container; page 71 yields a vector-cluster candidate; page 14 skewness panels share a `group_id` and can be embedded as one comparison |

Manual review gate: top-10 candidates for each real source are inspected by the
host; every known material figure appears, and no logo/background appears in
the top 5. This is the honest “value” check that heuristics alone cannot prove.

**Implemented acceptance mapping:** `scripts/check-visuals-v5.py` covers
A-V5-01..13 (candidate report, determinism, D2 concept, quantitative chart,
panel grouping, top-5 decoration, native extraction, decision/audit, crop,
asset check/materialize and summary). The `folisift-parse` unit tests cover
quality, score, classifier, crop math and audit provenance (A-V5-07..11,
A-V5-26..27). A-V5-19 is covered by the existing A-V3 crop build. A-V5-20 is
`make acceptance`. A-V5-21..24 are the manual calibration report, not automated
precision claims. A-V5-28..29 are enforced by the audit/caption tests and the
research links. A-V5-30 is the page 45/71/PD3 candidate checks.

## 18. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Logos/backgrounds score high | Repeat fingerprint, header/footer band, title-strip aspect rule, full-page penalty, text-inside penalty; manual top-5 review |
| Native image is low-resolution | Natural-width effective DPI; raster cap (no credit for upscaling); recommended width; `quality_exception` with reason; `describe` alternative |
| 6-up slides produce whole-slide candidates | Panel/image sub-candidate rule; `text_inside_chars` penalty; container rejection (> 50% page); manual `--bbox` refinement; no PDF-specific 6-up heuristic |
| Layout classifier mislabels a 6-up grid as a table | Table candidate requires header/rows/text; text-empty page-spanning blocks are containers; vector-cluster fallback |
| A chart is drawn as vector paths, not an image | Vector-cluster fallback; use `vector_graphics` bboxes and density; do not rely only on `image_refs` |
| A diagram spans two pages | `series`/`series_index` tag; host chooses one representative or both; no automatic stitching in v1 |
| D2 redraws a quantitative chart incorrectly | `d2_schematic` only for qualitative claims; caption says “şematik”; no invented values; source crop/table fallback; axes/data points → `unique` unless the host accepts a schematic with a reason |
| Decorative/seductive visuals increase liking but hurt learning | Relevance gate (`core｜supporting｜decorative｜seductive`); cap decorative/seductive at low score; research basis in the learning-science review |
| The guide overclaims “visual learning” | Docs say “evidence-informed”; no learning-style matching; cite Pashler et al. (2008) and the visual-learning review |
| Rotated/cropped PDF pages | Normalize by rendered page dimensions; crop from the render, not raw PDF coordinates; rotation unit test |
| PDFium version changes crop bytes | `renderer` metadata; `visual assets --materialize --refresh` updates hashes explicitly |
| Output PDF grows | JPEG passthrough for photos, PNG for diagrams, 8 MB warning, page-cost estimate |
| Schema/API churn | Optional audit fields, schema 1 stays valid, schema 17 for tools, old CLI kept |
| Performance on large PDFs | On-demand parse, bounded output, no persistent candidate cache, native extraction one page at a time |
| Image inspection cost | Text-only triage; `review_mode`; only top candidates need visual checks |
| “Value” overclaim | The field is named `salience`, always explained; the host decides; no semantic vision claim |
| Copyright | Personal study use; source/page citation required; assets local-only |

## 19. Open decisions (defaults proposed)

1. **Budget default:** `warn` at 16/source and 30/guide, no hard cap;
   data-mining sources are visual-rich, so the warning is deliberately loose.
2. **Content insertion:** return a Markdown snippet; leave `guide.md` untouched.
3. **Native vs crop default:** native only for a single raster covering ≥80% of
   the figure with no meaningful vector overlay; crop otherwise.
4. **Real-guide proof:** use the data-mining smoke guide for the real crop
   acceptance; existing PD2/PD3 diagrams stay D2-redrawn.
5. **Vision model:** out of scope. If true semantic “importance” is required
   later, it needs a separate ADR and a local vision model; the MCP server must
   stay LLM-free.
6. **D2 schematic boundary:** default is qualitative only, with a “şematik”
   caption and no invented values. Exact charts stay `unique`; the host may
   override with a reason.
7. **Relevance labels:** default is heuristic (`core`/`supporting`/
   `decorative`/`seductive`) plus host override. Do not introduce learning-style
   labels or “visual learner” claims.
8. **Caption style:** default is takeaway + source/page. A label-only caption
   (“Şekil 3”) fails the quality gate.

## 20. Release plan

1. Implement WP1–WP7 on a branch/worktree.
2. `cargo test --workspace`, `clippy -D warnings`, `fmt --check`.
3. `make acceptance` including `visuals-v5`.
4. Build PD0/PD2/PD3, capture visual baselines, confirm no regression.
5. Run the manual calibration protocol and record the report.
6. Bump workspace version to `0.22.0`, MCP schema to `17`.
7. Add ADR-0015, update `docs/VERSIONS.md`, and link the visual-learning
   review from the normative authoring/design docs.
8. Package, check package, install locally, reload OpenCode.
9. Do **not** push or tag; the maintainer owns remote releases.
