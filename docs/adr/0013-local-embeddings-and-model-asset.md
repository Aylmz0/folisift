# ADR-0013: Local static embeddings and the bundled model asset

- **Status:** Accepted
- **Date:** 2026-11-03
- **Deciders:** Maintainer

## Context

The study-guide author needs cross-document connections between source
sections. Folisift already extracts page/block geometry from LiteParse, but a
structural page map does not show that two sections are about the same idea.
The product goal is a single install that exposes the same capability through
the CLI and MCP, without a Python/uv step or a cloud API.

The existing MCP boundary says no LLM call, shell execution or network access.
A semantic helper must either respect that boundary or remain outside the
product. The maintainer chose to make it part of the product.

## Decision

1. **Local static embeddings only.** `model2vec-rs` runs a quantized
   multilingual Model2Vec model on the local CPU. It is an embedding model,
   not a generative LLM.
2. **No network in the MCP process.** The `model2vec-rs` dependency uses its
   local-only feature. `links` resolves a model directory and fails with an
   actionable error when it is absent; it never downloads.
3. **The model is a release sidecar.** The release archive contains
   `models/potion-multilingual-128M-int8/` with weights, tokenizer, manifest
   and MIT licence. `scripts/package.sh` requires it; `doctor` reports
   availability and hash verification.
4. **The helper is review-only.** `links` writes `data/document-links.json`
   and `data/document-links.md`. The edges are labelled `SIMILARITY`, not
   `EXTRACTED` or `INFERRED` facts. They never gate a guide build or alter
   coverage, claims or page budgets.
5. **One CLI/MCP surface.** `folisift links` and the MCP `links` tool call the
   same `folisift-links` library. `read_blocks` exposes bounded layout data for
   review. No separate helper binary is installed.
6. **Configurable, deterministic model resolution.** Resolution order is
   `FOLISIFT_MODEL_DIR`, `<workspace>/data/models/<model>`,
   `<workspace>/models/<model>`, then the binary sidecar. Weights are verified
   against `manifest.json` before loading.
7. **FastEmbed is a fallback, not the default.** If the model2vec pilot fails
   Turkish quality, `fastembed-rs` can replace the `Embedder` adapter; the
   graph schema and MCP contract stay unchanged.

## Consequences

- A normal Linux install gets CLI and MCP links with no Python, uv, API key or
  network requirement.
- The release archive grows by roughly the int8 model and tokenizer size. This
  is an explicit trade-off for the single-install goal.
- `links build` can be slower than a normal build and uses local CPU; it is an
  explicit action, not part of `build_guide`.
- A missing or hash-mismatched model makes `links` unavailable; it does not
  silently fall back to a network download or a different model.
- The model is not a source of truth. Users must inspect the cited pages; the
  pilot records a 15/20 useful rate and keeps the review step.
