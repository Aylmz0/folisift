# Folisift — Context

## What this is

Folisift is a production pipeline for university exam study guides. It accepts
one or more source PDFs (slides, lecture notes, material) and produces a
Turkish, paper-like PDF guide with source references and an auditable coverage
report.

It is **not** tied to a single course, department or file format. The only
course-specific material is the guide bundle under `guides/<slug>/`.

## Ubiquitous language

| Term | Meaning |
|---|---|
| **Guide** | One course/material study guide, built from one or more sources. |
| **Guide bundle** | The self-contained directory `guides/<slug>/` with content, config, assets and generated data. |
| **Source PDF** | The original lecture material; a local input, never committed. |
| **Inventory** | Machine-readable map of a source: pages, headings, low-text pages, page map. |
| **Low-text page** | A page with little/no text layer; usually a figure, code screenshot or table. |
| **Coverage report** | Comparison of source sections against the guide content. |
| **Engine** | Domain-agnostic code under `engine/` (reference) and `crates/` (v2). |
| **Template** | Reusable A–G skeleton: `engine/templates/guide.html` (reference) or `templates/paper.typ` (v2). |
| **Design system** | Paper palette, typography and layout rules in `engine/style.css` and `templates/paper.typ`. |
| **Output** | A generated PDF under `output/`; not a source of truth. |

## Invariants

- Source PDFs stay local in `sources/`; `.gitignore` protects them.
- A guide is data + content, not a fork of the engine.
- The coverage gate is part of “done”; see `docs/agents/domain.md`.
- The guide remains understandable without reading the engine code.
- The design system stays stable across guides; deviations need an ADR.

## Layout

```
engine/                 Python reference pipeline
crates/                 Rust v2: core, parse, render, mcp, cli
templates/              paper.typ + embedded fonts
guides/<slug>/          one course/material guide (guide.md in v2)
sources/                local PDF inputs (gitignored)
output/                 generated PDFs
docs/                   architecture, authoring, design, ADRs, plans, agent config
```

## Current guides

| Slug | Sources | Output | State |
|---|---|---|---|
| `pd0-pd1` | PD0, PD1 | `output/PD0-PD1_Calisma_Ozeti.pdf` | Python reference, 22 pages |
| `pd0-pd1` (v2) | PD0, PD1 | `output/PD0-PD1_v2.pdf` | Rust/Typst port, 23 pages, coverage 13/13 |

## Non-goals

- A replacement for the course itself.
- A generic PDF editor or LMS.
- A committed archive of copyrighted lecture PDFs.
- A quiz/flashcard generator.
