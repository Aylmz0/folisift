# Architecture

Folisift separates the reusable production engine from the course-specific
guide bundles. The split is the central architectural decision; see
`docs/adr/0001-separate-engine-and-guides.md`.

## Layers

```
sources/                    local PDF inputs (gitignored)
    │
    ▼
engine/tools/               inventory, page rendering, coverage, scaffold
    │
    ▼
guides/<slug>/              guide.html + config.json + assets/ + data/
    │
    ▼
engine/build.py             HTML → PDF (WeasyPrint)
    │
    ▼
output/                     generated PDFs
```

| Layer | Path | Responsibility |
|---|---|---|
| Engine | `engine/` | Build, templates, tools, CSS, brand assets. No course names. |
| Guide | `guides/<slug>/` | Content, config, guide-specific SVGs, generated data. |
| Inputs | `sources/` | Original PDFs; local only. |
| Outputs | `output/` | Generated guides for review. |
| Docs | `docs/` | Architecture, authoring, design, ADRs, agent config. |

## Build flow

1. `engine/build.py --guide guides/<slug>` reads `config.json`.
2. Paths are resolved **relative to the guide directory**.
3. The content HTML is loaded with its guide directory as base URL, so
   `assets/...` resolves inside the bundle.
4. The shared stylesheet is `engine/style.css`; it is linked from the content
   with a relative path.
5. WeasyPrint writes the configured output path.
6. With `--check`, the coverage gate runs before the PDF is written.

## Guide config

```json
{
  "slug": "pd0-pd1",
  "title": "…",
  "language": "tr",
  "content": "guide.html",
  "stylesheet": "../../engine/style.css",
  "output": "../../output/PD0-PD1_Calisma_Ozeti.pdf",
  "sources": ["../../sources/PD0.pdf", "../../sources/PD1.pdf"],
  "inventory": "data/source.json",
  "coverage": "data/coverage.md",
  "target_pages": "18-22"
}
```

Rules:

- `content`, `stylesheet`, `output`, `inventory`, `coverage` are resolved
  relative to the guide directory.
- `sources` is documentation and a scaffold input; the build itself does not
  read the PDFs.
- No absolute paths. A guide must work after the repository is moved.

## Generated artifacts

| Artifact | Producer | Committed? |
|---|---|---|
| `guides/<slug>/data/source.json` | `engine/tools/analyze_sources.py` | yes; reviewable inventory |
| `guides/<slug>/data/source.md` | same | yes; human-readable checklist |
| `guides/<slug>/data/coverage.md` | `engine/tools/check_coverage.py` | yes; audit trail |
| `guides/<slug>/data/pages/` | `engine/tools/prepare_pages.py` | no; visual cache |
| `output/*.pdf` | `engine/build.py` | yes, for review |
| `sources/*.pdf` | instructor/user | **no**; gitignored |

## Coverage model

The inventory is the mechanical checklist: sources, pages, headings and
low-text pages. The coverage tool checks that every top-level numbered section
has a matching heading/number in the guide. It is deliberately conservative:
a missing number is reported, not guessed.

The guide’s G section is the human audit: table/figure/example coverage,
low-text pages, exclusions and inaccessible content. The automated gate is not
allowed to claim completeness on its own.

## Extending the engine

Add course-independent behaviour to `engine/`. Add course-specific diagrams,
config and prose to `guides/<slug>/`. If a change would make the engine know a
course name, a source filename or a slide count, it belongs in the guide
config instead.

## Testing

There is no test suite yet. The minimum verification loop is:

```bash
python3 engine/build.py --guide guides/pd0-pd1 --check
pdfinfo output/PD0-PD1_Calisma_Ozeti.pdf
```

For engine changes, also scaffold a throwaway guide in `/tmp` and build it.

## Non-goals

Folisift is not a PDF library, an LMS, a quiz generator or a replacement for
the course. It is a repeatable authoring pipeline with an auditable output.
