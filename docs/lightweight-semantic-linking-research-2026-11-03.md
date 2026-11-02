# Lightweight semantic linking research — 2026-11-03

- **Question:** Can Folisift add a small, local helper that links source-text
  sections by meaning, without adopting a GraphRAG platform?
- **Status:** Research; no dependency selected
- **Preference:** FOSS, local-first, small blast radius, no SaaS lock-in
- **Constraint:** Folisift builds stay deterministic; the MCP server performs
  no LLM, shell or network calls.
- **Related:** [LiteParse full-utilization plan](plans/2026-11-02-liteparse-full-utilization.md)

## 1. What the helper should do

The useful first version is not a knowledge-graph platform. It is a small
"related sections" index:

```text
LiteParse page/block
  -> section text with source/page/block id
  -> lexical candidates (BM25)
  -> embedding similarity
  -> top-k related-section edges (with page/bbox)
  -> document-links.json
```

It should answer:

- Which sections in the three PDFs discuss the same idea?
- Which sections are likely prerequisites or follow-ups?
- Which definitions are duplicated or conflicting?
- Which source pages should the author read together?

It should **not** claim causality, truth or prerequisite relations without an
LLM review. Similarity is a proposal, not a fact.

## 2. Shortlist

| Option | License | Runtime | Turkish | Page provenance | Weight |
|---|---|---|---|---|---|
| **model2vec-rs** | MIT | Rust; tokenizers + safetensors; no ONNX/Python | multilingual model (101 languages) | attach page/block outside the model | **lowest** |
| **fastembed-rs** | Apache-2.0 | Rust + ONNX Runtime; model cache | multilingual E5/paraphrase models | attach page/block outside the model | medium |
| **bm25** | MIT | pure Rust | tokenizer/stemmer dependent | exact block metadata | low |
| **Tantivy** | MIT | pure Rust search index | tokenizer dependent | exact block metadata | medium |
| **txtai** | Apache-2.0 | Python 3.11+ / ML stack | model dependent | row metadata possible | high for Folisift |
| **iText2KG** | Apache-2.0 | Python + LangChain + LLM + embeddings | model dependent | no page field | high |
| **petgraph** | MIT/Apache-2.0 | pure Rust | n/a | graph structure only | low, optional |
| **usearch / hnsw_rs** | Apache-2.0 / MIT | vector index | n/a | no | unnecessary at this scale |
| **sqlite-vec** | MIT/Apache-2.0 | SQLite C extension | n/a | metadata possible | unnecessary |

## 3. model2vec-rs — best "small helper" candidate

`model2vec-rs` is the official Rust implementation of Model2Vec static
embeddings. It is MIT, has ~225 GitHub stars, and loads static embedding models
with `safetensors` + `tokenizers`.

Why it fits:

- **No Python and no ONNX Runtime.** The dependency surface is much smaller
  than FastEmbed's ONNX Runtime stack.
- **Fast CPU inference.** The project reports 671K texts/s single-thread and
  3.06M texts/s all-cores for an 8M English model; that is far beyond what 750
  blocks need.
- **Int8 quantization.** `quantize_int8()` keeps embeddings at roughly a
  quarter of f32 memory with high cosine agreement.
- **Multilingual model.** `potion-multilingual-128M` supports 101 languages,
  including Turkish (`tr`), and is MIT-licensed on Hugging Face.
- **Offline after cache.** Use the `local-only` feature or a local model
  directory to avoid runtime downloads.

Costs:

- The multilingual f32 checkpoint is ~512 MB; the tokenizer is ~18 MB. Int8
  in-memory embeddings are roughly 128 MB. This is an optional download, not a
  binary bundle.
- The Rust crate is young (0.4.0, ~225 stars). It needs a pilot and a pinned
  version.
- Static embeddings are less accurate than full transformer embeddings.
  Reddit reports a ballpark 10–15% quality drop for some retrieval tasks, in
  exchange for much higher speed. That is acceptable for "related sections",
  not for factual claims.

## 4. fastembed-rs — mature fallback

`fastembed-rs` is Apache-2.0, ~1.1k stars, and supports multilingual models
such as `MultilingualE5Small`, `ParaphraseMLMiniLML12V2Q` and BGE-M3. It also
has built-in cosine/top-k helpers and rerankers.

Why it might win:

- More mature and widely used than model2vec-rs.
- Many model choices, including multilingual and quantized variants.
- Local model cache; no Python.

Why it is not first:

- It depends on ONNX Runtime (`ort`), which adds a native runtime and a larger
  packaging/install surface.
- A full transformer model is slower and heavier than static embeddings.
- For 750 blocks and "related sections", the extra model quality may not
  justify the extra runtime.

Use it if a model2vec pilot fails Turkish quality or if reranking is needed.

## 5. bm25 / Tantivy — lexical baseline

The `bm25` crate is MIT and pure Rust. Tantivy is a mature Lucene-like search
engine with BM25 and `MoreLikeThisQuery`.

Use BM25 as:

- a deterministic first-stage candidate generator;
- a fallback when no embedding model is installed;
- a way to keep exact terminology, numbers and Turkish proper nouns.

Do not call BM25 "semantic". It is a strong lexical baseline and a good
hybrid partner for embeddings.

For 747 blocks, a simple in-memory BM25 index is enough; Tantivy is only worth
it if the helper grows into a full search index.

## 6. txtai — mature, but not a small Rust helper

txtai is Apache-2.0, ~13.1k stars, and combines vector search, graph networks
and relational data. It has semantic-graph examples and Rust bindings.

Why it is attractive:

- Mature, active and well documented.
- Semantic graph and graph traversal built in.
- Local operation is possible; no mandatory cloud.
- Reddit users describe it as easier and more performant than larger RAG
  frameworks.

Why it is not the first choice for Folisift:

- It is a Python 3.11+ framework with a large ML dependency stack.
- Using it from Rust means a Python service/API boundary, not a small library.
- It gives similarity/backlinks, not page-anchored typed facts by default.
- That is a large operational addition for a helper.

## 7. iText2KG — semantic KG, but not lightweight

iText2KG is Apache-2.0, ~974 stars, Python, and built around LangChain chat and
embedding models. It extracts entities/relations incrementally, with entity
resolution and optional temporal graphs.

Why it is interesting:

- It actually builds typed knowledge graphs, not just similarity links.
- Incremental updates and entity resolution are built in.
- It has a PDF utility script and an ATOM pipeline for atomic facts.

Why it is not a small helper:

- Requires an LLM and embeddings; Python and LangChain dependencies.
- The PDF script uses PyMuPDF; page provenance is not a core graph field.
- It is research/work-in-progress software, not a drop-in page-aware layer.
- It would add a second extraction pipeline beside LiteParse.

Use it only if typed relations (`prerequisite_of`, `part_of`, etc.) are worth a
separate LLM pipeline. For "related sections", model2vec + BM25 is cheaper.

## 8. Graph structure and vector index

At Folisift's scale:

- 3 PDFs, ~207 pages, ~750 blocks.
- Brute-force cosine over 750 vectors is trivial.
- A graph database or ANN index is unnecessary.
- An adjacency list or `petgraph` is enough; JSON is the portable artifact.
- `usearch`, `hnsw_rs` and `sqlite-vec` solve a scale problem we do not have.

Use `petgraph` only if we need shortest path, connected components or
centrality. Otherwise a `Vec<Node>` + `Vec<Edge>` is simpler and easier to
serialize.

## 9. Community signals

These are practitioner anecdotes, not controlled benchmarks.

- **Model2Vec quality/speed:** Reddit discussions describe static embeddings
  as roughly an order of magnitude faster with a ~10–15% quality drop for some
  retrieval tasks.
  <https://www.reddit.com/r/LocalLLaMA/comments/1glwbsq/using_highthroughput_model2vecpotion_embedding/>
- **Obsidian-style auto-linking:** users value local/offline operation and
  review-before-write. Projects combine deterministic title/alias matching with
  local embeddings and let the user approve links.
  <https://www.reddit.com/r/ObsidianMD/comments/1s46wxu/i_built_a_tool_that_automatically_adds_semantic/>
  <https://www.reddit.com/r/ObsidianMD/comments/1tnxv2q/plugin_showcase_semantic_autolinker_generate/>
- **FastEmbed:** local embedding users discuss it as a practical local option;
  the main trade-off is the ONNX Runtime dependency.
  <https://www.reddit.com/r/LocalLLaMA/comments/1f52xc6/hows_your_experience_with_fastembeddings/>
- **txtai:** users describe it as easier/less code and often performant, but it
  is a Python framework, not a Rust helper.
  <https://www.reddit.com/r/Rag/comments/1gcj429/vector_search_rag_landscape_a_review_with_txtai/>
- **GraphRAG/LightRAG caution:** flat RAG sometimes beats graph RAG for
  document Q&A; graph quality is prompt/source dependent.
  <https://www.reddit.com/r/Rag/comments/1qyvjt1/lightrag_graph_rag_performing_worse_than_open_web/>
- **PDF parser context:** users compare LiteParse as lightweight/fast and
  Docling as heavier but layout-rich.
  <https://www.reddit.com/r/Rag/comments/1shdula/rag_for_complex_pdfs_struggling_with_parsing_vs/>

## 10. Recommendation

**Build a small optional helper, not a semantic platform.**

Proposed stack:

```text
LiteParse blocks (page + bbox + section)
  -> BM25 candidate pairs
  -> model2vec-rs multilingual int8 embeddings
  -> cosine top-k with a conservative threshold
  -> document-links.json
  -> host/OpenCode reads links; author approves
```

Why this combination:

- one small Rust embedding dependency, no Python/ONNX/graph DB;
- works offline after the model cache is populated;
- page/block provenance stays in our JSON, independent of the model;
- lexical + semantic hybrid handles Turkish terms and exact terminology;
- no LLM call is required for the first version;
- typed relations remain a separate optional LLM experiment.

Do **not** start with txtai, iText2KG, GraphRAG or LightRAG for this helper.
They are good systems, but they either add Python/ML infrastructure or do not
give page-level provenance. Keep them as later experiments if the small helper
proves the value.

## 11. Pilot acceptance criteria

- [ ] 20 hand-checked "related section" pairs; at least 15 are genuinely useful.
- [ ] No page/block provenance loss.
- [ ] Turkish terms and cross-source definitions are retrieved correctly.
- [ ] False-link rate low enough that an author can review in a few minutes.
- [ ] Works offline after model download.
- [ ] No model/network dependency in the MCP server or normal build.
- [ ] Helper can be removed without changing guide output.

## 12. Sources

- model2vec-rs:
  <https://github.com/MinishLab/model2vec-rs>
- Model2Vec multilingual model:
  <https://huggingface.co/minishlab/potion-multilingual-128M>
- fastembed-rs:
  <https://github.com/Anush008/fastembed-rs>
- bm25 crate:
  <https://crates.io/crates/bm25>
- Tantivy:
  <https://github.com/quickwit-oss/tantivy>
- txtai:
  <https://github.com/neuml/txtai>
- iText2KG:
  <https://github.com/AuvaLab/itext2kg>
- petgraph:
  <https://github.com/petgraph/petgraph>
- sqlite-vec:
  <https://github.com/asg017/sqlite-vec>
