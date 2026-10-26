# Versions

Build numbers are derived with `git rev-list --count HEAD` at compile time.
SemVer is assigned by an annotated tag; the tag is the authority, not this
table.

| Tag | Date | Build | Commit | PDFium | What changed |
|---|---|---:|---|---|---|
| `v0.2.0` | 2026-10-27 | 22 | `9683e23f17bedb86333cbe8a3cf2f355ef3c2422` | `chromium/8058` | Chunked source reads/search, source cache, empty-section coverage gate, 10 MCP tools |
| `v0.1.0` | 2026-10-26 | 16 | `bb261447e5d20ca8da5de0e868ab8d821b0cda63` | `chromium/8058` | First Rust/Typst/MCP release; PD0–PD1 guide, 23 pages, coverage 13/13; Python engine removed |

## Notes

- `v0.2.0` is the LLM-hands release: the server provides bounded source text,
  search and content-aware coverage; the host agent still writes all prose.
- The canonical guide output is `output/PD0-PD1_Calisma_Ozeti.pdf`.
- The release archive contains `folisift` and the platform PDFium shared
  library. See ADR-0005.
- Do not tag an unreleased row without running the checklist in
  `docs/agents/versioning.md`.
