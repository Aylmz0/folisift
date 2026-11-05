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

Precedence is `coverage-manifest.json` (adopted combined) →
`coverage-manifest.review.json` (eligible review) →
`coverage-manifest.generated.json` (advisory) → legacy gate:

```bash
folisift coverage-manifest --guide <slug> --action build
# review data/coverage-manifest.review.json; never edit the generated skeleton
folisift coverage-manifest --guide <slug> --action validate --manifest review
folisift coverage-manifest --guide <slug> --action build --adopt
folisift build --guide <slug> --check
```

`build` writes the generated skeleton and never overwrites a review file.
`adopt` promotes an eligible review to combined. Generated manifests are
advisory by default; use `--manifest generated` or
`"audit": {"manifest_gate": "generated"}` only when you deliberately want that
hard gate. A failed check writes `output/<slug>_draft.pdf`; it does not replace
the normal output. `empty_sections` means zero body characters; a section with
1–399 characters is `weak_sections` (threshold `weak_section_chars`).

`coverage_manifest hint` is advisory. Merged guide headings such as
`## 2.1–2.2 ...` cover every source number in the range with score `1.0`;
`fields` and `max_tokens` trim the hint response like any other tool.

## Palette refresh

When the user is bored of the current colours, list the curated library and
switch only the palette:

```bash
folisift styles palettes
folisift styles restyle --guide <slug> --palette forest-sage
folisift build --guide <slug> --check
```

`--style` is optional; the guide keeps its current layout. MCP:
`style({action:"palettes"})` then
`style({action:"restyle", guide:"<slug>", palette:"berry"})`. Restyle does not
modify guide content and records `content_unchanged`. The library covers warm,
natural, cool, neutral, grayscale and high-contrast families; each palette has
a `use` note for the host.

## IRAC scaffold

```bash
folisift scaffold --slug law-101 --title "Law 101" \
  --style accessible --structure irac --domain law \
  --source sources/PD0.pdf
```

The scaffold writes ten `## Dava N` placeholders with `### Sorun`, `### Kural`,
`### Uygulama`, `### Sonuç`, uses the real source id in placeholder citations,
and puts raw source headings in `guides/<slug>/source-headings.md`. Existing
guides are preserved unless a new slug is used.

## Context briefs

`section_brief`, `chapter_brief` and `heading_brief` accept `max_tokens`.
The budget applies to content (`estimated_tokens`: summary + selected arrays),
not to the JSON envelope; `response_tokens` reports the full response separately.
A minimum useful summary is kept even at a tiny budget:

```bash
folisift section-brief --guide <slug> --source <id> --heading 4.4.3 --max-tokens 50
folisift read-section --guide <slug> --source <id> --heading 4.4.3 --mode summary
```

A small budget returns `truncated: true`, `next_offset` and `full_chars`; it
never returns the full 3,000-character section silently and never returns an
empty summary for a non-empty section. `chapter_brief` uses exact chapter
matching (`4` selects `4 ...`, not `1.4 ...`) and returns the parent body plus
one bounded summary per immediate child section.

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

Start with `folisift visual candidates --guide <slug> --source <path>` (MCP
`visual_candidates`). It returns ranked candidates with bbox, bounded context,
`relevance`, `reconstruction`, `score_parts`, `suggested` and `review_mode`.
Set `config.audit.visual_gate` to `required` for a chart-sensitive guide;
`warn` (the default) reports missing decisions without breaking existing guides.
Use `visual assets --check` as the strict preflight and `visual assets
--materialize` to regenerate a missing crop from its stored recipe.

## Visual decisions

Use `visual decision` to materialize and record the choice in one step:

```bash
folisift visual decision --guide <slug> --source sources/PD0.pdf \
  --page 45 --candidate <source>:p45:img1 --decision embed_image \
  --caption "ROC eğrisi ve eşik noktaları" \
  --alt "ROC eğrisi; yatay eksende FPR, dikey eksende TPR" \
  --reason "Eşik değerleri metinde yok."
```

It writes the asset, validates quality, upserts `visual-audit.json` and prints
the `::: figure` snippet. It never edits `guide.md`.

Use `native_table` when the source already contains a table, `redraw_table`
when a figure/matrix must become a table, and `redraw` for a D2 concept
diagram. Use `embed_crop` for vector/mixed source charts and `describe` for a
text-only chart note. `d2_schematic` is allowed only for a qualitative relation
(cluster blobs, box-plot sketch): caption it “şematik” and do not assert exact
values. Exact ROC/scatter/quantile charts stay `unique` (crop/table/describe).
Legacy `table` is accepted as `redraw_table` for one release. `table_extract`
returns `reason`, `table_likely`, `manual_action` and a text preview when no
table block exists.

## OCR and insights

`source_insights` reports `ocr_evidence`: policy, pages OCRed/skipped, candidate
pages and reasons. The default is off; `ocr=true` or `--ocr` is an explicit
opt-in. `source_brief` reports the same policy and low-text candidates. OCR is
advisory by decision; see
[ADR-0014](adr/0014-ocr-advisory-policy.md). It never becomes a coverage gate
or an authoritative numeric source.

Run `make feedback-v2` for the external A-V2 matrix (A-V2-01…18),
`make feedback-v3` for visual/chart A-V3, and `make feedback-v4` for the final
contract A-V4 checks.

`qa` includes an `environment` block with version, build, PDFium, LiteParse, D2,
model signature, OCR/reranker policy and the topic seed. Two runs with the same
inputs should report the same content hash, coverage and card/claim counts.
