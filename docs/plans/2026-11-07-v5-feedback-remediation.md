# V5 feedback remediation — detailed plan

- **Status:** Proposed — no code changes yet
- **Date:** 2026-11-07
- **Baseline:** Folisift 0.23.2, build 248, MCP schema 19, 41 tools
- **Primary source:** [`FOLISIFT-GERI-BILDIRIM-v5.md`](../FOLISIFT-GERI-BILDIRIM-v5.md) — 30 PASS / 9 PARTIAL / 2 FAIL
- **Reproduction workspace:** `/var/tmp/opencode/v5-repro` (copy of `/home/yilmaz/Documents/DersNot`; never committed)
- **Target releases:**
  - **0.23.3** — P0/P1 hotfix: scaffold outline, coverage canonical set, `read_source` budget, inline headings, `guide_status`, `visual_extract`, `verify_citations`, visual status.
  - **0.24.0** — P2/P3 contract cleanup: path/ID normalization, packaged resources, `visual_decision` validation, cards parity, style matrix.
- **Definition of done:** every acceptance row in §6 is green on a fresh build and on the external reproduction workspace, `cargo test`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all -- --check` pass, and no `sources/*.pdf` is tracked.
- **Not claimed:** 10/10. The v5 report still has 2 FAIL and 7 PARTIAL rows; the score may only be raised after the acceptance matrix is green and the external probes are rerun.

---

## 0. Short diagnosis

The v5 failures are not thirteen unrelated bugs. They cluster into five contracts
that were never written down:

1. **Greenfield authoring contract is missing.** `prepare_guide` and
   `scaffold_guide` create a guide from an inventory but do not turn the inventory
   into a navigable outline. The same `X.Y`-only filter is duplicated in MCP and
   CLI, and both emit a `::: front` page that the strict audit reads as a second
   chapter opener.
2. **Coverage manifest has three different row universes.** Generated rows use
   heading + TOC + body text, review rows use curated numbers, and `review_diff`
   uses a third normalized-title key. `review_eligible` then evaluates the merged
   file, so generated `missing` rows permanently block adoption.
3. **Response shaping is destructive.** `shape_response` is allowed to shrink a
   two-item array to `keep=0`, so `read_source` reports `returned_pages:2` while
   returning no page text. The same helper backs every MCP output.
4. **Source heading resolution is layout-kind-specific.** `source_sections` only
   starts a section at `LayoutBlockKind::Heading`; headings embedded in a
   paragraph (`4.4.4`) or a table cell (`10.2`) are invisible to all four brief
   tools even though the inventory and generated manifest already contain them.
5. **Ambiguous result schemas.** `missing` means "all unreferenced claims" in
   `verify_citations`, `status:"passed"` means "nothing failed" while
   `visual_gate:"warn"` means "not enforced", and `cards` excludes definitions
   while `qa_guide` includes them.

The fix order is therefore contract-first: define the row universe, the heading
universe, the budget contract and the status vocabulary; then make the tools use
the shared implementation.

---

## 1. Verified findings and root causes

Every row below was reproduced from code and from the external workspace; the
report was treated as a hypothesis, not as proof.

| ID | Pri | Finding | Root cause (code) | Fix owner |
|---|---|---|---|---|
| V5-01 | P0 | `prepare_guide` / `scaffold_guide` produce one chapter, `outline.items:[]` | `skeleton_markdown` and the `outline` loop both accept only numbers with exactly one dot (`matches('.').count() == 1`): `crates/folisift-mcp/src/lib.rs:2043-2058`, `:5574-5588`; the CLI duplicates the same filter in `crates/folisift-cli/src/main.rs:1500-1520` | WP0.1 |
| V5-02 | P0 | `::: front` causes strict audit `chapter opener split [2,3]` | `crates/folisift-core/src/scaffold.rs:174-183` writes a front page before the first chapter; `detect_chapter_splits` flags two consecutive `ChapterOpener` pages (`crates/folisift-parse/src/audit.rs:826-845`) | WP0.1 |
| V5-03 | P0 | Digital `coverage_manifest build` has 93 rows: `{heading:51,toc:28,text:14}` | `candidates()` ranks TOC above layout and raw body text above layout (`crates/folisift-parse/src/manifest.rs:370-488`); the raw-text regex accepts decimals and measures (`1.0`, `10.500`, `300.000`) and table-cell noise; `title_like` accepts `) Birim...` and `40 x azaldı*` | WP0.2 |
| V5-04 | P0 | `review_diff` reports `only_in_generated:92`, `only_in_review:40` for the same 41 rows | generated diff keys use `source:normalized_heading` while the row key uses `source:number`: `crates/folisift-mcp/src/lib.rs:3650-3668` vs `crates/folisift-parse/src/manifest.rs:641-646` | WP0.2 |
| V5-05 | P0 | `adopt:true` returns `adopted:false`; `review` then says "no coverage manifest to review" | `review_eligible` is computed on `generated.merge_with_review(review)` (`crates/folisift-mcp/src/lib.rs:3636-3643`); generated `Missing` rows survive the merge (`manifest.rs:277-303`) and make eligibility false. `build` never writes a review file when one is absent | WP0.2 |
| V5-06 | P1 | `read_source(max_tokens:200, pages:"1-2")` returns `returned_pages:2` but `pages:[{omitted:2,returned:0}]` | `read_source` ignores `max_tokens` when choosing the read budget (`crates/folisift-mcp/src/lib.rs:800-815`); `shrink_to_budget` collapses a two-item array to `keep=0` (`:5176-5221`); `read_from_source_at` clamps `max_chars` to at least 1000 (`crates/folisift-parse/src/source_read.rs:93-95`) | WP0.3 |
| V5-07 | P1 | `4.4.4` and `10.2` are "section not found" in all four brief tools | `source_sections` only handles `LayoutBlockKind::Heading` (`crates/folisift-parse/src/brief.rs:88-135`); `4.4.4` is inside paragraph `is-hukuku:p14:b5`; `10.2` is inside table cell `is-hukuku:p44:b0`; `recover_inline_headings` only scans paragraph/list/unknown (`crates/folisift-parse/src/parser.rs:255-293`) | WP1.1 |
| V5-08 | P1 | `guide_status.empty_sections:["3 kaynaklar","3"]` while `check_coverage.empty_sections:[]`; `inventory` path does not exist | `guide_status` uses `guide_body_chars` + `MIN_SECTION_CHARS < 400` (`crates/folisift-mcp/src/lib.rs:684-713`); the coverage gate uses `body_chars == 0` (`crates/folisift-parse/src/coverage.rs:159-167`); scaffold advertises `data/inventory.json` (`crates/folisift-core/src/scaffold.rs:79`) but neither `prepare_guide` nor `scaffold_guide` writes it | WP1.2 |
| V5-09 | P1 | `visual_extract out:"...png"` writes JPEG bytes, `format:"jpg"` | `extract_native` writes `image.bytes` unchanged and only uses the extension when `out` is absent (`crates/folisift-parse/src/visual_assets.rs:116-147`); hash and quality are computed on the native bytes, not the written file | WP1.3 |
| V5-10 | P1 | `verify_citations`: `checked:5`, `missing:75` | `missing` is `report.unreferenced` (all guide claims), `checked` is `rows.len()` (the first `max_claims`): `crates/folisift-mcp/src/lib.rs:3249-3257` | WP1.4 |
| V5-11 | P2 | `visual_gate:"warn"` and `visual.status:"passed"` coexist | build sets `status` to `passed` whenever `missing_figures` is empty, regardless of `visual_gate` (`crates/folisift-build/src/lib.rs:335-345`); `VisualReport` has no separate gate verdict (`crates/folisift-parse/src/visual_audit.rs:103-133`) | WP1.5 |
| V5-12 | P2 | Path/ID contracts differ across tools | `read_source`, `search_source`, `analyze_source`, `visual_*` call `Workspace::resolve_read(input)` directly and require `sources/x.pdf`; `read_blocks` and the brief tools use `source_id()` which accepts `PD0`, `PD0.pdf` and `sources/PD0.pdf` (`crates/folisift-parse/src/layout.rs:294-330`); `analyze_source.out` is workspace-write-only by design but the error does not name the writable roots | WP2.1 |
| V5-13 | P2 | `folisift://authoring-spec` and `folisift://design-system` are broken outside the repo | `read_resource` calls `read_workspace_markdown("docs/...")` (`crates/folisift-mcp/src/lib.rs:4356-4357`, `:5774-5781`); docs are not packaged with the installed binary | WP2.2 |
| V5-14 | P2 | `visual_decision embed_crop` validates one missing argument at a time | `embed_crop` requires `bbox` first, then `crop` when no candidate (`crates/folisift-parse/src/visual_workflow.rs:188-204`); no aggregate validation pass | WP2.3 |
| V5-15 | P3 | `cards` default 8/5/6, `qa_guide` 9/5/8 | MCP/CLI `cards` default `include_definitions:false` (`crates/folisift-mcp/src/lib.rs:4128-4140`; `crates/folisift-cli/src/main.rs:84-95`); build `qa` passes `true` (`crates/folisift-build/src/lib.rs:599-600`) | WP3.1 |
| V5-16 | P3 | `clean+law` and `workbook+irac` are only discoverable from the error message | `styles/clean/style.json` domains omit `law`; `styles/workbook/style.json` structures omit `irac`; `doctor` only validates existing guides (`crates/folisift-cli/src/main.rs:933-960`) | WP3.2 |

**Row-count note:** V5-03 and V5-04 are one root cause with two symptoms. Fix
them in one commit; splitting the diff would leave `review_diff` lying for a
release.

---

## 2. Root-cause detail

### 2.1 Greenfield scaffold (V5-01, V5-02)

`prepare_guide` does this:

1. `scaffold()` writes `config.json` and a minimal `guide.md` with `::: front`,
   `::: toc` and exactly one `::: chapter`.
2. `analyze_sources_with_layout()` produces an inventory and layouts.
3. If the guide is new, `prepare_guide` overwrites `guide.md` with
   `skeleton_markdown()`.
4. `skeleton_markdown()` emits one chapter per source and only adds a `##`
   heading when the first whitespace token has exactly one dot.

`scaffold_guide` repeats steps 1 and 3. The CLI `scaffold` command has a third
copy of the same skeleton logic. Consequences:

- A source with headings `1`, `2`, `3` or `1.2.3` produces an empty outline.
- A source with `1.1` produces a guide with headings but no chapter-level
  grouping; `outline.items` is the same filtered list.
- `::: front` renders as a title page. The following chapter opener is a second
  `ChapterOpener`; `detect_chapter_splits` flags the pair.
- `source_profile.units.sections` reads `source.sections` (TOC only) and stays
  `0` even when dozens of layout headings exist.

The fix is not "accept every heading". It is one canonical outline builder that
uses the same heading evidence as the coverage manifest and emits a document
structure the renderer and strict audit already understand.

### 2.2 Coverage manifest (V5-03, V5-04, V5-05)

`candidates()` currently inserts evidence in this priority order:

```text
heading (0) > toc (1) > raw source text (2) > layout block/table cell (3)
```

Lower rank wins, but every new number is still inserted. This is why digital
ends at 93 rows. The 28 TOC rows and 14 text rows are mostly false positives:

- `1.0`, `2.0`, `3.0`, `4.0` are decimal list markers, not sections.
- `10.500`, `12.500`, `40.000`, `55.000`, `300.000`, `500.000` are quantities.
- `1 ) Birim Satış Fiyatı...` is a table cell with a leading `)`.
- `40 x azaldı* Dönüşümü...` is a chart label.

The 41 curated review rows are all present in the generated set, but the
generated set also contains 52 extras. `review_diff` cannot show this because it
hashes `normalized_heading` instead of the number key used by `row_key()`.

`review_eligible` has the wrong subject. The review file is the curated truth;
the generated file is a proposal. Eligibility must be evaluated on the review
rows, not on the merged set. Adoption must write the review-backed combined file
and surface generated-only rows as advisory, or explicitly fail with a named
action.

### 2.3 `read_source` budget (V5-06)

Two budgets exist but are unrelated:

- `read_from_source_at()` uses `max_chars` (default 24,000, floor 1,000).
- `shape_response()` uses `max_tokens * 4` (floor 64) and may shrink the result.

With `max_tokens:200`, the read layer returns two full pages (4,429 chars). The
shape layer then tries to fit 800 chars. It finds the `pages` array of length 2,
computes `keep = 0`, replaces it with an omitted marker, and reports
`returned_pages:2` from the untouched top-level field. The result is not a
budget error; it is silent data loss.

The fix has to be in both layers:

1. `read_source` must translate `max_tokens` into the read budget before calling
   the parser, so the parser's offset/next-offset logic can preserve a partial
   first page.
2. `shrink_to_budget` must never reduce a non-empty data array to zero. If the
   array has one element that is too large, trim strings inside that element;
   if it has two, keep one and trim it.

### 2.4 Inline and table-cell headings (V5-07)

`source_sections()` is the single resolver behind `read_section`,
`section_brief`, `heading_brief` and `chapter_brief`. It only starts a section
when `block.kind == LayoutBlockKind::Heading`. The inventory already recovers
inline headings, but only from paragraph/list/unknown blocks. The two failing
cases are:

- `4.4.4. Asıl İşveren - Alt İşveren...` is appended to paragraph
  `is-hukuku:p14:b5`.
- `10.2. UlusalBayram ve GenelTatiller...` is the first cell of table
  `is-hukuku:p44:b0`.

The manifest sees `4.4.4` as `heading` and `10.2` as `text` (the text scan wins
over the layout table scan), which is why `source_brief` works while the four
section tools do not. The fix must make layout table-cell headings first-class
evidence and make `source_sections` split on the same evidence.

### 2.5 `guide_status` and inventory (V5-08)

`guide_status` computes body characters with `guide_section_chars()` and marks
every numeric key below `MIN_SECTION_CHARS` as empty. That function indexes both
`"3"` and `"3 kaynaklar"` for the same heading, so the status returns both. The
coverage gate uses the manifest report and marks a row empty only when
`body_chars == 0`.

`config.json` advertises `data/inventory.json`, but no MCP path writes that file.
`guide_status` prints the configured path unconditionally, so it points at a
non-existent file. `prepare_guide` and `scaffold_guide` already hold the
inventory in memory; writing it is a deterministic, cheap fix that also makes
subsequent `build` runs cache-hit.

### 2.6 `visual_extract` format (V5-09)

`extract_native` chooses the default extension from `image.format`, but when
`out` is supplied it writes the native bytes to that path without checking the
extension. A JPEG extracted as `probe.png` is a valid JPEG file with a `.png`
name; `format:"jpg"` correctly describes the bytes, but the path lies. The hash
and quality report also describe the native bytes, not the file on disk.

### 2.7 `verify_citations` schema (V5-10)

`report.claims` contains every extracted claim; `rows` is the first
`max_claims` (default 40, probe used 5). The response mixes:

- `checked = rows.len()` — claims actually verified;
- `missing = report.unreferenced` — all claims without a citation in the guide.

The field names do not say which universe they describe. `missing` must become
scoped to `checked`; the whole-guide count needs its own name.

### 2.8 Visual status vocabulary (V5-11)

`VisualReport.passed` means "no blocking failure under the configured gate".
Under `warn`, `missing_figures` is not blocking, so `passed:true` is correct.
The problem is `status:"passed"`, which reads as "the gate passed". The report
needs a gate verdict separate from the boolean and from the configured policy.

### 2.9 Path and ID contracts (V5-12)

`source_id()` already accepts `PD0`, `PD0.pdf`, `sources/PD0.pdf` and
`data/blocks/PD0.json`. The PDF-reading tools bypass it and call
`resolve_read()` directly. The result is one session where `read_source` wants
`sources/x.pdf`, `read_blocks` wants `x`, and `visual_extract` wants a page image
id. The fix is a single `resolve_source()` helper with the same accepted forms,
plus one-line examples in every tool description. Write paths stay restricted to
`guides/`, `output/` and `data/`; this is a security invariant, not a bug.

### 2.10 Packaged resources (V5-13)

The resource list advertises normative documents, but `read_resource` reads them
from the workspace. Installed binaries run in a user workspace that usually has
no `docs/` directory. The documents must be compiled into the binary, with the
workspace copy used only as an override when present.

### 2.11 `visual_decision embed_crop` (V5-14)

The current validation is sequential:

1. no `bbox` → "requires bbox or candidate_id";
2. `bbox` present, no `crop` and no candidate → "requires --crop";
3. both present → success.

`crop` alone is sufficient to render; `bbox` is only needed for quality metadata
and `CropAsset.bbox`. A stored layout page has width/height, so a crop-only
decision can derive the bbox. Validation must collect all missing fields in one
pass and return one actionable error.

### 2.12 Cards parity (V5-15)

`qa_guide` calls `cards_from_markdown_opts(..., true)`; MCP `cards` and CLI
`cards` default to `false`. The counts differ by definition cards only. There is
no technical reason for two policies; the export is more useful with definitions
and `qa` already treats them as part of the guide. One default, one breakdown.

### 2.13 Style matrix (V5-16)

The style packs are embedded and `style list` already returns full
`StylePack` records, including `supported_structures` and `supported_domains`.
The information exists; it is not surfaced where the host chooses a style.
`prepare_guide` fails with the pack's supported list but does not suggest another
pack. `doctor` only validates already-created guides, so a greenfield user sees
the matrix for the first time in an error.

---

## 3. Contract decisions

These decisions are part of the fix; code must not invent a second policy.

### 3.1 Coverage row universe

| Layer | Canonical? | Contents |
|---|---|---|
| `heading` | yes, rank 0 | `LayoutBlockKind::Heading`, recovered inline headings, table-cell headings |
| `layout` | yes, rank 1 | numbered heading evidence found in layout block/table-cell text |
| `toc` | fallback, rank 2 | only when no heading/layout row exists for that number; never overrides |
| `text` | last-resort fallback, rank 3 | only when the source has no layout artifact; strict numbered-heading filter |
| `review` / `manual` | curated truth | review file rows; manual additions are allowed |
| `generated` | proposal | machine-built skeleton; never silently adopted |

Rules:

- A row number must look like a section number: components are positive
  integers; reject `N.0`, `N.00`, `N.000`, dates, currencies, percentages,
  quantities and bare numbers above 99 unless the title starts with
  `Bölüm`/`Chapter`.
- A title must not start with `)`, `]`, `-`, `*`, `%` or a lowercase word.
- Duplicate numbers collapse to the highest-priority evidence; page 0 is
  allowed only for review rows.
- Raw `text` evidence never overrides `layout` evidence.
- `canonical_total` is the number of unique canonical rows, not the sum of
  candidates.

### 3.2 Review and adoption

- `build` writes `coverage-manifest.generated.json`.
- If `coverage-manifest.review.json` is absent, `build` writes a review template
  seeded from generated rows (`status:"missing"`, no reason). This makes the
  `review` action usable.
- `review_eligible` evaluates the **review file**:
  - every `Missing` row blocks;
  - every `Excluded` row needs a non-empty `reason`;
  - stamps are advisory.
- `adopt:true` requires an eligible review and writes the combined file from the
  review rows, enriching `guide_heading`/`guide_sections` from generated rows
  where possible.
- Generated rows not present in review are reported as
  `only_in_generated` warnings, not as silent additions to the combined gate.
  If a generated-only row is `Covered` and the review lacks it, the warning names
  it; the host may add it manually.
- `review_diff` uses `row_key()` for both sides.
- `build` is idempotent: second run produces byte-identical generated JSON and
  does not overwrite an existing review or combined file.

### 3.3 Heading and outline universe

- The canonical heading list is the union of explicit heading blocks, recovered
  inline headings and table-cell headings, filtered by the same section-number
  and title rules as §3.1.
- `source_sections()` splits at every canonical heading. For a heading inside a
  paragraph, the preceding text stays in the previous section and the match
  starts the new section. For a heading in a table cell, the table block starts
  the new section and its rows are attached to it.
- All four brief tools continue to use `source_sections()`; no tool gets its own
  resolver.
- The scaffold outline groups by source and top-level number:
  - one `::: chapter` per source (or per top-level group when the source has
    more than one top-level number);
  - `##` for top-level and first sublevel headings;
  - `###` for deeper headings;
  - `<!-- covers: <source>:<number> -->` before each heading.
- No `::: front` in a generated skeleton. Use a normal `## Giriş` placeholder
  under the first chapter; the host may add a front block later with real prose.

### 3.4 Read budget

- `max_tokens` is an output budget and a read budget: `max_chars = min(
  max_chars.unwrap_or(DEFAULT_MAX_CHARS), max_tokens * 4)`.
- The parser floor becomes `MIN_READ_CHARS = 200`; a budget below that returns a
  named `budget too small` error instead of an empty page list.
- The first requested page is never dropped. If it does not fit, return a
  truncated page plus `next_offset`.
- `shrink_to_budget` never sets `returned` to zero for a non-empty data array.
  It keeps at least one element and trims its largest string field. If even the
  marker cannot fit, it returns an explicit `truncated` object with a `reason`.
- `returned_pages` must equal the number of page objects actually returned.

### 3.5 Visual vocabulary

- `visual_gate`: `off | warn | required` — the configured policy.
- `visual.status`: `passed | advisory | failed | skipped`:
  - `passed` only when the gate is `required` and no blocking issue exists;
  - `advisory` when the gate is `warn`/`off` and no blocking issue exists;
  - `failed` when a blocking issue exists;
  - `skipped` when no audit was needed.
- Add `visual.gate_result`: `pass | warn | fail` — the verdict.
- `pass` and `advisory` both keep the build green; only `failed` blocks.

### 3.6 Source references and output paths

Accepted source forms for every PDF-reading tool:

```text
sources/PD0.pdf
PD0.pdf
PD0
data/blocks/PD0.json   # block/brief tools only
```

One `resolve_source()` helper tries, in order: the input as a workspace path,
`sources/<input>`, and `sources/<input>.pdf`. A bare id with no extension is
accepted only when the file exists. Error messages list the accepted forms.

Output paths remain confined to `guides/`, `output/` and `data/`. The error for
a rejected output names those roots; do not widen the sandbox.

### 3.7 Cards policy

- One default: `include_definitions = true` for build `qa`, MCP `cards` and CLI
  `cards`.
- `cards` returns `total`, `recall`, `definitions`, `definitions_included`.
- When definitions are explicitly excluded, return a warning
  `"definitions excluded; qa_guide includes them"`; counts must match `qa_guide`
  under the default.
- Keep `--include-definitions` as a no-op alias for one release; add
  `--no-definitions` as the explicit opt-out.

---

## 4. Work packages

### WP0.1 — Canonical outline and strict-safe scaffold (P0)

**Files:** `crates/folisift-parse/src/outline.rs` (new),
`crates/folisift-parse/src/lib.rs`, `crates/folisift-mcp/src/lib.rs`,
`crates/folisift-cli/src/main.rs`, `crates/folisift-core/src/scaffold.rs`,
`crates/folisift-core/src/structures.rs`.

**Changes:**

1. Add `folisift_parse::outline`:
   - `CanonicalHeading { source, number, title, page, level, origin, block_id }`
   - `canonical_headings(inventory, layouts) -> Vec<CanonicalHeading>`
   - `Outline { chapters: Vec<Chapter>, items: Vec<OutlineItem> }`
   - `skeleton_markdown(title, language, structure, inventory, layouts) -> String`
2. Reuse the §3.1 number/title filters from `manifest.rs`; move them to a shared
   `heading.rs` module rather than duplicating regexes.
3. Replace the MCP `skeleton_markdown` and the `outline` loop with the shared
   builder. `outline.items` is the full canonical list; `outline.total` matches.
4. Replace the CLI scaffold body builder with the same function.
5. Remove `::: front` from generated skeletons. Keep `::: toc`; start the first
   chapter with `## Giriş` when a source has no headings.
6. Make `source_profile.units.sections` use the canonical heading count when
   `source.sections` is empty.
7. Keep `scaffold()` as the bundle/config writer; it may write a minimal valid
   `guide.md`, but `prepare_guide`, `scaffold_guide` and CLI `scaffold` must
   overwrite it with the shared skeleton after analysis.

**Tests:**

- unit: headings `1`, `1.1`, `1.1.1`, `2`, `10.2` produce a non-empty outline;
- unit: the generated markdown has no `::: front` and has one `##` per
  canonical heading;
- unit: a source with no headings gets one chapter and a `## Giriş`;
- integration: build a synthetic multi-section guide and run
  `audit --strict`; expect 0 chapter-split and 0 orphan-heading issues.

**Acceptance:** A-V5-01, A-V5-02.

### WP0.2 — Coverage canonical set, diff keys and adoption (P0)

**Files:** `crates/folisift-parse/src/manifest.rs`,
`crates/folisift-mcp/src/lib.rs`, `docs/authoring.md`.

**Changes:**

1. Introduce `HeadingEvidence` and the rank order in §3.1. Insert only one row
   per number; higher rank wins; lower ranks never override.
2. Apply the strict section-number/title filter to all origins. Reject decimals,
   quantities, leading punctuation and lowercase titles.
3. Extend layout scanning to table headers/rows (already partly present) and
   rank it above raw text.
4. `toc` and `text` are fallbacks: only insert when the number has no
   heading/layout row. For a source with no layout artifact, `text` remains
   available.
5. `build` writes a review template when the review file is absent.
6. `review_eligible` evaluates the review file only.
7. `review_diff` uses `row_key()` for both sides; add `only_in_generated`
   warnings to the build response.
8. `adopt` writes the review-backed combined manifest and returns
   `adopted:true`, or returns `adopted:false` with `blocked_by` reasons and the
   review path. Never silently return `adopted:false` without a reason.
9. Update `docs/authoring.md` §"coverage-manifest" to state the new row universe
   and adoption semantics.

**Tests:**

- fixture: generated manifest for a source with headings + TOC + body text has
  zero `toc`/`text` rows when heading/layout evidence exists;
- fixture: `1.0`, `10.500`, `300.000`, `) Title`, `40 x azaldı` are rejected;
- fixture: `4.4.4` paragraph and `10.2` table cell survive as canonical;
- fixture: review with 41 rows over a generated set with 52 extras yields
  `review_eligible:true`, `only_in_review:0`, and `adopt:true` writes combined;
- fixture: second `build` is byte-identical and does not overwrite the combined
  file.

**Acceptance:** A-V5-03, A-V5-04, A-V5-05.

### WP0.3 — Budget-preserving read path (P0)

**Files:** `crates/folisift-parse/src/source_read.rs`,
`crates/folisift-mcp/src/lib.rs`.

**Changes:**

1. Add `ReadSourceParams.max_tokens` behavior through the flattened
   `OutputOptions`; compute the effective read budget in `read_source`.
2. Lower the parser floor to 200 chars; return a named error below 50 tokens.
3. Change `shrink_to_budget` to preserve at least one array element and trim
   strings inside it.
4. Recompute `returned_pages` from the final page list; never report a count
   that disagrees with `pages`.
5. Add `budget` metadata to the result: `requested_tokens`,
   `effective_chars`, `budget_limited`.

**Tests:**

- unit: `read_from_source_at(max_chars=800, offset=0)` returns one page with
  non-empty text and `next_offset`;
- unit: `shrink_to_budget` on a two-item array returns one item plus marker;
- MCP regression: `read_source(max_tokens=200, pages="1-2")` returns at least one
  page, `returned_pages == pages.len()`, and `next_offset` is present;
- MCP regression: `max_tokens=10` returns a named error, not empty pages.

**Acceptance:** A-V5-06.

### WP1.1 — Shared heading resolver (P1)

**Files:** `crates/folisift-parse/src/brief.rs`,
`crates/folisift-parse/src/parser.rs` (or a shared `heading.rs`),
`crates/folisift-mcp/src/lib.rs`.

**Changes:**

1. Move the inline-heading regex and `recover_inline_headings` into a shared
   module used by both inventory and `source_sections`.
2. Scan table cells (`block.header`, `block.rows`) in addition to paragraph,
   list and unknown blocks.
3. In `source_sections`, split text at recovered headings:
   - paragraph/list: prefix stays in the previous section; suffix starts the new
     section;
   - table cell: the table block starts the new section and its rows are
     attached to it.
4. Deduplicate by `(number, page)` and prefer explicit `Heading` blocks.
5. Ensure `section_brief`, `heading_brief`, `chapter_brief` and `read_section`
   all call `source_sections` (no tool-local resolver).
6. Add candidate suggestions to the "section not found" error: nearest numbers
   from the canonical list.

**Tests:**

- fixture: paragraph `... 4.4.4. Asıl İşveren...` resolves `4.4.4`;
- fixture: table cell `10.2. UlusalBayram...` resolves `10.2`;
- all four tools return the same page range and text for both fixtures;
- an unknown heading error includes `candidates`.

**Acceptance:** A-V5-07.

### WP1.2 — `guide_status` parity and inventory (P1)

**Files:** `crates/folisift-mcp/src/lib.rs`,
`crates/folisift-core/src/config.rs` (if inventory default changes),
`crates/folisift-core/src/scaffold.rs`.

**Changes:**

1. `guide_status` calls the same coverage path as `check_coverage`
   (`load_or_analyze_inventory`, `select_coverage_manifest`,
   `manifest.report`/`coverage_report`) and returns `report.empty_sections`.
2. Return `inventory` only when the file exists; add `inventory_present: bool`
   and `inventory_hint` when absent.
3. Write `data/inventory.json` in `prepare_guide` and `scaffold_guide` after
   analysis. Use the same bytes as the in-memory inventory.
4. If the guide has no manifest, fall back to the legacy coverage report; do not
   mix the two `empty_sections` definitions.
5. `guide_status` must be read-only idempotent: no inventory write inside the
   status tool itself.

**Tests:**

- fixture: status `empty_sections` equals `check_coverage.empty_sections` on a
  guide with a `## 3 Kaynaklar` section shorter than 400 chars;
- fixture: `inventory_present:false` before prepare/scaffold, `true` after;
- fixture: a second status call does not rewrite files.

**Acceptance:** A-V5-08.

### WP1.3 — `visual_extract` format and hash (P1)

**Files:** `crates/folisift-parse/src/visual_assets.rs`,
`crates/folisift-mcp/src/lib.rs`.

**Changes:**

1. Determine the requested format from `out`:
   - `.png` → decode native bytes and encode PNG;
   - `.jpg`/`.jpeg` → keep native JPEG or re-encode at quality 92;
   - no extension → append the native extension;
   - unsupported extension → named error listing `.png`, `.jpg`, `.jpeg`.
2. Hash and quality-check the bytes that are actually written.
3. Return `native_format` and `format` separately; `asset` always has the
   matching extension.
4. `visual_assets check` verifies the hash against the written file.

**Tests:**

- JPEG native image written to `.png` has PNG magic, `format:"png"` and a hash
  equal to the file hash;
- JPEG native image written to `.jpg` has JPEG magic and `format:"jpg"`;
- unsupported `.webp` returns a named error;
- `visual_assets check` passes for both outputs.

**Acceptance:** A-V5-09.

### WP1.4 — Citation schema (P1)

**Files:** `crates/folisift-mcp/src/lib.rs`, `docs/authoring.md`.

**Changes:**

1. Response fields:
   - `checked` — claims verified in this call;
   - `missing` — checked claims with `missing_citation` or no verified
     reference; always `<= checked`;
   - `unreferenced_total` — all claims in the guide without a citation;
   - `total_claims`, `referenced_total`, `unreferenced_total` — whole-guide
     counts;
   - `budget` — `max_claims` and whether truncation occurred.
2. Keep `missing` as the scoped count; do not alias it to the whole-guide count.
3. Add `checked_claim_lines` so a host can map the count to rows.
4. Update docs with the exact definitions.

**Tests:**

- fixture: 5 checked, 1 missing citation → `checked:5`, `missing:1`,
  `unreferenced_total:75`;
- invariant test: `missing <= checked` and `checked <= total_claims`;
- summary line uses the scoped count.

**Acceptance:** A-V5-10.

### WP1.5 — Visual gate truth (P2, landed with P1)

**Files:** `crates/folisift-parse/src/visual_audit.rs`,
`crates/folisift-build/src/lib.rs`, `docs/authoring.md`.

**Changes:**

1. Add `gate_result: "pass" | "warn" | "fail"` to `VisualReport`.
2. Set `status` per §3.5. Under `warn` with no blocking issue, status is
   `advisory`, not `passed`.
3. Keep `passed` as the boolean blocking signal; `pass` and `advisory` both keep
   the build green.
4. Update `build_guide` and `qa_guide` summaries to print
   `status/gate_result`, e.g. `visual advisory (warn)`.
5. Update docs and the design-system gate table.

**Tests:**

- fixture: clean audit + `warn` → `passed:true`, `status:"advisory"`,
  `gate_result:"warn"`;
- fixture: clean audit + `required` → `status:"passed"`, `gate_result:"pass"`;
- fixture: missing low-text decision + `warn` → `status:"failed"`,
  `gate_result:"fail"`, build blocked;
- fixture: `off` and no audit → `status:"skipped"`.

**Acceptance:** A-V5-11.

### WP2.1 — Source reference normalization (P2)

**Files:** `crates/folisift-mcp/src/lib.rs`,
`crates/folisift-core/src/paths.rs`, `docs/authoring.md`.

**Changes:**

1. Add `resolve_source(&Workspace, &str) -> Result<PathBuf>` to the MCP layer
   (or core if it needs reuse) with the accepted forms in §3.6.
2. Use it in `read_source`, `search_source`, `analyze_source`,
   `source_insights`, `visual_candidates`, `visual_brief`, `visual_evidence`,
   `visual_extract`, `visual_decision`, `crop_source`, `render_pages`,
   `table_extract` and `citation_snippet`.
3. Keep `source_id()` for block/brief tools; add the same examples to their
   descriptions.
4. Improve `resolve_write` errors to name `guides/`, `output/`, `data/`.
5. Add one-line examples to every source parameter in `JsonSchema` docs.

**Tests:**

- table-driven: `PD0`, `PD0.pdf`, `sources/PD0.pdf` resolve to the same file for
  `read_source`, `search_source` and `visual_extract`;
- `read_blocks` accepts the same three forms plus `data/blocks/PD0.json`;
- `analyze_source(out="outside.json")` fails with an error naming the writable
  roots and does not create a file.

**Acceptance:** A-V5-12.

### WP2.2 — Package normative resources (P2)

**Files:** `crates/folisift-mcp/src/lib.rs`, `Cargo.toml` (if a build script is
needed), `docs/architecture.md`.

**Changes:**

1. Embed `docs/authoring-spec.md`, `docs/design-system.md` and the diagram rules
   with `include_str!` at compile time.
2. `read_resource` returns the embedded text; if the workspace has an override
   at the same path, prefer the workspace copy and add
   `source:"workspace"`; otherwise `source:"embedded"`.
3. Add a build-time test that the embedded documents contain their expected
   top-level headings and are not empty.
4. Ensure the installed binary no longer depends on a repository checkout.

**Tests:**

- unit: all three resources are non-empty and contain their first heading;
- integration: run the MCP server from a temporary directory with no `docs/`
  and read all three resources successfully.

**Acceptance:** A-V5-13.

### WP2.3 — One-pass `visual_decision` validation (P2)

**Files:** `crates/folisift-parse/src/visual_workflow.rs`,
`crates/folisift-mcp/src/lib.rs`.

**Changes:**

1. Add `validate_decision(input, candidate, layout_page) -> Vec<String>` that
   collects every missing/invalid field before any file write.
2. `embed_crop` accepts:
   - `bbox` + optional `crop`;
   - `crop` alone, deriving the bbox from the stored layout page dimensions;
   - `candidate_id` + optional `crop`;
   - `candidate_id` alone, using `crop_from_bbox`.
3. `embed_image` accepts `image_id` or a candidate's image id.
4. Required for all embeds: `caption`, `alt`, `reason`, valid `dpi`, valid
   `width`.
5. Error format: one line per missing field, sorted, plus a `next` hint.
6. Never write an asset before validation passes.

**Tests:**

- `embed_crop` with `crop` only succeeds and returns a valid `CropAsset`;
- an empty `embed_crop` returns one error containing `bbox`, `crop` and
  `candidate`;
- `embed_image` without `image_id` and without a candidate reports both options;
- a failing validation leaves `visual-audit.json` and `assets/` unchanged.

**Acceptance:** A-V5-14.

### WP3.1 — Cards parity (P3)

**Files:** `crates/folisift-mcp/src/lib.rs`, `crates/folisift-cli/src/main.rs`,
`crates/folisift-core/src/cards.rs`, `crates/folisift-build/src/lib.rs`,
`docs/authoring.md`.

**Changes:**

1. Add `CardPolicy { include_definitions: bool }` with `Default = true`.
2. Build `qa`, MCP `cards` and CLI `cards` all use the same policy.
3. CLI: keep `--include-definitions` as a deprecated no-op; add
   `--no-definitions`.
4. Response: `total`, `recall`, `definitions`, `definitions_included`,
   `warning`.
5. Docs: definitions are included by default; opt out explicitly.

**Tests:**

- fixture: default `cards.total == qa.cards.total`, `recall` and `definitions`
  match;
- fixture: `--no-definitions` reduces the total by the definition count and
  emits the warning.

**Acceptance:** A-V5-15.

### WP3.2 — Style matrix visibility (P3)

**Files:** `crates/folisift-render/src/styles.rs`,
`crates/folisift-cli/src/main.rs`, `crates/folisift-mcp/src/lib.rs`,
`styles/clean/style.json`, `styles/workbook/style.json`,
`docs/design-system.md`.

**Changes:**

1. Decide the support policy:
   - add `irac` to `styles/workbook/style.json` because the workbook layout is
     structure-agnostic and the scaffold already emits IRAC sections;
   - add `law` to `styles/clean/style.json` because the clean layout has no
     legal-specific typography.
2. Add `styles::matrix()` returning every pack with its supported structures,
   domains and a compatibility verdict for the requested pair.
3. `doctor` adds a `style-matrix` check: pack count, structure/domain coverage,
   and warnings for unsupported default pairs.
4. `style list` without a guide returns the matrix, not just pack names.
5. `prepare_guide`/`scaffold_guide` errors include
   `compatible_styles: [reader, accessible, ...]` and a `next` action.
6. If the maintainer prefers not to expand the matrix, keep the metadata change
   out and make `style list`/`doctor` return the gap explicitly. The acceptance
   allows either, but the error must be actionable.

**Tests:**

- matrix test: every declared structure/domain pair validates against its pack;
- `prepare_guide` with `workbook+irac` succeeds and builds (if policy expands);
- `prepare_guide` with `clean+law` succeeds and builds (if policy expands);
- `doctor` reports the matrix and exits 0 on the default workspace;
- an unsupported pair error lists at least one compatible style.

**Acceptance:** A-V5-16.

---

## 5. Sequencing and commits

One logical change per commit; do not mix test-only and contract changes when
they are not the same contract.

1. `refactor(parse): share heading evidence between inventory and briefs`
   - WP1.1 groundwork; no behavior change.
2. `fix(mcp): preserve page text under a small read budget` — WP0.3.
3. `fix(scaffold): build a multi-section outline from canonical headings` —
   WP0.1.
4. `fix(coverage): rank heading evidence and align review keys` — WP0.2.
5. `fix(parse): resolve inline and table-cell headings` — WP1.1.
6. `fix(mcp): align guide status with the coverage gate` — WP1.2.
7. `fix(visual): match extract format, extension and hash` — WP1.3.
8. `fix(mcp): separate checked and whole-guide citation counts` — WP1.4.
9. `fix(visual): separate gate policy from status` — WP1.5.
10. `refactor(mcp): normalize source references` — WP2.1.
11. `fix(mcp): embed normative resources` — WP2.2.
12. `fix(visual): validate decisions in one pass` — WP2.3.
13. `fix(cards): include definitions by default` — WP3.1.
14. `fix(styles): expose and complete the compatibility matrix` — WP3.2.

P0 commits (1–4) are the 0.23.3 hotfix. P1 commits (5–9) are 0.24.0. P2/P3
commits (10–14) are 0.25.0 unless the maintainer chooses to fold them into
0.24.0. The maintainer owns tags, installer builds and remote pushes.

---

## 6. Acceptance matrix

| ID | Test | Pass condition | Evidence |
|---|---|---|---|
| A-V5-01 | Greenfield `prepare_guide` | `outline.items` non-empty; 8+ sections on the probe; no `::: front` | MCP probe output |
| A-V5-02 | Greenfield strict audit | `audit --strict` on the generated PDF has 0 issues, including 0 chapter-split | audit JSON |
| A-V5-03 | Digital generated manifest | canonical rows are heading/layout only; `toc`/`text` rows are 0 or fallback-only; `1.0`/`10.500`/`300.000` absent | `coverage_manifest build` |
| A-V5-04 | Review diff | `only_in_review:0`; `only_in_generated` contains only real generated extras; keys use numbers | `review_diff` |
| A-V5-05 | Adoption and idempotence | `adopt:true` writes combined; second build identical; missing review gets a template | build twice; `diff` |
| A-V5-06 | Small read budget | `max_tokens:200` returns ≥1 page with text and `next_offset`; no zero-return marker | MCP probe |
| A-V5-07 | Inline headings | `4.4.4` and `10.2` resolve in `read_section`, `section_brief`, `heading_brief`, `chapter_brief` | 8 calls |
| A-V5-08 | Status parity | `guide_status.empty_sections == check_coverage.empty_sections`; inventory path exists or is null with a hint | 2 calls |
| A-V5-09 | Extract consistency | `file` magic == `format` == extension; hash matches; `visual_assets check` passes | `file`, JSON |
| A-V5-10 | Citation schema | `missing <= checked`; `unreferenced_total` is the 75-style whole-guide count | MCP probe |
| A-V5-11 | Visual vocabulary | `warn` + clean → `advisory`/`warn`; `required` + clean → `passed`/`pass`; missing decision → `failed`/`fail` | 3 builds |
| A-V5-12 | Source refs | `PD0`, `PD0.pdf`, `sources/PD0.pdf` work in all source tools; write escape error names roots | table test |
| A-V5-13 | Packaged resources | all three resources readable from a temp workspace with no `docs/` | MCP integration |
| A-V5-14 | Decision validation | crop-only `embed_crop` works; empty call reports all missing fields in one pass | 2 calls |
| A-V5-15 | Cards parity | default `cards` counts == `qa.cards` counts | 2 calls |
| A-V5-16 | Style matrix | `doctor` and `style list` expose the matrix; unsupported pair is actionable | CLI output |

**Whole-plan gate:** all A rows green, no new PARTIAL/FAIL in the 41-tool matrix,
and the external reproduction workspace rebuilt with the same page counts
(26/10/19) and strict audit 0.

---

## 7. Test strategy

1. **Unit tests in the owning crate** for every pure helper: heading filters,
   row keys, budget shrink, format selection, citation counts, status mapping,
   decision validation.
2. **Fixture tests** for manifest and brief behavior, using in-memory
   `Inventory`/`SourceLayout` values. No real PDF is required.
3. **MCP integration tests** for the 13 external reproductions. Use a temporary
   workspace or `/var/tmp/opencode/v5-repro`; never write into `sources/`.
4. **Build/audit integration** for scaffold only. A synthetic markdown fixture
   may be rendered with the paper template; the strict audit is the acceptance,
   not the PDF's prose.
5. **Regression guard** in `crates/folisift-mcp/src/lib.rs` tests: every tool
   description includes at least one source-form example; no `sources/*.pdf`
   path is hard-coded.
6. **CI commands:**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test
   cargo run -p folisift-cli -- doctor
   ```

OCR stays off in the portable build; the MCP server remains network-free and
process-free. The plan does not add a model call, a shell step or a network
dependency.

---

## 8. Risks and rollback

| Risk | Mitigation |
|---|---|
| Stricter heading filters drop a real section | Keep the filtered-out candidates in `data/coverage-candidates.json` as advisory; add a `--include-fallback` path for unusual sources. |
| Review adoption hides a real generated heading | Report `only_in_generated` as warnings and fail the build only when the host opts into `manifest_gate:"generated"`. |
| `shrink_to_budget` change makes some outputs larger | Budget is enforced by trimming strings; if the marker cannot fit, return an explicit `truncated` object. Add a max-iterations guard. |
| Expanding `workbook`/`clean` matrices changes existing guides | Style packs are embedded per guide; `doctor` validates existing guides. Add an ADR if the maintainer treats the domain list as a design contract. |
| Embedded docs drift from the repo | Add a test that compares the embedded copy with `docs/` at build time; the workspace override keeps local edits visible. |
| `cards` default change breaks a host expectation | Keep `--include-definitions` as a no-op alias; return `definitions_included` so a host can assert the policy. |

Rollback is per commit: each work package is independently revertible. The
coverage manifest and scaffold changes are the only ones that alter persisted
data; both write new files or replace generated files, never `sources/`.

---

## 9. Non-goals

- No 10/10 claim in this plan or in the release notes.
- No OCR default change; OCR remains opt-in and unavailable in the portable
  build.
- No LLM call, network access or shell execution inside the MCP server.
- No redesign of the paper/design system. `::: front` is removed from the
  generated skeleton, not from the template.
- No broad rewrite of the coverage heuristic. The plan narrows the canonical
  set and adds fallbacks; it does not attempt semantic section detection.
- No change to the `sources/` write ban or the `guides/`/`output/`/`data/`
  sandbox.

---

## 10. Reproduction appendix

The external workspace is a copy, not a source tree. Commands are run from
`/var/tmp/opencode/v5-repro` (or a fresh copy) and must never be committed.

```bash
# Coverage: 93 rows, 51 heading / 28 toc / 14 text
folisift coverage-manifest --root . --guide dijital-donusum-temelleri --action build

# Inline heading failures
folisift read-section --root . --guide is-hukuku --source is-hukuku --heading 4.4.4
folisift read-section --root . --guide is-hukuku --source is-hukuku --heading 10.2

# Layout evidence
# 4.4.4 -> guides/is-hukuku/data/blocks/is-hukuku.json, page 14, block is-hukuku:p14:b5 (paragraph)
# 10.2  -> same file, page 44, block is-hukuku:p44:b0 (table cell)

# Status mismatch
folisift guide-status --root . --guide is-hukuku
folisift check-coverage --root . --guide is-hukuku

# Manifest diff and adoption
folisift coverage-manifest --root . --guide dijital-donusum-temelleri --action build --adopt
```

MCP probes use the same workspace and the tool names from the v5 report. The
expected probe outputs are recorded in the PR description, not in this plan.

---

## 11. Open decisions for the maintainer

1. **Release split:** keep P0 only in 0.23.3, or fold P1 into 0.24.0 as planned?
   Default: follow §5.
2. **Style matrix:** expand `workbook+irac` and `clean+law`, or only surface the
   gap? Default: expand both, because the layouts are structure/domain-agnostic
   and the error is otherwise a dead end.
3. **Review template seeding:** seed generated rows as `missing` (safe, requires
   curation) or seed the generated status (faster, risks adopting uncurated
   rows). Default: seed `missing`.
