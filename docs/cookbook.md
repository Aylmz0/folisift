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

For a multi-source or long guide, make the reviewed manifest authoritative:

```bash
folisift coverage-manifest --guide <slug> --action build
# review guides/<slug>/data/coverage-manifest.json
folisift coverage-manifest --guide <slug> --action validate
folisift build --guide <slug> --check
```

The build uses the manifest only when the file exists; otherwise the legacy
coverage gate remains the fallback.

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

## D2 diagrams

```bash
folisift diagram validate --guide <slug> --all
folisift diagram build --guide <slug> --id <id>
folisift diagram status --guide <slug>
```

`layout` accepts `tala`, `dagre` or `elk`; the chosen layout is recorded in the
manifest. Guide builds only embed pre-generated SVG and never execute D2.
