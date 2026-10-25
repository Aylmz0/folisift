# Guides

Each directory is one self-contained guide bundle. The engine never assumes a
course, a source filename or a slide count.

```text
guides/<slug>/
├── guide.md         # content, source order preserved in B
├── config.json      # paths relative to this directory
├── assets/          # guide-specific SVG diagrams
└── data/            # generated inventory + coverage report
```

## Current bundles

| Slug | Sources | Output |
|---|---|---|
| `pd0-pd1` | PD0, PD1 | `output/PD0-PD1_Calisma_Ozeti.pdf` |

## Create a bundle

```bash
cargo run -p folisift-cli -- scaffold \
  --slug <slug> \
  --title "<title>" \
  --course "<course>" \
  --source sources/<file>.pdf
```

Then follow `docs/authoring.md`.

## Rules

- `guide.md` is rendered by `templates/paper.typ`; do not add CSS.
- Guide-specific diagrams live in `assets/`.
- `data/source.json` and `data/coverage.md` are generated; do not hand-edit.
- `data/pages/` is a local visual cache and is gitignored.
- Source PDFs never live in a guide bundle; they stay in `sources/`.
