# Authoring a guide

This document describes the day-to-day workflow for producing a Folisift
guide. The normative prompt/spec is `docs/authoring-spec.md`; this file is the
practical runbook.

## Prerequisites

- Python 3.10+
- Poppler utilities: `pdfinfo`, `pdftotext`, `pdftoppm`
- Pillow
- WeasyPrint

```bash
python3 -m pip install --user weasyprint pillow
```

## Guide bundle anatomy

```
guides/<slug>/
├── guide.html       # A–G content; source order preserved inside B
├── config.json      # paths relative to this directory
├── assets/          # guide-specific SVG diagrams
└── data/
    ├── source.json  # generated inventory (do not hand-edit)
    ├── source.md    # human-readable inventory
    └── coverage.md  # generated coverage report
```

Source PDFs are **not** in the bundle. Place them in the repository-level
`sources/` directory; that directory is gitignored.

## Workflow

### 1. Put the sources in place

```bash
sources/PD2.pdf
```

Never commit them. `sources/README.md` explains the local convention.

### 2. Create the bundle

```bash
python3 engine/tools/scaffold.py \
  --slug pd2 \
  --title "Bölüm 2: Önemli Programlama Dillerinin Gelişimi" \
  --course "BMÜ325 Programlama Dilleri" \
  --sources sources/PD2.pdf
```

This creates `guides/pd2/` from `engine/templates/guide.html`, writes
`config.json`, and runs the source inventory.

### 3. Read the inventory

```bash
cd guides/pd2
python3 ../../engine/tools/analyze_sources.py \
  ../../sources/PD2.pdf \
  --out data/source.json --md data/source.md
```

Use `source.md` as the coverage checklist. Do not hand-count pages or headings.

### 4. Inspect low-text pages

```bash
python3 ../../engine/tools/prepare_pages.py ../../sources/PD2.pdf \
  --source-json data/source.json --filename PD2.pdf \
  --out data/pages/pd2 --cols 2 --rows 2
```

Open the contact sheets. Transcribe meaningful diagrams, tables and code.
Mark decorative images as decorative; do not reproduce them.

### 5. Write `guide.html`

Follow `docs/authoring-spec.md`:

- preserve source order in B;
- every section/subsection gets an explanation;
- use `[PD2, s. 57]` references;
- label added clarifications **Açıklayıcı not**;
- flag inconsistencies instead of silently fixing them;
- no quizzes, flashcards or generic study advice.

Build early and often:

```bash
cd ../..
python3 engine/build.py --guide guides/pd2
```

### 6. Finish C–G

- C: consolidated glossary and “sık karıştırılan ayrımlar”.
- D: methods as input → operation → output.
- E: formulas, rules, dates, numbers; inconsistencies.
- F: cross-source connections; label interpretation.
- G: coverage table, low-text pages, exclusions, final counts.

### 7. Run the coverage gate

```bash
python3 engine/build.py --guide guides/pd2 --check
```

`data/coverage.md` must say **GEÇTİ**. The gate checks top-level section
numbers and heading matches. Tables, figures, formulas and examples still need
the manual audit in G.

### 8. Review the PDF

Check:

- TOC page numbers and PDF bookmarks;
- running headers match page content;
- no orphan section openers;
- no table/code overflow;
- Turkish characters and hyphenation;
- figure captions and source refs;
- G counts match `data/source.json`.

## Current guide

```bash
python3 engine/build.py --guide guides/pd0-pd1 --check
```

Output: `output/PD0-PD1_Calisma_Ozeti.pdf` (22 pages).

## Common problems

| Problem | Fix |
|---|---|
| `pdfinfo`/`pdftotext` missing | Install Poppler. |
| `weasyprint` missing | `python3 -m pip install --user weasyprint`. |
| Coverage says `EKSİK` | Add the missing section to B or fix the heading; re-run. |
| TOC numbers wrong | Ensure the link target IDs exist and rebuild. |
| Image missing | Check the path is relative to `guide.html`; guide images live in `assets/`. |
| Source PDF accidentally staged | Remove from index; never commit `sources/*.pdf`. |

## Related docs

- `docs/authoring-spec.md` — normative content and design rules
- `docs/architecture.md` — engine/guide data flow
- `docs/design-system.md` — paper palette and components
- `CONTEXT.md` — domain vocabulary
