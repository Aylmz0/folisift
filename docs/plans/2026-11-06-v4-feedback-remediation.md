# V4 feedback remediation — detailed plan

- **Status:** Proposed
- **Date:** 2026-11-06
- **Baseline:** Folisift 0.22.0, build 230, MCP schema 17, 41 tools
- **Primary sources:**
  - [`FOLISIFT-GERI-BILDIRIM-v4AGENT.md`](../FOLISIFT-GERI-BILDIRIM-v4AGENT.md) — external test-user matrix: 26 PASS / 7 FAIL
  - [`FOLISIFT-GERI-BILDIRIM-v4.md`](../FOLISIFT-GERI-BILDIRIM-v4.md) — our PDF/system review: P1 visual-gate truth
  - [`2026-11-05-final-feedback-v3-remediation.md`](2026-11-05-final-feedback-v3-remediation.md) — completed prior remediation
- **Targets:**
  - **0.22.1** — P0 bug fixes: generated manifest, nested output budget, hint range/covers, path resolution, guide-plan source pages, visual-gate truth
  - **0.23.0** — P1/P2: `excluded` status, OCR availability, small visual/cards/image-id semantics, docs
- **Definition of done:** all 33 v4AGENT tests pass (or are explicitly environment-skipped), our P1 is closed, `make acceptance` is green, and a fresh 0.22.1 PDF passes `audit --strict`.

---

## 0. Short diagnosis

The 0.22.0 visual pipeline is real and the PDFs are strong, but the v4AGENT matrix exposes a split:

1. **Visual production is good:** glossary, figures, D2, visual decisions, audit.
2. **The surrounding contract is not finished:** generated coverage manifest, nested
   output budgets, path resolution in MCP, hint ranges, guide-plan page mapping.
3. **The old visual gate is still noisy:** it reads raw layout blocks, not the new
   candidate manifest. `qa.visual` reports 82/28/136 figure pages for
   PD0/PD2/PD3 while `passed:true` under `warn`.
4. **The 4-column glossary is already implemented and visually verified.** The
   remaining issue was one stale design-system sentence, corrected in
   `386194a`. This plan adds a regression test instead of redesigning it.

The critical path is 0.22.1. The `excluded` status and OCR are 0.23.0 because
they change the manifest contract or the build feature surface.

---

## 1. Verified baseline: four-column glossary

This is **not** an open design task. The system is already updated:

| Layer | Current state | Evidence |
|---|---|---|
| Markdown | `::: glossary` and `::: table kind="glossary"` both feed one collector | `crates/folisift-render/src/markdown.rs` (`collect_glossary_table`, `glossary_typst`) |
| Template | One `glossary_table`; group rows are full-width tinted cells; `glossary_row` uses `__TABLE_SIZE__`; leading `0.55em`; inset `1.6mm` | `templates/paper.typ` |
| Tests | `glossary_groups_share_one_table` asserts one `#glossary_table` and two `#glossary_row`; four-column test exists | `crates/folisift-render/src/markdown.rs` tests |
| PDF | PD0–PD1 s. 18–19: one four-column table, pale-sage group rows, body font/leading; v4AGENT sees 66/30 terms in is-hukuku/dijital | visual inspection; v4AGENT §1 |
| Docs | `docs/authoring-spec.md`, `docs/authoring.md` describe one four-column table; `docs/design-system.md` stale “two columns” line corrected | commit `386194a` |

**Residual work:** add an explicit regression fixture for a glossary that
crosses a page boundary and a group row that must repeat/continue cleanly. Do
not change the visual design in this plan.

---

## 2. Findings and root causes

Priority: **P0** blocks the 10/10 claim or a `required` gate; **P1** is a
contract gap; **P2** is polish/observability. Each finding has a reproducible
acceptance test.

| ID | Priority | Finding | Evidence | Root cause (code) |
|---|---|---|---|---|
| F-V4A-01 | P0 | `coverage_manifest generated` is 17/138; keys contain body prose; `4.4.4`/`10.2` absent | v4AGENT T07 | `CoverageManifest::build` uses `source.headings` and `source.sections`; `clean_heading_title`/`title_like` are heuristics, not canonical extraction. No fallback to TOC text or numbered source lines. |
| F-V4A-02 | P0 | `validate` ignores nested `fields`/`max_tokens`; `per_source[].sections[]` dumps | v4AGENT T08 | `shape_response` projects top-level paths and `shrink_to_budget` chooses arrays, but the validate response is not passed through a single recursive budget helper consistently; `read` summary returns before shaping. |
| F-V4A-03 | P0 | `hint` marks `2.2` likely-missing under `2.1–2.2` | v4AGENT T09 | `heading_range_contains` exists and passes unit tests, but the live path likely receives a different heading/manifest form; `covers_index` only matches `source:number`, not ranges or comma lists. |
| F-V4A-04 | P0 | `read_section` looks for `data/blocks/sources/PD0.pdf.json` | v4AGENT T27 | `read_section` joins `format!("{}.json", params.source)` without normalizing path/extension; `read_blocks` uses a different resolver. |
| F-V4A-05 | P0 | MCP `diagram list`/`validate` path errors; `guide` required inconsistently | v4AGENT T28 | `diagram` resolves `guide` via `Guide::load` but different actions require/omit it; no shared slug/path normalizer; spec-only `validate` should not need a guide. |
| F-V4A-06 | P0 | `guide_plan source_pages` empty for all sections | v4AGENT T29 | `guide_plan` matches `section.number == number` exactly; no fallback to `source.headings`, normalized numbers or `<!-- covers: … -->`. Inventory sections are often empty (PD0) or only TOC numbers. |
| F-V4A-07 | P0 | `qa.visual` reports 82/28/136 figure pages; `passed:true` with 40/24/118 missing | our v4 review; v4AGENT T21/§4.9 | `visual_audit::figure_pages` reads raw `data/blocks` `figure`/`table` blocks; it is not connected to `visual_candidates`, containers or area/score filters. `warn` semantics are reported as `passed`. |
| F-V4A-08 | P1 | No `status:"excluded"`; 21 curated-out rows deleted | v4AGENT T04/§4.4 | `CoverageStatus::Excluded` exists in the model and report, but `build`/review/adopt have no producer; rows are removed instead. |
| F-V4A-09 | P1 | OCR `ocr:true` fails: tesseract feature disabled, no server URL | v4AGENT T30 | `crates/folisift-parse/Cargo.toml` uses `liteparse` with `default-features = false`; no `tesseract` feature; `doctor` does not report OCR availability. |
| F-V4A-10 | P1 | `source_insights` has no image ids; `visual_extract` needs another call | v4AGENT §4.9 | `extract_insights` serializes `image_refs.len()` only; `ImageRef` metadata is discarded. |
| F-V4A-11 | P1 | `visual_decision decorative` becomes unresolved, especially with high priority | v4AGENT §4.9 | `VisualAudit` rejects `priority=high` + `decorative`; the cover/front use case needs a documented exception or a lower priority. |
| F-V4A-12 | P2 | `cards` can silently drop recall blocks | v4AGENT §4.9 | `cards` counts definitions but does not warn when a guide has no `::: recall`/`::: answer`; no minimum-content warning. |
| F-V4A-13 | P2 | `qa_guide visual_gate` “warn/pass” meaning undocumented | v4AGENT §4.9 | Report exposes `passed` and `visual_gate` but not `advisory`/`required` status. |
| F-V4A-14 | P2 | `source_insights image_id` missing (same as F-V4A-10) | v4AGENT §4.9 | add `image_refs_detail` with `id`, bbox, format, pixel dimensions. |
| F-V4A-15 | P2 | 4-column glossary page-break/group-row continuity is not regression-tested | our review | visual is correct; add fixture/test. |

---

## 3. Work packages

### WP0 — Reproduction fixture and truth baseline

**Goal:** stop fixing from prose. Build a small fixture that reproduces every
v4AGENT failure without the user's private workspace.

**Files**

- `crates/folisift-parse/tests/fixtures/manifest-v4/` (synthetic inventory +
  Markdown + TOC text)
- `crates/folisift-cli/tests/mcp_stdio.rs` (path matrix)
- `scripts/check-feedback-v5.py` (A-V4A harness)

**Fixture cases**

1. A source with body-contaminated headings and a missing `4.4.4`/`10.2` in
   `headings` but present in TOC text.
2. A guide with `## 2.1–2.2 …` and `<!-- covers: SRC:2.1–2.2 -->`.
3. A manifest with `status:"excluded"` + `exclude_reason`.
4. MCP source arguments in all four forms: `PD0`, `PD0.pdf`,
   `sources/PD0.pdf`, `data/blocks/PD0.json`.
5. A `visual-audit` with raw layout blocks that would produce 82 pages, and a
   candidate manifest that produces 4 real pages.

**Exit:** failing tests before fixes; each failure names its finding ID.

---

### WP1 — Canonical generated manifest (F-V4A-01)

**Files:** `crates/folisift-parse/src/manifest.rs`,
`crates/folisift-parse/src/inventory.rs`, `source_read.rs`/cache as needed.

**Change**

- Replace `candidates(source)` with a layered canonicalizer:
  1. layout headings with `level`/`block_id` and a title-like cleaned heading;
  2. TOC/canonical sections;
  3. numbered source lines (`^\d+(\.\d+)+ .+`) from cached source text;
  4. `<!-- covers: … -->` only as review evidence, never as a generated row.
- Canonical row key is **number only** (`source:4.4.4`), never body text.
- `clean_heading_title` must stop at sentence punctuation and use a maximum of
  120 characters / 18 words; reject lines that start with a verb/body marker
  (`Bu`, `Bir`, `Amaç`, `Örnek`, `Şekil`, `Tablo`, `Kaynak`).
- Missing nested numbers from TOC/text are additive; do not replace a better
  layout row.
- `canonical_total` must equal the union of unique numbers; `4.4.4` and `10.2`
  must be present when the source text/TOC contains them.

**Tests**

- Body-contaminated heading never becomes a row.
- `4.4.4` and `10.2` generated from TOC/text.
- Duplicate numbers are deduped by number, with origin priority
  `heading > toc > text`.
- `validate generated` on the fixture is 138/138 (or the documented canonical
  total + explicit `excluded`).

**Exit:** A-V4A-01, A-V4A-02.

---

### WP2 — One recursive output-budget function (F-V4A-02)

**Files:** `crates/folisift-mcp/src/lib.rs`, all tools using `shape_response`.

**Change**

- Introduce one `shape_tool_output(value, OutputOptions)` helper:
  - `fields` projection must recurse through arrays and objects;
  - `max_tokens` must truncate the largest nested arrays and strings;
  - every truncated array gets `{total, omitted}`;
  - `detail=summary` truncates arrays to a documented keep count;
  - summary/read fast paths must not bypass it.
- Apply it to `coverage_manifest` (`read`, `validate`, `review`, `hint`), `hint`,
  `guide_plan`, `visual_candidates`, `claims`, `graph`, and any other tool with
  nested arrays.
- Add a size guard: response contains `truncated`, `max_tokens`, `omitted_total`
  when the budget is hit.

**Tests**

- `validate` with `detail:"summary", max_tokens:50` returns a small object.
- `fields:["per_source.sections.number"]` keeps only that nested key.
- `fields:["missing"]` does not leak `per_source`.
- `max_tokens` never returns a payload larger than `max_tokens * 4 + overhead`.

**Exit:** A-V4A-03.

---

### WP3 — Hint ranges and covers (F-V4A-03)

**Files:** `crates/folisift-parse/src/hint.rs`, `manifest.rs`,
`crates/folisift-mcp/src/lib.rs`.

**Change**

- Parse `<!-- covers: SRC:2.1–2.2 -->`, comma lists (`2.1,2.2`), en/em dash and
  ASCII hyphen; return a set of `(source, number)`.
- Share one `covers_index` implementation between `manifest` and `hint`; stop
  duplicating regexes.
- In `similarity`, check the **range before** normalization and also after
  extracting `number` from the manifest row; if a manifest row is `Covered`,
  force `score=1.0` even if the lexical score is lower.
- `hint` should use the reviewed manifest rows when present, and never emit
  `likely-missing` for a row whose number is inside a guide range or covers
  comment.
- Add a regression for `2.1–2.2`, `3.1–3.4`, comma lists and single numbers.

**Exit:** A-V4A-04.

---

### WP4 — Shared path resolver for source blocks (F-V4A-04, F-V4A-05)

**Files:** `crates/folisift-parse/src/layout.rs` or a new
`source_path.rs`; `crates/folisift-mcp/src/lib.rs`; `diagram` module.

**Change**

- Add `resolve_block_path(guide_dir, source) -> PathBuf`:
  - accepts `PD0`, `PD0.pdf`, `sources/PD0.pdf`, `data/blocks/PD0.json`;
  - strips directories and `.pdf`/`.json`;
  - uses `guide.dir/data/blocks/<id>.json`;
  - rejects `..`/absolute escapes through the workspace.
- Use it in `read_blocks`, `read_section`, `source_brief` (where applicable),
  `visual_brief` and `visual_candidates` source lookup.
- Diagram MCP:
  - `validate` with a `spec` and no guide is legal (pure spec validation);
  - `save`, `list`, `status`, `render_plan`, `fit` require a normalized guide
    slug;
  - accept `guides/<slug>` as a convenience but resolve through
    `Workspace::guide_dir`;
  - `list`/`status` use the same resolver as the CLI; no `root/<slug>` double
    path.

**Tests**

- `read_section` with all four source forms returns the same content.
- `diagram validate` with a spec/no guide passes; `diagram list`/`status` with
  `guide:"pd3"` works under repo root; `guides/pd3` also resolves.
- MCP stdio path matrix.

**Exit:** A-V4A-05, A-V4A-06.

---

### WP5 — `guide_plan.source_pages` (F-V4A-06)

**Files:** `crates/folisift-mcp/src/lib.rs`, `inventory.rs`, `manifest.rs`.

**Change**

- Build a normalized number → pages index from:
  1. `source.sections` (number + page);
  2. `source.headings` (number prefix + page);
  3. `<!-- covers: … -->` ranges;
  4. reviewed manifest rows.
- Match guide heading numbers after trimming trailing dots and normalizing
  separators; for `2.1–2.2`, map both source numbers.
- Return `source_pages` sorted/deduped; if unknown, return `[]` plus
  `source_pages_unknown: true` rather than silently empty.
- Add `source_page_ranges` for merged headings.

**Tests**

- 0.x/1.x PD0–PD1 guide: no empty `source_pages` where the inventory has a
  heading.
- `2.1–2.2` guide heading maps both pages.
- A guide with no source page mapping gets `source_pages_unknown: true`.

**Exit:** A-V4A-07.

---

### WP6 — Candidate-backed visual gate (F-V4A-07)

**Files:** `crates/folisift-parse/src/visual_audit.rs`,
`visual_candidates.rs`, `crates/folisift-build/src/lib.rs`,
`crates/folisift-mcp/src/lib.rs`, `crates/folisift-cli/src/main.rs`.

**Change**

- Add `visual candidates --write` (MCP `write:true`) producing
  `guides/<slug>/data/visual-candidates.json`:
  - `schema_version`, `scorer_version`, `source_digest`, `generated_from`;
  - one row per candidate with `id`, `page`, `score`, `relevance`,
    `reconstruction`, `bbox`, `suggested`.
- `visual_audit::figure_pages` reads the candidate manifest when present:
  - gate pages = candidates with `relevance=core` and
    `score >= audit.visual_min_score` (default 50), plus tables with
    `native_table`/`redraw_table`;
  - no raw page-spanning `figure`/`table` blocks;
  - missing/stale manifest under `visual_gate=required` fails with
    `visual candidates --write` as the next step.
- Legacy path (no manifest) remains for `warn`, but the report marks
  `advisory:true` and `basis:"layout-blocks"`.
- `VisualReport` add `basis`, `advisory`, `candidate_manifest` and
  `missing_candidates`.
- `qa.visual` expose:
  - `status: advisory|required|passed|failed`
  - `advisory: true` under `warn`
  - `missing_candidates` instead of calling noisy raw pages “missing”.
- Add `audit.visual_min_score` (integer 0–100, default 50).

**Tests**

- PD0/PD2/PD3 `visual_gate=required` with a written candidate manifest:
  expected pages are the reviewed candidate pages, not 82/28/136.
- A stale source digest fails with an actionable error.
- `warn` report says `advisory:true`, not `passed:true`.
- Existing low-text hard gate unchanged.

**Exit:** A-V4A-08, A-V4A-09.

---

### WP7 — `excluded` status (F-V4A-08)

**Files:** `manifest.rs`, MCP/CLI review commands, docs.

**Change**

- Add optional `reason: Option<String>` to `ManifestRow`.
- `CoverageStatus::Excluded` is produced by:
  - a review author marking a row `excluded` + `reason`;
  - a migration tool that converts deleted rows in the old review into
    explicit `excluded` rows.
- `report()` counts `excluded`; `missing` must not include excluded; `passed`
  ignores them; `validate` reports `excluded_total` and lists them separately.
- `review_eligible()` accepts excluded rows; `adopt` preserves them.
- Docs: `excluded` is for content that is intentionally out of scope, not for
  “not done yet”; `missing` is still the only failure.

**Tests**

- A review with one covered + one excluded + one missing: excluded counted,
  missing 1, passed false; after covering the missing row, passed true.
- Adopt preserves excluded rows.
- Old review files without `reason` remain valid.

**Exit:** A-V4A-10.

---

### WP8 — OCR availability (F-V4A-09)

**Files:** `Cargo.toml`, `crates/folisift-parse/Cargo.toml`, `doctor`,
`docs/install.md`, `docs/cookbook.md`.

**Decision:** keep the default portable build without Tesseract; add an
**opt-in `ocr-tesseract` feature** and a documented local server path.

**Change**

- Add a Folisift feature `ocr-tesseract` that enables `liteparse/tesseract`;
  do not enable it by default.
- `doctor` reports:
  - `ocr: "disabled" | "tesseract" | "server"`;
  - the resolved tesseract binary/version or “OCR unavailable” with a next step.
- `source_insights ocr:true` when unavailable returns a structured error:
  `OCR_UNAVAILABLE`, suggestion `install tesseract + build --features
  ocr-tesseract` or `--ocr-server-url`.
- Document the exact local command and the server URL contract.

**Tests**

- `cargo test` without feature retains the structured error.
- CI builds `--features ocr-tesseract` on a runner with tesseract and runs one
  page test (or marks it skipped with an explicit reason).
- `doctor` never claims OCR is available when it is not.

**Exit:** A-V4A-11.

---

### WP9 — Small visual semantics (F-V4A-10, F-V4A-11, F-V4A-12, F-V4A-13)

**Files:** `visual_audit.rs`, `insights.rs`, `cards`, `qa`.

**Change**

- `source_insights`: add `image_refs_detail` with `id`, bbox, format,
  pixel dimensions (no bytes); `visual_extract` can use it directly.
- `visual_decision decorative`: allow `priority=high` with `decorative` when a
  reason/notes says it is a cover/front-matter decision; otherwise emit a
  **warning**, not unresolved. Keep the audit reason mandatory.
- `visual_decision embed_crop` with `candidate_id`: document that `crop` is
  optional because it is derived from bbox; keep explicit `crop` required only
  when no candidate is supplied.
- `qa.visual` advisory semantics from WP6.
- `cards`: if `include_definitions` is true and `recall=0`, add
  `warning:"no recall blocks; only definition cards"`; do not fail.
- `qa.cards` exposes `recall`, `definitions`, `total`, `warning`.

**Tests**

- image refs detail contains `p76_1`.
- cover decorative high-priority passes with reason; high+decorative without
  reason is rejected.
- cards warning appears only when recall=0.

**Exit:** A-V4A-12, A-V4A-13, A-V4A-14.

---

### WP10 — Glossary regression (F-V4A-15)

**Files:** `crates/folisift-render/src/markdown.rs` tests,
`tests/baselines`, `docs/design-system.md`.

**Change**

- Add a page-break/group-row fixture: 30+ terms across two groups, a group row
  at a page boundary, and a continuation table header.
- Assert one `#glossary_table`, four columns, `__TABLE_SIZE__`, leading
  `0.55em`, full-width group rows; capture a visual baseline.
- Keep `docs/design-system.md` statement as one four-column table (already
  corrected in `386194a`).

**Exit:** A-V4A-15.

---

## 4. Acceptance matrix

| ID | Check | Pass condition |
|---|---|---|
| A-V4A-01 | Generated canonical rows | Body-free keys; `4.4.4`/`10.2` present; `validate generated` 138/138 or documented canonical + excluded |
| A-V4A-02 | Generated provenance | Each row has `source`, number key, origin, page; no body prose |
| A-V4A-03 | Nested budget | `max_tokens=50`, `fields=per_source.sections.number` bounded; `omitted/total` present |
| A-V4A-04 | Hint ranges | `2.1–2.2`, `3.1–3.4`, comma covers → covered, score 1.0 |
| A-V4A-05 | Source path forms | `PD0`, `PD0.pdf`, `sources/PD0.pdf`, `data/blocks/PD0.json` equivalent |
| A-V4A-06 | Diagram MCP | `validate` spec-only works; `list/status` with `pd3` and `guides/pd3`; no double path |
| A-V4A-07 | Guide plan pages | `source_pages` non-empty for 0.x/1.x; merged range maps both |
| A-V4A-08 | Visual gate required | PD0/PD2/PD3 candidate manifest; expected pages are real candidates, not 82/28/136 |
| A-V4A-09 | Visual gate warn | `advisory:true`, not `passed:true`; no raw-page missing wall |
| A-V4A-10 | Excluded | excluded counted, not missing; adopts; old files valid |
| A-V4A-11 | OCR | doctor reports availability; unavailable error is structured; feature build passes |
| A-V4A-12 | Image refs | `image_refs_detail` has ids/bbox/dims; visual_extract uses it |
| A-V4A-13 | Decorative semantics | cover decorative high-priority allowed with reason; otherwise warning |
| A-V4A-14 | Cards warning | recall=0 + definitions → warning, not silent |
| A-V4A-15 | Glossary | one table, four columns, group rows, page-break fixture passes |
| A-V4A-16 | Regression | `make acceptance` green, `make visuals-v5` green, PDF audit clean |
| A-V4A-17 | v4AGENT matrix | 26 PASS / 7 FAIL → 33 PASS or documented environment skip |

Manual checks:

- Render PD0–PD1, PD2, PD3 after changes; compare the glossary and figure pages
  visually against the current baselines.
- Run `qa.visual` on all three and confirm the numbers are plausible against
  actual figure captions.

---

## 5. Order, dependencies, and effort

| Phase | Packages | Why first | Estimate |
|---|---|---|---:|
| 1 | WP0, WP1, WP2 | Generated manifest and budget are the critical truth bugs | 3 d |
| 2 | WP3, WP4, WP5 | Hint/path/guide-plan are independent and unblock MCP tests | 2.5 d |
| 3 | WP6 | Candidate-backed visual gate depends on the candidate report | 2 d |
| 4 | WP7, WP8, WP9, WP10 | Contract/extensions/docs | 3.5 d |
| 5 | Acceptance, visual baselines, release | 0.22.1 then 0.23.0 | 2 d |

Rough total: **13 engineering days**. WP1/WP2 can run in parallel with WP3–WP5
after WP0.

---

## 6. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Generated manifest changes canonical totals across guides | Keep explicit `canonical_total`, add a diff report, migrate reviewed manifests explicitly. |
| `fields` projection breaks existing clients | Additive keys only; keep top-level `guide`, `passed`, `total`; tests for old shapes. |
| Visual candidate manifest goes stale | Source digest + scorer version; `required` fails with `visual candidates --write`; `warn` marks advisory. |
| `excluded` hides real omissions | `reason` required; `excluded` never counts as `missing` but is reported separately; review diff shows it. |
| OCR feature bloats the portable build | Opt-in feature; default binary reports unavailable; doctor is explicit. |
| Diagram `guides/<slug>` convenience conflicts with slug validation | Normalize before `Guide::load`; never pass a path to `validate_slug`. |
| 4-column glossary regresses on page breaks | Fixture + baseline; no design change in this plan. |

---

## 7. Release plan

1. Implement 0.22.1 P0 (WP0–WP6 + F-V4A-10 if feasible).
2. Run `make acceptance`, `make visuals-v5`, `cargo test`, `fmt`, `clippy`.
3. Rebuild PD0–PD1/PD2/PD3; `audit --strict`; inspect glossary/figure pages.
4. Update `docs/VERSIONS.md`, `README`, `docs/agents/versioning.md` (schema 18
   only if the tool contract changes).
5. Tag `v0.22.1` locally; do not push unless explicitly authorized.
6. Implement 0.23.0 P1/P2; repeat acceptance; tag only after the maintainer
   approves.

---

## 8. Open decisions

1. **OCR:** opt-in `ocr-tesseract` feature (recommended) vs enabling Tesseract
   by default. The default build must stay portable.
2. **Visual gate manifest:** store under `data/visual-candidates.json` (recommended)
   vs recompute on every build. Stored manifest is needed for deterministic
   `required` gates and QA.
3. **Version split:** 0.22.1 for all P0 (recommended) vs folding everything into
   0.23.0.
4. **`excluded` schema:** optional `reason` on `ManifestRow` (recommended,
   backward-compatible) vs a separate excluded file.
