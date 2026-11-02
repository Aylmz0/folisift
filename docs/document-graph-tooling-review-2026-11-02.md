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

## 2. Ready systems

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

### Docling

Docling is a PDF/document parser, not a graph system. Its useful contribution
is provenance: every `DocItem` carries a `ProvenanceItem` with `page_no`,
bounding box and character span, and the body tree preserves reading order.
Tables, pictures, captions and formulas are typed items.

Fit: the best parser-side foundation if we ever build our own page-anchored
graph. Folisift already uses liteparse, so this would be a replacement or a
second parser, not a small addition.

## 3. Page-provenance comparison

| System | Graph? | Page provenance | LLM needed? | Fit for Folisift |
|---|---|---|---|---|
| **GraphRAG** | Yes | Text units carry `document_id`, not page by default | Yes for standard extraction | Global themes; needs page-aware chunking |
| **LightRAG** | Yes | File path + chunk content, no page contract | Yes | Incremental graph; page metadata DIY |
| **Neo4j LLM Graph Builder** | Yes | PDF chunk metadata can carry `page` | Yes | Closest page-aware ready system; Neo4j + LLM |
| **Graphify** | Yes | `source_file`; `source_location` often null | Yes for PDFs/docs | Quick concept graph; not citation-safe |
| **PaperQA2** | No | Page-level citations | Yes for answers | Research QA, not graph |
| **RAPTOR** | Tree | Chunk hierarchy, not page graph | Yes for summaries | Multi-level summary, not relations |
| **Docling** | No | Exact page/bbox/charspan | No (parser) | Best provenance source for a custom graph |

The pattern is clear: the systems that build semantic graphs do not guarantee
page-level provenance, and the system with the best provenance is not a graph.

## 4. What a Folisift-native graph could look like

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

## 5. Build vs buy

**Buy now:** GraphRAG, LightRAG, Neo4j LLM Graph Builder or Graphify would each
add a Python/LLM/graph stack before we know whether the graph improves the
guide. For a three-PDF course, that is a large bet.

**Build a thin layer:** liteparse already gives page text, headings and a page
map. A deterministic structural graph is a small Rust addition. The semantic
edge layer can be a host-authored JSON file at first, with no new runtime
dependency.

**Adopt a ready system when:** the corpus grows to many courses/documents, the
structural graph is insufficient, and a page-aware extraction policy is
defined. At that point Neo4j LLM Graph Builder or GraphRAG with page-level
chunks is a reasonable experiment.

## 6. Recommendation

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
6. **Promote to a product feature only if the experiment wins.** The first
   useful output is likely a `data/document-graph.md` or JSON sidecar, not a
   new MCP tool.

The user's idea is sound at the **authoring intelligence** layer. The mistake
would be treating a generic graph extractor as the source of truth. Folisift's
advantage is page-level auditability; the graph must inherit that, not bypass
it.

## 7. Sources

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
- Docling provenance:
  <https://github.com/docling-project/docling/blob/main/docs/concepts/docling_document.md>
  <https://github.com/docling-project/docling/blob/main/models/stages/reading_order/readingorder_model.py>
