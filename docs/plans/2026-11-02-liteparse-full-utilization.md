# LiteParse full-utilization plan — 2026-11-02

- **Status:** In progress; phases 0–2 implemented
- **Scope:** `folisift-parse`, source inventory, CLI/MCP source access
- **Non-goals:** renderer changes, semantic graph extraction, new runtime
  dependencies, OCR policy changes
- **Baseline:** `1ea1c6b`
- **Constraint:** the build stays deterministic; the MCP server still performs
  no LLM, shell or network calls.

## 0. Preconditions

- The repository is at a clean commit with no active changes in
  `crates/folisift-parse`.
- `sources/PD0.pdf`, `sources/PD2.pdf` and `sources/PD3.pdf` are present for
  the comparison runs. They stay local and untracked.
- A synthetic fixture PDF is added for CI; no course PDF is required by tests.
- The semantic helper is not started until this plan's phases 0–3 are stable.

## 1. Why this plan

Folisift uses LiteParse 2.17.0, but only its plain-text projection. The parser
can also emit classified layout blocks with page geometry, headings, tables,
figures and reading order. That is the missing structural layer for a
page-anchored document graph.

Current adapter behavior:

- `LiteparseParser` uses `OutputFormat::Text`.
- `extract_blocks` is not set.
- Headings come from `heading_of()` (first meaningful line heuristic), not from
  the layout classifier.
- Tables come from a separate Folisift pass (`table.rs`), not from LiteParse's
  table block and cell boxes.
- `source_cache` parses the PDF again for text, so analysis and reading do not
  share one parse result.

This plan changes the adapter to use the parser's structure without changing
the guide renderer or coverage contract.

## 2. Measured baseline

A throwaway probe parsed `sources/PD2.pdf` with:

```rust
LiteParseConfig {
    output_format: OutputFormat::Json,
    extract_blocks: true,
    ocr_enabled: false,
    quiet: true,
    ..Default::default()
}
```

Observed:

| Metric | Value |
|---|---:|
| Pages | 103 |
| Pages with blocks | 101 |
| Blocks | 747 |
| Heading blocks | 119 |
| Table blocks | 6 |
| Table cells | 69 |
| Serialized blocks | 148 KB |
| Parse time | 242 ms |

This is cheap enough to make blocks a first-class analysis output. The block
JSON is too large to put inside every `inventory.json`; it needs its own
artifact.

## 3. Target data model

Add a Folisift-owned layout model. Do **not** expose LiteParse types directly
in the inventory: a parser upgrade must not silently break the artifact schema.

```rust
pub struct LayoutSummary {
    pub schema_version: u32,
    pub pages_with_blocks: usize,
    pub blocks: usize,
    pub headings: usize,
    pub tables: usize,
    pub figures: usize,
    pub pages: Vec<PageLayoutSummary>,
}

pub struct PageLayoutSummary {
    pub page: usize,
    pub blocks: usize,
    pub headings: usize,
    pub tables: usize,
    pub figures: usize,
    pub content_bounds: Option<Rect>,
}
```

Full blocks live in a separate file:

```json
{
  "schema_version": 1,
  "source": "PD2",
  "page_count": 103,
  "pages": [
    {
      "page": 26,
      "page_label": null,
      "width": 960.0,
      "height": 540.0,
      "content_bounds": {"x": 40.0, "y": 20.0, "width": 880.0, "height": 500.0},
      "blocks": [
        {
          "id": "PD2:p26:b0",
          "kind": "heading",
          "level": 1,
          "text": "2.4 Fonksiyonel Programlama: LISP",
          "bbox": {"x": 48.0, "y": 36.0, "width": 620.0, "height": 28.0}
        },
        {
          "id": "PD2:p26:b1",
          "kind": "table",
          "bbox": {"x": 48.0, "y": 220.0, "width": 760.0, "height": 120.0},
          "header": [
            {"text": "Kavram", "bbox": {"x": 48.0, "y": 220.0, "width": 220.0, "height": 24.0}}
          ],
          "rows": [
            [
              {"text": "Atom", "bbox": {"x": 48.0, "y": 244.0, "width": 220.0, "height": 24.0}}
            ]
          ]
        }
      ]
    }
  ]
}
```

Block kinds to map:

```text
heading, paragraph, list_item, code, table, merged_table,
grid_fallback, rule, figure
```

Fields:

- `id`: stable within one parse: `<SOURCE_ID>:p<PAGE>:b<INDEX>`.
- `bbox`: page coordinates in the same top-left 72-DPI space as LiteParse.
- `text`: heading/paragraph/list item text.
- `level`: heading level or list nesting depth.
- `ordered`, `marker`: list metadata.
- `lines`, `lang`: code and grid fallback.
- `header`, `rows`, `header_rows`: table structure and cell boxes.
- `colspan`, `rowspan`: merged-table spans.
- `figure_id`, `figure_format`: figure references.
- `content_bounds`: visible page extent when available.

`PageRow` in `inventory.json` can gain compact counters:

```rust
pub struct PageRow {
    pub page: usize,
    pub first_line: String,
    pub chars: usize,
    pub low_text: bool,
    #[serde(default)]
    pub blocks: usize,
    #[serde(default)]
    pub headings: usize,
    #[serde(default)]
    pub tables: usize,
}
```

## 4. Storage strategy

Do **not** put full blocks in `inventory.json`. Tracked inventories are small
today (~21 KB for PD2); 148 KB per source would create noisy diffs and make
coverage reviews harder.

Use three layers:

| Artifact | Content | Tracking |
|---|---|---|
| `guides/<slug>/data/inventory.json` | existing inventory + `layout` summary | tracked, compact |
| `guides/<slug>/data/blocks/<source>.json` | full page/block/table/bbox data | generated; default ignored |
| `guides/<slug>/data/document-graph.json` | deterministic/semantic graph | separate plan; review before tracking |

Add `.gitignore` entries for generated blocks if the trial confirms they are
reproducible from the source PDFs. A guide that needs a reviewed block artifact
can opt in and track that one file explicitly.

## 5. Module seam and artifact ownership

The parser is the seam. It should return both products of one parse, not force
callers to parse twice:

```rust
pub struct AnalyzedSource {
    pub inventory: SourceInventory,
    pub layout: Option<SourceLayout>,
}

pub struct Analysis {
    pub inventory: Inventory,
    pub layouts: Vec<SourceLayout>,
}

pub struct SourceLayout {
    pub schema_version: u32,
    pub source: String,
    pub page_count: usize,
    pub pages: Vec<PageLayout>,
}

#[async_trait]
pub trait SourceParser: Send + Sync {
    async fn analyze(&self, path: &Path, low_text_threshold: usize)
        -> Result<AnalyzedSource>;
}
```

`SourceInventory` keeps only the compact `Option<LayoutSummary>`.
`SourceLayout` owns the full page/block data. The CLI/MCP write the artifact;
the parser does not choose workspace paths.

Compatibility adapters:

```rust
// Existing callers keep the inventory-only API.
pub async fn analyze_sources(paths: &[PathBuf], threshold: usize) -> Result<Inventory> {
    Ok(analyze_sources_with_layout(paths, threshold).await?.inventory)
}

pub async fn analyze_sources_with_layout(
    paths: &[PathBuf],
    threshold: usize,
) -> Result<Analysis>;
```

`Analysis { inventory, layouts }` is written by the CLI/MCP layer. Test
fixtures implement the same `SourceParser` seam.

This is a deep module: one parse, one interface, two outputs. The caller learns
one method and never handles the parser-specific block representation.

## 6. Phases

### Phase 0 — fixture and probe

- Add a small synthetic PDF fixture with a heading hierarchy, a table, a figure
  and a blank page.
- Add a parser test that asserts block kinds, heading levels, table cell boxes
  and `None` blocks on a blank page.
- Keep a probe/benchmark command or ignored test that reports block counts,
  serialized size and parse time.

**Done when:** the fixture test passes in CI without `sources/`, and the probe
reports block count, JSON size and parse time.

### Phase 1 — model and mapping

- Add `crates/folisift-parse/src/layout.rs`.
- Define Folisift-owned `LayoutSummary`, `PageLayout`, `LayoutBlock`,
  `LayoutCell`, `Rect` and `BlockKind`.
- Map `liteparse::layout::LayoutBlock` to the owned types.
- Add `layout: Option<LayoutSummary>` to `SourceInventory` with
  `#[serde(default)]`; old inventories still deserialize.
- Add block JSON writer/reader helpers.
- Add unit tests for mapping, JSON round-trip and stable block ids.

**Done when:** `cargo test -p folisift-parse` passes, old inventory fixtures
deserialize, and the new mapping has no LiteParse type in the public inventory
schema.

### Phase 2 — parse blocks

- Change `LiteparseParser::engine()` to `OutputFormat::Json` with
  `extract_blocks: true`.
- Collect blocks per page into `SourceLayout`.
- Return `AnalyzedSource { inventory, layout }`; the CLI/MCP writer writes the
  full block artifact.
- Compute `LayoutSummary` for the inventory.
- Derive headings from `heading` blocks first. Keep `heading_of()` as a
  fallback when a page has no blocks.
- Keep `canonical_sections()` from the TOC; do not replace the coverage set.
- Add optional `level` and `block_id` to `Heading` with serde defaults.
- Compare old and new headings on PD0, PD2 and PD3 before switching the
  primary source.

**Done when:** block JSON is written for PD0/PD2/PD3; block-derived headings
equal or improve the old set on those three PDFs; coverage reports do not
regress; `make exam-check` hashes are unchanged before any intentional content
change.

### Phase 3 — tables and figures

- Use LiteParse table blocks for a table/figure census and cell boxes.
- Keep the existing `table.rs` extractor until a parity test shows LiteParse is
  equal or better for ruled, borderless and merged-cell tables.
- Add a diagnostic report: tables found by each extractor, cells, missing
  tables, bbox coverage.
- Only then decide whether `table.rs` becomes a fallback or is retired.

**Done when:** a parity report covers ruled, borderless and merged-cell tables,
and there is one default path with a named fallback.

### Phase 4 — source access

- Add a bounded block view for hosts:
  - CLI: `read-source --blocks --pages 26-29` or a `source-blocks` command.
  - MCP: a `read_source` option or a dedicated `source_blocks` tool returning
    one page range, not the whole document.
- Keep the default `read_source` text-only; block JSON can be large.
- `prepare_guide` returns `layout` summary and the block artifact path.
- Add a `max_chars`/page-range budget so an agent cannot accidentally pull 148
  KB of blocks into one context window.

**Done when:** a page-bounded CLI/MCP call returns blocks with page ids and
`prepare_guide` returns the layout summary plus artifact path; no response
exceeds the configured budget.

### Phase 5 — default and cleanup

- Make block extraction the default for `analyze`, `prepare_guide` and
  `scaffold_guide`; add `--no-blocks` only for debugging.
- Regenerate tracked inventories; keep `layout` summaries compact.
- If heading parity holds, remove the primary `heading_of()` path but keep the
  fallback for blockless pages.
- Update `docs/authoring-spec.md`, `docs/architecture.md` and the inventory
  schema notes.

**Done when:** the default pipeline writes blocks, old inventories still load,
and the exam outputs are byte-identical to the pre-block baseline.

### Phase 6 — downstream document graph

Use the block artifact as the deterministic node layer for
`document-graph.json`:

```text
source -> page -> block -> table cell/figure
heading -> child blocks
page -> next page
term -> defined_on page
citation -> source page
```

Typed semantic edges are a separate decision. The selected similarity helper
is planned in
[`2026-11-03-semantic-link-helper.md`](2026-11-03-semantic-link-helper.md);
it is released with the product and exposes a `links` CLI/MCP surface, but it
uses local embedding inference only (no LLM, network or shell) and never gates
a guide build. See
[`document-graph-tooling-review-2026-11-02.md`](../document-graph-tooling-review-2026-11-02.md)
and the lightweight helper evaluation in
[`lightweight-semantic-linking-research-2026-11-03.md`](../lightweight-semantic-linking-research-2026-11-03.md).

## 7. Tests and gates

Unit:

- block kind mapping for heading, paragraph, list, table, merged table, code,
  figure, rule, fallback;
- table cell `bbox`, `colspan`, `rowspan`;
- blank/OCR-only page returns no blocks;
- stable block ids and JSON round-trip;
- old inventory without `layout` loads;
- heading derivation uses block levels and falls back without blocks.

Integration:

- analyze fixture + PD0/PD2/PD3; assert block counts and heading counts;
- inventory JSON remains under a size budget;
- guide builds produce the same PDF hashes when only block metadata is added;
- coverage, claims and page budgets unchanged;
- MCP `prepare_guide` and `read_source` responses stay bounded.

Performance budget:

- analysis time no more than 1.5× the text-only path on the same PDF;
- full block JSON under 1 MB per 100 pages;
- inventory summary under 30 KB per source;
- no duplicate full parse in the normal `prepare_guide` path.

Commands:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
make exam-check
make parity
make visual
make package-check
```

## 8. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Block classifier mislabels headings | Keep TOC/canonical sections; compare old/new heading sets before switching |
| Coverage regresses because headings move | Do not change coverage until heading parity tests pass |
| Merged tables lose cells | Keep `table.rs` until table parity; test merged cells explicitly |
| Inventory files become huge | Store blocks separately; keep only summary counters in inventory |
| Stale block artifacts | Include source digest and parser version; regenerate on digest change |
| Agent pulls too many blocks | Page-range and character budgets; default text-only `read_source` |
| Parser upgrade changes block shapes | Folisift-owned schema + mapping tests; pin dependency |
| Duplicate parse cost | Block extraction must not add a second parse. The existing `prepare_guide` text-cache parse remains a separate, tracked performance task; do not claim it is removed by this plan. |

## 8.1 Rollback and sequencing

- Phase 1 adds types and mapping only; `OutputFormat::Text` remains the
  default until parity tests pass.
- Phase 2 is gated by a heading parity report on PD0/PD2/PD3. If block headings
  regress, keep the old `heading_of()` path and leave block extraction opt-in.
- Phase 3 is gated by a table parity report. If LiteParse tables are worse,
  keep `table.rs` as the default and expose block tables as diagnostics only.
- Phase 5 flips the default only after all earlier gates pass; reverting the
  default is a one-line config change, not a data migration.
- The semantic helper starts only after phases 0–3, so it never re-parses PDFs.

## 9. Non-goals

- No semantic entity/relation extraction in this plan.
- No LLM, embeddings, graph database or network in the MCP server.
- No change to D2, Typst templates or PDF design.
- No OCR default change; scanned pages stay on the existing screenshot/visual
  audit path.
- No claim that block extraction improves learning outcomes.

## 10. Acceptance checklist

- [ ] Folisift-owned layout schema and mapping exist.
- [ ] LiteParse blocks are enabled with a measured size/time budget.
- [ ] Headings come from blocks where available, with a fallback.
- [ ] Tables have a parity report before the old path is retired.
- [ ] Inventory stays compact; full blocks are a separate generated artifact.
- [ ] Old inventories and guides still load.
- [ ] Exam PDF hashes are unchanged after metadata-only adoption.
- [ ] MCP/CLI block access is page-bounded.
- [ ] Docs describe the new artifact and its non-authoritative status.
- [ ] No new external runtime dependency.


## 11. Implementation log

- 2026-11-03: Phase 0–2 landed.
  - Added `crates/folisift-parse/src/layout.rs` with Folisift-owned
    `SourceLayout`, `PageLayout`, `LayoutBlock`, `LayoutCell`, `Rect`,
    `LayoutSummary` and JSON read/write helpers.
  - `SourceParser::analyze` now returns `AnalyzedSource` (inventory + layout);
    `analyze_sources` remains the inventory-only compatibility wrapper and
    `analyze_sources_with_layout` returns both.
  - `LiteparseParser` uses `OutputFormat::Json` with `extract_blocks: true`.
  - `inventory.headings` uses navigational heading blocks, normalized section
    numbers, and keeps the old first-line fallback.
  - CLI `analyze` and CLI/MCP scaffold/prepare write
    `data/blocks/<source>.json`; the path is gitignored.
  - Added `tests/fixtures/pdf/blocks.pdf` and a parser test for heading levels
    and table cells.
  - Parity probe on PD0/PD2/PD3: PD2 now recovers 22 numbered headings (old
    18) including missing 2.2/2.5/2.9/2.11–2.20; PD0/PD3 recover the numbered
    TOC sections without the old bogus first-line headings.
  - `make exam-check` still passes with the existing five PDF hashes.
- Remaining: Phase 3 table parity, Phase 4 bounded block reads, Phase 5
  default/cleanup, Phase 6 document graph.
