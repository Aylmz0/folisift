# Versions

Build numbers are derived with `git rev-list --count HEAD` at compile time.
SemVer is assigned by an annotated tag; the tag is the authority, not this
table.

| Tag | Date | Build | Commit | PDFium | What changed |
|---|---|---:|---|---|---|
| `v0.4.1` | 2026-10-29 | 45 | `0107114a232a44d0619dec16c4bf4989cdaec29a` | `chromium/8058` | Accessibility gates, domain profiles, cross-platform CI |
| `v0.4.0` | 2026-10-29 | 42 | `f14d2562e3a6e2a2b998f9a4c3a60f848ae6bc92` | `chromium/8058` | Schema-3 styles, paper/clean packs, `style` tool, covers metadata, tr/en labels |
| `v0.3.0` | 2026-10-28 | 40 | `8fba8aebe303e8524d595086ba977ac6904d82de` | `chromium/8058` | 22-page `paper` parity, front/TOC/chapter directives, `prepare_guide`, doctor and layout audit |
| `v0.2.0` | 2026-10-27 | 22 | `e41c1f8b1e1e5ff84716b6c8ab20dddaf0732f4e` | `chromium/8058` | Chunked source reads/search, source cache, empty-section coverage gate, 10 MCP tools |
| `v0.1.0` | 2026-10-26 | 16 | `bb261447e5d20ca8da5de0e868ab8d821b0cda63` | `chromium/8058` | First Rust/Typst/MCP release; PD0–PD1 guide, 23 pages, coverage 13/13; Python engine removed |

## Notes

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
