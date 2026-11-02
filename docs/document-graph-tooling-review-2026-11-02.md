# Document-graph tooling for source PDFs — 2026-11-02

- **Question:** Can a Graphify-like system be applied to the source PDFs so the
  authoring agent understands them better and builds cross-document connections?
- **Status:** Research and design evaluation; no runtime dependency added
- **Decision:** Promising as an **authoring aid**, but no ready generic system
  satisfies Folisift's page-level traceability contract out of the box. Do not
  add GraphRAG/LightRAG/Graphify as a runtime dependency yet.
- **Folisift context:** three PDFs, ~207 pages, page-level `source_scope`,
  citation-based coverage, deterministic build and no LLM/network/shell inside
  the MCP server.

## 1. The useful version of the idea

There are two possible layers:

1. **Deterministic document graph**
   Nodes: source, page, section, heading, term, table, figure, citation,
   guide section.
   Edges: `contains`, `next`, `cites`, `defines`, `compares`, `same_term`,
   `cross_source`.
   This can be generated from the existing liteparse inventory and needs no
   LLM.

2. **Semantic relationship layer**
   Nodes: concepts extracted from prose.
   Edges: `prerequisite_of`, `contrasts_with`, `causes`, `part_of`,
   `same_concept`.
   This needs an LLM or a carefully designed NLP extractor, and every edge
   needs a page/quote anchor to remain reviewable.

The first layer is safe and useful immediately. The second is the part the
user is asking about: it can surface connections across three PDFs that a
chapter-by-chapter reader would miss. But it is a hypothesis generator, not a
source of truth.

The graph should help the agent **find and propose** connections; it must not
replace citations, `source_scope`, coverage gates or claim spot-checks.

## 2. LiteParse: strong parser, not a graph engine

LiteParse is the right foundation for the deterministic layer, and Folisift is
currently using only a fraction of it.

Current adapter behavior:

- `LiteparseParser` uses `OutputFormat::Text`.
- It does not set `extract_blocks`.
- Headings are inferred from the first meaningful text line, not from the
  layout block classifier.
- Tables are extracted by a separate Folisift table pass, not from LiteParse's
  block/cell data.

LiteParse 2.17.0 can provide much more:

| Capability | Output | Use for Folisift |
|---|---|---|
| `extract_blocks` | `heading`, `paragraph`, `list_item`, `code`, `table`, `figure`, `rule` blocks in reading order, each with `bbox` | Page/section/block graph |
| table blocks | `header` and `rows` with per-cell `bbox` | Table → concept/term links; source-grounded table extraction |
| heading blocks | `level`, `text`, `bbox` | Exact heading hierarchy instead of first-line heuristics |
| `extract_text_metadata` | font height/weight/color, MCID, rotation | Typography-aware heading/term detection |
| `extract_structure_tree` | tagged-PDF logical structure | Better reading order for tagged PDFs |
| `extract_annotations` / links | annotation and link metadata | Cross-reference and URL edges |

LiteParse is explicitly a parser: its own contributing guide says it is less
interested in LLM integration or agent code. That is not a weakness; it means
we should use it for **blocks and provenance**, not ask it to infer semantics.

The immediate low-risk win is:

```text
OutputFormat::Json + extract_blocks
  → source/page/block/table/figure nodes with bbox and reading order
```

No new dependency, no LLM, no graph database.

## 3. Ready systems

### Microsoft GraphRAG

GraphRAG is a pipeline that uses an LLM to extract entities, relationships and
optional claims from text units, then builds community summaries and reports.
The standard method uses an LLM for entity extraction, relationship extraction,
summarization and community reports. A faster method swaps some extraction for
NLP noun phrases and co-occurrence, but still uses an LLM for community reports.

Its `text_units` output contains `document_id`, text and token counts, but not a
page number. Page provenance is not part of the core text-unit schema. You can
chunk page-by-page and prepend page metadata, but that is your responsibility.

Fit: good for global "what are the themes across this corpus?" questions; heavy
for three PDFs and not page-auditable by default.

### LightRAG

LightRAG combines a knowledge graph with vector retrieval. It extracts entities
and relationships with an LLM, supports document insertion with `file_paths`
for citation, and returns chunk-level references. Its citation contract is a
file path plus chunk content, not a page number.

Fit: lighter than GraphRAG for incremental updates, but still LLM-dependent and
not page-native. Page numbers would have to be encoded in the chunks/metadata.

### Neo4j LLM Graph Builder

LLM Graph Builder ingests PDFs and other sources, splits them into chunks, and
uses LangChain's `LLMGraphTransformer` to extract entities and relationships.
Its graph model is useful:

```text
Document -PART_OF-> Chunk -HAS_ENTITY-> Entity
Document -FIRST_CHUNK-> Chunk
Chunk -NEXT_CHUNK-> Chunk
```

The PDF loader records `page` and `total_pages` in document metadata, so a
chunk can carry a page number. That makes page-aware entity linking possible,
but it requires Neo4j, an LLM and a chunking policy. The entity itself is still
an LLM extraction, not a verified fact.

Fit: the closest ready system to a page-aware document graph, but too heavy for
Folisift's current three-PDF workflow.

### Graphify

Graphify is a general knowledge-graph tool: Tree-sitter for code, and an
optional semantic pass for documents, PDFs, images and transcripts. For PDFs it
needs the PDF extra (for example `pypdf`) and a configured LLM backend. Its
node schema has `source_file`, but `source_location` is optional and is often
`null`; page numbers are not a guaranteed part of the graph contract.

Fit: useful for a quick cross-document concept graph, but not safe as the
citation layer. Splitting PDFs into per-page files before extraction is a
workaround, but it weakens cross-page context and still leaves LLM extraction
as the source of edges.

### PaperQA2

PaperQA2 indexes scientific PDFs and answers questions with page-level
citations. It is a retrieval/QA system, not an entity-relationship graph. It is
a better fit for "ask these papers a question" than for "build a concept map
for a guide".

Fit: useful as a research assistant during authoring; does not solve graph
construction or coverage.

### RAPTOR

RAPTOR builds a recursive tree of chunk summaries: leaves are text chunks,
higher layers are LLM-generated summaries of clusters. It is a hierarchy for
retrieval, not an entity graph. It can help multi-level summarization, but it
does not produce `A relates_to B` concept edges.

### iText2KG

iText2KG builds a knowledge graph from text sections with an LLM and embeddings.
It supports incremental graph updates and relationship observation dates. Its
input contract is a list of section strings, so page provenance must be encoded
by the caller; it is a semantic-extraction library, not a PDF/page graph.

Fit: plausible for the semantic layer after LiteParse has produced page/block
sections. It does not solve parsing or page citations by itself.

### txtai

txtai combines embeddings, vector search and graph networks. It can build a
semantic graph from indexed rows, run openCypher-style graph queries and combine
`SIMILAR` vector search with graph traversal. Its graph is similarity/row
oriented; entity/relation extraction is an example pipeline, not a fixed
document-graph contract.

Fit: good for Obsidian-like "related sections" and backlinks; metadata can carry
page/source fields, but the page graph is still our schema.

### GraphRAG SDK (FalkorDB)

GraphRAG SDK provides PDF ingestion, entity/relation extraction, citation-aware
retrieval and incremental `apply_changes` over FalkorDB. It is a complete
pipeline with a graph database and LLM/embedder dependencies.

Fit: closer to a production system than a library, but heavier than a
three-PDF course needs.

### WhyHow Knowledge Graph Studio

WhyHow provides self-hosted Docker deployment, PDF ingestion, schema-driven
entity/relation extraction and graph querying. It is an API platform with
workspace/S3-style document handling, not a small Rust component.

Fit: worth a separate pilot if the graph becomes a product; too heavy for the
first experiment.

### TrustGraph

TrustGraph is an Apache-2.0 FOSS context backend. Its PDF extraction path is
explicitly page-aware:

```text
Document → Pages → Chunks → Triples/Embeddings
```

The page and chunk nodes are stored in the same graph as extracted facts, and
each triple is linked back to its immediate chunk. Its documentation describes
this as extraction-time provenance: a fact can be traversed back to its exact
page/chunk instead of only its source file.

Fit: the closest ready FOSS platform to Folisift's page-level requirement, but
it brings a Docker/Pulsar/graph-database deployment and an LLM extraction
pipeline. It is a pilot candidate, not a small dependency.

### InfraNodus

InfraNodus builds text networks, topic clusters and content-gap reports through
a hosted API. It is strong at "what topics are connected and where are the
gaps?", but it is not a page-anchored citation graph and the analysis backend is
external.

Fit: useful as a research/authoring service; not a local deterministic gate.

### RAGFlow

RAGFlow has document understanding and citations, but its standalone GraphRAG
feature was deprecated/restructured in v0.27.0 in favor of knowledge compilation
tasks. That makes it a moving target for this use case.

Fit: do not build the Folisift contract on its GraphRAG feature.

### Docling

Docling is a PDF/document parser, not a graph system. Its useful contribution
is provenance: every `DocItem` carries a `ProvenanceItem` with `page_no`,
bounding box and character span, and the body tree preserves reading order.
Tables, pictures, captions and formulas are typed items.

Fit: the best parser-side foundation if we ever build our own page-anchored
graph. Folisift already uses liteparse, so this would be a replacement or a
second parser, not a small addition.

## 4. Page-provenance comparison

| System | Graph? | Page provenance | LLM needed? | Fit for Folisift |
|---|---|---|---|---|
| **GraphRAG** | Yes | Text units carry `document_id`, not page by default | Yes for standard extraction | Global themes; needs page-aware chunking |
| **LightRAG** | Yes | File path + chunk content, no page contract | Yes | Incremental graph; page metadata DIY |
| **Neo4j LLM Graph Builder** | Yes | PDF chunk metadata can carry `page` | Yes | Closest page-aware ready system; Neo4j + LLM |
| **Graphify** | Yes | `source_file`; `source_location` often null | Yes for PDFs/docs | Quick concept graph; not citation-safe |
| **iText2KG** | Yes | Section strings; page metadata DIY | Yes + embeddings | Semantic layer after page-aware blocks |
| **txtai** | Graph/vector | Row metadata; no page contract | Optional/LLM examples | Related-section backlinks; page schema DIY |
| **GraphRAG SDK** | Yes | PDF loader; citation support; page metadata needs checking | Yes | Complete but FalkorDB-heavy |
| **WhyHow KG Studio** | Yes | Platform ingestion; page behavior needs pilot | Yes | Self-hosted but heavy |
| **TrustGraph** | Yes | Document → page → chunk → triple DAG | Yes | Closest FOSS page-provenance platform; heavy deployment |
| **InfraNodus** | Text network | No page contract; hosted API | Service-side | Topics/gaps, not citation graph |
| **RAGFlow** | GraphRAG deprecated | Parser has citations; graph feature moving | Yes | Do not target its old GraphRAG |
| **PaperQA2** | No | Page-level citations | Yes for answers | Research QA, not graph |
| **RAPTOR** | Tree | Chunk hierarchy, not page graph | Yes for summaries | Multi-level summary, not relations |
| **Docling** | No | Exact page/bbox/charspan | No (parser) | Best provenance source for a custom graph |
| **LiteParse** | No | Page, block and table-cell bbox | No | Best structural input; semantic layer is separate |

The pattern is clear: the systems that build semantic graphs do not guarantee
page-level provenance, and the system with the best provenance is not a graph.

## 5. What a Folisift-native graph could look like

For three PDFs, a graph database is unnecessary. A JSON artifact plus a small
Rust validator is enough.

Proposed artifact:

```text
guides/<slug>/data/document-graph.json
```

Deterministic layer:

```json
{
  "schema_version": 1,
  "nodes": [
    {"id": "PD2:p26", "kind": "page", "source": "PD2", "page": 26},
    {"id": "PD2:p26:s2.4", "kind": "section", "source": "PD2", "page": 26, "title": "LISP"},
    {"id": "term:lisp", "kind": "term", "label": "LISP", "source": "PD2", "page": 26}
  ],
  "edges": [
    {"from": "PD2:p26", "to": "PD2:p26:s2.4", "relation": "contains", "confidence": "EXTRACTED"},
    {"from": "term:lisp", "to": "PD2:p26", "relation": "defined_on", "confidence": "EXTRACTED"}
  ]
}
```

Optional semantic layer, written by the host agent outside the MCP server:

```json
{"from": "term:lisp", "to": "term:lambda_calculus",
 "relation": "implemented_with",
 "confidence": "INFERRED",
 "source": "PD2", "page": 26, "quote": "Sentaks lambda calculus'a dayalıdır."}
```

Rules:

- Every semantic edge needs `source`, `page` and a short quote.
- `EXTRACTED` means the relation is explicit in the text; `INFERRED` means the
  agent supplied the connection. Both are reviewable; only the first is a
  citation.
- The graph is advisory. Coverage, claims and page references remain the gates.
- The MCP server can read and validate a precomputed graph, but must not run an
  extractor, LLM or network call.

## 6. Build vs buy

Do **not** build a semantic extraction engine from scratch. Entity extraction,
relation extraction, coreference, entity resolution, Turkish handling and
evaluation are a product-sized problem. That is the user's concern, and it is
valid.

Build only the adapter and the contract:

- read LiteParse blocks and page geometry;
- attach stable page/block ids;
- send page-tagged sections to an existing extractor;
- validate every returned edge has a source page and quote;
- store the result as graph JSON;
- keep the graph advisory.

Do not build the model, the graph database or the retrieval UI ourselves unless
a ready system fails a measured pilot.

### Shortlist by integration cost

| Approach | Work | Best for | Main catch |
|---|---|---|---|
| **iText2KG** | Python library; LLM + embeddings | Typed semantic edges from page-tagged sections | No page field; we add page metadata around sections |
| **TrustGraph** | Docker/Pulsar/graph DB + LLM | Page-level provenance as a platform | Operational weight; overkill for three PDFs |
| **txtai** | Python; local embeddings + graph | Obsidian-style related sections/backlinks | Similarity, not typed facts; page metadata DIY |
| **LightRAG** | Python service; graph + vector; LLM | Incremental graph + local/global retrieval | Page provenance DIY; Reddit reports mixed quality |
| **GraphRAG** | Python pipeline; LLM; Parquet/vector store | Global themes/community summaries | Expensive indexing; page provenance DIY |
| **Neo4j LLM Graph Builder** | Neo4j + LLM + loader | Page-aware entity graph with a UI | Heavy platform; not a small dependency |
| **Own semantic engine** | High | Full control | Do not do this now |

### Recommended first experiment

Use **iText2KG** as the semantic extractor, fed by LiteParse page/block sections:

```text
LiteParse block
  -> section text with [source, page, block_id]
  -> iText2KG graph (entities + relations)
  -> Folisift validation (page + quote + confidence)
  -> document-graph.json
```

If page provenance is the hard requirement and the Docker/Pulsar/graph-DB
footprint is acceptable, pilot **TrustGraph** instead. Its document model is
already `Document -> Pages -> Chunks -> Triples`, which is the closest ready
FOSS match to Folisift.

### What not to do

Do not start with Microsoft GraphRAG or LightRAG just because they are popular.
Both can produce useful global summaries, but for a three-PDF authoring task
they add an LLM extraction pipeline and a graph/vector store before we know the
graph improves the guide. Reddit reports in section 8 are a warning against
assuming "more graph" means "better document answers".

**Adopt a ready system when:** a measured pilot on the three PDFs produces
useful, page-anchored edges with acceptable false-positive rate and operational
cost. Until then, the only code we should own is the adapter and validator.

## 7. Recommendation

1. **Do not add a generic GraphRAG/Graphify PDF pipeline as a runtime
   dependency.** It would import LLM nondeterminism, page-provenance loss and a
   new storage/toolchain layer.
2. **Try a page-anchored document graph on the three PDFs as an authoring
   experiment.** Use liteparse for pages and headings; generate deterministic
   structural edges first.
3. **Add semantic edges only with page + quote anchors.** Prefer a host-side
   LLM pass outside the MCP server; store `EXTRACTED`/`INFERRED` confidence.
4. **Use the graph to produce a connection memo**, not prose: cross-source
   concepts, prerequisites, contradictions, duplicate definitions, candidate
   diagram edges.
5. **Measure:** useful connections found, false edges, pages that had to be
   re-read, time saved, and whether the guide's coverage/claims changed.
6. **Promote to a product feature only if the experiment wins.** The selected
   similarity layer is planned as a release `links` CLI/MCP surface over a JSON
   sidecar; typed semantic edges remain a separate experiment.

The user's idea is sound at the **authoring intelligence** layer. The mistake
would be treating a generic graph extractor as the source of truth. The
implementation sequence for the structural half is
[`plans/2026-11-02-liteparse-full-utilization.md`](plans/2026-11-02-liteparse-full-utilization.md). Folisift's
advantage is page-level auditability; the graph must inherit that, not bypass
it.

## 8. Community signals from Reddit

These are practitioner anecdotes, not controlled benchmarks. They are useful
for risk framing, not for proving quality.

- **LightRAG vs GraphRAG:** a recurring comparison is that LightRAG is cheaper
  and easier to update incrementally, while its graph can feel less coherent;
  extraction quality depends heavily on the prompt, source type and model.
  <https://www.reddit.com/r/Rag/comments/1jtdrtt/graphrag_vs_lightrag/>
- **Graph RAG vs flat RAG for PDF Q&A:** one practitioner with LightRAG +
  Neo4j + Milvus reported that a plain Docling + embedding setup answered
  document questions better; the graph was "more brittle" and missed context.
  <https://www.reddit.com/r/Rag/comments/1qyvjt1/lightrag_graph_rag_performing_worse_than_open_web/>
- **Cost:** multiple threads describe GraphRAG's indexing cost as driven by
  large numbers of LLM calls; graph storage is not the expensive part.
  <https://www.reddit.com/r/LLMDevs/comments/1is4pat/graphrag_isnt_just_a_technique_its_a_paradigm/>
  <https://www.reddit.com/r/Rag/comments/1mkqjjq/my_experience_with_graphrag/>
- **Messy PDF extraction:** practitioners warn that entity extraction from
  regulatory/technical PDFs can produce noisy entities and relations unless the
  extraction schema and entity resolution are controlled.
  <https://www.reddit.com/r/LocalLLaMA/comments/1qreog0/how_to_create_a_knowledge_graph_from_100s_of/>
- **Obsidian-style automatic links:** users value local/offline operation and
  review-before-write. Successful projects combine deterministic title/alias
  matching with local embeddings, and let the user approve links rather than
  silently rewriting notes.
  <https://www.reddit.com/r/ObsidianMD/comments/1s46wxu/i_built_a_tool_that_automatically_adds_semantic/>
  <https://www.reddit.com/r/ObsidianMD/comments/1tnxv2q/plugin_showcase_semantic_autolinker_generate/>

The recurring signal supports the same design: deterministic structure first,
semantic links as proposals, human review before they affect content. For the
small local helper option (BM25 + model2vec-rs), see
[`lightweight-semantic-linking-research-2026-11-03.md`](lightweight-semantic-linking-research-2026-11-03.md)
and the selected implementation plan
[`plans/2026-11-03-semantic-link-helper.md`](plans/2026-11-03-semantic-link-helper.md).

## 9. Sources

- Microsoft GraphRAG indexing methods and outputs:
  <https://github.com/microsoft/graphrag/blob/main/docs/index/methods.md>
  <https://github.com/microsoft/graphrag/blob/main/docs/index/outputs.md>
- LightRAG:
  <https://github.com/hkuds/lightrag>
- Neo4j LLM Graph Builder backend:
  <https://github.com/neo4j-labs/llm-graph-builder/blob/main/docs/backend/backend_docs.adoc>
- Graphify:
  <https://github.com/graphify-labs/graphify>
  <https://github.com/graphify-labs/graphify/blob/v8/graphify/skills/agents/references/extraction-spec.md>
- PaperQA2:
  <https://github.com/future-house/paper-qa>
- RAPTOR:
  <https://github.com/parthsarthi03/raptor>
- iText2KG:
  <https://github.com/auvalab/itext2kg>
- txtai graph examples:
  <https://github.com/neuml/txtai>
- GraphRAG SDK:
  <https://github.com/falkordb/graphrag-sdk>
- TrustGraph provenance:
  <https://github.com/trustgraph-ai/trustgraph/blob/master/docs/tech-specs/extraction-time-provenance.md>
- WhyHow Knowledge Graph Studio:
  <https://github.com/whyhow-ai/knowledge-graph-studio>
- InfraNodus Graph View:
  <https://github.com/infranodus/infranodus-vscode-extension>
- RAGFlow GraphRAG deprecation:
  <https://github.com/infiniflow/ragflow/blob/main/docs/references/glossary.mdx>
- Docling provenance:
  <https://github.com/docling-project/docling/blob/main/docs/concepts/docling_document.md>
  <https://github.com/docling-project/docling/blob/main/models/stages/reading_order/readingorder_model.py>
