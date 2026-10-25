# Phase 6 — Parity Audit Report

- **Date:** 2026-10-26
- **Status:** Audit complete; removal and release pending maintainer approval
- **Plan:** [`2026-10-26-rust-mcp-typst.md`](2026-10-26-rust-mcp-typst.md)

## Compared artifacts

| Artifact | Path | Pages |
|---|---|---:|
| Python reference | `output/PD0-PD1_Calisma_Ozeti.pdf` | 22 |
| Rust/Typst v2 | `output/PD0-PD1_v2.pdf` | 23 |

## Content parity

| Check | Python reference | Rust v2 | Result |
|---|---:|---:|---|
| Numbered subsections (coverage) | 13/13 | 13/13 | exact |
| `definition` boxes | 4 | 4 | exact |
| `critical` boxes | 4 | 4 | exact |
| `example` boxes | 3 | 3 | exact |
| `note` boxes | 2 | 2 | exact |
| Figures | 4 | 4 | exact |
| H2 headings | 16 | 16 | exact |
| H3 headings | 48 | 48 | exact |
| PDF pages | 22 | 23 | +4.5% (inside ±10%) |
| TOC | present | present | pass |
| PDF bookmarks | present | present | pass |
| Turkish glyphs | pass | pass | pass |
| Tagged PDF | — | yes | pass |
| Metadata title/author | pass | pass | pass |

The raw `pdftotext` word sequences are not identical: the Typst PDF uses
hyphenation and different line breaking, and the PDFs contain different front
matter/TOC text. This is a layout/projection difference, not missing content;
the source headings, directive counts, figures and coverage report agree.

## Visual review

Selected pages were rendered to PNG and inspected:

- cover and TOC;
- front matter and definition boxes;
- first chapter, lists and source references;
- tables and table headers;
- the compiler pipeline and parse-tree figures;
- code blocks and example boxes;
- the G coverage section.

No clipped tables, figures or code blocks were found. The table header bug
found during the first render was fixed before this audit.

## Removal checklist (not executed)

1. Maintainer accepts the 23-page Rust output and the +4.5% page delta.
2. Switch `guides/pd0-pd1/config.json` output to the v2 PDF.
3. Remove `legacy_content` and the Python build/tool dependencies.
4. Delete `engine/` and the Python-specific docs after the final reference
   build is archived.
5. Tag `v0.1.0` and attach `folisift` + `libpdfium.so`.
6. Update `docs/VERSIONS.md` and the README.

Until then, both builds remain valid and the Python engine is the reference.
