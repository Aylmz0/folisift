# Versions

Build numbers are derived with `git rev-list --count HEAD` at compile time.
SemVer is assigned by an annotated tag; the tag is the authority, not this
table.

| Tag | Date | Build | Commit | PDFium | What changed |
|---|---|---:|---|---|---|
| `v0.7.0` | 2026-10-29 | 78 | `64b0e2620be05551716c9e1e743785a85b90268e` | `chromium/8058` | D2-only diagram pipeline, diagram MCP tool, OpenCode runbook |
| `v0.6.2` | 2026-10-29 | 68 | `1fa1a7b2a6dc40c18e6b46ce4460289df6628569` | `chromium/8058` | Tall-figure fitting, synthetic PDF defect fixtures, 24-table parity test |
| `v0.6.1` | 2026-10-29 | 64 | `48503766022525e25cb0a0df8141c6ebe8de9009` | `chromium/8058` | Image-overflow audit, seeded gate fixtures |
| `v0.6.0` | 2026-10-29 | 60 | `06417467029510a016d598c997bda5e767192683` | `chromium/8058` | Structure packs, visual regression, table/diagram contracts, asset hardening, HTTP auth |
| `v0.5.0` | 2026-10-29 | 48 | `b2f56c1b7bd56621a7a758b85efe2b0e2a9c1739` | `chromium/8058` | Streamable HTTP MCP, cross-platform CI, install/platform docs |
| `v0.4.1` | 2026-10-29 | 45 | `0107114a232a44d0619dec16c4bf4989cdaec29a` | `chromium/8058` | Accessibility gates, domain profiles, cross-platform CI |
| `v0.4.0` | 2026-10-29 | 42 | `f14d2562e3a6e2a2b998f9a4c3a60f848ae6bc92` | `chromium/8058` | Schema-3 styles, paper/clean packs, `style` tool, covers metadata, tr/en labels |
| `v0.3.0` | 2026-10-28 | 40 | `8fba8aebe303e8524d595086ba977ac6904d82de` | `chromium/8058` | 22-page `paper` parity, front/TOC/chapter directives, `prepare_guide`, doctor and layout audit |
| `v0.2.0` | 2026-10-27 | 22 | `e41c1f8b1e1e5ff84716b6c8ab20dddaf0732f4e` | `chromium/8058` | Chunked source reads/search, source cache, empty-section coverage gate, 10 MCP tools |
| `v0.1.0` | 2026-10-26 | 16 | `bb261447e5d20ca8da5de0e868ab8d821b0cda63` | `chromium/8058` | First Rust/Typst/MCP release; PD0–PD1 guide, 23 pages, coverage 13/13; Python engine removed |

## Notes

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
- The canonical guide output is `output/PD0-PD1_Calisma_Ozeti.pdf`.
- The release archive contains `folisift` and the platform PDFium shared
  library. See ADR-0005.
- Do not tag an unreleased row without running the checklist in
  `docs/agents/versioning.md`.
