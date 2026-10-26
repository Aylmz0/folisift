# AGENTS.md

Folisift is a domain-agnostic study-guide production pipeline. It turns lecture
PDFs into paper-like, exam-oriented Turkish guides. The engine is shared; each
course lives in its own guide bundle under `guides/`.

**One-sentence constraint:** any course, any source, one engine; every guide is
reproducible and auditable.

## How to decide

- Evidence over preference. Measure the output, the page count, the parse
  parity and the coverage report before calling a change an improvement.
- Boring technology, narrowly applied. Do not add a framework to avoid a small
  function.
- A guide bundle is data and content, never a fork of the engine.
- Documentation is English. The generated guide content is Turkish.
- Prefer one clear rule in `docs/` over repeated explanations in prompts.

## Commands

Run from the repository root unless stated otherwise.

```bash
# Build, test and inspect
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run -p folisift-cli -- workspace-info
cargo run -p folisift-cli -- guide-status pd0-pd1
cargo run -p folisift-cli -- analyze --source sources/PD0.pdf \
  --out guides/pd0-pd1/data/source.json --md guides/pd0-pd1/data/source.md
cargo run -p folisift-cli -- render-pages --source sources/PD0.pdf \
  --pages 7 --out guides/pd0-pd1/data/pages
cargo run -p folisift-cli -- read-source --source sources/PD0.pdf --pages 1-20
cargo run -p folisift-cli -- search-source --source sources/PD0.pdf --query TIOBE
cargo run -p folisift-cli -- build --guide pd0-pd1 --check
cargo run -p folisift-cli -- scaffold --slug pd2 --title "..." \
  --course "BMÜ325" --source sources/PD2.pdf
cargo run -p folisift-cli -- mcp
```

The Rust binary loads `libpdfium.so` (or the platform equivalent) from its own
directory. Set `FOLISIFT_PDFIUM_DIR` for a development copy. Do not add runtime
extraction.

## Architecture

- `crates/` — Rust workspace:
  - `folisift-core` — config/schema, workspace path safety, scaffold, version.
  - `folisift-parse` — liteparse adapter, inventory, coverage, screenshots.
  - `folisift-render` — Markdown/directives to Typst PDF.
  - `folisift-mcp` — stdio tools, resources and prompts.
  - `folisift-cli` — the `folisift` binary.
- `guides/<slug>/` — one self-contained course/material bundle:
  `guide.md`, `config.json`, `assets/`, `data/`.
- `sources/` — personal/course PDFs; **not tracked by git**.
- `output/` — generated PDFs, tracked for review.
- `docs/` — architecture, authoring, design system, ADRs, plans and agent config.

Guide config paths are relative to the guide directory. Do not hard-code a
course name, a page count or a source path in the engine.

## Invariants

- `sources/*.pdf` must never be committed. They are local inputs.
- Never claim full coverage without `folisift build --check` passing; the v0.2
  gate also rejects empty sections.
- Do not invent facts, formulas, dates or page references.
- Label added clarifications with **Açıklayıcı not** in the guide.
- The design system stays stable across guides; deviations need an ADR.
- One guide bundle must be usable without changing another bundle.
- No LLM call, shell execution or network access inside the MCP server.
- Writes stay under `guides/`, `output/` and `data/`; `sources/` is read-only.

## Working agreement

1. Read `CONTEXT.md`, the relevant ADRs and `docs/plans/` before changing
   architecture.
2. Make the smallest change that proves the point; add tests for behavior that
   can regress.
3. Run the relevant gate before reporting done. If a gate cannot run, say why.
4. Keep docs links valid and update the contract when behavior changes.
5. Commit with Conventional Commits. Do not push branches; the maintainer owns
   remote pushes and releases.
6. Record a decision in `docs/adr/` when it changes a boundary, not when it
   merely renames a local variable.

## Agent skills

### Issue tracker

Issues live in GitHub Issues (`Aylmz0/folisift`). See `docs/agents/issue-tracker.md`.

### Triage labels

The five canonical labels are used unchanged. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context repository: read `CONTEXT.md`, `docs/adr/`, `docs/architecture.md`
and the active plan before changing architecture. See `docs/agents/domain.md`.

### Versioning

`docs/agents/versioning.md` is the authority; `AGENTS.md` only points to it.
`docs/VERSIONS.md` records released tags and builds.

## Where detail lives

- Product boundaries and data flow: `docs/architecture.md`.
- Authoring workflow: `docs/authoring.md`.
- Normative content rules: `docs/authoring-spec.md`.
- Paper design: `docs/design-system.md` and `templates/paper.typ`.
- Migration decisions: `docs/plans/` and `docs/adr/`.
- Active work: GitHub Issues and `docs/plans/`.
