# Folisift

Folisift is a domain-agnostic study-guide production pipeline. It turns lecture
PDFs into source-traceable, paper-like study guides with a deterministic build,
layout audit and local review tools. The engine is shared; each course is a
self-contained guide bundle.

**Current release: `v0.14.3`** — Linux archive, source builds for macOS and
Windows, and a source-available PolyForm Strict license.

- Repository: <https://github.com/Aylmz0/folisift>
- Documentation: [`docs/`](docs/)
- License: [PolyForm Strict 1.0.0](LICENSE)

## What it does

- Parses source PDFs once with LiteParse: pages, blocks, headings, tables,
  figures and bounding boxes.
- Builds page-aware sections and reviews them locally with Turkish BM25 plus a
  bundled multilingual static-embedding model.
- Writes a deterministic document graph: source → page → block → table cell /
  figure, plus heading and citation edges.
- Renders Markdown guides to tagged A4 PDFs with linked tables of contents,
  paper/reader/clean/revision/notes/workbook/reference layouts and 12 tested
  palettes.
- Enforces coverage, page budget, typography, contrast, reading order,
  bookmark, overflow and visual-regression gates. A reviewed coverage manifest
  makes the source-heading set authoritative for multi-source guides.
- Exposes the same build, source, block, link, graph, claim and card operations
  through a CLI and an MCP server for agent hosts such as OpenCode. Section
  briefs, source topics, guide plans, cost reports and bounded section reads
  keep source context small; `qa` returns one release report; MCP profiles keep
  the active tool schema small; and `::: glossary` renders dense four-column
  term tables.
- Runs offline after installation. The MCP server makes no LLM, network or
  shell calls.

Generated PDFs are reviewable artifacts. The host agent writes the guide prose;
Folisift parses, renders, audits and reports. It never invents facts to fill a
coverage gap.

## Install

### Linux release

```bash
curl -fsSL https://raw.githubusercontent.com/Aylmz0/folisift/main/scripts/install.sh | bash
folisift doctor
```

The installer resolves the latest release, verifies its SHA-256 and installs
under `~/.local/lib/folisift` with a link at `~/.local/bin/folisift`. If that
directory is not on `PATH`, add it first.

The archive contains:

```text
folisift              # CLI and MCP server
libpdfium.so          # PDF text and page rendering
d2                    # pinned v0.9.0, authoring only
models/               # local int8 Model2Vec model
LICENSE
THIRD_PARTY_NOTICES.md
styles/               # built-in style packs
domains/              # built-in domain profiles
templates/fonts/      # embedded OFL fonts
```

A normal install does not need Rust, Python, Node, Graphviz or a separate D2
installation. D2 is used only by `folisift diagram build`; guide builds embed
pre-generated SVG and never execute D2.

### Manual archive

```bash
tar -xzf folisift-v0.14.3-linux-x86_64.tar.gz
sha256sum -c folisift-v0.14.3-linux-x86_64.tar.gz.sha256
./folisift doctor
```

### macOS and Windows

The release binaries are Linux-only. macOS and Windows are source builds; CI
compiles and tests both platforms. See [install and platforms](docs/install.md)
for the current status.

## Quick start

Run from a workspace that contains `sources/`, `guides/` and `output/`:

```bash
folisift doctor
folisift analyze --source sources/source-a.pdf \
  --out guides/my-course/data/source.json \
  --md guides/my-course/data/source.md
folisift build --guide my-course --check
folisift audit --pdf output/my-course.pdf --guide my-course --strict
```

A guide bundle contains `guide.md`, `config.json`, optional diagrams and assets,
and generated data:

```text
workspace/
├── sources/          # local PDFs; never committed
├── guides/
│   └── my-course/
│       ├── guide.md
│       ├── config.json
│       ├── diagrams/
│       ├── assets/
│       └── data/     # inventories, blocks, reports
├── output/           # generated PDFs
└── styles/           # optional style overrides
```

## Quality gates

The normal review path is:

```bash
folisift coverage-manifest --guide my-course --action build
folisift build --guide my-course --check
folisift qa --guide my-course
folisift audit --pdf output/my-course.pdf --guide my-course --strict
folisift claims --guide my-course
folisift cards --guide my-course
```

`--check` fails on missing coverage, empty sections and page-budget violations.
`audit --strict` checks tagged PDF structure, bookmarks, minimum type size,
content-box and image overflow, underfilled pages and typography metrics.
Reading-order and component-contrast preflights are available as development
scripts and run in the full exam gate.

Selective formats such as revision sheets, knowledge organisers, notes,
workbooks and reference sheets declare a source scope in frontmatter:

```yaml
source_scope: "source-a:26-29"
# or, resolved from inventory headings:
source_scope: "source-a:section:3.4"
```

Every page in the resolved scope must be cited at least once.

## Semantic links and document graph

After layout blocks exist:

```bash
folisift links status --guide my-course
folisift links build --guide my-course
folisift links build --guide my-course --explore
folisift links read --guide my-course --section source-a:p26:s1
folisift graph build --guide my-course
folisift graph status --guide my-course
```

`links` proposes related sections with source/page/block provenance and writes
`data/document-links.json`. `graph` writes a deterministic
`data/document-graph.json` with structural and citation edges. Both are
review-only: they do not gate a build, and a similarity is not a verified fact.
The default link preset is precision-first (`0.75 / 0.55`, top-k 5);
`--explore` widens it to `0.65 / 0.45`, top-k 8.

The semantic model is bundled in the release archive. It runs locally on the
CPU, is hash-checked against its manifest and never downloads weights. A source
checkout without the model can set `FOLISIFT_MODEL_DIR` or use `--no-model` for
a BM25-only baseline.

## Use with OpenCode

```bash
cd /path/to/your/workspace
opencode mcp add folisift -- folisift mcp
opencode mcp list
```

Or declare it directly:

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "servers": {
      "folisift": {
        "type": "local",
        "command": ["folisift", "mcp"],
        "cwd": "/absolute/path/to/your/workspace"
      }
    }
  }
}
```

Ask the host to run `doctor`, prepare the guide, read the sources, write the
guide, build with the coverage gate and audit the PDF. The MCP exposes 31 tools
at schema 10; profiles (`core`, `source`, `authoring`, `review`, `fast`, `deep`)
hide the rest before `tools/list`, and `core` is ten tools. The normal path is
`prepare_guide` → `section_brief` / `read_section` / `read_source` → host writes
`guide.md` → `build_guide` → `qa_guide` (`audit_guide` for the standalone
check). `coverage_manifest`, `guide_plan`, `source_topics`, `read_blocks`,
`links` and `graph` provide bounded review data. The MCP server stays inside the
workspace root and never calls a network, LLM or shell.

## Common commands

```bash
folisift doctor
folisift workspace-info
folisift guide-status my-course
folisift coverage-manifest --guide my-course --action build
folisift build --guide my-course --check
folisift qa --guide my-course
folisift audit --pdf output/my-course.pdf --guide my-course --strict
folisift claims --guide my-course
folisift cards --guide my-course --review-intervals 1,3,7
folisift read-source --source sources/source-a.pdf --pages 1-20
folisift search-source --source sources/source-a.pdf --query topic
folisift read-blocks --guide my-course --source source-a --pages 26-29
folisift section-brief --guide my-course --source source-a --heading "3.1"
folisift read-section --guide my-course --source source-a --heading "3.1"
folisift budget --guide my-course
folisift styles estimate --guide my-course
folisift graph build --guide my-course
folisift links build --guide my-course
folisift diagram build --guide my-course --id pipeline
folisift mcp --root /path/to/workspace
```

## Develop from source

Requirements: Rust 1.92+, `libpdfium.so` in the binary directory or
`FOLISIFT_PDFIUM_DIR`, and D2 v0.9.0 only for diagram builds.

```bash
cargo build --release
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
```

Release and audit commands:

```bash
make exam-check        # build, audit, TOC, reading order and hashes
make package-check     # Linux archive + clean install smoke test
make dep-audit         # RustSec audit
make style-audit       # every style preview, minimum type and audit
make d2-pdf-check      # diagram PDF baseline
```

## Architecture

```text
crates/       Rust workspace: core, parse, render, build, mcp, cli, links
templates/    Typst layouts and embedded OFL fonts
styles/       layout packs, palettes, font pairings, labels
domains/      domain profiles
guides/       guide bundles
sources/      local PDFs; gitignored
output/       generated PDFs
docs/         architecture, authoring, design, ADRs, plans
```

The main data path is:

```text
source PDF -> inventory + layout blocks -> host-authored Markdown
           -> Typst render -> tagged PDF -> audit + coverage report
           -> links / document graph (review-only)
```

See [architecture](docs/architecture.md) for boundaries and data contracts.

## License

Folisift is source-available under the **PolyForm Strict License 1.0.0**.
Personal, educational, research and other noncommercial use is permitted.
The license does **not** grant distribution, modification, derivative works,
forks or maintenance rights. **Commercial use and any redistribution or
maintenance require prior written permission** from the copyright holder.
See [`LICENSE`](LICENSE).

This is not an OSI-approved open source license. Bundled PDFium, D2, fonts and
the embedding model keep their own licenses; see
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md).

## Documentation

- [Install and platforms](docs/install.md)
- [Cookbook and common errors](docs/cookbook.md)
- [Architecture](docs/architecture.md)
- [Authoring](docs/authoring.md)
- [Authoring specification](docs/authoring-spec.md)
- [Design system](docs/design-system.md)
- [Evidence base](docs/authoring-evidence.md)
- [Structure packs](docs/structures.md)
- [Diagram engine decision](docs/diagram-engine-evaluation.md)
- [Accessibility review](docs/accessibility-review-2026-11-03.md)
- [OpenCode production loop](docs/agents/opencode-guide.md)
- [Production feedback remediation](docs/plans/2026-11-03-production-feedback-remediation.md)
- [Versioning and releases](docs/agents/versioning.md)
- [Release history](docs/VERSIONS.md)
