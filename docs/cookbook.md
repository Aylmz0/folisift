# Cookbook and common errors

This page maps the most common build/audit failures to a concrete next action.
It is a host-facing cookbook; the normative rules stay in
[authoring-spec.md](authoring-spec.md).

## Fast diagnosis

```bash
folisift doctor
folisift guide-status <slug>
folisift qa --guide <slug>
```

`qa` returns one report with build, coverage, audit, visual, cards, claims and
diagram counts. A failed gate is never hidden behind a bare `passed`.

## Common errors

| Message | Meaning | Action |
|---|---|---|
| `page count is N, expected M` | The PDF changed size. | Check the page budget and update `target_pages` only when the content change is intentional. |
| `underfilled body pages: [N]` | A body page is below 35% of the median. | Use `folisift budget --guide <slug>`; move a figure/table or cut the largest section. TOC/chapter/figure pages are role-exempt. |
| `E_TABLE_WIDTHS: ... sum to N%` | Table widths do not sum to 100. | Fix the `widths=` list; the message names the column count and sum. |
| `structure "irac" validation failed` | An IRAC heading is at the wrong level. | IRAC uses `## Dava` plus `### Sorun/Kural/Uygulama/Sonuç`. Read the reported line and expected `###`. |
| `coverage ... missing` | A manifest/source heading has no guide heading or `<!-- covers -->` claim. | Add the section or mark it `merged`/`excluded` in the reviewed manifest. |
| `review manifest ignored: N row(s) still marked missing` | A generated skeleton is sitting at the review path. | Review every canonical row; do not rename a skeleton `*.review.json`. |
| `empty_sections` | No substantive body is mapped to a covered row. | Map the source row to the exact guide chapter; a short body is `weak_sections`, not empty. |
| `coverage_manifest` finds no file | The guide has neither review, generated nor combined manifest. | Run `coverage-manifest build`; legacy `check_coverage` remains available. |
| `no recall/answer pairs` | `cards` found no `::: recall`/`::: answer` blocks. | Add the blocks or accept an empty deck explicitly; the warning is intentional. |
| `unknown FOLISIFT_MCP_PROFILE` | The host asked for an unknown profile. | Use `core`, `source`, `authoring`, `review`, `fast`, `deep` or `full`. |

## MCP profiles

The MCP server defaults to `full`. A profile hides tools before `tools/list`, so
the active schema stays small:

```bash
folisift mcp --profile core
folisift mcp --profile fast
```

- `core` — ten tools for prepare/read/brief/build/qa/coverage.
- `source` — reading, briefs, topics, citation and visual evidence.
- `authoring` — scaffold, build, QA, coverage, diagrams and cards.
- `review` — audit, QA, coverage, claims, links, graph and diagrams.
- `fast` — short source/brief/build/QA path.
- `deep` — source topics, table extraction, links and graph.

## Coverage manifest

Precedence is `coverage-manifest.review.json` → `coverage-manifest.generated.json`
→ `coverage-manifest.json` → legacy gate:

```bash
folisift coverage-manifest --guide <slug> --action build
# review data/coverage-manifest.review.json; never edit the generated skeleton
folisift coverage-manifest --guide <slug> --action validate --manifest review
folisift build --guide <slug> --check
```

`build` writes the generated skeleton and never overwrites a review file. A
review file with any `missing` row is reported and ignored by `qa`/`build`; fix
the review before trusting `coverage_source=review`. `empty_sections` means no
substantive body; a short body is `weak_sections`.

## IRAC scaffold

```bash
folisift scaffold --slug law-101 --title "Law 101" \
  --style accessible --structure irac --domain law \
  --source sources/PD0.pdf
```

The scaffold writes ten `## Dava N` placeholders with `### Sorun`, `### Kural`,
`### Uygulama`, `### Sonuç` and puts raw source headings in
`guides/<slug>/source-headings.md`. Existing guides are preserved unless a new
slug is used.

## Context briefs

`section_brief`, `chapter_brief` and `heading_brief` accept `max_tokens`.
The budget applies to the returned JSON, not just the summary:

```bash
folisift section-brief --guide <slug> --source <id> --heading 4.4.3 --max-tokens 50
folisift read-section --guide <slug> --source <id> --heading 4.4.3 --mode summary
```

A small budget returns `truncated: true`, `next_offset` and `full_chars`; it
never returns the full 3,000-character section silently. `chapter_brief` uses
exact number matching: `4` selects `4 ...`, not `1.4 ...`.

## D2 diagrams

```bash
folisift diagram validate --guide <slug> --all
folisift diagram build --guide <slug> --id <id>
folisift diagram fit --guide <slug> --id <id> --width 60
folisift diagram status --guide <slug>
```

D2 is found from `FOLISIFT_D2`, `PATH`, the binary’s sibling, or
`~/.local/lib/folisift/d2`. `layout` accepts `tala`, `dagre` or `elk`; the chosen
layout is recorded in the manifest. `width` changes the rendered bbox; a tall
diagram needs `allow_page_span=true`. Guide builds only embed pre-generated SVG
and never execute D2.

## Chart-heavy sources

Use **A + B**:

- **A:** describe the chart in prose, extract its table when present, and use a
  D2 schematic only for conceptual relationships;
- **B:** crop the source chart when its exact appearance matters. Keep the
  caption, source page, alt text and crop hash; the crop is local-only.

Do not ask D2 to draw XY/scatter/ROC/quantile charts. It is a diagram engine,
not a chart engine. A chart renderer is only safe when the data comes from a
table or an explicit JSON file.

`visual_evidence` lists figure/table/vector evidence. Set
`config.audit.visual_gate` to `required` for a chart-sensitive guide; `warn`
(the default) reports missing decisions without breaking existing guides.

## Visual decisions

Use `native_table` when the source already contains a table, `redraw_table` when
a figure/matrix must become a table, and `redraw_figure` when the figure itself
must be redrawn. Use `embed_crop` for a source crop and `describe` for a
text-only chart note. Legacy `table` is accepted as `redraw_table` for one
release. `table_extract` returns `reason`, `table_likely`, `manual_action` and a
text preview when no table block exists.

## OCR and insights

`source_insights` reports `ocr_evidence`: policy, pages OCRed/skipped, candidate
pages and reasons. The default is off; `ocr=true` or `--ocr` is an explicit
opt-in. `source_brief` reports the same policy and low-text candidates.

Run `make feedback-v2` for the external acceptance matrix (A-V2-01…18).

`qa` includes an `environment` block with version, build, PDFium, LiteParse, D2,
model signature, OCR/reranker policy and the topic seed. Two runs with the same
inputs should report the same content hash, coverage and card/claim counts.
