# ADR-0008: Use liteparse as the only PDF parser

- **Status:** Accepted
- **Date:** 2026-10-26
- **Deciders:** Maintainer

## Context

The reference pipeline shells out to Poppler (`pdfinfo`, `pdftotext`,
`pdftoppm`). That is another runtime dependency and gives the server no
structured page model. `liteparse` is a Rust parser with PDFium text extraction,
bounding boxes, page screenshots, complexity detection and optional OCR.

## Decision

- `liteparse 2.17` with `default-features = false` (no Tesseract) is the only
  parser.
- OCR is off by default. Low-text pages are rendered to PNG for the host agent.
- `folisift-parse` exposes a `SourceParser` trait; production uses
  `LiteparseParser`.
- Inventory keeps the existing JSON shape: page map, words, chars, headings,
  low-text pages and source identity.
- The low-text rule remains “fewer than 120 non-whitespace characters”.

## Consequences

- Poppler is no longer needed by the v2 path.
- PDFium is a runtime sidecar (ADR-0005).
- `is_complex` is broader than the low-text rule; it is used for routing, not
  as the inventory's low-text column.
- Text parity on PD0/PD1 is exact for pages, chars, low-text pages and headings;
  word counts differ by +52/−4 and first-line strings differ only in
  projection/typography details.
