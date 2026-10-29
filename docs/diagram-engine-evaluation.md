# Diagram engine evaluation

- **Status:** Research / decision input
- **Date:** 2026-10-29
- **Scope:** layout and SVG generation for Folisift diagrams
- **Related:** `docs/plans/2026-10-29-production-hardening-and-visuals.md`,
  ADR-0007 (Typst renderer), ADR-0009 (workspace sandbox)

## Question

Should Folisift keep its own diagram layout and SVG renderer, or use an
existing diagram library? The answer must respect the product invariants:

- no shell, browser, Node, Java or network inside MCP;
- offline, reproducible builds;
- deterministic output for the same spec and engine version;
- accessible SVG (`role`, `<title>`, `<desc>`, `viewBox`, no scripts or
  external references);
- style-token-driven colours and typography;
- pedagogical limits (nodes, edges, levels, caption, provenance);
- a small, cross-platform install (binary + PDFium sidecar).

## Executive summary

Keep the typed Folisift DSL, validation, SVG accessibility layer and Typst
integration. Replace only the **layout algorithm** with the pure-Rust
`elk-layout` crate (Eclipse Layout Kernel port).

ELK gives us the part that is expensive to build correctly: layered graph
layout, orthogonal edge routing, crossing reduction, port/compound-node
support and tree layout. We keep the part that makes Folisift different:
semantic validation, visual budget, accessible SVG, deterministic hashing,
caption/provenance rules and style tokens.

Do **not** adopt Mermaid or PlantUML as the default. Do **not** make Graphviz or
D2 the default runtime. Keep Graphviz/D2 as optional CLI importers for authors
who already have DOT/D2 sources.

## Candidates

| Candidate | Form | Runtime | Layout quality | Diagram breadth | Determinism |
|---|---|---|---|---|---|
| Current Folisift engine | Rust | none | basic layered/tree/sequence; no orthogonal routing or crossing minimisation | 4 kinds | strong; own hash |
| ELK (`elk-layout`) | pure Rust crate | none | high; layered, orthogonal, ports, clusters, mrtree, radial, force, stress | general graphs/trees, not sequence/UML semantics | strong when version-pinned; JSON output |
| Graphviz (`dot`) | C library / CLI | libgvc or `dot` binary | very high, mature | DOT graphs; labels/records/clusters | layout can change across versions; pin the package |
| D2 | Go CLI | `d2` binary | good (dagre/ELK/TALA) | modern architecture/sequence/class-like diagrams | salt controls IDs; pin CLI version |
| d2-little | pure Rust crate | none | dagre port; young | D2 subset | claimed byte-identical port; low adoption |
| Mermaid | JS | Node 22.12+ + browser/Chromium for CLI | dagre/ELK via JS | very broad | historically not deterministic (date-based IDs); browser-dependent |
| PlantUML | Java | JVM (+ Graphviz for some diagrams) | good | UML-heavy | version/Java/Graphviz dependent |

## Evidence

### ELK pure Rust

`elk-layout` is a pure-Rust port of Eclipse Layout Kernel, published as
`0.12.1` (2026-10-23), licensed `EPL-2.0 OR GPL-3.0-or-later`. It exposes an
`elkjs`-compatible JSON API and returns laid-out node coordinates plus edge
`sections`/bend points. It has no system library dependency.

A local prototype with a five-node flow produced:

```text
elapsed_ms=41 deterministic=true
"src"   x=12  y=12 w=140 h=38
"lex"   x=172 y=12 w=150 h=38
"parse" x=342 y=12 w=150 h=38
"sem"   x=512 y=12 w=150 h=38
"code"  x=682 y=12 w=130 h=38
edge "e1" sections=[{"startPoint":{"x":152,"y":31},"endPoint":{"x":172,"y":31},...}]
```

The prototype used 115 normal dependency edges and produced a 15 MB unstripped
release binary for a hello-world program. That is not the marginal size in
Folisift (Typst and embedded fonts already dominate), but it is a real
dependency-footprint cost. It must be measured after integration and release
profile tuning.

Pros:

- no C, JVM, Node or browser;
- one Cargo dependency; cross-compiles like the rest of the workspace;
- far better layered layout than the current row-by-row placement;
- orthogonal edge routing and bend points are available;
- supports ports, clusters and multiple algorithms for future diagram kinds;
- exact version pinning makes regression testing possible.

Cons:

- 115 transitive crates and all ELK algorithms are compiled in; no obvious
  fine-grained feature flags;
- a small maintainer team (9 GitHub stars at the time of writing); young project;
- EPL-2.0 weak copyleft needs a licence notice and a policy for modifications;
- it returns geometry, not SVG, so accessibility and styling still belong to us;
- sequence diagrams are not an ELK concern; keep the custom sequence layout or
  use a dedicated engine.

### Graphviz

Graphviz is the reference implementation for DOT layout and remains the
quality benchmark. It is licensed EPL-2.0. It has a C library API and a CLI.
Its SVG output already gives each node/edge a `<title>`, but the upstream
accessibility request for graph-level `desc`/alt text is still open, so an
accessible wrapper still needs post-processing.

Integration options:

- **CLI** (`dot -Tsvg`): easy, but violates the no-shell rule if used from MCP;
  acceptable only as an author-time CLI importer.
- **C library via FFI** (`libgvc`, `wrappedviz`, custom bindings): keeps
  process isolation but adds a platform-specific native build, unsafe FFI,
  packaging and crash surface. The available wrapper crates are immature;
  `wrappedviz` docs.rs builds fail and `graphviz-rust` shells out to `dot`.

Graphviz is the best choice when the source is already DOT and the author
accepts an external tool. It is not the best default for a single-binary
offline product.

### D2

D2 has excellent authoring ergonomics and modern defaults. Its layout engines
are dagre (default), ELK and TALA. The official tool is a Go binary; the Rust
port `d2-little` is a large (≈53k lines) port with 3k downloads and 28%
documentation, so it is not yet a safe production dependency. D2 also has a
`salt` option for unique IDs, which means determinism depends on passing a fixed
salt and pinning the CLI/engine.

Use D2 as an optional author-time importer, not as the default MCP runtime.

### Mermaid

Mermaid is JavaScript and the CLI ultimately needs Node 22.12+ and a Chromium
instance via Puppeteer. It has good accessibility options (`accTitle`,
`accDescr`, ARIA), but the browser dependency is exactly what the product
architecture excludes. It has a history of non-deterministic IDs (date-based
IDs in issue #727), and some renderers emit `foreignObject`/HTML that is not
portable to a PDF/SVG pipeline without cleanup.

Mermaid is attractive for authors who already write it. It is the wrong
default for Folisift.

### PlantUML

PlantUML is GPL and needs a JVM; some diagram types also use Graphviz. It is
strong for UML, but the runtime and licence are poor fits for the default
engine. Optional import only, if ever.

## Weighted decision

Weights reflect the product invariants, not general popularity.

| Criterion | Weight | Current | ELK + our renderer | Graphviz | D2 | Mermaid |
|---|---:|---:|---:|---:|---:|---:|
| Offline / no extra runtime | 20 | 5 | 5 | 3 | 2 | 1 |
| Layout quality | 20 | 2 | 5 | 5 | 4 | 4 |
| Determinism / reproducibility | 15 | 5 | 4 | 3 | 3 | 2 |
| Accessibility control | 15 | 5 | 5 | 3 | 3 | 4 |
| Product/pedagogy rules | 10 | 5 | 5 | 2 | 2 | 2 |
| Install/packaging | 10 | 5 | 4 | 2 | 2 | 1 |
| Maintenance risk | 5 | 3 | 3 | 5 | 4 | 4 |
| Licence fit | 5 | 5 | 4 | 4 | 4 | 4 |
| **Weighted total** | 100 | **4.30** | **4.60** | **3.35** | **2.90** | **2.60** |

Scores are 1–5. The current engine wins on control and packaging; ELK wins on
the expensive layout work while preserving control.

## Recommended architecture

```text
guide Markdown
  └─ ```diagram JSON spec
       ├─ Folisift validation: kinds, limits, labels, tree root, provenance
       ├─ layout backend:
       │    ├─ ELK layered  → concept / flow
       │    ├─ ELK mrtree   → tree
       │    └─ custom       → sequence
       ├─ Folisift SVG renderer: theme, arrows, labels, role/title/desc
       ├─ blake3 asset hash + cache
       └─ Typst figure with caption, alt text, provenance
```

Rules:

- Pin `elk-layout` to an exact version; a version bump is a visual-regression
  change and must update baselines.
- Keep the custom layout behind a feature flag for one release so the old and
  new SVG can be compared page-by-page.
- Add a licence notice for EPL-2.0 and document that Folisift does not modify
  ELK.
- Keep Graphviz/D2 import as CLI-only, behind an explicit command, with the
  same SVG sanitizer and no MCP shell.
- Keep the 12-node/20-edge limit. ELK can lay out more, but the pedagogical
  budget is a product rule, not an engine limit.

## Migration plan

1. Add `elk-layout = "=0.12.1"` behind a `layout-elk` feature.
2. Convert `DiagramSpec` to ELK JSON:
   `layered` for flow/concept, `mrtree` for tree, direction and spacing from
   style tokens.
3. Read node `x/y/width/height` and edge `sections`/`bendPoints`.
4. Render SVG paths with our existing theme, labels and accessibility elements.
5. Keep the current layout as `layout-legacy` for one release.
6. Add golden SVG tests for all four kinds, deterministic double-run tests and
   visual baselines for both styles.
7. Measure binary size, cold layout time and PDF page count; remove the legacy
   layout only after baselines pass.

Estimated integration effort: 2–4 engineering days, plus 1–2 days of visual
review. A full in-house replacement of ELK-class layout would be weeks to
months and would still be weaker on crossing minimisation and edge routing.

## Sources

- ELK Rust crate and licence:
  <https://docs.rs/elk-layout/latest/elk_layout/>
- ELK Rust repository and validation policy:
  <https://github.com/openedges/elk-rs>
- ELK project:
  <https://eclipse.dev/elk/>
- ELK layered algorithm:
  <https://eclipse.dev/elk/reference/algorithms/org-eclipse-elk-layered.html>
- Graphviz licence:
  <https://graphviz.org/license/>
- Graphviz SVG output:
  <https://graphviz.org/docs/outputs/svg/>
- Graphviz C library API:
  <https://graphviz.org/docs/library/>
- Graphviz accessibility issue:
  <https://gitlab.com/graphviz/graphviz/-/issues/2220>
- Graphviz Rust CLI wrapper:
  <https://docs.rs/graphviz-rust/latest/graphviz_rust/>
- Graphviz C wrapper:
  <https://docs.rs/wrappedviz/latest/wrappedviz/>
- D2 layouts:
  <https://d2lang.com/tour/layouts/>
- D2 ELK layout:
  <https://d2lang.com/tour/elk/>
- D2 licence:
  <https://github.com/d2lang/d2>
- d2-little crate:
  <https://docs.rs/d2-little/latest/d2_little/>
- Mermaid usage and runtime:
  <https://mermaid.js.org/config/usage.html>
- Mermaid accessibility:
  <https://mermaid.js.org/config/accessibility.html>
- Mermaid non-deterministic IDs:
  <https://github.com/mermaid-js/mermaid/issues/727>
- Mermaid CLI Chromium dependency:
  <https://github.com/mermaid-js/mermaid-cli/blob/master/docs/already-installed-chromium.md>
- PlantUML licence:
  <https://plantuml.com/faq>
- libavoid orthogonal routing (ELK integration):
  <https://eclipse.dev/elk/blog/posts/2022/22-11-17-libavoid.html>
