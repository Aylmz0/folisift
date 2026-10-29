# Diagram engine evaluation

- **Status:** Decision input
- **Date:** 2026-10-29
- **Scope:** layout and SVG generation for Folisift diagrams
- **Related:** ADR-0007 (Typst renderer), ADR-0009 (workspace sandbox),
  `docs/plans/2026-10-29-production-hardening-and-visuals.md`

## Decision in one page

Use **Graphviz `dot` as the primary external layout engine at authoring time**.
Keep the Folisift JSON spec, caption/provenance rules and sanitizer. Keep the
native SVG renderer for sequence diagrams and as a fallback. Add **D2** as an
optional CLI importer for architecture/container diagrams. Do **not** adopt
ELK-rs (9 stars, created 2026), Mermaid, PlantUML or Kroki as the default
runtime.

The external engine runs **only in the CLI**, never in the MCP server. The
guide build and the MCP server consume a sanitized, hashed SVG asset. This
preserves the no-shell/no-network rule while gaining a mature layout engine.

## Why this changed

The previous recommendation was pure-Rust `elk-layout`. That was rejected on
maintenance grounds: the Rust port is new (repository created February 2026),
has 9 stars and is not a proven dependency for a product that claims
reliability. Eclipse ELK itself is mature, but using the Java or JavaScript
implementation would add a JVM or Node/Chromium runtime, which conflicts with
the offline single-binary architecture.

Writing a Graphviz-class layout engine in-house is also not rational. Layered
layout, crossing minimisation, port constraints, orthogonal routing and
compound graphs are research-grade problems. The sensible boundary is:

```text
mature external layout  →  Folisift sanitiser/accessibility  →  Typst embed
```

## Candidates and current evidence

Metrics collected on 2026-10-29.

| Project | Adoption signal | Runtime | Licence | Fit |
|---|---|---|---|---|
| **Graphviz 16.1.0** | 40+ years; distro package; GitLab 1,475 stars / 438 forks; last commit 2026-10-29 | C library or `dot` CLI | EPL-2.0 | **Primary layout** |
| **D2 v0.9.0** | GitHub 25,755 stars / 775 forks; 177,765 Linux downloads for v0.9.0; last commit 2026-10-11 | Go single binary | MPL-2.0 | **Optional importer** |
| Mermaid 12.2.0 | GitHub 90,920 stars / 9,424 forks; 2,010 open issues; last commit 2026-10-28 | Node 22.12+ + Chromium for CLI | MIT | Import only; no default |
| PlantUML 1.2026.9 | GitHub 13,385 stars / 1,248 forks; last commit 2026-10-28 | JVM; some diagrams need Graphviz | LGPL-3.0 | Not a fit |
| ELK-rs 0.12.1 | GitHub 9 stars / 4 forks; created 2026-02-10 | pure Rust | EPL-2.0 / GPL-3.0 | **Rejected: too young** |
| d2-little | 3,087 downloads; 28% documented | pure Rust | MPL-2.0 | Rejected: immature port |
| diagrams (mingrammer) | GitHub 42,763 stars / 2,732 forks; last commit 2026-10-15; latest release v0.25.1 on 2025-11-22 | Python 3.9+ + Graphviz | MIT | Cloud-only; not a general engine |
| Kroki | 4,388 stars | server/Docker, multiple tools | MIT | Rejected: service dependency |
| Current Folisift | in-tree | none | in-house | Keep for sequence/fallback |

Stars are not a quality metric on their own. Graphviz's 1,475 GitLab stars
understate its adoption: it is packaged by every major Linux distribution,
used by documentation systems and has been the reference DOT implementation
for decades. For comparison, D2 has 25k stars, but its tested edge/sequence
labels overlap the connector line and its SVG has no accessibility metadata;
both need our post-processing, and Graphviz wins on edge-label readability. D2's 25k stars reflect a modern, active authoring tool; its
single Go binary is a much better runtime fit than Node/Java.

## Activity snapshot

| Project | Last commit | Latest release | Release age at 2026-10-29 |
|---|---|---|---|
| Graphviz | 2026-10-29 | 16.1.0 (2026-09-04) | ~8 weeks |
| D2 | 2026-10-11 | v0.9.0 (2026-09-07) | ~7 weeks |
| Mermaid | 2026-10-28 | 12.2.0 (2026-10-13) | ~2 weeks |
| PlantUML | 2026-10-28 | 1.2026.9 (2026-10) | current month |
| diagrams | 2026-10-15 | v0.25.1 (2025-11-22) | ~11 months |
| ELK-rs | 2026-10-23 | 0.12.1 (2026-10-23) | current, but 9 stars |
| Structurizr CLI | 2026-02-01 | archived | not active |
| Excalidraw | 2026-10-28 | active | manual editor |
| draw.io | 2026-10-23 | active | manual editor |

A project can have recent commits but a stale release (diagrams), or a recent
release but a young codebase (ELK-rs). Both signals matter. For production we
require **recent commits, a recent tagged release, a stable licence and a
runtime we can actually ship**.

## Local evidence

### Graphviz

Same DOT input, five runs, locally installed Graphviz 14.1.4. The current
stable release at the time of research is 16.1.0; CI/release must pin 16.1.0
and repeat this test, because layout can change between major versions.

```text
327c7738bf50debd0d88828394cab1d0d0205da3a2c5576fdd8655fc808f2bf8
327c7738bf50debd0d88828394cab1d0d0205da3a2c5576fdd8655fc808f2bf8
327c7738bf50debd0d88828394cab1d0d0205da3a2c5576fdd8655fc808f2bf8
327c7738bf50debd0d88828394cab1d0d0205da3a2c5576fdd8655fc808f2bf8
327c7738bf50debd0d88828394cab1d0d0205da3a2c5576fdd8655fc808f2bf8
```

A styled five-node flow produced a compact `869 × 50` viewBox, 14 pt node
labels and 11 pt edge labels. At the 510 pt content width used by the paper
template, node text renders at about 8.2 pt and edge text at about 6.5 pt:
inside the 6.5 pt minimum and near the 7.5 pt target. Graphviz emits a
`<title>` for each node and edge, but not a graph-level `<desc>` or ARIA role;
we add those in post-processing.

Graphviz strengths:

- reference-quality layered/branching layout and crossing minimisation;
- deterministic for a fixed input and version in our test;
- compact output, controllable spacing, fonts, colours and shapes;
- SVG/PDF/JSON outputs;
- EPL-2.0; no browser/Node/JVM.

Graphviz costs:

- system package or bundled native binary; not a Rust crate;
- version must be pinned; layout can change between releases;
- no graph-level accessible description; needs our accessibility wrapper;
- `graphviz-rust` only shells out to `dot`;
- `wrappedviz`/C FFI is immature and adds unsafe native packaging.

### D2

Two runs with `--salt fixed` produced byte-identical SVG. A five-node flow
with `--pad 20` produced a `1486 × 108` viewBox; at 510 pt width the 20 px
labels become roughly 6.9 pt. D2 v0.9.0 has no `<title>`, `<desc>` or ARIA
attributes in the tested output and embeds base64 WOFF fonts.

D2 strengths:

- 25k stars, active releases, single 18 MB binary;
- modern architecture/container/sequence diagrams;
- dagre/ELK/TALA layouts, themes, sketch mode;
- deterministic with a fixed salt and pinned version;
- SVG/PNG/PDF/PPTX output with no browser.

D2 costs:

- in the tested output, edge and sequence labels sit on the connector line, so
  the line crosses the text; Graphviz places labels beside the edge;
- no accessibility metadata; we must add it;
- wide diagrams have a large canvas and small relative text unless spacing is
  tuned or the graph is split;
- extra 18 MB binary per platform;
- MPL-2.0 file-level copyleft (notice/relink obligations for modifications);
- less control over fine-grained spacing than DOT attributes.

### Diagrams (mingrammer)

`diagrams` is the project the user found. It is a **Python DSL for cloud
architecture diagrams**, not a general layout engine. It is healthy and active
on the icon side: 42,763 stars, 2,732 forks, last commit 2026-10-15. The last
PyPI release is v0.25.1 from 2025-11-22, so release cadence is slower than the
commit cadence. It has 400 open issues.

It requires Python 3.9+ and Graphviz, and its documentation explicitly says it
uses Graphviz to render. Its value is the provider icon catalog (AWS, Azure,
GCP, Kubernetes, on-prem, C4, programming). It does not provide sequence
diagrams, concept maps, general flowcharts or accessibility metadata, and its
layout control is limited.

Reddit signals are consistent: users like that a presentable cloud diagram is
possible in minutes, but describe the library as tricky to learn and limited to
infrastructure work:
<https://www.reddit.com/r/aws/comments/gjxil2/diagrams_as_code_python_with_aws_icon_support/>
<https://www.reddit.com/r/devops/comments/1o4lbp8/diagram_tools/>

Verdict: **not a core dependency**. It is a domain-specific layer over
Graphviz. If we ever need cloud-architecture icons, it can be an optional
authoring importer, but adopting Python would violate the single-binary/no
extra runtime boundary for no gain on pedagogical diagrams.

### Mermaid

Mermaid is the most starred project, but its CLI needs Node 22.12+ and a
Chromium instance through Puppeteer. It has a history of non-deterministic
IDs (date-based IDs, issue #727) and some renderers emit `foreignObject`/HTML.
It is excellent for Markdown-native docs, but not for a no-browser build
pipeline.

### PlantUML and Kroki

PlantUML requires a JVM and is LGPL-3.0; some diagram types delegate layout to
Graphviz. Kroki is a server that wraps many diagram tools, which adds a
service/network dependency. Neither is a default candidate.

### ELK-rs

The Rust port is technically promising, but a 9-star repository created in
February 2026 is not the dependency to bet a production documentation system
on. If ELK layout is ever needed, use the mature Java/JS ELK behind an
authoring service, not the young port.

### Screened out

| Tool | Why not the default |
|---|---|
| Structurizr CLI/Lite | GitHub repositories are archived (last push 2026-02-01); C4 model is a different product scope |
| Excalidraw | 134k stars, but a manual whiteboard; no deterministic code-to-diagram layout |
| draw.io | 9k stars, manual editor; XML/HTML model, not a layout library |
| nomnoml | JavaScript, smaller ecosystem; browser/JS runtime |
| Pikchr | deterministic and tiny, but manual coordinates; no automatic graph layout |
| InfraSketch | commercial/natural-language service; not offline or deterministic |

## What users say

These are qualitative signals, not benchmarks:

- D2 users praise the output and the speed of authoring compared with
  drag-and-drop tools; D2 is repeatedly described as easier to read/write than
  Mermaid for architecture diagrams.
  <https://www.reddit.com/r/selfhosted/comments/1gin6ut/holy_crap_d2_diagrams_are_impressive/>
- In a D2 discussion, users note that unlike Mermaid, D2 lets you influence
  layout and that Mermaid flowcharts can get ugly.
  <https://www.reddit.com/r/programming/comments/1qqizxa/your_ai_diagram_looks_great_and_nobody_will_read/>
- Mermaid complaints focus on wide diagrams, layout quality, flashing/resizing
  and themes; it remains popular because it is embedded in Markdown tools.
  <https://www.reddit.com/r/Anytype/comments/1eby9xr/does_mermaid_really_need_to_be_that_horrible/>
  <https://www.reddit.com/r/dotnet/comments/18cwzda/what_is_your_favorite_tool_for_designing_complex/>
- Graphviz is repeatedly recommended for maintainable, text-based diagrams,
  especially network/architecture documentation, with the trade-off of a
  dated default look that must be styled.
  <https://www.reddit.com/r/networking/comments/8dbucs/tools_to_create_maintainable_network_diagrams/>
  <https://www.reddit.com/r/rust/comments/5ljshj/diagrams_in_documentation/>

The recurring user requirement is not “the most beautiful default”; it is
“readable at a glance, reproducible, and cheap to update”. That favours
Graphviz for graph-shaped study diagrams and D2 for modern architecture
figures.

## Recommended architecture

```text
Host LLM / author
  └─ Folisift diagram spec (JSON)  or  existing .dot/.d2 file
       └─ CLI: folisift diagram build
            ├─ Graphviz dot (pinned)  → SVG
            ├─ native sequence layout → SVG
            └─ D2 (optional)          → SVG
                 ↓
            sanitize + accessibility wrapper
            (role, title, desc, viewBox, no scripts/external refs)
                 ↓
            blake3 hash + engine manifest
            assets/generated/<hash>.svg
                 ↓
            guide build / MCP: embed-only, no external process
```

### Boundaries

- MCP never invokes `dot`, `d2`, Node or a browser. It only reads assets.
- The CLI checks the engine version and records it in
  `assets/generated/manifest.json`.
- The graph spec remains engine-neutral. `engine: "graphviz"` and
  `engine: "d2"` select the authoring backend; `native` remains the fallback.
- Every imported SVG passes the existing sanitizer and accessibility wrapper.
- A Graphviz/D2 failure is an authoring error with the exact command and
  version, not a silent fallback.
- The native sequence layout stays because sequence diagrams are not a natural
  DOT graph; D2 can be used when a user explicitly imports one.

### Why Graphviz first, not D2

- Graphviz has the strongest graph-layout algorithms and decades of use.
- It gives compact output with controllable font size; D2's wide canvas can
  fall below the 6.5 pt minimum after fitting.
- It is EPL-2.0 and available as a package on Linux/macOS/Windows.
- D2 is a better authoring tool for architecture and sequence diagrams, so it
  stays as an optional importer rather than the default.

## Migration plan

1. Add a CLI `diagram build` that converts a `DiagramSpec` to styled DOT.
2. Pin Graphviz 14.x and run it as a subprocess **outside MCP**.
3. Post-process SVG: add root `role="img"`, `<title>`, `<desc>`,
   `aria-labelledby`, strip external references/scripts.
4. Hash the sanitized SVG and write a manifest with engine, version, font and
   input hash.
5. Make build/MCP reject an inline spec unless a pre-generated asset exists.
6. Add golden SVG tests, five-run determinism tests and paper/clean visual
   baselines.
7. Add an optional `diagram import --engine d2` for `.d2` files, with the same
   sanitizer and manifest.
8. Keep the current native renderer for sequence and as a one-release fallback.

Estimated effort: 3–5 days for the Graphviz path, 2 days for the D2 importer,
plus visual review.

## Sources

- Graphviz licence: <https://graphviz.org/license/>
- Graphviz SVG output: <https://graphviz.org/docs/outputs/svg/>
- Graphviz library API: <https://graphviz.org/docs/library/>
- Graphviz accessibility issue: <https://gitlab.com/graphviz/graphviz/-/issues/2220>
- Graphviz GitLab project: <https://gitlab.com/graphviz/graphviz>
- graphviz-rust (CLI wrapper): <https://docs.rs/graphviz-rust/latest/graphviz_rust/>
- wrappedviz (C wrapper): <https://docs.rs/wrappedviz/latest/wrappedviz/>
- D2 repository: <https://github.com/d2lang/d2>
- D2 layouts: <https://d2lang.com/tour/layouts/>
- D2 ELK layout: <https://d2lang.com/tour/elk/>
- D2 CLI flags: <https://d2lang.com/tour/man/>
- D2 release assets and checksums:
  <https://github.com/d2lang/d2/releases/tag/v0.9.0>
- d2-little: <https://docs.rs/d2-little/latest/d2_little/>
- diagrams repository: <https://github.com/mingrammer/diagrams>
- diagrams docs/requirements: <https://diagrams.mingrammer.com/docs/getting-started/installation>
- diagrams release v0.25.1: <https://github.com/mingrammer/diagrams/releases/tag/v0.25.1>
- Mermaid usage/runtime: <https://mermaid.js.org/config/usage.html>
- Mermaid accessibility: <https://mermaid.js.org/config/accessibility.html>
- Mermaid non-deterministic IDs: <https://github.com/mermaid-js/mermaid/issues/727>
- Mermaid CLI Chromium dependency:
  <https://github.com/mermaid-js/mermaid-cli/blob/master/docs/already-installed-chromium.md>
- PlantUML repository/licence: <https://github.com/plantuml/plantuml>
- ELK-rs repository: <https://github.com/openedges/elk-rs>
- ELK project: <https://eclipse.dev/elk/>
- Kroki: <https://github.com/yuzutech/kroki>
