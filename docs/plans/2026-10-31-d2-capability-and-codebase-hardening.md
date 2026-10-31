# Folisift v0.11 — D2 capability expansion and codebase hardening

- **Status:** Proposed; no code changes until approved
- **Date:** 2026-10-31
- **Baseline:** `v0.10.0` (`8aef654`; 135 tests)
- **Trigger:** the user wants the full, appropriate D2 capability set; a
  professional diagram grammar; and a whole-repository audit folded into the
  plan
- **Glossary:** [`GLOSSARY.md`](../../GLOSSARY.md)
- **Research:** [`docs/d2-capability-research.md`](../d2-capability-research.md),
  [`docs/codebase-audit-2026-10-31.md`](../codebase-audit-2026-10-31.md)
- **Related:** [`2026-10-31-typography-standardization-and-font-system.md`](2026-10-31-typography-standardization-and-font-system.md)
- **Decision in one sentence:** expand D2 from one generic node/edge contract
  to a typed diagram grammar that uses the right D2 capability for each
  learning task, while rejecting static-PDF-incompatible or unsafe features.

## 1. What “all D2 capabilities” should mean

D2 can do much more than boxes and arrows, but a study guide is a static,
accessible PDF. “Use all capabilities” means **use every capability that
serves the learning task**, not every flag the binary exposes.

| Capability | D2 support (v0.9.0) | Study-guide value | Decision |
|---|---|---|---|
| Flow/concept/tree | core nodes/edges/containers | structure, dependencies, hierarchy | **use** |
| Sequence diagram | `shape: sequence_diagram`; actors, messages, spans, groups, notes, self-messages | protocols, algorithms, causal order | **use** |
| UML class | `shape: class`; fields/methods | type systems, OOP, schemas | **use** |
| SQL table / ERD | `shape: sql_table`; PK/FK/UNQ; edges point to rows with TALA/ELK | databases, data models | **use** |
| Grid | `grid-rows`, `grid-columns`, gaps | spatial arrangements | **reject in diagrams; use a Typst table** |
| Containers/groups | nested containers, `near` (object near only TALA) | modules, layers, parts | **use** |
| Markdown labels | Markdown in labels | short emphasis only | **constrain; bold/italic, no link soup** |
| Code labels | fenced code with language | annotated code diagrams | **reject; code belongs in Typst body text** |
| LaTeX/TeX | MathJax blocks | one-line formula labels | **constrain; short math only** |
| Icons/images | URL or local path; hosted icons | recognition, architecture | **reject; decorative/network risk** |
| Themes | built-in light/dark themes | palette consistency | **constrain; one paper theme only** |
| Sketch | `--sketch` | hand-drawn aesthetic | **reject; no learning evidence** |
| Layouts | dagre, ELK, TALA; feature differences | graph type fit | **TALA only; reproducibility** |
| PNG/PDF/PPTX/GIF export | CLI supports | none for Typst pipeline | **reject; SVG only** |
| Animation/scenarios/layers | `--animate-interval`, targets | static PDF cannot animate | **reject; render one board** |
| Imports/vars | compile-time composition | reusable fragments | **host-side only; local files only** |
| Links/tooltips | D2 links/tooltips; SVG appendix | source links | **constrain; validate PDF links** |

### 1.1 What must be rejected, and why

- **Animations/scenarios/layers:** a static PDF has no timeline; exporting one
  frame silently loses content. If a scenario is useful, render it as a
  separate diagram with its own caption/source.
- **Remote icons/URLs/imports:** network access is non-deterministic and an
  injection surface. D2 is not a sandbox: it can fetch URLs and execute
  `d2plugin-*` binaries found on `PATH`. Keep the generator-owned D2 source
  free of icons/imports and spawn D2 with a scrubbed environment and private
  `PATH`.
- **PNG/PPTX/GIF exports:** they bypass Typst typography, accessibility tags and
  vector scaling. SVG is the only build input.
- **Full LaTeX documents:** D2 uses MathJax, not a TeX layout engine. Use it
  for one-line formulas only, not pages or derivations.
- **Raw D2/SVG/Typst from the host:** the host writes typed JSON; Folisift
  writes D2 and sanitises SVG.

## 2. Current Folisift D2 contract

Confirmed from `crates/folisift-render/src/diagram.rs`:

- `SPEC_SCHEMA_VERSION = 2`; `D2_VERSION = v0.9.0`.
- `kind` accepts only `flow`, `concept`, `tree`, `sequence`.
- `DiagramSpec` has only `id`, `kind`, `title`, `caption`, `alt`, `source`,
  `direction`, `layout`, `nodes`, `edges`, `groups`.
- `Node` has only `id`, `label`, `group`.
- `Edge` has only `from`, `to`, `label`.
- `Group` has only `id`, `label`, `nodes`.
- `MAX_NODES = 12`, `MAX_EDGES = 20`, `MAX_DEPTH = 4`.
- `layout` must be `tala`; dagre/ELK are rejected.
- `d2_source` hard-codes label font sizes (`28`/`22`), border radius and colors;
  it does not use typography tokens.
- No themes, sketch, icons, SQL tables, UML classes, grid, markdown/code/
  LaTeX labels, imports, layers/scenarios or target boards.
- The MCP validates/saves specs; D2 executes only in `diagram build` in the
  CLI. This boundary is correct and must stay.

This is a good safety foundation, but it is a **single generic diagram** with a
few labels, not a professional diagram grammar.

## 3. Proposed D2 schema v3

### 3.1 What to add

Schema v2 is a generic node/edge/group contract. v3 adds only the capabilities
the research supports:

1. **Sequence diagrams** — actors, messages, notes (start with notes; groups/
   spans later).
2. **UML class** — classes, fields, methods, visibility.
3. **SQL table / ERD** — tables, columns, PK/FK/UNQ, crow's-foot arrowheads.
4. **Edge semantics** — `arrowhead` (`triangle`, `none`, `cf-one`, `cf-many`,
   `cf-one-required`, `cf-many-required`) and `line` (`solid`, `dashed`).
5. **Node shape allowlist** — `rectangle`, `document`, `diamond`, `cylinder`,
   `cloud`, `person`; not the full 25-shape catalogue.
6. **Style classes** — semantic roles (`definition`, `example`, `warning`,
   `process`), not per-diagram decoration.
7. **Narrow math labels** — `label_format: "math"` for one-line MathJax;
   no code blocks, no general Markdown soup.

### 3.2 Sequence

```json
{
  "schema_version": 3,
  "id": "tcp-handshake",
  "kind": "sequence",
  "title": "TCP three-way handshake",
  "caption": "SYN, SYN-ACK and ACK establish sequence numbers before data.",
  "alt": "Client and server exchange SYN, SYN-ACK and ACK before data transfer.",
  "source": "PD4, s. 18",
  "layout": "tala",
  "actors": [
    {"id": "client", "label": "Client"},
    {"id": "server", "label": "Server"}
  ],
  "messages": [
    {"from": "client", "to": "server", "label": "SYN"},
    {"from": "server", "to": "client", "label": "SYN-ACK"},
    {"from": "client", "to": "server", "label": "ACK"}
  ],
  "notes": [{"actor": "client", "label": "seq = x"}]
}
```

Rules: ≤6 actors, ≤10 messages, at least two actors, order is semantic, no
self-message unless the source shows one. Groups/spans are a later extension.

### 3.3 UML class

```json
{
  "schema_version": 3,
  "id": "expr-types",
  "kind": "class",
  "title": "Expression types",
  "caption": "Expr is the base; Literal and Binary are subclasses.",
  "alt": "UML class diagram of Expr, Literal and Binary.",
  "source": "PD3, s. 61",
  "layout": "tala",
  "classes": [
    {"id": "expr", "label": "Expr", "fields": ["span: Span"], "methods": ["eval(): Value"]},
    {"id": "literal", "label": "Literal", "fields": ["value: Value"], "methods": []},
    {"id": "binary", "label": "Binary", "fields": ["left: Expr", "right: Expr"], "methods": []}
  ],
  "edges": [
    {"from": "literal", "to": "expr", "label": "inherits", "arrowhead": "triangle"},
    {"from": "binary", "to": "expr", "label": "inherits", "arrowhead": "triangle"}
  ]
}
```

Rules: ≤6 classes, ≤12 members each; fields/methods only if examinable;
visibility only when it matters; no OOP decoration.

### 3.4 SQL table / ERD

```json
{
  "schema_version": 3,
  "id": "orders-erd",
  "kind": "erd",
  "title": "Orders schema",
  "caption": "A customer has many orders; each order has many lines.",
  "alt": "Customers, orders and order lines with primary and foreign keys.",
  "source": "PD5, s. 42",
  "layout": "tala",
  "tables": [
    {"id": "customers", "label": "customers", "columns": [
      {"name": "id", "type": "int", "constraint": "PK"},
      {"name": "email", "type": "text", "constraint": "UNQ"}
    ]},
    {"id": "orders", "label": "orders", "columns": [
      {"name": "id", "type": "int", "constraint": "PK"},
      {"name": "customer_id", "type": "int", "constraint": "FK"}
    ]}
  ],
  "edges": [{"from": "orders.customer_id", "to": "customers.id", "label": "many-to-one", "arrowhead": "cf-many"}]
}
```

Rules: ≤6 tables, ≤12 columns each; mark PK/FK; use crow's-foot cardinality;
TALA is required for exact-row FK edges.

### 3.5 Limits and layout

| Kind | Limit | Layout |
|---|---|---|
| `concept`/`flow` | 12 nodes / 20 edges | TALA |
| `tree` | 12 nodes / depth 4 | TALA |
| `sequence` | 6 actors / 10 messages | TALA |
| `class` | 6 classes / 12 members | TALA |
| `erd` | 6 tables / 12 columns | TALA |

Keep **TALA only**. D2’s own docs say Dagre/ELK have feature gaps (container
routing, `near`, row-level FK edges on Dagre); exposing engine choice makes
output non-reproducible and asks the host to tune layout. TALA’s randomness is
controlled with `--tala-seeds`; pin the seeds and record them.

### 3.6 Themes and typography

- One paper theme; theme 0/neutral plus brand palette postprocess. No theme
  picker, no dark mode.
- Diagram label sizes come from typography tokens:
  - node label ≈ 0.86 × body,
  - edge label ≈ 0.78 × body,
  - note ≈ 0.78 × body.
- Use the font pairing’s heading/body/mono fonts; validate Turkish glyphs.
- No hard-coded 28/22 px in `d2_source`.
- Minimum effective printed label size 8.5–9 pt; split a diagram that needs
  smaller text.

### 3.7 Composition and safety

- Reject imports, vars, layers, scenarios, steps, animation, icons, links,
  tooltips, `near`/`far` (far is not a v0.9.0 keyword).
- Keep SVG only; D2 PDF is raster, and PNG/PPTX/GIF bypass Typst.
- P0: spawn D2 with `env_clear()`, a private/empty `PATH`, a temporary `HOME`,
  and no inherited `D2_*` variables. Add a fake `d2plugin-evil` regression test
  that must not execute.
- Add a generated-source allowlist test: emitted `.d2` may contain only
  direction, styles, nodes, edges, groups and sequence/class/SQL payloads.
- Keep exact version check, 120 s timeout, output-path confinement and the SVG
  sanitizer as hard gates.

## 4. Diagram grammar and evidence

| Learning task | Diagram | Evidence | Rule |
|---|---|---|---|
| Structure/relationships | `concept` | **Strong** for studied maps g≈0.43; construction stronger | 5–9 nodes; label every edge; one proposition per edge; ≤2 cross-links |
| Process/state machine | `flow` | **None** (representational fit) | 4–8 steps; verb-labelled edges; no cycles unless the point |
| Protocol/interaction | `sequence` | **None** (notation semantics) | ≤6 actors/10 messages; order is content |
| Strict taxonomy | `tree` | **Weak** (indirect) | One root; depth ≤4; no cross-links |
| Type system | `class` | **Moderate** (UML standard) | ≤6 classes; only examinable members |
| Relational schema | `erd` | **Moderate** (SQL/ERD standard) | ≤6 tables; PK/FK; crow's-foot |
| Architecture/boundaries | `concept` + one group | **Weak** | ≤3 groups; no icons |
| Short formula | `concept` with math label | **None** for diagrams | One line; derivation in prose |
| Comparison/matrix | Typst table | **Moderate** (table convention) | Do not use D2 grid |
| Chronology | Typst table/flow | **None** | Do not call it a timeline; D2 has no time axis |

Default figure contract:

1. One figure, one takeaway; caption states the takeaway.
2. A diagram must earn its space; a table/sentence may be clearer.
3. Label every meaningful edge.
4. One accent for the main relation; no rainbow.
5. No icons, links, tooltips, sketch or animation.
6. 12/20/depth-4 hard cap; 5–9 nodes practical.
7. Source reference mandatory.
8. Prefer one strong figure over three weak ones.

## 5. Codebase audit findings to fold in

This section is populated from the whole-repo audit; confirmed findings:

| Id | Severity | Finding | Fix |
|---|---|---|---|
| A1 | high | MCP `build_guide` does not run the revision grammar gate; CLI `build --check` does | put the gate in a shared build service used by CLI and MCP |
| A2 | high | MCP `build_guide` runs A–G coverage for revision/organiser guides; CLI skips it; reproduced with `build_guide(pd2-rev, check=true)` → `20/20 headings missing` | share the genre-aware check policy |
| A3 | high | Knowledge-organiser one-page limit is not enforced; a two-page organiser can pass | add page-count gate for `knowledge-organiser` |
| A4 | medium | D2 `layout` is TALA-only; dagre/ELK rejected | keep TALA only; document the reproducibility/feature-gap reason |
| A5 | medium | D2 label sizes are hard-coded 28/22 | typography tokens + diagram scale |
| A6 | medium | `unsafe { std::env::set_var }` in PDFium init can race with parser threads | initialise before worker threads or use a process-level loader without env mutation |
| A7 | medium | `audit` body-font heuristic can misclassify a single-size document | require two sizes or accept an explicit `--body-font` |
| A8 | medium | Revision/organiser `build --check` skips canonical coverage entirely | genre-specific coverage model, not “no coverage” |
| A9 | low | `synthesis_page` can be sparse (one column/5 bullets) | content gate for 2 comparisons + 3 traps + map/terms |
| A10 | low | `paper` still carries 6.9 pt labels/7.4 pt code | legacy exemption or migrate in a parity ADR |
| A11 | high | D2 CLI inherits `PATH`; `d2plugin-*` binaries can execute | spawn with `env_clear()`, private/empty `PATH`, temp `HOME` |
| A12 | high | Generated D2 source is not allowlisted; a future field could emit imports/icons/links | source allowlist test; reject imports/icons/links/vars/layers/scenarios/steps/animate |
| A13 | high | `find_d2` falls back to `PATH`, so an untrusted `d2` can be selected | prefer explicit/sidecar; remove implicit `PATH` fallback or require an opt-in |
| A14 | low | `scaffold` doc says schema-2 but writes schema 3 | fix comment and add a schema test |
| A15 | high | `paper`/`reader`/`accessible` sizes are not a single scale | shared `TypographyScale`; layout chooses only body base |
| A16 | low | `styles preview` forces `a-g`, so revision/organiser previews are not genre previews | preview the pack’s first supported genre structure |
| A17 | medium | 22-page paper parity freezes the legacy file and hides reflow drift | mark paper legacy; structural parity for new styles/ADR |

Additional high/medium findings from the full audit:

| Id | Severity | Finding | Fix |
|---|---|---|---|
| B1 | high | `prepare_guide` reuses an existing guide config but returns the newly requested style/sources | reject changes or update config atomically |
| B2 | high | style/domain IDs are not sandboxed; MCP can read outside the workspace | validate IDs; canonicalize and contain under `styles/`/`domains/` |
| B3 | high | inventory has no source digest; stale inventory is trusted | store BLAKE3 digests; re-analyze on mismatch |
| B4 | high | config `coverage`/`inventory` paths can overwrite guide/output files | enforce bundle containment and `.md`/`.json` |
| B5 | high | MCP reports coverage and visual separately; no top-level pass | `passed = coverage && visual && layout` |
| B12 | medium | PDF is written before gates run | render to temp, gate, atomic rename |
| B13 | medium | English revision/organiser labels are Turkish | move labels to pack strings; EN test |
| B16/B17 | medium | heading/mono tokens ignored; contrast gate narrow | shared typography/palette roles and all-text contrast |
| B22/B23 | medium | revision table row limit off by one; canonical TOC numbering wrong | parser fixes + fixtures |

The full audit file adds source-cache, packaging, licence, schema-1, heading
scan, source-read loop, image parsing and diagnostic-line findings; each high
finding gets a regression test before the fix.

## Implementation status — 2026-10-31

Implemented in the working tree:

- D2 schema v3: sequence actors/messages/notes, UML class, SQL/ERD, edge
  arrowheads and solid/dashed lines, shape allowlist; v2 remains accepted;
- deterministic spec hashing that preserves v2 hashes;
- generated-source allowlist and scrubbed D2 environment with no implicit
  `PATH` lookup;
- shared CLI/MCP genre, page-count, coverage and top-level gates;
- atomic staged PDF output and referenced-only diagram loading;
- source digests and stale-inventory re-analysis; verified source cache
  headers and page sequences; resumable large-page reads;
- style/domain ID sandboxing and bundle-path checks;
- packaging fails without PDFium unless explicitly overridden; PDFium and font
  notices are shipped;
- schema 1 guide configs are rejected with a clear unsupported-schema error;
- raw HTML is rejected instead of dropped; Markdown code fences are lint-safe;
- `style_rationale` is enforced for schema 3 guides;
- EN revision/organiser labels and stronger TOC/production smoke checks;
- D2 v3 fixtures compile and postprocess when `FOLISIFT_D2` is set;
- style-aware audit margins/minimum font, visual-page underfill policy and
  caption counting;
- revision/organiser `source_scope` page coverage;
- duplicate top-level section detection;
- pinned TALA seeds, theme/font digests, fake-plugin regression test and
  per-kind postprocessed SVG baselines;
- determinism, aria/title and source allowlist tests.

Still open:

- per-kind **PDF** baselines (the SVG baselines cover the engine output);
- macOS/Windows archives and release signing;
- per-kind PDF baselines (SVG baselines cover the engine output).

## 6. Implementation phases

### Phase 0 — Audit baseline

- [x] Freeze the audit findings and severity.
- [x] Add fixtures for MCP/CLI parity (revision, organiser, reader).
- [x] Add D2 fixtures per kind.
- [x] Add red tests for A1–A3, B1, B2, B4, B5, B12.
- **Exit:** red-capable tests for the P0 set.

### Phase 1 — Shared build service

- [x] Extract the remaining CLI/MCP orchestration into `folisift-build`:
      structure → genre grammar → layout audit → genre coverage → report.
- [x] Render to temp, gate, atomic rename.
- [x] Return one top-level `passed`; `coverage && visual && layout`.
- [x] Validate style/domain IDs and inventory/coverage paths.
- [x] Store source digests and invalidate stale inventory.
- [x] Make `prepare_guide` reject or atomically apply style/source changes.
- [x] Add `knowledge-organiser` page-count gate.
- **Exit:** CLI and MCP produce the same pass/fail for the same guide; no
      out-of-workspace reads or overwrite.

### Phase 2 — D2 schema v3

- [x] Add typed payloads: `sequence`, `class`, `erd`.
- [x] Add edge arrowheads/line styles and a small node-shape allowlist.
- [x] Add per-kind validation/limits; keep TALA only.
- [x] Add theme/typography tokens; no icons/sketch.
- [x] Keep SVG-only, local-only, no animations/layers.
- [x] Add visual baselines per kind.
- **Exit:** one pilot per kind builds; invalid specs fail with actionable errors.

### Phase 3 — Diagram grammar and prompts

- [x] Add `diagram grammar` docs: which kind for which task, with evidence.
- [x] Extend MCP `diagram` validation/status and `prepare_guide` visual targets.
- [x] Add source-grounding checks: caption/alt/source, no invented labels.
- [x] Add lint: sequence ≥2 actors, tree no cross-links, per-kind limits;
      concept/tree edges may be unlabelled when the relation is self-evident.
      ERD has a key; no icons/code blocks/links in generated source.
- **Exit:** the host can choose a diagram kind from the task, not guess.

### Phase 4 — D2 hardening

- [x] Reject imports/icons/links/vars/layers/scenarios/steps/animation.
- [x] Scrub D2 environment/PATH; fake-plugin regression test.
- [x] Theme/postprocess palette and typography tests; TALA seeds pinned.
- [x] PDF aria/title checks for SVG.
- [x] Determinism: salt, TALA seeds, font digest, layout, version.
- **Exit:** no network in MCP; builds reproducible.

### Phase 5 — Full audit remediation

- [x] Fix all high/medium findings from `docs/codebase-audit-2026-10-31.md`
      (paper remains the labelled legacy exemption; A16 is low).
- [x] Add regression tests and gates.
- [x] Re-run tests, clippy, fmt, parity, visual, revision/organiser, TOC,
      package and clean install.
- **Exit:** no open high-severity finding.

## 7. Acceptance criteria

- [x] D2 supports concept/flow, tree, sequence, class and ERD with typed specs;
      grid/annotated remain rejected.
- [x] Layout allow-list works; no unrestricted engine string.
- [x] Theme/typography tokens are validated; icons are rejected.
- [x] SVG-only, local-only, static-only; no animations/layers/remote URLs.
- [x] CLI and MCP use the same build gates.
- [x] Organiser is exactly one page or fails.
- [x] Revision grammar cannot be bypassed through MCP.
- [x] All high/medium audit findings closed with regression tests.
- [x] Visual baselines per kind/engine; no clipping/overflow.

## 8. Decisions requested

1. **D2 scope:** implement sequence + class + ERD in v0.11 (recommended), or
   only sequence until a course needs class/ERD?
2. **Layouts:** keep TALA-only (recommended; research says Dagre/ELK have
   feature gaps), or expose a tested per-kind allow-list?
3. **Icons:** reject (recommended; coherence/network risk), or allow a vetted
   local icon pack?
4. **Sketch mode:** reject (recommended; no learning evidence), or optional?
5. **Audit scope:** fix every high/medium finding before v0.11 (recommended),
   or split into v0.11 (correctness) and v0.12 (polish)?

No code changes until these are confirmed.
