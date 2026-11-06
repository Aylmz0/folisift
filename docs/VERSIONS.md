# Versions

Build numbers are derived with `git rev-list --count HEAD` at compile time.
SemVer is assigned by an annotated tag; the tag is the authority, not this
table.

| Tag | Date | Build | Commit | PDFium | What changed |
|---|---|---:|---|---|---|
| `v0.23.2` | 2026-11-06 | 248 | tag `v0.23.2` | `chromium/8058` | Audit patch: stale `visual-candidates.json` scorer/source-digest detection, full v4AGENT harness in `make acceptance`, and MCP instruction regression test. No schema change; 41 tools at schema 19. |
| `v0.23.1` | 2026-11-06 | 245 | tag `v0.23.1` | `chromium/8058` | Installer patch: `FOLISIFT_VERSION` accepts `0.23.1` or `v0.23.1`, resolves the versioned release asset name, and installs the `v0.23.0` feature set unchanged. No engine or schema change. |
| `v0.23.0` | 2026-11-06 | 241 | tag `v0.23.0` | `chromium/8058` | V4 feedback close-out and MCP guidance: canonical generated manifest with text/layout fallback and range covers, recursive output budgets, shared source/diagram path resolution, guide-plan source pages, candidate-backed visual gate with advisory status (`visual_min_score`), explicit `excluded` reasons, optional `ocr-tesseract` + CLI OCR server path, image-ref detail, decorative/card semantics, glossary page-break fixture, and the five-layer MCP guidance contract (`start-here`, concise summaries, output schemas). Schema 19 with 41 tools. |
| `v0.22.0` | 2026-11-05 | 230 | `874dd06` | `chromium/8058` | Source visual pipeline: bbox-level `visual_candidates` with salience/relevance/reconstruction, native image extraction, adaptive 200–500 DPI crops, visual quality/audit provenance, `visual_decision`/`visual_assets`, D2 concept/schematic vs unique charts, vector-cluster fallback, soft visual budget; schema 17 with 41 tools; A-V5-01..13 green |
| `v0.20.0` | 2026-11-04 | 195 | `8507525` | `chromium/8058` | Visual evidence and chart policy: `visual_evidence`, figure/table visual gate (`off`/`warn`/`required`), `embed_crop`/`describe` decisions, provenance-aware `crop_source` and `::: figure`, actionable table fallback, global Model2Vec discovery, digital review action and V3 acceptance harness; schema 15 with 37 tools |
| `v0.19.0` | 2026-11-04 | 190 | `2e8975206cf9961ddca76cf2d3a15a72f6b9e4ff` | `chromium/8058` | Feedback v2 P2: real PDF bbox width regression, OCR evidence matrix, bounded graph path/explain, QA environment/determinism report, cookbook and manifest review docs |
| `v0.18.0` | 2026-11-04 | 188 | `a1fd480fea2f22a5ae67c4c89f8c70a5447a087f` | `chromium/8058` | Feedback v2 P0/P1: review-manifest coverage truth, brief budgets/exact matches, semantic topics, claims/graph budgets, shared guide metrics, citation sentence scoring, D2 sidecar discovery, visual taxonomy, style over_budget, QA cards, IRAC scaffold, source-brief review merge and QA cost report; schema 13 with 35 tools |
| `v0.17.1` | 2026-11-04 | 185 | `8726bd792a3c6fa0ad60f335cc30e8d46f8af1c6` | `chromium/8058` | Fix `citation_snippet` byte-offset panic on Turkish uppercase case folding (`İ`/`I`); the MCP request now returns a bounded snippet instead of hanging. Schema stays 13 with 35 tools |
| `v0.17.0` | 2026-11-04 | 183 | `60b70fbcf86bcf028f933d33c60717d96b473c1c` | `chromium/8058` | Review close-out: D2 classes/grid/vars/imports/dimensions, allow_page_span/aspect_ratio/label_lines, unified DiagramResult, audit intentional pages and table-free line stats, actual section pages, brief cache, chapter/heading briefs, coverage hints, output budget contract, doctor style checks; schema 13 with 35 tools |
| `v0.16.0` | 2026-11-04 | 179 | `5aff3ba036aa8c0ce969e72994cebc4e20b96709` | `chromium/8058` | TOC include/exclude, structured build failures, doctor MCP, section read modes, definition cards, source-page resources, D2 node metadata; schema 12 with 33 tools |
| `v0.15.1` | 2026-11-04 | 175 | `9137268d889654cbaf127d31fee57f89d23c0a17` | `chromium/8058` | Model2Vec-backed `topics` CLI clustering with heading-term fallback |
| `v0.15.0` | 2026-11-04 | 172 | `1f99c6bf84385673c6c8b730ba6a63f7badf9a9f` | `chromium/8058` | M5: opt-in LiteParse annotations/forms/structure/vector/complexity insights, D2 sketch rendering, MCP schema 11 with 32 tools |
| `v0.14.6` | 2026-11-04 | 170 | `9a5c4a13a269db56efbff028bd8fdfd14740d7d5` | `chromium/8058` | Guide summaries expose guide_kind/source_count/case_count; authoring and architecture docs cover manifest, QA, glossary and continuity options |
| `v0.14.5` | 2026-11-04 | 167 | `3f8c77564f1ce930f7bea3ab3ce72879fc200029` | `chromium/8058` | QA diagram count uses generated diagram assets only; architecture schema-2/3 wording corrected |
| `v0.14.4` | 2026-11-04 | 164 | `2993e5f4f1240338f50280a1d8d2154e1e8d5486` | `chromium/8058` | Guide-aware style compatibility in `styles list`/MCP `style list`; docs audit and architecture/output-artifact corrections |
| `v0.14.3` | 2026-11-04 | 162 | `bc179f0050e87dfa4991e03e06a1ab4d51f79f02` | `chromium/8058` | Report real source lines for E_MARKDOWN after frontmatter; final plan/gate reconciliation |
| `v0.14.2` | 2026-11-04 | 159 | `afb574c66c81ecdd732c048993261346116f5b41` | `chromium/8058` | TOC depth/subs, table/diagram placement, per-group D2 direction, page-complete and OCR policy fields; W2/W13 close |
| `v0.14.1` | 2026-11-04 | 157 | `87614fe25cc6e83ab73d889c223918b4ddc69fb8` | `chromium/8058` | Layout-continuity options, typed Diagnostic envelope, draft persistence, TOC/body heading review, table placement options and incremental section hashes |
| `v0.14.0` | 2026-11-04 | 154 | `99f301428baf3f2e845cd9c523ccc5f4715f8cdc` | `chromium/8058` | M2/M3 workflow core: QA/audit tools, MCP profiles, style estimate/restyle, IRAC scaffold, page roles, graph query/path/explain, claims severity, guide plan/briefs/topics/cost, D2 dagre/elk, schema 10 with 31 tools |
| `v0.13.0` | 2026-11-03 | 152 | `afb40ad3b1dbd669297731e5c90dde3d6cac24fe` | `chromium/8058` | Reviewed coverage manifest, section brief/read context tools, D2 24-node/decision support, four-column glossary tables, MCP schema 9 with 21 tools |
| `v0.12.4` | 2026-11-03 | 143 | `8038559` | `chromium/8058` | Hotfix: revision template accepts per-diagram width; compact figure sizing applies to revision diagrams too |
| `v0.12.3` | 2026-11-03 | 142 | `8038559` | `chromium/8058` | Repository hygiene: remove the orphaned pre-rotation diagram asset; correct the v0.12.1/v0.12.2 release records |
| `v0.12.2` | 2026-11-03 | 141 | `4e9c937` | `chromium/8058` | PolyForm Strict 1.0.0, compact figure sizing, per-diagram width control and glossary term spacing |
| `v0.12.1` | 2026-11-03 | 140 | `804d688` | `chromium/8058` | Accessible monochrome code blocks, strict component-contrast gate and release documentation corrections |
| `v0.12.0` | 2026-11-03 | 136 | `9b0bfb8a6d12206c935ac8f95ded161fdcb31f4d` | `chromium/8058` | Section-level source scope, deterministic document graph, local semantic links, noncommercial license, output identity and tagged reading-order/contrast checks |
| `v0.9.0` | 2026-10-30 | 100 | `2a601794355690176dc0de2d87105a088a473a2a` | `chromium/8058` | Reader/professional styles, linked TOC, canonical/visual gates, adaptive ranges |
| `v0.8.0` | 2026-10-29 | 87 | `be98e0ce17a504c4341096e48c47475f62c163f2` | `chromium/8058` | MCP object structuredContent, schema 3, public install |
| `v0.7.3` | 2026-10-29 | 84 | `49c02d5dc795c249917938059fae91c13cb5b2dc` | `chromium/8058` | MCP style-preview fix, D2 spec hardening |
| `v0.7.2` | 2026-10-29 | 82 | `65f9d15567c8ea1586ddd4b8ab620ae8719a0a94` | `chromium/8058` | Actionable diagram status, pinned D2 archive check |
| `v0.7.1` | 2026-10-29 | 80 | `3df53dc9f7ad1c3f0f1e45328b6dbe83c1982e88` | `chromium/8058` | D2 manifest/cache hardening, licence-complete archive |
| `v0.7.0` | 2026-10-29 | 77 | `1c71f9b95393b899279447f810e63b902b22868a` | `chromium/8058` | D2-only diagram pipeline, diagram MCP tool, OpenCode runbook |
| `v0.6.2` | 2026-10-29 | 67 | `e31b57331fbdf647bf37a3a2942bbad6978dec26` | `chromium/8058` | Tall-figure fitting, synthetic PDF defect fixtures, 24-table parity test |
| `v0.6.1` | 2026-10-29 | 63 | `816f9dda7b5c27d931458857250e7c26b79744e0` | `chromium/8058` | Image-overflow audit, seeded gate fixtures |
| `v0.6.0` | 2026-10-29 | 59 | `e970c19db777a62f2d22148490a53eef1fbd6a98` | `chromium/8058` | Structure packs, visual regression, table/diagram contracts, asset hardening, HTTP auth |
| `v0.5.0` | 2026-10-29 | 47 | `f366f7f96e97ead1f049441d5b9733ef1db1187c` | `chromium/8058` | Streamable HTTP MCP, cross-platform CI, install/platform docs |
| `v0.4.1` | 2026-10-29 | 44 | `3f1a867d29b4bc9d8ed4f0ae2fade0d859297de8` | `chromium/8058` | Accessibility gates, domain profiles, cross-platform CI |
| `v0.4.0` | 2026-10-29 | 41 | `bcbaa37550cc35e9eb4c0ee33fcb819c2c1ad28d` | `chromium/8058` | Schema-3 styles, paper/clean packs, `style` tool, covers metadata, tr/en labels |
| `v0.3.0` | 2026-10-28 | 39 | `c810f0316a2828cab6ab5ddb846ff2cab82add7e` | `chromium/8058` | 22-page `paper` parity, front/TOC/chapter directives, `prepare_guide`, doctor and layout audit |
| `v0.2.0` | 2026-10-27 | 23 | `1d6e20f1d2129aa901a4b9f4a574b34c0b6cfc78` | `chromium/8058` | Chunked source reads/search, source cache, empty-section coverage gate, 10 MCP tools |
| `v0.1.0` | 2026-10-26 | 16 | `fb7fe59de483f8e4e88d4346cb10f4c91f3445e2` | `chromium/8058` | First Rust/Typst/MCP release; 23-page reference guide, coverage 13/13; Python engine removed |

## Notes

- `v0.23.2` hardens the visual gate: `data/visual-candidates.json` is rejected
  when its schema/scorer version or a source digest is stale. `make acceptance`
  now runs the full v4AGENT harness, and the MCP server instruction has a
  length/content regression test. Build 247; schema 19; 41 tools.

- `v0.23.0` closes the v4 remediation plan. The generated coverage manifest now
  uses layout headings, TOC sections and embedded numbered source text; body
  prose and date-like numbers are rejected. `hint`/`covers` support ranges and
  comma lists. MCP path arguments are normalized, `guide_plan` maps source
  pages, and the candidate-backed visual gate replaces raw page-spanning
  layout blocks. The visual gate exposes `basis`, `advisory`, `status` and
  `missing_candidates`; `required` needs `data/visual-candidates.json`.
  `excluded` rows carry a reason and survive merge/adopt. OCR is opt-in via
  `ocr-tesseract`; the CLI can use `--ocr-server-url`, while MCP stays
  network-free. `docs/mcp-guidance.md` defines the five-layer instruction,
  prompt, schema and result contract. Build 241; schema 19; 41 tools.

- `v0.22.0` adds the source visual pipeline. `visual_candidates` ranks
  bbox-level candidates with `relevance` (core/supporting/decorative/seductive)
  and `reconstruction` (d2_concept/d2_schematic/table/text/unique), merges
  fragmented panels and rejects page-spanning containers. `visual_decision`
  materializes a native image or adaptive-DPI crop, validates blank/resolution/
  caption/alt/hash quality, upserts `visual-audit.json` and returns the exact
  Markdown snippet without editing `guide.md`. `visual_assets` checks,
  materializes or refreshes recorded assets. D2 remains a concept/schematic
  engine; exact charts stay source crops/tables. Visual assets are local-only
  under `data/crops/` and `data/figures/`. The manual calibration report records
  the reviewed top candidates and known gaps; `make visuals-v5` and the full
  acceptance suite pass.

- `v0.20.0` adds the visual evidence contract and A+B chart policy. The build
  now reports figure/table pages from `data/blocks`, requires a decision when
  `config.audit.visual_gate="required"` (default `warn` keeps existing guides
  buildable), and supports `describe` and provenance-checked `embed_crop`
  decisions. `crop_source`/`crop-source` writes local, gitignored crops and
  `::: figure` renders them with caption, source and alt text; the renderer
  also validates and cache-keys the crop asset. `table_extract`
  returns an actionable reason and text preview instead of a bare `[]` for
  drawn tables. `find_model_dir` searches the installed sidecar and global
  user directories. `coverage_manifest review` reports the unresolved digital
  rows while `qa` labels the legacy 41-section fallback. A-V3-01..12 and the v2
  acceptance suite pass; conditional reranking remains the only explicit
  deferral.

- `v0.19.0` closes the feedback-v2 P2 code: PDF image-bbox width regression,
  LiteParse OCR evidence, bounded graph path/explain, QA environment metadata,
  expanded cookbook/authoring docs and deterministic brief/topic cache tests.
  The final one-to-one A-V2 acceptance harness remains the release gate for 1.0.

- `v0.18.0` is the feedback-v2 P0/P1 close-out. Main coverage is 141/141
  through the review manifest; digital stays on the 41-section legacy gate
  because its review file has 60 unresolved rows. Briefs, claims and graph are
  budgeted; IRAC metrics are 10 cases/14 sections; QA cards are 9/5/8;
  `source_topics` reports a model2vec/fallback method; `source_brief` merges
  the two missing reviewed headings.

- `v0.17.1` fixes the `citation_snippet` panic when the page text contains a
  Turkish uppercase character before the query. Lowercasing can change byte
  length, so the old byte offset was not a valid char boundary. The tool now
  maps lowercase offsets back to the original text and returns normally.

- `v0.17.0` is the review close-out of the v0.16.0 plan: D2 metadata and
  size contracts, audit intentional pages/table-free line stats, actual section
  pages, brief cache reuse, chapter/heading briefs, coverage hints, response
  budgets, doctor style checks and the `make acceptance` suite. MCP schema is
  13 with 35 tools. Conditional reranking remains an explicit deferral.

- `v0.16.0` closes the remaining core gaps: TOC include/exclude, structured
  build failures with `draft_path`, MCP `doctor`, `read_section` modes and
  `max_tokens`, opt-in definition/must-know cards, source-page resources and
  resource templates, D2 multiline labels/tooltip/link/local icon metadata,
  Model2Vec topics file consumed by MCP, and the first-three page audit
  heuristic removed. MCP schema is 12 with 33 tools.

- `v0.15.1` adds `folisift topics --guide <slug>`: greedy clustering over
  heading embeddings from the bundled local Model2Vec model, with `--no-model`
  as the deterministic heading-term fallback. No network or LLM call.

- `v0.15.0` closes M5 optional polish. `folisift insights --source` and MCP
  `source_insights` expose annotations, AcroForm fields, tagged structure,
  vector geometry, image refs and per-page complexity as read-only diagnostics.
  D2 specs gain an opt-in `sketch` flag; legal/medical guides keep it off by
  default. MCP schema is 11 with 32 tools.

- `v0.14.6` exposes `guide_kind`, `source_count` and `case_count` in
  `list-guides`/MCP guide summaries, and updates the authoring/architecture
  docs for the reviewed manifest, `qa`, four-column glossaries and continuity
  options. No guide content or page layout changes.

- `v0.14.5` fixes the `qa` diagram count: it now counts only
  `assets/generated/*.svg` diagram assets, not unrelated figures. The
  architecture contract now states that schema 2 is legacy and schema 3 is the
  current shape.

- `v0.14.4` adds a guide-aware compatibility column to `styles list
  --guide <slug>` and MCP `style list`; it also corrects the architecture
  docs for untracked output PDFs and the schema-3-only contract. No rendered
  guide content changes.

- `v0.14.3` maps renderer line numbers back to the source Markdown after
  frontmatter, so `E_MARKDOWN` points at the real line. No layout or content
  change.

- `v0.14.2` closes W2/W13: TOC `subs`/`toc_depth`, table and diagram
  placement options, per-group D2 direction, front/table close-order docs, and
  incremental section hashes. `read_source` reports `page_complete`; source
  briefs report `pages_ocr: 0` and `manual_review`. Engine-depth W15–W17 remain
  partial and are recorded in the plan.

- `v0.14.1` adds chapter/columns/table/diagram continuity options
  (`keep_with_next`, `break_before`, `break_after`, `allow_split`,
  `full_page_opener`, `allow_page_break`, `min_space_mm`), a typed
  `Diagnostic` envelope with `E_MARKDOWN` directive stacks, checked-build
  `_draft.pdf` persistence with `draft_path`, TOC/body heading review rows and
  incremental section hashes. The release remains schema 10 with 31 tools.

- `v0.14.0` adds the M2/M3 workflow core: one `qa_guide` release report,
  `audit_guide`, MCP tool profiles (`core` is ten tools; active profiles stay
  ≤15), paginated resources, style compatibility/estimate/restyle, IRAC case
  scaffolds with explicit heading levels, page roles, graph query/path/explain,
  claim severity and `verify_citations`, page budget, source brief/topics, guide
  plan, visual brief, citation snippets, cost reports and D2 dagre/elk layouts.
  MCP schema is 10 with 31 tools. W2/W3/W5/W9/W13/W15–W17 remain partial; their
  named gaps are recorded in the remediation plan.

- `v0.13.0` adds the reviewed coverage manifest (`coverage-manifest` CLI/MCP),
  bounded `section_brief` / `read_section` retrieval, D2 decision diagrams with
  24 nodes / 40 edges / depth 8, and the four-column `::: glossary` table with
  compact headers and atomic term/definition rows. The MCP tool schema is 9 and
  exposes 21 tools. PD0–PD1 is 23 pages, PD2 24 and PD3 21; all five exam PDFs
  pass the strict audit, TOC, reading-order and contrast gates.

- `v0.12.4` fixes the revision template so it accepts the per-diagram width
  argument added in v0.12.2; revision diagrams now use the same 90% default and
  14 cm cap as the other layouts.

- `v0.12.3` removes the orphaned SVG left by the v0.12.2 diagram rotation and
  corrects the tag-target commit records for v0.12.1 and v0.12.2. No rendered
  guide content changes.

- `v0.12.2` moves the project from PolyForm Noncommercial to PolyForm Strict
  1.0.0. Noncommercial use remains permitted; distribution, modification,
  derivative works, forks, maintenance and commercial use require prior written
  permission. Earlier releases keep the license they were published with.
  Figures now use 90% of the content width by default with a 14 cm height cap,
  diagrams can request 50–100% width, and glossary terms get explicit block
  spacing under their headings.

- `v0.12.1` disables Typst's default raw syntax-highlighting colours so code
  keeps the body ink contrast, makes the component-contrast check strict in
  `make exam-check`, and refreshes the release documentation.

- `v0.12.0` adds section-level `source_scope`, the deterministic
  `document-graph.json` and `graph` CLI/MCP surface, local Model2Vec links with
  precision-first defaults, tagged-PDF reading-order and rendered contrast
  checks, the offline exam copy target and the PolyForm Noncommercial license.
  Generated PDFs carry the Folisift identity on every page and the repository
  link on the cover. Generated PDFs are no longer tracked by git.

- `v0.9.0` adds the reader/journal/brief/accessible packs, linked TOC,
  canonical source-topic coverage, visual-audit and underfilled-page gates,
  adaptive page ranges, recall/review/explain components and the production
  report.
- `v0.8.0` wraps `list_guides` and `style list` results in objects so strict
  MCP clients accept `structuredContent`; the MCP tool schema is now 3. The
  repository is public again, so the one-command installer works.
- `v0.7.3` fixes MCP style preview and rejects duplicate diagram group
  membership.
- `v0.7.2` reports invalid diagram specs in status and rejects a package whose
  D2 sidecar is not pinned.
- `v0.7.1` rejects unknown diagram fields, locks the layout to TALA, hashes
  diagram fonts, requires audited D2 source, and packages the D2 licence.
- `v0.7.0` makes D2 v0.9.0 the single external diagram engine. D2 runs only in
  `diagram build`; guide builds and MCP embed pre-generated SVG. Adds the
  diagram MCP tool (schema 2), OpenCode runbook and D2 packaging notice.
- `v0.6.2` fits tall diagrams inside the page and adds the remaining synthetic
  PDF defect fixtures plus the 24-table parity test.
- `v0.6.1` adds the image-overflow layout gate and checked-in seeded fixtures.
- `v0.6.0` adds structure packs, the table/diagram content contracts, pixel
  visual regression, bookmark and asset gates, source-page cropping and HTTP
  bearer auth. The Linux archive is built by `make package`; macOS and Windows
  remain source builds. Archives are checksum-verified but unsigned.
- `v0.5.0` adds streamable HTTP MCP and the install/platform contract. Archives
  are checksum-verified but unsigned.
- `v0.4.1` adds accessibility gates and domain profiles; `paper` remains the
  stable default, `clean` is experimental.
- `v0.3.0` is the paper-parity release: the default template matches the
  22-page reference, and `make parity` checks the page map plus layout audit.
- `v0.2.0` is the LLM-hands release: the server provides bounded source text,
  search and content-aware coverage; the host agent still writes all prose.
- The release archive contains `folisift`, PDFium, D2, the semantic model and
  the license/notice files. See ADR-0005.
- Do not tag an unreleased row without running the checklist in
  `docs/agents/versioning.md`.
