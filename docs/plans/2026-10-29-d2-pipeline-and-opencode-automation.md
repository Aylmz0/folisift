# Folisift v0.7 — D2-only diagram pipeline and OpenCode automation

- **Status:** Released in v0.7.3
- **Date:** 2026-10-29
- **Baseline:** `v0.6.2` (105 tests, 12 MCP tools, native diagram compiler)
- **Release:** `v0.7.3` (107 tests, 13 MCP tools, D2 v0.9.0)
- **Target:** `v0.7.0`
- **Decision:** D2 v0.9.0 is the **only** external diagram engine
- **Related:**
  `docs/plans/2026-10-29-production-hardening-and-visuals.md`,
  `docs/plans/2026-10-28-roadmap.md`,
  `docs/diagram-engine-evaluation.md`,
  `docs/architecture.md`, `docs/authoring.md`, ADR-0007, ADR-0009
- **Goal:** make diagrams reproducible and auditable through one pinned D2
  engine, and let an OpenCode host run the whole production loop through MCP
  plus a small, explicit CLI render step.

## 1. Why this plan exists

The engine has a correct content pipeline, coverage/audit gates, visual
regression and a stable 12-tool MCP surface. Diagrams are still the weak part:

- the in-tree compiler lays out JSON diagrams itself and has no D2 dependency;
- the D2 decision exists only in `docs/diagram-engine-evaluation.md`;
- there is no `diagram build` CLI, no D2 source generator and no asset manifest;
- the MCP has no diagram tools, so the host cannot validate, save or check a
  diagram;
- `build_guide` can still compile inline diagram JSON, which is incompatible
  with “D2 is the only external engine”;
- there is no OpenCode runbook that tells the host the exact order of calls;
- no D2 version, font, salt, layout or post-processor is pinned/tested.

This plan closes those gaps. It also fixes the boundary that caused confusion:
**D2 is used by the CLI, not by the MCP server.** The MCP server never executes
D2, a shell or a network request. It validates specs, writes specs, checks the
manifest and embeds prebuilt SVG.

## 2. Non-negotiable invariants

1. **One external diagram engine: D2 v0.9.0.** No Graphviz, no Mermaid, no
   PlantUML, no ELK-rs, no second fallback.
2. **MCP is hands; OpenCode is the brain.** No LLM inside the MCP server.
3. **No shell, network or D2 execution inside MCP.** D2 runs only in the CLI
   authoring step.
4. **Build is embed-only.** `folisift build` and `build_guide` read committed
   SVG + manifest; they never run D2.
5. **No raw D2, raw Typst, raw CSS or arbitrary SVG from the LLM.** The host
   writes the typed JSON spec; Folisift writes D2 and sanitises the SVG.
6. **Deterministic output.** Same spec + same D2 version + same fonts +
   post-processor version → same SVG hash.
7. **Pinned binary.** The D2 version, platform binary and checksum are
   recorded; a mismatch fails loudly.
8. **Visuals are content.** Every diagram has a takeaway caption, alt text,
   provenance and source page.
9. **No silent repair.** Missing/stale assets fail with the exact command to
   rebuild.
10. **Source PDFs stay local.** No generated source text or D2 cache is
    committed; specs, D2 source and final SVG are.

## 3. Current state and gaps

| # | Gap | Priority | Current state |
|---|---|---|---|
| D1 | D2 integration absent | P0 | decision doc only; native layout still used |
| D2 | No D2 source generator | P0 | fenced JSON goes to custom layout |
| D3 | No D2 asset manifest | P0 | generated SVG has no engine/version/hash record |
| D4 | No diagram CLI | P0 | no `diagram build/status/validate` |
| D5 | No MCP diagram tool | P0 | 12 tools; no validate/save/status/render plan |
| D6 | `build_guide` accepts inline specs | P0 | can bypass the single-engine rule |
| D7 | No SVG accessibility post-processor | P0 | D2 has no root title/desc/ARIA |
| D8 | Edge/sequence labels can collide | P0 | D2 default places labels on the line |
| D9 | No OpenCode runbook | P1 | host prompt omits diagrams and D2 |
| D10 | No D2 packaging/licence notice | P1 | no binary, checksum, MPL notice |
| D11 | No golden/visual tests for D2 | P1 | only native diagram tests |
| D12 | Graphviz references remain in plan | P2 | architecture confusion |

## 4. Target pipeline

```text
source PDFs
  │
  ├─ analyze / inventory / low-text page inspection
  │
  ▼
OpenCode host LLM
  ├─ writes guide.md prose
  ├─ writes diagrams/<id>.json (typed spec)
  └─ does not write D2 or SVG
        │
        ▼
MCP (no shell)
  ├─ diagram validate / save / status / render_plan
  └─ build_guide: embed only
        │
        ▼
CLI authoring step (the only D2 consumer)
  ├─ generate data/d2/<id>.d2 deterministically
  ├─ run pinned d2 v0.9.0 --layout=tala
  ├─ post-process + sanitise + hash
  └─ write assets/generated/<hash>.svg + data/diagrams/<id>.json
        │
        ▼
guide build (no D2)
  ├─ read ::: diagram id="..." directives
  ├─ resolve manifest + SVG
  ├─ embed as Typst figure with caption/alt
  └─ coverage + structure + layout + visual gates
```

## 5. Bundle layout

```text
guides/<slug>/
├── guide.md                       # ::: diagram id="pipeline"
├── config.json
├── diagrams/
│   └── pipeline.json              # typed spec, source of truth
├── data/
│   ├── d2/
│   │   └── pipeline.d2            # generated, audited
│   ├── diagrams/
│   │   └── pipeline.json          # manifest: hashes, engine, version
│   ├── source.json
│   ├── source.md
│   └── coverage.md
└── assets/
    └── generated/
        └── <svg-hash>.svg         # committed, embed-only
```

Rules:

- `diagrams/<id>.json` is the content source; the LLM owns it.
- `data/d2/<id>.d2` is generated; never hand-edited.
- `assets/generated/<svg-hash>.svg` is committed so a fresh build does not
  need D2.
- `data/diagrams/<id>.json` is committed and is the audit record.
- The guide references the stable ID, not the SVG hash.
- Manual SVG assets (source crops, annotated figures) remain allowed; they are
  not D2 output and still pass the sanitizer.

## 6. Diagram spec v2

```json
{
  "schema_version": 2,
  "id": "pipeline",
  "kind": "flow",
  "title": "Derleme hattı",
  "caption": "Kaynak programdan amaç koda beş aşamalı dönüşüm.",
  "alt": "Kaynak program sözcüksel, sözdizimsel ve anlamsal analizden geçerek amaç koda dönüşür.",
  "source": "PD0, s. 35",
  "direction": "right",
  "layout": "tala",
  "nodes": [
    { "id": "src", "label": "Kaynak program" },
    { "id": "lex", "label": "Sözcüksel analiz" },
    { "id": "parse", "label": "Sözdizim analizi" },
    { "id": "sem", "label": "Anlamsal analiz" },
    { "id": "code", "label": "Kod üretimi" }
  ],
  "edges": [
    { "from": "src", "to": "lex", "label": "karakterler" },
    { "from": "lex", "to": "parse", "label": "tokenlar" },
    { "from": "parse", "to": "sem", "label": "ağaç" },
    { "from": "sem", "to": "code", "label": "ara kod" }
  ],
  "groups": []
}
```

Validation:

- `schema_version` must be 2.
- `id` matches `^[a-z0-9][a-z0-9_-]{0,63}$` and equals the filename.
- `kind` ∈ `flow`, `concept`, `tree`, `sequence` for v0.7.0. Future kinds are
  additive, not a backend choice.
- `title`, `caption`, `alt`, `source` are non-empty.
- `alt` describes the takeaway, not “image of …”.
- 1–12 nodes, 0–20 edges, tree depth ≤ 4.
- unique node IDs; edge endpoints exist.
- `source` is the raw provenance string, e.g. `PD0, s. 35` or `original`.
  The renderer adds brackets and validates the final caption (`[PD0, s. 35]`).
- no `engine`, `theme`, `font` or shell field; the engine is always D2.
- `direction` is `right`, `down`, `left` or `up` (default by kind).
- `layout` defaults to `tala`; `sequence` ignores it.

Captions and alt text are rendered as selectable text, not baked into the SVG.

## 7. D2 source generator

`folisift-render::diagram::d2` converts the spec to a deterministic D2 file.

Mapping:

| Spec | D2 |
|---|---|
| `flow` / `concept` | `direction: right` (or spec), nodes + edges |
| `tree` | direction `down`, root/child edges |
| `sequence` | `shape: sequence_diagram`, actors + messages |
| `groups` | nested D2 containers with absolute IDs |
| node ID | generated safe key (`n0`, `n1`, …); label is escaped |
| edge label | D2 connection label (post-processed background) |

Style block emitted once:

```d2
direction: right
style: { fill: "#FAF6EC" }
*.style: {
  fill: "#FAF6EC"
  stroke: "#8A3E2F"
  font-color: "#211E19"
  font-size: 28
  border-radius: 3
}
(* -> *)[*].style: {
  stroke: "#8A3E2F"
  font-color: "#5E574B"
  font-size: 22
}
```

Rules:

- Never interpolate raw user text into D2 identifiers. Use generated keys.
- Escape quotes, backslashes and newlines in labels.
- No D2 `import`, `vars`, `icon`, remote URL or arbitrary file reference from
  the spec.
- The generated file starts with a comment containing `spec_hash` and
  `engine_version`; comments do not affect layout.
- The generator is pure; it does not execute D2.

## 8. D2 binary contract

- Version: **v0.9.0**, exact. A different version fails before rendering.
- Binary lookup:
  1. `FOLISIFT_D2` environment variable;
  2. `./d2` or `./d2.exe` next to the `folisift` binary;
  3. `d2` on `PATH`.
- Invocation uses `std::process::Command` with an argument array, never a shell
  string.
- Fixed arguments:

```text
d2
  --layout=tala
  --salt fsl-<spec-hash-prefix>
  --pad 20
  --theme 0
  --no-xml-tag
  --omit-version
  --font-regular templates/fonts/Inter-Regular.ttf
  --font-bold templates/fonts/Inter-Bold.ttf
  --font-italic templates/fonts/Inter-Italic.ttf
  --font-semibold templates/fonts/Inter-SemiBold.ttf
  --font-mono templates/fonts/NotoSansMono-Regular.ttf
  data/d2/<id>.d2
  <temp-svg>
```

- Sequence uses the D2 sequence engine; `--layout` is ignored by that shape.
- Capture stdout/stderr with a bounded tail; report exit code and command.
- Write to a temporary path in the same directory, validate the SVG, then
  atomically rename.
- Record `d2 --version` output in the manifest.
- No fallback to native layout. If D2 is missing or fails, the CLI exits
  non-zero with the exact command and version requirement.

## 9. SVG post-processor

D2 SVG is an implementation detail; the committed SVG is Folisift-owned.

Steps, in order:

1. Read the D2 SVG as UTF-8; reject invalid XML/non-SVG.
2. Inject into the root `<svg>`:
   - `role="img"`;
   - `aria-labelledby="title-<hash> desc-<hash>"`;
   - a `<title id="title-<hash>">` with `title`;
   - a `<desc id="desc-<hash>">` with `alt`.
3. For every D2 edge/sequence label (`class="text-italic"`), add:
   - `paint-order="stroke"`;
   - `stroke="<paper>"`;
   - `stroke-width="4"`;
   - `stroke-linejoin="round"`.
   This prevents the connector from crossing the text.
4. Add the diagram background/fill if D2 did not emit it.
5. Run the existing `assets::sanitize_svg`:
   - reject scripts, event handlers, foreignObject, external references, CSS
     imports;
   - require `viewBox`;
   - reject absolute/parent asset paths where applicable.
6. Reject if no `<title>` or `<desc>` remains after post-processing.
7. Compute `blake3(svg)` and use it as the asset filename.

The post-processor is deterministic and has a version constant included in the
cache key and manifest.

## 10. Manifest and cache

`data/diagrams/<id>.json`:

```json
{
  "schema_version": 2,
  "id": "pipeline",
  "kind": "flow",
  "spec_sha256": "…",
  "svg_sha256": "…",
  "svg": "assets/generated/<hash>.svg",
  "d2_version": "v0.9.0",
  "layout": "tala",
  "salt": "fsl-<spec-prefix>",
  "fonts": ["Inter-Regular.ttf", "Inter-SemiBold.ttf", "Inter-Italic.ttf", "NotoSansMono-Regular.ttf"],
  "font_digest": "…",
  "postprocess_version": 1,
  "caption": "…",
  "alt": "…",
  "source": "PD0, s. 35"
}
```

Rules:

- No timestamp in the manifest; build numbers are separate.
- `spec_sha256` covers the canonical JSON (sorted keys, no whitespace drift).
- `svg_sha256` covers the final post-processed SVG.
- The build cache key includes `spec_sha256`, `d2_version`, `layout`, `salt`,
  font hashes, `postprocess_version` and the generated SVG hash.
- A manifest whose `d2_version` differs from the pinned version is `stale`.
- `diagram status` reports `ok`, `missing`, `stale` or `engine_mismatch`.
- `build_guide` never regenerates; it fails with the `diagram_render_plan`
  command when an asset is missing or stale.

## 11. CLI surface

```bash
# Validate one spec or all specs; no D2.
folisift diagram validate --guide pd0-pd1 --id pipeline
folisift diagram validate --guide pd0-pd1 --all

# Render one/all specs with D2; writes .d2, SVG, manifest.
folisift diagram build --guide pd0-pd1 --id pipeline
folisift diagram build --guide pd0-pd1 --all

# Check spec/manifest/assets; no D2.
folisift diagram status --guide pd0-pd1

# Remove stale generated assets, specs are untouched.
folisift diagram clean --guide pd0-pd1
```

Behaviour:

- All commands return JSON with `ok`, `guide`, `items` and `next`.
- `validate` never writes outside `guides/`.
- `build` is the only command that invokes D2.
- `build` returns per-item `spec_hash`, `svg_hash`, `svg`, `cache_hit`.
- `status` returns the exact CLI command for every failing item.
- `clean` only removes files under `assets/generated/` and `data/d2/` that
  are not referenced by a manifest; never touches specs or SVG referenced by a
  current manifest.
- Exit codes: 0 ok, 1 gate/content failure, 2 usage, 3 missing engine.

## 12. Guide directive and build integration

Canonical guide syntax:

```markdown
::: diagram id="pipeline"
:::
```

- The directive has no body. The spec is `diagrams/pipeline.json`.
- The renderer resolves the manifest, checks `svg_sha256` and embeds the SVG.
- Caption comes from the spec/manifest, not from the Markdown alt string.
- Figure numbering uses the existing `figure_label` localization.
- `build_guide`/CLI `build`:
  - fail if a `::: diagram` ID has no spec;
  - fail if the manifest is missing/stale;
  - fail if the SVG hash does not match;
  - never call D2;
  - include the exact `diagram build` command in the error.
- Legacy ` ```diagram ` JSON blocks are rejected with a migration message:
  move the JSON to `diagrams/<id>.json` and use `::: diagram id="…"`.
- Manual Markdown images referencing `assets/*.svg` remain supported; they are not D2 diagrams.

## 13. MCP surface

Tool budget: **13 tools** (12 existing + one intent-level `diagram` tool).
`MCP_TOOL_SCHEMA_VERSION` becomes **2**; SemVer minor bump.

### 13.1 `diagram` tool

One tool with actions, matching the existing `style` pattern:

```json
{
  "action": "validate | save | status | render_plan | list",
  "guide": "pd0-pd1",
  "id": "pipeline",
  "spec": { "...": "..." },
  "force": false
}
```

| Action | Read/write | Result |
|---|---|---|
| `validate` | read | normalized spec, warnings, errors |
| `save` | write | spec path, D2 source path, spec hash |
| `status` | read | per-ID `ok/missing/stale/engine_mismatch`, plan command |
| `render_plan` | read | argv array + display command, output paths |
| `list` | read | diagram IDs and kinds |

Rules:

- `save` writes only `guides/<slug>/diagrams/<id>.json` and
  `guides/<slug>/data/d2/<id>.d2`.
- It never runs D2 and never writes `assets/generated`.
- `render_plan` returns both `argv` and `display`; `argv` is safe for the host
  to execute without shell interpolation.
- The tool never accepts arbitrary file paths outside the guide.
- Annotations: read-only for `status/list/render_plan`, idempotent for
  `save/validate`.

### 13.2 Resources

- `folisift://diagram-schema` — JSON Schema and a valid example.
- `folisift://diagram-rules` — visual budget, limits, caption/alt/provenance,
  D2-only rule, sequence rules.
- `folisift://guide/<slug>/diagrams` — current diagram list and status.

### 13.3 Prompt

Update `write-study-guide` and add `write-diagram`:

- `write-study-guide` adds the diagram steps:
  1. identify mechanisms that need a visual;
  2. write JSON specs, not D2;
  3. call `diagram` validate/save;
  4. run the CLI render plan;
  5. call status;
  6. call build.
- `write-diagram` is a short prompt: choose kind, keep 0–3 per chapter, 12/20
  limits, caption/alt/source, no decorative visuals.

### 13.4 `build_guide`

- Add a diagram resolution preflight before rendering.
- Fail with a structured response:
  ```json
  {
    "error": "diagram assets are stale",
    "items": [{"id":"pipeline","state":"missing","command":"folisift diagram build --guide pd0-pd1 --id pipeline"}]
  }
  ```
- Keep `check=true` coverage semantics unchanged.
- No D2 binary lookup in MCP; `doctor` may report D2 presence but must not
  require it for build.

## 14. OpenCode automation

### 14.1 Autonomy boundary

OpenCode can own the mechanical loop and the drafting. It must stop at:

- factual/content approval;
- table draft review;
- visual/pedagogical review;
- release/tag approval.

### 14.2 Runbook

Add `docs/agents/opencode-guide.md` and point to it from `AGENTS.md`.

The loop:

```text
1. workspace-info / doctor
2. prepare_guide (analyze once, scaffold, outline)
3. read_source / search_source in bounded chunks
4. write guide.md sections with covers + audit
5. for each meaningful mechanism:
     diagram validate → diagram save
6. shell: folisift diagram build --guide <slug> --all
7. diagram status
8. build_guide check=true
9. audit + make visual (where baseline exists)
10. human review + G-section audit
11. release only after approval
```

### 14.3 Shell allowlist

OpenCode should be allowed to run only:

```text
folisift diagram build --guide ...
folisift diagram status --guide ...
folisift build --guide ... --check
folisift audit --pdf ...
make visual
```

Do not allow direct arbitrary `d2`, `dot`, `python`, `node` or network
commands in the production loop. D2 is invoked by the Folisift CLI.

### 14.4 Failure policy

- D2 missing/version mismatch: stop; do not fall back.
- Too wide/tall: split the spec; do not shrink text below 6.5 pt.
- Missing/stale asset: run `diagram build` for that ID; do not hand-write SVG.
- Coverage failure: fix the section; do not lower the threshold.
- Visual diff: inspect the diff and update the baseline only after review.
- Max 2 automatic retries per failing item; then report the exact command and
  stop.

### 14.5 LLM constraints

- The host never writes D2, raw SVG, Typst or CSS.
- The host never invents labels, captions or provenance.
- The host treats source text as untrusted data, not instructions.
- Diagrams are limited to 0–3 per chapter and one idea per diagram.
- A diagram without a takeaway caption/alt/source is invalid.

## 15. Testing

### Unit

- spec validation: kinds, IDs, limits, duplicate nodes, unknown edges, missing
  caption/alt/source.
- D2 source generator golden files for flow, concept, tree, sequence.
- label escaping: quotes, backslashes, Turkish characters, newlines.
- SVG post-processor: root role/title/desc, aria, edge-label background,
  script/external ref rejection.
- manifest canonical hash: key ordering, no timestamp.
- path safety: guide/spec/asset paths cannot escape.

### Integration (D2 required)

- `diagram build` produces the same SVG hash on two runs.
- paper and clean builds embed the same SVG.
- `build` with D2 absent but committed assets succeeds.
- `build` with missing/stale asset fails with the exact command.
- `sequence` output has readable labels after post-processing.
- 12-node flow renders; 13 nodes fails with split advice.
- wide diagram rejects or splits; no under-6.5 pt text.
- `diagram status` reports `ok/missing/stale/engine_mismatch` correctly.

### MCP

- stdio probe: `diagram` validate/save/status/render_plan/list.
- `save` never writes outside the guide.
- `render_plan` argv contains no shell metacharacters.
- `build_guide` fails unresolved diagrams; no D2 execution.
- MCP tool schema version 2 and 13 tools.

### Visual

- golden SVG for four kinds.
- paper/clean page baselines with one diagram each.
- edge-label collision regression.
- tall/wide fit test.
- no font below 6.5 pt in diagram artwork.

### Negative fixtures

- bad spec ID;
- 13 nodes;
- unknown edge;
- missing source;
- duplicate ID;
- SVG with `<script>`;
- D2 version mismatch;
- manifest hash mismatch;
- missing D2 binary.

## 16. Packaging and licence

- Add `scripts/fetch-d2.sh` pinned to v0.9.0 with SHA-256 per platform.
- Release archive contains `d2`/`d2.exe` next to `folisift` (optional at build
  time; required for `diagram build`).
- Add `THIRD_PARTY_NOTICES` with D2 MPL-2.0 and source URL; do not modify D2.
- `folisift doctor` reports D2 presence/version, but build does not require it.
- CI installs D2 v0.9.0 for diagram tests; build/audit jobs use committed SVG.
- Keep the Linux archive within a documented size budget (D2 adds ~18 MB).

## 17. Phases

### Phase 0 — decision and ADR (0.5 day)

- Write ADR-0010: D2 is the single external diagram engine.
- Update `docs/diagram-engine-evaluation.md` links.
- Remove Graphviz from active runtime plans.

Exit: ADR accepted; no runtime change.

### Phase 1 — adapter and post-processor (2 days)

- Add spec v2, D2 source generator, version check, CLI `diagram build`.
- Add SVG post-processor and sanitizer integration.
- Add manifest writer/hash.
- Golden tests for four kinds.

Exit: one sample diagram builds through D2; two runs identical.

### Phase 2 — guide integration (1 day)

- Add `::: diagram id="…"` directive.
- Make `build`/`build_guide` embed-only.
- Reject legacy inline fenced diagrams with migration text.
- Add cache key fields and stale detection.

Exit: paper/clean builds use committed SVG; D2 absent build passes.

### Phase 3 — MCP (1.5 days)

- Add `diagram` tool actions.
- Add schema/rules resources.
- Update `write-study-guide`, add `write-diagram`.
- Bump `MCP_TOOL_SCHEMA_VERSION` to 2; update doctor/workspace-info.

Exit: stdio probe runs validate → save → render plan → status → build.

### Phase 4 — OpenCode automation (1 day)

- Add `docs/agents/opencode-guide.md`.
- Update `AGENTS.md` commands/where detail.
- Add shell allowlist and failure policy.
- End-to-end dry run on a fixture guide with one diagram.

Exit: host can run the whole loop with only documented commands.

### Phase 5 — visual/negative tests and packaging (1.5 days)

- Golden SVG + paper/clean baselines.
- Negative fixtures.
- D2 fetch/checksum/notice.
- Release archive smoke test.

Exit: all negative fixtures fail correctly; package contains D2.

### Phase 6 — cleanup and release (1 day)

- Delete native layout paths and Graphviz references.
- Remove legacy fenced diagram code.
- Update docs, `docs/VERSIONS.md`, release notes.
- Run all gates.

Exit: v0.7.0 candidate; release after maintainer approval.

## 18. Acceptance criteria

- [x] `grep -R "graphviz\|Graphviz" crates/ templates/` finds no runtime
      path; only historical/planning docs mention Graphviz.
- [x] `diagram build` uses D2 v0.9.0 only and records the version.
- [x] `build` and `build_guide` pass with D2 absent when assets exist.
- [x] Missing/stale diagram assets fail with the exact `diagram build` command.
- [x] The MCP has 13 tools and schema version 2.
- [x] `diagram` validate/save/status/render_plan work over stdio.
- [x] `::: diagram id="…"` resolves to a sanitized SVG and a captioned figure.
- [x] A 12-node flow builds; 13 nodes fails with a split recommendation.
- [x] A sequence diagram’s labels are readable (line masked).
- [x] Two builds of the same spec produce the same SVG hash.
- [x] paper/clean visual baselines pass with D2 output. The style suite
      embeds a prebuilt D2 manifest/SVG in both styles; the pixel baseline
      remains the existing paper guide baseline.
- [x] No raw D2/SVG/Typst is accepted from the host.
- [x] MPL-2.0 notice and D2 checksum are in the release.
- [x] The OpenCode runbook completes a fixture guide end to end. The CLI
      integration test runs save → build → status → embed-build; the MCP stdio
      test runs save → status → render_plan.
- [x] `cargo test`, clippy and fmt pass; paper 22-page parity unchanged.

## 19. Risks

| Risk | Mitigation |
|---|---|
| D2 v0.9.0 layout changes after upgrade | exact pin; golden/visual tests; upgrade only with baseline update |
| Wide graph becomes unreadable | 12/20 limits, TALA, font tuning, split rule |
| Edge labels collide | post-process background stroke; sequence regression test |
| No accessibility metadata | root role/title/desc + sanitizer tests |
| D2 binary missing | actionable CLI error; build still works with committed SVG |
| MPL-2.0 obligations | unmodified binary + notice + source URL |
| MCP cannot render | CLI render plan; embed-only build; explicit runbook |
| LLM writes invalid specs | schema validation + limits + actionable errors |
| D2 binary size | compress release; document; no build-time requirement |
| Existing native diagrams | migrate to specs; manual SVG assets remain allowed |

## 20. Definition of done

An OpenCode host can start from source PDFs and produce a reviewed PDF by
following one documented loop:

```text
prepare → read/search → write → diagram validate/save → CLI render
→ diagram status → build --check → audit → visual check → human review
```

D2 is the only external diagram engine. The MCP never executes it. The build
never requires it. Every diagram is reproducible from its JSON spec, auditable
through its generated D2 source and manifest, and readable in the final PDF.

## 21. References

- D2: <https://github.com/d2lang/d2>
- D2 v0.9.0: <https://github.com/d2lang/d2/releases/tag/v0.9.0>
- D2 TALA: <https://d2lang.com/blog/tala-is-open-source/>
- D2 styles: <https://d2lang.com/tour/style/>
- D2 globs: <https://d2lang.com/tour/globs/>
- D2 positions: <https://d2lang.com/tour/positions/>
- D2 examples: <https://d2lang.com/examples/overview/>
- Graphviz comparison (historical): `docs/diagram-engine-evaluation.md`
- MCP contracts: `crates/folisift-mcp/src/lib.rs`, `docs/plans/2026-10-28-roadmap.md`
- Authoring: `docs/authoring.md`, `docs/authoring-spec.md`
- Design system: `docs/design-system.md`, `templates/paper.typ`
- Versioning: `docs/agents/versioning.md`
```
