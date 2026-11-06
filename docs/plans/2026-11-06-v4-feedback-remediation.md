# V4 feedback remediation — detailed plan

- **Status:** Proposed
- **Date:** 2026-11-06
- **Baseline:** Folisift 0.22.0, build 230, MCP schema 17, 41 tools
- **Primary sources:**
  - [`FOLISIFT-GERI-BILDIRIM-v4AGENT.md`](../FOLISIFT-GERI-BILDIRIM-v4AGENT.md) — external test-user matrix: 26 PASS / 7 FAIL
  - [`FOLISIFT-GERI-BILDIRIM-v4.md`](../FOLISIFT-GERI-BILDIRIM-v4.md) — our PDF/system review: P1 visual-gate truth
  - [`2026-11-05-final-feedback-v3-remediation.md`](2026-11-05-final-feedback-v3-remediation.md) — completed prior remediation
- **Targets:**
  - **0.22.1** — P0: F-V4A-01…08 — generated manifest, nested output budget, hint range/covers, source-path resolution, diagram MCP, guide-plan source pages, candidate-backed visual gate, advisory truth. Expected build 231, MCP schema 18.
  - **0.23.0** — P1/P2: F-V4A-09…14 — `excluded` status, OCR availability, image-ref detail, decorative semantics, cards warning, glossary regression. Expected build 232, MCP schema 19.
- **Definition of done (whole plan):** all 33 v4AGENT checks pass (or are explicitly environment-skipped and documented), our P1 visual-gate finding is closed, `make acceptance` is green, and a fresh 0.22.1 PDF passes `audit --strict`.
- **Not claimed:** 10/10. The v4AGENT matrix still has 7 FAIL and our visual gate still reports noisy figure pages until 0.22.1 ships.

---

## 0. Short diagnosis

The v4AGENT matrix is 26 PASS / 7 FAIL: T07, T08, T09, T27, T28 and T29 are
tool bugs; T30 is an OCR environment/feature gap. The 0.22.0 visual pipeline is
real and the PDFs are strong, but the matrix exposes a split:

1. **Visual production is good:** glossary, figures, D2, visual decisions, audit.
2. **The surrounding contract is not finished:** generated coverage manifest, nested
   output budgets, path resolution in MCP, hint ranges, guide-plan page mapping.
3. **The old visual gate is still noisy:** it reads raw layout blocks, not the new
   candidate manifest. `qa.visual` reports 82/28/136 figure pages for
   PD0/PD2/PD3 while `passed:true` under `warn`.
4. **The 4-column glossary is already implemented and visually verified.** The
   remaining issue was one stale design-system sentence, corrected in
   `386194a`. This plan adds a regression test instead of redesigning it.

The critical path is 0.22.1. `excluded` and OCR are 0.23.0 because they change
the manifest contract or the build feature surface.

---

## 1. Verified baseline: four-column glossary

This is **not** an open design task. The system is already updated:

| Layer | Current state | Evidence |
|---|---|---|
| Markdown | `::: glossary` and `::: table kind="glossary"` both feed one collector | `crates/folisift-render/src/markdown.rs` (`collect_glossary_table`, `glossary_typst`) |
| Template | One `glossary_table`; group rows are full-width tinted cells; `glossary_row` uses `__TABLE_SIZE__`; leading `0.55em`; inset `1.6mm` | `templates/paper.typ` |
| Tests | `glossary_groups_share_one_table` asserts one `#glossary_table` and two `#glossary_row`; four-column test exists | `crates/folisift-render/src/markdown.rs` tests |
| PDF | PD0–PD1 s. 18–19: one four-column table, pale-sage group rows, body font/leading; v4AGENT independently sees the same structure in is-hukuku (66 terms) and dijital (30 terms) | visual inspection; v4AGENT §1 |
| Docs | `docs/authoring-spec.md`, `docs/authoring.md` describe one four-column table; `docs/design-system.md` stale “two columns” line corrected | commit `386194a` |

**Residual work:** add an explicit regression fixture for a glossary that
crosses a page boundary and a group row that must repeat/continue cleanly. Do
not change the visual design in this plan.

---

## 2. Findings and root causes

Priority: **P0** blocks the 10/10 claim or a `required` gate; **P1** is a
contract gap; **P2** is polish/observability. Every finding has one acceptance
row with the same numeric suffix.

| ID | Priority | Finding | Evidence | Root cause (code) |
|---|---|---|---|---|
| F-V4A-01 | P0 | `coverage_manifest generated` is 17/138; keys contain body prose; `4.4.4`/`10.2` absent | v4AGENT T07 | `CoverageManifest::build` uses `source.headings` and `source.sections`; `clean_heading_title`/`title_like` are heuristics, not canonical extraction. No fallback to TOC text or numbered source lines. |
| F-V4A-02 | P0 | `validate` ignores nested `fields`/`max_tokens`; `per_source[].sections[]` dumps | v4AGENT T08 | `validate` calls `shape_response`, but that helper projects only top-level paths and `shrink_to_budget` truncates selected arrays; `per_source[].sections[]` is never reached. `read` summary returns before shaping at all. |
| F-V4A-03 | P0 | `hint` marks `2.2` likely-missing under `2.1–2.2` | v4AGENT T09 | `heading_range_contains` works in isolation, but `hint` builds the source number from the first token of `row.heading`, which F-V4A-01 shows can be body prose; `covers_index` also only matches `source:number`, not ranges/comma lists. A guide range cannot compensate for a contaminated row number. |
| F-V4A-04 | P0 | `read_section` looks for `data/blocks/sources/PD0.pdf.json` | v4AGENT T27 | `read_section` joins `format!("{}.json", params.source)` without normalizing path/extension; `read_blocks` uses a different resolver. |
| F-V4A-05 | P0 | MCP `diagram list`/`validate` path errors; `guide` required inconsistently | v4AGENT T28 | `diagram` always calls `Guide::load` at the top, so spec-only `validate` needs a guide; `guides/<slug>` is passed to `validate_slug`; no shared slug/path normalizer. |
| F-V4A-06 | P0 | `guide_plan source_pages` empty for all sections | v4AGENT T29 | `guide_plan` matches `section.number == number` exactly against `source.sections` only; PD0 has no sections and the fallback to `headings`, `page_map` or `<!-- covers: … -->` is missing. |
| F-V4A-07 | P0 | `qa.visual` reports 82/28/136 figure pages; `passed:true` with 40/24/118 missing | our v4 review; v4AGENT T21/§4.9 | `visual_audit::figure_pages` reads raw `data/blocks` `figure`/`table` blocks; it is not connected to `visual_candidates`, containers or area/score filters. |
| F-V4A-08 | P0 | `visual_gate="warn"` is reported as plain `passed:true`; advisory vs enforced is invisible | v4AGENT §4.9 | `VisualReport.passed` conflates “no low-text/unresolved” with “figure gate enforced”; the build overrides `passed` to true under `warn` but does not expose `advisory`, `basis` or `status`. |
| F-V4A-09 | P1 | No `status:"excluded"`; 21 curated-out rows deleted | v4AGENT T04/§4.4 | `CoverageStatus::Excluded` exists in the model and report, but `build`/review/adopt has no producer; rows are removed instead. |
| F-V4A-10 | P1 | OCR `ocr:true` fails: tesseract feature disabled, no server URL | v4AGENT T30 | `crates/folisift-parse/Cargo.toml` uses `liteparse` with `default-features = false`; no `tesseract` feature; `LiteParseConfig.ocr_server_url` is never wired; `doctor` does not report OCR availability. |
| F-V4A-11 | P1 | `source_insights` has no image ids; `visual_extract` needs another call | v4AGENT §4.9 | `extract_insights` serializes `image_refs.len()` only; `ImageRef` metadata (`id`, bbox, format, pixel dimensions) is discarded. |
| F-V4A-12 | P1 | `visual_decision decorative` becomes unresolved, especially with high priority | v4AGENT §4.9 | `VisualAudit` rejects every `priority=high` + `decorative` entry; the cover/front-matter use case needs a documented exception. |
| F-V4A-13 | P2 | `cards` can silently drop recall blocks | v4AGENT §4.9 | `cards` counts definitions but does not warn when a guide has no `::: recall`/`::: answer`; no minimum-content warning. |
| F-V4A-14 | P2 | 4-column glossary page-break/group-row continuity is not regression-tested | our review | visual is correct; no fixture proves a page boundary or continuation row. |

---

## 3. Work packages

### WP0 — Reproduction fixture and truth baseline

**Goal:** stop fixing from prose. Build a small fixture that reproduces every
v4AGENT failure without the user's private workspace.

**Files**

- `crates/folisift-parse/tests/fixtures/manifest-v4/` (synthetic inventory +
  Markdown + TOC text)
- `crates/folisift-cli/tests/mcp_stdio.rs` (path matrix)
- `scripts/check-feedback-v4-agent.py` (A-V4A harness; distinct from the
  existing `check-feedback-v4.py`)
- `Makefile` target `feedback-v4-agent`; `scripts/acceptance.sh` runs it

**Fixture cases**

1. A source with body-contaminated headings and a missing `4.4.4`/`10.2` in
   `headings` but present in TOC text.
2. A guide with `## 2.1–2.2 …` and `<!-- covers: SRC:2.1–2.2 -->`.
3. A manifest with `status:"excluded"` + `reason`.
4. MCP source arguments in all four forms: `PD0`, `PD0.pdf`,
   `sources/PD0.pdf`, `data/blocks/PD0.json`.
5. A `visual-audit` with raw layout blocks that would produce 82 pages, and a
   candidate manifest that produces 4 real pages.
6. A `warn` report with missing figure decisions that must say `status:"advisory"`,
   not `passed:true`.

**Exit:** failing tests before fixes; each failure names its finding ID.

---

### WP1 — Canonical generated manifest (F-V4A-01)

**Files:** `crates/folisift-parse/src/manifest.rs`,
`crates/folisift-parse/src/inventory.rs`, cache/source-read helpers as needed.

**Change**

- Replace `candidates(source)` with a layered canonicalizer:
  1. layout headings with `level`/`block_id` and a title-like cleaned heading;
  2. TOC/canonical sections;
  3. numbered source lines (`^\d+(\.\d+)+ .+`) from cached source text;
  4. `<!-- covers: … -->` only as review evidence, never as a generated row.
- Canonical row key:
  - `source:<number>` when the section has a number;
  - otherwise a normalized, title-like heading slug from TOC/heading data;
  - never raw body prose, never a sentence fragment.
- `clean_heading_title` must stop at sentence punctuation, cap at 120
  characters / 18 words, and reject body markers (`Bu`, `Bir`, `Amaç`, `Örnek`,
  `Şekil`, `Tablo`, `Kaynak`, `Not`).
- Missing nested numbers from TOC/text are additive; do not replace a better
  layout row.
- Deduplicate by `(source, number)` with origin priority
  `heading > toc > text`; keep the first page and non-empty title.
- `canonical_total` is the union of unique canonical keys; `4.4.4` and `10.2`
  must be present when the source text/TOC contains them.
- `validate generated` reports `canonical_total`, `claimed`, `missing` and
  `excluded_total` separately.

**Tests**

- Body-contaminated heading never becomes a row.
- `4.4.4` and `10.2` generated from TOC/text.
- Duplicate numbers deduped by number, origin priority respected.
- `validate generated` on the fixture is 138/138 (or the documented canonical
  total + explicit `excluded`).
- Every generated row has `source`, canonical key, `origin`, page/section.

**Exit:** A-V4A-01.

---

### WP2 — One recursive output-budget function (F-V4A-02)

**Files:** `crates/folisift-mcp/src/lib.rs`, all tools using `shape_response`.

**Change**

- Introduce one `shape_tool_output(value, OutputOptions)` helper:
  - `fields` projection recurses through objects **and arrays**; a path such as
    `per_source.sections.number` applies the remaining path to each array item;
  - `max_tokens` truncates the largest nested arrays and strings deterministically;
  - every truncated array gets `{total, returned, omitted, next}`;
  - `detail=summary` truncates arrays to a documented keep count;
  - summary/read fast paths must not bypass it.
- Apply it to `coverage_manifest` (`read`, `validate`, `review`, `hint`), `hint`,
  `guide_plan`, `visual_candidates`, `claims`, `graph`, `qa` and any other tool
  with nested arrays.
- Add a size guard: response contains `truncated`, `max_tokens`,
  `omitted_total` when the budget is hit.
- Keep top-level `guide`, `passed`, `total` and `next` stable for existing
  clients; add fields, do not rename.

**Tests**

- `validate` with `detail:"summary", max_tokens:50` returns a small object.
- `fields:["per_source.sections.number"]` keeps only that nested key.
- `fields:["missing"]` does not leak `per_source`.
- `max_tokens` never returns a payload larger than `max_tokens * 4 + overhead`.
- `review_diff` is subject to the same budget.

**Exit:** A-V4A-02.

---

### WP3 — Hint ranges and covers (F-V4A-03)

**Files:** `crates/folisift-parse/src/hint.rs`, `manifest.rs`,
`crates/folisift-mcp/src/lib.rs`.

**Change**

- Parse `<!-- covers: SRC:2.1–2.2 -->`, comma lists (`2.1,2.2`), en/em dash and
  ASCII hyphen; return a set of `(source, number)`.
- Share one `covers_index` implementation between `manifest` and `hint`; stop
  duplicating regexes.
- Share one `row_number(row)` helper:
  - prefer the canonical number prefix from `row.normalized_heading`;
  - fall back to `row.heading`'s first token;
  - return `None` for body prose, never a guessed number.
- In `similarity`, check the **range before** normalization and also after
  extracting `number` from the manifest row; if a manifest row is `Covered`,
  force `score=1.0` even if the lexical score is lower.
- `hint` uses reviewed manifest rows when present, and never emits
  `likely-missing` for a row whose number is inside a guide range or covers
  comment.
- Keep `likely-missing` advisory; it must not be treated as proof.

**Tests**

- `2.1–2.2` marks both `2.1` and `2.2` covered with score `1.0`.
- `3.1-3.4` ASCII hyphen and `2.1,2.2` comma list both work.
- A single-number heading still matches.
- A covered reviewed-manifest row overrides a weak lexical score.

**Exit:** A-V4A-03.

---

### WP4 — Shared path resolver for source blocks and diagrams (F-V4A-04, F-V4A-05)

**Files:** `crates/folisift-parse/src/layout.rs` or a new
`source_path.rs`; `crates/folisift-mcp/src/lib.rs`; diagram module.

**Change**

- Add `resolve_source_id(source) -> Result<String>`:
  - accepts `PD0`, `PD0.pdf`, `sources/PD0.pdf`, `data/blocks/PD0.json`;
  - strips directories, `.pdf` and `.json`;
  - rejects empty names, `..`, absolute paths and path separators after
    normalization.
- Add `resolve_block_path(guide, source) -> Result<PathBuf>`:
  - returns `guide.dir/data/blocks/<id>.json`;
  - errors with the accepted forms when missing.
- Use it in `read_blocks`, `read_section`, `visual_brief` block lookup and any
  other block-backed tool. `visual_candidates` keeps the workspace PDF resolver;
  it does not read blocks.
- Diagram MCP:
  - `validate` with a `spec` and no guide is legal (pure spec validation);
  - `validate` without a spec needs a guide to read stored specs; if neither is
    present, return `diagram validate needs spec or guide`;
  - `save`, `list`, `status`, `fit`, `render_plan` require a normalized guide
    slug;
  - accept `guides/<slug>` and a trailing slash as a convenience, strip the
    prefix, then call `validate_slug`; never pass a path to `Guide::load`;
  - `list`/`status` use the same resolver as the CLI; no `root/<slug>` double
    path.

**Tests**

- `read_section` with all four source forms returns the same content.
- `diagram validate` with a spec/no guide passes.
- `diagram list`/`status` with `guide:"pd3"` works under repo root;
  `guides/pd3` also resolves.
- `diagram save` still requires a guide and writes under the resolved bundle.
- MCP stdio path matrix.

**Exit:** A-V4A-04, A-V4A-05.

---

### WP5 — `guide_plan.source_pages` (F-V4A-06)

**Files:** `crates/folisift-mcp/src/lib.rs`, `inventory.rs`, `manifest.rs`.

**Change**

- Build a normalized `source number → pages` index from:
  1. `source.sections` (number + page);
  2. `source.headings` (number prefix + page);
  3. `source.page_map` (first line + page) when the heading text is wrapped;
  4. `<!-- covers: … -->` ranges;
  5. reviewed manifest rows.
- Match guide heading numbers after trimming trailing dots and normalizing
  separators; for `2.1–2.2`, map both source numbers to their pages.
- Drop page `0` (TOC-only rows); return `source_pages` sorted/deduped. If no
  nonzero mapping exists, return `[]` plus `source_pages_unknown: true` for
  that row rather than silently empty.
- Add `source_page_ranges` for merged headings and a top-level
  `source_pages_unknown` count.

**Tests**

- PD0–PD1 0.x/1.x guide: no empty `source_pages` where the inventory has a
  heading.
- `2.1–2.2` guide heading maps both pages.
- A guide with no source page mapping gets `source_pages_unknown: true`.
- Guide headings with trailing dots or ASCII hyphens still match.

**Exit:** A-V4A-06.

---

### WP6 — Candidate-backed visual gate and advisory truth (F-V4A-07, F-V4A-08)

**Files:** `crates/folisift-core/src/config.rs`,
`crates/folisift-parse/src/visual_audit.rs`, `visual_candidates.rs`,
`crates/folisift-build/src/lib.rs`, `crates/folisift-mcp/src/lib.rs`,
`crates/folisift-cli/src/main.rs`, `docs/authoring.md`.

**Change**

- Add `visual candidates --write` (MCP `write:true`) producing
  `guides/<slug>/data/visual-candidates.json`:
  - deterministic envelope: `schema_version`, `scorer_version`,
    `generated_from`, `sources[]`;
  - each source has `source`, `source_digest`, `pages` and `candidates[]`;
  - each candidate has `id`, `page`, `score`, `relevance.label`,
    `reconstruction.kind`, `bbox`, `suggested`, `kind`, `image_id`;
  - `--write` upserts the source entry by source id, preserves other sources
    and writes atomically; PD0–PD1 runs the command once per source;
  - no image bytes, no wall-clock timestamp;
  - update the MCP annotation: `read_only_hint=false`, `idempotent_hint=true`,
    `destructive_hint=false`, because the tool can write the manifest;
  - `--write` always persists the full report, independent of `detail`/`limit`;
    the returned response may still be bounded.
- Thread the policy explicitly: `VisualAudit::validate_with_policy` takes a
  small `VisualGatePolicy` (or `visual_min_score`) plus the candidate-manifest
  path; `folisift-build` passes the guide config, and CLI/MCP QA use the same
  helper.
- `visual_audit::figure_pages` reads the candidate manifest when present:
  - gate pages = candidates with `relevance.label=core` and
    `score >= audit.visual_min_score`, plus candidates with
    `reconstruction.kind=table` and the same score floor;
  - low-text pages remain mandatory even when no candidate exists;
  - no raw page-spanning `figure`/`table` blocks;
  - update the entry check/message from “not a low-text page” to “not an
    expected visual page”, because candidate pages now enter the set.
- Legacy path (no manifest) remains for `warn`, but the report marks
  `basis:"layout-blocks"` and `advisory:true`.
- Missing/stale manifest under `visual_gate=required` fails with
  `folisift visual candidates --guide <slug> --source <path> --write` as the
  next step.
- `VisualReport` adds `basis`, `advisory`, `candidate_manifest`,
  `missing_candidates`, `expected_pages`.
- Keep the build gate page-level: `missing_figures` blocks under `required`.
  `missing_candidates` is advisory in `qa`; `visual assets --check` remains the
  strict per-candidate preflight.
- `qa.visual` / `qa_guide.visual` expose:
  - `visual_gate`: `off|warn|required`;
  - `enforced`: bool;
  - `status`: `passed|advisory|failed`;
  - `advisory`: bool;
  - `basis`: `candidates|layout-blocks|none`;
  - `missing_figures` and `missing_candidates`.
- Add `audit.visual_min_score` to `AuditPolicy`:
  - integer `0–100`, default `50`;
  - validated in `Guide::validate` with a clear config error;
  - documented in `docs/authoring.md` and `docs/cookbook.md`.

**Tests**

- PD0/PD2/PD3 `visual_gate=required` with a written candidate manifest:
  expected pages are the reviewed candidate pages, not 82/28/136.
- Writing a second source upserts it and preserves the first source's entry.
- A stale source digest fails with an actionable error.
- `warn` report says `status:"advisory"`, not plain `passed:true`.
- `visual_min_score=80` reduces the expected set; `0` includes all core
  candidates.
- Existing low-text hard gate unchanged.
- Missing manifest under `required` fails before rendering.

**Exit:** A-V4A-07, A-V4A-08.

---

### WP7 — `excluded` status (F-V4A-09)

**Files:** `manifest.rs`, MCP/CLI review commands, docs.

**Change**

- Add optional `reason: Option<String>` to `ManifestRow`.
- `CoverageStatus::Excluded` is produced by:
  - a review author marking a row `excluded` + `reason`;
  - a migration tool that converts deleted rows in the old review into
    explicit `excluded` rows;
  - combine logic never silently drops a generated row: absent review rows are
    `missing`, not deleted.
- `report()` counts `excluded`; `missing` must not include excluded; `passed`
  ignores them; `validate` reports `excluded_total` and lists them separately.
- Keep `review_eligible()` accepting excluded rows and add an explicit test;
  `adopt` preserves them.
- Docs: `excluded` is for content intentionally out of scope, not for “not done
  yet”; `missing` is still the only failure.

**Tests**

- A review with one covered + one excluded + one missing: excluded counted,
  missing 1, passed false; after covering the missing row, passed true.
- Adopt preserves excluded rows.
- Old review files without `reason` remain valid.
- Combine does not delete an unreviewed generated row.

**Exit:** A-V4A-09.

---

### WP8 — OCR availability (F-V4A-10)

**Files:** `Cargo.toml`, `crates/folisift-parse/Cargo.toml`,
`crates/folisift-mcp/Cargo.toml`, `crates/folisift-cli/Cargo.toml`,
`crates/folisift-parse/src/insights.rs`,
`crates/folisift-cli/src/main.rs`, `crates/folisift-mcp/src/lib.rs`,
`scripts/acceptance.sh`, `Makefile`, `docs/install.md`, `docs/cookbook.md`,
`docs/agents/versioning.md`.

**Decision:** keep the default portable build without Tesseract; add an
**opt-in `ocr-tesseract` feature** and a documented local/server path.

**Change**

- Feature wiring:
  - `crates/folisift-parse/Cargo.toml`:
    `ocr-tesseract = ["liteparse/tesseract"]`;
  - `crates/folisift-mcp/Cargo.toml`:
    `ocr-tesseract = ["folisift-parse/ocr-tesseract"]`;
  - `crates/folisift-cli/Cargo.toml`:
    `ocr-tesseract = ["folisift-parse/ocr-tesseract",
    "folisift-mcp/ocr-tesseract"]`;
  - build/run:
    `cargo build -p folisift-cli --features ocr-tesseract` and
    `cargo run -p folisift-cli --features ocr-tesseract -- insights
    --ocr ...`.
- Wire `ocr_server_url`:
  - add optional `ocr_server_url` to `SourceInsightsParams` (MCP
    `source_insights`) and the CLI `insights` command;
  - pass it into `LiteParseConfig.ocr_server_url`;
  - document the HTTP OCR server contract and `FOLISIFT_OCR_SERVER_URL`.
- `doctor` reports:
  - `ocr: "disabled" | "tesseract" | "server"`;
  - compile-time feature state, resolved tesseract binary/version, or server
    URL presence;
  - “OCR unavailable” with a next step when neither is available;
  - `ok:true` because OCR is optional; availability is separate from the
    required environment checks.
- `source_insights ocr:true` when unavailable returns a structured
  `OCR_UNAVAILABLE` error (typed core error; JSON message payload) with
  suggestions: install tesseract + build with `ocr-tesseract`, or pass
  `ocr_server_url`.
- Never auto-enable OCR; never download models at runtime.
- Keep the portable gate portable: `scripts/acceptance.sh` and
  `docs/agents/versioning.md` must not run `--all-features` once the OCR feature
  exists, because `liteparse/tesseract` pulls `tesseract-rs` with
  `build-tesseract`. Use the default feature set for the normal gate and a
  separate OCR CI job with the required native toolchain.

**Tests**

- Default `cargo test` retains the structured `OCR_UNAVAILABLE` error.
- CI builds `--features ocr-tesseract` on a runner with the tesseract build
  toolchain and runs one page test, or marks it skipped with an explicit
  environment reason.
- The portable `make acceptance` gate stays green without tesseract installed.
- `doctor` never claims OCR is available when it is not.
- Server URL path passes the URL into `LiteParseConfig`.

**Exit:** A-V4A-10.

---

### WP9 — Small visual and cards semantics (F-V4A-11, F-V4A-12, F-V4A-13)

**Files:** `crates/folisift-parse/src/insights.rs`,
`crates/folisift-parse/src/visual_audit.rs`,
`crates/folisift-parse/src/visual_workflow.rs`,
`crates/folisift-core/src/cards.rs`, `crates/folisift-build/src/lib.rs`,
`crates/folisift-cli/src/main.rs`, `crates/folisift-mcp/src/lib.rs`.

**Change**

- `source_insights`: add `image_refs_detail` per page with `id`, `bbox`,
  `format`, `pixel_width`, `pixel_height`, `rotation`, `bits_per_pixel`,
  `colorspace`; no bytes. `visual_extract` can use `id` directly.
- `visual_decision decorative`:
  - a `priority=high` + `decorative` entry is allowed only when the reason or
    notes says it is a cover/front-matter decision (`kapak`, `ön söz`,
    `cover`, `front-matter`) and the audit reason is non-empty;
  - a missing reason stays **unresolved**;
  - a present reason that is not cover/front-matter emits a **warning**, not
    `unresolved`.
- `visual_decision embed_crop` with `candidate_id`:
  - `crop` is optional at decision time because it is derived from the
    candidate bbox; the stored audit entry must contain the derived crop;
  - direct hand-editing of `visual-audit.json` still requires `crop`.
- `cards` (CLI/MCP) and `qa.cards`: if `include_definitions` is true and
  `recall=0` while `definitions>0`, add
  `warning:"no recall blocks; only definition cards"`, plus `recall`,
  `definitions`, `total`; do not fail.

**Tests**

- `image_refs_detail` contains `p76_1` with bbox and pixel dimensions.
- Cover decorative high-priority passes with a cover reason; high+decorative
  without a cover reason warns but does not become unresolved.
- `visual_decision embed_crop` with only `candidate_id` writes a crop into the
  audit and validates.
- Cards warning appears only when `recall=0` and `definitions>0`.

**Exit:** A-V4A-11, A-V4A-12, A-V4A-13.

---

### WP10 — Glossary regression (F-V4A-14)

**Files:** `crates/folisift-render/src/markdown.rs` tests,
`tests/baselines`, `docs/design-system.md`.

**Change**

- Add a page-break/group-row fixture: 30+ terms across two groups, a group row
  at a page boundary, and a continuation table header.
- Assert one `#glossary_table`, four columns, `__TABLE_SIZE__`, leading
  `0.55em`, full-width group rows; capture a visual baseline.
- Keep `docs/design-system.md` statement as one four-column table (already
  corrected in `386194a`).

**Exit:** A-V4A-14.

---

## 4. Acceptance matrix

One row per finding, then two cross-cutting gates.

| ID | Finding | Check | Pass condition |
|---|---|---|---|
| A-V4A-01 | F-V4A-01 | Generated canonical rows | Body-free keys; `4.4.4`/`10.2` present; `validate generated` 138/138 or documented canonical + excluded; each row has source/key/origin/page |
| A-V4A-02 | F-V4A-02 | Nested budget | `max_tokens=50`, `fields=per_source.sections.number` bounded; `total/returned/omitted/next` present; no nested leak |
| A-V4A-03 | F-V4A-03 | Hint ranges | `2.1–2.2`, `3.1-3.4`, comma covers → covered, score 1.0; reviewed covered row wins |
| A-V4A-04 | F-V4A-04 | Source path forms | `PD0`, `PD0.pdf`, `sources/PD0.pdf`, `data/blocks/PD0.json` equivalent |
| A-V4A-05 | F-V4A-05 | Diagram MCP | `validate` spec-only works; `list/status` with `pd3` and `guides/pd3`; no double path; `save` still requires guide |
| A-V4A-06 | F-V4A-06 | Guide plan pages | `source_pages` non-empty for 0.x/1.x; merged range maps both; unknown flag explicit |
| A-V4A-07 | F-V4A-07 | Visual gate required | PD0/PD2/PD3 candidate manifest; expected pages are real candidates, not 82/28/136; stale/missing fails actionably |
| A-V4A-08 | F-V4A-08 | Visual gate advisory | `warn` reports `status:"advisory"`, `enforced:false`, `advisory:true`, not plain `passed:true` |
| A-V4A-09 | F-V4A-09 | Excluded | excluded counted, not missing; adopts; old files valid; combine never deletes unreviewed rows |
| A-V4A-10 | F-V4A-10 | OCR | doctor reports availability; unavailable error is structured; feature build passes; server URL wired |
| A-V4A-11 | F-V4A-11 | Image refs | `image_refs_detail` has ids/bbox/dims; `visual_extract` uses it |
| A-V4A-12 | F-V4A-12 | Decorative semantics | cover decorative high-priority allowed with reason; otherwise warning, not unresolved |
| A-V4A-13 | F-V4A-13 | Cards warning | recall=0 + definitions>0 → warning, not silent |
| A-V4A-14 | F-V4A-14 | Glossary | one table, four columns, group rows, page-break fixture passes |
| A-V4A-15 | cross-cutting | Regression | `make acceptance` green, `make feedback-v4-agent` green, `make visuals-v5` green, `cargo test`, `fmt`, portable `clippy`, PDF `audit --strict` clean |
| A-V4A-16 | cross-cutting | v4AGENT matrix | 26 PASS / 7 FAIL → 33 PASS, or an explicit documented environment skip (OCR) |

Manual checks:

- Render PD0–PD1, PD2, PD3 after changes; compare the glossary and figure pages
  visually against the current baselines.
- Run `qa.visual` on all three and confirm the numbers are plausible against
  actual figure captions.
- Inspect one `warn` and one `required` guide side by side; the status string
  must tell the host whether a figure gap is advisory or blocking.

---

## 5. Order, dependencies, and effort

| Phase | Packages | Why first | Estimate |
|---|---|---|---:|
| 1 | WP0, WP1, WP2 | Generated manifest and budget are the critical truth bugs | 3 d |
| 2 | WP3, WP4, WP5 | Hint/path/guide-plan are independent and unblock MCP tests | 2.5 d |
| 3 | WP6 | Candidate-backed gate depends on the candidate report; closes our P1 and F-V4A-08 | 2 d |
| 4 | WP7, WP8, WP9, WP10 | Contract/extensions/docs | 3.5 d |
| 5 | Acceptance, visual baselines, release | 0.22.1 then 0.23.0 | 2 d |

Rough total: **13 engineering days**. WP1/WP2 can run in parallel with WP3–WP5
after WP0.

---

## 6. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Generated manifest changes canonical totals across guides | Keep explicit `canonical_total`, add a diff report, migrate reviewed manifests explicitly. |
| `fields` projection breaks existing clients | Additive keys only; keep top-level `guide`, `passed`, `total`, `next`; tests for old shapes. |
| Visual candidate manifest goes stale | Source digest + scorer version; `required` fails with `visual candidates --write`; `warn` marks advisory and falls back to layout blocks. |
| `warn` fallback stays noisy | Report `basis:"layout-blocks"` and `advisory:true`; never present raw-page gaps as enforced failures. |
| `excluded` hides real omissions | `reason` required; `excluded` never counts as `missing` but is reported separately; review diff shows it. |
| OCR feature bloats the portable build | Opt-in feature; default binary reports unavailable; doctor is explicit; no runtime download. |
| Diagram `guides/<slug>` convenience conflicts with slug validation | Normalize before `Guide::load`; never pass a path to `validate_slug`. |
| 4-column glossary regresses on page breaks | Fixture + baseline; no design change in this plan. |
| Schema 18 is a contract bump even though the change is additive | Keep old fields, make new params optional, keep output schemas permissive; document schema 18 as backward-compatible; if a change becomes breaking, move it to 0.23.0. |

---

## 7. Release plan

1. Implement 0.22.1 P0: WP0–WP6 (F-V4A-01…08). No P1 carry-over.
2. Run `make acceptance`, `make feedback-v4-agent`, `make visuals-v5`,
   `cargo test`, `cargo fmt`, and
   `cargo clippy --workspace --all-targets -- -D warnings` (portable feature
   set). Run the OCR feature job separately.
3. Rebuild PD0–PD1/PD2/PD3; `audit --strict`; inspect glossary/figure pages.
4. Release **0.22.1 / expected build 231 / MCP schema 18**. The build number is
   derived from `git rev-list --count HEAD`; bump the workspace version and
   `crates/folisift-core/src/version.rs` schema. Update `docs/VERSIONS.md`,
   `README`, `docs/agents/versioning.md`.
5. Tag `v0.22.1` locally; do not push unless explicitly authorized.
6. Implement 0.23.0 P1/P2: WP7–WP10 (F-V4A-09…14). Release **0.23.0 /
   expected build 232 / MCP schema 19** if the tool contract changes (it does:
   `reason`, image-ref detail, decorative semantics, OCR params).
7. Repeat acceptance; tag only after the maintainer approves.
8. Claim 10/10 only when A-V4A-15 and A-V4A-16 are green; an OCR environment
   skip must be recorded in the matrix.

---

## 8. Open decisions

1. **OCR:** opt-in `ocr-tesseract` feature (recommended) vs enabling Tesseract
   by default. The default build must stay portable.
2. **Visual gate manifest:** store under `data/visual-candidates.json`
   (recommended) vs recompute on every build. Stored manifest is needed for
   deterministic `required` gates and QA.
3. **Version split:** 0.22.1 for all P0 (recommended) vs folding everything into
   0.23.0.
4. **`excluded` schema:** optional `reason` on `ManifestRow` (recommended,
   backward-compatible) vs a separate excluded file.
5. **Visual threshold:** `audit.visual_min_score` default `50` (recommended) vs
   `40`. The value must be validated and documented either way.

---

## Appendix A — MCP guidance delivery (research note)

This appendix records how Folisift currently sends guidance to an LLM and what
the best-practice target is. It is not a v4 defect; it is a contract decision
for 0.23.0 or later.

### A.1 Current delivery (0.22.0)

| Layer | Current format | Where |
|---|---|---|
| Server instructions | One plain-text sentence, ~130 characters | `ServerConfig::with_instructions` in `crates/folisift-mcp/src/lib.rs` |
| Tool descriptions | Plain English strings compiled from Rust doc comments | `#[tool(description = …)]`; 41 tools |
| Input schema | JSON Schema (generated) | `Parameters<T>` / `schemars` |
| Tool results | JSON (`Json<Value>` → `structuredContent` plus a serialized-JSON text block) | rmcp `Json<T>` wrapper |
| Prompts | Two long numbered plain-text workflows (`write-study-guide`, `write-diagram`) | `get_prompt` |
| Resources | Markdown (`authoring-spec`, `design-system`, guide/outline), JSON (`capabilities`, `diagram-schema`) | `list_resources`/`read_resource` |

So the answer is: **not one `.md` file and not one JSON file**. Folisift uses a
mix: a short plain-text server instruction, natural-language tool descriptions,
JSON Schema for arguments, JSON structured results, plain-text prompts and
Markdown/JSON resources.

### A.2 What the protocol says

- MCP tools are **model-controlled**; prompts are **user-controlled**; resources
  are **application-driven** ([MCP server overview](https://modelcontextprotocol.io/specification/2026-07-28/server)).
- Server `instructions` are optional **natural-language guidance for LLMs on how
  to use the server effectively**
  ([MCP discovery](https://modelcontextprotocol.io/specification/2026-07-28/server/discover)).
- Each tool has a human-readable `description`, a JSON Schema `inputSchema`, an
  optional `outputSchema`; structured results go in `structuredContent`. A tool
  that returns structured content **SHOULD** also return the serialized JSON in
  a text block for compatibility
  ([MCP tools](https://modelcontextprotocol.io/specification/2026-07-28/server/tools)).
- Anthropic's tool-writing guidance recommends clear tool boundaries,
  namespacing, meaningful context, token-efficient responses, and
  prompt-engineered descriptions/specs with actionable errors
  ([Writing effective tools for agents](https://www.anthropic.com/research/writing-tools-for-agents)).

### A.3 Recommended layering

Do **not** put the whole authoring manual into `instructions`, and do **not**
send a giant JSON instruction blob. Use five layers:

1. **Server instructions — short plain text/Markdown.**
   Five lines maximum: what the server does, the first call, the write
   boundary, the gate truth, and “read the spec resource before authoring”.
   This is global context; every extra paragraph costs tokens in every session.

2. **Tool descriptions and input schemas — precise natural language + JSON
   Schema.**
   Each description says when to use the tool, what it returns, what the next
   step is, and what the error means. Enums, ranges and required fields belong
   in the schema, not in prose. This is the layer the model reads when choosing
   a tool.

3. **Structured results — JSON `structuredContent` plus a concise text
   summary.**
   Keep exact fields (`passed`, `missing`, `next`, `omitted`) in JSON; make the
   text block a short human/LLM-readable summary and next action, not a raw JSON
   dump. Raw JSON is precise but token-heavy and harder to scan; prose alone is
   readable but loses machine-checkable fields. The current rmcp `Json<T>`
   wrapper gives JSON plus serialized JSON text; the target is JSON plus a
   one-paragraph summary for the high-traffic tools (`coverage_manifest`,
   `guide_plan`, `qa_guide`, `build_guide`).

4. **Prompts — workflow templates in plain text/Markdown.**
   Keep `write-study-guide` and `write-diagram` as user-invoked recipes. Add a
   short `start-here` prompt that names the first three calls and the resources
   to read. Prompts are not a substitute for tool descriptions because clients
   do not auto-inject them.

5. **Resources — long reference documents.**
   Markdown for normative prose (`authoring-spec`, `design-system`,
   `diagram-rules`); JSON Schema for machine contracts (`diagram-schema`,
   `capabilities`). Resources are loaded on demand, so they can be detailed
   without polluting every session.

### A.4 Why not one format

- **One giant Markdown instruction**: readable, but every session pays the
  token cost, clients may truncate it, and it mixes global rules with
  tool-specific details that belong in descriptions.
- **One giant JSON instruction**: machine-parseable, but poor for nuanced
  procedural guidance, full of escaping overhead, and harder for an LLM to
  follow than short prose.
- **Only tool descriptions**: good for selection, but no place for
  cross-tool invariants (write boundaries, gate truth, “never edit sources”).
- **Only prompts**: prompts are user-controlled; a host may never invoke them.

### A.5 Concrete target (optional 0.23.0 follow-up)

If approved, add a small WP11:

- Replace the one-sentence server instruction with a five-line plain-text
  contract.
- Add a `start-here` prompt and link the three resources by URI.
- Add concise text summaries to `coverage_manifest`, `guide_plan`, `qa_guide`
  and `build_guide` while keeping `structuredContent` unchanged.
- Add `outputSchema` to those four tools if rmcp does not already emit it.
- Add a snapshot test: server instruction ≤ 600 characters; every tool
  description non-empty; every resource URI referenced by the prompt exists.

This is additive; it does not change the v4 acceptance matrix.
