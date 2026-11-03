# Accessibility review — 2026-11-03

- **Status:** Automated checks complete; manual screen-reader listen still a
  release smoke step
- **Scope:** `PD0-PD1`, `PD2`, `PD3`, `PD2_Revizyon`,
  `PD2_LISP_Bilgi_Organizatoru`
- **Tools:** PyMuPDF scripts, Orca 50.3 installed but no graphical session in
  this environment

## 1. Tagged-PDF reading order

`scripts/check-reading-order.py` checks the structure a screen reader consumes:

- `/MarkInfo /Marked true`, `/StructTreeRoot` and a non-empty `/Lang`
- structure-tree page order and heading order move forward only
- every page has `/StructParents`; the parent tree exists
- paragraph, heading and table roles are present; every table has `TR`, `TD`
  and header `TH` cells
- content figures carry `/Alt`; the decorative cover motif is the documented
  page-1 exception
- every page's content-stream sequence is monotonic
- the outline has at least three entries and does not jump backwards

Result:

| PDF | Pages | Structure elements | Headings | Figures | Outline | Order |
|---|---:|---:|---:|---:|---:|---|
| `PD0-PD1` | 22 | 1623 | 55 | 5 | 71 | monotonic |
| `PD2` | 24 | 1935 | 59 | 4 | 82 | monotonic |
| `PD3` | 21 | 1785 | 50 | 6 | 58 | monotonic |
| Revision | 4 | 234 | 1 | 1 | 9 | monotonic |
| Organiser | 1 | 128 | 5 | 0 | 5 | monotonic |

The check runs in `make exam-check` and as `make reading-order-check`.

Manual protocol for a release where a graphical session is available:

1. Open each PDF in a tagged-PDF-capable viewer (Evince, Okular or a browser)
   with Orca/NVDA running.
2. Start at the cover and use the screen reader's next-element command, not
   arrow keys, through cover → TOC → chapter heading → body → table → figure
   caption.
3. Confirm the reading order matches the visible order, table cells are
   announced with their header, every content figure has a description and
   the outline entries announce the printed section names.
4. Record the pass in this file before publishing the archive.

## 2. Component-level contrast

`scripts/check-component-contrast.py` renders every page at 2× and compares
each text span with the dominant colour inside its box. This catches text on
table fills, callouts, code backgrounds and diagram nodes, which the palette
pair check cannot see.

Result:

| PDF | Text spans | Below threshold |
|---|---:|---:|
| `PD0-PD1` | 1876 | 0 |
| `PD2` | 2173 | 0 |
| `PD3` | 2031 | 0 |
| Revision | 241 | 0 |
| Organiser | 98 | 0 |

The earlier code-token findings were cleared by disabling Typst's default raw
syntax-highlighting colours (`set raw(theme: none)`), so code inherits the
body ink colour on the sand surface. The strict contrast check now passes and
runs in `make exam-check`.

Use `make component-contrast-check` for the report and
`make component-contrast-strict` for the gate.

## 3. Multi-page overflow

`tests/fixtures/pdf/multipage-overflow.pdf` places the defect on page 2;
`crates/folisift-parse/tests/audit.rs` asserts
`content_box_pages == [2]`. This proves the audit does not only inspect the
first page.

Every genre preview is already rendered and audited by
`scripts/check-style-previews.sh` (`make style-audit`), so the multi-page
preview path is covered for notes, workbook, reference, revision and the
reader layouts. The synthetic fixture is the negative case for a page break.

## 4. Public docs pass

- README examples use generic guide/source IDs instead of course IDs.
- `docs/architecture.md` shows a schema-3 config with neutral paths.
- Release and development versions remain separate: `v0.9.0` is the latest
  tag; `v0.11.0` is the development build.
- Course-specific plans, pilots and parity reports stay under `docs/plans/`
  and dated evidence files; they are not generic instructions.

## 5. Offline exam copy

`make offline-exam` copies exactly the five final PDFs to
`output/offline-exam/` and ignores `output/style-preview-*`. The folder is
gitignored; regenerate it from the immutable exam set after `make exam-check`.
