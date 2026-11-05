# Source visual calibration — v0

- **Status:** Ground truth captured before implementation; implementation calibration v1 recorded below
- **Date:** 2026-11-05
- **Related plan:** [`2026-11-05-source-visual-pipeline.md`](2026-11-05-source-visual-pipeline.md)
- **Research basis:** [`../learning-science-and-genre-review-2026-11-02.md`](../learning-science-and-genre-review-2026-11-02.md#6-visual-learning-evidence-2026-11-05-update)
- **Purpose:** freeze what a human host actually saw, so the scorer cannot
  define its own truth

## 1. Method

1. Render source pages at 140 DPI with Folisift:
   `verimadenciliği.pdf` pages 14, 45 and 71; `PD3.pdf` page 76;
   `PD2.pdf` page 53.
2. Inspect the rendered pages directly (image input available to the host):
   `verimadenciliği` 14/45/71, `PD3` 76 and `PD2` 53.
3. Cross-check embedded image objects with `pdfimages -list` for pixel sizes
   and placement.
4. Cross-check the remaining PD2/PD3 labels against the existing D2 assets and
   `visual-audit.json` reasons; those diagrams are already shipped and were
   not re-labeled from scratch.
5. Label each visual:
   - `material`: `0` decorative/unnecessary, `1` supporting, `2` central;
   - `reconstruction`: `d2_concept`, `d2_schematic`, `table`, `text`, `unique`;
   - `expected action`: `embed_image`, `embed_crop`, `native_table`,
     `redraw`, `describe`, `skip`;
   - `quality`: `high`, `medium`, `low`.
6. Freeze the labels in this file. The scorer is calibrated against them; a
   disagreement is investigated, not hidden by changing the labels.

## 2. Ground truth labels

`Material 2` maps to `relevance=core`, `1` to `supporting`, and `0` to
`decorative`. A separate `seductive` label is reserved for interesting but
irrelevant visuals; the research basis explains why they are skipped even when
they increase liking.

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
| `PD2:5` | corner | “Real programmers code in binary” cartoon | 0 | `seductive` | skip | — |
| `PD2:29` | slide | two LISP list representations | 2 | `d2_concept` | `redraw` (D2) | high |
| `PD2:53` | slide | BASIC genealogy (nodes/arrows) | 1 | `d2_concept` | `redraw` (D2) | high |
| `PD2:45` | slide | historical COBOL photo | 0 | photo | skip | — |
| `PD2:101` | slide | decorative cartoon | 0 | photo | skip | — |
| `PD3:73/74` | slides | attribute-flow diagrams | 2 | `d2_concept` | `redraw` (D2) | high |
| `PD3:76/77` | slides | syntax graphs (boxes/arrows) | 2 | `d2_concept` | `redraw` (D2) | high |
| `PD3:80` | slide | three meaning-definition methods | 2 | `d2_concept` | `redraw` (D2) | high |
| `PD3:5/26/60` | slides | definition/error/EBNF tables | 2 | `table` | `native_table` | high |
| `PD0:1`, `PD1:1`, `PD3:1` | cover | cover motif | 0 | `decoration` | skip | — |

## 3. Observations that changed the design

### 3.1 Page-level evidence is too coarse

`verimadenciliği:14` has six slides:

- 13: text;
- 14: formulas;
- 15: three skewness curves;
- 16: box plot;
- 17: quantile plot;
- 18: scatter plot.

A page-level `chart_likely` flag cannot express four different visuals with
different actions. Candidate granularity must be a bbox.

### 3.2 A page can mix table, chart and diagram

`verimadenciliği:45`:

- slide 49: native table;
- slide 50: ROC chart + confusion matrix;
- slide 52: classification workflow (D2-rebuildable);
- slides 51/53/54: text.

One page-level decision would force `native_table`, `embed_crop` and `redraw`
into one bucket. Per-candidate decisions are required.

### 3.3 Native extraction is not automatically better

`pdfimages -list` shows the real plot images in the 6-up pages:

| Page | Image | Size | Placement density |
|---|---|---|---|
| 14 | quantile/scatter panels | `251×106`, `255×99`, `249×142`, `321×158` | ~150 ppi |
| 45 | ROC chart | `207×155` | ~150 ppi |
| 71 | multi-panel clustering | small panels | ~150 ppi |

These are small rasters. Extracting them natively is the exact original data,
but placing them at 90% guide width would be ~45 DPI. The quality gate must
compute **natural-width DPI**, recommend a smaller width, or prefer
`describe`.

### 3.4 Decorative images are abundant

The same pages contain:

- 1-pixel-high title strips: `142×1`, `158×1`, `3441×1`;
- 30×23 logos;
- repeated header motifs on every slide.

These must be filtered before scoring. A page with 31 image objects is not a
page with 31 figures.

### 3.5 D2 is for concept diagrams and qualitative schematics

D2 is a good fit for:

- PD2 language family tree;
- PD2 LISP list representations;
- PD2 BASIC genealogy;
- PD3 attribute-flow diagrams;
- PD3 syntax graphs;
- PD3 three meaning-definition methods;
- data-mining classification workflow.

D2 can also sketch **qualitative** relations:

- cluster blobs / circle clusters (“these groups are separated/overlapping”);
- a box-plot schematic (rectangles/lines, quartiles without exact values);
- axes as schematic axes, not as a data plot.

D2 is not a plotting library. It cannot place arbitrary XY points or draw
arbitrary smooth curves. It must not assert exact values, thresholds or point
positions that are not in the source text/table. `d2_schematic` captions must say “şematik”; if exact data matters,
the source crop or a table must accompany the schematic.

The classifier must never assign `d2_schematic` with high confidence to a
quantitative chart whose values the learner must read.

### 3.6 Layout blocks alone are not enough on 6-up pages

A manual inspection of the stored layout blocks (`data/blocks/verimadenciliği.json`)
shows why the candidate builder cannot rely only on `figure`/`table` blocks:

| Page | Layout evidence | Problem |
|---|---|---|
| 14 | 7 figure blocks, bboxes 36×67 to 160×97 pt | Good panel-level candidates; they need a `group_id` so the three skewness curves are treated as one visual, not three unrelated ones. |
| 45 | `table` block `p45:b14` covers 480×639 pt (nearly the whole page), `text_chars=0` | A page-spanning container, not a table. The ROC chart and workflow are inside it; a single table decision would be wrong. |
| 71 | no `figure` blocks; several `table`/`grid_fallback` blocks around the scatter panels; 3,452 vector objects | The clustering scatter is vector graphics misclassified as table-like. It needs a vector-cluster candidate. |

Consequences:

1. Reject `figure`/`table` blocks that cover > 50% of the page as
   **containers**, and split them into image/vector candidates.
2. Add a **vector-cluster fallback**: cluster vector graphics by spatial
   proximity into candidate bboxes when no usable figure block exists.
3. Add a **group_id** for multi-panel visuals on the same page (e.g., the three
   skewness curves), so the host can embed them as one comparison.
4. Keep table candidates only when the block has extractable rows/header or a
   plausible table text preview; a text-empty page-spanning grid is not a
   table decision.

## 4. Calibration targets

After the scorer is implemented, run it on the labels above and require:

| Metric | Target |
|---|---|
| Top-5 precision (material/central) | ≥ 0.80 |
| Top-10 material recall | ≥ 0.90 |
| Decorative images in top 5 | 0 |
| Representation agreement (`d2_concept｜d2_schematic｜table｜unique`) | ≥ 0.85 |
| Action agreement | ≥ 0.80 |
| Exact chart classified as high-confidence `d2_schematic` without a “şematik” caption/source fallback | 0 |

Expected score bands for the manual review:

| Class | Expected band | Expected action |
|---|---|---|
| Core exact chart, high-resolution | 75–95 | `embed_image`/`embed_crop` |
| Core exact chart, low-resolution | 50–70 | `describe` or `embed_crop` with exception |
| Core concept diagram | 70–90 | `redraw` |
| Core table | 75–95 | `native_table` |
| Supporting visual | 40–65 | `describe` or optional embed |
| Decorative/seductive | 0–20 | `skip` |

Keep PD0/PD1 as an out-of-sample check. Record every disagreement with the
candidate crop and the score breakdown in the calibration report.

## 5. Manual review loop

1. Run `visual_candidates` on the source.
2. For each candidate in the top 10, render the candidate crop.
3. Inspect the crop visually.
4. Compare with the label above.
5. If the score is undeserved:
   - identify the feature that caused it;
   - adjust the weight/penalty or the classifier rule;
   - re-run the whole calibration set, not just the failing item;
   - bump `scorer_version` if weights change.
6. Do not add source-specific filename/page rules. Fix the generic feature.

## 6. Implementation calibration v1 (2026-11-05)

The scorer was run against the frozen labels after implementation. The
candidate report for `verimadenciliği` pages 14/45/71 produced 17 candidates;
the top five contained no decorative image. The reviewed top candidates were:

| Candidate | Score | Suggested | Manual judgment |
|---|---:|---|---|
| `verimadenciliği:p45:vec1` | 69 | `redraw` | Slide 52 classification workflow. **D2 concept is correct.** The score is mid-range because the slide has 321 vector primitives and a low-quality source; the action is right. |
| `verimadenciliği:p71:vec1` | 50 | `describe` / `unique` | Four-panel algorithm comparison (hierarchical/DBSCAN/K-means/Max). Because exact point assignments are the lesson, the classifier now returns `unique`; **D2 schematic is only a host-chosen qualitative alternative**, not the default. |
| `verimadenciliği:p14:fig2` | 57 | `describe` | Four skewness/box panels grouped into one comparison (`group_members=4`). Low-resolution source; **describe/crop is right, not D2**. |
| `verimadenciliği:p45:tbl1` | 61 | `native_table` | The example table is present; **native table is right**. |
| `verimadenciliği:p45:img1` | 52 | `describe` | ROC image is low-resolution and title detection picked axis labels; **describe is the safe default**, host may crop with an exception. |
| `PD3:p76:img1` | 62 | `redraw` | 31 fragmented syntax-graph pieces collapsed into one candidate; **D2 concept is right**. |
| `PD2:p5:img1` | 62 | `redraw` | Language family tree; **D2 concept is right**. |
| `PD2:p53:img1` | 89 | `redraw` | BASIC genealogy; **D2 concept is right**. |
| `PD2:p29:img1` | 72 | `redraw` | LISP list representations; **D2 concept is right**. |
| `PD2:p5:fig1` | 20 | `skip` | Cartoon beside the tree; **decorative is right**. |
| `verimadenciliği:p14:fig1` | 9 | `skip` | Logo/title fragment; **skip is right**. |

What the implementation pass changed:

- The first version emitted 20+ text-slide vector clusters and 33 PD3 syntax
  graph fragments. Text-heavy clusters are now classified `text`; page-spanning
  table blocks are rejected as containers; fragmented visuals collapse into one
  group candidate.
- Page 45 slide 51 was initially classified as a chart because “model”/“ilişki”
  appeared in body text. Classification now uses the title plus text inside the
  candidate, not the neighbouring slide's context.
- Text-heavy vectors (p71 vec2/vec3, p14 vec1/vec2) are capped at 45 and
  suggested `skip`; they no longer outrank actual charts.
- Algorithm-comparison charts (`DBSCAN`, `K-means`, `Max`, hierarchical) are
  `unique`; D2 schematic remains for circle/cluster concepts without exact
  algorithm output.
- A small secondary image on a page with a dominant diagram and no page-level
  chart evidence is marked decorative. This caught the binary cartoon beside
  the PD2 language tree; the tree itself, as a large image with a page-level
  “Genealogy” signal, is `d2_concept`.
- The first version assigned axis labels as slide titles. Whole-slide vector
  candidates now prefer the topmost short line inside their own region.

Known gaps (documented, not hidden):

1. Page 14 panel titles are still weak (“ve mod değerleri”); grouping keeps the
   comparison together, but the caption seed is not a good takeaway.
2. Page 45 ROC image title detection picked axis labels (“FPR …”) rather than
   the slide title; the action is conservative (`describe`), but the host
   should inspect it.
3. The four-panel clustering scatter is now `unique`; D2 schematic is an
   explicit host alternative for a qualitative cluster concept only. The guide
   must not present it as the exact algorithm output.
4. The PD2 p5 tree still has a weak `heading` seed (`None`); the page-level
   diagram signal drives the action, but the caption must be written by the
   host. This is a caption-seed gap, not an action error.
4. The full precision/recall targets in §4 were not claimed: the labelled set
   is small and the top-10 review was manual. The next calibration pass should
   add PD0/PD1 as an out-of-sample set and record a confusion matrix.

## 7. Calibration status — v1 complete

The v1 calibration is complete for the reviewed set. Top candidates from
`verimadenciliği` pages 14/45/71 and PD2 pages 5/29/53 plus PD3 page 76 were
rendered and inspected. The final report has no decorative image in the top
five, and the reviewed actions are:

- classification workflow → `d2_concept` / redraw;
- example table → `native_table`;
- skewness group → `describe` (low-resolution, exact curves);
- scatter/ROC/quantile images → `unique` / `describe`;
- PD2 trees and LISP representations → `d2_concept` / redraw;
- PD2 binary cartoon → decorative/skip;
- PD3 syntax graph fragments → one merged `d2_concept` candidate / redraw.

This is a **v1** calibration, not a claim of perfect pedagogical importance.
The remaining gaps are listed in §6 and the precision/recall targets in §4
remain manual review targets, not automated proof.

## 8. Open calibration questions

- Should the box plot on `verimadenciliği:14/s16` be `describe`, a D2
  schematic, or a crop? It is a concept illustration, but it also encodes
  quartile positions. Default: `describe` + optional D2 schematic.
- Should the three skewness curves be a single candidate (one panel group) or
  three candidates? Default: one panel-group candidate, because the
  comparison is the point.
- Should the four-panel clustering figure on `verimadenciliği:71/s31` be
  cropped as one group or split into four? Default: one group; splitting
  destroys the comparison.
- What is the minimum readable width for a 251 px chart? Default: if 35% width
  falls below 96 DPI, prefer `describe`; the host may override with
  `quality_exception`.
