# Production feedback remediation plan — 2026-11-03

- **Status:** Proposed; requirements captured, implementation not started
- **Source:** [`docs/FOLISIFT-GERI-BILDIRIM.md`](../FOLISIFT-GERI-BILDIRIM.md)
- **Baseline:** Folisift 0.12.4, build 143, MCP schema 8
- **Purpose:** turn every finding in the production feedback into an owned,
  testable work item without losing the deterministic/local product boundary
- **Primary outcome:** “passed” must mean complete, visually continuous and
  machine-diagnosable; context must get cheaper without adding network calls

## 0. How to use this plan

This plan is the single remediation map for the feedback. It has four layers:

1. **Workstreams** — the design and interface for each class of fix.
2. **Finding register** — every feedback section and sub-finding, with an ID,
   workstream and evidence status.
3. **Acceptance suite** — the tests that prove the fix, not just the intent.
4. **Milestones** — the order of implementation and release gates.

Workstream IDs are stable. A finding is not “done” until its register row
points to a passing test and, where relevant, a migrated artifact.

**Priority legend**

- **P0** — trust or correctness blocker; must land before the next release.
- **P1** — required workflow fix; next release after P0.
- **P2** — quality, docs or ergonomics; scheduled when P0/P1 are stable.
- **P3** — optional/experimental; never release-blocking and requires explicit
  opt-in/evidence.

**Evidence status values**

- `confirmed` — reproduced or directly visible in the current code/artifacts.
- `needs-repro` — plausible and important, but the exact case must be replayed
  before changing behavior.
- `partially-fixed` — a recent change covers part of the case; regression test
  is still missing.
- `policy-gap` — behavior is undefined or undocumented rather than a code bug.

## 1. Executive decision

The feedback is correct on the three trust-critical points:

1. **Coverage can report a false pass.** Seven canonical sections passed while
   the source contained 141 real headings.
2. **Layout can split a logical unit across pages.** Chapter openers, columns,
   tables and diagrams have no keep-with-next policy.
3. **Errors are strings, not diagnostics.** They do not carry page, line,
   block, directive stack or a concrete next action.

Those three must be fixed before adding more tools. The order is deliberate:

- **Trust first:** manifest, layout continuity, structured diagnostics, D2
  decision support.
- **Workflow second:** style/restyle, QA consolidation, IRAC scaffold, counts,
  page roles.
- **Context economics third:** `section_brief`, `source_topics`, token budgets
  and cost reporting.
- **Engine depth last:** LiteParse complexity/metadata, D2 layout engines,
  Model2Vec clustering, conditional reranker, targeted OCR.

The current strengths are non-negotiable invariants: local-only inference,
deterministic builds, no LLM/network/shell in the MCP server, page provenance,
and human-authored prose.

## 2. Evidence reconciliation

### 2.1 The four heading numbers

| Measurement | Value | Meaning | Decision |
|---|---:|---|---|
| PDF pages | 117 | physical source pages | keep |
| Layout heading blocks | 267 | every heading-class block, including repeated/running headings | diagnostic hint only |
| Scan headings | 139 | one scan pass after filtering | diagnostic hint only |
| Manual topic map | 141 | user-reviewed headings | seed for manifest |
| Canonical sections | 7 | current gate input | **retire as sole gate** |

The product needs one authoritative artifact and several evidence layers:

- **`coverage_manifest.json`** — authoritative reviewed rows.
- **`source_headings.json`** — machine scan, with confidence and provenance.
- **`coverage.md`** — generated report from the manifest, never hand-written.

`267` is not “wrong” if it means raw heading blocks; it is wrong when presented
as the coverage denominator. The report must label every number.

### 2.2 Measurements to explain, not hide

The plan must expose:

- pages, source tables, scan headings, layout headings, manifest rows, covered
  rows, guide headings, generated pages and page budget;
- `tables` split into Markdown tables, wrapped tables and captioned tables;
- per-source counts in multi-source guides;
- per-section `body_chars`, no shared body values;
- 1-based page numbers everywhere;
- page roles and intentional layout exceptions.

### 2.3 Current-tree caveat

The report is based on 0.12.4, but several recent changes touch the same paths:
the revision template now accepts `width`, figures default to 90% width with a
14 cm cap, and `pd3` is 21 pages. Items marked `needs-repro` (especially D2
`width`, `direction`, `\n`) must be replayed on the current tree before they
are called regressions. The plan keeps them because a production user observed
them; the first task is to reproduce or disprove each one with a test.

### 2.4 “Passed” is a contract, not a word

A passing build must include:

```json
{
  "passed": true,
  "coverage": {"total": 141, "claimed": 141, "missing": []},
  "layout": {"chapter_splits": [], "underfilled": []},
  "audit": {"page_roles": "ok", "overflow": [], "bookmarks": "ok"},
  "counts": {"source_pages": 117, "manifest_rows": 141, "guide_sections": 17},
  "warnings": []
}
```

No component may return a bare `passed` without the counts that justify it.

## 3. What already works and must be preserved

1. `audit --strict` catches underfilled pages, overflow, orphan headings,
   bookmark mismatches, missing glyphs and tagged-PDF state.
2. D2 validate/build/status is deterministic, cached and mostly actionable.
3. `read_source` / `search_source` keep the host anchored to page-numbered
   source text.
4. `claims` / `cards` produce clean CSV and deterministic recall/answer pairs.
5. `doctor` gives a fast local diagnosis.
6. `list_guides` / `workspace_info` are practical inventory views.
7. `style preview` is a useful palette/font exploration surface.
8. The semantic model and D2 run locally with no external service.
9. `build_guide check=true` gives fast feedback.
10. `visual-audit.json` is the right idea for low-text source pages.
11. Multi-source synthesis is possible: a 207-page, 3-source, 21-low-text
    guide reached 19 pages and passed audit.

The remediation must not weaken these. New gates add evidence; they do not
remove local execution, page provenance or deterministic output.

## 4. Root cause map

| Symptom | Likely cause | Owning module |
|---|---|---|
| Coverage 7/7 vs 141 headings | canonical section set is the only gate; no reviewed manifest | coverage |
| 7 claims share `body_chars: 808` | claims bind to one body when several headings share a region | coverage |
| Empty section threshold feels semantic | fixed `400` character floor | coverage |
| Chapter kicker/title split | chapter block is atomic; no keep-with-next | render/layout |
| Columns jump to next page | one monolithic grid/container | render/layout |
| Table/diagram leaves a hole | no keep-with-next or minimum remaining space | render/layout |
| `::: front` line 2 error | no directive stack in diagnostic | markdown |
| D2 12-node ceiling | hard limits in diagram validator | diagram |
| `direction: down` becomes wide | TALA chooses aspect ratio; direction is only one input | diagram |
| `width` reportedly ineffective | needs exact replay; width path exists but may be height-capped or bypassed | diagram/render |
| `\n` label lost | D2 quote/escape path does not preserve multiline labels | diagram |
| `status: null`, `state: ok` | separate result structs | diagram/MCP |
| `recommend` ignores structure | recommendation does not filter compatibility | styles |
| `accessible` becomes 14 pages | no pre-build page estimate | styles/build |
| Manual config edits for palette | no restyle surface | styles/MCP |
| Turkish headings run together | regex/font/TOC reconciliation incomplete | parse |
| `page: 0` in coverage | mixed 0/1-based page origins | coverage/parse |
| `tables: 3` | only wrapped tables counted | build stats |
| TOC/underfilled false positive | no page roles | audit |
| IRAC scaffold wrong | generic scaffold, not structure-aware | prepare/scaffold |
| `### Sorun` rejected | structure validator only sees `##` | structure gate |
| Audit/QA split across tools | no compact release gate | MCP/QA |
| `build check` produces no draft | check failure aborts before output | build |
| Links readable only by ID | no heading labels | links |
| Claims/cards noisy | no severity/filter/defaults | claims/cards |
| Page budget thrash | no per-section page accounting | build |
| Graph has no section/heading nodes | deterministic graph is block-first only | graph |
| Agent reads too much source | no `section_brief`/`read_section` | context |
| Token cost invisible | no cost/usage report | context |
| OCR considered for text PDFs | no complexity gate/policy | parse/OCR |
| D2/LiteParse/Model2Vec capabilities unused | narrow adapters | engine depth |

## 5. Workstreams

### W0 — Measurement and release baseline

**Findings:** §0.1, §37, §37.6, §29.1, §33.

**Change**

- Add a host-visible `cost_report` artifact (JSON/Markdown) that records:
  source chars, chars read, guide words, build iterations, tool calls, source
  pages read visually, estimated tokens, elapsed time and optional API cost.
- Keep the MCP server free of provider SDKs and API keys. Cost is computed by
  the host or a read-only CLI/report from workspace artifacts.
- Define benchmark tasks before changing behavior:
  - short PDF + one-page summary;
  - dense PDF + full guide;
  - multi-source PDF + cross-source synthesis;
  - revision sheet and IRAC workbook.
- Baselines:
  - direct PDF attachment to the LLM;
  - extracted-text prompt;
  - Folisift MCP flow.
- Metrics: `time_to_first_draft`, `time_to_pass`, `tokens_in/out`, `tool_calls`,
  `source_chars_read/source_chars_total`, `manual_fix_count`,
  `build_iterations`, `coverage_manual_rows`, `visual_page_reads`.

**Acceptance:** the benchmark report exists and a release claim about savings
is only made when the measured result beats the text baseline by at least 25%
cost or 30% time at equal coverage/audit quality.

### W1 — Coverage manifest and multi-source completeness

**Findings:** §0.1, §2, §3, §8, §9, §12.1, §32, §33, §40.3, §40.5.

**Change**

- Introduce `HeadingManifest` and `CoverageManifest` as the sole coverage
  contract. `coverage_manifest` supports `build`, `read`, `update`, `validate`.
- A row carries: source, source page (1-based), heading, normalized heading,
  guide heading/node, status (`covered`, `missing`, `merged`, `excluded`),
  evidence anchor and confidence.
- `prepare_guide` writes a manifest skeleton from scan headings + TOC; it never
  claims the scan is complete.
- `check_coverage` validates every manifest row, not a seven-section shortcut.
- Multi-source guides keep a separate manifest per source and a combined
  summary. A 31-slide source cannot silently contribute zero rows.
- `coverage.md` and the G section counts are generated from the manifest.
- Separate “empty section” from a fixed character floor: report no substantive
  block, below threshold, and the configured threshold; make the threshold a
  config value instead of an unexplained 400.
- Add a manual heading review artifact `headings.review.md`; overrides support
  `add`, `rename`, `merge`, `page`.
- Page numbers are 1-based in manifest, claims, coverage and MCP.

**Interface**

```text
coverage_manifest { slug, action: build|read|update|validate, entries? }
```

**Acceptance**

- 141 headings -> `total: 141, claimed: 141, missing: []`.
- Remove one row -> `missing: ["..."]`, `passed: false`.
- Three-source guide -> three per-source totals and one combined total.
- `coverage.md` and `topic-map` cannot disagree; the latter becomes generated.

### W2 — Layout continuity and page composition

**Findings:** §0, §1.1, §2, §4.1–4.4, §11, §18, §23, §32.

**Change**

- Add `ChapterOptions`: `keep_with_next`, `break_before`, `break_after`,
  `allow_split`, `full_page_opener`.
- Add `ColumnsOptions`: `flow`, `min_rows_before_break`, `keep_with_next`.
  Glossaries no longer use `columns`; they use the four-column glossary table
  in W22. `columns` remains for non-glossary side-by-side prose.
- Add table/diagram placement options: `keep_with_next`, `min_space_mm`,
  `allow_page_break`.
- Default `keep_with_next=true` for chapter opener plus first content block,
  but `full_page_opener=true` or `break_before=true` opts out. Existing
  full-page chapter openers keep their current behavior through an explicit
  migration flag, not a silent layout change.
- Add `chapter_split` audit only for a split kicker/title or a title separated
  from its first content; do not flag an intentional full-page opener.
- Document `flow=true` explicitly: it pulls the chapter into the previous
  page's remaining space.
- `::: front` + nested `::: table` gets a directive-stack diagnostic and a
  documented two-close example; decide and document whether `front` may
  auto-close before the next chapter.
- Define `columnbreak` semantics: new column vs new page, minimum content per
  column, and interaction with `flow`.
- Add `TocOptions`: `subs` means “hide headings below this chapter's first
  level”, `toc_depth` is explicit, `toc_include`/`toc_exclude` are lists;
  document exactly which heading levels leave the TOC.
- Table/diagram `width` and `height` are applied or rejected; never silently
  ignored. A diagram may span at most two pages only when explicitly opted in;
  otherwise return `E_DIAGRAM_PAGE_SPAN`.
- Cover/title/subtitle spacing is token-driven and audited like any other
  component; no ad-hoc guide-local spacing edits.

**Acceptance**

- Chapter kicker/title/first block stay together or produce `E_CHAPTER_SPLIT`.
- A long glossary renders through W22 as a four-column table; a legacy
  `columns` glossary is migrated or flagged.
- A table that does not fit moves with its heading, not alone.
- `width="60%"` changes the rendered bbox by a measurable amount or fails.

### W3 — Structured diagnostics and draft builds

**Findings:** §0, §2, §4.4, §5, §12.1, §13, §23, §32.

**Change**

- Add `ErrorCode` and `Diagnostic` with stable codes so an MCP client can
  retry or repair without parsing prose:
  `code`, `message`, `line`, `page`, `block_id`, `directive_stack`,
  `details`, `suggestions`.
- Every CLI/MCP error returns JSON. Human text remains a rendering of the same
  object.
- Codes: `E_PAGE_BUDGET`, `E_UNDERFILLED`, `E_OVERFLOW`, `E_STRUCTURE`,
  `E_COVERAGE`, `E_MARKDOWN`, `E_VISUAL`, `E_DIAGRAM`, `E_CHAPTER_SPLIT`,
  `E_TABLE_WIDTHS`, `E_DIAGRAM_WIDTH`.
- Directive parser tracks open directives with opening lines; an unclosed
  `front` reports the actual open stack, not the frontmatter line.
- `build_guide check=true` always writes `output/_draft.pdf` on failure and
  returns `draft_path` plus diagnostics. It either runs the full strict audit
  or clearly exposes `audit` as a separate mandatory step; `qa_guide` is the
  one-call contract.
- `qa_guide` returns the same diagnostic objects, not new strings.

**Acceptance**

- Malformed front/table returns the expected `E_MARKDOWN` with directive stack.
- Underfilled build returns page, chars, ratio, page role and suggestion.
- `build check=true` failure leaves a readable draft PDF.

### W4 — D2 decision diagrams and result contract

**Findings:** §0, §2, §6.1–6.5, §24, §32, §41.2–41.3.

**Change**

- Raise limits for `tree`/`decision`: 24 nodes, 40 edges, depth 8; keep a
  page-fit warning above 20 nodes.
- Add `kind: decision` with explicit terminal/leaf semantics.
- Add `layout` choices (`tala`, `elk`, `dagre`) and `aspect_ratio`; `direction`
  must be honored or rejected, not silently ignored.
- Add per-diagram `width`/`height`/`max_width`/`max_height`/`fit`.
- Add multiline labels via `label_lines` and, if safe, escaped `\n`.
- Unify `DiagramResult`: `id`, `ok`, `state`, `cached`, `svg`, `errors`,
  `warnings`; bump the result schema and mark old `status`/`ok` fields
  deprecated for one release.
- Add `diagram_fit`/`diagram preview` that reports estimated page share before
  a full build. Allow a two-page diagram only when `allow_page_span=true`;
  otherwise reject it with a clear diagnostic.
- Document that TALA can choose aspect ratio; if `direction` cannot be
  guaranteed, say so and expose the chosen layout.

**Acceptance**

- 20-node legal decision tree validates and fits one page.
- `width=60%` changes the rendered width; otherwise `E_DIAGRAM_WIDTH`.
- Multiline label renders two lines.
- `diagram status` has one state field and one result schema.

### W5 — Turkish heading extraction and page indexing

**Findings:** §0.1, §3, §8, §9, §12.1, §40.3, §40.5.

**Change**

- Combine numbered-heading regex, font size/weight from LiteParse metadata,
  TOC reconciliation and canonical section sequence.
- Normalize Turkish `İ/ı/ş/ğ/ç/ö/ü` consistently; test glued headings such as
  `1.İş`, `5.2.İş`, `13.4.Tutulacak`.
- Generate `headings.review.md` with uncertain rows; support manual overrides.
- Reconcile TOC against body headings; report missing and duplicate candidates.
- Use 1-based pages in all public artifacts; internal indexes may be 0-based
  only inside parser code and must not leak.

**Acceptance**

- The two missing legal headings are found or flagged for review.
- Turkish glued headings separate into number + title.
- Coverage and `read_source` agree on page 1.
- TOC/body mismatch produces a review row, not a silent omission.

### W6 — Style compatibility, estimate and restyle

**Findings:** §0, §2, §7.1–7.4, §12.1, §19, §27, §28.1.

**Change**

- `style recommend` filters by structure/domain/target pages, treats
  `supported_structures` as mandatory, and returns the closest compatible
  option with a reason when the requested one is invalid. For `irac + law` the
  known compatible set is `reader` and `accessible`; the matrix is data, not a
  hard-coded list.
- `style info` reports supported structures/domains and page-cost modifiers.
- `styles list` includes a compatibility column for the current guide.
- Scaffold style rationale is explicit, not a leaking default; the host can
  replace it without a build-time surprise.
- `style.aesthetic` is authoritative for schema 3; legacy `theme` must match or
  be rejected with a diagnostic.
- `style_estimate` estimates pages from content, style, palette, fonts and
  accessibility before a full build.
- `restyle_guide` changes style/palette/fonts without touching content hash.
- Style preview optionally uses a real guide section and includes table, box,
  recall/answer and chapter opener.
- `accessible` documents its expected page increase; incompatible targets warn
  before build.

**Acceptance**

- `irac + law + practice` never recommends an unsupported workbook.
- `restyle_guide` returns `content_unchanged: true`.
- `accessible` over-budget is reported before full build.
- Preview shows real components, not only generic prose.

### W7 — MCP surface and one QA gate

**Findings:** §0, §2, §12.1, §13, §25, §31, §32, §37.5.

**Change**

- Add `audit_guide` and `qa_guide`. `qa_guide` runs build, coverage, audit,
  visual, cards, claims and diagrams and returns one compact report with
  `build`, `coverage`, `audit`, `visual`, `cards`, `claims`, `diagrams`,
  `warnings` and `passed`.
- Add MCP tool profiles (`core`, `source`, `authoring`, `review`) and keep one
  active profile at or below 15 tools; see W21. Do not expose the full future
  tool catalog in every session.
- Add `coverage_manifest`, `restyle_guide`, `style_estimate`, `table_extract`,
  `doctor`, and graph query/path/explain.
- Align names: `styles` vs `style`; `diagram build` vs `save`/`render_plan`;
  document which action writes SVG.
- Add `detail: summary|normal|full`, `max_tokens`/`fields`, `format: md|json`
  to large responses.
- `prepare_guide` outline returns `total`, `next`, full pagination and explicit
  `existing` semantics; no overwrite without `force`; its `style` description
  lists every selectable style, not only paper/clean.
- `prepare_guide` vs `scaffold_guide` gets a documented decision table.
- `build_guide` returns `draft_path` on failure.
- `workspace_info`/`list_guides` expose `guide_kind`, `case_count`,
  `source_count` and `guide_count` with documented meanings.
- `doctor` includes style/structure compatibility and reports the authoritative
  style field (`style.aesthetic` vs legacy `theme`).

**Acceptance**

- One `qa_guide` call reports build/coverage/audit/visual/cards/claims/diagrams.
- No release task requires dropping to CLI except optional D2/table authoring.
- `style`/`styles` naming is consistent in CLI and MCP.
- Outline pagination has `total` and `next`.

### W8 — IRAC scaffold and structure validator

**Findings:** §0.1, §2, §12, §32.

**Change**

- Structure-aware scaffold: `irac` produces 10 case placeholders with
  `Sorun/Kural/Uygulama/Sonuç`; source headings go to `source-headings.md`.
- Write frontmatter `sources` automatically.
- Do not overwrite an existing guide without `force`.
- Add `StructureSection.level` to every structure pack. `irac` declares
  `level: 2` or `3` explicitly; other packs default to `2`. The validator
  compares against the declared level, not a global “only `##`” rule.
- Error message names the missing section, the expected level and the actual
  line.

**Acceptance**

- `prepare_guide` IRAC skeleton has cases before source headings.
- `### Sorun` passes when the IRAC pack declares level 3, or returns “use
  `##`” with the exact line when it declares level 2.
- Existing content is preserved without `force`.

### W9 — Table and statistics truth

**Findings:** §0.1, §2, §10, §17, §21, §32.

**Change**

- Count all Markdown tables, wrapped tables, captioned tables and glossary
  tables separately: `tables.markdown`, `tables.wrapped`, `tables.captioned`,
  `tables.glossary`, `tables.total`.
- Caption numbering has `number=false`/`caption_number=false`; stable `id`
  references remain. Support caption-only labels such as “Şekil 2’nin
  dalları” without inventing a table number.
- `widths` validation reports the actual sum and column count.
- Table extraction/read surface exposes header, rows and per-cell bbox.

**Acceptance**

- A guide with 7 Markdown and 3 wrapped tables reports both counts.
- `widths="24,38"` reports `E_TABLE_WIDTHS` with sum 62.
- Caption numbering can be disabled without breaking PDF links.

### W10 — Audit page roles and visual audit

**Findings:** §0.1, §2, §11, §18, §28.1, §32, §40.2.

**Change**

- Add `PageRole`: `front_matter`, `toc`, `chapter_opener`, `figure_page`,
  `table_page`, `body`, `intentional_blank`.
- Underfilled/overflow decisions use role + text + visual content, not chars
  alone.
- Add `chapter_split`, `figure_only`, `table_only` detectors.
- `line_chars_p50/p90` exclude table cells and metadata text or report a second
  plain-text series.
- Explain `font_sizes` entries below 2 pt; they are metadata unless proven
  otherwise.
- `visual-audit` can inspect both source and output PDF; low-text source pages
  and figure-only output pages are separate reports.
- `intentional_pages`/`layout_exceptions` allow deliberate TOC/figure pages;
  replace the “first three pages” heuristic with page roles entirely.

**Acceptance**

- TOC page is not underfilled when `page_role=toc`.
- Figure-only page is flagged with a role-aware warning.
- Chapter split is detected separately from orphan heading.
- Audit report explains every unusual font-size bucket.

### W11 — Graph and link readability

**Findings:** §2, §15, §28.1, §31, §32, §38.4.

**Change**

- `links read` includes `from_heading`, `to_heading`, guide section, source
  page and a human confidence label (`high`, `medium`, `low`).
- Rename `blocks` to `source_blocks`/`guide_sections`/`links`.
- Graph adds `heading` and `section` node kinds and `contains` edges.
- `graph` supports `query`, `path`, `explain` in addition to build/status/read.
- Explain why an edge exists using shared terms/section score; never claim
  factual equivalence.

**Acceptance**

- No link read output is only `p44:s131`; heading text is present.
- `graph query "fesih"` returns section nodes.
- `graph path` and `explain` return bounded results.

### W12 — Claims and cards signal

**Findings:** §2, §16, §28.1, §37.3.

**Change**

- `cards`: zero recall blocks returns a warning and suggested directive.
- Optional cards from `definition`/`must-know` with explicit opt-in.
- `claims`: add section, severity and `missing_citation`; filter short
  transitions/questions.
- Default review intervals `1,3,7,14,30`; empty remains an explicit choice.
- `verify_citations` links claim to source page/quote.

**Acceptance**

- Empty cards never silently look successful.
- Claims output separates real claims from noise.
- Default intervals appear in CSV tags.

### W13 — Page budget UX

**Findings:** §0.1, §2, §7.2, §23, §32, §37.3.

**Change**

- `page_budget` report: pages per chapter, table/diagram share, overflow/underflow
  estimate, suggested cut/move; name the pages that exceed the target.
- `search_source` reports match count/capped count, rank, heading and section;
  a 109-page/20-result query must say that results were capped.
- `incremental_analysis` hashes sections/content blocks and reuses parse/brief
  artifacts for unchanged sections. PDF layout is always globally recomputed
  because pagination, TOC, bookmarks and page numbers are global. Report
  `reused_sections`, `rebuilt_sections`, `layout_recomputed: true`.
- `build_guide` reports which sections consume pages and what changed between
  builds.
- `style_estimate` and `diagram_fit` feed the budget before full build.

**Acceptance**

- A 25→26 page change identifies the section/figure responsible.
- Budget failure suggests a concrete section or figure move.

### W14 — Context economics and agent-first retrieval

**Findings:** §37.1–37.7, §38.4, §40.7, §41.6, §42.

**Change**

- Make briefs hierarchical and token-budgeted:

  | Brief | Count in the 141-heading case | Token budget |
  |---|---:|---:|
  | `chapter_brief` | 17 | 300–600 each |
  | `section_brief` | 17 | 150–300 each |
  | `heading_brief` | 141 | 80–150 each, on demand |

  The agent reads chapter/section briefs first and drills into heading briefs
  only when needed. A flat 141 × 300–600 brief is explicitly rejected as too
  expensive.
- `read_section`: heading-based full/summary/claims/tables reading.
- `source_topics`: embed/cluster headings, representative headings + page
  ranges + authority notes.
- `source_brief`: per-source brief.
- `visual_brief`: low-text page role + OCR text + recommendation.
- `guide_plan`: per-section target words/pages, must-cover terms, source pages,
  source order and topic-unity constraints.
- `source_topics` preserves within-source differences and records terminology
  variants; it does not flatten “Not 1 / Not 2” differences.
- `citation_snippet`: exact source sentence for a claim.
- `cost_report`: source chars, read chars, guide words, tool calls, builds,
  estimated tokens/time.
- `effort`/`token_budget` on `prepare_guide`.
- **Phasing:** `read_section`, hierarchical `section_brief`, `cost_report`,
  `detail`/`max_tokens` land in M1/M2. `source_topics`, clustering,
  classification, reranking and advanced retrieval land in M3. The expensive
  source-reading path is not left until M3.
- Large responses default to `detail: summary`; `full` is opt-in. Apply it
  first to `style info`, `coverage`, `links read`, `build_guide` and
  `prepare_guide`, which the report names as oversized.
- No generative summary in the deterministic MCP path.

**Acceptance**

- Agent can write a guide from section briefs without reading all 117 pages.
- A benchmark shows reduced source chars read and tool calls without coverage
  loss.
- Token/cost report is reproducible and contains no secrets.

### W15 — LiteParse advanced capabilities

**Findings:** §39, §41.1–41.2, §41.6–41.8.

**Change**

- Add `is_complex`/complexity policy:
  `needs_ocr`, reasons, column/table/figure coverage, garbled/full-page image.
- Add `extract_text_metadata`, `extract_vector_graphics`, `extract_images`,
  `extract_structure_tree`, `target_pages`, `continue_on_page_error`.
- Keep `extract_blocks` and table cell/bbox.
- Use metadata for heading confidence and page roles.
- Use vector/image signals for figure-only/decorative classification.
- Use `output_format="markdown"` only as a diagnostic/export, not the primary
  schema.
- Expose `keep_headers_footers`, `preserve_very_small_text`, annotations/forms
  and `ocr_server_url`/`dpi`/`num_workers` as measured options; they are not
  on by default.
- OCR policy in W18.

**Acceptance**

- A text-layer PDF reports OCR not needed from evidence.
- A low-text PDF reports per-page reasons and targeted OCR candidates.
- 267 raw heading blocks become a reviewed/ranked set, not a gate.

### W16 — D2 advanced capabilities

**Findings:** §41.3, §41.6–41.8.

**Change**

- Add `elk`/`dagre` layouts, per-container direction, `near`/`top`/`left`.
- Add node/container width/height, classes, tooltip/link, grid, `vars`,
  `imports` and theme integration.
- Add sequence/class/sql_table only where they add real value.
- Use `classes` for legal outcome branches (valid/invalid/conditional) with
  palette tokens.
- Keep sketch/icons off by default for legal/medical guides.
- Pin layout engine and version in manifests/reports.

**Acceptance**

- A two-column worker/employer decision tree renders vertically within one page.
- `classes` colors are accessible and deterministic.
- Layout engine is recorded; same spec + same engine = same SVG.

### W17 — Model2Vec and conditional reranker

**Findings:** §38.1–38.5, §41.4–41.5, §41.6.

**Change**

- Use embeddings for clustering (`source_topics`), classification
  (`coverage_hint`) and retrieval (`section_brief`).
- Use `max_length`/batch controls; deterministic tie-break by page/block order.
- Evaluate `encode_as_sequence` and truncation on long legal sections.
  `int8` is a model artifact; PCA/SIF are distillation-time choices, not
  runtime flags. Record them in the model manifest instead of exposing fake
  runtime controls.
- Preserve the useful legal bridge examples as review-only evidence; do not
  turn them into claims.
- Add FastEmbed reranker only behind a measured flag; keep model hash/top-k.
- `links read` gets heading labels and why/score; still review-only.
- Add topic clusters and read-next queue.

**Acceptance**

- Topic clusters are stable across runs.
- Coverage hint finds likely missing headings without claiming verification.
- Reranker flag increases precision in the benchmark or stays off.

### W18 — OCR policy

**Findings:** §39.1–39.6, §40.6, §41.1.

**Change**

- Default is `ocr: "off"`; `ocr: "auto"` is opt-in only through an explicit
  profile or parameter.
- `is_complex.needs_ocr` is advisory: it produces a recommendation, never an
  automatic OCR run.
- When auto is opted in:
  - skip when the text layer is sufficient;
  - run only on `low_text_pages` / complexity reasons;
  - language `tur`;
  - targeted table regions;
  - compare with the text layer and remove duplicates;
  - record confidence and `manual_review`.
- Never full-PDF OCR by default.
- Report `pages_ocr`, `pages_skipped`, `confidence`; expose `ocr_server_url`,
  `dpi`, `num_workers` as cost controls, not defaults.
- Do not OCR text-layer sources merely because OCR exists.

**Acceptance**

- 117-page text PDF: `pages_ocr=0`, report explains why.
- 21-low-text guide: only flagged pages/regions OCR; no duplicate text.
- Turkish characters and legal numbers are spot-checked.

### W19 — Determinism, documentation and release

**Findings:** §28, §28.1, §29, §33, §35, §36, §41.6.1, §41.8.

**Change**

- Record `liteparse_version`, `d2_version`, `model_hash`, `layout_engine`,
  OCR version/language and reranker hash in reports.
- Stable tie-breaks for extractive briefs and clusters.
- Cache key: source digest + model hash + brief version + policy version.
- Record quantization (int8/float), preprocessing and reranker top-k; changing
  any of them invalidates the cache.
- `docs/authoring-cookbook.md`: minimal example + closing for every directive;
  front+table nesting; `front` auto-close policy; `columnbreak`; flow; subs;
  `toc_depth/include/exclude`; diagram width/status schema; style compatibility
  matrix; cards 0; table counts including glossary; table caption numbering;
  four-column glossary examples and odd-term rows; `theme` vs
  `style.aesthetic`; counter rules.
- “Which tool when?” decision tree: prepare vs scaffold, style vs restyle,
  build vs qa, links vs graph.
- Release gate: `qa_guide` + `make exam-check` + benchmark/cost report.

**Acceptance**

- Two runs of the same guide produce identical artifacts (except timestamps
  explicitly excluded).
- Docs cover every complaint in §28/§28.1 with a runnable example.
- Release note states measured coverage, layout, QA and cost results.

### W20 — agent-fast profile

**Findings:** §37.7, §41.6, §41.6.1, §41.8.

**Change**

Make the agent-first profile explicit and opt-in:

```json
{
  "profile": "agent-fast",
  "liteparse": {"ocr": "off", "ocr_language": "tur", "complexity": true,
    "extract_blocks": true, "extract_text_metadata": true,
    "extract_vector_graphics": true, "extract_images": true,
    "extract_content_bounds": true, "continue_on_page_error": true},
  "d2": {"layouts": ["tala", "elk", "dagre"], "groups": true,
    "per_container_direction": true, "node_width_height": true,
    "near_top_left": true, "classes": true},
  "target_pages": "auto",
  "model2vec": {"clustering": true, "classification": true,
    "retrieval_top_k": 8, "max_length": 512},
  "outputs": {"detail": "summary", "max_tokens": 1200, "format": "md+json"},
  "mcp": {"profile": "core", "max_active_tools": 15, "schema_budget": "5% context"}
}
```

- `fast` means smaller briefs and fewer source pages; `deep` means more
  evidence and optional vision/OCR. Both are deterministic and page-anchored.
- `ocr:auto` is an explicit opt-in; the default profile keeps OCR off.
- The profile never enables network calls, LLM summaries or full-PDF OCR by
  default.
- Record every profile choice and model/engine version in the run report.

**Acceptance**

- `effort=fast` cuts source chars read and tool calls versus `deep` on the same
  guide without losing manifest coverage.
- Any capability that changes output is recorded in the report; reruns match.

### W21 — MCP context architecture

**Findings:** §37.1–37.7, §38.4, §41.6, §42.

**Evidence**

| Source | Pattern | Folisift consequence |
|---|---|---|
| MCP client best practices | Progressive discovery; ~150k tokens of definitions vs ~2k on demand; code mode turns ~100k+ intermediate tokens into a small script/result | Do not load the full future catalog; keep a stable small profile |
| MCP pagination spec | Opaque cursors for `tools/list`, `resources/list`, `resources/templates/list` | Every list/read surface gets cursor + `next` |
| Context7 | Resolve → query; one concept per query | `source_topics` → `section_brief`; do not combine ten concepts |
| GitHub MCP | Toolsets, dynamic toolsets, read-only, `fields`, `perPage` ≤100, line ranges | Task profiles, field selection, bounded pages |
| Sentry MCP | Search returns a list; detail is a separate resource; limit/period/compact | `search_source` never doubles as full read |
| Notion MCP | Search → page Markdown; block children cursor; Markdown beats block JSON | Resource reads return Markdown/brief, not raw blocks |
| Playwright MCP | Snapshot `target`/`depth`, file output, image omission | Bound page/block payloads; write large output to a file |
| Linear MCP | `first`/`after` cursor and `pageInfo` | Use cursor/`next` consistently |
| Matt Pocock skills | Metadata → `SKILL.md` → linked files; smart zone ~150k; phase boundaries | Coordinator reads the smallest artifact that answers the question |

**Change**

- **Coordinator rule.** The host agent is the coordinator. Folisift supplies a
  compact catalog/outline (`folisift://capabilities`) and explicit drill-down
  tools; it never pushes the source PDF, full block JSON or a 10k-line Markdown
  file by default.
- **Tool profiles.** Add `folisift mcp --profile core|source|authoring|review`.
  - `core` (default): `workspace_info`, `list_guides`, `guide_status`,
    `prepare_guide`, `build_guide`, `audit_guide`, `qa_guide`, `read_source`,
    `search_source` (≤10).
  - `source`: `analyze_source`, `read_source`, `search_source`, `read_blocks`,
    `section_brief`, `read_section`, `source_topics`, `visual_brief` (≤8).
  - `authoring`: `prepare_guide`, `coverage_manifest`, `restyle_guide`,
    `style_estimate`, `diagram`, `table_extract`, `guide_plan` (≤7).
  - `review`: `links`, `graph`, `claims`, `cards`, `cost_report`,
    `audit_guide`, `qa_guide`, `list_guides` (≤8).
  A host connects the profile(s) needed for the task. No profile exceeds 15
  tools; a profile whose serialized definitions exceed 5% of the context budget
  must split.
- **Stable tool list.** Choose the profile before the session and keep tool
  order/schema stable. Do not enable/disable tools per turn: it invalidates
  prompt caching. Append new definitions at the end only at a session boundary.
  Do not implement one giant `folisift(action, args)` mega-tool; it hides
  selection and observability. Use profiles and stable names.
- **Resources before payloads.** Add read-only MCP resources:
  - `folisift://guide/{slug}/outline`
  - `folisift://guide/{slug}/coverage/{row}`
  - `folisift://source/{id}/page/{page}`
  - `folisift://source/{id}/heading/{number}`
  Use `resources/list`/`resources/templates/list` with cursor and
  `resources/read` for one resource. A resource read has a `max_tokens` cap and
  returns Markdown or a brief, not raw block JSON.
- **Hierarchy.** `source_topics` → `chapter_brief` → `section_brief` →
  `heading_brief` → `read_section` → raw `read_source` only on explicit
  request. `read_blocks` is a diagnostic tool, not a default context source.
- **Output contract.** Every large result supports:
  `detail: summary|normal|full`, `fields`, `max_tokens`, `cursor`/`next`,
  `truncated`, `page_complete`. Large artifacts go to a file/resource and return
  a path plus a summary. Images are omitted unless explicitly requested.
- **Prompt-cache discipline.** The tools array is a cacheable prefix. Profiles
  keep it short and unchanged; resources carry the variable data. Discovery
  happens before the first turn, not mid-conversation.
- **Programmatic-call compatibility.** Tool names, typed JSON outputs and
  deterministic behavior must be stable so hosts that support code mode or
  programmatic calling can compose calls without passing intermediates through
  the model.
- **Budget metrics.** Record `tool_schema_tokens`, `tool_result_tokens`,
  `tool_calls`, `context_tokens`, `cache_read_tokens`, `source_chars_read` and
  `source_chars_total` in `cost_report`.

**Acceptance**

- A default `core` session is ≤10 tools and tool definitions are ≤5% of the
  configured context budget.
- `source` profile can answer a section question without loading `authoring` or
  `review` tools.
- `resources/read` for one page/section returns a bounded Markdown brief, not
  raw block JSON.
- `read_source` returns `page_complete=false` and a cursor when a page is cut.
- Two identical sessions with the same profile produce identical tool ordering
  and no mid-session schema churn.
- The benchmark records lower schema/result tokens than the current all-tools
  baseline.


### W22 — Glossary as a four-column table

**Findings:** §43 (production layout decision, 2026-11-03).

**Decision**

A glossary is not a two-column prose block. It is a **four-column table**
holding two term/definition pairs per row:

```text
| Terim | Tanım | Terim | Tanım |
```

The source experience is the 66-term İş Hukuku glossary: converting the
`::: columns` block to a four-column table improved density, kept term/definition
pairs together and removed the monolithic-column page jump. This becomes the
default glossary layout for every guide.

**Change**

- Add a `::: glossary` directive (or `::: table kind="glossary"`) that renders
  the four-column table. Do not model glossaries as `::: columns`.
- Group glossaries by their existing `###` headings; each group gets one table
  with a header row and balanced rows.
- Column widths default to `18,32,18,32`; the term/definition pair is atomic.
  A row may contain one pair plus a spanning definition, but never a split
  term/definition pair.
- Page breaks occur at row boundaries; repeat the header when a table
  continues on a new page. A `keep_with_next` heading rule prevents a heading
  from being separated from its first row.
- Preserve source references inside definition cells. Do not move a citation
  into a footnote or silently drop it.
- Sort terms in source/guide order; balancing is presentation only. If the
  number of terms is odd, the last row uses one pair and leaves the remaining
  cells empty or spans the definition.
- Count glossary tables separately in stats: `tables.glossary`, in addition to
  `tables.markdown`/`tables.wrapped`/`tables.captioned`.
- The glossary table must keep tagged-table structure (`TH`, `TR`, `TD`) and
  reading order for screen readers.

**Interface**

The canonical contract is `GlossaryOptions` in §7; this workstream owns its
rendering and validation, not a second copy of the struct.

**Acceptance**

- A 66-term glossary renders as a four-column table, not a two-column
  `::: columns` block.
- No term/definition pair is split across a page or table row.
- The glossary heading stays with its first row.
- The table repeats its header on a page break and keeps `TH`/`TD` tagging.
- `stats.tables.glossary` equals the number of glossary groups, and the total
  table count includes them.
- Search/link/graph outputs still point to the original source page, not the
  table row.

## 6. API surface

| Tool | Action | Output discipline |
|---|---|---|
| `coverage_manifest` | build/read/update/validate | compact summary by default; rows paginated |
| `audit_guide` | run strict audit for a guide/PDF | diagnostics + compact counts |
| `qa_guide` | build + coverage + audit + visual + cards + claims + diagrams | one release report |
| `restyle_guide` | style/palette/fonts | content hash before/after |
| `style_estimate` | estimate pages for style | page range + confidence |
| `read_section` | heading, mode full/summary/claims/tables | bounded to max_tokens |
| `section_brief` | heading/level | hierarchical: chapter 300–600, section 150–300, heading 80–150 tokens |
| `source_brief` | source | bounded source overview |
| `source_topics` | source, k | cluster labels + page ranges |
| `guide_plan` | slug/target pages | per-section word/page budget |
| `visual_brief` | source/page | role + OCR + recommendation |
| `citation_snippet` | source/page/query | exact quote + page |
| `verify_citations` | slug | claim↔source checks |
| `table_extract` | source/pages | header/rows/cell bbox |
| `graph` | build/status/read/query/path/explain | bounded nodes/edges |
| `cost_report` | slug/run | tokens/time/calls, no secrets |
| `doctor` | root | environment + style/structure compatibility |
| `diagram` | validate/save/build/status/render_plan/fit/list | unified `DiagramResult` |
| `build_guide` | check/build | `draft_path` on failure |

**Schema discipline:** one result envelope with `ok`, `code`, `diagnostics`,
`data`; large responses support `detail`, `max_tokens`, `fields`, `format`.
A MCP schema bump is required for the new tool set; old tools keep aliases
until the next major version.

**Resource surface**

| URI | Read returns | Write |
|---|---|---|
| `folisift://capabilities` | profile/tool catalog, usage recipes and budgets | no |
| `folisift://guide/{slug}/outline` | chapter/section/heading outline and counts | no |
| `folisift://guide/{slug}/coverage/{row}` | one manifest row + evidence | no |
| `folisift://source/{id}/page/{page}` | bounded page text or page brief | no |
| `folisift://source/{id}/heading/{number}` | section brief + source page | no |
| `folisift://guide/{slug}/report/{kind}` | audit/coverage/claims summary | no |

`resources/list` and `resources/templates/list` use MCP cursors. `resources/read`
never returns raw block JSON or unbounded source; the default is Markdown or a
brief with `max_tokens`. Writes remain tools, never resources.

## 7. Data contracts

```rust
struct HeadingEntry {
    source: String,
    heading: String,              // raw source text
    normalized_heading: String,   // matching key
    page: usize,                  // 1-based
    guide_heading: Option<String>,
    status: CoverageStatus,
    evidence: Option<String>,
    confidence: Confidence,
}

enum ErrorCode {
    PageBudget, Underfilled, Overflow, Structure, Coverage,
    Markdown, Visual, Diagram, ChapterSplit, TableWidths, DiagramWidth,
}

struct Diagnostic {
    code: ErrorCode,
    message: String,
    line: Option<usize>,
    page: Option<usize>,
    block_id: Option<String>,
    directive_stack: Vec<String>,
    details: serde_json::Value,
    suggestions: Vec<String>,
}

enum PageRole { FrontMatter, Toc, ChapterOpener, FigurePage, TablePage, Body, IntentionalBlank }

struct ChapterOptions {
    keep_with_next: bool,
    break_before: bool,
    break_after: bool,
    allow_split: bool,
    full_page_opener: bool, // opt out of keep_with_next
}

struct ColumnsOptions { flow: bool, min_rows_before_break: usize, keep_with_next: bool }

struct TocOptions {
    subs: bool,
    toc_depth: Option<u8>,
    include: Vec<String>,
    exclude: Vec<String>,
}

struct GlossaryOptions {
    columns: u8,           // default 4
    term_width: f32,       // default 18
    definition_width: f32, // default 32
    repeat_header: bool,   // default true
    balance_rows: bool,    // default true
}

struct DiagramOptions {
    max_nodes: usize,
    max_edges: usize,
    max_depth: usize,
    layout: DiagramLayout,
    aspect_ratio: Option<f32>,
    width: Option<String>,
    height: Option<String>,
    max_width: Option<String>,
    max_height: Option<String>,
    fit: Option<String>,
    allow_page_span: bool,
    multiline: bool,
}

struct StructureSection {
    id: String,
    tr: String,
    en: String,
    level: u8, // 2 or 3, explicit per structure pack
}

struct QaReport {
    build: BuildReport,
    coverage: CoverageReport,
    audit: AuditReport,
    visual: VisualReport,
    cards: CardReport,
    claims: ClaimsReport,
    diagrams: DiagramReport,
    warnings: Vec<Diagnostic>,
    passed: bool,
}

struct StyleApply { slug: String, style: String, palette: String, fonts: String, output: Option<String> }
```

These are deep-module seams: callers learn one contract, not the details of
Markdown parsing, Typst layout, D2 or PDF auditing.

## 8. Milestones

### M0 — Baseline and instrumentation (short)

- Cost/usage report schema.
- Benchmark fixtures and baseline runs.
- Repro tests for the four critical findings.
- Release gate checklist.

### M1 — Trust P0

- W1 coverage manifest (including multi-source).
- W2 layout continuity.
- W3 diagnostics + draft.
- W4 D2 decision support.
- W5 Turkish heading/page index.
- W21 MCP profiles, resources, output budgets and stable tool list.
- W14a `read_section`, hierarchical briefs, `cost_report`, `detail`/`max_tokens`.

**Exit:** a 141-heading source cannot pass with fewer than 141 manifest rows;
chapter/columns/table/diagram units do not split silently; errors are
machine-readable; a 20-node decision tree fits; `read_section` and the
hierarchical briefs reduce source chars read on the benchmark.

### M2 — Workflow P1

- W6 style compatibility/estimate/restyle.
- W7 MCP QA surface.
- W8 IRAC scaffold/validator.
- W9 tables/stats/numbering.
- W10 page roles/audit.
- W11 graph/links.
- W12 claims/cards.
- W13 page budget UX.
- W22 four-column glossary table.
- W14b `guide_plan`, compact outputs and per-tool field filters.

**Exit:** one `qa_guide` call is the release gate; IRAC scaffold is usable
without rewriting; counts agree across reports.

### M3 — Advanced context economics

- W14c `source_topics`, clustering/classification, advanced retrieval.
- Benchmark against direct PDF and extracted text; verify the M1/M2 briefs.
- Conditional reranker only if the benchmark justifies it.

**Exit:** at least 25% cost or 30% time improvement at equal quality on the
benchmark, or the savings claim is removed from product messaging.

### M4 — Engine depth

- W15 LiteParse complexity/metadata/vector/image/structure.
- W16 D2 advanced layouts/classes/constraints.
- W17 Model2Vec clustering/classification/conditional reranker.
- W18 OCR policy.
- W19 determinism/docs/release.
- W20 agent-fast/deep profiles.

**Exit:** advanced capabilities are opt-in, measured and deterministic; no
silent OCR, no undocumented layout behavior.

### M5 — Optional polish (P3)

- Annotations/forms ingestion.
- Sketch/icon D2 experiments.
- Non-blocking documentation and UX polish.

**Exit:** none of these block a release; each needs its own evidence and
opt-in flag.

## 8.1 §30 roadmap mapping

| Source roadmap item | Plan milestone |
|---|---|
| P0 coverage manifest + gate | M1 / W1 |
| P0 chapter/columns/table/diagram continuity | M1 / W2 |
| P0 structured errors + draft | M1 / W3 |
| P0 D2 20–24 nodes, decision, direction, width | M1 / W4 |
| P1 restyle + style estimate + compatible recommend | M2 / W6 |
| P1 Turkish headings + TOC reconciliation | M1 / W5 |
| P1 qa_guide + audit_guide | M2 / W7 |
| P1 tables count + page 1-based | M2 / W9/W5 |
| P1 IRAC scaffold | M2 / W8 |
| P1 diagram status schema | M2 / W4 |
| P1 doctor + table extraction MCP | M2 / W7 |
| P2 graph query/path/explain | M3 / W11 |
| P2 claims/cards filters | M2 / W12 |
| P2 documentation cookbook | M4 / W19 |
| P2 table numbering options | M2 / W9 |
| P2 subs/toc_depth | M2 / W2 |

## 9. Acceptance suite

| ID | Test | Expected |
|---|---|---|
| A1 | 141-heading source | `total=141`, `claimed=141`, `missing=[]` |
| A2 | Remove one manifest row | `missing=["..."]`, `passed=false` |
| A3 | Three-source guide | per-source + combined totals; 31-slide source nonzero |
| A4 | Chapter flow + table | kicker/title/first block together |
| A5 | Long glossary columns | split at row boundary, heading not orphaned |
| A6 | Diagram 20 nodes | valid, vertical, one page |
| A7 | `width=60%` | bbox change or `E_DIAGRAM_WIDTH` |
| A8 | Multiline label | two rendered lines |
| A9 | Underfilled TOC | `page_role=toc`, not flagged |
| A10 | Figure-only page | `figure_page` warning |
| A11 | Malformed front/table | `E_MARKDOWN` + directive stack + real line |
| A12 | Build check failure | `_draft.pdf` + `draft_path` |
| A13 | `tables` | Markdown + wrapped + captioned + glossary counts |
| A14 | Page 1 | every public report says page 1 |
| A15 | IRAC scaffold | 10 cases × 4 sections, sources in frontmatter |
| A16 | `### Sorun` | accepted or exact “use ##” error |
| A17 | `recommend irac+law` | no unsupported workbook |
| A18 | `restyle_guide` | content hash unchanged |
| A19 | `accessible` over budget | pre-build estimate warning |
| A20 | `cards` without recall | 0 + actionable warning |
| A21 | `qa_guide` | one call returns build/coverage/audit/visual/cards/claims |
| A22 | `graph query "fesih"` | section nodes |
| A23 | `links read` | from/to heading populated |
| A24 | `read_source` truncation | `page_complete=false` + next offset |
| A25 | OCR auto on text PDF | `pages_ocr=0` with reasons |
| A26 | 21-low-text guide | targeted OCR only; confidence/manual review |
| A27 | deterministic rerun | identical manifest/brief/clusters (no timestamps) |
| A28 | cost benchmark | tokens/time/calls measured; no secrets |
| A29 | hierarchical briefs | chapter/section/heading budgets; extractive provenance |
| A30 | multi-column D2 | per-container direction works or is rejected |
| A31 | `audit_guide` MCP | returns the same diagnostics as CLI `audit --strict` |
| A32 | `prepare_guide` outline | `total`, `next` and complete pagination |
| A33 | `theme` vs `style.aesthetic` | one authority documented; mismatch is rejected |
| A34 | workspace/list metadata | `guide_kind`, `case_count`, `source_count` documented |
| A35 | agent-fast profile | rerun deterministic; fast uses fewer chars/tokens than deep |
| A36 | MCP `core` profile | ≤10 tools; schema ≤5% of context budget |
| A37 | MCP profile isolation | source task works without authoring/review tools |
| A38 | resource pagination | `resources/list` cursor and bounded `resources/read` |
| A39 | output contract | `detail`/`fields`/`max_tokens`/`next`/`truncated` work |
| A40 | prompt cache stability | same profile has stable tool order; no mid-session churn |
| A41 | no raw block dump | default read returns brief/Markdown, not full block JSON |
| A42 | coordinator flow | analyze→topics→plan→brief→read→write→qa→restyle |
| A43 | 66-term glossary | renders as a four-column table, not `::: columns` |
| A44 | glossary pair atomicity | no term/definition pair splits across row/page |
| A45 | glossary continuation | heading stays with first row; header repeats |
| A46 | glossary stats | `tables.glossary` equals glossary groups and is in total |
| A47 | glossary accessibility | `TH`/`TR`/`TD` and reading order pass |

## 9.1 §34 evidence corpus and §35 smoke commands

The feedback artifacts are evidence for the benchmark, not committed fixtures.
Recreate them with synthetic or licensed sources and keep the same shapes:

| Source artifact | Plan use |
|---|---|
| 26-page main guide | layout/budget benchmark |
| 11-page IRAC workbook | IRAC/structure benchmark |
| 3 palette variants | style estimate regression |
| 4-page style previews | preview component regression |
| 8/5 cards CSV | cards regression |
| 141-row topic map | coverage manifest seed |
| 12-node legal decision tree | D2 decision benchmark |
| claims.md (146 rows) | claims precision benchmark |
| multi-source 207-page guide | coverage/multi-source benchmark |
| 21 low-text pages | OCR/visual role benchmark |

The minimal command list in §35 becomes the CLI/MCP parity smoke suite:
`analyze`, `build --check`, `audit --strict`, `diagram validate/build/status`,
`cards`, `claims`, `styles recommend`, `links build/read`, `graph build/status`.
Every command gets a matching MCP action or an explicit “CLI-only” label.

## 9.2 §36 one-week sequence

| Order | Source action | Plan |
|---:|---|---|
| 1 | Coverage manifest + 141 gate | M1/W1 |
| 2 | chapter keep-with-next + columns flow + table/diagram keep | M1/W2 |
| 3 | Diagnostic enum + audit_guide + qa_guide | M1/M2/W3/W7 |
| 4 | D2 20-node/decision/direction/width | M1/W4 |
| 5 | restyle_guide + style_estimate | M2/W6 |
| 6 | Turkish heading parser + TOC reconciliation | M1/W5 |
| 7 | page 1-based + tables count + page roles | M2/W5/W9/W10 |
| 8 | graph query/path/explain | M3/W11 |
| 9 | docs cookbook + common errors | M4/W19 |

## 10. Finding register

The register is the completeness contract. Every row from the feedback must
appear here. `Evidence` records what we know at plan time, not final status.

### 10.1 §0 — short verdict

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-0-1 | Core is good but decision points fail | W0/W19 | P0 | confirmed |
| F-0-2 | Coverage gate falsely passes 7/7 vs 141 | W1 | P0 | confirmed |
| F-0-3 | Chapter/columns/table/diagram continuity breaks | W2 | P0 | confirmed |
| F-0-4 | Errors are unstructured | W3 | P0 | confirmed |
| F-0-5 | D2 is too narrow for decision trees | W4/W16 | P0 | confirmed |
| F-0-6 | Style changes are manual | W6 | P1 | confirmed |
| F-0-7 | MCP surface is incomplete | W7 | P0/P1 | confirmed |
| F-0-8 | Multi-source synthesis exposes separate failures | W1/W14/W18 | P0/P1 | confirmed |

### 10.2 §0.1 — measurements

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-0.1-1 | 117 pages, 18 tables | W1/W9 | P0 | confirmed |
| F-0.1-2 | 139 scan headings vs 267 layout vs 141 manual vs 7 canonical | W1/W5 | P0 | confirmed |
| F-0.1-3 | 26-page guide, 3 D2, 3 counted tables (undercount), 47 bookmarks | W9/W10 | P1 | confirmed |
| F-0.1-4 | IRAC 11 pages, 5/8 cards | W8/W12 | P1 | confirmed |
| F-0.1-5 | reader+forest-sage 11, berry 11, accessible 14 (over 10–12) | W6 | P1 | confirmed |
| F-0.1-6 | Page budget oscillation 25→22→24→25→26→27→26 | W13 | P1 | confirmed |
| F-0.1-7 | Digital guide 19 pages, 207 source pages, 21 low-text, 41 canonical, 4 D2, 12 tables, 40-term glossary, 6 cards, 146 claims | W1/W9/W14/W18 | P0/P1 | confirmed |
| F-0.1-8 | “passed” is insufficient when numbers disagree | W0/W1 | P0 | confirmed |

### 10.3a §1.1 — chronological break points

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-1.1-1 | Source analysis: 117 pages, 18 tables, 139 headings; 4.4.4/10.2 missing; layout 267 | W1/W5 | P0 | confirmed |
| F-1.1-2 | Main guide: 17 sections, 3 D2, 26 pages; coverage 7/7; underfilled 4/8/19; page loop 25→...→26 | W1/W2/W13 | P0 | confirmed |
| F-1.1-3 | Dismissal diagram: 7→12 nodes; down became horizontal; width ineffective; `\n` failed | W4 | P0 | needs-repro |
| F-1.1-4 | C section: chapter kicker/title split; 66-term columns jumped; had to use a table | W2 | P0 | confirmed |
| F-1.1-5 | IRAC: wrong scaffold; `### Sorun` not counted; 7 empty sections; page 10 underfilled | W8/W10 | P1 | confirmed |
| F-1.1-6 | Style: manual config; accessible 14 pages; recommend workbook/incompatible | W6 | P1 | confirmed |
| F-1.1-7 | Digital: 207 pages, 21 low-text, 41 canonical, 31-slide source at 0, `page: —`, figures 2–3 too large | W1/W10/W13/W18 | P0 | confirmed |
| F-1.1-8 | QA: build + audit + cards + claims separate; no MCP audit/qa | W7 | P1 | confirmed |

### 10.3 §1–§2 — session and priority matrix

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-1-1 | 17-section, 26-page main guide; 3 D2; page loop | W2/W13 | P0 | confirmed |
| F-1-2 | IRAC guide, 10 events, 11 pages | W8 | P1 | confirmed |
| F-1-3 | Palette experiments require manual config | W6 | P1 | confirmed |
| F-2-1 | Coverage false pass is P0 | W1 | P0 | confirmed |
| F-2-2 | Chapter/columns/table layout splits are P0 | W2 | P0 | confirmed |
| F-2-3 | Unstructured errors and wrong line numbers are P0 | W3 | P0 | confirmed |
| F-2-4 | D2 12-node/4-depth/horizontal limits are P0 | W4 | P0 | confirmed |
| F-2-5 | Manual style changes are P1 | W6 | P1 | confirmed |
| F-2-6 | Turkish heading parsing is P0/P1 | W5 | P0 | confirmed |
| F-2-7 | 0/1-based page mix is P1 | W5 | P1 | confirmed |
| F-2-8 | Missing `tables` count is P1 | W9 | P1 | confirmed |
| F-2-9 | Audit lacks page roles | W10 | P1 | confirmed |
| F-2-10 | IRAC scaffold is wrong | W8 | P1 | confirmed |
| F-2-11 | QA tools are fragmented | W7 | P1 | confirmed |
| F-2-12 | MCP lacks audit/qa | W7 | P0/P1 | confirmed |
| F-2-13 | `build check=true` produces no draft | W3/W7 | P1 | confirmed |
| F-2-14 | `prepare_guide` outline is inconsistent | W1/W5/W7 | P1 | confirmed |
| F-2-15 | `diagram status` schema inconsistent | W4 | P2 | confirmed |
| F-2-16 | `flow` undocumented | W2/W19 | P2 | confirmed |
| F-2-17 | Links/graph lack heading labels | W11 | P2 | confirmed |
| F-2-18 | Claims/cards noisy | W12 | P2 | confirmed |
| F-2-19 | Structure validator only `##` | W8 | P1 | confirmed |
| F-2-20 | Multi-source 31-slide gap | W1/W14 | P0/P1 | confirmed |

### 10.4 §3 — coverage

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-3-1 | 139 scan headings miss `4.4.4` and `10.2` | W5 | P0 | confirmed |
| F-3-2 | Manual map adds two headings -> 141 | W1/W5 | P0 | confirmed |
| F-3-3 | Gate only checks 7 canonical sections | W1 | P0 | confirmed |
| F-3-4 | 7 claims share `body_chars: 808` | W1 | P0 | confirmed |
| F-3-5 | 141/141 in G is hand-written | W1 | P0 | confirmed |
| F-3-6 | Need `coverage_manifest` build/read/update/validate | W1 | P0 | confirmed |
| F-3-7 | Need heading override add/rename/merge/page | W1/W5 | P0 | confirmed |
| F-3-8 | Need acceptance: 141/141, missing fail, docs agree | W1 | P0 | confirmed |
| F-3-9 | `MIN_SECTION_CHARS=400` treats “empty” as a character floor | W1 | P0 | confirmed |
| F-3-10 | `coverage gate failed: 0/7 headings missing, 7 empty sections` has no diagnostic detail | W1/W3 | P0 | confirmed |

### 10.5 §4 — layout

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-4.1-1 | Chapter kicker/title split across pages | W2 | P0 | confirmed |
| F-4.1-2 | `flow=true` and `flow=false` are both insufficient | W2 | P0 | confirmed |
| F-4.1-3 | Need keep_with_next/break/allow_split | W2 | P0 | confirmed |
| F-4.1-4 | Need chapter_split audit | W2/W10 | P0 | confirmed |
| F-4.2-1 | Monolithic 66-term columns block jumps | W2 | P0 | confirmed |
| F-4.2-2 | Need columns flow and row-boundary splits | W2 | P0 | confirmed |
| F-4.2-3 | `columnbreak` new-page policy undefined | W2/W19 | P2 | policy-gap |
| F-4.3-1 | Table/diagram does not flow into remaining space | W2 | P0 | confirmed |
| F-4.3-2 | Diagram width must work or error | W2/W4 | P0 | needs-repro |
| F-4.4-1 | `::: front` + table close error points to line 2 | W3 | P0 | confirmed |
| F-4.4-2 | Need directive stack and documented nesting | W3/W19 | P0 | confirmed |

### 10.6 §5 — error model

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-5-1 | Unclosed front reports frontmatter line | W3 | P0 | confirmed |
| F-5-2 | Underfilled pages only list numbers | W3/W10 | P0 | confirmed |
| F-5-3 | Page budget error gives no per-section cause | W3/W13 | P0 | confirmed |
| F-5-4 | Structure error gives no line/level | W3/W8 | P0 | confirmed |
| F-5-5 | Coverage error gives no row/evidence | W3/W1 | P0 | confirmed |
| F-5-6 | Need ErrorCode/Diagnostic envelope | W3 | P0 | confirmed |
| F-5-7 | Need page/line/block/directive_stack/suggestions | W3 | P0 | confirmed |
| F-5-8 | Need JSON output for every error | W3/W7 | P0 | confirmed |
| F-5-9 | Coverage/empty-section failure is one prose line | W3/W1 | P0 | confirmed |

### 10.7 §6 — D2

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-6.1-1 | `MAX_NODES=12`, edges 20, depth 4 | W4 | P0 | confirmed |
| F-6.1-2 | Legal decision tree needs 15–20 nodes | W4 | P0 | confirmed |
| F-6.1-3 | Need optional two-page diagram support or explicit rejection | W4 | P1 | planned |
| F-6.2-1 | `direction=down` still produced horizontal output | W4/W16 | P0 | needs-repro |
| F-6.2-2 | Need aspect_ratio/rankdir/layout | W4/W16 | P0 | confirmed |
| F-6.3-1 | `width=60/70/90%` appeared ineffective | W4 | P0 | needs-repro |
| F-6.3-2 | Need max_width/max_height/fit or validation | W4 | P0 | confirmed |
| F-6.4-1 | `\n` in labels did not render | W4 | P1 | needs-repro |
| F-6.4-2 | Need `label_lines` alternative | W4 | P1 | confirmed |
| F-6.5-1 | `status:null` vs `state:ok` | W4 | P2 | confirmed |
| F-6.5-2 | Need unified DiagramResult and schema version | W4 | P2 | confirmed |

### 10.8 §7 — style

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-7.1-1 | `recommend irac+practice` returns workbook | W6 | P1 | confirmed |
| F-7.1-2 | Need structure/domain/page filter | W6/W19 | P1 | confirmed |
| F-7.2-1 | `accessible` produced 14 pages for 10–12 target | W6/W13 | P1 | confirmed |
| F-7.2-2 | Need style estimate before build | W6/W13 | P1 | confirmed |
| F-7.3-1 | Palette change required manual config edit | W6/W7 | P1 | confirmed |
| F-7.3-2 | Need `restyle_guide` with content hash guard | W6/W7 | P1 | confirmed |
| F-7.4-1 | `style preview` is generic | W6 | P2 | confirmed |
| F-7.4-2 | Need real components and page estimate | W6 | P2 | confirmed |

### 10.9 §8 — Turkish headings

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-8-1 | `1.İş HukukununTanımı...` glued | W5 | P0 | confirmed |
| F-8-2 | `5.2.İş SözleşmesininTürleri` glued | W5 | P0 | confirmed |
| F-8-3 | `13.4.Tutulacak Defter...` glued | W5 | P0 | confirmed |
| F-8-4 | Missing `4.4.4` and `10.2` | W5 | P0 | confirmed |
| F-8-5 | Need font/regex/TOC reconciliation | W5/W15 | P0 | confirmed |
| F-8-6 | Need `headings.review.md` and overrides | W1/W5 | P0 | confirmed |
| F-8-7 | Need Turkish normalization tests | W5 | P0 | confirmed |
| F-8-8 | Need TOC/body cross-check | W5 | P0 | confirmed |

### 10.10 §9–§12 — page index, tables, audit, IRAC

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-9-1 | Coverage section page 0 vs PDF page 1 | W5 | P1 | confirmed |
| F-9-2 | Need 1-based everywhere or explicit page_index/page_number | W5 | P1 | confirmed |
| F-10-1 | `stats.tables:3` misses Markdown/captioned tables | W9 | P1 | confirmed |
| F-10-2 | Need markdown/wrapped/captioned/total split | W9 | P1 | confirmed |
| F-11-1 | TOC page is underfilled flagged | W10 | P1 | confirmed |
| F-11-2 | Figure-only page not detected | W10 | P1 | confirmed |
| F-11-3 | Chapter split not detected | W2/W10 | P1 | confirmed |
| F-11-4 | Need page roles and exceptions | W10 | P1 | confirmed |
| F-11-5 | Need figure_only/table_only detectors | W10 | P1 | confirmed |
| F-11-6 | Replace first-three-pages heuristic with page roles | W10 | P1 | planned |
| F-12-1 | IRAC scaffold emits 60+ source headings | W8 | P1 | confirmed |
| F-12-2 | `sources: []` in scaffold | W8 | P1 | confirmed |
| F-12-3 | IRAC sections appended at end | W8 | P1 | confirmed |
| F-12-4 | `### Sorun` not accepted | W8 | P1 | confirmed |
| F-12-5 | Need structure-aware scaffold and overwrite guard | W8 | P1 | confirmed |

### 10.11 §12.1 — MCP surface

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-12.1-1 | `audit --strict` absent from MCP | W7 | P0/P1 | confirmed |
| F-12.1-2 | `doctor` absent from MCP | W7 | P1 | confirmed |
| F-12.1-3 | `qa_guide` absent | W7 | P0/P1 | confirmed |
| F-12.1-4 | `restyle_guide` absent | W6/W7 | P1 | confirmed |
| F-12.1-5 | `coverage_manifest` absent | W1/W7 | P0 | confirmed |
| F-12.1-6 | graph query/path/explain absent | W11/W7 | P2 | confirmed |
| F-12.1-7 | table extraction absent from MCP | W7/W9 | P2 | confirmed |
| F-12.1-8 | diagram build vs save/render_plan unclear | W4/W7 | P2 | confirmed |
| F-12.1-9 | style apply absent | W6/W7 | P1 | confirmed |
| F-12.1-10 | `diagram` result fields differ (`ok/cache_hit/state`) | W4 | P2 | confirmed |
| F-12.1-11 | `build_guide` uses `passed/layout_passed/genre_passed` | W3/W7 | P1 | confirmed |
| F-12.1-12 | `styles` vs `style` naming | W7 | P2 | confirmed |
| F-12.1-13 | `prepare_guide` style description says paper/clean | W7/W19 | P2 | confirmed |
| F-12.1-14 | `prepare_guide` outline only first 10, no total/next | W7 | P1 | needs-repro |
| F-12.1-15 | `covers` keys malformed | W1/W5 | P0 | confirmed |
| F-12.1-16 | layout.headings 267 vs scan 139 unexplained | W0/W1/W5 | P0 | confirmed |
| F-12.1-17 | `existing:false` no force semantics | W7/W8 | P1 | confirmed |
| F-12.1-18 | prepare vs scaffold ambiguous | W7/W19 | P2 | confirmed |
| F-12.1-19 | check=true does not produce draft | W3/W7 | P1 | confirmed |
| F-12.1-20 | check=true does not run full audit | W7/W13 | P1 | confirmed |

### 10.12 §13–§28 — workflow gaps

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-13-1 | QA requires 7 separate operations | W7 | P1 | confirmed |
| F-13-2 | Need `qa_guide` release report | W7 | P1 | confirmed |
| F-13-3 | Report needs build/coverage/audit/visual/cards/claims/diagrams fields | W7 | P1 | planned |
| F-14-1 | `flow` undocumented | W2/W19 | P2 | confirmed |
| F-14-2 | `flow=true` effect unclear | W2/W19 | P2 | confirmed |
| F-15-1 | `blocks:1` in links output unclear | W11 | P2 | confirmed |
| F-15-2 | links lack heading labels | W11 | P2 | confirmed |
| F-15-3 | confidence label not human-readable | W11 | P2 | confirmed |
| F-15-4 | graph lacks query/path/explain | W11 | P2 | confirmed |
| F-15-5 | graph lacks section/heading nodes | W11 | P2 | confirmed |
| F-16-1 | cards 0 without warning | W12 | P2 | confirmed |
| F-16-2 | claims lacks section/severity | W12 | P2 | confirmed |
| F-16-3 | short transitions counted as claims | W12 | P2 | confirmed |
| F-16-4 | empty review intervals default | W12 | P2 | confirmed |
| F-17-1 | table numbering follows figures | W9 | P2 | confirmed |
| F-17-2 | need number=false/id refs | W9 | P2 | confirmed |
| F-17-3 | need caption-only labels without table numbering | W9 | P2 | planned |
| F-18-1 | visual-audit only source, not output | W10 | P2 | confirmed |
| F-18-2 | figure-only output page not role-aware | W10 | P2 | confirmed |
| F-19-1 | style compatibility matrix absent | W6 | P2 | confirmed |
| F-19-2 | recommend offers incompatible styles | W6 | P1 | confirmed |
| F-19-3 | IRAC + law real compatible set is only `reader` and `accessible` | W6 | P1 | confirmed |
| F-20-1 | `subs=false` semantics unclear | W2/W19 | P2 | confirmed |
| F-20-2 | need toc_depth/include/exclude | W2 | P2 | confirmed |
| F-20-3 | Need explicit `TocOptions` contract | W2/W19 | P2 | planned |
| F-20-4 | Need exact `subs=false`, `toc_depth`, include/exclude docs | W2/W19 | P2 | planned |
| F-21-1 | table widths validation not explicit | W9 | P2 | needs-repro |
| F-22-1 | search returns broad/opaque results | W13 | P2 | confirmed |
| F-22-2 | `next_pages/next_offset` can repeat | W13 | P2 | confirmed |
| F-22-3 | truncation mid-page has no flag | W13 | P2 | confirmed |
| F-22-4 | Broad search can match 109 pages and return 20 silently | W13 | P2 | confirmed |
| F-23-1 | page budget gives no section costs | W13 | P2 | confirmed |
| F-23-2 | no “cut this section” suggestion | W13 | P2 | confirmed |
| F-24-1 | diagram status fields inconsistent | W4 | P2 | confirmed |
| F-25-1 | prepare/scaffold overlap | W7/W19 | P2 | confirmed |
| F-26-1 | workspace guide/source counts unclear | W7/W19 | P2 | confirmed |
| F-26-2 | IRAC section_count hides case_count | W7/W8 | P2 | confirmed |
| F-27-1 | doctor lacks style compatibility | W7 | P2 | confirmed |
| F-27-2 | doctor does not report authoritative style field | W7/W19 | P2 | confirmed |
| F-28-1 | docs lack front+table example | W19 | P2 | confirmed |
| F-28-2 | docs lack flow/subs/width/cards/tables rules | W19 | P2 | confirmed |
| F-28-3 | need authoring cookbook and tool decision tree | W19 | P2 | confirmed |
| F-28-4 | need front auto-close and columnbreak docs | W19 | P2 | confirmed |
| F-28-5 | need diagram status schema and style matrix docs | W19/W4/W6 | P2 | confirmed |
| F-28-6 | need cards-0 and table-count docs | W19/W12/W9 | P2 | confirmed |

### 10.13 §28.1 — small frictions

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-28.1-1 | `line_chars_p50:4` polluted by table cells | W10 | P2 | confirmed |
| F-28.1-2 | tiny 1.0pt font bucket unexplained | W10 | P2 | confirmed |
| F-28.1-3 | guide/source counts unclear | W7 | P2 | confirmed |
| F-28.1-4 | IRAC `section_count:54` hides 10 cases | W7/W8 | P2 | confirmed |
| F-28.1-5 | default style rationale leaks | W6 | P2 | confirmed |
| F-28.1-6 | `theme` vs `style.aesthetic` authority unclear | W19 | P2 | confirmed |
| F-28.1-7 | prepare style doc says paper/clean only | W7/W19 | P2 | confirmed |
| F-28.1-8 | diagram MCP/CLI action names differ | W7 | P2 | confirmed |
| F-28.1-9 | cards default intervals empty | W12 | P2 | confirmed |
| F-28.1-10 | claims counts transitions/questions | W12 | P2 | confirmed |
| F-28.1-11 | links confidence opaque | W11 | P2 | confirmed |
| F-28.1-12 | graph lacks heading/section nodes | W11 | P2 | confirmed |
| F-28.1-13 | read_source truncation mid-page | W13 | P2 | confirmed |
| F-28.1-14 | search has no rank/heading | W13 | P2 | confirmed |
| F-28.1-15 | table widths error unclear | W9 | P2 | needs-repro |
| F-28.1-16 | TOC underfilled exception only first pages | W10 | P2 | confirmed |
| F-28.1-17 | front+table nesting undocumented | W19 | P2 | confirmed |
| F-28.1-18 | flow/subs undocumented | W19 | P2 | confirmed |
| F-28.1-19 | caption numbering cannot be disabled | W9 | P2 | confirmed |
| F-28.1-20 | tables count only wrapped tables | W9 | P2 | confirmed |
| F-28.1-21 | build check no draft | W3 | P1 | confirmed |

### 10.14 §29–§33 — strengths, root causes, code suggestions

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-29-1 | Preserve audit strict | W10/W19 | P0 | confirmed |
| F-29-2 | Preserve D2 deterministic build | W4/W19 | P0 | confirmed |
| F-29-3 | Preserve bounded source reads | W13 | P0 | confirmed |
| F-29-4 | Preserve claims/cards | W12 | P1 | confirmed |
| F-29-5 | Preserve doctor | W7 | P2 | confirmed |
| F-29-6 | Preserve list/workspace info | W7 | P2 | confirmed |
| F-29-7 | Preserve style preview concept | W6 | P2 | confirmed |
| F-29-8 | Preserve local model/D2 | W17/W19 | P0 | confirmed |
| F-29-9 | Preserve fast build feedback | W3 | P1 | confirmed |
| F-29-10 | Preserve visual-audit idea | W10 | P1 | confirmed |
| F-29-11 | Preserve multi-source capability | W1/W14/W18 | P0 | confirmed |
| F-29.1-1 | Coverage root cause: source_candidates only top level | W1 | P0 | confirmed |
| F-29.1-2 | Claims bind shared body | W1 | P0 | confirmed |
| F-29.1-3 | Empty section is 400 chars | W1 | P0 | confirmed |
| F-29.1-4 | Chapter block atomic/no keep-with-next | W2 | P0 | confirmed |
| F-29.1-5 | Columns monolithic | W2 | P0 | confirmed |
| F-29.1-6 | TOC underfilled role missing | W10 | P1 | confirmed |
| F-29.1-7 | Figure-only not detected | W10 | P1 | confirmed |
| F-29.1-8 | D2 limits | W4 | P0 | confirmed |
| F-29.1-9 | TALA aspect ratio | W4/W16 | P0 | needs-repro |
| F-29.1-10 | Width path | W4 | P0 | needs-repro |
| F-29.1-11 | D2 newline escape | W4 | P1 | needs-repro |
| F-29.1-12 | Diagram result structs differ | W4 | P2 | confirmed |
| F-29.1-13 | Style recommendation lacks filter | W6 | P1 | confirmed |
| F-29.1-14 | No style page estimate | W6/W13 | P1 | confirmed |
| F-29.1-15 | IRAC scaffold generic | W8 | P1 | confirmed |
| F-29.1-16 | Page origin mixed | W5 | P1 | confirmed |
| F-29.1-17 | Table stats wrapped-only | W9 | P1 | confirmed |
| F-29.1-18 | Error strings | W3 | P0 | confirmed |
| F-29.1-19 | MCP audit not registered | W7 | P0 | confirmed |
| F-29.2-1 | HeadingManifest contract | W1 | P0 | planned |
| F-29.2-2 | Diagnostic/ErrorCode | W3 | P0 | planned |
| F-29.2-3 | PageRole | W10 | P1 | planned |
| F-29.2-4 | ChapterOptions | W2 | P0 | planned |
| F-29.2-5 | ColumnsOptions | W2 | P0 | planned |
| F-29.2-6 | DiagramOptions | W4 | P0 | planned |
| F-29.2-7 | QaReport | W7 | P1 | planned |
| F-29.2-8 | StyleApply | W6 | P1 | planned |
| F-29.2-9 | GraphQuery | W11 | P2 | planned |
| F-30-1 | P0 roadmap: manifest/layout/diagnostics/D2 | W1/W2/W3/W4 | P0 | planned |
| F-30-2 | P1 roadmap: style/heading/QA/counts/IRAC/schema/doctor | W6/W5/W7/W9/W8 | P1 | planned |
| F-30-3 | P2 roadmap: graph/claims/docs/numbering/subs | W11/W12/W19/W9/W2 | P2 | planned |
| F-31-1 | API surface list | W7/W14 | P1 | planned |
| F-32-1 | Acceptance suite list | §9 | P0/P1 | planned |
| F-33-1 | Trust trio: manifest/layout/D2 | W1/W2/W4 | P0 | confirmed |

### 10.15 §34–§41 — files, commands, agent economics, engine depth

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-34-1 | Reproduced output/guide file inventory | W0 | P2 | confirmed |
| F-35-1 | Minimal command replay | W19 | P2 | confirmed |
| F-36-1 | One-week priority list | W0/W1/W2/W3/W4 | P0 | planned |
| F-37-1 | Folisift is a backend, not yet agent-first | W14 | P0 | confirmed |
| F-37-2 | PDF/render/audit saved; source understanding still expensive | W14/W15 | P0 | confirmed |
| F-37-3 | Coverage manual | W1 | P0 | confirmed |
| F-37-4 | Layout iteration expensive | W2/W13 | P0 | confirmed |
| F-37-5 | Style trial needs full build | W6/W13 | P1 | confirmed |
| F-37-6 | MCP outputs too large | W14/W7 | P1 | confirmed |
| F-37-7 | No token budget/visibility | W0/W14 | P1 | confirmed |
| F-37-8 | No single-call QA | W7 | P1 | confirmed |
| F-37-9 | Need read_section/section_brief/guide_plan | W14 | P0 | planned |
| F-37-10 | Need qa_guide, draft, manifest, style estimate | W7/W3/W1/W6 | P0/P1 | planned |
| F-37-11 | Need restyle/diagram_fit/citation/verify/incremental/cost/effort | W6/W4/W14/W0 | P1 | planned |
| F-37-12 | Success metrics defined | W0 | P1 | planned |
| F-37-13 | Ideal flow analyze→brief→plan→write→build→qa→restyle | W14/W7 | P0 | planned |
| F-38-1 | Model2Vec links worked locally | W17 | P1 | preserve |
| F-38-2 | Preserve the six useful legal bridges as review-only examples | W17/W11 | P1 | preserve |
| F-38-3 | Some bridges useful, manually verified | W17/W11 | P1 | preserve |
| F-38-4 | Similarity is not meaning/coverage | W17/W1 | P0 | confirmed |
| F-38-5 | Need heading labels/priority/clusters/coverage hints | W17/W11/W1 | P1 | planned |
| F-38-6 | Use as router, not reader | W14/W17 | P0 | planned |
| F-39-1 | OCR not useful for text-layer PDF | W18 | P1 | confirmed |
| F-39-2 | OCR useful for scanned/low-text/table/regions only | W18/W15 | P1 | confirmed |
| F-39-3 | Turkish OCR/number risk | W18 | P1 | confirmed |
| F-39-4 | Need auto OCR policy/report | W18 | P1 | planned |
| F-39-5 | OCR does not solve understanding/layout | W18/W14 | P0 | confirmed |
| F-40-1 | 207-page, 3-source synthesis passed | W1/W14 | P0 | preserve |
| F-40-2 | Heterogeneous granularity/terminology | W14/W15 | P1 | confirmed |
| F-40-3 | 21 low-text pages visual audit | W10/W18 | P1 | confirmed |
| F-40-4 | 31-slide source at 0 canonical | W1 | P0 | confirmed |
| F-40-5 | Product report pages `page: —` | W1/W5/W15 | P0 | confirmed |
| F-40-6 | Shared `body_chars` | W1 | P0 | confirmed |
| F-40-7 | Figure size/page budget | W2/W4/W13 | P1 | confirmed |
| F-40-8 | Targeted OCR/vision for low-text | W18/W15 | P1 | planned |
| F-40-9 | source_brief/visual_brief/source_topics/figure_fit/page_budget | W14/W18/W4/W13 | P1 | planned |
| F-41-1 | LiteParse `is_complex` unused | W15 | P1 | confirmed |
| F-41-2 | text metadata/vector/image/structure tree unused | W15 | P1 | confirmed |
| F-41-3 | table cell/bbox not exposed in MCP | W15/W9/W7 | P1 | confirmed |
| F-41-4 | D2 dagre/elk/per-container direction/near/top/left unused | W16 | P1 | confirmed |
| F-41-5 | D2 classes/tooltip/link/grid/shapes unused | W16 | P1 | planned |
| F-41-6 | Model2Vec clustering/classification/retrieval unused | W17 | P1 | confirmed |
| F-41-7 | FastEmbed reranker unused | W17 | P2 | conditional |
| F-41-8 | agent-fast profile | W14/W15/W16/W17/W18 | P1 | planned |
| F-41-9 | determinism guardrails | W19 | P0 | planned |
| F-41-10 | avoid full OCR/LLM summaries/sketch/always-reranker | W18/W14/W16/W17 | P1 | planned |
| F-41-11 | capability priority order | W15/W16/W17/W9 | P1 | planned |
| F-41-12 | Context7 sources | W19 | P2 | reference |

### 10.16 Detailed §37–§41 sub-findings

The following rows expand the grouped register entries so the agent-economics,
semantic-link, OCR and engine-capability findings are not lost.

#### §37.3 — where the system does not save cost

| ID | Finding | Workstream | Priority |
|---|---|---|---|
| F-37.3-1 | Reading 117 pages still requires 6–8 large chunks | W14/W15 | P0 |
| F-37.3-2 | No section-level extractive brief | W14 | P0 |
| F-37.3-3 | 141-row coverage matrix written by hand | W1 | P0 |
| F-37.3-4 | Layout debugging requires PDF render + visual inspection | W2/W10/W13 | P0 |
| F-37.3-5 | Style trials require full build | W6/W13 | P1 |
| F-37.3-6 | MCP responses are too large and lack detail modes | W7/W14 | P1 |
| F-37.3-7 | No token budget or usage visibility | W0/W14 | P1 |
| F-37.3-8 | No single-call QA gate | W7 | P1 |

#### §37.2 — where it already saves

| ID | Finding | Workstream | Priority |
|---|---|---|---|
| F-37.2-1 | PDF text extraction avoided | W15 | P1 |
| F-37.2-2 | Page-by-page visual reading reduced | W10/W18 | P1 |
| F-37.2-3 | PDF production avoided | W2/W19 | P1 |
| F-37.2-4 | Layout audit automated | W10 | P1 |
| F-37.2-5 | D2 schema validation/build automated | W4 | P1 |
| F-37.2-6 | Card/claim export automated | W12 | P2 |
| F-37.2-7 | Source reference lookup automated | W13 | P2 |
| F-37.2-8 | Style discovery preview automated | W6 | P2 |

#### §37.4 — cost table

| ID | Stage | Workstream | Priority |
|---|---|---|---|
| F-37.4-1 | Source analysis remains expensive | W14/W15 | P0 |
| F-37.4-2 | Coverage completion is very expensive | W1 | P0 |
| F-37.4-3 | Writing 7,000 words remains unavoidable | W14 | P1 |
| F-37.4-4 | Layout visual checking is expensive | W2/W10 | P0 |
| F-37.4-5 | D2 redesign is medium/high cost | W4/W16 | P1 |
| F-37.4-6 | Style trial is medium cost | W6 | P1 |
| F-37.4-7 | QA interpretation is medium cost | W7 | P1 |
| F-37.4-8 | Cards/claims are already good | W12 | P2 |

#### §37.5 — agent-first recommendations

| ID | Recommendation | Workstream | Priority |
|---|---|---|---|
| F-37.5-1 | `read_section` by heading, not page | W14 | P0 |
| F-37.5-2 | `section_brief` 300–600 tokens | W14 | P0 |
| F-37.5-3 | `guide_plan` with word/page budget | W14 | P0 |
| F-37.5-4 | `qa_guide` compact single call | W7 | P0 |
| F-37.5-5 | Draft PDF on failed check | W3/W7 | P1 |
| F-37.5-6 | `coverage_manifest` automatic rows | W1 | P0 |
| F-37.5-7 | `style_estimate` before build | W6 | P1 |
| F-37.5-8 | `restyle_guide` with content hash guard | W6 | P1 |
| F-37.5-9 | `diagram_fit` page estimate | W4 | P1 |
| F-37.5-10 | `citation_snippet` exact source sentence | W14 | P1 |
| F-37.5-11 | `verify_citations` claim↔page | W12 | P1 |
| F-37.5-12 | `incremental_analysis` by section hash; PDF layout global | W13 | P2 |
| F-37.5-13 | `cost_report` per run | W0 | P1 |
| F-37.5-14 | `effort`/`token_budget` on prepare | W14 | P1 |

#### §37.6 — success metrics

| ID | Metric | Workstream | Priority |
|---|---|---|---|
| F-37.6-1 | `time_to_first_draft` | W0 | P1 |
| F-37.6-2 | `time_to_pass` | W0 | P1 |
| F-37.6-3 | `tool_calls_per_section` | W0/W14 | P1 |
| F-37.6-4 | `tokens_in/tokens_out` | W0/W14 | P1 |
| F-37.6-5 | `source_chars_read/source_chars_total` | W0/W14 | P1 |
| F-37.6-6 | `manual_fix_count` | W0 | P1 |
| F-37.6-7 | `build_iterations` | W0/W13 | P1 |
| F-37.6-8 | `coverage_manual_rows` | W0/W1 | P0 |
| F-37.6-9 | `visual_page_reads` | W0/W10 | P1 |

#### §37.7 — ideal flow

| ID | Finding | Workstream | Priority |
|---|---|---|---|
| F-37.7-1 | analyze → section_brief → guide_plan → write → build draft → qa → restyle | W14/W7 | P0 |
| F-37.7-2 | Agent should not read all pages page-by-page | W14 | P0 |
| F-37.7-3 | Agent should drill down only when needed | W14 | P0 |

#### §38.2 — Model2Vec gaps

| ID | Finding | Workstream | Priority |
|---|---|---|---|
| F-38.2-1 | No summary | W14/W17 | P1 |
| F-38.2-2 | No heading label | W11/W17 | P1 |
| F-38.2-3 | No priority/importance | W17 | P1 |
| F-38.2-4 | No cluster/topic map | W17 | P1 |
| F-38.2-5 | No coverage contribution | W1/W17 | P1 |
| F-38.2-6 | No contradiction detection | W14/W17 | P2 |
| F-38.2-7 | No “read next” queue | W17 | P2 |
| F-38.2-8 | No explanation for similarity | W11/W17 | P2 |

#### §38.3 — Model2Vec cost effect

| ID | Finding | Workstream | Priority |
|---|---|---|---|
| F-38.3-1 | Source reading impact near zero | W14 | P1 |
| F-38.3-2 | Cross-section bridge impact medium | W11/W17 | P1 |
| F-38.3-3 | F-section writing impact medium | W11 | P2 |
| F-38.3-4 | Coverage impact zero | W1 | P1 |
| F-38.3-5 | Layout impact zero | W2 | P2 |
| F-38.3-6 | Token impact low overhead, low saving | W0/W14 | P1 |
| F-38.3-7 | Trust still needs manual verification | W11/W19 | P1 |

#### §38.4 — better Model2Vec uses

| ID | Use | Workstream | Priority |
|---|---|---|---|
| F-38.4-1 | `source_topics` clustering | W17 | P1 |
| F-38.4-2 | `section_brief` top-k retrieval | W14/W17 | P0 |
| F-38.4-3 | `coverage_hint` classification | W1/W17 | P1 |
| F-38.4-4 | `query_source` semantic search | W14 | P1 |
| F-38.4-5 | `links read` headings/why | W11 | P1 |
| F-38.4-6 | `cluster_map` graph | W16/W17 | P2 |
| F-38.4-7 | `read_next` queue | W17 | P2 |

#### §39.2–§39.4 — OCR decision

| ID | Finding | Workstream | Priority |
|---|---|---|---|
| F-39.2-1 | Scanned PDF/no text layer: high OCR benefit | W18 | P1 |
| F-39.2-2 | Low-text pages: high benefit | W18 | P1 |
| F-39.2-3 | Table/image text: medium | W18/W15 | P1 |
| F-39.2-4 | Handwriting/stamp/signature: medium/low | W18 | P2 |
| F-39.2-5 | Encoding/missing glyph: medium | W18 | P2 |
| F-39.2-6 | Rotated/skewed page: medium | W18 | P2 |
| F-39.2-7 | Text-layer source: no benefit | W18 | P1 |
| F-39.3-1 | OCR duplicates existing text | W18 | P1 |
| F-39.3-2 | Turkish `İ/ı/ş/ğ/ç` risk | W18 | P1 |
| F-39.3-3 | Legal numbers/dates risk | W18 | P1 |
| F-39.3-4 | Table structure risk | W18/W15 | P1 |
| F-39.3-5 | CPU/time cost | W0/W18 | P1 |
| F-39.3-6 | OCR does not solve heading/coverage/layout/D2 | W18/W1/W2/W4 | P0 |
| F-39.4-1 | Target low-text pages only | W18 | P1 |
| F-39.4-2 | Table/figure region OCR | W18/W15 | P1 |
| F-39.4-3 | Compare OCR with text layer | W18 | P1 |
| F-39.4-4 | Turkish dictionary/number validation | W18 | P1 |
| F-39.4-5 | OCR only missing pages | W18 | P1 |
| F-39.5-1 | `ocr:auto`, `low_text_pages_only`, `min_chars_per_page` policy | W18 | P1 |
| F-39.5-2 | `table_regions`, `language:tr`, confidence/manual-review report | W18 | P1 |
| F-39.5-3 | Merge OCR with text layer and deduplicate | W18 | P1 |
| F-39.6-1 | OCR is pixel-to-text, not comprehension | W18/W14 | P0 |

#### §40.1–§40.8 — multi-source digital guide

| ID | Finding | Workstream | Priority |
|---|---|---|---|
| F-40.1-1 | Three sources mix slide/book/report granularity | W14/W15 | P1 |
| F-40.1-2 | Remove repeated definitions | W14 | P1 |
| F-40.1-3 | Preserve source order and topic unity | W14 | P1 |
| F-40.1-4 | Unify terminology/abbreviations | W14/W17 | P1 |
| F-40.1-5 | Preserve within-source differences | W14 | P0 |
| F-40.1-7 | Preserve DPP 2026 vs 18 February 2027 distinction | W14 | P0 |
| F-40.1-6 | Convert visual slides to text/table | W18/W15 | P1 |
| F-40.2-1 | 21 low-text pages require visual audit | W10/W18 | P1 |
| F-40.2-2 | Most are decorative/dividers | W18/W10 | P2 |
| F-40.2-3 | OCR cannot decide exam relevance | W18/W10 | P1 |
| F-40.2-4 | 21 low-text distribution is 11 + 8 + 2 | W10/W18 | P1 |
| F-40.3-1 | 31-slide source has 0 canonical sections | W1 | P0 |
| F-40.3-2 | 14 product sections have `page: —` | W1/W5/W15 | P0 |
| F-40.3-3 | `body_chars` repeated across headings | W1 | P0 |
| F-40.3-4 | 41/41 pass is misleading | W1 | P0 |
| F-40.4-1 | Figures 2–3 too large | W2/W4/W13 | P1 |
| F-40.4-2 | Cover/title spacing issue | W2/W6 | P2 |
| F-40.4-3 | Four D2 fit/page pressure | W4/W13 | P1 |
| F-40.4-4 | 12 tables / one converted from low-text page | W9/W15 | P1 |
| F-40.5-1 | Slide page = PDF page assumed | W1/W5 | P1 |
| F-40.5-2 | Book pages 40–122 mapped | W1/W5 | P1 |
| F-40.5-3 | Product report page mapping missing | W1/W5/W15 | P0 |
| F-40.6-1 | Targeted OCR/vision may help 21 pages | W18/W15 | P1 |
| F-40.7-1 | `source_brief` per source/section | W14 | P1 |
| F-40.7-2 | `visual_brief` role + OCR + recommendation | W18/W10 | P1 |
| F-40.7-3 | `coverage_manifest` per source | W1 | P0 |
| F-40.7-4 | `source_page_map` for `page: —` | W1/W5 | P0 |
| F-40.7-5 | `source_topics` cross-source clusters | W17 | P1 |
| F-40.7-6 | `figure_fit` before build | W4/W13 | P1 |
| F-40.7-7 | `page_budget` per section | W13 | P1 |
| F-40.8-1 | Multi-source synthesis is more expensive | W0/W14 | P1 |
| F-40.8-2 | Low-text pages force visual audit | W18/W10 | P1 |
| F-40.8-3 | Coverage must see every source | W1 | P0 |
| F-40.8-4 | Missing page map weakens citations | W1/W5 | P0 |
| F-40.8-5 | Targeted OCR/vision, not full OCR | W18 | P1 |

#### §41.1–§41.8 — unused engine capabilities

| ID | Capability/finding | Workstream | Priority |
|---|---|---|---|
| F-41.1-1 | LiteParse `is_complex` unused | W15 | P1 |
| F-41.1-2 | `extract_text_metadata` unused | W15/W5 | P1 |
| F-41.1-3 | `extract_vector_graphics` unused | W15/W10 | P1 |
| F-41.1-4 | `extract_images` unused | W15/W10 | P1 |
| F-41.1-5 | `extract_structure_tree` unused | W15/W5 | P1 |
| F-41.1-6 | `target_pages` unused | W15/W18 | P1 |
| F-41.1-7 | `ocr_language="tur"` unused | W18 | P1 |
| F-41.1-8 | `output_format="markdown"` unused | W15 | P2 |
| F-41.1-9 | `continue_on_page_error`/`page_errors` unused | W15 | P1 |
| F-41.1-10 | `keep_headers_footers` policy absent | W15 | P2 |
| F-41.1-11 | `extract_annotations`/forms unused | W15 | P3 |
| F-41.1-12 | `preserve_very_small_text` unused | W15 | P2 |
| F-41.1-13 | table cell/bbox not in MCP | W15/W9/W7 | P1 |
| F-41.1-14 | `extract_blocks` is used but table cells not surfaced | W15/W9 | P1 |
| F-41.1-15 | `extract_content_bounds` only partially used | W15/W10 | P1 | needs-repro |
| F-41.1-16 | `ocr_server_url` unused | W18 | P2 |
| F-41.1-17 | `dpi`/`num_workers` OCR cost controls unused | W18/W0 | P2 |
| F-41.2-1 | 18/12 tables not cell-verified | W15/W9 | P1 |
| F-41.3-1 | D2 dagre/elk unused | W16 | P1 |
| F-41.3-2 | per-container direction unused | W16 | P1 |
| F-41.3-3 | node/container width/height unused | W16 | P1 |
| F-41.3-4 | `near`/`top`/`left` unused | W16 | P1 |
| F-41.3-5 | `classes` styling unused | W16 | P1 |
| F-41.3-6 | `tooltip`/`link` unused | W16 | P2 |
| F-41.3-7 | `grid` unused | W16 | P2 |
| F-41.3-8 | sequence/class/sql_table limited | W16 | P2 |
| F-41.3-9 | `vars`/`imports`/`theme` unused | W16 | P2 |
| F-41.3-10 | sketch/icons unused | W16 | P3 |
| F-41.4-1 | Model2Vec clustering unused | W17 | P1 |
| F-41.4-2 | classification unused | W17 | P1 |
| F-41.4-3 | retrieval only pairwise | W14/W17 | P1 |
| F-41.4-4 | `encode_as_sequence` unused | W17 | P2 |
| F-41.4-5 | `max_length`/truncation unused | W17 | P1 |
| F-41.4-6 | int8/PCA/SIF controls: PCA/SIF are distillation-time, not runtime flags | W17 | P2 | policy-gap |
| F-41.4-7 | reranking unused | W17 | P2 |
| F-41.5-1 | FastEmbed-rs reranker not evaluated | W17 | P2 |
| F-41.5-2 | Offline `TextRerank`/user-defined model path not tested | W17 | P2 | needs-repro |
| F-41.5-3 | Reranker latency/model cost not measured | W0/W17 | P2 |
| F-41.6-1 | agent-fast profile not implemented | W14/W15/W16/W17/W18 | P1 |
| F-41.6.1-1 | LiteParse version in report | W19 | P0 |
| F-41.6.1-2 | D2 version in report | W19 | P0 |
| F-41.6.1-3 | model hash in report | W19 | P0 |
| F-41.6.1-4 | layout engine in report | W19 | P0 |
| F-41.6.1-5 | OCR version/language in report | W19 | P1 |
| F-41.6.1-6 | extractive brief deterministic tie-break | W14/W19 | P0 |
| F-41.6.1-7 | cache key source/model/policy | W19 | P0 |
| F-41.6.1-8 | D2 layout/model pinned | W19 | P0 |
| F-41.7-1 | Avoid full OCR | W18 | P1 |
| F-41.7-2 | Avoid generative summaries in deterministic path | W14 | P1 |
| F-41.7-3 | Avoid sketch/icons in legal guides | W16 | P2 |
| F-41.7-4 | Avoid always-on reranker | W17 | P2 |
| F-41.7-5 | Avoid heavy metadata on every source | W15 | P2 |
| F-41.8-1 | Priority: LiteParse complexity/metadata/vector | W15 | P1 |
| F-41.8-2 | Priority: D2 groups/direction/near/classes | W16 | P1 |
| F-41.8-3 | Priority: Model2Vec cluster/classify/retrieve | W17 | P1 |
| F-41.8-4 | Priority: LiteParse table cells | W15/W9 | P1 |
| F-41.8-5 | Priority: conditional FastEmbed reranker | W17 | P2 |


### 10.17 §42 — MCP context architecture research

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-42-1 | MCP clients should expose progressive tool discovery, not full catalog upfront | W21/W7 | P0 | confirmed |
| F-42-2 | MCP example: ~150k tokens of definitions vs ~2k on demand | W0/W21 | P1 | confirmed |
| F-42-3 | MCP pagination uses opaque cursors for list/read surfaces | W21 | P0 | confirmed |
| F-42-4 | Programmatic tool calling keeps intermediates out of context | W21/W14 | P1 | confirmed |
| F-42-5 | Prompt caching requires a stable tools array | W21 | P0 | confirmed |
| F-42-6 | Context7 uses resolve→query and one concept per query | W14/W21 | P1 | confirmed |
| F-42-7 | GitHub MCP uses toolsets, dynamic toolsets, read-only and `fields` | W7/W21 | P0 | confirmed |
| F-42-8 | Sentry separates `search_issues` list from detail resource | W14/W21 | P1 | confirmed |
| F-42-9 | Notion prefers Markdown page reads and cursor-paginated blocks | W14/W21 | P1 | confirmed |
| F-42-10 | Playwright bounds snapshots by target/depth/file and can omit images | W21 | P1 | confirmed |
| F-42-11 | Linear uses `first`/`after` cursor and `pageInfo` | W21 | P1 | confirmed |
| F-42-12 | Community reports tool definitions consuming 30–50% of context and tool confusion above ~50 tools | W0/W21 | P1 | confirmed |
| F-42-13 | Folisift future surface exceeds 30 tools without profiles | W7/W21 | P0 | planned |
| F-42-14 | Folisift needs MCP resources for guide/source/coverage reads | W21 | P0 | planned |
| F-42-15 | Folisift needs chapter→section→heading brief hierarchy | W14/W21 | P0 | planned |
| F-42-16 | Folisift needs `detail`/`fields`/`max_tokens`/cursor output contract | W14/W21 | P0 | planned |
| F-42-17 | Folisift needs a coordinator flow, not a single mega-tool | W21 | P0 | planned |
| F-42-18 | Folisift needs context/tool metrics in `cost_report` | W0/W21 | P1 | planned |
| F-42-19 | Folisift must keep a stable profile/tool order per session | W21 | P0 | planned |
| F-42-20 | Folisift must not return raw block JSON or full source by default | W21/W14 | P0 | planned |
| F-42-21 | `resources/list`/`resources/templates/list` need cursor pagination | W21 | P1 | planned |
| F-42-22 | Large outputs should go to file/resource with a path+summary | W21 | P1 | planned |
| F-42-23 | Profile schema budget target is ≤5% of context and ≤15 tools | W21 | P0 | planned |
| F-42-24 | Benchmark must compare all-tools baseline vs profile+resources | W0/W21 | P1 | planned |


### 10.18 §43 — glossary layout decision

| ID | Finding | Workstream | Priority | Evidence |
|---|---|---|---|---|
| F-43-1 | Two-column glossary block jumps as one monolithic unit | W2/W22 | P1 | confirmed |
| F-43-2 | 66-term İş Hukuku glossary improved after conversion to a four-column table | W22 | P1 | confirmed |
| F-43-3 | Four-column glossary becomes the default glossary layout | W22 | P1 | planned |
| F-43-4 | Two term/definition pairs per row: Terim + Tanım + Terim + Tanım | W22 | P1 | planned |
| F-43-5 | Default widths `18,32,18,32`; pair is atomic | W22 | P1 | planned |
| F-43-6 | Break only at row boundaries; repeat header on continuation | W22 | P1 | planned |
| F-43-7 | Heading must keep with the first glossary row | W2/W22 | P1 | planned |
| F-43-8 | Preserve source references inside definition cells | W22 | P0 | planned |
| F-43-9 | Odd term count: one pair plus empty/spanning cells, no split | W22 | P2 | planned |
| F-43-10 | Tagged table structure and reading order required | W22/W10 | P1 | planned |
| F-43-11 | Count `tables.glossary` separately and include in total | W9/W22 | P1 | planned |
| F-43-12 | Legacy `::: columns` glossaries migrate or warn | W22/W19 | P1 | planned |
| F-43-13 | Glossary table affects page budget; report it | W13/W22 | P2 | planned |
| F-43-14 | Cookbook must show four-column and odd-term examples | W19/W22 | P2 | planned |


## 11. Open questions and risks

1. **Coverage semantics.** Is a “heading” every layout heading block, every
   TOC row, or a reviewed topic? The plan chooses reviewed manifest rows as
   the gate and keeps the other counts as evidence.
2. **D2 width repro.** The current code path passes width, but the production
   report says it did not change size. M1 includes a bbox regression test;
   if it fails, fix or reject the attribute.
3. **Direction under TALA.** TALA may legitimately choose aspect ratio. The
   product must either expose the chosen layout or offer ELK/dagre; it must not
   promise a direction it cannot honor.
4. **OCR cost.** Auto OCR must be measured; low-text pages can be expensive.
   Default stays off for text-layer PDFs.
5. **Extractive briefs.** They are deterministic but less fluent than an LLM
   summary. The host may still ask the LLM to rewrite, but then the source
   context is already compressed.
6. **Schema compatibility.** New MCP tools require a schema bump; old clients
   keep aliases for one minor release.
7. **Cost claim.** No release note may claim token savings until W0 benchmark
   passes. Architecture alone is not evidence.
8. **Host profile support.** If the host cannot run separate MCP profiles, the
   fallback is a single `core` server with ≤10 stable tools; resources still
   carry the variable data. A giant mega-tool is not the fallback.
9. **Prompt-cache invalidation.** Any per-turn tool enable/disable can cost more
   than it saves. Profiles are selected at session start, not changed mid-task.
10. **Tokenizer variance.** Token budgets are approximate across providers; the
    report records the tokenizer/estimate used and never presents an estimate as
    a provider bill.

## 12. Non-goals

- No cloud dependency, API key, LLM call, shell execution or network access in
  the MCP server.
- No full-PDF OCR by default.
- No generative summarization inside the deterministic brief pipeline.
- No graph database or hosted service.
- No claim that a similarity link is a verified fact or coverage proof.
- No silent overwrite of an existing guide.
- No “passed” result without counts and diagnostic evidence.

## 13. Definition of done

The feedback is closed when:

- every register row has a passing test or an explicit accepted deferral;
- the 141-heading source gate is exact;
- chapter/columns/table/diagram continuity has regression tests;
- every failure is machine-readable and actionable;
- D2 can express a 20-node decision tree with a deterministic layout;
- one `qa_guide` call is the release gate;
- MCP profiles keep the active surface at or below 15 tools and tool schemas at
  or below 5% of the context budget;
- resources carry page/section/coverage reads; tools carry actions;
- glossaries use the four-column table with atomic term/definition pairs and
  correct table tagging;
- `section_brief`/`read_section`/`source_topics` reduce measured source
  reading without losing coverage;
- LiteParse/D2/Model2Vec advanced capabilities are opt-in, measured and
  deterministic;
- the release report includes counts, audit, cost/time and remaining risks.

## 14. Feedback section coverage map

This table proves that every section of the source report is represented in the
finding register.

| Feedback section | Register |
|---|---|
| §0 short verdict | §10.1 |
| §0.1 measurements | §10.2 |
| §1 session flow | §10.3 |
| §1.1 chronology | §10.3a |
| §2 priority matrix | §10.3 |
| §3 coverage | §10.4 |
| §4 layout | §10.5 |
| §5 error model | §10.6 |
| §6 D2 | §10.7 |
| §7 style | §10.8 |
| §8 Turkish headings | §10.9 |
| §9 page indexing | §10.10 |
| §10 tables | §10.10 |
| §11 audit roles | §10.10 |
| §12 IRAC | §10.10 |
| §12.1 MCP | §10.11 |
| §13 QA | §10.12 |
| §14 flow docs | §10.12 |
| §15 links/graph | §10.12 |
| §16 claims/cards | §10.12 |
| §17 numbering | §10.12 |
| §18 visual audit | §10.12 |
| §19 style matrix | §10.12 |
| §20 subs | §10.12 |
| §21 table widths | §10.12 |
| §22 read/search | §10.12 |
| §23 page budget | §10.12 |
| §24 diagram status | §10.12 |
| §25 prepare/scaffold | §10.12 |
| §26 workspace/list | §10.12 |
| §27 doctor | §10.12 |
| §28 docs gaps | §10.12 |
| §28.1 frictions | §10.13 |
| §29 strengths | §10.14 |
| §29.1 root causes | §10.14 |
| §29.2 code suggestions | §10.14 |
| §30 roadmap | §10.14 |
| §31 API | §10.14 / §6 |
| §32 tests | §10.14 / §9 |
| §33 final note | §10.14 |
| §34 files | §10.15 |
| §35 commands | §10.15 |
| §36 one-week list | §10.15 |
| §37 agent economics | §10.15 / §10.16 |
| §38 Model2Vec | §10.15 / §10.16 |
| §39 OCR | §10.15 / §10.16 |
| §40 multi-source | §10.15 / §10.16 |
| §41 engine capabilities | §10.15 / §10.16 |
| §42 MCP context architecture research | §10.17 |
| §43 glossary layout decision | §10.18 |
