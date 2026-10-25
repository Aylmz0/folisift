# Architecture

Folisift is a Rust workspace that turns source PDFs into Turkish, paper-like,
exam-oriented study guides. The guide bundle is the unit of content; the engine
never knows a course name, a source filename or a slide count.

## Layers

```text
sources/                      local PDF inputs (gitignored)
    │
    ▼
folisift-parse                liteparse inventory, coverage, screenshots
    │
    ▼
folisift-render               Markdown + directives → Typst → PDF
    │
    ▼
guides/<slug>/                guide.md + config.json + assets/ + data/
    │
    ▼
folisift-cli / mcp            CLI and stdio MCP tools
```

| Layer | Path | Responsibility |
|---|---|---|
| Core | `crates/folisift-core/` | Config/schema, workspace sandbox, scaffold, version. |
| Parse | `crates/folisift-parse/` | liteparse adapter, inventory, coverage, screenshots. |
| Render | `crates/folisift-render/` | Markdown/directives, Typst world, PDF. |
| MCP | `crates/folisift-mcp/` | stdio tools, resources and prompt. |
| CLI | `crates/folisift-cli/` | `folisift` binary. |
| Template | `templates/paper.typ` | Shared paper design and embedded fonts. |
| Guide | `guides/<slug>/` | Content, config, assets, generated data. |
| Inputs | `sources/` | Original PDFs; local only. |
| Outputs | `output/` | Generated guides for review. |

## Build flow

1. `folisift build --guide <slug>` loads `guides/<slug>/config.json`.
2. Paths resolve relative to the guide directory and are checked against the
   workspace root.
3. `folisift-render` parses Markdown + YAML frontmatter + four directives.
4. The Typst template renders the PDF with embedded fonts.
5. `--check` compares the inventory with the Markdown headings and writes
   `data/coverage.md`.
6. The PDF is written to the configured output path.

The MCP server wraps the same operations. It performs no LLM call, no shell
execution and no network access. The host agent writes content; the server
analyzes, renders and audits.

## Guide config (schema 2)

```json
{
  "schema_version": 2,
  "slug": "pd0-pd1",
  "title": "Programlama Dilleri — Çalışma Kılavuzu",
  "language": "tr",
  "content": "guide.md",
  "output": "../../output/PD0-PD1_Calisma_Ozeti.pdf",
  "sources": ["../../sources/PD0.pdf", "../../sources/PD1.pdf"],
  "inventory": "data/source.json",
  "coverage": "data/coverage.md",
  "target_pages": "18-22"
}
```

Rules:

- `content`, `output`, `inventory` and `coverage` resolve relative to the guide
  directory.
- `sources` are read-only inputs and must stay under `sources/`.
- No absolute paths. A guide must work after the repository is moved.
- `schema_version` 1 is legacy and is no longer used by a live guide.

## Content format

Markdown + YAML frontmatter + `::: definition`, `::: critical`, `::: example`
and `::: note` directives. Source references use `[PD0, s. 3]`. Figures use
Markdown images; the Typst template owns numbering and captions are kept from
the source. Raw Typst/CSS in a guide is forbidden.

## Generated artifacts

| Artifact | Producer | Committed? |
|---|---|---|
| `guides/<slug>/data/source.json` | `folisift analyze` | yes; reviewable |
| `guides/<slug>/data/source.md` | same | yes; checklist |
| `guides/<slug>/data/coverage.md` | `folisift build --check` | yes; audit trail |
| `guides/<slug>/data/pages/` | `folisift render-pages` | no; visual cache |
| `output/*.pdf` | `folisift build` | yes, for review |
| `sources/*.pdf` | instructor/user | **no**; gitignored |

## Coverage model

The inventory is the mechanical checklist: sources, pages, headings and
low-text pages. The gate checks that every top-level numbered section has a
matching normalized heading in the guide. It is deliberately conservative: a
missing number is reported, never guessed.

The guide’s G section is the human audit: tables, figures, examples, low-text
pages, exclusions and inaccessible content. The automated gate cannot claim
completeness on its own.

## Testing

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo run -p folisift-cli -- build --guide pd0-pd1 --check
```

The Rust tests cover path safety, config validation, Markdown directives,
inventory/coverage logic and a minimal Typst render. The MCP server is probed
with a stdio client during release checks.

## Non-goals

Folisift is not a PDF library, an LMS, a quiz generator or a replacement for
the course. It is a repeatable authoring pipeline with an auditable output.
