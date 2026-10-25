# folisift

**Folisift** is a domain-agnostic production pipeline that turns lecture slides
into Turkish, paper-like, exam-oriented study guides.

First guide: **BMÜ325 Programlama Dilleri, PD0–PD1** — 22 pages in the Python
reference, 23 pages in the Rust v2 port, 260/260 slides.

## Layout

```text
folisift/
├── engine/          # Python reference: build, tools, template, CSS
├── crates/          # Rust v2 workspace
│   ├── folisift-core/   # config, schema, paths, scaffold, version
│   ├── folisift-parse/  # liteparse, inventory, coverage, screenshots
│   ├── folisift-render/ # Markdown/directives → Typst → PDF
│   ├── folisift-mcp/    # stdio MCP tools/resources/prompts
│   └── folisift-cli/    # folisift binary
├── templates/       # paper.typ and embedded OFL fonts
├── guides/          # one bundle per course/material
│   └── pd0-pd1/
│       ├── guide.md     # v2 content
│       ├── guide.html   # Python reference content
│       ├── config.json
│       ├── assets/
│       └── data/
├── docs/            # architecture, authoring, design, ADRs, plans
├── sources/         # PDFs; local only, gitignored
└── output/          # generated PDFs
```

## Quick start

Python reference:

```bash
python3 engine/build.py --guide guides/pd0-pd1 --check
```

Rust v2:

```bash
cargo build --release
# put libpdfium.so next to target/release/folisift, or set FOLISIFT_PDFIUM_DIR
cargo run -p folisift-cli -- build --guide pd0-pd1 \
  --output output/PD0-PD1_v2.pdf --check
```

MCP server:

```bash
folisift mcp --root /path/to/workspace
```

## New guide

```bash
cargo run -p folisift-cli -- scaffold \
  --slug pd2 --title "Bölüm 2" --course "BMÜ325" \
  --source sources/PD2.pdf
```

The MCP `scaffold_guide` tool creates the same bundle and seeds headings from
the source inventory.

## Source PDF policy

`sources/*.pdf` are personal/course material and are **not tracked by git**.
Place them under `sources/` manually on a new machine. The repository can be
made public; evaluate the rights of the derivative PDFs under `output/`
separately.

## Requirements

Python reference:

```bash
python3 -m pip install --user -r requirements.txt
```

Poppler tools (`pdfinfo`, `pdftotext`, `pdftoppm`) must be installed.

Rust v2:

- Rust 1.92+
- PDFium shared library: `libpdfium.so` (Linux), `libpdfium.dylib` (macOS) or
  `pdfium.dll` (Windows), installed next to the binary.
- No Python, Poppler, WeasyPrint or system Typst installation.

## Documentation

- [Architecture](docs/architecture.md)
- [Authoring](docs/authoring.md)
- [Authoring specification](docs/authoring-spec.md)
- [Design system](docs/design-system.md)
- [ADRs](docs/adr/)
- [Plans](docs/plans/)
- [Versions](docs/VERSIONS.md)
- [Agent configuration](docs/agents/)
- [Agent operating rules](AGENTS.md)
- [Domain context](CONTEXT.md)
