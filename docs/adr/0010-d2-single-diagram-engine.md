# ADR-0010: Use D2 as the single external diagram engine

- **Status:** Accepted
- **Date:** 2026-10-29
- **Deciders:** Maintainer
- **Related:** ADR-0007, ADR-0009, `docs/diagram-engine-evaluation.md`,
  `docs/plans/2026-10-29-d2-pipeline-and-opencode-automation.md`

## Context

Folisift needs flow, concept, tree and sequence diagrams in a paper-like PDF.
The first implementation laid out JSON diagrams in Rust. That is not a good
use of engine effort: graph layout, edge routing, crossing minimisation and
compound containers are mature external problems.

The alternatives were compared in `docs/diagram-engine-evaluation.md`:

- D2: 25k stars, active releases, one Go binary, sequence/containers, TALA;
- Graphviz: strongest graph layout history, but a C/system dependency and no
  native sequence diagrams;
- Mermaid/PlantUML: Node/Chromium or JVM;
- ELK-rs: a 9-star port created in 2026, too young for production;
- `diagrams`: Python cloud-icon DSL over Graphviz, too narrow.

The product constraint is “any course, any source, one engine; every guide is
reproducible and auditable”. Using Graphviz would require a second engine for
sequence diagrams. D2 covers the whole diagram range with one pinned binary.

## Decision

1. **D2 v0.9.0 is the only external diagram engine.**
2. The host LLM writes a typed JSON spec; Folisift generates deterministic D2
   source. The host never writes D2, SVG or Typst.
3. D2 runs only in `folisift diagram build` (CLI). It never runs inside the MCP
   server or `folisift build`.
4. The guide build embeds committed, sanitised SVG plus a manifest.
5. The native diagram layout is removed from the user-facing path.
6. D2 is pinned by exact version, salt, layout, fonts and post-processor
   version. A mismatch is an error, never a fallback.
7. D2 is distributed unmodified under MPL-2.0 with a notice.

## Consequences

- `guides/<slug>/diagrams/*.json` becomes the diagram source of truth.
- `data/d2/*.d2` is generated and audited; `data/diagrams/*.json` records
  hashes and engine versions.
- `assets/generated/<hash>.svg` is committed; a fresh build does not need D2.
- The MCP gains one `diagram` tool and schema version 2.
- OpenCode runs one extra CLI step between saving specs and building the guide.
- Graphviz and the native layout code are not runtime fallbacks.
- D2’s SVG needs accessibility and edge-label post-processing; Folisift owns
  that layer.

## Alternatives considered

- **Graphviz:** rejected as the single engine because it cannot cover sequence
  diagrams and adds a C/system dependency.
- **ELK-rs:** rejected for maturity; ELK Java/JS would add a runtime.
- **Mermaid:** rejected for Node/Chromium and non-deterministic history.
- **PlantUML:** rejected for JVM/GPL/LGPL runtime.
- **Python `diagrams`:** rejected as a cloud-specific Graphviz wrapper.
- **Continue in-house:** rejected; the layout problem is not the product.

## Compliance

- `docs/authoring.md`, `docs/architecture.md` and `AGENTS.md` state the D2-only
  boundary.
- `docs/agents/opencode-guide.md` defines the host loop and allowlist.
- `docs/plans/2026-10-29-d2-pipeline-and-opencode-automation.md` defines the
  implementation and acceptance criteria.
