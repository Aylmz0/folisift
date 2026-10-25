# Phase 3 — Typst Renderer and Content Port Report

- **Date:** 2026-10-26
- **Status:** Complete — parity gate passed
- **Plan:** [`2026-10-26-rust-mcp-typst.md`](2026-10-26-rust-mcp-typst.md)

## What changed

- Added `folisift-render`: Markdown + YAML frontmatter + `::: definition`,
  `::: critical`, `::: example`, `::: note` directives to Typst.
- Added `templates/paper.typ`: cover, TOC, running header, page numbers,
  paper palette, headings, boxes, tables, figures and code.
- Embedded Noto Serif, Inter and Noto Sans Mono in the binary
  (`templates/fonts/`).
- Ported `guides/pd0-pd1/guide.html` to `guide.md` with a one-off converter.
- Added the Rust Markdown coverage gate and `folisift build`.
- `config.json` is now schema 2 with `content: guide.md`;
  `legacy_content: guide.html` keeps the Python reference build alive.

## Render result

| Check | Result |
|---|---|
| Pages | **23** (Python reference: 22; +4.5%) |
| Coverage | **13/13 PASS** |
| PDF version | 1.7 |
| Tagged PDF | yes |
| Metadata | title and author set |
| TOC | present |
| PDF bookmarks | present, including H1/H2/H3 |
| Turkish glyphs | `ğüşıöç ĞÜŞİÖÇ` intact |
| Figures | 4 SVG figures rendered; source figure numbers preserved |
| Tables | headers and rows render; no clipping observed |
| Code | raw blocks with mono font; no clipping observed |
| Warnings | 0 |

The page-count change comes from the Typst flow and the 10.2 pt body size. It
is inside the plan's ±10% parity band.

## Build

```bash
cargo run -p folisift-cli -- build --guide pd0-pd1 \
  --output output/PD0-PD1_v2.pdf --check
```

The Python reference still builds:

```bash
python3 engine/build.py --guide guides/pd0-pd1 --check
```

## Known differences / follow-ups

- Source references render as muted `[PD0, s. 3]`; the HTML design used a
  small inline span. Visual intent is preserved.
- Figure captions keep the source numbers (`Şekil 0.2`) instead of Typst
  auto-numbering, so source references stay stable.
- The PDF bookmarks include H3 entries; the visible TOC is depth 2.
- The one-off HTML→Markdown converter is not committed; `guide.md` is the
  source of truth from now on.
- The Python engine remains the reference until the MCP server and the final
  parity review (Phase 6).

## Next

Phase 4: `rmcp` stdio server and the tool/resource/prompt contract.
