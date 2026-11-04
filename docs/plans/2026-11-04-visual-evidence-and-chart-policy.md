# Visual evidence and chart policy plan — 2026-11-04

- **Status:** Completed — v0.20.0, build 193; A-V3-01..12 and A-V2 green
- **Source:** [`docs/FOLISIFT-GERI-BILDIRIM-v2.md`](../FOLISIFT-GERI-BILDIRIM-v2.md),
  the `verimadenciliği.pdf` review, and the A+B decision for chart-heavy
  sources
- **Previous plan:** [`2026-11-04-production-feedback-v2-remediation.md`](2026-11-04-production-feedback-v2-remediation.md)
- **Baseline:** Folisift 0.19.0, build 190, MCP schema 13, 35 tools
- **Target:** Folisift 0.20.0, MCP schema 15, 37 tools
- **Purpose:** make the global MCP honest and useful for text-heavy and
  visual/data-heavy PDFs without adding PDF-specific heuristics
- **Decision:** use **A + B**:
  - **A:** describe a chart, extract its table when present, and draw only a
    conceptual D2 schematic when that helps;
  - **B:** embed a high-resolution source crop for charts that cannot be
    faithfully reconstructed.
  - **C (chart renderer) is not in scope** until a reliable data source exists.

## 0. Why this plan exists

The v2 plan is complete through 0.19.0, but the data-mining source exposes a
class of content the current gates do not cover:

- `verimadenciliği.pdf` is 97 A4 pages, six slides per page, with a text layer.
- Page 14 has 5,662 text chars, 1 figure, 7 image refs, 186 vector objects and
  `table-likely`; page 45 has 6,240 chars, 1 figure, 1 image ref, 3,303 vector
  objects and `table-likely`.
- Both pages contain scatter/quantile/ROC/normal-distribution graphics. They
  are **not low-text pages**, so the old build visual gate does not require a
  decision for them.
- `table_extract` in native mode returned `tables: []` for pages 14, 45 and 71.
  The ROC table exists visually but is not detected as a native table.
- D2 can express flow, concept, tree, decision, sequence, class and ERD
  diagrams, but not an XY/scatter/ROC chart. There is no data-plot primitive.

The result is predictable: a text-only guide can pass while the important
charts and several tables are silently absent. This plan closes that gap
without claiming that a diagram engine can reproduce statistical plots.

## 1. Findings not closed by v2

| ID | Finding | Evidence | Disposition |
|---|---|---|---|
| F-V3-1 | Digital review manifest is still a skeleton | `coverage-manifest.review.json` has 62 source rows, 2 covered, 60 missing; `qa` falls back to legacy 41 | Review action reports the unresolved rows; legacy fallback is explicit and labeled |
| F-V3-2 | Model2Vec discovery is cwd-sensitive in tests | Installed MCP finds `~/.local/lib/folisift/models/...` and returns `model2vec`; debug MCP from an external root previously fell back | Global search order plus an installed-binary acceptance check |
| F-V3-3 | Build visual gate only covers low-text pages | Page 14/45 have figures but high text; they never enter `visual-audit` | Figure/table page gate derived from `data/blocks` |
| F-V3-4 | Native table extraction misses drawn tables | `table_extract` returns `[]` on pages 14/45/71 | Actionable reason, evidence flag and text preview |
| F-V3-5 | No chart policy | Charts are neither tables nor D2 diagrams; current output silently loses them | A+B policy |
| F-V3-6 | No source-crop contract | `render_pages` can crop pages but the guide has no provenance-aware figure directive | Crop asset contract plus `::: figure` |
| F-V3-7 | Reranker is still deferred | v2 plan explicit deferral | Keep deferred; no release claim |

## 2. Goals and non-goals

### Goals

1. Every page with substantial figure/table/vector evidence is visible to the
   host and requires an explicit disposition before a checked build passes when
   the guide opts into `visual_gate="required"`.
2. A host can choose A, B or a table for each visual page without inventing
   data.
3. A source crop is traceable: source, page, bbox, hash, caption, alt text and
   local-storage policy.
4. Global MCP instances find the bundled Model2Vec model regardless of the
   working directory.
5. The data-mining source can produce an auditable text/table/selected-crop
   guide, and the gate says what is still missing.
6. No PDF-specific slide-grid or filename heuristic.

### Non-goals

- No automatic chart reconstruction from pixels.
- No new chart renderer in this plan; C is deferred until data extraction is
  reliable.
- No full-PDF OCR by default.
- No PDF-specific 6-up segmentation; generic page/region evidence is enough.
- No claim that a source crop is a redrawn or verified figure.
- No cloud/LLM/network calls inside the MCP server.

## 3. Workstreams

### E1 — Generic visual evidence inventory

**Findings:** F-V3-3, F-V3-5; v2 V16.

**Change**

- Add a read-only report derived from `source_insights` for one source and an
  optional page range:
  - per page: `page`, `chars`, `figures`, `image_refs`, `vector_graphics`,
    `ruled_table_count`, `text_table_run_count`, `chart_likely`,
    `table_likely`, `needs_ocr`, `reasons`;
  - summary: figure/table/chart page counts and item count;
  - page lists: `figure_pages`, `table_pages`, `chart_pages`.
- `chart_likely` is evidence, not a classification of the chart type:
  - figure/image evidence plus vector graphics or a dense-graphics/embedded-
    images reason;
  - text-table runs and ruled tables make `table_likely` true;
  - axis words such as `eksen`, `x`, `y`, `dağılım`, `scatter`, `ROC` are hints
    only, never a truth claim.
- The report is derived on demand. It is not stored under the guide: a stored
  copy would duplicate `source_insights` and go stale after re-analysis.
- Expose it through `source_insights.visual_evidence` and a bounded MCP
  `visual_evidence` tool/CLI `visual-evidence` command. The report never writes
  source files.

**Interface**

```text
visual_evidence { source, pages? }
→ { source, visual_evidence: {
      total_pages, parsed_pages, figure_pages, table_pages, chart_pages,
      summary, items, policy } }
```

**Acceptance:** A-V3-01, A-V3-02; data-mining pages 14/45/71 are reported.

### E2 — Figure-bearing-page audit gate (V16)

**Findings:** F-V3-3.

**Change**

- Extend the visual-audit expected set:
  - all `low_text_pages` (existing hard requirement);
  - all pages with a `figure` or table layout block when the visual gate is
    `warn` or `required`;
  - explicit `intentional_pages` exceptions are removed from the figure/table
    set.
- Keep one decision per source page. A page-level decision covers every visual
  item on that page; this preserves the existing manifest shape and avoids a
  per-figure schema migration. A `describe`/`embed_crop` reason names the chart
  or table it covers.
- Add `config.audit.visual_gate: "off" | "warn" | "required"`:
  - `off` keeps the legacy low-text-only gate;
  - `warn` (default) reports missing figure/table decisions without failing;
  - `required` fails `build --check` until every low-text and figure/table
    evidence page is decided.
- The visual gate is applied to every checked guide structure, not only course
  coverage. Low-text pages remain hard requirements in every mode; `warn` only
  relaxes missing figure/table decisions.
- An audit file is mandatory when low-text pages exist, even with `warn`.
- `visual_brief` returns `low_text`, `has_figure`, `has_table`, `chart_likely`,
  `table_likely`, `evidence_kind`, vector/image counts, role, recommendation and
  the recorded decision.
- A text-heavy page with a chart is no longer silently exempt.

**Acceptance:** A-V3-03, A-V3-04; data-mining page 14/45 appears in
`visual_evidence` and is required when `visual_gate=required`.

### E3 — Chart/table policy A+B

**Findings:** F-V3-4, F-V3-5.

**Change**

- Extend `visual-audit.json` decisions:
  - `native_table` — source already has a table;
  - `redraw_table` — figure/matrix becomes a table;
  - `redraw_figure`/`redraw` — conceptual redraw;
  - `embed_crop` — use the source crop for fidelity;
  - `describe` — text-only description of a chart;
  - `text`, `decorative`, `unreadable`.
- `embed_crop` and `describe` require `reason`.
- `describe` must name the chart type, axes/units and trend; it must not invent
  data values. This is a content-review rule; the validator enforces only that
  a reason exists.
- `embed_crop` must provide `asset`, `crop`, `caption`, `alt`, `hash` and
  `local_only=true`. The source is the entry's `source` field. The asset must
  live under `data/crops/`; the hash is checked against the file.
- A page may contain a table and a chart, but the manifest remains one
  page-level decision. Use `embed_crop` or `describe` with a reason that names
  both, or add the table in prose/Markdown beside the figure.

**Acceptance:** A-V3-05, A-V3-06; a chart page can use A or B, and a required
gate fails without a decision.

### E4 — Source-crop contract and figure directive

**Findings:** F-V3-6.

**Change**

- Add a `::: figure` directive for provenance-aware source images:
  - `src`, `caption`, `source`, `alt`, optional `width` (50%–100%, default 90%).
- Add a crop command/MCP action:
  - `crop_source { guide, source, page, crop, out?, dpi? }`;
  - writes `guides/<slug>/data/crops/<blake3>.png` (generated, gitignored)
    and returns the path, hash, page, crop, width, height and `local_only`.
- `visual-audit` `embed_crop` resolves the asset under the guide bundle; the
  audit validates asset containment, crop syntax, hash, caption, alt, reason
  and `local_only`. The renderer fails on a missing figure asset.
- The figure renders with caption, source and alt in the PDF; the crop is not
  committed to the public repo.
- Add `.gitignore` entries for `guides/*/data/crops/` and document the local
  asset policy.

**Interface**

```text
crop_source { guide, source, page, crop: "left,top,width,height", out?, dpi? }
→ { asset, hash, page, crop, width, height, local_only: true }
```

**Acceptance:** A-V3-07, A-V3-08; a crop appears in the PDF with caption,
source and alt; missing hash or a non-local crop fails.

### E5 — Table extraction fallback

**Findings:** F-V3-4.

**Change**

- `table_extract` always returns `mode`, `source_region`, `table_likely`,
  `reason`, `manual_action` and a bounded `text_preview`.
- Add `mode="layout"` fallback when a guide is supplied:
  - use layout blocks/cell bboxes;
  - if still empty, return `table_likely=true` with a manual-transcription
    suggestion, not a bare `[]`;
  - include a bounded text preview for the host to build a Markdown table.
- `table_likely` comes from `source_insights` evidence and is OR-ed with the
  native draft. This makes a drawn table visible even when cell extraction
  fails.
- No automatic OCR or network.

**Acceptance:** A-V3-09; data-mining ROC page returns an actionable
explanation, even when exact cells are unavailable.

### E6 — Global Model2Vec discovery

**Findings:** F-V3-2.

**Change**

- `find_model_dir` search order:
  1. `FOLISIFT_MODEL_DIR`;
  2. workspace `data/models`/`models`;
  3. binary sibling `models` (installed sidecar);
  4. `~/.local/lib/folisift/models`;
  5. `~/.local/share/folisift/models`;
  6. `$XDG_DATA_HOME/folisift/models`.
- `doctor` reports the resolved model path and signature.
- `source_topics semantic=true` must use the model from any cwd; if missing,
  return `method=heading-terms`, `fallback` and `warning`.
- `make feedback-v3` uses the installed binary for the external-cwd check and
  asserts `method=model2vec` when the model exists.

**Acceptance:** A-V3-10; the installed MCP from `DersNot` returns `model2vec`.

### E7 — Digital review manifest closure

**Findings:** F-V3-1.

**Change**

- Add a `coverage_manifest review` action that emits a bounded JSON checklist
  for the review manifest:
  - source page, source heading, candidate guide heading, status, confidence.
- The host reviews and marks each row `covered`, `merged` or `excluded`;
  `reviewed_at`/`reviewed_by` are recorded.
- A file with any `missing` row remains ineligible; `qa`/`build` use legacy
  canonical rows with an explicit warning.
- `qa` reports `coverage_source`, `coverage_warning`, review rows and missing
  count through the review action and coverage report.
- The v2 acceptance target remains a reviewed digital manifest; until then the
  legacy 41/41 result is labeled legacy, not reviewed.

**Acceptance:** A-V3-11; the review action reports the unresolved row count and
`qa` labels the legacy fallback explicitly (accepted disposition for 0.20.0).

### E8 — Documentation, determinism and acceptance

**Finding:** F-V3-7 and the v2 docs/determinism work.

**Change**

- Document A+B, `embed_crop`, `describe`, crop storage/licence, chart limits,
  table fallback and the visual gate.
- `qa.environment` records model path/signature, LiteParse, D2, OCR/reranker
  policy, the visual-evidence policy version and the configured gate.
- `make feedback-v3` runs the visual/data-mining acceptance checks in
  `scripts/check-feedback-v3.py`.
- Keep the reranker explicit deferral.

**Acceptance:** A-V3-12; docs and environment record exactly what was used.

## 4. Data contracts

```text
VisualEvidenceReport {
  source: String,
  total_pages: usize,
  parsed_pages: usize,
  figure_pages: [usize],
  table_pages: [usize],
  chart_pages: [usize],
  summary: { figure_pages, table_pages, chart_pages, items },
  items: [{
    page, chars, figures, image_refs, vector_graphics,
    ruled_table_count, text_table_run_count,
    table_likely, chart_likely, needs_ocr, reasons
  }],
  policy: String
}

VisualEntry {
  source: String,
  page: usize,
  decision: native_table|redraw_table|redraw_figure|redraw|embed_crop|describe|text|decorative|unreadable|table,
  reason: Option<String>,
  asset: Option<String>,
  crop: Option<String>,
  caption: Option<String>,
  alt: Option<String>,
  hash: Option<String>,
  local_only: bool,
  priority: Option<String>
}

CropReport {
  asset: String,
  hash: String,
  page: usize,
  crop: String,
  width: u32,
  height: u32,
  local_only: bool
}

VisualReport {
  low_text_pages, entries, decision counts,
  figure_pages, visual_gate,
  missing, missing_low_text, missing_figures, unresolved, passed
}
```

## 5. Milestone

### 0.20.0 — visual evidence and A+B contract

- E1 visual evidence inventory.
- E2 figure-bearing-page audit and gate modes.
- E3 chart/table decisions.
- E4 source crop contract and figure directive.
- E5 table fallback.
- E6 global Model2Vec discovery.
- E7 digital review checklist and legacy labeling.
- E8 documentation and V3 acceptance harness.

**Exit:** A-V3-01..12 pass; repo guides stay buildable with `warn`; a
chart-sensitive smoke guide builds with `required`; installed MCP from an
external cwd uses `model2vec`; v2 acceptance remains green.

## 6. Finding register

| ID | Finding | Workstream | Priority | Target |
|---|---|---|---|---|
| F-V3-1 | Digital review manifest 62 rows, 60 missing | E7 | P0 | A-V3-11 |
| F-V3-2 | Model2Vec discovery is cwd-sensitive | E6 | P0 | A-V3-10 |
| F-V3-3 | Figure-bearing pages bypass visual audit | E2 | P0 | A-V3-03 |
| F-V3-4 | Native table extraction misses drawn tables | E5 | P1 | A-V3-09 |
| F-V3-5 | No chart policy; charts silently disappear | E3 | P1 | A-V3-05 |
| F-V3-6 | No provenance-aware crop contract | E4 | P1 | A-V3-07 |
| F-V3-7 | No visual/chart release gate | E8 | P1 | A-V3-12 |
| F-V3-8 | 6-up pages can hide charts when evidence is page-only | E1/E2 | P2 | A-V3-04 |
| F-V3-9 | `table_likely` is not actionable | E5 | P1 | A-V3-09 |
| F-V3-10 | Chart reconstruction without data is unsafe | E3 | P1 | A-V3-06 |
| F-V3-11 | Crop licence/storage policy undefined | E4 | P1 | A-V3-08 |
| F-V3-12 | Chart/table counts not in QA | E8 | P2 | A-V3-12 |
| F-V3-13 | `visual_brief` chart kind is missing | E2/E3 | P1 | A-V3-05 |
| F-V3-14 | Model fallback can look like success | E6 | P0 | A-V3-10 |

## 7. Acceptance suite

| ID | Test | Expected |
|---|---|---|
| A-V3-01 | `visual_evidence` data-mining pages 14/45/71 | figure/table/vector evidence, `chart_likely`/`table_likely` |
| A-V3-02 | Evidence report is bounded | `items`, `summary`, page lists; no raw blocks |
| A-V3-03 | Data-mining page 14/45 build gate | `visual_evidence` lists the pages; `visual_gate=required` fails without a decision and passes after one |
| A-V3-04 | 6-up page | page-level evidence retains the figure/table/vector signal; no PDF-specific slide-grid heuristic |
| A-V3-05 | Chart page A | `describe` names chart type/axes/trend in the host review; validator rejects a missing reason; `visual_brief` reports `chart_likely`, `table_likely` and `evidence_kind` |
| A-V3-06 | Chart page B | `embed_crop` uses a source crop; no generated chart claim |
| A-V3-07 | Crop provenance | PDF caption/source text and `/Alt` structure entry; hash and crop present in the audit |
| A-V3-08 | Crop policy | local/gitignored asset; `local_only=true`; missing hash/non-local crop fails |
| A-V3-09 | `table_extract` ROC page | native `mode`, `table_likely`, `reason`, text preview; layout mode returns tables or an actionable reason, never a bare `[]` |
| A-V3-10 | Installed MCP from external cwd | `source_topics(semantic=true)` → `model2vec` |
| A-V3-11 | Digital review | review action reports 62 rows/60 missing; `qa` labels legacy 41 as `coverage_source=legacy` with a warning |
| A-V3-12 | Data-mining smoke guide | text + table + selected crop builds; `warn` report names an omitted visual page; `required` passes after decisions |

## 8. Risks and open questions

1. **Chart data.** Without a reliable table/JSON, the system must not invent
   chart values. A+B makes this explicit.
2. **Crop quality.** Source crops can be low-resolution or copyrighted. The
   `local_only` field and gitignore policy are mandatory.
3. **Visual gate cost.** Figure-bearing pages require host inspection; this is
   intentional for trust, but the report must be compact.
4. **6-up handouts.** Page-level evidence is the global contract. No slide-grid
   parser is added; the pipeline must not drop a chart just because six slides
   share one page.
5. **Table fallback.** If layout cells are unavailable, the host gets a text
   preview and a manual-transcription action, not a fake table.
6. **Model path.** Installed MCP finds the sidecar; source builds use the global
   search order and the installed-binary acceptance check.
7. **Per-page decisions.** Two different visual items on one page share one
   disposition. The reason must name both; a per-item schema is deferred until
   a real guide needs it.
8. **Reranker.** Remains deferred; no plan claim depends on it.

## 9. Definition of done

- A-V3-01..12 pass through `make feedback-v3`.
- Data-mining pages 14/45 are in the visual-audit required set when analysis
  produces their layout blocks.
- A chart page can be described or cropped with provenance; no chart is
  fabricated from pixels.
- A drawn table page returns an actionable extraction/fallback result.
- Installed MCP finds Model2Vec from any cwd.
- Digital review is either a reviewed manifest or an explicitly labeled legacy
  fallback.
- The v2 acceptance suite remains green.
- The reranker is the only explicit deferral.
