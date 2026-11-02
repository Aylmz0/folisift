# Semantic link helper with model2vec-rs — 2026-11-03

- **Status:** Implemented and piloted; threshold decision recorded
- **Decision:** use **model2vec-rs** for local semantic similarity
- **Scope:** `folisift-links` library, `folisift links` CLI, MCP `links` tool,
  release model asset
- **Depends on:** [LiteParse full-utilization plan](2026-11-02-liteparse-full-utilization.md)
- **Blocking gate:** do not start until `data/blocks/<source>.json` exists and
  is stable for the pilot corpus (LiteParse plan phases 0–3)
- **Release contract:** the model and helper ship in the normal archive; a
  single install gives the user the CLI and MCP surface
- **Runtime contract:** local CPU inference only; no LLM, network or shell in
  the MCP server

## 1. What this helper is

A small local index that proposes related sections across source PDFs:

```text
LiteParse blocks
  -> sections with source/page/block ids
  -> BM25 candidate pairs
  -> model2vec-rs embeddings
  -> cosine + lexical score
  -> document-links.json + document-links.md
```

The output is a review aid. It answers "read these two sections together",
not "A causes B" or "A is a prerequisite of B". Similarity is a proposal, not
a verified relation.

It is part of the installed product:

- `folisift links ...` in the CLI;
- `links` in the MCP server;
- the model asset in the release archive.

It is not part of the guide build gate. A normal `folisift build` does not
require the model or the link index.

## 2. Why model2vec-rs

Selected because it is the smallest credible local semantic layer:

- **MIT** (repository; crates.io metadata is missing a license field, so pin
  and record the repository licence).
- Rust implementation, no Python service.
- No ONNX Runtime; uses `safetensors` + `tokenizers`.
- `potion-multilingual-128M` supports 101 languages including Turkish.
- Static embeddings are fast on CPU; int8 quantization reduces the embedding
  memory footprint substantially.
- Works offline from a bundled or locally verified model directory.
- Small enough to include in the release without a Python/uv install step.

Trade-offs:

- Static embeddings are less accurate than full transformer embeddings.
- The crate is young (0.4.0, ~225 stars); pin it and run a quality pilot.
- The multilingual f32 checkpoint is ~512 MB; the tokenizer is ~18 MB. The
  release must ship an int8 model, not the f32 checkpoint.
- The model is a release asset, not a git-tracked file.

`fastembed-rs` is the fallback if model2vec quality fails: Apache-2.0, more
mature (~1.1k stars), more model choices, but it adds ONNX Runtime.

`txtai` and `iText2KG` are not selected for this helper. `txtai` is a Python
framework, and iText2KG is a Python/LLM graph-construction pipeline without a
page-provenance contract. They remain later options for typed graphs.

## 3. Module seam

Create a workspace library crate:

```text
crates/folisift-links/
  src/lib.rs
  src/model.rs
  src/section.rs
  src/link.rs
  src/store.rs
```

No separate `folisift-links` binary. `folisift-cli` and `folisift-mcp` both
depend on the library, so the CLI and MCP cannot drift.

The deep module is the link builder. It should be pure and testable without a
model download:

```rust
pub struct LinkOptions {
    pub top_k: usize,
    pub min_cosine: f32,
    pub min_score: f32,
    pub max_section_chars: usize,
}

pub trait Embedder {
    fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
}

pub struct LinkIndex;

impl LinkIndex {
    pub fn build(
        sections: &[Section],
        embedder: &dyn Embedder,
        options: &LinkOptions,
    ) -> Result<LinkReport>;
}
```

- `Section` is built from LiteParse blocks; it carries `source`, `page_start`,
  `page_end`, `block_ids`, `heading` and `text`.
- `Embedder` is the only model seam. Production uses model2vec-rs; tests use a
  deterministic fake.
- `LinkIndex::build` returns data. It does not read files, download models or
  write artifacts.
- The CLI/MCP layer reads `data/blocks/*.json`, calls the builder and writes the
  report.
- `Model2VecEmbedder` is loaded lazily and called from a blocking worker; the
  async MCP handler must not block the runtime on model inference.

Deletion test: disabling the `links` tool must not change any guide PDF, MCP
tool result for another action, or coverage result.

## 3.1 Governance and invariants

This changes the MCP surface, not the generation pipeline. Record an ADR before
implementation:

- model2vec is a **local static embedding model**, not a generative LLM;
- `links` may run local CPU inference, but it performs no network, shell or LLM
  call;
- the model is installed with the release and verified by `doctor`;
- `build_guide` and coverage/claims/page-budget gates do not depend on links;
- the `links` action can be disabled without rebuilding or changing guides.

The existing MCP policy remains: no outbound network, no shell execution, no
generative model call. The ADR must make the embedding exception explicit.

## 4. Inputs and outputs

Input:

```text
guides/<slug>/data/inventory.json
guides/<slug>/data/blocks/<source>.json
```

The blocks artifact comes from the LiteParse plan. The helper does not parse
PDFs again.

Output:

```text
guides/<slug>/data/document-links.json
guides/<slug>/data/document-links.md
```

JSON shape:

```json
{
  "schema_version": 1,
  "guide": "pd2",
  "model": "minishlab/potion-multilingual-128M",
  "model_sha256": "...",
  "model2vec_rs_version": "0.4.0",
  "options": {
    "top_k": 5,
    "min_cosine": 0.75,
    "min_score": 0.55
  },
  "sections": [
    {
      "id": "PD2:p26:s1",
      "source": "PD2",
      "page_start": 26,
      "page_end": 27,
      "block_ids": ["PD2:p26:b0", "PD2:p26:b1"],
      "heading": "LISP",
      "chars": 820
    }
  ],
  "links": [
    {
      "from": "PD2:p26:s1",
      "to": "PD3:p12:s2",
      "relation": "semantically_similar_to",
      "direction": "bidirectional",
      "confidence": "SIMILARITY",
      "cosine": 0.83,
      "lexical": 0.61,
      "score": 0.74,
      "from_source": "PD2",
      "from_page": 26,
      "to_source": "PD3",
      "to_page": 12
    }
  ]
}
```

Every link carries source/page provenance. The Markdown report is a review
surface:

```markdown
# Related sections

| Section | Related section | Score | Pages |
|---|---|---:|---|
| LISP (PD2 p26) | Symbolic AI (PD3 p12) | 0.74 | PD2 s.26-27 → PD3 s.12 |
```

## 5. Algorithm

### 5.1 Sectioning

- A `heading` block starts a new section; its `level` controls nesting.
- Paragraph, list, table and figure blocks attach to the current section.
- A section spans at most ~400 tokens / `max_section_chars` (default 1200);
  split at block boundaries, not inside a block.
- If a page has no heading, use a page-level section.
- Tables contribute header + rows; figures contribute their caption/alt text
  when available, otherwise only a reference marker.
- Repeated headers/footers are suppressed by the LiteParse block classifier
  where possible.

### 5.2 Lexical candidates

Use the `bm25` crate:

- Index section text (heading + body).
- For each section, take top 20 candidates.
- Union the candidate pairs in both directions.
- Keep exact terms, numbers and Turkish proper nouns available even when the
  embedding model is weak.

### 5.3 Embeddings

- Encode each section as `heading + "\n" + body`.
- Normalize vectors.
- `model2vec-rs` loads the bundled int8 model.
- Compute cosine similarity for candidate pairs.
- Use brute force; ~750 sections is trivial. No vector database.

### 5.4 Score and edges

Initial formula:

```text
score = 0.40 * lexical_normalized + 0.60 * cosine
```

Rules:

- require `cosine >= min_cosine` **and** `score >= min_score`;
- cap each section to `top_k` outgoing links;
- keep an edge if either direction passes; mark `direction: "bidirectional"`
  when both pass;
- break ties by section id for deterministic output;
- optionally add a small cross-source bonus after the pilot, not before.

The thresholds are initial values. Phase 0 must tune them on the three-PDF
corpus. Do not hard-code a "magic" threshold without evidence.

## 6. Model packaging and release

The release archive must be single-install:

```text
folisift-v<version>-linux-<arch>/
  folisift
  libpdfium.so
  d2
  models/
    potion-multilingual-128M-int8/
      config.json
      tokenizer.json
      model.safetensors
      manifest.json
      LICENSE
  styles/
  domains/
  templates/fonts/
```

Rules:

- Package an **int8** model, not the 512 MB f32 checkpoint. The maintainer
  preparation step uses the Python `model2vec` tool with `quantize_to="int8"`;
  runtime uses `model2vec-rs`.
- The model is not committed. `scripts/package.sh` requires a prepared model
  directory and fails without it, like D2 and PDFium.
- `scripts/install.sh` already extracts the whole archive; no Python/uv step
  on the user machine.
- Resolve the model from, in order:
  1. `FOLISIFT_MODEL_DIR`;
  2. `<binary-dir>/models/potion-multilingual-128M-int8`;
  3. `<workspace>/data/models/potion-multilingual-128M-int8` for development.
- `folisift doctor` checks model presence, file hashes and the model2vec-rs
  load path.
- The MCP server never downloads a model. If files are missing, `links`
  returns an actionable error naming the install path.
- Add the model licence to `THIRD_PARTY_NOTICES.md` and the archive.

## 7. CLI and MCP surface

CLI:

```bash
# Build links for one guide
folisift links build --guide pd2 --top-k 5 --min-cosine 0.75

# BM25-only mode; no model required
folisift links build --guide pd2 --no-model

# Review / inspect
folisift links read --guide pd2 --section PD2:p26:s1
folisift links status --guide pd2
```

MCP tool `links`:

```json
{
  "action": "status | build | read",
  "guide": "pd2",
  "section": "PD2:p26:s1",
  "top_k": 5,
  "min_cosine": 0.75,
  "force": false
}
```

- `status`: model found, blocks found, links current/stale, counts.
- `build`: builds/refreshes JSON and Markdown; returns counts and paths.
- `read`: returns a bounded set of links; never the whole graph by default.
- `build` runs model inference in `spawn_blocking`, not on the async runtime.
- Tool annotations: `open_world_hint = false`; `build` is not read-only.
- `prepare_guide` returns the links status and artifact path; `next` includes
  "run links build when blocks are ready".
- `build_guide` does not auto-run links and does not fail without a model.
- Bump `MCP_TOOL_SCHEMA_VERSION` to **6**; tool count becomes **16**.
- Update MCP stdio tests to call `links status` and assert model/blocks state.

## 8. Phases

### Phase 0 — model and quality spike

- Prepare an int8 model directory from `potion-multilingual-128M`.
- Verify `model2vec-rs` loads it offline.
- Encode 20 Turkish sections; measure load time, encode time, memory and size.
- Hand-check 20 related pairs; record useful / false / missed.
- Compare with a BM25-only baseline.

**Done when:** the pilot has a measured quality and latency table, and the team
decides whether static embeddings are good enough.

### Phase 1 — sections from blocks

- Implement `Section` construction from LiteParse page/block JSON.
- Preserve source, page range and block ids.
- Add tests for heading hierarchy, page-only sections, table text and long
  section splitting.

**Done when:** every section has a page range and at least one block id; no
section crosses a source identity.

### Phase 2 — lexical index

- Add the `bm25` crate.
- Build deterministic candidate pairs.
- Add Turkish tokenization tests; document stemming limitations.
- Add a BM25-only report so the tool is useful without a model.

**Done when:** `links build --no-model` produces a valid JSON/Markdown report
with no network or model.

### Phase 3 — model2vec adapter

- Implement `Model2VecEmbedder`.
- Load the bundled model; support `FOLISIFT_MODEL_DIR` and binary-sidecar
  resolution.
- Add a fake embedder for unit tests.
- Add model hash and version to the report.
- Use `spawn_blocking` for MCP calls.

**Done when:** `links build` works offline with the packaged model, and unit
tests never download a model.

### Phase 4 — link builder

- Implement scoring, thresholds, degree cap and deterministic ordering.
- Add page/source provenance to every edge.
- Write JSON and Markdown.
- Add `--dry-run`/`no-write` for tests.

**Done when:** output is byte-identical across repeated runs with the same
model and inputs; every edge has page provenance.

### Phase 5 — CLI and MCP integration

- Add `folisift links` subcommands.
- Add the MCP `links` tool with `status`, `build`, `read`.
- Update `prepare_guide`, `doctor`, schema version and `mcp_stdio` tests.
- Keep `build_guide` independent of links.

**Done when:** a user can build and read links from both CLI and MCP without
any model download, and `build_guide` behavior is unchanged.

### Phase 6 — release packaging

- Add the int8 model to `scripts/package.sh`.
- Add model checks to `scripts/check-package.sh` and `doctor`.
- Add licence/notice entries.
- Verify a clean archive install exposes `folisift links` and MCP `links`.
- Record the archive size and enforce a size budget in `check-package.sh`.

**Done when:** `make package-check` passes from a clean directory with the
model present; the archive contains the model and licence.

### Phase 7 — host review

- Run on the three-PDF course.
- Produce a Markdown review and 20 hand-checked pairs.
- Add the report path to the authoring workflow.
- Do not auto-insert links into guide prose.

**Done when:** an author can review the links in a few minutes and the guide
output is unchanged.

### Phase 8 — optional typed graph

Only if needed later:

- send page-tagged sections to an LLM or iText2KG;
- require `source`, `page`, `quote` and confidence on every typed edge;
- keep it a separate artifact and a separate experiment.

This phase is not part of the first release.

## 9. Tests and gates

Unit:

- sectioning: heading hierarchy, page-only sections, long splits;
- BM25 candidate determinism;
- fake embedder and cosine ranking;
- threshold/degree-cap behavior;
- every edge has source/page;
- JSON round-trip;
- model path resolution and manifest verification;
- no model/network in unit tests.

Integration:

- build on a small fixture with a fake embedder;
- build on PD2 with the real model (ignored unless cache is present);
- repeated runs produce identical output;
- MCP `links status` and `links read` over stdio;
- `doctor` passes in a clean package with the model;
- `folisift build` and exam PDF hashes unchanged;
- package archive contains the model and licence.

Commands:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
make exam-check
make package-check
```

## 10. Risks

| Risk | Mitigation |
|---|---|
| Static embeddings miss Turkish nuance | Phase 0 hand-check; fallback to fastembed-rs |
| Model is large | Int8 release asset; compressed archive; documented size |
| Model/crate is young | Pin exact versions; record hashes; legal review |
| Similarity is mistaken for fact | `SIMILARITY` confidence; review-only; no typed claims |
| Threshold produces hub nodes | Degree cap; mutual/top-k; tune on pilot |
| Stale links after edits | Source/block digest; `links status` marks stale |
| MCP blocks on inference | Lazy load; `spawn_blocking`; build is explicit |
| Model missing after install | `doctor`; actionable `links` error; no network fallback |
| Helper becomes a platform | Feature-scoped library; typed graph separate |
| Release archive bloat | Int8 model; size budget; test package size |
| Model licence/model drift | Manifest, licence file, hash verification |

## 11. Acceptance checklist

- [x] Model2Vec pilot shows useful Turkish pairs and acceptable latency.
- [x] Blocks are the only source of section text; no PDF re-parse.
- [x] Every link has source/page/block provenance.
- [x] BM25-only mode works without a model.
- [x] Model2Vec mode works offline from the installed model.
- [x] `folisift links` CLI works after a clean archive install.
- [x] MCP `links status/build/read` works after a clean archive install.
- [x] `doctor` checks model files and hashes.
- [x] Output is deterministic.
- [x] No model, network or LLM in MCP's other tools; links uses local inference only.
- [x] `build_guide` and exam PDF hashes are unchanged.
- [x] 20 hand-checked pairs reviewed; false-link rate documented.
- [x] Model and licence are in the release archive.
- [x] ADR records local embeddings in MCP and the model asset contract.

## 12. References

- model2vec-rs:
  <https://github.com/MinishLab/model2vec-rs>
- multilingual model:
  <https://huggingface.co/minishlab/potion-multilingual-128M>
- BM25 crate:
  <https://crates.io/crates/bm25>
- LiteParse block plan:
  [2026-11-02-liteparse-full-utilization.md](2026-11-02-liteparse-full-utilization.md)
- Research and alternatives:
  [lightweight-semantic-linking-research-2026-11-03.md](../lightweight-semantic-linking-research-2026-11-03.md)


## 13. Implementation log

- 2026-11-03: first implementation slice landed.
  - Added `crates/folisift-links` with page-aware sectioning, BM25 Turkish
    candidates, model2vec-rs embeddings and JSON report read/write.
  - Added `folisift links status|build|read` and MCP `links` (schema 7, 17
    tools).
  - Added `read_blocks` for bounded layout access.
  - Prepared `models/potion-multilingual-128M-int8` (123 MB model, 18 MB
    tokenizer) and included it in the local package path; the directory is
    gitignored and copied by `scripts/package.sh`.
  - `links build --no-model` works on PD2 as a BM25 baseline; model-backed
    build produces 8 links from 100 sections at the initial thresholds.
  - Clean-package check passes with the model in the archive, and the installed
    MCP `links status/read` path was exercised successfully.
  - Quality pilot: 20 hand-checked pairs, 15 useful / 3 weak / 2 false (75%
    useful). Default thresholds remain `0.75 / 0.55`; `0.65 / 0.45` is the
    exploratory setting. See [`semantic-link-pilot-2026-11-03.md`](../semantic-link-pilot-2026-11-03.md).
- Remaining: duplicate-target suppression and a second-corpus pilot.
