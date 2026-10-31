# Codebase audit — 2026-10-31

- **Baseline:** `v0.10.0`, commit `8aef654` plus docs plan commits
- **Scope:** `crates/`, `templates/`, `styles/`, `structures/`, `guides/`
  configs, `scripts/`, `Makefile`, docs contracts
- **Method:** code inspection, targeted MCP/CLI runs, `cargo test`, `clippy`,
  `fmt`, PDF audits, and the D2 v0.9.0 research note
- **Status:** confirmed findings only; risks are labelled separately

## 1. Confirmed high/medium findings

### A1 — MCP build bypasses the revision grammar gate

- **Severity:** high
- **Where:** `crates/folisift-mcp/src/lib.rs:946-989`
- **Evidence:** CLI `build --check` runs
  `folisift_core::genre::revision_issues` before layout/coverage
  (`crates/folisift-cli/src/main.rs:735`), but MCP `build_guide` never calls it.
  A guide with long paragraphs/missing recall can pass through MCP.
- **Impact:** the same guide passes or fails depending on the host interface.
- **Fix:** extract one shared build service; both CLI and MCP call it.
- **Test:** MCP `build_guide` on a bad revision fixture must return an error
  containing the grammar issue.

### A2 — MCP build runs A–G coverage for revision/organiser guides

- **Severity:** high
- **Where:** `crates/folisift-mcp/src/lib.rs:957-977`
- **Evidence:** `build_guide(pd2-rev, check=true)` returns
  `coverage gate failed: 20/20 headings missing, 20 empty sections`. CLI skips
  A–G coverage for `revision-sheet`/`knowledge-organiser`
  (`crates/folisift-cli/src/main.rs:745`). MCP also loads the visual audit for
  these genres.
- **Impact:** the MCP cannot build the new genres with `check=true`.
- **Fix:** share the genre-aware check policy; do not run A–G coverage for
  revision/organiser. Add a genre-specific source-coverage model later.
- **Test:** MCP build of `pd2-rev`/`pd2-ko` must pass; A–G guide must still
  fail when coverage is missing.

### A3 — Knowledge-organiser one-page limit is not enforced

- **Severity:** high
- **Where:** `templates/revision.typ` organiser mode; CLI/MCP build checks
- **Evidence:** there is no page-count assertion for
  `structure == "knowledge-organiser"`. Only the render test checks a tiny
  fixture (`crates/folisift-render/tests/style.rs`).
- **Impact:** a two-page organiser passes `build --check`, violating the genre.
- **Fix:** parse the built PDF page count and require exactly 1 page for
  organisers; add a deliberately long fixture that fails.
- **Test:** long organiser fixture → error; normal fixture → pass.

### A4 — D2 contract is far narrower than the engine

- **Severity:** medium/product
- **Where:** `crates/folisift-render/src/diagram.rs:221-256`
- **Evidence:** only `flow`, `concept`, `tree`, `sequence`; only `tala`; nodes
  are id/label/group; no sequence actors/messages beyond generic edges, no
  class/ERD, no arrowheads, no shapes.
- **Impact:** the pipeline cannot express protocols, type systems or relational
  schemas; the user’s “use D2 capabilities” request is unmet.
- **Fix:** schema v3 from the D2 plan: sequence, class, ERD, edge
  arrowheads/line, small shape allowlist; keep TALA.
- **Test:** one fixture per kind plus visual baseline.

### A5 — D2 labels are hard-coded at 28/22 px

- **Severity:** medium
- **Where:** `crates/folisift-render/src/diagram.rs:446-480`
- **Evidence:** `font-size: 28` / `22` are emitted regardless of body,
  palette or density.
- **Impact:** diagram labels can look oversized/undersized relative to the
  page; revision/reader/accessible diagrams are inconsistent.
- **Fix:** diagram label sizes from `TypographyScale`; min effective 8.5–9 pt.
- **Test:** font histogram/ratio for diagram text in the final PDF.

### A6 — D2 executes plugins found on `PATH`

- **Severity:** high (security)
- **Where:** `crates/folisift-cli/src/diagram_cmd.rs:196`;
  `crates/folisift-cli/src/diagram_cmd.rs:379`
- **Evidence:** `Command::new(d2)` inherits the environment; D2 v0.9.0 scans
  `PATH` for `d2plugin-*` and runs their `info` command. `find_d2` also falls
  back to `PATH`.
- **Impact:** an untrusted `d2`/plugin can execute during `diagram build`.
- **Fix:** spawn with `env_clear()`, private/empty `PATH`, temp `HOME`, no
  inherited `D2_*`; prefer explicit/sidecar binary; remove implicit PATH
  fallback or require opt-in.
- **Test:** fake `d2plugin-evil` on PATH must not execute; sidecar binary is
  selected.

### A7 — Generated D2 source has no allowlist

- **Severity:** high (security/determinism)
- **Where:** `crates/folisift-render/src/diagram.rs:419-480`
- **Evidence:** the generator is trusted today, but there is no test that
  emitted source contains only the generator-owned directives. Future fields
  could accidentally emit `import`, `icon`, `link`, `vars`, layers/scenarios,
  or `animate`.
- **Impact:** a code change could open network/file reads or non-deterministic
  output without a gate.
- **Fix:** generated-source allowlist test; reject those directives in
  validation.
- **Test:** snapshot source for every kind; assert forbidden tokens absent.

### A8 — Revision/organiser builds skip canonical coverage entirely

- **Severity:** medium
- **Where:** `crates/folisift-cli/src/main.rs:745`
- **Evidence:** `if !matches!(structure, revision-sheet | knowledge-organiser)`
  skips inventory/coverage. There is no alternative source-coverage report for
  these genres.
- **Impact:** a revision sheet can omit half the source and still pass
  `--check`; only the revision grammar checks structure, not coverage.
- **Fix:** add a genre-specific coverage model: canonical source sections vs
  revision sections, with claimed ranges; not A–G headings.
- **Test:** revision fixture missing a canonical source section must fail.

### A9 — `audit` margins/min-font are hard-coded for paper

- **Severity:** high
- **Where:** `crates/folisift-parse/src/audit.rs:201,355`
- **Evidence:** content box uses `margin_x = 46.0`, top 52, bottom 45; min font
  fails only below 6 pt. `revision` uses 32 mm margins and `accessible` 20 mm;
  the audit can miss text inside their larger margins and accepts 6.9 pt.
- **Impact:** layout gates are not style-aware; a bad accessible/revision page
  can pass.
- **Fix:** `audit --guide` reads style margins/floors; add `--margin-*`,
  `--min-font`; keep 6 pt only as an absolute hard floor.
- **Test:** seeded fixtures for accessible margin intrusion and 6.9 pt label.

### A10 — Underfilled-page gate treats visual pages as prose

- **Severity:** medium
- **Where:** `crates/folisift-parse/src/audit.rs:398`
- **Evidence:** `detect_underfilled` uses character count only; pages with a
  large diagram/table can be flagged. Revision/organiser pages are especially
  visual.
- **Impact:** false failures or pressure to add filler prose.
- **Fix:** skip/weight pages with figure captions, images or table markers;
  report visual density separately.
- **Test:** visual page with low text but a figure must not fail.

### A11 — `figure_captions` counts “Şekil” anywhere

- **Severity:** low
- **Where:** `crates/folisift-parse/src/audit.rs:388`
- **Evidence:** `page_text.matches("Şekil")` counts prose mentions, not only
  captions.
- **Impact:** inflated figure counts; visual audit reporting becomes noisy.
- **Fix:** count caption-like lines at line start after quotes/numbering.
- **Test:** prose mention vs caption.

### A12 — `styles.rs` still uses global string replacement and a modifier hack

- **Severity:** high
- **Where:** `crates/folisift-render/src/styles.rs:339,380`
- **Evidence:** `body_size` replaces every `9.5pt`; `accessible` uses
  `component_scale` to avoid a real shared scale. Revision uses a third path.
- **Impact:** code/table/caption sizes drift; one role can change accidentally.
- **Fix:** `TypographyScale` for all templates; remove `component_scale` and
  global `9.5pt` replacement.
- **Test:** placeholder-only templates; no `9.5pt`/`component_scale` in built
  styles.

### A13 — `paper` still carries 6.9 pt labels/7.4 pt code

- **Severity:** medium (accessibility/legacy)
- **Where:** `templates/paper.typ`; `styles/paper/tokens.json`
- **Evidence:** paper parity is frozen at 9.5/6.9/7.4; `reader` and `revision`
  diverge. The `paper` PDF still passes the 6 pt absolute gate.
- **Impact:** legacy guides remain hard to read; docs may imply a standard that
  paper does not meet.
- **Fix:** keep `paper` as a labelled legacy fixture, or retire exact parity
  and migrate it; do not call it the standard.
- **Test:** docs/CLI label `paper` as legacy; new guides default to `reader`/
  `revision`.

### A14 — `scaffold` says schema-2 but writes schema-3

- **Severity:** low (docs)
- **Where:** `crates/folisift-core/src/scaffold.rs:14`
- **Evidence:** comment says “schema-2 guide bundle”, code writes
  `schema_version: 3`.
- **Impact:** maintainer confusion.
- **Fix:** update comment and add a schema-version test.

### A15 — `paper`/`reader`/`accessible` sizes are not a single scale

- **Severity:** high
- **Where:** templates and token files
- **Evidence:** paper 9.5/12/8.2/7.9/6.9; revision 11/15.4/12.1/9.5/8.4;
  accessible 12/15.6/10.7/10.3/9.0. Ratios differ by layout.
- **Impact:** the same role means different visual hierarchy; docs cannot
  state one rule.
- **Fix:** typography plan: shared role ratios and floors; layout chooses only
  body base.

### A16 — `styles preview` forces structure `a-g`

- **Severity:** low
- **Where:** `crates/folisift-cli/src/main.rs:636`
- **Evidence:** preview sets `structure: "a-g"` so revision/organiser previews
  are not genre previews. `revision` supports `a-g` only to make this work.
- **Impact:** previews do not exercise the genre grammar; a broken organiser
  preview may pass.
- **Fix:** preview uses the pack’s first supported genre structure and a
  matching fixture; keep `a-g` only for paper/clean.
- **Test:** preview each genre with its structure.

### A17 — `paper`/`reader` 22-page parity can hide reflow problems

- **Severity:** medium (process)
- **Where:** `AGENTS.md`, `Makefile`, docs
- **Evidence:** parity is a fixed 22-page gate; new code/label sizes would
  break it, so paper is frozen while new styles drift.
- **Impact:** the “standard” is the legacy file, not the design system.
- **Fix:** mark paper legacy; add structural parity (sections/components/TOC/
  audit) for new styles; retire exact page parity with an ADR.

## 2. Risks / hypotheses (not confirmed bugs)

| Id | Risk | Why it matters | Next probe |
|---|---|---|---|
| R1 | `audit` body-font mode can be wrong on a one-size document | single-size fixture/false orphan | add `--body-font`; fixture |
| R2 | `postprocess_svg` role/aria handling may duplicate or miss attributes | accessibility regression | SVG attribute test across D2 output |
| R3 | D2 theme/font postprocess can drift from palette tokens | brand/contrast inconsistency | visual baseline per palette |
| R4 | `source_read`/cache key may not include all options | stale content | cache-key test with max_chars/pages |
| R5 | `Guide::load` accepts `theme` and `style.aesthetic` that disagree | host confusion | config validation test |
| R6 | `revision`/organiser coverage is undefined | silent omissions | genre coverage fixture |
| R7 | `scaffold` sets inventory/coverage for revision though checks skip them | dead config | config test |
| R8 | `D2` `--tala-seeds` defaults are not recorded | layout nondeterminism across D2 versions | pin seeds in manifest |

## 3. What is healthy

- Workspace path sandboxing is lexical + canonical; `sources/` is read-only.
- D2 execution is CLI-only; MCP never runs D2, shell or network.
- SVG sanitizer rejects scripts, foreignObject, external refs and event
  handlers.
- `audit` catches empty/overflow/orphan/page-count issues; TOC links are
  machine-checked.
- 135 tests, clippy `-D warnings`, fmt, paper parity, visual baselines,
  revision/organiser audits and package checksum pass.
- The revision grammar gate is real in CLI; the gap is MCP parity.

## 4. Priority fix list

1. A1/A2/A8: shared genre-aware build service; CLI/MCP parity.
2. A3: organiser page-count gate.
3. A6/A7: D2 process isolation and source allowlist.
4. A9/A10/A11: style-aware audit gates.
5. A12/A15: shared TypographyScale; remove global replacement/component hack.
6. A4/A5: D2 schema v3 and diagram typography.
7. A13/A17: paper legacy policy/ADR.
8. A14/A16: docs/preview cleanup.

## 5. Tests/gates to add

- MCP `build_guide` parity tests for revision, organiser and A–G.
- Organiser 1-page/2-page fixtures.
- D2 fake-plugin execution test; source allowlist snapshot.
- Audit fixtures: accessible margin intrusion, 6.9 pt label, visual page,
  prose “Şekil” mention.
- Typography histogram/code-body ratio test.
- TALA seed/manifest determinism test.

## 6. Verification commands used

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
make revision
make parity
make visual
python3 scripts/check-toc-links.py output/*.pdf
python3 scripts/check-production-report.py
```

The full D2 capability detail is in
[`docs/d2-capability-research.md`](d2-capability-research.md); the typography
plan is in
[`docs/plans/2026-10-31-typography-standardization-and-font-system.md`](plans/2026-10-31-typography-standardization-and-font-system.md).

## 7. Additional confirmed findings (whole-repo audit)

### B1 — `prepare_guide` ignores requested style/source changes on an existing guide

- **Severity:** high
- **Where:** `crates/folisift-mcp/src/lib.rs:687-713`
- **Evidence:** the existing guide config is reused, but the response returns the
  newly requested `style`; changing `style` or `sources` does not update
  `config.json`. A probe on `pd2-ko` with `style:"paper"` returned
  `aesthetic:"paper"` while the stored config stayed `revision`.
- **Impact:** host and build use different style/source sets.
- **Fix:** reject style/source changes on existing guides or update the config
  atomically; return the actual stored style.

### B2 — style/domain IDs are not sandboxed

- **Severity:** high (security)
- **Where:** `crates/folisift-render/src/styles.rs:257-268`;
  `crates/folisift-mcp/src/lib.rs:421-433`
- **Evidence:** `styles info id="../../../../../var/tmp/outside"` can read
  outside the workspace; the same applies to domain loads.
- **Impact:** contradicts the workspace sandbox contract; local file disclosure.
- **Fix:** validate IDs against built-ins or canonicalize and require
  containment under `styles/`/`domains/`; reject separators and `..`.

### B3 — stale inventory is trusted; no source digest

- **Severity:** high
- **Where:** `crates/folisift-parse/src/inventory.rs:9-15`;
  `crates/folisift-cli/src/main.rs:769-774`;
  `crates/folisift-mcp/src/lib.rs:1066-1076`
- **Evidence:** `Inventory` stores no source hash; an existing
  `data/inventory.json` is reused after the source PDF changes.
- **Impact:** coverage/visual gates can describe the old PDF.
- **Fix:** store BLAKE3 digests and re-analyze on mismatch; test source
  replacement.

### B4 — inventory/coverage paths can overwrite arbitrary writable files

- **Severity:** high
- **Where:** `crates/folisift-core/src/config.rs:299-306`;
  `crates/folisift-cli/src/main.rs:784-785`
- **Evidence:** a config `"coverage":"guide.md"` makes `build --check` write the
  coverage report over the guide. No extension/bundle containment check.
- **Impact:** data loss.
- **Fix:** require inventory `.json` and coverage `.md` inside the guide
  bundle; reject path escapes.

### B5 — visual-audit failure is not conjoined with coverage passed

- **Severity:** high
- **Where:** `crates/folisift-mcp/src/lib.rs:1028-1063`
- **Evidence:** response reports `coverage.passed` separately from `visual`;
  there is no top-level pass that requires both.
- **Impact:** an MCP consumer can treat a build as passing while visual
  decisions are unresolved.
- **Fix:** top-level `passed = coverage && visual && layout`, plus a test.

### B6 — `doctor` fails on a documented empty workspace

- **Severity:** medium
- **Where:** `crates/folisift-cli/src/main.rs:375-381`
- **Evidence:** `doctor` requires `!guides.is_empty()`, but install docs tell a
  new user to run `doctor` before creating a guide.
- **Impact:** first-run failure.
- **Fix:** zero guides is a warning, not an error; test empty workspace.

### B7 — packaging can succeed without PDFium

- **Severity:** medium/high
- **Where:** `scripts/package.sh:45-50`; `README.md:23-25`;
  `docs/install.md:17-25`
- **Evidence:** missing `libpdfium.so` only warns, while docs promise the
  archive contains it.
- **Impact:** broken archive that passes its own build.
- **Fix:** fail unless an explicit dev override; test archive contents and
  `doctor` from a clean directory.

### B8 — third-party notices omit PDFium and font licences

- **Severity:** medium (legal)
- **Where:** `THIRD_PARTY_NOTICES.md`; `scripts/package.sh`
- **Evidence:** the archive ships `libpdfium.so` and OFL fonts; notices cover
  D2 only.
- **Impact:** licence non-compliance.
- **Fix:** add PDFium BSD and OFL/font notices; include licence files.

### B9 — schema 1 is accepted but unbuildable

- **Severity:** medium
- **Where:** `crates/folisift-core/src/config.rs:11,436-440`;
  `crates/folisift-cli/src/main.rs:702-707`
- **Evidence:** schema 1 is marked supported but uses HTML; build rejects
  non-Markdown.
- **Impact:** a config can load as valid and fail at build.
- **Fix:** drop schema 1 or implement migration; add a build test.

### B10 — coverage/structure gates accept nested or duplicate headings

- **Severity:** medium
- **Where:** `crates/folisift-parse/src/coverage.rs:392-439`;
  `crates/folisift-core/src/structures.rs:150-175`
- **Evidence:** heading scans include `##`/`###` inside directives and allow
  duplicates; a `HashMap` silently overwrites duplicate numbers.
- **Impact:** gates can be satisfied without real top-level sections.
- **Fix:** parse only top-level `##` outside directives; fail duplicates.

### B11 — malformed structure packs silently disable validation

- **Severity:** medium
- **Where:** `crates/folisift-core/src/structures.rs:72-79`
- **Evidence:** `filter_map(...ok())` drops invalid packs; `validate()` returns
  `Ok` when the id is absent.
- **Impact:** a JSON typo removes the gate.
- **Fix:** load packs with `Result` at startup; test all packs load.

### B12 — PDF is written before gates run

- **Severity:** medium
- **Where:** `crates/folisift-cli/src/main.rs:740`;
  `crates/folisift-mcp/src/lib.rs:942`
- **Evidence:** output is written, then audit/coverage can fail.
- **Impact:** a failed build leaves a fresh-looking PDF.
- **Fix:** render to temp, gate, atomic rename.

### B13 — English revision/organiser labels are Turkish

- **Severity:** medium
- **Where:** `crates/folisift-render/src/markdown.rs:233-270`;
  `templates/revision.typ:295,308`
- **Evidence:** EN probe extracts `HATIRLA`, `CEVAP`,
  `FOLISIFT · SINAV TEKRARI`; these are hardcoded defaults.
- **Impact:** broken localization; Turkish text in English guides.
- **Fix:** move labels to pack strings; test EN output has no Turkish defaults.

### B14 — lint rejects raw-Typst-looking text in ordinary code fences

- **Severity:** medium
- **Where:** `crates/folisift-render/src/markdown.rs:478-511`
- **Evidence:** C `#include <stdio.h>` inside a non-Typst fence is rejected as
  raw Typst/HTML/CSS.
- **Impact:** code examples cannot be written.
- **Fix:** skip all fenced content; test C/shell blocks.

### B15 — HTML and math are silently dropped

- **Severity:** medium
- **Where:** `crates/folisift-render/src/markdown.rs:837-842`;
  `docs/authoring-spec.md:267`
- **Evidence:** `Html`, `InlineMath`, `DisplayMath` events are ignored; docs
  recommend `<pre><code>`.
- **Impact:** silent content loss.
- **Fix:** support a subset or reject with a clear error; update docs.

### B16 — typography heading/mono tokens are ignored

- **Severity:** medium
- **Where:** `crates/folisift-render/src/styles.rs:328-334`
- **Evidence:** only `body` font is replaced and only `"Noto Serif"`; `heading`
  and `mono` tokens have no effect; `faint`/`cream`/`sand` palette roles are
  not replaced for paper/clean.
- **Impact:** style/font customization silently does nothing.
- **Fix:** role placeholders for body/heading/mono and all palette roles.

### B17 — accessibility validation is too narrow

- **Severity:** medium
- **Where:** `crates/folisift-render/src/styles.rs:176-217`
- **Evidence:** only `ink/paper` contrast and body ≥9 pt are checked;
  `muted`/`faint` contrast and component floors are not.
- **Impact:** low-contrast captions/table text pass.
- **Fix:** validate all text roles and component floors; test a low-contrast
  `muted` override.

### B18 — `read_source` can loop on a page larger than `max_chars`

- **Severity:** medium
- **Where:** `crates/folisift-parse/src/source_read.rs:98-108`
- **Evidence:** if one page exceeds `max_chars`, `next_pages` is the current
  page; re-reading returns the same prefix.
- **Impact:** infinite/blocked reading loop.
- **Fix:** return a character offset or page remainder; test a 70k-char page.

### B19 — partial source cache is accepted

- **Severity:** medium
- **Where:** `crates/folisift-parse/src/source_cache.rs:51-64`
- **Evidence:** any non-empty parsed cache is accepted; the `# pages:` header
  is not verified; `sha256` is actually BLAKE3.
- **Impact:** truncated cache silently drops pages.
- **Fix:** verify page count/sequence; rename/implement the hash correctly.

### B20 — diagram build requires manifests for unreferenced specs

- **Severity:** medium
- **Where:** `crates/folisift-render/src/diagram.rs:707-716`
- **Evidence:** `load_assets` loads every `diagrams/*.json`; an extra
  saved-but-unbuilt spec blocks the guide.
- **Impact:** a harmless draft blocks builds.
- **Fix:** load only referenced IDs or warn/ignore unreferenced specs.

### B21 — TOC/production smoke gates are weak

- **Severity:** medium
- **Where:** `scripts/check-toc-links.py:16-45`;
  `scripts/check-production-report.py:72-87`
- **Evidence:** TOC check only looks at page 3 and can pass with zero links;
  production smoke accepts null `style_rationale` and reuses `pd2` only.
- **Impact:** false green gates.
- **Fix:** discover TOC pages, assert expected link count, require rationale,
  and cover `pd2-rev`/`pd2-ko`.

### B22 — `revision_issues` table row limit is off by one

- **Severity:** medium
- **Where:** `crates/folisift-core/src/genre.rs:83-93`
- **Evidence:** header and separator are counted as data rows; 8 data rows are
  reported as 10 and rejected.
- **Impact:** valid tables fail the grammar.
- **Fix:** count data rows only; test an 8-row table.

### B23 — canonical TOC major-only numbering is wrong

- **Severity:** medium
- **Where:** `crates/folisift-parse/src/parser.rs:201-206`
- **Evidence:** major-only TOC entries get `{first_heading_prefix}.{major}`,
  turning “2 Methods” into “1.2”.
- **Impact:** wrong canonical headings/coverage.
- **Fix:** preserve major numbering when the TOC uses major sections; test.

### B24 — `style_rationale` is not enforced

- **Severity:** low/medium
- **Where:** `crates/folisift-core/src/config.rs:152-157`;
  `crates/folisift-core/src/scaffold.rs:82`
- **Evidence:** field is optional, scaffold writes null, build accepts null,
  while `prepare_guide` advertises `style_rationale_required:true`.
- **Impact:** host contract is not enforceable.
- **Fix:** require non-empty rationale for schema 3 or drop the claim.

### B25 — diagnostics line numbers are off by one

- **Severity:** low
- **Where:** `crates/folisift-render/src/lib.rs:328-343`
- **Evidence:** line count based on `lines().count()`; first-line errors report
  line 0.
- **Impact:** confusing diagnostics.
- **Fix:** use 1-based line numbers; test an error on line 1/2.

### B26 — image parsing is naive

- **Severity:** low
- **Where:** `crates/folisift-render/src/assets.rs:198-215`
- **Evidence:** scans all lines including code fences; cannot handle Markdown
  image titles/angle destinations.
- **Impact:** false asset paths or missed images.
- **Fix:** fence-aware parsing or pulldown events.

## 7.1 Remediation status — 2026-10-31

Closed with code and tests: A1–A3, A6, A7, A12 (heading/mono/palette roles),
A14, B1–B6, B7, B8, B9, B12–B20, B22, B24, B25, B26.

Partially closed: A8 (genre coverage still undefined), A9 (margins and an
optional minimum font are style-aware via `audit --guide`; the 6 pt absolute
floor remains), A10/A11 (underfill is still prose-shaped and caption counting
is approximate), A13/A17 (paper is labelled legacy; exact parity remains by
choice), B10/B11 (structure heading gate is stricter; duplicate canonical
numbers still need a report field), B21 (TOC/production checks strengthened),
B23 (major-only TOC fixed when no chapter marker is present).

Open: A4/A5 residual diagram typography/per-kind baselines, A16 preview uses
a generic structure, B18/B19 follow-up limits, and the full style-aware audit.

## 8. Expanded priority backlog

| Priority | Findings | Next action |
|---|---|---|
| P0 security/data | B2, B4, A6, A7, A13 | sandbox IDs, atomic writes, D2 env/allowlist |
| P0 correctness | A1, A2, A3, B1, B5, B12 | shared build service; temp output; top-level pass |
| P1 gates | A8, A9, A10, A11, B10, B11, B22, B23 | genre coverage, style-aware audit, parser fixes |
| P1 typography | A12, A15, B16, B17 | shared scale, palette/font roles, contrast floors |
| P1 content | B13, B14, B15, B18, B19 | localization, fences, math/HTML, source cache |
| P2 D2 | A4, A5, B20 | schema v3, diagram typography, referenced assets |
| P2 docs/process | A14, A16, A17, B6, B7, B8, B9, B21, B24, B25, B26 | audit and release gates |
