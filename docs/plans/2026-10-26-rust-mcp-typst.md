# Folisift v2 — Rust MCP + Typst Migration Plan

- **Status:** Phase 0–4 complete; Phase 1 CI/licences pending; Phase 5 next
- **Date:** 2026-10-26
- **Owner:** Maintainer
- **Supersedes:** the Python-only engine described in `docs/architecture.md`
- **Related:** `AGENTS.md`, `CONTEXT.md`, `docs/adr/`

## 1. Context

Folisift currently works: a Python engine turns source PDFs into a 22-page
Turkish paper-like guide. The next product step is an **MCP server** usable from
OpenCode, Codex and other MCP clients. The server should do deterministic
"grunt work"; the host agent/LLM does reasoning, structure and summarisation.

The current Python stack is a poor fit for a single-binary MCP product:

- PDF text extraction depends on Poppler CLI tools.
- Page screenshots depend on Poppler.
- PDF rendering depends on WeasyPrint (Python + Pango).
- The tool must be installed and kept working across MCP clients.

The maintainer accepted:

- single static binary as a product goal;
- **liteparse** as the PDF parser;
- Rust for the final product;
- English project documentation;
- a TradeSeeker-style versioning/commit/release discipline.

## 2. Decision summary

| Decision | Choice | Status |
|---|---|---|
| Product surface | Local stdio MCP server + CLI in one binary | Proposed |
| Distribution | Two-file archive: `folisift` + `libpdfium.so` sidecar next to it (not static) | Locked |
| Language | Rust end-to-end | Proposed |
| PDF parser | `liteparse` (Rust, PDFium, OCR off by default) | Locked |
| PDF renderer | Typst embedded as a Rust library | Proposed |
| Content format | Markdown + YAML frontmatter + typed directives | Proposed |
| MCP tools | Granular deterministic tools; no LLM calls | Proposed |
| Filesystem | Workspace-root sandbox; sources read-only | Proposed |
| Migration | Strangle the Python engine; parity then remove | Proposed |
| Versioning | TradeSeeker three-layer model + guide `schema_version` | Proposed |
| Docs | English; guide output Turkish | Locked |

“Proposed” means this plan is the recommendation, not yet an implementation
contract. The remaining open questions are in §16.

Phase 0 (risk spike) is complete. The parser, renderer and Turkish glyphs work;
a literal static binary does not, because `liteparse` loads PDFium at runtime.
The maintainer selected the **two-file sidecar** variant: `folisift` and the
platform's PDFium shared library are installed next to each other. No runtime
extraction. See [`2026-10-26-phase0-report.md`](2026-10-26-phase0-report.md)
and [ADR-0005](../adr/0005-pdfium-sidecar.md).

## 3. Target architecture

```text
MCP client (OpenCode / Codex / Claude / ...)
        │  stdio (JSON-RPC)
        ▼
┌──────────────────────────────────────────────┐
│ folisift (single Rust binary)                │
│                                              │
│  folisift-mcp      MCP tools/resources       │
│  folisift-cli      human CLI                 │
│  folisift-core     config, schema, paths,    │
│                    coverage, version         │
│  folisift-parse    liteparse adapter         │
│  folisift-render   Markdown → Typst → PDF    │
└──────────────────────────────────────────────┘
        │                          │
        ▼                          ▼
  sources/*.pdf              output/*.pdf
  guides/<slug>/             data/*.json
```

The same binary exposes:

```bash
folisift mcp --root /path/to/workspace
folisift build --guide guides/pd0-pd1 --check
folisift analyze --source sources/PD0.pdf
folisift render-pages --source sources/PD0.pdf --pages 7,17
folisift scaffold --slug pd2 --title "..." --sources sources/PD2.pdf
```

`folisift mcp` is the only command an MCP client needs.

## 4. Rust workspace

Proposed layout (names can change before implementation):

```text
Cargo.toml                 # workspace
crates/
  folisift-core/           # config, schema, workspace/path safety, coverage, version
  folisift-parse/           # liteparse wrapper, inventory, page rendering
  folisift-render/          # Markdown parser, Typst template API, PDF output
  folisift-mcp/             # rmcp server, tools, resources, prompts
  folisift-cli/             # clap CLI + `folisift` binary
  folisift-testkit/         # fixtures, golden helpers
templates/
  paper.typ                 # shared design system
  fonts/                    # embedded OFL fonts
guides/
  pd0-pd1/
sources/                    # local PDFs, gitignored
output/                     # generated PDFs
```

Rules:

- `folisift-core` must not depend on MCP or CLI.
- `folisift-parse` owns all PDFium/liteparse knowledge.
- `folisift-render` owns all Typst knowledge.
- `folisift-mcp` only validates inputs and calls core services.
- The binary must not shell out to Python or external converters.

## 5. MCP contract

### 5.1 Tools

All tools are deterministic, idempotent where possible, and return
**paths + structured summaries**, never PDF bytes or base64 images.

| Tool | Input | Output |
|---|---|---|
| `workspace_info` | — | root, version, guides, schema version |
| `list_guides` | — | guides with status |
| `guide_status` | `slug` | config, sections, source count, last build, coverage |
| `analyze_source` | `source` path | inventory path, page count, low-text pages, headings |
| `render_pages` | `source`, pages | PNG/contact-sheet paths, page numbers |
| `scaffold_guide` | `slug`, `title`, `sources` | created bundle paths |
| `build_guide` | `slug`, `check` | PDF path, page count, warnings |
| `check_coverage` | `slug` | `PASS`/`FAIL`, report path, missing sections |

### 5.2 Resources

- `folisift://authoring-spec` — content rules.
- `folisift://design-system` — palette/layout rules.
- `folisift://guide/{slug}` — guide metadata and section map.

### 5.3 Prompts

- `write-study-guide` — a reusable prompt that tells the host agent how to
  analyse a source, fill A–G, and call the tools. This is a prompt, not an LLM
  call inside the server.

### 5.4 Explicit non-goals

- No `generate_guide` opaque tool.
- No LLM/API calls from the server.
- No arbitrary shell execution.
- No network access by default.
- No PDF byte streaming through MCP.

## 6. Guide data model

`guides/<slug>/config.json`:

```json
{
  "schema_version": 2,
  "slug": "pd0-pd1",
  "title": "Programlama Dilleri — Çalışma Kılavuzu",
  "language": "tr",
  "content": "guide.md",
  "output": "../../output/PD0-PD1_Calisma_Ozeti.pdf",
  "sources": ["../../sources/PD0.pdf", "../../sources/PD1.pdf"],
  "target_pages": "18-22",
  "theme": "paper"
}
```

Guide bundle:

```text
guide.md                    # content, Markdown + directives
config.json
assets/                     # guide-specific SVG diagrams
data/
  inventory.json            # generated by analyze
  inventory.md
  coverage.json
  coverage.md
```

Migration note: existing `guide.html`, `data/source.json`, `style.css` are
reference inputs, not the final format. The HTML is ported once; CSS becomes a
Typst template.

Schema note: `pd0-pd1` is now schema 2 (`content: guide.md`). Its
`legacy_content: guide.html` field keeps the Python reference build and its
HTML coverage checker working until Phase 6. Schema 1 remains accepted for
other bundles during the strangle.

## 7. Content format

Use **Markdown + YAML frontmatter + a small typed directive set**. The server
compiles it to Typst; the agent never writes Typst or CSS.

```markdown
---
title: "Bölüm 0 — Hazırlık"
sources: [PD0]
---

## 0.1 Giriş ve amaçlar

Church–Turing hipotezi... [PD0, s. 3]

:::definition
**Programlama dili:** ...
:::

:::critical
**Sözdizimi / anlambilim:** ...
:::

:::example
...
:::

:::note
Açıklayıcı not...
:::
```

Rules:

- Directives: `definition`, `critical`, `example`, `note`.
- Tables use Markdown tables; figures use `![caption](assets/x.svg)`.
- Source refs use `[PD0, s. 3]`; the parser validates the source id.
- Unknown directive or malformed frontmatter → line-numbered error.
- No raw HTML or CSS in guide content.
- The Typst template owns headings, spacing, colours, TOC, headers/footers.
- The coverage gate reads `## <number> <title>` headings from Markdown and
  applies the same normalization/threshold as the current HTML checker. The
  HTML parser is removed only after Markdown coverage reaches parity.

Why Markdown over JSON/Typst:

- agents write prose naturally;
- diffs are readable;
- the design stays in the server;
- a small parser is cheaper than making agents write an AST.

## 8. Renderer: Typst

Port the current paper design to `templates/paper.typ`:

- A4, margins, cream paper, ink/brick/sage/gold palette.
- Noto Serif body, Inter headings, and a permissively licensed mono for code.
  DejaVu Sans Mono is acceptable only if its Bitstream Vera licence is shipped;
  an OFL mono is simpler. Typst's embedded fonts must be licensed explicitly.
- Cover, front matter, TOC, running headers, page numbers, bookmarks.
- Definition, critical, example, note boxes.
- Tables without vertical rules; figure captions; code blocks.
- Source refs and figure numbering.
- PDF metadata: title, author, language `tr`.

Implementation:

- Use the `typst` crate as a library; do not require a `typst` CLI at runtime.
- Embed fonts and the template in the binary (OFL licences included).
- Generate Typst from the Markdown AST, then compile to PDF.
- Keep the design in one template; guides cannot inject raw Typst.

Parity gate:

- same section coverage and visible design;
- page count within ±10% of the current 22-page pilot;
- no missing Turkish glyphs;
- TOC/bookmarks correct;
- tables/code/figures not clipped.

## 9. Parser: liteparse

Use `liteparse` as the only PDF parser.

- Rust crate, PDFium, OCR off by default.
- Page-level text + bounding boxes + headings/tables where available.
- Page screenshots for low-text slides.
- `is-complex`/low-text detection to decide when to render pages.
- Turkish text must be validated on PD0/PD1.

`folisift-parse` exposes a `SourceParser` trait so tests can use fixtures; the
production implementation is liteparse only. Do not add a second parser.

Inventory output must retain:

- file name, page count, word count;
- page-level first line and char count;
- heading-like lines with page numbers;
- low-text pages;
- source identity used by coverage.

Parity check: compare `/PD0.pdf` + `/PD1.pdf` inventory against the current
`guides/pd0-pd1/data/source.json`; differences must be explained.

Phase 0 findings that constrain this crate:

- `liteparse` uses PDFium through `libloading`. Native builds do **not** link
  PDFium statically; the shared library is downloaded at build time and loaded
  at runtime. The parser adapter must let the CLI/MCP layer choose the library
  directory before the first parse.
- `default-features = false` disables Tesseract. The OCR path must be opt-in,
  not a default dependency.
- Text parity was 5,386 vs 5,334 words on PD0 and 6,852 vs 6,856 on PD1.
  The existing low-text rule (<120 non-whitespace characters) matched exactly:
  40/40 and 15/15 pages.
- `is_complex` is a broader signal (75/123 and 30/137 pages); use it for
  routing and OCR decisions, not as the inventory's `low_text_pages` column.

## 10. Security model

- `--root` (or `FOLISIFT_ROOT`) defines the workspace; default is CWD only if
  explicitly allowed.
- All paths are canonicalised and checked to remain under root.
- `sources/` is read-only.
- Writes are allowed only under `guides/`, `output/`, and guide `data/`.
- Symlinks that escape root are rejected.
- No shell commands, no network, no LLM calls.
- Errors never echo secrets or arbitrary file contents.

## 11. Versioning and release

Adopt the TradeSeeker three-layer model, adapted:

| Layer | Source | Owner |
|---|---|---|
| Build | `git rev-list --count HEAD`, embedded at compile time | nobody (derived) |
| SemVer | annotated tag `vX.Y.Z` | agent creates and pushes tags |
| Commit | Conventional Commits | author |

Additional:

- `schema_version` in `config.json` is independent of SemVer.
- `mcp_tool_schema_version` may be exposed in `workspace_info`; tool contract
  changes require at least a MINOR bump.
- Build from a shallow clone must fail in CI.
- `docs/VERSIONS.md` records tag, date, build, commit, what changed, what
  invalidates prior measurements.
- Agent may commit and tag; **branch pushes remain human-owned**.
- A distributed binary must embed build/semver identity; do not read `.git` at
  runtime.

## 12. Distribution

Primary:

- GitHub Releases: Linux x86_64 (glibc first; musl later), macOS arm64/x86_64,
  Windows x86_64.
- Each archive contains `folisift` and the platform PDFium shared library
  (`libpdfium.so`, `libpdfium.dylib` or `pdfium.dll`) in the same directory.
- The loader resolves PDFium next to the executable by default. A missing or
  mismatched library is a clear startup error, not a panic.
- MCP config:

```json
{
  "mcpServers": {
    "folisift": {
      "command": "folisift",
      "args": ["mcp", "--root", "/path/to/workspace"]
    }
  }
}
```

Secondary:

- `cargo install folisift` for developers.
- No `curl | sh` installer until the binary/signing story is settled.

The release checklist must verify the binary runs without Python, Poppler,
WeasyPrint, or a system Typst installation.

## 13. Migration phases

### Phase 0 — Risk spike (go/no-go) — complete

- Built a throwaway Rust binary with liteparse + Typst.
- Rendered the paper template with Turkish glyphs, TOC, bookmarks and metadata.
- Measured liteparse inventory, low-text parity and screenshots on PD0/PD1.
- **Gate result: GO with a packaging change.** A static binary is not feasible
  with liteparse's native loading model. The selected distribution is a
  two-file archive: `folisift` + `libpdfium.so` next to it.
- Details: [`2026-10-26-phase0-report.md`](2026-10-26-phase0-report.md).

### Phase 1 — Workspace skeleton — in progress

- Cargo workspace with `folisift-core` and `folisift-cli`.
- `folisift-core`: config/schema, workspace path sandbox, version/build info.
- `folisift-cli`: `workspace-info`, `list-guides`, `guide-status`.
- `folisift-testkit`, parser/render/MCP crates follow in Phases 2–4.
- **Gate:** `cargo test` and `cargo clippy -D warnings` pass. Both pass as of
  this phase; CI and licence headers are still pending.

### Phase 2 — Parser parity — complete

- `folisift-parse`: liteparse adapter, `SourceParser` trait, inventory
  JSON/Markdown, low-text detection, page screenshots.
- `folisift analyze` and `folisift render-pages` CLI commands.
- PDFium sidecar loading with a clear error when the library is missing.
- Parity against the current inventory: pages, chars, low-text pages, headings
  and titles are exact. Words differ by +52/-4; 13/6 first-line strings differ
  only in projected bullets, whitespace or curly/straight apostrophes.
- **Gate:** pass with explained formatting differences; Turkish text intact.

### Phase 3 — Renderer and content — complete

- `folisift-render`: Markdown + frontmatter + four directives, Typst world with
  embedded fonts and sandboxed asset loading.
- `templates/paper.typ`: paper palette, cover, TOC, running headers, page
  numbers, boxes, tables, figures, code.
- `guides/pd0-pd1/guide.md`: PD0–PD1 content ported from `guide.html`.
- `folisift build` and Rust Markdown coverage gate.
- Config is schema 2 (`content: guide.md`); `legacy_content: guide.html` keeps
  the Python reference build working during the strangle.
- **Gate:** 23 pages vs 22 (+4.5%, inside ±10%), coverage 13/13 PASS, TOC and
  PDF bookmarks present, Turkish glyphs intact, no clipped tables/figures.
- Details: [`2026-10-26-phase3-report.md`](2026-10-26-phase3-report.md).

### Phase 4 — MCP server — complete

- `folisift-mcp`: `rmcp` stdio server with 8 tools:
  `workspace_info`, `list_guides`, `guide_status`, `analyze_source`,
  `render_pages`, `scaffold_guide`, `build_guide`, `check_coverage`.
- Resources: `folisift://authoring-spec`, `folisift://design-system`,
  `folisift://guide/{slug}`.
- Prompt: `write-study-guide`.
- CLI `folisift mcp`; host agent can analyze → scaffold → build → check.
- **Gate:** MCP handshake, `tools/list`, `tools/call`, resources, prompts and
  the full analyze → scaffold → build → check flow all pass.
- Details: [`2026-10-26-phase4-report.md`](2026-10-26-phase4-report.md).

### Phase 5 — Docs and agent contract

- Translate all docs to English.
- Rewrite `AGENTS.md` in the TradeSeeker working-agreement style.
- Add ADRs for Rust/Typst/liteparse/sandbox.
- Add versioning authority and `docs/VERSIONS.md`.
- **Gate:** docs links valid, AGENTS commands run.

### Phase 6 — Parity and removal

- Full PD0–PD1 visual/content comparison.
- Remove Python engine and WeasyPrint after parity.
- Release v0.1.0 of the single binary.
- **Gate:** no Python runtime needed; output accepted.

## 14. Testing and verification

- Unit: config/schema, path sandbox, Markdown directives, coverage matching.
- Golden: inventory JSON for PD0/PD1; coverage report; TOC/bookmark structure.
- Integration: MCP stdio handshake + each tool; CLI happy path.
- PDF: page count, text extraction of key terms, Turkish glyph check, visual
  diff of selected pages.
- Performance: record parse time, render time, peak RSS, binary size; no silent
  thresholds — write measured numbers in the release note.
- Cross-platform: Linux first; macOS/Windows before a public release.

## 15. Documentation and AGENTS.md

All project docs become English. The guide output remains Turkish.

`AGENTS.md` structure (TradeSeeker/HydroIndex-inspired):

1. The one-sentence constraint:
   *“Any course, any source, one engine; every guide is reproducible and
   auditable.”*
2. How to decide: evidence, simplicity, boring technology, coherence.
3. How to write: comments, docs, naming, errors.
4. Invariants: sources local, no invented facts, coverage gate, design system,
   no LLM in server, no Python runtime.
5. Working agreement: loop, validation, commit, release.
6. Agent skills block (issue tracker, triage labels, domain docs).
7. Where detail lives: `CONTEXT.md`, ADRs, `docs/agents/versioning.md`.

`docs/agents/versioning.md` is the authority; `AGENTS.md` only points to it.

## 16. Open questions

1. **Q6 approval:** accept Typst + Markdown + 7 tools + sandbox + strangle,
   with the two-file sidecar distribution?
2. **Packaging variant — resolved:** two-file archive (`folisift` +
   `libpdfium.so`); no runtime extraction. See ADR-0005.
3. **Platform priority:** Linux only for v0.1, or Linux/macOS/Windows at once?
   Linux glibc is validated; musl and the other platforms are not.
4. **OCR:** off by default; when should image-only slides enable it?
5. **Parity tolerance:** is ±10% page count acceptable, or must the 22-page
   layout be exact? Low-text inventory parity is already exact.
6. **MCP transport:** stdio only for v0.1; HTTP later?
7. **Release signing:** unsigned binaries initially, or sign/notarize?
8. **AGENTS length:** how much TradeSeeker-style philosophy is useful before it
   becomes noise for this smaller repo?

## 17. Success criteria

- The `folisift` binary plus its `libpdfium.so` sidecar start an MCP server on
  a clean machine with no Python, Poppler, WeasyPrint or Typst installation.
  There is no runtime extraction step.
- An MCP client can analyze a source, scaffold a guide, build a PDF and run the
  coverage gate.
- The PD0–PD1 guide preserves its sections, design language, Turkish text and
  source references.
- Coverage reports `PASS`; no claim of completeness without it.
- Docs are English; guide output is Turkish.
- Version identity is derived, tagged and recorded; no hand-typed version.

## 18. Non-goals

- Generating quiz/flashcard content.
- An LLM inside the MCP server.
- A general-purpose PDF editor.
- Supporting every document format in v0.1.
- Committing source PDFs or generated output beyond the agreed policy.
