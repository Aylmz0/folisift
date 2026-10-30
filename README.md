# folisift

**Folisift** is a domain-agnostic production pipeline that turns lecture slides
into Turkish, paper-like, exam-oriented study guides.

First guide: **BMÜ325 Programlama Dilleri, PD0–PD1** — 22 pages, 260/260
slides. Latest release: **v0.9.0** — professional style packs, linked TOC, adaptive
page ranges and the evidence-based production loop.

## Install

Linux release (recommended):

```bash
curl -fsSL https://raw.githubusercontent.com/Aylmz0/folisift/main/scripts/install.sh | bash
# ~/.local/bin/folisift and ~/.local/lib/folisift/{libpdfium.so,d2,...}

folisift doctor
```

If `~/.local/bin` is not on `PATH`, add it first. The archive contains the
binary, PDFium, D2 v0.9.0, fonts and built-in style packs. You do not need Rust,
Python, Node or a separate D2/Graphviz installation for a normal user install.

Manual archive install:

```bash
tar -xzf folisift-v0.9.0-linux-x86_64.tar.gz
sha256sum -c folisift-v0.9.0-linux-x86_64.tar.gz.sha256
./folisift doctor
```

The release is currently Linux-only; macOS and Windows are source builds.
See [install](docs/install.md).

## Use with OpenCode

Run from the workspace that contains `sources/`, `guides/` and `output/`:

```bash
cd /path/to/your/workspace
opencode mcp add folisift -- folisift mcp
opencode mcp list
```

Or add the equivalent declarative config:

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
    },
  },
}
```

`cwd` is the workspace root. The MCP server has no access to paths outside
that workspace; keep `sources/` there. Ask OpenCode:

> Use Folisift. Run `doctor`, then `prepare_guide`. Read the sources, write the
> guide, build with the coverage gate, and audit the PDF.

The MCP exposes 13 tools. The normal path is `prepare_guide` → `read_source` /
`search_source` → host writes `guide.md` → `build_guide` → `audit`. Diagram
authoring is optional; see [the OpenCode runbook](docs/agents/opencode-guide.md).

## What a workspace looks like

```text
workspace/
├── sources/          # local PDFs; never committed
├── guides/
│   └── my-course/
│       ├── guide.md
│       ├── config.json
│       ├── diagrams/ # optional D2 specs
│       ├── assets/
│       └── data/
├── output/           # generated PDFs
└── styles/           # optional style overrides
```

## Common commands

```bash
folisift doctor
folisift workspace-info
folisift guide-status my-course
folisift build --guide my-course --check
folisift audit --pdf output/my-course.pdf
folisift mcp --root /path/to/workspace
```

For diagrams, the D2 binary is bundled with the release; generate assets with:

```bash
folisift diagram build --guide my-course --id pipeline
folisift diagram status --guide my-course
```

`folisift build` never runs D2; it embeds the generated SVG. A normal user
does not need to touch D2.

## Develop from source

```bash
cargo build --release
cargo test
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Put `libpdfium.so` next to the binary or set `FOLISIFT_PDFIUM_DIR`. Source
builds need D2 v0.9.0 only for `diagram build`; `scripts/fetch-d2.sh` stages the
pinned binary. The full command list and authoring rules are in
[docs/authoring.md](docs/authoring.md).

## Layout

```text
crates/          # Rust workspace: core, parse, render, mcp, cli
templates/       # paper.typ, clean.typ and embedded OFL fonts
styles/          # paper/reader/clean/journal/brief/accessible packs
guides/          # guide bundles
docs/            # architecture, authoring, design, ADRs, plans
sources/         # local PDFs; gitignored
output/          # generated PDFs
```

## Source PDF policy

`sources/*.pdf` are personal/course material and are **not tracked by git**.
Place them under `sources/` manually on a new machine. The repository can be
made public; evaluate the rights of the derivative PDFs under `output/`
separately.

## Documentation

- [Install and platforms](docs/install.md)
- [Architecture](docs/architecture.md)
- [Authoring](docs/authoring.md)
- [Authoring specification](docs/authoring-spec.md)
- [Evidence base](docs/authoring-evidence.md)
- [Structure packs](docs/structures.md)
- [Diagram engine decision](docs/diagram-engine-evaluation.md)
- [OpenCode production loop](docs/agents/opencode-guide.md)
