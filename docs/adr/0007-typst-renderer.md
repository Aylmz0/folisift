# ADR-0007: Render guides with Typst from Markdown

- **Status:** Accepted
- **Date:** 2026-10-26
- **Deciders:** Maintainer

## Context

The reference renderer is HTML/CSS through WeasyPrint. A single-file product
cannot carry Python, Pango and WeasyPrint. Typst is a Rust typesetting engine
that supports A4 pages, fonts, tables, code, SVG, TOC and PDF bookmarks, and it
can be embedded as a library.

Agents write prose naturally in Markdown. Letting them write Typst would leak
the design system into content and make the output brittle.

## Decision

- Content format: Markdown + YAML frontmatter + four directives
  (`definition`, `critical`, `example`, `note`).
- Renderer: `typst` + `typst-pdf` as a library, with `templates/paper.typ`.
- Fonts are embedded in the binary: Noto Serif, Inter, Noto Sans Mono.
- The Typst template owns cover, TOC, headings, boxes, tables, figures, code,
  headers, footers and page numbers.
- Guides never contain raw Typst or CSS.

## Consequences

- The HTML/CSS design is ported once; the design system moves from
  `engine/style.css` to `templates/paper.typ`.
- Page count changes because the layout engine changes. The parity band is
  ±10%; the current pilot is 23 pages vs 22.
- Font licensing and binary size become release concerns.
- A Markdown coverage gate replaces the HTML-only checker for v2.
