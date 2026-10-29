# folisift

**Folisift** is a domain-agnostic production pipeline that turns lecture slides
into Turkish, paper-like, exam-oriented study guides.

First guide: **BMÜ325 Programlama Dilleri, PD0–PD1** — 22 pages, 260/260
slides. Latest release: **v0.6.1** — visual regression, structure packs, table/diagram contracts, hardened assets and HTTP auth.

## Layout

```text
folisift/
├── crates/          # Rust workspace
│   ├── folisift-core/   # config, schema, paths, scaffold, version
│   ├── folisift-parse/  # liteparse, inventory, coverage, screenshots
│   ├── folisift-render/ # Markdown/directives → Typst → PDF
│   ├── folisift-mcp/    # stdio MCP tools/resources/prompts
│   └── folisift-cli/    # folisift binary
├── templates/       # paper.typ, clean.typ and embedded OFL fonts
├── styles/          # versioned paper/clean style packs
├── guides/          # one bundle per course/material
│   └── pd0-pd1/
│       ├── guide.md
│       ├── config.json
│       ├── assets/
│       └── data/
├── docs/            # architecture, authoring, design, ADRs, plans
├── sources/         # PDFs; local only, gitignored
└── output/          # generated PDFs
```

## Quick start

```bash
cargo build --release
# put libpdfium.so next to target/release/folisift, or set FOLISIFT_PDFIUM_DIR
cargo run -p folisift-cli -- build --guide pd0-pd1 --check

# Read/search bounded source text
cargo run -p folisift-cli -- read-source --source sources/PD0.pdf --pages 1-20
cargo run -p folisift-cli -- search-source --source sources/PD0.pdf --query TIOBE

# Diagnose the environment and audit the rendered layout
cargo run -p folisift-cli -- doctor
cargo run -p folisift-cli -- audit --pdf output/PD0-PD1_Calisma_Ozeti.pdf --expected-pages 22

# Style packs
cargo run -p folisift-cli -- styles list
cargo run -p folisift-cli -- styles preview clean --language en

# Crop a source figure for a guide asset
cargo run -p folisift-cli -- render-pages --source sources/PD0.pdf --pages 7 \
  --out guides/pd0-pd1/assets/source --crop 0.12,0.08,0.76,0.40

# Draft table extraction from a native-text PDF page
cargo run -p folisift-cli -- table extract --source output/PD0-PD1_Calisma_Ozeti.pdf --page 4

# Pixel-level visual regression against a committed baseline
cargo run -p folisift-cli -- visual compare \
  --reference tests/baselines/pd0-pd1.pdf \
  --current output/PD0-PD1_Calisma_Ozeti.pdf \
  --pages 1,4,8 --out data/visual --fail-on-diff
```

What v0.6 adds:

- **Structure packs** — IMRAD, IRAC, SOAP, formula sheet, literature matrix,
  lecture notes and executive brief; missing required headings fail the build.
- **Visual contract** — deterministic accessible SVG diagrams, table captions
  and ids, figure provenance/alt text, no decorative stock images.
- **Verification** — pixel baselines, bookmark destination checks, content-box,
  orphan-heading and image-overflow audits, and a 22-page `paper` parity gate.
- **Safer surfaces** — SVG sanitizer, asset path confinement and bearer auth for
  non-loopback MCP HTTP.

MCP server (stdio or streamable HTTP):

```bash
folisift mcp --root /path/to/workspace
folisift mcp --root /path/to/workspace --http 127.0.0.1:8000
# HTTP endpoint: http://127.0.0.1:8000/mcp

# Non-loopback binds require a bearer token.
folisift mcp --root /path/to/workspace --http 0.0.0.0:8000 \
  --http-token "$FOLISIFT_HTTP_TOKEN"
# Requests must send: Authorization: Bearer <token>
```

Install and platform notes: `docs/install.md`.

## New guide

```bash
cargo run -p folisift-cli -- scaffold \
  --slug pd2 --title "Bölüm 2" --course "BMÜ325" \
  --style paper --structure a-g --language tr --domain cs \
  --source sources/PD2.pdf
```

The MCP `prepare_guide` tool is the default entry point: it analyzes the
sources, creates the same bundle and returns the read/write plan in one call.
`scaffold_guide` remains for granular recovery.

## Source PDF policy

`sources/*.pdf` are personal/course material and are **not tracked by git**.
Place them under `sources/` manually on a new machine. The repository can be
made public; evaluate the rights of the derivative PDFs under `output/`
separately.

## Requirements

- Rust 1.92+
- PDFium shared library: `libpdfium.so` (Linux), `libpdfium.dylib` (macOS) or
  `pdfium.dll` (Windows), installed next to the binary.
- No Python, Poppler, WeasyPrint or system Typst installation.

## Documentation

- [Architecture](docs/architecture.md)
- [Authoring](docs/authoring.md)
- [Authoring specification](docs/authoring-spec.md)
- [Structure packs](docs/structures.md)
- [Design system](docs/design-system.md)
- [ADRs](docs/adr/)
- [Plans](docs/plans/)
- [Versions](docs/VERSIONS.md)
- [Agent configuration](docs/agents/)
- [Agent operating rules](AGENTS.md)
- [Domain context](CONTEXT.md)
