# Versions

Build numbers are derived with `git rev-list --count HEAD` at compile time.
SemVer is assigned by an annotated tag; the tag is the authority, not this
table.

| Tag | Date | Build | Commit | PDFium | What changed |
|---|---|---:|---|---|---|
| `v0.12.2` | 2026-11-03 | 141 | `804d688` | `chromium/8058` | PolyForm Strict 1.0.0, compact figure sizing, per-diagram width control and glossary term spacing |
| `v0.12.1` | 2026-11-03 | 140 | `c97fe65` | `chromium/8058` | Accessible monochrome code blocks, strict component-contrast gate and release documentation corrections |
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
