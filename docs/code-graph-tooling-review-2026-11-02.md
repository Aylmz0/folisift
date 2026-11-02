# Code-graph tooling review — 2026-11-02

- **Question:** Should Folisift add Graphify or a similar code-graph system?
- **Status:** Evaluation only; no runtime dependency added
- **Decision:** Use a code graph, if at all, as an optional developer tool. Do not ship it in the Folisift binary, MCP server, release archive or build gates.
- **Evaluated:** Graphify 0.9.97 on the Folisift worktree at `90dcb2b`

## 1. Two different ideas

A code graph can mean two unrelated things for this project:

1. **Developer tooling:** an index of the Folisift Rust workspace that helps an
   agent navigate crates, types, functions and docs while changing the engine.
2. **Product feature:** a graph of source-document concepts inside a study
   guide, helping a learner see relationships across PDFs.

Graphify addresses the first. It does not address the second. Folisift already
has deterministic source inventory, `source_scope` coverage, structures and D2
concept diagrams for the second problem.

This distinction matters because the product has a hard boundary: the MCP
server performs no LLM calls, shell execution or network access. A developer
tool can sit beside that boundary; a runtime graph extractor cannot.

## 2. What Graphify is

Graphify is a Python 3.10+ CLI and AI-assistant skill. It writes
`graphify-out/graph.json`, `GRAPH_REPORT.md` and an HTML graph. The graph has
typed nodes and edges; edges are marked `EXTRACTED`, `INFERRED` or
`AMBIGUOUS` so a reader can distinguish direct evidence from inference.

For code, it parses files locally with Tree-sitter and does not call an LLM.
The semantic pass is reserved for documents, papers, images and transcripts and
may send content to a configured model. Its security model describes it as a
local development tool; MCP over stdio is optional, and an HTTP listener is
opt-in.

Relevant commands:

```bash
graphify extract . --code-only --no-cluster --cargo
graphify query "how does the build service enforce coverage?"
graphify path "A" "B"
graphify explain "GuideConfig"
graphify update .          # incremental code update, no LLM
graphify watch .           # local rebuild on changes
```

Installation is `uv tool install graphifyy` (or `pipx`/`pip`). The official
repository is Apache-2.0; the project website still says MIT in one place, so
the repository licence file is the authority. The documentation also disagrees
on the language count (19, 25 and ~40 appear in different pages); Rust support
is present, but the exact count should not be treated as a contract.

## 3. Empirical test on Folisift

The test ran in a throwaway worktree, not in the repository:

```bash
graphify extract . --code-only --no-cluster --cargo \
  --out /var/tmp/graphify-folisift
```

Result:

| Metric | Value |
|---|---:|
| Wall time | 1.3 s |
| Code files found | 188 |
| Nodes | 1,231 |
| Edges | 2,645 |
| `graph.json` | 944 KB |
| AST cache | 2.4 MB |
| Corpus size | ~61,550 words / ~82k tokens |
| Benchmark token reduction | 44.9× per query |

The most connected nodes were plausible architectural hubs:

```text
DiagramSpec, Workspace, render(), FolisiftServer, StyleProfile,
Inventory, Command, check_coverage(), markdown_to_typst(),
Guide, SourceInventory
```

A query about coverage and page budgets returned the expected files and
symbols (`build()`, `Build`, `check_coverage()`, `CoverageReport`,
`folisift-build/src/lib.rs`, `folisift-parse/src/coverage.rs`). The graph also
captured the six Cargo crates and their dependency edges.

Limits observed:

- The graph is AST-shaped, not compiler-shaped. Rust macros, trait resolution,
  generic instantiations and generated code can be incomplete.
- Ambiguous names require path-qualified queries. `explain "build()"` matched
  five different nodes and had to be disambiguated.
- `path "Folisi" "BuildRequest"` found no directed path; the query was too
  vague, not necessarily a graph failure.
- The extractor skipped 10 style `tokens.json` files as potentially sensitive
  and reported eight code files with no symbols, including some `lib.rs` and
  `Cargo.toml` files.
- Query output is traversal context, not a verified answer. The 44.9× token
  figure measures context size, not correctness.

## 4. Fit with Folisift

### What it would help

- Onboarding to the six-crate workspace.
- Finding the call path from CLI/MCP to `folisift-build`, `folisift-parse`,
  `folisift-render` and `folisift-core`.
- Architecture questions where a list of related symbols is more useful than a
  full-file grep.
- Reviewing the blast radius of a type or function change before editing.

### What it would not fix

- It does not make guide content more accurate.
- It does not replace `source_scope`, coverage reports or visual audits.
- It does not validate claims or page references.
- It does not run inside the MCP server without violating the no-shell,
  no-network, no-LLM contract.
- It does not remove the need for `cargo check`, Clippy, tests and PDF gates.

### Cost

If kept as a developer tool, the cost is a Python/uv toolchain, a pinned
Graphify version, an optional OpenCode integration and a stale-graph update
step. The graph artifacts should be gitignored and regenerated after refactors.

If shipped, the cost is much higher: Python runtime packaging, licence and
notice handling, a second graph schema, a refresh policy, network/LLM policy
for semantic extraction, release-archive growth and another failure mode in
`doctor`/CI. That is not justified by the current evidence.

## 5. Alternatives

| Option | Strength | Cost / limitation |
|---|---|---|
| **Graphify** | Fast local AST graph, Rust support, query/path/explain, optional OpenCode skill | Python tool; AST-only; docs disagree on scope; semantic pass can call an LLM |
| **SCIP + a Rust indexer** | Precise symbol/navigation protocol; language-agnostic | Needs an indexer per language and a consumer; not a ready-made agent graph |
| **CodeGraph Rust** | Rust-native indexer, MCP tools, SurrealDB graph, LSP/SCIP and embeddings | SurrealDB, embeddings and a much larger operational surface |
| **CodeGraphContext** | MCP server, 20 languages, graph DB, SCIP option | External graph database and Python stack; heavy for a six-crate repo |
| **Tree-sitter directly** | Minimal parser dependency; full control | We would build symbol extraction, edges, storage, queries and freshness ourselves |
| **`cargo metadata` + rust-analyzer** | Exact Rust workspace and symbol facts | Not a graph UI; narrower than a cross-language knowledge graph |

SCIP is a protocol, not an agent UI. CodeGraph Rust and CodeGraphContext are
closer to complete systems, but both add a database and a larger dependency
surface than Graphify. For Folisift's current size, Graphify is the cheapest
experiment; it is not automatically the best long-term architecture.

## 6. Recommendation

**Do not add Graphify as a product dependency. Trial it as an optional
developer tool, outside the release boundary.**

A safe trial:

1. Run it in a worktree or a temporary copy.
2. Use `--code-only`; do not send `sources/` or guide PDFs through a semantic
   pass.
3. Keep `graphify-out/` gitignored.
4. Do not run `graphify opencode install` until the manual CLI proves useful;
   that command writes `AGENTS.md` and an OpenCode plugin.
5. Pin the CLI version and record the Python/uv requirement.
6. Measure five real architecture questions against `rg`/`cargo metadata`:
   correctness, files opened, time and stale-graph failures.
7. Revisit only if the repo grows enough that grep and direct reading stop
   scaling.

Adoption criteria:

- fewer wrong-file reads or faster onboarding on real tasks;
- graph freshness is maintainable after refactors;
- Rust relationships are accurate enough for the questions asked;
- no source PDFs or private notes leave the machine;
- the tool remains outside the `folisift` binary, MCP server and release
  archive.

## 7. Sources

- Graphify repository and README:
  <https://github.com/graphify-labs/graphify>
- Graphify security model:
  <https://github.com/graphify-labs/graphify/blob/v8/SECURITY.md>
- Graphify extraction dispatch and fallback:
  <https://github.com/graphify-labs/graphify/blob/v8/graphify/extract.py>
- Graphify package metadata (Python >=3.10):
  <https://github.com/graphify-labs/graphify/blob/v8/pyproject.toml>
- Graphify licence:
  <https://github.com/graphify-labs/graphify/blob/v8/LICENSE>
- Graphify documentation site:
  <https://graphify.net/>
- CodeGraph Rust:
  <https://github.com/jakedismo/codegraph-rust>
- CodeGraphContext architecture:
  <https://github.com/codegraphcontext/codegraphcontext/blob/main/docs/ARCHITECTURE.md>
- SCIP:
  <https://github.com/scip-code/scip>
- Tree-sitter:
  <https://github.com/tree-sitter/tree-sitter>

The empirical numbers in section 3 come from the local Graphify 0.9.97 run
recorded in this review; they are not a vendor benchmark.
