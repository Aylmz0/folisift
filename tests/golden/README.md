# Golden reference

`output/old-python.pdf` is the visual and structural reference for the default
`paper` style.

- **Pages:** 22
- **SHA-256:** `115b97568cf4bf133f79d1f5cde30b563d640b18881e48f100947686583662ed`
- **Engine:** Python + WeasyPrint (retired)
- **Role:** golden reference only; not a product output

## Capture

```bash
make parity-capture
```

This writes page PNGs, extracted text and `pdfinfo` to
`target/parity/reference/` (gitignored). The script uses Poppler only in the
development environment; the product does not depend on it.

Selected design pages:

```text
1  cover
2  front matter
3  TOC
4  chapter 0 opener
5  definition + tables
10 SDLC figure + audit table
11 chapter 1 opener
18 C glossary + comparison table
19 D tables
20 E tables
21 F tables
22 G coverage
```

## Rules

- Do not edit the old PDF.
- Do not commit generated captures.
- Current output must match the page map and selected-page visual thresholds in
  `docs/plans/2026-10-28-visual-parity-and-production.md`.
- The C page is the canary: two columns and the comparison table must fit on
  one page.
