# D2 v0.9.0 capability research for study guides

- **Status:** Research note, not a decision record
- **Date:** 2026-10-31
- **Scope:** D2 v0.9.0 only, as pinned by Folisift

## 1. Method and version pin

Primary sources only: the official D2 docs at `d2lang.com`, the D2 source at tag
`v0.9.0`, the v0.9.0 release notes, the local D2 v0.9.0 binary's help, and the Folisift
source/docs. The docs site is current, so version-sensitive claims were checked against
the tag (`d5a5174`) and the local binary.

Folisift pins the engine in `crates/folisift-render/src/diagram.rs:14`
(`D2_VERSION = "v0.9.0"`). The staged binary reports `d2 v0.9.0`
(`/home/yilmaz/.local/lib/folisift/d2 --version`). No source code was changed.

Evidence labels: **Strong** = meta-analysis or directly verified engine behaviour;
**Moderate** = consistent experimental programme or primary design standard;
**Weak** = indirect/small-sample/design judgement; **None** = notation semantics or
engineering preference, with no learning-effect claim.

## 2. What changed in v0.9.0, and why the exact pin matters

The [v0.9.0 release notes](https://github.com/d2lang/d2/releases/tag/v0.9.0) list four
changes that affect this pipeline:

1. **TALA is bundled and open source.** `--layout=tala` no longer needs a separate
   plugin or licence key. Before v0.9.0 it was an external, licensed engine.
2. **PNG, GIF, PDF and PPTX use the built-in renderer.** v0.8.2 still prompted for a
   Chromium download for PNG; v0.9.0 removes that browser dependency
   ([0.8.2 notes](https://github.com/d2lang/d2-docs/blob/master/docs/releases/0.8.2.md)).
3. **Markdown labels render as native SVG, not HTML `foreignObject`.** This is the most
   important portability change for a PDF pipeline.
4. **`--tala-seeds` exists.** TALA has randomness; fixed seeds make its attempts
   reproducible.

v0.8.2 also replaced the embedded Dagre/ELK JavaScript runtimes with native Go ports,
explicitly changing coordinates, ordering and edge routes. A v0.9.0 SVG is therefore not
expected to match a pre-0.8.2 SVG byte-for-byte or geometrically. The exact pin is what
makes golden SVG and visual baselines meaningful.

Version-sensitive cautions:

- **No `timeline` or `mindmap` shape.** The v0.9.0 shape list is closed and contains
  neither ([`d2target.go`](https://github.com/d2lang/d2/blob/v0.9.0/d2target/d2target.go)).
  The roadmap's "git-commit timeline" is a future layout, not an available type
  ([roadmap](https://d2lang.com/tour/future/)).
- **No `far` keyword.** D2 has `near` with eight fixed anchors; `far` is not a reserved
  attribute ([`d2ast/keywords.go`](https://github.com/d2lang/d2/blob/v0.9.0/d2ast/keywords.go),
  [positions](https://d2lang.com/tour/positions/)). A `far:` line may compile as an
  ordinary child key, not as positioning.
- The docs may describe post-v0.9.0 features; do not infer them from the current site.

## 3. Diagram types: only what D2 actually supports

D2 is a node-link language first. "Flow", "concept map" and "tree" are topologies, not
separate D2 types. Sequence, UML class and SQL table are special shapes with their own
semantics.

| Study need | D2 v0.9.0 representation | What is supported |
|---|---|---|
| Flow / pipeline / state machine | Ordinary nodes and directed edges; global `direction` | Yes. No special flow type; layout, not source order, decides position. |
| Concept map | Ordinary nodes, labelled edges, optional containers | Yes. Labelled edges are what make it a concept map rather than a bubble diagram. |
| Tree / taxonomy | Ordinary nodes with one parent; `direction: down` convention | Yes as topology. No semantic tree type; Folisift's `kind: tree` is its own convention. |
| Sequence / protocol | `shape: sequence_diagram` | First-class: actors, messages, spans, groups, notes, self-messages. Definition order is display order. ([sequence](https://d2lang.com/tour/sequence-diagrams/)) |
| UML class / type schema | `shape: class` | Fields, methods, types, visibility prefixes `+ - #`. ([UML classes](https://d2lang.com/tour/uml-classes/)) |
| SQL schema / ERD | `shape: sql_table` | Columns, types, `primary_key`/`foreign_key`/`unique`, multiple constraints, FK edges. D2 calls this the way to diagram ERDs. ([SQL tables](https://d2lang.com/tour/sql-tables/)) |
| Crow's-foot ERD | `cf-one`, `cf-one-required`, `cf-many`, `cf-many-required` arrowheads | Yes, as connection arrowheads. ([connections](https://d2lang.com/tour/connections/)) |
| Table / matrix | Grid containers (`grid-rows`, `grid-columns`) or a Markdown table | Yes, but a grid is a layout container, not a semantic table. ([grid](https://d2lang.com/tour/grid-diagrams/)) |
| Architecture / boundaries | Labelled, nested containers | First-class. |
| Timeline / Gantt | — | **No shape, no timeline layout.** |
| Mind map / radial | — | **No dedicated shape.** ELK's `radial` is a tree layout, not a notation. ([ELK](https://d2lang.com/tour/elk/)) |

Consequences:

- **Sequence is the only type with true temporal ordering.** Elsewhere D2 has no
  source-order semantics; layout decides. A "timeline" of boxes/arrows is a flow, not a
  timeline: no axis, scale or interval semantics.
- **SQL table edges point to exact rows only with TALA or ELK**, not Dagre
  ([SQL tables](https://d2lang.com/tour/sql-tables/)). Folisift's TALA default is right
  if ERD output is added.

## 4. Shapes, icons, containers, labels, tooltips and links

**Shapes.** v0.9.0 has 25: rectangle (default), square, page, parallelogram, document,
cylinder, queue, package, step, callout, stored_data, person, c4-person, diamond, oval,
circle, hexagon, cloud, text, code, class, sql_table, image, sequence_diagram and
hierarchy ([`d2target.go`](https://github.com/d2lang/d2/blob/v0.9.0/d2target/d2target.go)).
Circle and square preserve 1:1 ([shapes](https://d2lang.com/tour/shapes/)). Folisift
emits only rectangles, leaving process/document/decision/actor shapes unused.

**Icons.** D2 accepts any URL, a local path (`./my_cat.png`) and the hosted collection at
`icons.d2lang.com`; `shape: image` makes a standalone image ([icons](https://d2lang.com/tour/icons/)).
Remote icons are fetched by the CLI. Reject this for study guides: network/supply-chain
surface, usually decorative, and contrary to the coherence principle (§12).

**Containers.** First-class: nesting, labels, and `_` for the parent scope
([containers](https://d2lang.com/tour/containers/)). TALA has first-class container
consideration; Dagre needs a shim for container-to-container routing
([layouts](https://d2lang.com/tour/layouts/), [TALA](https://d2lang.com/tour/tala/)).
Folisift supports one flat group level.

**Labels.** A shape's key is its default label; `label:` overrides it; keys are
case-insensitive. Folisift quotes every label and escapes quotes/backslashes
(`d2_quote` in `crates/folisift-render/src/diagram.rs`).

**Tooltips and links.** `tooltip:` becomes an HTML `title`; `link:` becomes `<a href>`.
PNG exports turn tooltips into a numbered appendix; `--force-appendix` adds it to SVG.
Links stay clickable in D2's PDF ([interactive](https://d2lang.com/tour/interactive/),
[exports](https://d2lang.com/tour/exports/)). In print, tooltips have no surface and links
are a distraction/security hazard. Folisift has no fields for either, and its sanitizer
rejects external `href`/`src` (`crates/folisift-render/src/assets.rs`). Keep it that way.

## 5. Text, code, mathematics and rich content

- **Markdown labels.** A label can be Markdown if the shape is explicitly declared;
  standalone text is Markdown by default. v0.9.0 emits native SVG text, not
  `foreignObject` ([text](https://d2lang.com/tour/text/), [release](https://github.com/d2lang/d2/releases/tag/v0.9.0)).
- **Code blocks.** Change `md` to a language for Chroma highlighting; unknown languages
  fall back to plain text. Arbitrary block-string delimiters handle code containing `|`
  ([text](https://d2lang.com/tour/text/)).
- **LaTeX.** `latex`/`tex` uses MathJax, not full LaTeX. It is for notation, not document
  layout; `font-size` does not apply inside it, and there are no automatic line breaks
  (`\tiny`, `\small`, `\normal`, `\large`, `\huge`; `displaylines` workaround)
  ([text](https://d2lang.com/tour/text/)).
- **Rich text.** Markdown bold, italic, underline, strikethrough, links, inline code and
  tables work in labels; v0.9.0 keeps them as SVG primitives
  (`d2renderers/d2svg/markdown_native_test.go`).
- **Foreign objects.** The v0.9.0 non-test SVG renderer contains no `foreignObject`
  output; the Markdown test asserts its absence. This makes v0.9.0 SVG safer to embed in
  Typst than pre-0.9.0 SVG.

Folisift escapes every label as plain text, so Markdown/code/math are unused. That is a
gap, but not one to close indiscriminately: code blocks and multi-line formulas compete
with the prose that carries the explanation. A narrow `label_format: math` for short
formulas is a better next step than full Markdown labels.

## 6. Layout, direction, spacing, sketch, themes and dark mode

The local binary lists three bundled engines: Dagre (default), ELK (layered default;
`force`, `stress`, `mrtree`, `radial` alternatives) and TALA (`d2 layout`;
[layouts](https://d2lang.com/tour/layouts/), [ELK](https://d2lang.com/tour/elk/)). D2's
own comparison is candid: Dagre is fast but unmaintained and strictly hierarchical; ELK
is more mature at crossings and container routing; TALA is designed for architecture
diagrams, with symmetry, per-container direction, `top`/`left` locking and
`near`-to-another-shape ([Dagre](https://d2lang.com/tour/dagre/), [TALA](https://d2lang.com/tour/tala/)).

TALA has randomness: a small label change can cascade into a different layout.
`--tala-seeds` (default `[1,2,3]`) makes attempts deterministic and selects the best
complete result (`d2 layout tala`). Folisift's salt stabilises SVG IDs; the seeds
stabilise geometry, and the current invocation relies on the defaults.

`direction` is global except in TALA, where it can be set per container. `top`/`left`
locking and `near`-to-an-object are TALA-only ([positions](https://d2lang.com/tour/positions/),
[layouts](https://d2lang.com/tour/layouts/)). Spacing flags are engine-specific
(`--dagre-nodesep`, `--dagre-edgesep`, `--elk-nodeNodeBetweenLayers`,
`--elk-padding`, `--elk-edgeNodeBetweenLayers`, `--elk-nodeSelfLoop`) plus global
`--pad` (`d2 layout dagre`, `d2 layout elk`, `d2 --help`).

`--sketch` gives a hand-drawn look ([sketch](https://d2lang.com/tour/sketch/)). The local
binary exposes 20 themes: 18 light, 2 dark (`d2 themes`); theme 0 is neutral default.
`--dark-theme` adds a browser-preference CSS switch, irrelevant to print
([themes](https://d2lang.com/tour/themes/)). Custom TTF fonts cover regular, italic,
bold, semibold and mono variants ([fonts](https://d2lang.com/tour/fonts/)); D2 embeds
WOFF faces in the SVG stylesheet, so the SVG is self-contained.

Folisift's choices are sound for paper: TALA, `--pad=20`, theme 0, paper colours, Inter
and NotoSansMono, font sizes 28/22, 120 s timeout. It does not expose layout choice,
per-container direction, spacing or sketch.

## 7. Composition: imports, variables, classes, near, scenarios and animation

| Feature | v0.9.0 behaviour | Decision |
|---|---|---|
| Imports | `@file` and spread `...@file`; relative to the importing file; absolute paths allowed; only `.d2` opened ([imports](https://d2lang.com/tour/imports/)) | **Reject.** Arbitrary local reads; the generator owns the source. |
| Vars / substitutions | `vars` plus `${}`; nested, scoped; single quotes bypass; `d2-config` can set layout/theme/pad/sketch ([vars](https://d2lang.com/tour/vars/)) | **Reject in content.** Generator may use internally; hosts write JSON. |
| Style classes | Reusable attribute bundles; multiple classes apply left-to-right; written to SVG `class` ([classes](https://d2lang.com/tour/classes/)) | **Use narrowly** for semantic node roles, not decoration. |
| UML class shape | A different feature: fields, methods, visibility ([UML classes](https://d2lang.com/tour/uml-classes/)) | **Use** only for type/schema content. |
| `near` / `far` | `near`: eight anchors plus `center-center`, `outside-*`, `border-*` for labels/icons; **no `far` keyword** ([positions](https://d2lang.com/tour/positions/)) | **Reject.** Caption/prose give context; do not invent `far`. |
| Layers / scenarios / steps | Independent boards; scenarios inherit from base; steps inherit from previous ([composition](https://d2lang.com/tour/composition/)) | **Reject for print.** |
| Animation | `--animate-interval` packages boards into an animated SVG/GIF ([formats](https://d2lang.com/tour/composition-formats/)) | **Reject.** Print cannot carry it; the map-animation evidence is k=2. |

Folisift generates one flat board with none of these. That is correct. The one feature
worth adopting is style classes for semantic roles, because it turns colour/shape into a
consistent signalling system rather than per-diagram decoration.

## 8. Export and what survives in a PDF study guide

D2 exports SVG (default), PNG, PDF, PPTX, GIF, ASCII (`.txt`, beta) and stdout
([exports](https://d2lang.com/tour/exports/); `d2 --help`).

- **SVG is vector and self-contained.** CSS and subsetted fonts are embedded; assets and
  layers are bundled by default (`--bundle`, default true). IDs carry a deterministic hash
  prefix; `--salt` changes the suffix ([exports](https://d2lang.com/tour/exports/);
  `d2 --help`).
- **D2's PDF is raster.** The docs say PDF exports are PNG exports placed on PDF pages,
  with headers and fonts. Links stay clickable; animation does not appear
  ([exports](https://d2lang.com/tour/exports/)). Do not use it as the study-guide source:
  it throws away selectable text and vector line art.
- **Folisift embeds SVG into Typst.** Local verification on the built PD3 guide shows
  diagram labels are extractable text (`pdftotext`) and Inter/NotoSansMono are embedded
  (`pdffonts`), so the vector/text path works.

| Element | Survives? | Note |
|---|---|---|
| Vector geometry, strokes, fills, theme colours | Yes | Typst renders SVG as vector content; paint attributes are preserved. |
| Text labels | Yes, selectable | Verified on `output/PD3_Calisma_Ozeti.pdf`. |
| Embedded fonts | Yes | Typst emits embedded subsets; verified with `pdffonts`. |
| Caption and alt text | Yes | Passed into `#figure(...)`; the PDF is tagged. |
| Tooltips (`<title>`) | No usable print affordance | Hover-only; caption must carry the takeaway. |
| External links (`<a href>`) | No, intentionally blocked | Sanitizer rejects external `href`/`src`. |
| Animation / layers / scenarios | No | Single static board only. |
| D2 PDF export | Not used | Rasterised PNG on a page; diagram text not selectable. |

Rule: **SVG is the interchange format; the caption is the teaching surface.** Anything a
student must know belongs in the caption or prose, not in a tooltip or link.

## 9. Security and safety: what must stay host-side

D2's language is declarative; there is no script or `exec` primitive in diagram source.
The CLI is still a host-side program with file/network access and one easy-to-miss
execution path.

| Vector | v0.9.0 behaviour | Folisift today | Required posture |
|---|---|---|---|
| Local imports | Reads `.d2` files, including absolute paths ([imports](https://d2lang.com/tour/imports/)) | Generator emits none | Keep raw D2 out of host/MCP input; if ever allowed, confine to a temp dir. |
| Remote icons/images | Fetches any `http(s)` URL, data URIs and local files; bounded bytes, content sniffing, DTD rejection, but no private-address denylist in the resolver ([`resolve.go`](https://github.com/d2lang/d2/blob/v0.9.0/lib/imageasset/resolve.go)) | Generator emits none | Keep icons rejected; local assets only if ever added. SSRF is real (`localhost`, metadata IPs). |
| External plugins | At startup, scans `$PATH` for `d2plugin-*` and runs external plugin `info` commands ([`plugin.go`](https://github.com/d2lang/d2/blob/v0.9.0/d2plugin/plugin.go), [`main.go`](https://github.com/d2lang/d2/blob/v0.9.0/d2cli/main.go)) | `Command::new(d2)` inherits `PATH` | **P0:** spawn with `env_clear()`, private/empty `PATH`, temp `HOME`, no `D2_*`. |
| SVG scripts/handlers | D2 output has no scripts; Markdown links are plain `<a>` | Sanitizer rejects `<script>`, `foreignObject`, `iframe/object/embed`, `javascript:`, `@import`, external `url(...)`, external `href`/`src`, event handlers (`assets.rs`) | Keep sanitizer as a hard gate on every SVG. |
| Shell/MCP | D2 is a separate binary | MCP never runs D2; `build` embeds prebuilt SVG | Keep the ADR-0010 boundary. |
| Output/timeout/version | Writes the path given; `--timeout` default 120 s; exact version check | Under `guides/<slug>/data/d2` and `assets/generated`; 120 s; checks `v0.9.0` | Keep path resolution under the guide root; fail closed. |

**D2 is not a sandbox.** It reads files, fetches URLs and can execute plugin binaries.
That is acceptable only because Folisift keeps it in the CLI authoring step, feeds it
generator-owned D2, and never exposes it to MCP or `build`. The missing piece is process
isolation from the ambient `PATH`.

## 10. What Folisift supports today, and the gap

Current contract (`crates/folisift-render/src/diagram.rs`):

- Schema v2; kinds `flow`, `concept`, `tree`, `sequence`; 1–12 nodes, ≤20 edges, tree
  depth ≤4; one tree root; no multiple parents/duplicate IDs. Nodes: `id`, `label`,
  optional `group`; edges: `from`, `to`, optional `label`; one flat group level.
- Direction `right/down/left/up`; layout must be `tala`; theme from paper tokens; font
  sizes 28/22; `--pad=20`, `--theme=0`, `--salt`, `--timeout=120`, `--no-xml-tag`,
  `--omit-version`; Inter/NotoSansMono.
- Post-processing: `role="img"`, `aria-labelledby`, `<title>`, `<desc>`, edge-label
  background stroke, sanitizer. Manifest: spec/SVG hashes, D2 version, layout, salt,
  fonts, post-process version. `build` embeds prebuilt assets only; never runs D2.

Usage: 12 specs across checked-in guides (some duplicated between `pd2` and `pd2-rev`) —
11 `concept`, 1 `flow`, no `sequence`, no `tree`. The pipeline supports more than the
content uses.

| Capability | D2 v0.9.0 | Folisift | Gap / decision |
|---|---|---|---|
| Flow / concept | Yes | Yes | Use; concept is the workhorse. |
| Tree | Yes as topology | Yes, validated | Constrain to strict hierarchies. |
| Sequence | Actors, messages, spans, groups, notes | Actors/messages only | Extend only if a course needs protocol tracing. |
| UML class | Fields, methods, visibility | No | Add a `class` kind. |
| SQL table / ERD | Columns, keys, crow's foot | No | Add an `erd` kind. |
| Node shapes | 25 | Rectangles only | Add a small allowlist (document, diamond, cylinder, cloud). |
| Edge semantics | Arrowheads, dashed, labels, classes | Label only | Add `arrowhead` and `line` (solid/dashed). |
| Containers | Nested, labelled | One flat group | Keep one level unless needed. |
| Markdown/code/math | Yes, native SVG | Plain text | Constrain to short math; reject multi-line code. |
| Icons / images | URL/local/hosted | No | Reject (decorative/networked). |
| Tooltips / links | Yes | No | Reject for print; caption carries meaning. |
| Layouts | Dagre/ELK/TALA | TALA only | Keep TALA; no engine picker. |
| Composition | Imports/vars/scenarios/layers/animation | None | Reject; single board is correct. |
| Export | SVG/PNG/PDF/PPTX/GIF/ASCII | SVG → Typst | Keep SVG; D2 PDF is raster. |
| Safety | Plugins/imports/remote images | Sanitizer good; inherited `PATH` | P0: scrub env/PATH. |

## 11. Capability matrix

Decision: **use** = adopt; **constrain** = narrow generator contract; **reject** = do not
expose.

| Capability | D2 support | Study-guide value | Evidence / standard | Decision | Reason |
|---|---|---|---|---|---|
| Concept/flow node-link | Yes | High | Strong: studied maps g≈0.43; static g≈0.39 | **Use** | Best fit for structure and relationships. |
| Tree topology | Yes | Medium | Weak/indirect | **Constrain** | Strict single-parent hierarchies only; otherwise concept. |
| Sequence | First-class | High for protocols | None; notation semantics | **Use** | Only D2 type with true temporal order. |
| UML class | Yes | High for type systems | Moderate; UML standard | **Use** (new kind) | Schema literacy needs standard notation. |
| SQL table/ERD | Yes | High for schemas | Moderate; SQL/ERD standard | **Use** (new kind) | Entities, keys, relations are the content. |
| Crow's-foot arrows | Yes | High for ERD | Moderate; standard notation | **Use** with ERD | Makes cardinality explicit. |
| Grid/matrix | Yes | Low | Tables are standard; grid adds complexity | **Reject in diagrams** | Use a Markdown table in prose. |
| Timeline/Gantt | **No** | Low | None reviewed | **Reject** | No D2 timeline; do not fake one. |
| Mind map/radial | No dedicated shape | Low | None reviewed | **Reject** | ELK radial is a layout, not a notation. |
| Containers | Yes, nested | Medium | Design judgement | **Constrain** | One flat group is enough for figures. |
| Icons/images | Yes | Low | Coherence principle | **Reject** | Decoration, network risk, inconsistent style. |
| Tooltips/links | Yes | Low in print | None; no print affordance | **Reject** | Caption/prose carry the takeaway. |
| Markdown rich text | Yes, native SVG | Medium | Moderate (signalling/contiguity) | **Constrain** | Bold/italic only where it signals structure. |
| Code blocks | Yes, Chroma | Low in diagrams | None; coherence | **Reject** | Code belongs in body text. |
| LaTeX/math | Yes, MathJax subset | Medium for math | None for diagrams | **Constrain** | Short formulas only. |
| Dagre / ELK | Yes | Low/medium | Engine trade-off | **Reject as exposed choice** | TALA is better for containers/orthogonal routes; one engine keeps output reproducible. |
| TALA | Yes, bundled | High | Repo decision; local A/B | **Use** | Best fit for paper figures/containers. |
| Per-container direction | TALA only | Low | None | **Constrain** | Global direction is enough. |
| Spacing flags | Yes | Medium | Legibility, not learning | **Constrain** | Generator-owned `--pad`; no per-diagram tuning. |
| Sketch mode | Yes | Low | No learning evidence | **Reject** | Conflicts with professional consistency. |
| Themes / dark mode | Yes, 20 themes; CSS dark switch | Low for print | Design consistency | **Constrain / reject dark** | One paper theme; print has no dark mode. |
| Imports / vars | Yes, incl. absolute paths | Low for hosts | Security | **Reject in content** | Arbitrary local reads; generator may use vars internally. |
| Style classes | Yes | Medium | Signalling principle | **Use narrowly** | Semantic roles, not decoration. |
| `near` constants | Yes | Low | None | **Reject** | Caption/prose give context. |
| `far` | **No** | None | — | **Reject** | Not a D2 keyword. |
| Scenarios/layers/steps/animation | Yes | Low for print | Weak (k=2) | **Reject** | Static single board only; thin animation evidence. |
| SVG export | Yes, default | High | Verified vector/text | **Use** | Only format preserving text and vectors. |
| PNG/PDF/PPTX/GIF | Yes | Low | D2 PDF is raster | **Reject for source** | Keep SVG as source of truth. |
| Custom fonts | Yes, TTF flags | High | Brand consistency | **Use** | Inter/NotoSansMono already pinned. |

## 12. Recommended diagram grammar for study guides

The evidence does not support "more diagrams = better learning". It supports a specific,
well-labelled node-link map for structure, plus three multimedia principles: spatial
contiguity, signalling and coherence
([Mayer, Cambridge Handbook](https://www.cambridge.org/core/books/cambridge-handbook-of-multimedia-learning/principles-for-reducing-extraneous-processing-in-multimedia-learning-coherence-signaling-redundancy-spatial-contiguity-and-temporal-contiguity-principles/CD5B7AE1279A9AB81F8EEBB53DBEC86E)).

What the learning evidence actually says:

- **Concept maps have meta-analytic support, but use matters.** Nesbit and Adesope (2006)
  synthesised 55 studies (5,818 students) and found increased retention, with effects from
  small to large; static maps had `g = 0.393` (32 studies), animated maps `g = 0.739`
  (only 2 studies) ([SAGE](https://journals.sagepub.com/doi/10.3102/00346543076003413),
  [full text](https://www.sfu.ca/~jcnesbit/research/NesbitAdesope2006.pdf)).
- **Studying a pre-made map is weaker than constructing one.** Schroeder et al. (2018)
  synthesised 142 effect sizes (11,814 learners): overall `g = 0.58`; creating `g = 0.72`;
  studying `g = 0.43` ([DOI](https://doi.org/10.1007/s10648-017-9403-9),
  [record](https://rex.libraries.wsu.edu/esploro/outputs/journalArticle/Studying-and-Constructing-Concept-Maps-a/99900601155801842)).
  A study guide is mostly the "studying" case, so `g ≈ 0.43` is the honest planning
  number, not `0.72`.
- **Maps may bias toward central ideas.** Nesbit and Adesope note the greater benefit for
  central ideas suggests a bias toward summary. Use maps for structure; keep prose for
  detail and edge cases.
- **Animation evidence is too thin.** Two studies is not a basis for animated boards,
  especially in print.
- **Sequence/class/ERD/timeline have no learning-effect claim here.** They are notation
  choices: use them when the domain's objects and relations are the learning target.
- **Icons and sketch have no supporting learning evidence.** Coherence argues against
  decoration.

| Learning task | Use | Evidence | Rule |
|---|---|---|---|
| Structure, definitions, relationships | `concept` | **Strong** (studied g≈0.43) | 5–9 nodes in practice; label every edge with a relation; one proposition per edge; hierarchy plus ≤2 cross-links. |
| Process, pipeline, state machine | `flow` | **None** (representational fit) | 4–8 steps; verb-labelled edges; direction right/down; cycles only if the cycle is the point. |
| Protocol, interaction over time | `sequence` | **None** (notation semantics) | ≤6 actors, ≤10 messages; order is the content; notes only for a real constraint. |
| Strict taxonomy | `tree` | **Weak** (indirect) | One root; depth ≤4; no cross-links; switch to `concept` if one is needed. |
| Type system / OOP schema | `class` | **Moderate** (UML standard) | ≤6 classes; fields/methods only if examinable; show visibility when it matters. |
| Relational schema | `erd` (`sql_table`) | **Moderate** (SQL/ERD standard) | ≤6 tables; mark PK/FK; crow's-foot cardinality; exact-row edges require TALA. |
| Comparison/matrix | Markdown table in prose | **Moderate** (table convention) | Do not use a D2 grid; a table is clearer and cheaper. |
| Chronology | Prose table, or a flow only if order is the objective | **None** | Do not call it a timeline; D2 has no time axis. |
| Architecture/boundaries | `concept` with one group level | **Weak** | Only when boundaries are examinable; ≤3 groups; no icons. |
| Short formula | `concept`/`flow` with a math label | **None** for diagrams | One line only; derivations in prose. |

Default figure contract:

1. **One figure, one takeaway.** Caption states it, not "image of"; Folisift already
   rejects `alt` starting with "image of"/"resim".
2. **A diagram must earn its space.** Use it when relations, order or hierarchy are the
   content; otherwise use prose or a table. One strong figure beats three weak ones.
3. **Label every meaningful edge.** Unlabelled arrows are ambiguous; in a concept map they
   turn relations into guesswork.
4. **Keep the palette semantic.** One accent for the main relation, muted labels, no
   rainbow. This is the signalling principle applied consistently.
5. **No icons, links, tooltips or sketch.** Caption/prose carry context; the figure
   carries structure. Stay inside 12 nodes / 20 edges / depth 4; 5–9 nodes is the
   practical target.
6. **Cite the source.** Every figure gets a page reference; no provenance, no figure.

## 13. Plan-ready recommendations

**P0 — safety, before features.**

- Spawn D2 with `env_clear()`, a private/empty `PATH`, temp `HOME`, and no inherited
  `D2_*`. Regression test: a fake `d2plugin-evil` on `PATH` must not execute.
- Add a generated-source allowlist test: only direction, styles, nodes, edges and groups.
  Reject `import`, `icon`, `link`, `tooltip`, `vars`, `layers`, `scenarios`, `steps`,
  `animate`, `near`. Keep the version check, 120 s timeout, path confinement and
  sanitizer as hard gates.

**P1 — capability, only where a course needs it.**

- Add `arrowhead` (`triangle`, `none`, `cf-one`, `cf-many`, `cf-one-required`,
  `cf-many-required`) and `line` (`solid`, `dashed`) to edges. This unlocks ERD
  cardinality and optional/async edges without a new engine.
- Add a small shape allowlist (`rectangle`, `document`, `diamond`, `cylinder`, `cloud`,
  `person`) and `kind: class`/`kind: erd` (`sql_table`). Both kinds are first-class and
  serve type-system/database courses; validate counts and keep 12 nodes.
- Extend `sequence` only if needed: notes/groups/spans are supported, but the generator
  emits actors/messages only. Start with notes.

**P2 — content quality.**

- Add `label_format: plain|math` with a one-line MathJax allowlist. No general Markdown
  or code blocks in labels; keep code in prose.
- Use style classes for semantic roles (definition, example, warning, process): the one
  composition feature worth adopting.
- Document the grammar in `docs/authoring.md` with one example per kind. Lint: every edge
  labelled for `concept`; no `sequence` with <2 actors; no `tree` with cross-links; no
  `erd` without a key. Keep TALA and one paper theme; no picker.

**Explicit non-goals:** timeline, Gantt, mind map, grid diagrams, icons, tooltips,
external links, sketch, dark mode, animation, layers/scenarios/steps, imports and
host-authored D2. Each is unsupported by v0.9.0, unreadable in print, or a
security/reproducibility liability.

## 14. Sources
### D2 docs

Tour index <https://d2lang.com/tour/intro/> · shapes, text, icons, containers:
`/tour/shapes/`, `/tour/text/`, `/tour/icons/`, `/tour/containers/` · sequence, SQL,
UML: `/tour/sequence-diagrams/`, `/tour/sql-tables/`, `/tour/uml-classes/` · layout:
`/tour/layouts/`, `/tour/dagre/`, `/tour/elk/`, `/tour/tala/` · style/export:
`/tour/positions/`, `/tour/themes/`, `/tour/sketch/`, `/tour/fonts/`, `/tour/exports/` ·
composition/security: `/tour/imports/`, `/tour/vars/`, `/tour/classes/`,
`/tour/interactive/`, `/tour/grid-diagrams/`, `/tour/composition/`, `/tour/layers/`,
`/tour/scenarios/`, `/tour/steps/`, `/tour/composition-formats/`, `/tour/future/`.

### D2 source at `v0.9.0`

Release <https://github.com/d2lang/d2/releases/tag/v0.9.0> · shape catalogue
`d2target/d2target.go` · `near` constants `d2ast/keywords.go` · plugin discovery/exec
`d2plugin/plugin.go`, `d2plugin/exec.go` · CLI startup `d2cli/main.go` · image resolver
`lib/imageasset/resolve.go` · native Markdown test `d2renderers/d2svg/markdown_native_test.go`
· 0.8.2 notes <https://github.com/d2lang/d2-docs/blob/master/docs/releases/0.8.2.md>.
D2 source paths are under <https://github.com/d2lang/d2/blob/v0.9.0/>; 0.8.2 notes live in `d2-docs`.

### Local verification

- Staged binary `/home/yilmaz/.local/lib/folisift/d2`; `--version`, `--help`, `layout`,
  `layout dagre`, `layout elk`, `layout tala`, `themes`.
- Folisift: `crates/folisift-render/src/diagram.rs`, `crates/folisift-cli/src/diagram_cmd.rs`,
  `crates/folisift-render/src/assets.rs`, `docs/diagram-engine-evaluation.md`,
  `docs/adr/0010-d2-single-diagram-engine.md`, `docs/authoring.md`.
- Specs `guides/*/diagrams/*.json` (11 `concept`, 1 `flow`); PDF survival via `pdftotext`
  and `pdffonts` on `output/PD3_Calisma_Ozeti.pdf`.

### Learning research

- Nesbit, J. C., & Adesope, O. O. (2006). Learning with concept and knowledge maps: A
  meta-analysis. *Review of Educational Research, 76*(3), 413–448.
  <https://doi.org/10.3102/00346543076003413>; full text
  <https://www.sfu.ca/~jcnesbit/research/NesbitAdesope2006.pdf>
- Schroeder, N. L., Nesbit, J. C., Anguiano, C. J., & Adesope, O. O. (2018). Studying
  and constructing concept maps: A meta-analysis. *Educational Psychology Review, 30*(2),
  431–455. <https://doi.org/10.1007/s10648-017-9403-9>
- Mayer, R. E. (2014). Principles for reducing extraneous processing in multimedia
  learning: Coherence, signalling, redundancy, spatial contiguity and temporal
  contiguity. In *The Cambridge Handbook of Multimedia Learning*.
  <https://www.cambridge.org/core/books/cambridge-handbook-of-multimedia-learning/principles-for-reducing-extraneous-processing-in-multimedia-learning-coherence-signaling-redundancy-spatial-contiguity-and-temporal-contiguity-principles/CD5B7AE1279A9AB81F8EEBB53DBEC86E>
