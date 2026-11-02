# Table extraction parity — 2026-11-03

- **Status:** Decision record
- **Scope:** PD0, PD1, PD2, PD3 course PDFs
- **Compared:** LiteParse `table`/`merged_table` blocks vs the legacy
  `table.rs` native-text draft extractor
- **Decision:** layout table blocks are the primary path; the native-text
  extractor remains a source-only fallback.

## Method

For every page where LiteParse emitted a table block, the page was sent to
`folisift table extract` using the legacy extractor. Counts were compared with
the layout artifact in `data/blocks/<source>.json`.

## Results

| Source | Pages with layout tables | Layout tables | Legacy `table.rs` tables |
|---|---:|---:|---:|
| PD0 | 10 | 13 | 0 |
| PD1 | 4 | 5 | 0 |
| PD2 | 6 | 6 | 0 |
| PD3 | 10 | 13 | 0 |
| **Total** | **30** | **37** | **0** |

The legacy extractor found no tables on the real slide pages. It still finds
the synthetic `blocks.pdf` table, but that is not representative of the
course material.

## Consequences

- `folisift table extract --guide <slug>` now reads LiteParse table blocks and
  returns header, rows and per-cell bounding boxes.
- `folisift table extract` without `--guide` keeps the legacy draft path for
  source-only diagnostics.
- Guide authoring should use the layout table view; the legacy output is not a
  coverage or content source.
- `table.rs` is not used by the guide build and is now a fallback, not the
  primary extraction path.
