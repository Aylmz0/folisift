# Production feedback v2 remediation plan — 2026-11-04

- **Status:** Completed through v0.19.0; external A-V2 acceptance passes
- **Source:** [`docs/FOLISIFT-GERI-BILDIRIM-v2.md`](../FOLISIFT-GERI-BILDIRIM-v2.md)
- **Previous plan:** [`2026-11-03-production-feedback-remediation.md`](2026-11-03-production-feedback-remediation.md)
- **Baseline:** Folisift 0.17.1, build 185, MCP schema 13, 35 tools
- **Purpose:** turn the second production feedback into an owned, testable
  remediation map without weakening the deterministic/local boundary
- **Primary outcome:** 141/141 and 41/41 coverage gates pass through the
  reviewed manifest, context briefs obey their token budgets, semantic topics
  are real, every metric agrees across tools, and the release gate proves it
  on the four-source production workspace.

## 0. Decision

v0.17.1 is a strong release and is **not** being reopened wholesale. The
feedback says 8.0/10; this plan treats the 12 remaining items as the complete
scope for 10/10.

- **P0 hotfix — 0.17.2:** coverage truth, brief budgets, heading matching,
  semantic topics, claims/graph budgets, guide metrics, citation verification.
- **P1 — 0.18:** style estimate, visual-audit taxonomy, cards/QA parity, IRAC
  scaffold, source-heading completeness, D2 discovery, QA cost report.
- **P2 — 1.0:** D2 decision depth/width, LiteParse insight integration, OCR
  policy, bounded graph path/explain, documentation and determinism manifests.

The plan is complete only when every register row is `verified` or explicitly
deferred, and every acceptance test in §6 passes on the real workspace and the
five repository guides.

## 0.1 Baseline evidence — v0.17.1 test matrix

`PASS` rows are regression contracts, not work items. `FAIL`/`PARTIAL` rows are
the remediation scope. The `Workstream` column maps each row to §2.

### 0.1.1 Test environment

| Item | Value |
|---|---|
| Build | Folisift 0.17.1, build 185, MCP schema 13, 35 tools |
| D2 | v0.9.0; sidecar symlink was required before `doctor` passed |
| Model | `potion-multilingual-128M-int8` |
| Sources | `is-hukuku.pdf` 117 pp; `Ders_Notu_1_VİZE` 31 slides; `DijitalÇağdaÜretimYönetimi` 134 pp; `DijitalDönüşümdeÜrünYönetimi` 42 pp |
| Outputs | 26 / 10 / 19 page guides; 9 / 5 / 8 cards |
| Context policy | `detail=summary`, `max_tokens`, `fields` exercised; heavy claims/graph calls separate |
| Verification | `audit_guide --strict` and `qa_guide` |

| ID | Test | Expected | Observed in 0.17.1 | Status | Workstream |
|---|---|---|---|---|---|
| T01 | `qa_guide` main guide | 26 pages, audit pass | 26, pass | PASS | Preserve |
| T02 | `qa_guide` IRAC | 10–12 pages, audit pass | 10, pass | PASS | Preserve |
| T03 | `qa_guide` digital | 18–20 pages, audit pass | 19, pass | PASS | Preserve |
| T04 | `audit_guide` main | no underfilled/overflow | none | PASS | Preserve |
| T05 | `audit_guide` IRAC | no underfilled/overflow | none | PASS | Preserve |
| T06 | `audit_guide` digital | no underfilled/overflow | none | PASS | Preserve |
| T07 | `coverage_manifest build` main | 141 rows | 138; 4.4.4/10.2 missing | FAIL | V1 |
| T08 | 141-row review manifest validate | 141/141, no empty | claimed 141, empty 141 | FAIL | V1 |
| T09 | `coverage_manifest build` digital | 41/41 | 62 rows, claimed 2, missing 60 | FAIL | V1 |
| T10 | `source_brief` main | 141 headings, OCR off | 139; 4.4.4/10.2 missing | PARTIAL | V12 |
| T11 | `source_brief` digital slides | 11 low-text, OCR candidates | 11, candidates present | PASS | Preserve |
| T12 | `section_brief max_tokens=50` | <=50 tokens | 939 tokens | FAIL | V2 |
| T13 | `heading_brief max_tokens=20` | <=20 tokens | 939 tokens | FAIL | V2 |
| T14 | `chapter_brief heading=4` | `4 4857...` | `1.4 Sosyal Güvenlik` | FAIL | V2 |
| T15 | `source_topics semantic=true` | `model2vec` | `heading-terms` | FAIL | V3 |
| T16 | `claims summary max_tokens=1200` | compact | thousands of lines | FAIL | V4 |
| T17 | `graph query max_tokens=800` | <=800 tokens | 20 nodes + bboxes | FAIL | V4 |
| T18 | `list_guides` IRAC | `case_count=10` | 0 | FAIL | V5 |
| T19 | `guide_status` IRAC | `section_count=14` | 0 | FAIL | V5 |
| T20 | `verify_citations max_claims=5` | claim sentence | page header, verified true | PARTIAL | V6 |
| T21 | `table_extract` digital p.26 | native table | `[]` | PARTIAL | V8 |
| T22 | `style estimate accessible` | 12–14, over budget true | 12–14, over_budget false | FAIL | V9 |
| T23 | `cards include_definitions` | same as qa | CSV 9, qa 8 | PARTIAL | V10 |
| T24 | `diagram status` | one schema | ok/state/status/errors | PASS | Preserve |
| T25 | `graph query "fesih"` | heading nodes | heading nodes | PASS | Preserve |
| T26 | `source_insights` digital p.2 | needs_ocr + reasons | sparse-text, embedded-images | PASS | Preserve |
| T27 | `visual_brief` digital p.2 | role + recommendation | visual-evidence-required | PASS | Preserve |
| T28 | `citation_snippet` main p.14 | snippet | alt işveren text | PASS | Preserve |
| T29 | `cost_report` main | token/char | 59,535 source tokens | PASS | Preserve |
| T30 | D2 `doctor` | D2 found | failed before symlink | ENV | V7 |
| T31 | IRAC `###` build | 10 pages | 10 | PASS | Preserve |
| T32 | Output audits | 26/10/19 | 26/10/19 | PASS | Preserve |

## 0.2 Scorecard reconciliation

| Area | v0.17.1 score | Target | Why | Gap owner |
|---|---:|---:|---|---|
| Build / PDF production | 9.5 | 10.0 | fast, deterministic, staging, incremental; still needs final QA/cost proof | V12/V15 |
| Audit / layout | 9.0 | 10.0 | page roles and detectors work; visual taxonomy still ambiguous | V8 |
| QA single call | 9.5 | 10.0 | one gate works; card and cost fields still incomplete | V10/V14 |
| Coverage | 4.0 | 10.0 | 7/7 legacy gate versus 141 review rows; manifest body mapping is broken | V1 |
| Source/context economics | 6.5 | 10.0 | `source_brief` is useful; brief budgets, topics and large outputs are not bounded | V2/V3/V4 |
| D2 | 8.0 | 10.0 | status schema is stable; discovery and full decision/width proof remain | V7/V13 |
| Style | 8.5 | 10.0 | estimate exists; over_budget and restyle proof are incomplete | V9 |
| API consistency | 8.0 | 10.0 | tools are stable; some metrics and limits disagree | V4/V5/V10 |
| Determinism | 9.0 | 10.0 | local/hash/cache are solid; manifest and policy evidence still missing | V15 |
| Documentation | 8.0 | 10.0 | good base; IRAC `###`, D2 PATH, manifest and budget docs are missing | V12/V15 |

Weighted target: **10/10 only when A-V2-01..18 pass**.

## 0.3 Debug scan evidence — v0.17.1 real workspace

These are direct observations from a read-only MCP session against the
four-source production workspace. They confirm the feedback and add the
`F-V2-DBG-*` rows.

| Probe | Observed |
|---|---|
| `list_guides` IRAC | `case_count=0`, `section_count=14`, `content_words=2347` |
| `guide_status` IRAC | `section_count=0`, `content_words=3223` |
| `section_brief(4.4.3, max_tokens=50)` | `estimated_tokens=939` |
| `chapter_brief(4, max_tokens=200)` | `heading="1.4. Sosyal Güvenlik Hukuku"` |
| `source_topics(semantic=true)` | `method="heading-terms"`, no fallback warning |
| `claims(detail=summary, max_tokens=1200)` | 191 claims returned |
| `graph query("fesih", max_tokens=800)` | 20 nodes, bbox on every node |
| `style estimate accessible` | `12-14` against `10-12`, `over_budget=false` |
| `source_brief` main | 30 returned by default; `heading_count=139` |
| `table_extract` digital p.26 | `mode="layout"`, `tables=[]`, no reason |
| `visual_brief` digital p.26 | `role="text-page"` because the page is not low-text, although the page has a figure |

The review file also contains 62 digital rows while the canonical source set is
41; this is a plan-level distinction, not a missing review row.

## 0.4 Implementation status — 0.18.0 worktree

| Area | Result |
|---|---|
| Main coverage | review manifest 141/141, `empty_sections=[]`, `qa.coverage.total=141` |
| Digital coverage | invalid review (60 missing) ignored with warning; legacy canonical gate 41/41 |
| Briefs | `section_brief(50)` 107 tokens + `truncated`; `heading_brief(20)` 81 + `truncated`; `chapter_brief(4)` returns `4. 4857...` |
| Topics | `semantic=true` returns `model2vec` when the model is available; otherwise explicit fallback warning |
| Claims/graph | claims summary 949 tokens, 40/191 returned; graph query 200 tokens, 20/42 nodes, no bbox |
| Metrics | IRAC list/status `case_count=10`, `section_count=14`, `content_words=2347` |
| Citation | sentence scoring, header avoidance, `match_terms`/`confidence`; weak match `verified=false` |
| Style | accessible estimate 12–15 vs 10–12, `over_budget=true` |
| Cards | QA 9/5/8, definitions and intervals included |
| Source brief | `headings_total=141`, `review_added=2`, explicit `truncated`/`next` |
| QA | cost block present; main/digital/IRAC all pass |

P2 is closed. `make feedback-v2` covers the external main/digital/IRAC workspace
and the five repository guides; it also runs the restyle and scaffold unit
checks. The conditional reranker remains the only explicit deferral.

## 0.5 P2 implementation status — 0.19.0 worktree

| Area | Result |
|---|---|
| D2 width | Real PDF image bbox test: 60% renders narrower than 90% |
| D2 decision depth | 24 nodes/40 edges/depth 8, `direction`, `aspect_ratio`, `allow_page_span`, `label_lines` |
| LiteParse/OCR | `source_insights.ocr_evidence` reports policy, pages_ocr/skipped, candidates and reasons |
| Graph shape | `path`/`explain` bounded; geometry opt-in |
| Determinism | brief/topic cache keys and fallback tests; `qa.environment` records versions, model signature, OCR/reranker policy and seed |
| Docs | manifest precedence, brief budgets, IRAC `###`, D2 sidecar, visual taxonomy, OCR and cards documented |
| Acceptance | `make feedback-v2` runs A-V2-01…18 on the external workspace and repo guides; all pass |

## 1. Root cause map

| ID | Symptom | Root cause | Code seam | Fix |
|---|---|---|---|---|
| R1 | 141 review rows all `empty` | `ManifestRow` has no guide-section mapping; `report()` keys `body_chars` by source number (`1.1`) while the guide may only expose `## 1.` | `crates/folisift-parse/src/manifest.rs`, `coverage.rs::guide_body_chars` | add `guide_sections`/`guide_heading`, index h2+h3, fallback to parent chapter |
| R2 | build emits 138 not 141 | `candidates()` only uses scan headings/canonical sections; manual review rows are not merged | `manifest.rs::candidates` | review-first merge; preserve manual rows and gaps |
| R3 | digital build 62 vs 41 | multi-source headings are appended without canonical dedupe and per-source manifests | `manifest.rs`, `Inventory` | canonical sections primary; per-source rows + combined summary |
| R4 | brief ignores `max_tokens` | MCP converts tokens to chars, `section_brief` clamps to >=200, `estimated_tokens` is full-text not payload | `brief.rs`, `mcp/src/lib.rs::brief_for` | budget the serialized brief; hard min/max; truncation fields |
| R5 | `4` matches `1.4` | `find_section` uses substring `contains` | `brief.rs::find_section` | exact number → exact heading → prefix → fuzzy; ambiguity result |
| R6 | semantic topics fall back | MCP only reads `data/topics.json`; missing artifact silently falls back to heading terms | `mcp/src/lib.rs::source_topics` | compute on demand, cache, explicit fallback/warning |
| R7 | claims unbounded | `claims` handler never calls `shape_response`; `max_tokens` not applied | `mcp/src/lib.rs::claims` | summary envelope, max claims/severity, hard budget |
| R8 | graph query unbounded | `query`/`path`/`explain` branches bypass `shape_response` | `mcp/src/lib.rs::graph` | shared bounded result envelope |
| R9 | IRAC metrics 0 | `list_guides` hardcodes `## Dava`/`## Case`; `guide_status.section_count` uses the body-character map instead of `##` headings | `core/src/config.rs`, `mcp/src/lib.rs::guide_status` | shared `GuideMetrics`; case prefixes; `##` section count |
| R10 | citation says verified on header | `verify_citations` takes first claim word and page text; no sentence scoring or header filter | `mcp/src/lib.rs::verify_citations` | key-term sentence score, confidence, match_terms, threshold |
| R11 | D2 not found | `doctor` only checks `FOLISIFT_D2`/PATH; sidecar is next to binary | `mcp/src/lib.rs::d2_available`, `doctor` | search binary dir, package root, `~/.local/lib/folisift/d2` |
| R12 | style over_budget false | `estimate` compares `low > max`, not `high > max` | `render/src/styles.rs::estimate` | upper-bound comparison + reason/warning |
| R13 | cards 9 vs qa 8 | `qa` calls `cards_from_markdown` without definitions; `cards` uses `include_definitions` | `build/src/lib.rs::qa` | one card policy; return recall/definitions/total |
| R14 | IRAC scaffold generic | `prepare_guide` skeleton writes `sources: []`; existing guide is preserved without a migration path; source headings are not separated | `mcp/src/lib.rs::skeleton_markdown`, `prepare_guide`, `core/src/scaffold.rs` | 10 cases, sources frontmatter, `source-headings.md`, no overwrite |
| R15 | visual `table` vs native table | decision taxonomy conflates native table with figure-to-table redraw | `visual-audit` contract, `table_extract` | typed `VisualDecision`; `table_likely`/reason; layout cross-check |
| R16 | digital review has 62 rows, gate expects 41 | review manifest contains 21 supplemental rows beyond canonical 41; no canonical/extended distinction | `manifest.rs`, review JSON | `canonical_total=41`, `rows_total=62`; extras must be `merged`/`excluded` |
| R17 | review manifest path unreachable | `coverage_manifest` read/validate only opens `data/coverage-manifest.json`; review file is ignored | `mcp/src/lib.rs::coverage_manifest`, `build/src/lib.rs` | `manifest=review|generated|path` selector |
| R18 | `source_brief` silently caps at 30 | `max_headings` default 30; caller sees only 30 of 139/141 without a total/next | `mcp/src/lib.rs::source_brief` | `headings_total`, `returned`, `truncated`, `next` |
| R19 | `claims` has no claim limit | `ClaimsParams` has no `max_claims`/`min_severity`; response ignores `detail` | `mcp/src/lib.rs::claims` | claim policy + shape response |
| R20 | `graph query` always carries bbox | query branch returns `graph.query()` nodes with geometry and no shape response | `mcp/src/lib.rs::graph` | drop bbox unless `include_geometry`, hard budget |
| R21 | style estimate undercounts tables | `estimate` counts only `::: table`, not Markdown tables or glossary groups | `render/src/styles.rs::estimate` | count all table kinds; explain counts |
| R22 | qa ignores guide review intervals | `qa` uses hard-coded `[1,3,7,14,30]` instead of `guide.config.review_intervals` | `build/src/lib.rs::qa` | one card policy from config/request |
| R23 | review topics can be stale | `source_topics` reads any `data/topics.json` without inventory digest/model hash/k validation | `mcp/src/lib.rs::source_topics` | digest/model/k cache key; stale warning |
| R24 | `source_brief` has no review-heading merge | it maps only `source.headings`, so `4.4.4`/`10.2` never appear | `mcp/src/lib.rs::source_brief` | merge `headings.review.md`/review manifest; report `review_added` |
| R25 | `table_extract` empty has no reason | empty `tables: []` does not say native table absent or figure-only | `mcp/src/lib.rs::table_extract` | `reason`, `table_likely`, `mode` |
| R26 | `table_extract` layout mode unbounded | the `guide` layout branch returns before `shape_response` | `mcp/src/lib.rs::table_extract` | wrap both branches |
| R27 | `source_brief` ignores response options | `SourceBriefParams.response` is declared but never used | `mcp/src/lib.rs::source_brief` | shape response; headings total/next |
| R28 | `visual_brief` calls a figure page `text-page` | it trusts the low-text flag and ignores layout figure/table blocks | `mcp/src/lib.rs::visual_brief`, `SourceLayout` | use page blocks + low-text + decision |

## 2. Workstreams

### V1 — Coverage truth and manifest precedence (P0)

**Findings:** T07, T08, T09; §2.1; §0 blockers 1.

**Change**

- Extend `ManifestRow` with:
  - `guide_sections: Vec<String>` — exact `##`/`###` guide headings that cover
    the row;
  - `guide_heading: Option<String>` — optional reviewed display anchor;
  - `body_chars: usize` — computed, not persisted as authority.
- `guide_body_chars` indexes every `##` and `###` heading, not only top-level
  chapters. A row with `1.1` first matches `## 1.1`; if absent it falls back to
  the parent `## 1.` body once, not zero.
- `coverage_manifest` precedence:
  1. `data/coverage-manifest.review.json` (host-reviewed, authoritative);
  2. `data/coverage-manifest.generated.json`;
  3. `data/coverage-manifest.json` (legacy combined generated);
  4. legacy canonical-section gate.
- The review file may contain supplemental rows beyond the canonical set. Report
  `canonical_total` (41 for digital) separately from `rows_total` (62 in the
  current file). Supplemental rows must be `merged` or `excluded`; a canonical
  missing row still fails.
- Add optional `reviewed_at`/`reviewed_by` metadata (recommended, not required
  for legacy files). A file with rows still `missing` is a generated skeleton,
  not an accepted review. The current digital file has 62 rows, 2 covered and
  60 missing, so it is **not** review-eligible; the host must review the
  canonical 41 before `coverage_source=review` is reported.
- `build` writes `coverage-manifest.generated.json`; never overwrites a review
  manifest. If a review manifest exists, build reports a row diff and requires
  an explicit `adopt=true` to promote it.
- `validate` accepts `manifest="review|generated|path"` and returns
  `coverage_source`, `manifest_path`, `per_source`, `canonical_total`,
  `rows_total`, and `empty_sections`.
- `qa_guide` and `build_guide --check` use the review manifest when present and
  include `coverage_source` in the report.
- `ManifestRow` rows are deduplicated by `(source, normalized_heading)`.
  Canonical `source.sections` are primary; headings supplement page numbers;
  manual review rows (`4.4.4`, `10.2`) are preserved even if scan headings miss
  them.
- Multi-source: one manifest per source plus a combined `summary.total`.
  `coverage.md` and `qa_guide` use the combined canonical total; supplemental
  rows are counted separately.

**Interface**

```text
coverage_manifest {
  action: build|read|validate|hint,
  guide, manifest?: "review"|"generated"|path,
  adopt?: bool, detail?: summary|full
}

data/coverage-manifest.review.json   # host-reviewed, authoritative
data/coverage-manifest.generated.json
data/coverage-manifest.json          # combined generated summary
data/coverage/<source>.json          # per-source rows
```

**Acceptance:** A-V2-01, A-V2-02, A-V2-03; T07–T09. The digital test must show
`canonical_total=41`, `rows_total=62`, `missing=[]`, and 21 supplemental
`merged`/`excluded` rows.

### V2 — Brief budget and exact heading matching (P0)

**Findings:** T12, T13, T14; §2.2, §2.3.

**Change**

- Add `BriefBudget { max_tokens, max_chars, level }`; `max_tokens` is a hard
  budget for the final JSON payload, not a label.
- `SectionBrief` gains `truncated`, `next_offset`, `full_chars`,
  `budget_tokens`, `budget_chars`, `candidates` (ambiguity).
- Budget order:
  1. derive `max_chars = min(max_chars, max_tokens * 4)`;
  2. trim `summary` at a sentence boundary;
  3. drop `numbers`, `table_rows`, `citations`, then `blocks` until the
     serialized payload fits;
  4. if metadata alone cannot fit, return `E_BRIEF_BUDGET` with the minimum
     viable budget.
- `estimated_tokens` is computed on the returned payload, never on the full
  section.
- `find_section` resolution:
  1. exact normalized heading (`4 4857...`);
  2. exact number token (`4`, `4.`, `4 `);
  3. number-prefix match;
  4. title-token fuzzy score only as fallback.
  Multiple hits return `ambiguous=true` and `candidates[]`; never silently pick
  a substring match.
- `chapter_brief` requires a chapter-level number; `section_brief` accepts a
  section; `heading_brief` accepts any heading. All three share the same
  resolver and budget.

**Interface**

```text
chapter_brief|section_brief|heading_brief {
  guide, source, heading, max_tokens?, max_chars?
}
→ { heading, level, summary, estimated_tokens, budget_tokens,
    truncated, next_offset, full_chars, cache_hit, ambiguous?, candidates? }
```

**Acceptance:** A-V2-04, A-V2-05; T12–T14.

### V3 — Real semantic topics (P0)

**Findings:** T15; §2.4.

**Change**

- Move topic clustering into one shared service (parse/links) used by CLI and
  MCP.
- `semantic=true`:
  - load the local Model2Vec model (`potion-multilingual-128M-int8` default);
  - embed headings, cluster with deterministic centroid/tie-break;
  - return `method: "model2vec"` or `"model2vec+heading"`;
  - persist `data/topics.json` keyed by inventory digest + model hash + `k`;
  - reject stale artifacts whose digest/model/k does not match;
  - return representative heading, page range, terms and cluster size.
- Missing model or artifact:
  - fallback to heading terms;
  - return `fallback: "heading-terms"` and `warning`;
  - never silently claim semantic.
- `prepare_guide` may warm the artifact, but a missing model must not fail the
  guide.

**Acceptance:** A-V2-06; T15; deterministic repeat test; stale artifact reports
`fallback`/`warning` instead of silently reusing old clusters.

### V4 — Hard output budgets for claims and graph (P0)

**Findings:** T16, T17; §2.5.

**Change**

- Apply the shared `shape_response` contract to `claims`, `graph query`,
  `graph path` and `graph explain`.
- `claims`:
  - `detail=summary` default;
  - `max_claims` and `min_severity`;
  - return counts, first N claims, `omitted`, `total`, `next`;
  - write full Markdown to disk and return the path;
  - `ClaimsParams.response` is currently declared but unused; wire it.
- `graph`:
  - `detail=summary` returns counts/kinds only;
  - `query` returns `node_limit` nodes plus `omitted`/`total`;
  - `path`/`explain` drop bboxes unless `include_geometry=true`;
  - `max_tokens` is a hard serialized-payload budget; the query branch currently
    bypasses `shape_response`.
- `shape_response` must add `truncated`, `omitted`, `total`, `next` whenever it
  shortens an array, not only when `max_tokens` is set.

**Acceptance:** A-V2-07, A-V2-08; T16–T17.

### V5 — One guide-metrics contract (P0)

**Findings:** T18, T19; §2.6.

**Change**

- Move metric computation to `GuideMetrics` in `folisift-core`:
  - `guide_kind` from the structure pack;
  - `case_count` from the structure pack’s case labels (`Dava`, `Case`,
    `Olay`, `Event`, or pack-declared), not a hard-coded pair;
  - `section_count` = `##` top-level sections (14 for IRAC); add a separate
    `heading_count` only if a caller needs `###` detail;
  - `content_words` = the same `prose_words` helper for both tools;
  - `source_count`, `guide_count`.
- `list_guides`, `guide_status`, `workspace_info` and `qa_guide` call the same
  helper.
- IRAC `guide_kind` requires `case_count`; a mismatch is a warning, not a
  silent zero.

**Acceptance:** A-V2-09; T18–T19; list vs status equality.

### V6 — Citation verification quality (P0)

**Findings:** T20; §2.7.

**Change**

- Extract claim key terms (3+ chars, stop-word filtered).
- Split the source page into candidate sentences; skip header/footer lines
  (first/last two lines or repeated page title).
- Score each sentence by key-term overlap + phrase proximity; choose the best.
- Return `snippet`, `match_terms`, `confidence` (`high|medium|low`), `verified`
  only when the score passes the threshold.
- If no sentence reaches the threshold, return `verified: false` with
  `reason: "no supporting sentence"`; never mark a page header as verified.

**Acceptance:** A-V2-10; T20.

### V7 — D2 discovery and diagnostics (P1)

**Findings:** T30; §2.8.

**Change**

- `find_d2` search order:
  1. `FOLISIFT_D2`;
  2. `$PATH/d2`;
  3. the binary’s sibling `d2` (packaged sidecar);
  4. `<workspace>/../lib/folisift/d2` and `~/.local/lib/folisift/d2`;
  5. `D2` package root recorded in the install manifest.
- `doctor` reports the found path, version and a copyable `FOLISIFT_D2=...`
  command when missing.
- `docs/install.md` documents sidecar/PATH behavior.

**Acceptance:** A-V2-11; T30.

### V8 — Visual-audit taxonomy and table extraction (P1)

**Findings:** T21; §2.9.

**Change**

- Replace the ambiguous `decision: "table"` with:
  - `native_table` — the source already has a table;
  - `redraw_table` — figure/matrix must become a table;
  - `redraw_figure` — figure redraw;
  - `text` — text extraction is enough;
  - `decorative` — no content;
  - `unreadable` — needs human inspection.
- Accept legacy `table` as `redraw_table` for one release with a migration
  warning; `redraw` remains `redraw_figure`.
- `visual_brief` combines low-text, layout figure/table blocks and the recorded
  decision; it must not call a figure page `text-page` just because the text
  layer exists.
- `table_extract` returns:
  - `tables`, `reason` (`no table block`, `figure only`, `table_likely`),
  - `mode: native|layout`, `table_likely` and `source_text_layer`,
  - cross-check `source_insights.complexity.layout.ruled_table_count`;
  - the current response has `mode: layout` but no `reason`/`table_likely`;
  - both the layout and native branches must pass through `shape_response`.
- Visual-audit docs and templates use the new enum.

**Acceptance:** A-V2-12; T21.

### V9 — Style estimate and restyle contract (P1)

**Findings:** T22; §2.10; §5 item 8.

**Change**

- `over_budget = high > target_max` (not `low > max`).
- Count all table kinds (`Markdown`, `::: table`, glossary) and all diagrams;
  the current estimate counts only `::: table`.
- Add `target_pages`, `over_budget_reason`, `warning` and `estimated_range`.
- `accessible` documents the +2–4 page expectation; estimate warns before a
  full build.
- Keep `restyle` content-hash guard; add a regression that content hash is
  unchanged.

**Acceptance:** A-V2-13, A-V2-15; T22.

### V10 — Cards/QA parity (P1)

**Findings:** T23; §2.11.

**Change**

- One card policy for CLI, MCP and `qa_guide`.
- `qa_guide.cards` returns `recall`, `definitions`, `total`, `review_intervals`
  and `includes_definitions`.
- `include_definitions` and `review_intervals` are read from the guide config
  (or request) and are visible in both outputs; `qa` currently ignores
  `config.review_intervals`.

**Acceptance:** A-V2-14; T23.

### V11 — IRAC scaffold and migration (P1)

**Findings:** §2.12; §5 item 9.

**Change**

- New IRAC guide:
  - exactly 10 `## Dava/Case` blocks;
  - `### Sorun/Kural/Uygulama/Sonuç` (or English equivalents);
  - frontmatter `sources` populated from the inventory;
  - source headings written to `source-headings.md`, not appended to `guide.md`.
- Existing guide:
  - never overwrite without `force`;
  - if IRAC content lacks case blocks, return `E_STRUCTURE` with
    `suggestion: "run irac migration"`, not a silent generic skeleton;
  - `prepare_guide` response includes `scaffold.kind`, `case_count`,
    `source_headings_path`.
- `scaffold()` and `prepare_guide()` share one skeleton function.

**Acceptance:** A-V2-16; T31.

### V12 — Source-heading completeness and docs (P1/P2)

**Findings:** T10; §2.1.1; §5 items 13, 19.

**Change**

- Merge `headings.review.md` and TOC rows into `source_brief`; add manual
  `4.4.4`/`10.2` entries when the scan misses them.
- `source_brief` reports `headings_total`, `headings_returned`, `truncated`,
  `next`, `review_added`, `missing_candidates`; default 30 is a page size, not
  a silent cap; `SourceBriefParams.response` must be wired.
- Update docs: IRAC `###`, manifest review precedence, brief budgets, D2
  sidecar, `flow`/`subs`, table numbering, visual decision taxonomy.
- Add a runnable example for every corrected behavior.

**Acceptance:** A-V2-17; T10; `headings_total` is 141 and `4.4.4`/`10.2` appear
or are explicitly `review_added`.

### V13 — D2 decision depth and width regression (P2)

**Findings:** §5 items 15; §4 D2 score.

**Change**

- Verify 20+ node `decision` trees; current 24/40/depth 8 limits stay.
- Enforce vertical output for `direction: down` or report the chosen layout.
- Add a rendered-bbox test for `width=60%` vs `width=90%` (not only geometry).
- `allow_page_span=true` is required for a two-page diagram; otherwise
  `E_DIAGRAM_PAGE_SPAN`.

**Acceptance:** A-V2-18.

### V14 — QA cost report and profile metrics (P1)

**Findings:** §5 item 14.

**Change**

- `qa_guide` returns a compact `cost` block: source chars, read chars, guide
  words, build iterations, tool count, estimated tokens, cache hits.
- No secrets, no network, no provider SDK.
- `cost_report` and `qa_guide.cost` share one schema.

**Acceptance:** A-V2-18; release report includes cost.

### V15 — Determinism and documentation gate (P2)

**Findings:** §5 items 19–20; §4 Determinism/Docs.

**Change**

- Record model hash, LiteParse version, D2 version/layout, OCR policy and
  topic/reranker seed in run reports.
- Same input produces the same manifest, brief, topics, cards and PDF hashes.
- `make acceptance` is extended with the V2 suite and the real-workspace
  smoke test.

**Acceptance:** A-V2-17, A-V2-18.

## 3. Data contracts

```rust
struct CoverageManifest {
    schema_version: u32,
    reviewed_at: Option<String>,
    reviewed_by: Option<String>,
    canonical_total: Option<usize>,
    rows: Vec<ManifestRow>,
}

struct ManifestRow {
    source: String,
    heading: String,
    normalized_heading: String,
    page: usize,
    guide_heading: Option<String>,
    guide_sections: Vec<String>,
    status: CoverageStatus, // covered|missing|merged|excluded
    evidence: Option<String>,
    confidence: f32,
}

struct CoverageReportV2 {
    manifest_source: ManifestSource, // review|generated|legacy
    total: usize,
    claimed: usize,
    missing: Vec<String>,
    empty_sections: Vec<String>,
    per_source: Vec<SourceCoverage>,
    passed: bool,
}

struct BriefBudget {
    max_tokens: usize,
    max_chars: usize,
    level: BriefLevel, // chapter|section|heading
}

struct SectionBriefV2 {
    heading: String,
    level: String,
    summary: String,
    numbers: Vec<String>,
    table_rows: Vec<String>,
    citations: Vec<String>,
    estimated_tokens: usize,
    budget_tokens: usize,
    truncated: bool,
    next_offset: Option<usize>,
    full_chars: usize,
    ambiguous: bool,
    candidates: Vec<String>,
}

struct TopicReport {
    method: String, // model2vec|model2vec+heading|heading-terms
    fallback: Option<String>,
    warning: Option<String>,
    clusters: Vec<TopicCluster>,
}

struct GuideMetrics {
    guide_kind: String,
    case_count: usize,
    section_count: usize,
    content_words: usize,
    source_count: usize,
}

struct CitationCheck {
    claim: String,
    snippet: Option<String>,
    match_terms: Vec<String>,
    confidence: String, // high|medium|low
    verified: bool,
    reason: Option<String>,
}

enum VisualDecision {
    NativeTable,
    RedrawTable,
    RedrawFigure,
    Text,
    Decorative,
    Unreadable,
}

struct StyleEstimateV2 {
    estimated_pages: String,
    target_pages: Option<String>,
    over_budget: bool,
    over_budget_reason: Option<String>,
    warning: Option<String>,
}

struct CardReport {
    recall: usize,
    definitions: usize,
    total: usize,
    includes_definitions: bool,
    review_intervals: Vec<u32>,
}

struct ClaimSummary {
    total: usize,
    returned: usize,
    omitted: usize,
    next: Option<usize>,
    max_claims: usize,
    min_severity: String,
    claims: Vec<Claim>,
}

struct GraphSummary {
    total: usize,
    returned: usize,
    omitted: usize,
    include_geometry: bool,
    nodes: Vec<GraphNode>,
}

struct SourceBriefHeadings {
    headings_total: usize,
    headings_returned: usize,
    truncated: bool,
    next: Option<usize>,
    review_added: usize,
    items: Vec<HeadingRow>,
}
```

## 4. Milestones

### M1 — 0.17.2 P0 hotfix

- V1 coverage truth.
- V2 brief budget + exact matching.
- V3 semantic topics.
- V4 claims/graph budgets.
- V5 guide metrics.
- V6 citation verification.

**Exit:** A-V2-01…A-V2-10 pass; main 141/141, digital 41/41;
`section_brief(50)` and `heading_brief(20)` obey budgets; `chapter_brief(4)`
never returns `1.4`; `source_topics(semantic=true)` returns `model2vec`;
claims/graph stay under budget; IRAC metrics are 10/14.

### M2 — 0.18 P1

- V7 D2 discovery.
- V8 visual taxonomy + table extraction.
- V9 style estimate/restyle.
- V10 cards/QA parity.
- V11 IRAC scaffold.
- V12 source headings + docs.
- V14 QA cost report.

**Exit:** A-V2-11…A-V2-16 pass; style estimate over_budget true; visual
taxonomy explicit; cards totals agree; IRAC scaffold has 10 cases and sources;
source_brief shows 141 headings.

### M3 — 1.0 P2

- V13 D2 depth/width/vertical.
- V15 determinism/docs/release gate.
- LiteParse/insight integration and OCR policy evidence.
- Bounded graph path/explain.

**Exit:** A-V2-17…A-V2-18 pass; two runs produce identical manifest/brief/
clusters; D2 bbox changes with width; docs cover every corrected behavior.

## 5. Finding register

Evidence values: `confirmed` (seen in 0.17.1), `verified` (has a passing test
after the fix), `deferred` (explicitly accepted), `regression` (preserve).

### 5.1 Verdict and blocker register

| ID | Finding | Workstream | Priority | Evidence | Target |
|---|---|---|---|---|---|
| F-V2-0-1 | Coverage 7/7 vs 141 | V1 | P0 | confirmed | A-V2-01 |
| F-V2-0-2 | Brief token budget ignored | V2 | P0 | confirmed | A-V2-04 |
| F-V2-0-3 | Chapter 4 matches 1.4 | V2 | P0 | confirmed | A-V2-05 |
| F-V2-0-4 | Semantic topics lexical | V3 | P0 | confirmed | A-V2-06 |
| F-V2-0-5 | Claims/graph unbounded | V4 | P0 | confirmed | A-V2-07/08 |
| F-V2-0-6 | IRAC metrics zero | V5 | P0 | confirmed | A-V2-09 |
| F-V2-0-7 | Citation verification header | V6 | P0 | confirmed | A-V2-10 |
| F-V2-0-8 | D2 PATH missing | V7 | P1 | confirmed | A-V2-11 |
| F-V2-0-9 | Style over_budget false | V9 | P1 | confirmed | A-V2-13 |
| F-V2-0-10 | Visual table taxonomy | V8 | P1 | confirmed | A-V2-12 |
| F-V2-0-11 | Cards 9 vs qa 8 | V10 | P1 | confirmed | A-V2-14 |
| F-V2-0-12 | IRAC scaffold generic | V11 | P1 | confirmed | A-V2-16 |

### 5.2 Test matrix register (T01–T32)

| ID | Finding | Workstream | Priority | Evidence | Target |
|---|---|---|---|---|---|
| F-V2-T01 | qa main 26 pages/pass | Preserve | regression | PASS | A-V2-18 |
| F-V2-T02 | qa IRAC 10 pages/pass | Preserve | regression | PASS | A-V2-18 |
| F-V2-T03 | qa digital 19 pages/pass | Preserve | regression | PASS | A-V2-18 |
| F-V2-T04 | audit main clean | Preserve | regression | PASS | A-V2-18 |
| F-V2-T05 | audit IRAC clean | Preserve | regression | PASS | A-V2-18 |
| F-V2-T06 | audit digital clean | Preserve | regression | PASS | A-V2-18 |
| F-V2-T07 | build 138 not 141 | V1 | P0 | FAIL | A-V2-01 |
| F-V2-T08 | review 141 empty | V1 | P0 | FAIL | A-V2-01 |
| F-V2-T09 | digital 62 not 41 | V1 | P0 | FAIL | A-V2-03 |
| F-V2-T10 | source_brief 139 | V12 | P1 | PARTIAL | A-V2-17 |
| F-V2-T11 | digital OCR candidates | Preserve | regression | PASS | A-V2-18 |
| F-V2-T12 | brief 939 vs 50 | V2 | P0 | FAIL | A-V2-04 |
| F-V2-T13 | heading brief 939 vs 20 | V2 | P0 | FAIL | A-V2-04 |
| F-V2-T14 | chapter 4 -> 1.4 | V2 | P0 | FAIL | A-V2-05 |
| F-V2-T15 | topics heading-terms | V3 | P0 | FAIL | A-V2-06 |
| F-V2-T16 | claims unbounded | V4 | P0 | FAIL | A-V2-07 |
| F-V2-T17 | graph query unbounded | V4 | P0 | FAIL | A-V2-08 |
| F-V2-T18 | case_count 0 | V5 | P0 | FAIL | A-V2-09 |
| F-V2-T19 | section_count 0 | V5 | P0 | FAIL | A-V2-09 |
| F-V2-T20 | header snippet verified | V6 | P0 | PARTIAL | A-V2-10 |
| F-V2-T21 | table_extract [] no reason | V8 | P1 | PARTIAL | A-V2-12 |
| F-V2-T22 | over_budget false | V9 | P1 | FAIL | A-V2-13 |
| F-V2-T23 | cards 9 vs 8 | V10 | P1 | PARTIAL | A-V2-14 |
| F-V2-T24 | diagram status schema | Preserve | regression | PASS | A-V2-18 |
| F-V2-T25 | graph heading query | Preserve | regression | PASS | A-V2-18 |
| F-V2-T26 | source insights reasons | Preserve | regression | PASS | A-V2-18 |
| F-V2-T27 | visual brief role | Preserve | regression | PASS | A-V2-18 |
| F-V2-T28 | citation snippet p14 | Preserve | regression | PASS | A-V2-18 |
| F-V2-T29 | cost report | Preserve | regression | PASS | A-V2-18 |
| F-V2-T30 | doctor D2 missing | V7 | P1 | ENV | A-V2-11 |
| F-V2-T31 | IRAC ### 10 pages | Preserve | regression | PASS | A-V2-18 |
| F-V2-T32 | output audits | Preserve | regression | PASS | A-V2-18 |

### 5.3 Confirmed fixed behavior to preserve (§1)

| ID | Finding | Workstream | Priority | Evidence | Target |
|---|---|---|---|---|---|
| F-V2-1.1 | qa one release gate | Preserve | regression | PASS | A-V2-18 |
| F-V2-1.2 | audit page roles/detectors | Preserve | regression | PASS | A-V2-18 |
| F-V2-1.3 | source brief/insights | Preserve | regression | PASS | A-V2-17 |
| F-V2-1.4 | visual/citation/table tools | Preserve | regression | PASS | A-V2-18 |
| F-V2-1.5 | style estimate exists | V9 | P1 | PARTIAL | A-V2-13 |
| F-V2-1.6 | cards definitions/intervals | V10 | P1 | PARTIAL | A-V2-14 |
| F-V2-1.7 | stats.tables object | Preserve | regression | PASS | A-V2-18 |
| F-V2-1.8 | incremental report | Preserve | regression | PASS | A-V2-18 |
| F-V2-1.9 | diagram status merge | Preserve | regression | PASS | A-V2-18 |
| F-V2-1.10 | graph heading/query | Preserve | regression | PASS | A-V2-18 |
| F-V2-1.11 | doctor style checks | Preserve | regression | PASS | A-V2-11 |
| F-V2-1.12 | IRAC ### structure | Preserve | regression | PASS | A-V2-18 |

### 5.4 Broken behavior register (§2)

| ID | Finding | Workstream | Priority | Evidence | Target |
|---|---|---|---|---|---|
| F-V2-2.1-1 | build 138; 4.4.4/10.2 missing | V1 | P0 | confirmed | A-V2-01 |
| F-V2-2.1-2 | review 141 claimed, 141 empty | V1 | P0 | confirmed | A-V2-01 |
| F-V2-2.1-3 | `body_chars` keyed by source number | V1 | P0 | confirmed | A-V2-01 |
| F-V2-2.1-4 | digital 62/2/60 | V1 | P0 | confirmed | A-V2-03 |
| F-V2-2.1-5 | qa uses 7/7, review unused | V1 | P0 | confirmed | A-V2-02 |
| F-V2-2.1-6 | no per-source/combined manifest | V1 | P0 | confirmed | A-V2-03 |
| F-V2-2.2-1 | max_tokens ignored | V2 | P0 | confirmed | A-V2-04 |
| F-V2-2.2-2 | max_chars clamp 200 | V2 | P0 | confirmed | A-V2-04 |
| F-V2-2.2-3 | estimated_tokens full section | V2 | P0 | confirmed | A-V2-04 |
| F-V2-2.2-4 | no truncated/next/full_chars | V2 | P0 | confirmed | A-V2-04 |
| F-V2-2.2-5 | hierarchical budgets not enforced | V2 | P0 | confirmed | A-V2-04 |
| F-V2-2.3-1 | contains matching | V2 | P0 | confirmed | A-V2-05 |
| F-V2-2.3-2 | exact/prefix/fuzzy missing | V2 | P0 | confirmed | A-V2-05 |
| F-V2-2.3-3 | ambiguity not reported | V2 | P0 | confirmed | A-V2-05 |
| F-V2-2.3-4 | no heading_id | V2 | P0 | confirmed | A-V2-05 |
| F-V2-2.4-1 | semantic silently falls back | V3 | P0 | confirmed | A-V2-06 |
| F-V2-2.4-2 | no on-demand model | V3 | P0 | confirmed | A-V2-06 |
| F-V2-2.4-3 | no fallback/warning field | V3 | P0 | confirmed | A-V2-06 |
| F-V2-2.4-4 | no deterministic cluster seed | V3 | P0 | confirmed | A-V2-06 |
| F-V2-2.5-1 | claims not shaped | V4 | P0 | confirmed | A-V2-07 |
| F-V2-2.5-2 | graph query not shaped | V4 | P0 | confirmed | A-V2-08 |
| F-V2-2.5-3 | graph path/explain not shaped | V4 | P0 | confirmed | A-V2-08 |
| F-V2-2.5-4 | omitted/total/next missing | V4 | P0 | confirmed | A-V2-07/08 |
| F-V2-2.6-1 | case_count hardcoded | V5 | P0 | confirmed | A-V2-09 |
| F-V2-2.6-2 | guide_status section_count uses body map (0) while list counts h2 (14) | V5 | P0 | confirmed | A-V2-09 |
| F-V2-2.6-3 | content_words mismatch | V5 | P0 | confirmed | A-V2-09 |
| F-V2-2.6-4 | guide_kind/case_count not enforced | V5 | P0 | confirmed | A-V2-09 |
| F-V2-2.7-1 | verify uses first word | V6 | P0 | confirmed | A-V2-10 |
| F-V2-2.7-2 | header chosen as snippet | V6 | P0 | confirmed | A-V2-10 |
| F-V2-2.7-3 | verified by page only | V6 | P0 | confirmed | A-V2-10 |
| F-V2-2.7-4 | no confidence/match_terms | V6 | P0 | confirmed | A-V2-10 |
| F-V2-2.8-1 | doctor misses sidecar | V7 | P1 | confirmed | A-V2-11 |
| F-V2-2.8-2 | no exact export command | V7 | P1 | confirmed | A-V2-11 |
| F-V2-2.8-3 | install docs missing | V7 | P1 | confirmed | A-V2-11 |
| F-V2-2.9-1 | table/redraw ambiguous | V8 | P1 | confirmed | A-V2-12 |
| F-V2-2.9-2 | table_extract [] no reason | V8 | P1 | confirmed | A-V2-12 |
| F-V2-2.9-3 | no insights cross-check | V8 | P1 | confirmed | A-V2-12 |
| F-V2-2.9-4 | no native/layout mode | V8 | P1 | confirmed | A-V2-12 |
| F-V2-2.10-1 | over_budget low vs high | V9 | P1 | confirmed | A-V2-13 |
| F-V2-2.10-2 | no reason/range | V9 | P1 | confirmed | A-V2-13 |
| F-V2-2.10-3 | no accessible warning | V9 | P1 | confirmed | A-V2-13 |
| F-V2-2.11-1 | qa excludes definitions | V10 | P1 | confirmed | A-V2-14 |
| F-V2-2.11-2 | no card breakdown | V10 | P1 | confirmed | A-V2-14 |
| F-V2-2.12-1 | sources: [] | V11 | P1 | confirmed | A-V2-16 |
| F-V2-2.12-2 | 60+ source headings in guide | V11 | P1 | confirmed | A-V2-16 |
| F-V2-2.12-3 | IRAC sections appended | V11 | P1 | confirmed | A-V2-16 |
| F-V2-2.12-4 | no force/migration guard | V11 | P1 | confirmed | A-V2-16 |

### 5.4b Debug scan additions (2026-11-04)

These are additional findings from the code-level scan, beyond the literal
feedback rows. They are part of the same remediation scope.

| ID | Finding | Root cause | Workstream | Priority | Target |
|---|---|---|---|---|---|
| F-V2-DBG-1 | Digital review has 62 rows but the gate expects 41 | review file mixes canonical 41 with 21 supplemental rows; report has no canonical/extended split | V1 | P0 | A-V2-03 |
| F-V2-DBG-2 | Review manifest is not selectable | read/validate always open `data/coverage-manifest.json` | V1 | P0 | A-V2-01 |
| F-V2-DBG-3 | `source_brief` silently caps at 30 | `max_headings` default 30; only `heading_count` hints at the rest | V12 | P1 | A-V2-17 |
| F-V2-DBG-4 | `claims` has no claim/severity limit | `ClaimsParams` has `response` but no `max_claims`/`min_severity`; handler ignores response | V4 | P0 | A-V2-07 |
| F-V2-DBG-5 | `graph query` always returns bbox | query branch uses `graph.query()` directly and bypasses shape response | V4 | P0 | A-V2-08 |
| F-V2-DBG-6 | `style estimate` counts only wrapped tables | `estimate` filters `::: table`; Markdown/glossary tables are ignored | V9 | P1 | A-V2-13 |
| F-V2-DBG-7 | `qa` ignores guide review intervals | `qa` hard-codes `[1,3,7,14,30]` | V10 | P1 | A-V2-14 |
| F-V2-DBG-8 | Stale topic artifacts are accepted | `source_topics` reads `data/topics.json` without digest/model/k validation | V3 | P0 | A-V2-06 |
| F-V2-DBG-9 | `source_brief` has no review-heading merge | only `source.headings` is read; manual rows never appear | V12 | P1 | A-V2-17 |
| F-V2-DBG-10 | `table_extract` empty result has no reason | `tables: []` is returned without `reason`/`table_likely` | V8 | P1 | A-V2-12 |
| F-V2-DBG-11 | `table_extract` layout branch ignores `max_tokens` | early return before `shape_response` | V8 | P1 | A-V2-12 |
| F-V2-DBG-12 | `source_brief` ignores `detail`/`fields`/`max_tokens` | `SourceBriefParams.response` is unused | V12 | P1 | A-V2-17 |
| F-V2-DBG-13 | `visual_brief` calls figure page 26 `text-page` | role uses `low_text` only, not layout figure/table blocks | V8 | P1 | A-V2-12 |
| F-V2-DBG-14 | Digital “review” file is 62 rows with 60 missing | generated skeleton is stored at the review path; no `reviewed_at`/`reviewed_by` or canonical eligibility check | V1 | P0 | A-V2-03 |

### 5.5 Output inventory (§3)

| ID | Output | Pages/cards | Status | Workstream |
|---|---|---|---|---|
| F-V2-3-1 | `output/is-hukuku.pdf` | 26 | audit pass | Preserve |
| F-V2-3-2 | `output/is-hukuku-irac.pdf` | 10 | audit pass | Preserve |
| F-V2-3-3 | `output/dijital-donusum-temelleri.pdf` | 19 | audit pass | Preserve |
| F-V2-3-4 | `is-hukuku-irac-forest-sage.pdf` | 10 | audit pass | V9 |
| F-V2-3-5 | `is-hukuku-irac-berry.pdf` | 10 | audit pass | V9 |
| F-V2-3-6 | `is-hukuku-cards.csv` | 9 | updated | V10 |
| F-V2-3-7 | `is-hukuku-irac-cards.csv` | 5 | updated | V10 |
| F-V2-3-8 | `dijital-donusum-temelleri-cards.csv` | 8 | updated | V10 |

### 5.6 Priority roadmap register (§5)

| ID | Priority item | Workstream | Priority | Target |
|---|---|---|---|---|
| F-V2-5-1 | Coverage body mapping | V1 | P0 | A-V2-01 |
| F-V2-5-2 | Brief token budget | V2 | P0 | A-V2-04 |
| F-V2-5-3 | Chapter exact match | V2 | P0 | A-V2-05 |
| F-V2-5-4 | Semantic topics | V3 | P0 | A-V2-06 |
| F-V2-5-5 | Claims/graph budget | V4 | P0 | A-V2-07/08 |
| F-V2-5-6 | IRAC metrics | V5 | P0 | A-V2-09 |
| F-V2-5-7 | Citation snippet | V6 | P0 | A-V2-10 |
| F-V2-5-8 | Restyle/style estimate | V9 | P1 | A-V2-13/15 |
| F-V2-5-9 | IRAC scaffold | V11 | P1 | A-V2-16 |
| F-V2-5-10 | Table extract reason | V8 | P1 | A-V2-12 |
| F-V2-5-11 | D2 PATH | V7 | P1 | A-V2-11 |
| F-V2-5-12 | Cards QA count | V10 | P1 | A-V2-14 |
| F-V2-5-13 | Source brief 141 | V12 | P1 | A-V2-17 |
| F-V2-5-14 | QA cost report | V14 | P1 | A-V2-18 |
| F-V2-5-15 | D2 decision/width/vertical | V13 | P2 | A-V2-18 |
| F-V2-5-16 | Source insights full | V12 | P2 | A-V2-17 |
| F-V2-5-17 | OCR auto policy | V15 | P2 | A-V2-18 |
| F-V2-5-18 | Bounded graph path/explain | V4 | P2 | A-V2-08 |
| F-V2-5-19 | Docs cookbook | V12/V15 | P2 | A-V2-17 |
| F-V2-5-20 | Determinism manifest | V15 | P2 | A-V2-18 |

### 5.7 User-flow register (§0.3)

| ID | Flow | Finding | Workstream | Target |
|---|---|---|---|---|
| F-V2-FLOW-1 | Main guide | 26 pages, audit pass; coverage still 7/7 | V1 | A-V2-01/02 |
| F-V2-FLOW-2 | IRAC | 10 cases, 10 pages, `###` accepted | V11 | A-V2-16 |
| F-V2-FLOW-3 | Digital | 19 pages, 3 sources, 21 low-text, 4 D2 | V1/V8 | A-V2-03/12 |
| F-V2-FLOW-4 | Style | reader 10, berry 10, accessible 12–14 vs 10–12 | V9 | A-V2-13 |
| F-V2-FLOW-5 | Cards/graph/links | 9/5/8 cards; graph query large; links `blocks:1/3` unclear | V4/V10 | A-V2-07/14 |

### 5.8 Scorecard register (§4)

| ID | Area | Current | Target | Workstream |
|---|---|---:|---:|---|
| F-V2-SC-1 | Build / PDF | 9.5 | 10.0 | V15 |
| F-V2-SC-2 | Audit / layout | 9.0 | 10.0 | V8 |
| F-V2-SC-3 | QA single call | 9.5 | 10.0 | V10/V14 |
| F-V2-SC-4 | Coverage | 4.0 | 10.0 | V1 |
| F-V2-SC-5 | Source/context | 6.5 | 10.0 | V2/V3/V4 |
| F-V2-SC-6 | D2 | 8.0 | 10.0 | V7/V13 |
| F-V2-SC-7 | Style | 8.5 | 10.0 | V9 |
| F-V2-SC-8 | API consistency | 8.0 | 10.0 | V4/V5/V10 |
| F-V2-SC-9 | Determinism | 9.0 | 10.0 | V15 |
| F-V2-SC-10 | Documentation | 8.0 | 10.0 | V12/V15 |

### 5.9 Environment register (§0.1)

| ID | Finding | Workstream | Priority | Target |
|---|---|---|---|---|
| F-V2-ENV-1 | D2 not on PATH initially | V7 | P1 | A-V2-11 |
| F-V2-ENV-2 | Model potion-multilingual-128M-int8 | V3 | P0 | A-V2-06 |
| F-V2-ENV-3 | Source set 117/31/134/42 pages | V1 | P0 | A-V2-03 |
| F-V2-ENV-4 | Four-source production workspace | V1/V12 | P0 | A-V2-18 |
| F-V2-ENV-5 | `detail/max_tokens/fields` used | V4 | P0 | A-V2-07/08 |
| F-V2-ENV-6 | Audit/QA verification path | Preserve | regression | A-V2-18 |

### 5.10 Final twelve (§7)

| ID | Required outcome | Workstream | Priority | Target |
|---|---|---|---|---|
| F-V2-7-1 | 141/141 manifest | V1 | P0 | A-V2-01 |
| F-V2-7-2 | brief budgets | V2 | P0 | A-V2-04 |
| F-V2-7-3 | chapter exact | V2 | P0 | A-V2-05 |
| F-V2-7-4 | semantic topics | V3 | P0 | A-V2-06 |
| F-V2-7-5 | claims/graph bounded | V4 | P0 | A-V2-07/08 |
| F-V2-7-6 | IRAC metrics | V5 | P0 | A-V2-09 |
| F-V2-7-7 | citation sentence | V6 | P0 | A-V2-10 |
| F-V2-7-8 | D2 PATH | V7 | P1 | A-V2-11 |
| F-V2-7-9 | over_budget | V9 | P1 | A-V2-13 |
| F-V2-7-10 | visual taxonomy | V8 | P1 | A-V2-12 |
| F-V2-7-11 | cards QA | V10 | P1 | A-V2-14 |
| F-V2-7-12 | IRAC scaffold | V11 | P1 | A-V2-16 |

## 6. Acceptance suite

The suite runs in `make acceptance` after V2 integration. Every test uses the
real workspace when available and a synthetic fixture in CI.

| ID | Test | Expected |
|---|---|---|
| A-V2-01 | review manifest validate | 141 total, 141 claimed, missing [], empty [], passed true |
| A-V2-02 | `qa_guide` main coverage | `coverage.total=141`, `coverage_source=review` |
| A-V2-03 | digital multi-source | 3 per-source manifests nonzero; `canonical_total=41`; current 62-row file is rejected as a review because 60 rows are missing; after review, 21 supplemental rows merged/excluded and missing [] |
| A-V2-04 | `section_brief` 50 / `heading_brief` 20 | estimated_tokens <= budget or `truncated=true` + next_offset |
| A-V2-05 | `chapter_brief heading=4` | `4 4857...` or ambiguity; never `1.4` |
| A-V2-06 | `source_topics semantic=true` | method `model2vec`; fallback warning if model missing |
| A-V2-07 | `claims detail=summary max_tokens=1200` | payload <= 1200 tokens, `max_claims`/`min_severity` honored, `truncated`/`omitted` when cut |
| A-V2-08 | `graph query max_tokens=800` | payload <= 800 tokens; bbox omitted unless `include_geometry=true`; `path`/`explain` bounded |
| A-V2-09 | IRAC list/status | case_count 10, section_count 14, equal across tools |
| A-V2-10 | `verify_citations max_claims=5` | claim sentence, header excluded, verified false on weak match |
| A-V2-11 | `doctor` | finds sidecar D2; missing case prints exact export command |
| A-V2-12 | visual audit/table extract | native_table vs redraw_table; `reason`, `table_likely`, `mode` returned; layout branch obeys `max_tokens` |
| A-V2-13 | `style estimate accessible` 10–12 | 12–14, over_budget true, warning; all table kinds counted |
| A-V2-14 | `cards include_definitions` + qa | same total, breakdown recall/definitions |
| A-V2-15 | `restyle` | content_hash unchanged |
| A-V2-16 | IRAC scaffold | 10 cases × 4 `###`, sources frontmatter, source-headings separate |
| A-V2-17 | source_brief main | `headings_total=141`, `4.4.4`/`10.2` present or review-added, explicit page/next not a silent 30 cap, `detail`/`fields`/`max_tokens` honored |
| A-V2-18 | full regression | five repo guides + four-source workspace; qa/audit/coverage/cost/determinism pass |

## 7. Release gates

- `cargo test --workspace`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `make exam-check`, `visual`, `type-check`, `parity`, `genre-check`,
  `d2-pdf-check`
- `make acceptance` extended with the V2 suite
- Real workspace: `is-hukuku` 141/141, `is-hukuku-irac` 10 cases,
  `dijital-donusum-temelleri` 41/41
- Two identical runs produce identical manifest, brief, topic and card hashes
- Release note states the measured score and any explicit deferral

## 8. Risks and open questions

1. **Manifest ambiguity.** A source heading can map to multiple guide sections;
   `guide_sections` is a list, not a single string. Review precedence must be
   explicit or two tools will disagree again.
2. **Token estimator.** Four chars/token is an estimate; never present it as a
   provider bill. The budget test uses the same estimator.
3. **Model availability.** Semantic topics must degrade with a warning, not a
   silent lexical fallback.
4. **Existing guides.** IRAC migration must not overwrite host-authored content;
   `force` is explicit and audited.
5. **Visual-audit migration.** Legacy `table` values are accepted for one
   release and reported as legacy; no silent rewrite.
6. **D2 packaging.** The sidecar search order must work from the release
   archive, a source checkout and `~/.local`.
7. **Real workspace fixtures.** `sources/` is local-only; CI uses synthetic
   equivalents with the same shapes, and the release gate uses the user
   workspace when present.

## 9. Non-goals and preserved invariants

- No cloud dependency, API key, LLM call, shell execution or network access in
  the MCP server.
- No full-PDF OCR by default; `source_insights` remains advisory.
- No generative summary inside the deterministic brief/topic pipeline.
- No silent overwrite of a host-authored guide; `force` is explicit.
- No similarity link or page match is presented as a verified fact.
- No `passed` without counts, source and evidence.
- No release claim of coverage, token savings or 10/10 before A-V2 passes.

## 10. Definition of done

The feedback is closed when:

- every row in §5 is `verified` or explicitly `deferred`;
- A-V2-01…A-V2-18 pass;
- main manifest is 141/141 and digital combined is 41/41;
- brief, claims and graph outputs obey their declared budgets;
- IRAC metrics and cards agree across list/status/qa/cards;
- `source_topics` reports `model2vec` when asked;
- `verify_citations` never marks a page header as verified;
- D2 is found without a manual symlink;
- `style estimate` and visual taxonomy are unambiguous;
- `make acceptance` is the single release gate;
- docs and release notes include the measured evidence and the remaining
  explicit deferral.
