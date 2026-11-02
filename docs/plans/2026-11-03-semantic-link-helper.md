# Semantic link helper with model2vec-rs — 2026-11-03

- **Status:** Proposed; implementation not started
- **Decision:** use **model2vec-rs** for local semantic similarity
- **Scope:** optional `folisift-links` helper, block-level section links
- **Depends on:** [LiteParse full-utilization plan](2026-11-02-liteparse-full-utilization.md)
- **Blocking gate:** do not start until `data/blocks/<source>.json` exists and
  is stable for the pilot corpus (LiteParse plan phases 0–3).
- **Non-goals:** typed fact extraction, graph database, LLM inside MCP,
  changing guide prose or build gates
- **Constraint:** helper is removable; default `folisift` build and release
  archive do not require it

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

The helper is a separate binary, not an MCP tool. The MCP server may later read
the precomputed JSON, but it never loads a model or performs inference.

## 2. Why model2vec-rs

Selected because it is the smallest credible local semantic layer:

- **MIT** (repository; crates.io metadata is missing a license field, so pin
  and record the repository licence).
- Rust implementation, no Python service.
- No ONNX Runtime; uses `safetensors` + `tokenizers`.
- `potion-multilingual-128M` supports 101 languages including Turkish.
- Static embeddings are fast on CPU; int8 quantization keeps memory around
  ~128 MB for embeddings.
- Works offline after the model cache is populated.
- Small enough to keep as an optional helper binary.

Trade-offs:

- Static embeddings are less accurate than full transformer embeddings.
- The crate is young (0.4.0, ~225 stars); pin it and run a quality pilot.
- The multilingual f32 checkpoint is ~512 MB and tokenizer ~18 MB. The model
  is not bundled; it is downloaded once into `data/models/` or a user cache.

`fastembed-rs` is the fallback if model2vec quality fails: Apache-2.0, more
mature (~1.1k stars), more model choices, but it adds ONNX Runtime.

`txtai` and `iText2KG` are not selected for this helper. `txtai` is a Python
framework, and iText2KG is a Python/LLM graph-construction pipeline without a
page-provenance contract. They remain later options for typed graphs.

## 3. Module seam

Create an optional workspace crate:

```text
crates/folisift-links/
  src/lib.rs
  src/model.rs
  src/section.rs
  src/link.rs
  src/main.rs
```

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
- `Embedder` is the only external seam. Production uses model2vec-rs; tests
  use a deterministic fake.
- `LinkIndex::build` returns data. It does not read files, download models or
  write artifacts.
- The CLI layer reads `data/blocks/*.json`, calls the builder and writes the
  report.

Deletion test: removing `folisift-links` must not change any guide PDF, MCP
tool or coverage result.

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
- Repeated headers/footers are already suppressed by LiteParse's block
  classifier where possible.

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
- `model2vec-rs` loads the local model and uses `quantize_int8()`.
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

## 6. Model management

- Pin `model2vec-rs = "=0.4.0"` in `folisift-links`.
- Default model: `minishlab/potion-multilingual-128M`.
- Store models under `data/models/<model>/` or `FOLISIFT_MODEL_DIR`.
- Add the model directory to `.gitignore`; never commit weights.
- Provide:

```bash
folisift-links model fetch
folisift-links model verify
folisift-links build --guide pd2 --offline
```

`model fetch`:

1. downloads the model files from Hugging Face;
2. writes a manifest with model id, file sizes and SHA-256;
3. prints the licence and cache path.

`model verify` checks hashes before `build`.

No network is used by `build` when `--offline` is set. A missing model is an
actionable error, not a silent fallback.

## 7. CLI

```bash
# Build links for one guide
folisift-links build --guide pd2 --top-k 5 --min-cosine 0.75
folisift-links build --guide pd2 --no-model   # BM25-only report

# Rebuild the Markdown review only
folisift-links report --guide pd2

# Explain one section and its neighbours
folisift-links explain --guide pd2 --section PD2:p26:s1
```

`build` writes both JSON and Markdown atomically.

The helper does not get a `folisift` subcommand and is not part of the MCP
server. If the host needs links, it reads the JSON path returned by
`prepare_guide` or a small resource later.

## 8. Phases

### Phase 0 — model and quality spike

- Download `potion-multilingual-128M` into a temp cache.
- Encode 20 Turkish sections from the three PDFs (or a representative local
  corpus).
- Measure load time, encode time, memory and model size.
- Hand-check 20 related pairs; record useful / false / missed.
- Compare with a BM25-only baseline.

**Done when:** the pilot has a measured quality and latency table, and the
team decides whether static embeddings are good enough.

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
- Add a BM25-only report so the helper is useful without a model.

**Done when:** `build --no-model` produces a valid JSON/Markdown report with
no network or model.

### Phase 3 — model2vec adapter

- Implement `Model2VecEmbedder`.
- Add `model fetch`, `verify`, `--offline`.
- Add a fake embedder for unit tests.
- Add model hash and version to the report.

**Done when:** `build` works offline with a verified local model, and unit
tests never download a model.

### Phase 4 — link builder

- Implement scoring, thresholds, degree cap and deterministic ordering.
- Add page/source provenance to every edge.
- Write JSON and Markdown.
- Add a `--dry-run`/`--no-write` path for tests.

**Done when:** the output is byte-identical across repeated runs with the same
model and inputs; every edge has page provenance.

### Phase 5 — host review

- Run on the three-PDF course.
- Produce a Markdown review and 20 hand-checked pairs.
- Add the report path to the authoring workflow.
- Do not auto-insert links into guide prose.

**Done when:** an author can review the links in a few minutes and the helper
can be deleted without changing the guide.

### Phase 6 — optional typed graph

Only if needed later:

- send page-tagged sections to an LLM or iText2KG;
- require `source`, `page`, `quote` and confidence on every typed edge;
- keep it a separate artifact and a separate experiment.

This phase is not part of the first helper.

## 9. Tests and gates

Unit:

- sectioning: heading hierarchy, page-only sections, long splits;
- BM25 candidate determinism;
- fake embedder and cosine ranking;
- threshold/degree-cap behavior;
- every edge has source/page;
- JSON round-trip;
- model manifest verification;
- no model/network in unit tests.

Integration:

- build on a small fixture with a fake embedder;
- build on PD2 with the real model (ignored unless cache is present);
- repeated runs produce identical output;
- `folisift` build and exam hashes unchanged;
- helper not present in release archive by default.

Commands:

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
cargo run -p folisift-links -- build --guide pd2 --offline
make exam-check
```

## 10. Risks

| Risk | Mitigation |
|---|---|
| Static embeddings miss Turkish nuance | Phase 0 hand-check; fallback to fastembed-rs |
| Model is large | Optional download, int8, cache outside repo |
| Model/crate is young | Pin exact versions; record hashes; legal review |
| Similarity is mistaken for fact | `SIMILARITY` confidence; review-only; no typed claims |
| Threshold produces hub nodes | Degree cap; mutual/top-k; tune on pilot |
| Stale links after edits | Include source digest; rebuild on block/inventory change |
| Helper becomes a platform | Separate crate, no MCP/model calls, deletion test |
| Model licence/model drift | Manifest, licence file, hash verification |
| Turkish tokenization/BM25 weakness | Keep exact terms; test; embeddings carry the semantic part |

## 11. Acceptance checklist

- [ ] Model2Vec pilot shows useful Turkish pairs and acceptable latency.
- [ ] Blocks are the only source of section text; no PDF re-parse.
- [ ] Every link has source/page/block provenance.
- [ ] BM25-only mode works without a model.
- [ ] Model2Vec mode works offline after cache verification.
- [ ] Output is deterministic.
- [ ] No model, network or LLM in MCP.
- [ ] Helper is excluded from the default release archive.
- [ ] Removing the helper leaves guide builds unchanged.
- [ ] 20 hand-checked pairs reviewed; false-link rate documented.

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
