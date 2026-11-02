# Exam-readiness and production review — 2026-11-02

- **Status:** Analysis and remediation plan; P0 fixes started
- **Baseline commit:** `3651b43`
- **Scope:** whole repository, rendered PDFs, all style packs, CLI/MCP, D2
  pipeline, packaging, public documentation, learner-facing gaps
- **Trigger:** exam week; output must be trustworthy, dense, printable and
  reproducible now

## 1. Method

- Repository walk: crates, templates, styles, structures, guides, scripts,
  CI, docs and public README.
- Static gates: `cargo test --workspace`, clippy `-D warnings`, fmt.
- PDF gates: page count, strict layout audit, TOC links, bookmarks, tagged
  PDF, missing glyphs, font histogram, code/body ratio, line p50/p90,
  underfilled/empty/overflow pages.
- Preview audit: every style pack at 4–5 pages; minimum font and line metrics.
- D2 state: every guide’s diagrams, manifest freshness, pinned TALA seeds,
  theme/font digest, determinism baselines.
- Public review: Reddit threads on NotebookLM and student note-taking apps;
  Typst layout guidance via Context7.
- No user conversations, private sources or personal notes are included in
  this document.

## 2. Current evidence

### Build and test

| Check | Result |
|---|---|
| `cargo test --workspace` | 173 tests pass |
| `cargo clippy -D warnings` | pass |
| `cargo fmt --check` | pass |
| PD0–PD1 / PD2 / PD3 | 22 / 24 / 22 pages |
| Revision / organiser | 4 / 1 pages |
| Cold build (PD2) | 18.17 s |
| Warm cached build (PD2) | 0.89 s, `cache_hit: true` |

### PDF audit

All five tracked guide outputs pass strict audit:

| Output | Body | Code | Ratio | p90 | Min font | Glyphs | Tagged | TOC |
|---|---:|---:|---:|---:|---:|---:|---|---|
| PD0–PD1 | 9.5 | 8.4 | 0.884 | 58 | 6.6 | 0 | yes | 42 links |
| PD2 | 10.0 | 9.5 | 0.950 | 53 | 7.0 | 0 | yes | 54 links |
| PD3 | 10.0 | 9.2 | 0.920 | 51 | 7.0 | 0 | yes | 24 links |
| PD2 revision | 10.5 | 10.0 | 0.952 | 41 | 7.5 | 0 | yes | 9 bookmarks |
| PD2 organiser | 10.5 | — | — | 24 | 8.0 | 0 | yes | 5 bookmarks |

No empty, overflow, content-box, orphan-heading or image-overflow pages.
Paper remains the labelled legacy exemption for 6.6–7.9 pt components; new
styles meet the 8.0/8.5 pt floors.

### Style previews

All previews render and audit. Preview minimum fonts before the P0 fixes were
6.9–7.0 pt for paper-family presets; after adding TOC tokens and regenerating:

- journal / brief / accessible: no text below 8 pt.
- notes / workbook / reference: 8.0–8.2 pt minimum.
- revision: 7.5 pt minimum (compact labels), strict audit passes.

### D2

- 12 diagrams across PD2, PD3 and PD2 revision; all `ok`.
- TALA seeds `1,2,3`, theme/font digests present, SVG hashes verified.
- PATH-plugin isolation and deterministic output tests pass.
- Per-kind sequence/class/ERD postprocessed SVG baselines exist.

## 3. Findings and fixes

### Fixed in this pass

| ID | Finding | Fix |
|---|---|---|
| P0-1 | `styles preview journal/brief` selected a genre structure the generic preview did not satisfy | `a-g` is the preview structure for generic presets; genre packs keep their genre preview |
| P0-2 | `notes` used clean-template colours/fonts while its tokens declared warm paper/editorial | notes now uses the clean-family palette defaults and a technical/Inter pairing; warm palette applies correctly |
| P0-3 | `summary`, `workspace`, `answer-key` labels were hard-coded English | labels now come from style strings; TR/EN previews verified |
| P0-4 | `workbook` “Practice” ordering check matched only exact `## Practice` | numbering/heading-insensitive detection |
| P0-5 | build error said “revision grammar failed” for every genre | error names the actual structure |
| P0-6 | journal/brief/accessible TOC kept paper sizes | explicit TOC tokens added |
| P0-7 | stale tracked `output/old-python.pdf` | removed |
| P0-8 | README referenced a course-specific first guide and stale layout | replaced with a generic reference example and current crate list |

### Open findings

| ID | Severity | Finding | Plan |
|---|---|---|---|
| O-1 | medium | journal/brief previews still use a generic `a-g` sample, not an IMRAD/BLUF sample | P1 |
| O-2 | medium | style previews are not automatically audit-gated | P1 |
| O-3 | medium | no negative fixtures for notes/workbook/reference (only positive previews + unit rules) | P1 |
| O-4 | medium | candidate fonts are not selectable and no font licence/glyph matrix is published | P2 |
| O-5 | medium | no per-kind PDF baseline for D2 (SVG baseline only) | fixed: `make d2-pdf-check` |
| O-6 | medium | no automated citation-claim verification; page refs are traceability, not truth | fixed: `claims` review report |
| O-7 | low | revision preview has no PDF baseline; genre baselines cover only notes/workbook/reference | fixed: `genre-visual` covers revision |
| O-8 | low | `cargo audit` / dependency vulnerability gate is not in CI | P2 |
| O-9 | low | PDF reading order is not tested with a screen reader; tagged/bookmarks are proxies | P3 |
| O-10 | low | source scope is page-level, not section-level, for selective genres | P2 |
| O-11 | low | `styles preview` is the only style smoke test; no multi-page overflow fixture | P3 |
| O-12 | low | public docs still describe an unreleased v0.11 development build beside the v0.9 release | P1 |

## 4. Research: what learners miss

### NotebookLM / AI notebook tools

Public Reddit threads repeatedly surface these gaps:

1. **Offline and portable output.** Learners want a clean PDF/Word/Markdown
   export of explanations, mind maps and flashcards, especially before exams
   or with poor connectivity.
   - <https://www.reddit.com/r/hyderabadstartups/comments/1qus5sk/>
   - <https://www.reddit.com/r/notebooklm/comments/1q6alec/>
2. **Citations are not verification.** A cited passage does not prove that the
   claim’s subject, number, date, causal wording or qualifier is supported.
   Learners ask for real bibliography entries, page numbers and spot-check
   workflows.
   - <https://www.reddit.com/r/notebooklm/comments/1veqt1q/>
   - <https://www.reddit.com/r/notebooklm/comments/1lkhjdk/>
3. **Incomplete source coverage is silent.** Context limits and partial source
   loading can produce confident answers from incomplete inputs.
   - <https://www.reddit.com/r/notebooklm/comments/1l2aosy/>
   - <https://www.reddit.com/r/edtech/comments/1s74e9e/>
4. **Density and technical depth collapse.** Users report short, fluffy
   outputs when they need dense, exhaustive revision material.
   - <https://www.reddit.com/r/notebooklm/comments/1r9q5iv/>
5. **Output control and organization are weak.** Folder structure, bulk
   actions, selecting source groups, output format and rich note editing are
   recurring requests.
   - <https://www.reddit.com/r/notebooklm/comments/1rale3g/>
   - <https://www.reddit.com/r/notebooklm/comments/1mm7xv6/>
6. **Notebook-scoped memory and source bleed.** Big notebooks cite the same
   few sources; learners want per-source comparisons and explicit “which
   sources were not used?” checks.
   - <https://www.reddit.com/r/notebooklm/comments/1wmbqhd/>

### Student note-taking apps

Recurring public requests:

- reliable **PDF export**, offline/standalone use and no lock-in;
- **active recall**, spaced repetition, quizzes and exam-prep mode;
- **rich tables/formulas**, better typography and freedom over page shape;
- **folders, source organization and search**;
- stability with large PDFs and predictable builds.

Folisift already matches the strongest needs: deterministic offline PDF,
page-referenced source coverage, revision/recall/answer-key blocks, table and
formula layouts, and a reproducible engine. The gaps are verification,
spaced-review metadata, negative fixtures and source organization/reporting.

## 4.1 Output-intent layer — implemented 2026-11-02

The first implementation now separates reader task from layout:

- `exam-summary` → revision / revision-sheet / 2–4 pages
- `exam-revision` → revision / revision-sheet / 6–10 pages
- `concept-summary` → reader / concept-summary / 4–8 pages
- `study` → reader / a–g / 10–18 pages
- `deep` → reader / a–g / 16–28 pages
- `reference` → reference / formula-sheet / 2–8 pages
- `practice` → workbook / workbook / 6–12 pages
- `one-page` → revision / knowledge-organiser / 1 page

`styles recommend --intent` and MCP `style recommend` return the mapping.
`prepare_guide` accepts `intent` and `target_pages`, persists them in
`config.json`, and returns a genre checklist. The build service rejects a PDF
outside its declared page range. `concept-summary` uses `source_scope`, a
required explanatory example/table-or-diagram and explanatory prose; it is not
a bullet-only exam sheet.

Evidence and limits are recorded in
[`learning-science-and-genre-review-2026-11-02.md`](../learning-science-and-genre-review-2026-11-02.md).

## 4.2 Review artifacts — implemented 2026-11-02

- `folisift claims --guide <slug>` writes `data/claims.md`: high-signal
  numeric/date/comparison/causal sentences, their source page, and a
  missing-reference marker. It skips recall questions and never claims truth.
- `folisift cards --guide <slug>` pairs recall/answer blocks in document order
  and writes `data/cards.csv` with `front,back,tags,source`. `review_intervals`
  in `config.json` adds `review-<days>d` tags.
- MCP exposes `claims` and `cards`; the tool schema is now **5**.
- `make d2-pdf-check` renders sequence/class/ERD diagrams through the PDF and
  compares them with `tests/baselines/d2/pdf-pilot.pdf`.
- `make dep-audit` and the CI `audit` job fail on RustSec vulnerabilities.
- Candidate font review: keep the embedded trio; see
  [`font-candidate-review-2026-11-02.md`](../font-candidate-review-2026-11-02.md).

## 5. Plan

### P0 — Exam lockdown (today)

- [x] Fix preview structure selection for journal/brief.
- [x] Fix notes palette/font contract.
- [x] Localize summary/workspace/answer-key labels.
- [x] Fix workbook heading detection.
- [x] Add TOC tokens for journal/brief/accessible.
- [x] Remove stale `old-python.pdf`.
- [x] Rebuild all five guide outputs and all previews.
- [x] Freeze a checksum manifest of the five exam PDFs and baselines
      (`make exam-check`).
- [x] Run the full gate sequence once more from a clean cache:
  tests, clippy, fmt, type/palette/genre/parity/visual/revision,
  TOC, production report, package + clean install.
- [x] Add `make exam-check` for the five guided PDFs and their SHA-256 hashes.
- [ ] Copy only the five final PDFs to the offline exam folder; ignore
  `output/style-preview-*` for study use.

**Exit:** one immutable PDF set + checksums; no build or audit failure.

### P1 — Correctness and preview gates (before exams if time)

- [x] Add negative fixtures for notes/workbook/reference and assert the exact
  failure messages.
- [x] Add `make style-audit`: render every preview, run `audit --strict`, and
  fail on min font/overflow.
- [x] Make journal/brief previews genre-specific (IMRAD / executive brief).
- [x] Add a build-service integration test that exercises CLI/MCP parity on a
  temp workspace (`crates/folisift-cli/tests/build_parity.rs`).
- [x] Document `source_scope` in `authoring-spec.md` as normative.
- [x] Add release-status wording to README: released vs development build.

**Exit:** preview regressions and genre typos fail CI, not review.

### P2 — Research-informed study features (after exams)

- [x] Citation spot-check report: `claims` writes numeric/date/comparison
  claims with their source page or a missing-reference marker, plus a
  “citation is not verification” warning.
- [x] Guide-level spaced-review metadata: `review_intervals` in `config.json`
  and `review-<days>d` tags in the card export. Mutable `last_reviewed` and
  confidence stay in the review app; the PDF and CSV carry the schedule, not
  the app state.
- [x] Anki/CSV export for recall/answer blocks (`cards`, CLI and MCP; no
  network).
- [x] Per-kind PDF baselines for D2 sequence/class/ERD
  (`make d2-pdf-check`, `tests/baselines/d2/pdf-pilot.pdf`).
- [x] Revision preview PDF baseline (`tests/baselines/genres/revision.pdf`).
- [x] RustSec dependency gate (`make dep-audit`, CI `audit` job).
- [x] Candidate font review with licence, Turkish glyph, x-height and package
  weight evidence; decision: keep the embedded trio
  ([review](../font-candidate-review-2026-11-02.md)).
- [ ] Section-level source scope in addition to page ranges. Page-level scope
  is the current contract; add section scope only when a real guide needs it.

**Exit:** measured learning-workflow improvements, not feature count.

### P3 — Accessibility and long-term polish

- [ ] Screen-reader reading-order test on tagged PDFs.
- [ ] Component-level contrast checks beyond palette pairs.
- [ ] Multi-page overflow fixtures for every genre.
- [ ] Public docs pass: no course-specific examples in generic sections,
  no personal references, accurate release/development versions.

## 6. Acceptance gates

A guide is exam-ready only when all pass:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
make type-check
make palette-check
make genre-check
make d2-pdf-check
make dep-audit
make parity
make visual
make revision-check
python3 scripts/check-toc-links.py output/PD0-PD1_Calisma_Ozeti.pdf output/PD2_Calisma_Ozeti.pdf output/PD3_Calisma_Ozeti.pdf
python3 scripts/check-production-report.py
make package-check
```

Plus a manual review of the five exam PDFs at 100% print size: no clipped
tables, no orphan headings, diagrams readable, answer keys complete.

## 7. Non-goals

- No LLM, network or shell execution inside the MCP server.
- No runtime D2 execution during guide builds.
- No fabricated citations, page numbers or source claims.
- No fourth font family until licence/glyph/package evidence exists.
