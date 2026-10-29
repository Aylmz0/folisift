# Diagram engine decision

- **Status:** Final decision
- **Date:** 2026-10-29
- **Scope:** diagram layout and SVG generation for Folisift
- **Related:** ADR-0007 (Typst renderer), ADR-0009 (workspace sandbox),
  `docs/plans/2026-10-29-production-hardening-and-visuals.md`

## Decision

Use **D2 v0.9.0 as the single external diagram engine**.

There is no second external backend. There is no Graphviz fallback. The native
Rust code keeps only the typed JSON contract, validation, D2 source generation,
SVG sanitisation/accessibility post-processing, hashing and the Typst
integration. The native layout code is retired as a user-facing backend.

```text
typed JSON spec
  → deterministic D2 source
  → d2 --layout=tala --salt folisift --pad 20
  → SVG post-processing
  → sanitise + role/title/desc + edge-label background
  → blake3 hash + engine manifest
  → Typst embed
```

Graphviz is not used. `diagrams`, Mermaid, PlantUML, Structurizr and Kroki are
not used.

## Why D2, not Graphviz

The project motto is:

> any course, any source, one engine; every guide is reproducible and auditable.

D2 fits the “one engine” part better than Graphviz because it covers the whole
diagram range we need:

| Need | D2 | Graphviz |
|---|---|---|
| Flow / pipeline | yes | yes |
| Concept map | yes | yes |
| Tree / hierarchy | yes | yes |
| Sequence diagram | **first-class** | not native |
| Containers / architecture | **first-class** | clusters are limited |
| Matrix / table-like | grid / SQL table | HTML labels / records |
| One binary for Linux/macOS/Windows | **yes** | C library/package, platform build |
| Offline | yes | yes |
| Deterministic with fixed input | yes, fixed salt | yes, fixed version |
| Accessibility | no; we post-process | partial; we post-process |
| Dense graph layout | good | better |
| Edge-label readability | TALA + post-process | good by default |
| Default aesthetics | strong | dated |
| Community | 25.7k stars, active | decades of use, GitLab 1.5k stars |

The “dense graph layout” advantage of Graphviz is not relevant to Folisift: the
visual budget is deliberately 0–3 diagrams per chapter and 12 nodes / 20 edges
per diagram. At that size, D2’s TALA layout is good, and D2’s breadth
(sequence, containers, themes, one binary) removes the need for a second
engine.

Graphviz’s edge-label and compactness advantages are real. They are the reason
this was a close decision. They are addressed in the D2 pipeline by:

- `--layout=tala` for graph-shaped diagrams;
- `*.style.font-size: 28` and connection-label font size 22;
- `--pad 20` to remove default whitespace;
- a post-process background stroke behind edge labels;
- the existing 17 cm figure cap and centering.

## Activity evidence

Collected 2026-10-29.

| Project | Last commit | Latest release | Adoption |
|---|---|---|---|
| **D2** | 2026-10-11 | **v0.9.0** (2026-09-07) | GitHub 25,755 stars / 775 forks; Linux v0.9.0 asset 177,765 downloads |
| Graphviz | 2026-10-29 | **16.1.0** (2026-09-04) | GitLab 1,475 stars / 438 forks; distro package |
| Mermaid | 2026-10-28 | 12.2.0 (2026-10-13) | GitHub 90,920 stars / 9,424 forks |
| PlantUML | 2026-10-28 | 1.2026.9 | GitHub 13,385 stars |
| diagrams | 2026-10-15 | v0.25.1 (2025-11-22) | GitHub 42,763 stars; Python + Graphviz |
| ELK-rs | 2026-10-23 | 0.12.1 | GitHub **9 stars**, created 2026-02-10 |
| Structurizr CLI | 2026-02-01 | **archived** | no |
| Excalidraw | 2026-10-28 | active | 134,793 stars; manual editor |
| draw.io | 2026-10-23 | active | 9,061 stars; manual editor |

D2 is active in both commits and releases. Graphviz is also active and has the
strongest history, but it is a C/system dependency and cannot cover sequence
diagrams without another engine. The 9-star ELK-rs port is rejected; the mature
Eclipse ELK would add a JVM or JS runtime.

`diagrams` is a Python cloud-architecture DSL over Graphviz. Its own README
says it uses Graphviz to render. It has 400 open issues and a release from
November 2025. It is not a general diagram engine and would add Python to the
offline product. It is not selected.

## Aesthetic comparison

### D2

- Modern default themes, including a colorblind-safe theme.
- Containers, sequence diagrams, SQL tables, classes and architecture are
  first-class.
- Cleaner authoring syntax; smaller prompts if the LLM ever writes D2 directly.
- TALA places flow labels above connectors in our local test.
- Sequence labels still sit on the lifeline in the tested output; the post-
  processor adds a background stroke so the line does not cross the text.
- Default canvas is wide; font-size and `--pad` must be set by the generator.

### Graphviz

- Default output is functional/technical, not “paper-like”.
- With a house DOT template (Inter, paper fill, accent stroke, box nodes,
  `nodesep`, `ranksep`, `fontsize`) it looks clean and compact.
- Edge labels are placed beside the edge.
- No sequence diagram support.
- Best when graphs are large or crossing minimisation matters; our budget is
  small.

### Local A/B evidence

Same five-node flow:

| Metric | Graphviz 14.1.4 | D2 v0.9.0 with TALA + font tuning |
|---|---|---|
| ViewBox | 869 × 50 | 1695 × 123 (font 28/22) |
| Node text at 510 pt width | ≈ 8.2 pt | ≈ 8.4 pt |
| Edge text at 510 pt width | ≈ 6.5 pt | ≈ 6.6 pt |
| Edge label placement | beside edge | above edge with TALA; sequence masked by post-process |
| Accessibility metadata | node/edge titles | none; we add root metadata |
| Same input twice | 5/5 identical | 2/2 identical with fixed salt |

Graphviz is more compact, but D2’s tuned output reaches the same readable text
size and covers sequence/containers with one binary.

## Output examples

D2:

- Official examples: <https://d2lang.com/examples/overview/>
- D2 playground: <https://play.d2lang.com>
- Architecture example: <https://blog.jread.com/posts/d2-is-a-great-language-for-architecture-diagrams/>
- D2 vs Graphviz examples:
  <https://github.com/andrewleech/d2plugin-graphviz>

Graphviz:

- Official gallery: <https://graphviz.org/gallery/>
- Cluster example: <https://graphviz.org/Gallery/directed/cluster.html>
- Online DOT editor: <https://edotor.net>

## OpenCode MCP design

The MCP must not run `d2` or any shell. The LLM must not write D2; it writes the
typed JSON spec, and Folisift generates D2.

| MCP surface | Purpose |
|---|---|
| `diagram_schema` resource | JSON Schema: kind, nodes, edges, caption, source, engine options |
| `diagram_validate` | Validate and normalise a spec |
| `diagram_save` | Write `guides/<slug>/diagrams/<id>.json` |
| `diagram_status` | Compare spec hash with the SVG manifest; report missing/stale |
| `diagram_render_plan` | Return the exact CLI command and output path; do not execute |
| `build_guide` | Embed only prebuilt, sanitised SVG; fail on unresolved spec |
| `write-diagram` prompt | Visual budget, limits, caption/alt/provenance rules |

Workflow:

```text
diagram_validate
  → diagram_save
  → OpenCode shell: folisift diagram build
  → diagram_status
  → build_guide
```

This is one extra shell step by design. Running D2 inside MCP would violate the
no-shell invariant; embedding the Go runtime in Rust is not practical. The LLM’s
job is small: produce JSON, then call one CLI command.

## D2 pipeline rules

- Pin the D2 binary version and record it in the asset manifest.
- Use `--salt folisift` (constant) so SVG IDs are stable.
- Use `--layout=tala` for flow/concept/tree; sequence uses D2’s sequence engine.
- Use `--pad 20`, `--no-xml-tag` and a fixed theme/font.
- Generate `*.style.font-size: 28` and connection labels 22 for 510 pt width;
  adjust only through the generator, not per diagram.
- Post-process every SVG:
  - add `role="img"`, `aria-labelledby`, `<title>`, `<desc>`;
  - add `paint-order: stroke` and background stroke to `text-italic` edge
    labels;
  - reject scripts, external refs, foreignObject and CSS imports;
  - hash the final SVG.
- Limit D2 to 12 nodes / 20 edges and 4 hierarchy levels; split large diagrams.
- The generated D2 source is stored next to the JSON spec for audit.
- MPL-2.0 notice is included in the release.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| D2 output changes between versions | pin exact version; golden SVG + visual baselines |
| Edge-label line collision | TALA + background stroke; regression test |
| No accessibility metadata | deterministic post-processor + tests |
| Wide canvas / small text | tuned font sizes, pad, split rule, 17 cm cap |
| Extra 18 MB binary | compress in release; D2 is optional at authoring time, not needed to build PDFs |
| MPL-2.0 | distribute unmodified binary + notice; no source modifications |
| Render cannot run in MCP | CLI authoring + embed-only MCP; `diagram_render_plan` guides the host |

## Rejected tools

- **Graphviz:** strongest layout history, but C/system dependency, no sequence,
  and would require a second engine for the full diagram range.
- **diagrams:** Python cloud-icon DSL over Graphviz; not general, adds Python.
- **Mermaid:** Node + Chromium, non-deterministic history, layout issues.
- **PlantUML:** JVM, LGPL, verbose.
- **Structurizr:** archived repositories.
- **Excalidraw/draw.io:** manual editors, not deterministic code-first layout.
- **ELK-rs:** 9 stars, created 2026; too young.
- **d2-little:** 3k downloads, incomplete documentation.
- **Kroki:** server/network dependency.

## Migration plan

1. Vendor/pin D2 v0.9.0 for Linux, macOS and Windows.
2. Replace the native `diagram` fence renderer with the D2 source generator.
3. Add the SVG post-processor (accessibility + edge-label background).
4. Add `diagram_schema`, `diagram_validate`, `diagram_save`, `diagram_status`
   and `diagram_render_plan` MCP tools.
5. Make `build_guide` embed-only for diagrams; unresolved specs fail.
6. Add golden SVG tests, 2-run determinism, paper/clean visual baselines and a
   tall/wide figure test.
7. Remove Graphviz/D2 choices from the user-facing spec; `engine` is always D2.
8. Keep the native sequence layout only until the D2 sequence post-processor
   passes; then delete it.

Estimated effort: 4–6 engineering days plus visual review.

## Sources

- D2 repository: <https://github.com/d2lang/d2>
- D2 examples: <https://d2lang.com/examples/overview/>
- D2 layouts: <https://d2lang.com/tour/layouts/>
- D2 ELK: <https://d2lang.com/tour/elk/>
- D2 TALA: <https://d2lang.com/blog/tala-is-open-source/>
- D2 styles: <https://d2lang.com/tour/style/>
- D2 globs: <https://d2lang.com/tour/globs/>
- D2 positions: <https://d2lang.com/tour/positions/>
- D2 v0.9.0 release:
  <https://github.com/d2lang/d2/releases/tag/v0.9.0>
- Graphviz gallery: <https://graphviz.org/gallery/>
- Graphviz licence: <https://graphviz.org/license/>
- Graphviz SVG: <https://graphviz.org/docs/outputs/svg/>
- Graphviz accessibility: <https://gitlab.com/graphviz/graphviz/-/issues/2220>
- diagrams: <https://github.com/mingrammer/diagrams>
- diagrams release: <https://github.com/mingrammer/diagrams/releases/tag/v0.25.1>
- Mermaid: <https://mermaid.js.org/config/usage.html>
- PlantUML: <https://github.com/plantuml/plantuml>
- Structurizr CLI: <https://github.com/structurizr/cli>
- ELK-rs: <https://github.com/openedges/elk-rs>
- Reddit D2: <https://www.reddit.com/r/selfhosted/comments/1gin6ut/holy_crap_d2_diagrams_are_impressive/>
- Reddit Mermaid: <https://www.reddit.com/r/programming/comments/1qqizxa/your_ai_diagram_looks_great_and_nobody_will_read/>
- Reddit Graphviz:
  <https://www.reddit.com/r/networking/comments/8dbucs/tools_to_create_maintainable_network_diagrams/>
- Reddit diagrams:
  <https://www.reddit.com/r/aws/comments/gjxil2/diagrams_as_code_python_with_aws_icon_support/>
