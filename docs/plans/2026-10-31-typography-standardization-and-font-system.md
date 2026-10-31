# Folisift v0.10.1 — Typography standardization and font system

- **Status:** Proposed; no code changes until approved
- **Date:** 2026-10-31
- **Baseline:** `v0.10.0` (`8aef654`; 135 tests; revision pilot 4 pages;
  organiser 1 page; paper parity 22 pages)
- **Trigger:** the PD2 revision body is too large; PD2 summary code blocks are
  too small relative to body text; every layout uses a different size logic
- **Follow-on:** [`2026-10-31-d2-capability-and-codebase-hardening.md`](2026-10-31-d2-capability-and-codebase-hardening.md)
- **Glossary:** [`GLOSSARY.md`](../../GLOSSARY.md)
- **Audit:** [`docs/codebase-audit-2026-10-31.md`](../codebase-audit-2026-10-31.md)
- **Research note:** [`docs/typography-standardization-research.md`](../typography-standardization-research.md)
- **Decision in one sentence:** one computed **TypographyScale** for every
  layout; code and tables get proper ratios/floors; palettes and font pairings
  become independent, validated layers instead of hard-coded template values.

## 1. What is wrong now

### 1.1 The two visible failures

| Artifact | Problem | Measurement |
|---|---|---|
| `PD2_Revizyon.pdf` | Body feels too large | 11 pt Inter, x-height 0.546 em → visual body larger than 10 pt Noto Serif |
| `PD2_Revizyon.pdf` | Code too small | 8.9 pt code / 11 pt body = **0.81** |
| `PD2_Calisma_Ozeti.pdf` | Code too small | 7.4 pt block code / 10 pt body = **0.74** |
| `PD2_Calisma_Ozeti.pdf` | Inline code small | 8.4 / 10 = **0.84** |
| `PD3_Calisma_Ozeti.pdf` | Code-heavy pages | 5,114 characters at 7.4 pt code vs 10 pt body |

The user is right on both counts. Code is the content in PD3; at 0.74 body
ratio it is the least readable element on the page.

### 1.2 The structural cause

| Layer | Current behavior | Consequence |
|---|---|---|
| `paper.typ` / `clean.typ` | Hard-coded 6.9/7.4/7.9/8.4/8.9/9.5 pt | No shared scale |
| `revision.typ` | Placeholders for some roles | Revision only partially standardized |
| `styles.rs` | Global `9.5pt` replacement + `component_scale` | Hacks; roles can drift |
| `accessible` | `component_scale: 1.3` | Works, but is an exception, not a system |
| `tokens.json` | Body/leading/palette only | No code/table/caption roles |
| Templates | Font families hard-coded | Multiple fonts/palettes are hard to add safely |

The fix is not “make the revision body 10 pt”. The fix is a single scale with
roles and floors, applied by every template.

## 2. Typography Standard v1

### 2.1 Roles and ratios

| Role | Ratio to body | Floor | Notes |
|---|---:|---:|---|
| Body | 1.00 | 9.0 | genre/density chooses base |
| H1 | 2.00 | 18.0 | chapter/section band |
| H2 | 1.40 | 14.0 | section heading |
| H3 | 1.10 | 11.0 | subheading/label |
| Lead | 1.12 | 11.0 | front/section lead |
| Table | 0.86 | 8.5 | header/body same size |
| Caption | 0.86 | 8.5 | takeaway caption |
| Label | 0.78 | 8.0 | uppercase, 0.08–0.12 em tracking |
| Source ref | 0.78 | 8.0 | contrast ≥ 4.5:1 |
| Code block | **0.92** | **9.0** | monospace, line-height 1.45–1.5 |
| Inline code | **0.95** | **9.0** | slightly larger than block |
| TOC title | 0.95 | 9.0 | |
| TOC sub | 0.88 | 8.5 | |
| TOC page | 0.82 | 8.0 | |

### 2.2 Body sizes by layout

| Layout | Body | Code block | Table | Label/source |
|---|---:|---:|---:|---:|
| `paper` legacy | 9.5 Noto Serif | 7.4 (frozen) | 7.9 (frozen) | 6.9 (frozen) |
| `reader` | 10.0 Noto Serif | **9.2** | **8.6** | **8.0** |
| `clean` | 9.5 Inter | 8.8 | 8.2 | 8.0 |
| `journal` | 10.5 Noto Serif | 9.7 | 9.0 | 8.2 |
| `brief` | 10.5 Inter | 9.7 | 9.0 | 8.2 |
| `revision` | **10.5 Inter** | **9.7** | **9.0** | **8.2** |
| `accessible` | 12.0 Noto Serif | **11.0** | **10.3** | **9.4** |

Revision body goes **11.0 → 10.5**. If the preview still feels large, the next
candidate is 10.0 with 32–34 mm margins; code/tables are not reduced to
compensate.

### 2.3 Revision deltas

| Role | Today | Proposed |
|---|---:|---:|
| Body | 11.0 | 10.5 |
| H1 | 22.0 | 21.0 |
| H2 | 15.4 | 14.7 |
| H3 | 12.1 | 11.6 |
| Lead | 12.3 | 11.8 |
| Table/caption | 9.5 | 9.0 |
| Label/source | 8.4 | 8.2 |
| Code block | 8.9 | **9.7** |
| Inline code | 8.9 | **10.0** |
| TOC | 10.5/9.7/9.0 | 10.0/9.2/8.6 |

### 2.4 Reader/PD2 deltas

| Role | Today | Proposed |
|---|---:|---:|
| Body | 10.0 | 10.0 |
| Table | 7.9 | 8.6 |
| Caption | 7.9 | 8.6 |
| Code block | 7.4 | **9.2** |
| Inline code | 8.4 | **9.5** |
| Label | 6.9 | 8.0 |
| Source ref | 6.9 | 8.0 |
| Note | 8.9 | 9.0 |

PD2 may grow by 1–3 pages because code and tables get larger. That is
acceptable: no fixed page target. Rebuild and inspect; do not shrink code to
keep 23 pages.

## 3. Palettes and fonts as independent layers

### 3.1 Semantic palette roles

```json
{
  "paper": "#FAF6EC",
  "ink": "#211E19",
  "muted": "#5E574B",
  "faint": "#75664E",
  "accent": "#8A3E2F",
  "secondary": "#42594F",
  "gold": "#B08A4F",
  "rule": "#D8CEBD",
  "code_bg": "#F1EADB",
  "cover_bg": "#F3EBDD",
  "cream": "#F4ECDE",
  "zebra": "#F7F1E5",
  "headerfill": "#EFE7D8"
}
```

### 3.2 Palette library (12, not 3)

All core text roles pass 4.5:1 on paper; each palette file also defines
`cream`, `zebra`, `headerfill`, `code_bg` and `cover_bg`, tested against `ink`.

| Id | Family | Paper | Ink | Accent | Faint | Worst text |
|---|---|---|---:|---:|---:|---:|
| `warm-paper` | warm/editorial | `#FAF6EC` | `#211E19` | `#8A3E2F` | `#75664E` | 5.17 |
| `sepia-archive` | warm/archive | `#F4ECD8` | `#2B2118` | `#7B2D26` | `#7A6650` | 4.64 |
| `clay` | warm/earthy | `#FAF6F2` | `#2E211C` | `#9C4A2F` | `#7A655B` | 5.09 |
| `berry` | warm/plum | `#FBF7F8` | `#2B1E24` | `#7A2E4A` | `#78656E` | 5.09 |
| `cool-slate` | cool/technical | `#FFFFFF` | `#0F172A` | `#1D4ED8` | `#64748B` | 4.76 |
| `academic-navy` | cool/academic | `#FBF8F1` | `#14213D` | `#1D3557` | `#6B7280` | 4.56 |
| `cobalt` | cool/strong | `#F8FAFC` | `#0F172A` | `#1E3A8A` | `#64748B` | 4.55 |
| `forest-sage` | natural | `#F7F5EF` | `#1F2A24` | `#2F5D50` | `#5F7265` | 4.72 |
| `ocean-teal` | natural/cool | `#F7FBFC` | `#0B2B33` | `#0F6E7A` | `#527078` | 4.82 |
| `graphite` | neutral | `#FAFAFA` | `#111111` | `#374151` | `#6B7280` | 4.63 |
| `mono-print` | grayscale | `#FFFFFF` | `#000000` | `#000000` | `#555555` | 7.46 |
| `high-contrast` | accessibility | `#FFFFFF` | `#000000` | `#A40000` | `#4D4D4D` | 8.14 |

Palette and genre are independent: a revision sheet can be warm, cool or
grayscale. Palette is not a “mood”; it is a contrast-tested semantic layer.

### 3.3 Palette rules

- Validate every text/background pair, not only `ink/paper`.
- Print a grayscale proof; if categories differ only by hue, add a label,
  pattern, icon or rule.
- Do not use red/green as the only distinction. Colour-blind prevalence is
  about 1 in 12 men and 1 in 200 women of Northern European descent.
- Colour psychology is context-dependent; do not claim “blue improves focus”.
  Use colour for hierarchy and signalling (Elliot & Maier, 2014).
- `gold` stays a rule/accent colour unless a darker text variant passes 4.5:1.

### 3.4 Font pairing library

| Id | Body | Heading | Mono | Status | Use |
|---|---|---|---|---|---|
| `editorial` | Noto Serif | Inter | Noto Sans Mono | embedded, verified | paper, reader, journal |
| `technical` | Inter | Inter | Noto Sans Mono | embedded, verified | clean, revision, brief |
| `academic` | Source Serif 4 | Source Sans 3 | Source Code Pro | candidate, OFL; Turkish glyph test required | academic reader/report |
| `accessible` | Atkinson Hyperlegible Next | Atkinson Hyperlegible Next | Noto Sans Mono | candidate, OFL; Turkish glyph test required | large-print |
| `plex` | IBM Plex Serif | IBM Plex Sans | IBM Plex Mono | candidate, OFL; Turkish glyph test required | technical/academic |

Do not add a pairing until it passes licence, embedding, Turkish glyph and
x-height tests. For v0.10.1, keep the embedded trio; add one candidate only
after the palette/scale work is stable.

### 3.5 Why three families, not more

A serif body, sans heading and mono code cover the functional contrasts:
reading texture, structural clarity, literal code. A fourth family adds
package weight and visual noise without a reading task. If a new pairing is
needed, it replaces one of the three, not adds to it.

### 3.6 Evidence map by style

“Proven style” must mean proven **learning technique** or accessibility standard,
not a claim that a palette or font raises exam scores.

| Style/genre | Evidence | Strength | What is actually supported |
|---|---|---|---|
| Long-form reader | Signalling, coherence, dual coding, segmenting | strong/moderate | Headings, captions, adjacent visuals, no decoration; prose remains appropriate |
| Revision sheet | Retrieval, spacing, concept maps, worked examples, graphic organizers | strong/moderate | Recall prompts + answers, maps, matrices, faded examples, review plan |
| Knowledge organiser | Graphic organizers, concept maps; practitioner genre | moderate | One-page structure, labelled map/timeline, key terms; no proven “one-page effect” |
| Guided notes | Guided notes d≈0.55; note-taking mixed | moderate | Provided framework + blanks + cues + summary |
| Workbook | Worked examples, retrieval, self-explanation, ICAP | strong/moderate | Full → faded → independent; answer key; mixed review |
| Reference sheet | Information design/typography | standard | Lookup speed, units, conditions, contrast; not first learning |
| Academic paper | Genre conventions (Swales, Hyland) | standard | IMRAD/argument/citations; not a study technique |
| Executive brief | Minto/BLUF | professional standard | Answer-first, findings, recommendation, risks |
| Clinical/legal note | SOAP/IRAC | professional standard | Facts/rules/assessment/plan; not exam revision |
| Palette/font choice | WCAG/PDF/UA, x-height, typography practice | standard/heuristic | Contrast, legibility, hierarchy; no learning-outcome claim |

### 3.7 Layout × palette × font matrix

| Layout | Default palette | Default fonts | Allowed palettes | Allowed fonts |
|---|---|---|---|---|
| `paper` legacy | `warm-paper` | `editorial` | warm, neutral (frozen) | editorial (frozen) |
| `reader` | `warm-paper` | `editorial` | all 12 | editorial, academic |
| `clean` | `cool-slate` | `technical` | all cool/neutral | technical, plex |
| `journal` | `academic-navy` | `editorial` | all academic/warm | editorial, academic |
| `brief` | `cool-slate` | `technical` | all cool/neutral | technical, plex |
| `revision` | `warm-paper` | `technical` | all 12 | technical, accessible |
| `accessible` | `high-contrast` | `accessible` | high-contrast, mono-print | accessible |

This is the actual “multiple styles” model: a genre selects the layout and
structure; a palette and font pairing are independent, validated choices. The
style id is not a fourth dimension that silently changes all three.

## 4. Architecture

### 4.1 Target model

```text
style pack
  ├── genre: reader | revision-sheet | knowledge-organiser | ...
  ├── layout: paper | clean | revision
  ├── palette: warm-paper | cool-slate | academic-navy | ... | high-contrast
  ├── fonts: editorial | technical | academic | accessible | plex
  ├── density: compact | comfortable | spacious
  └── accessibility: standard | large-print
```

### 4.2 `TypographyScale`

Add a `TypographyScale` in `crates/folisift-render/src/styles.rs`:

```rust
struct TypographyScale {
    body: f64,
    h1: f64,
    h2: f64,
    h3: f64,
    lead: f64,
    table: f64,
    caption: f64,
    label: f64,
    source: f64,
    code_block: f64,
    code_inline: f64,
    toc_title: f64,
    toc_sub: f64,
    toc_page: f64,
}
```

Tokens:

```json
{
  "typography": {
    "body_size": 10.5,
    "leading_em": 0.72,
    "scale": {
      "h1": 2.0, "h2": 1.4, "h3": 1.1, "lead": 1.12,
      "table": 0.86, "caption": 0.86,
      "label": 0.78, "source": 0.78,
      "code_block": 0.92, "code_inline": 0.95,
      "toc_title": 0.95, "toc_sub": 0.88, "toc_page": 0.82
    },
    "floor": {
      "component_pt": 8.0,
      "table_pt": 8.5,
      "code_pt": 9.0,
      "source_pt": 8.0
    }
  }
}
```

### 4.3 Templates

`paper.typ`, `clean.typ` and `revision.typ` use placeholders only:

```typst
#set text(size: __BODY_SIZE__)
#set raw(... size: __CODE_SIZE__)
#set table(... size: __TABLE_SIZE__)
```

No global `9.5pt` replacement. No `component_scale` exception. `accessible`
uses body 12 and the same ratios/floors.

### 4.4 Validation

- body 9–18 pt;
- table/code ≥ 9.0 in reader/revision/accessible; labels/source ≥ 8.0;
- `code_block / body` 0.90–0.95;
- `table / body` 0.84–0.90;
- `body / smallest_component ≤ 1.45` (revision/accessible ≤ 1.35);
- every text role ≥ 4.5:1;
- no code line overflow; audit reports code font size and ratio.

## Implementation status — 2026-10-31

Implemented in the working tree:

- shared role placeholders and code roles (`code_block`, `code_inline`);
- revision 10.5 pt body, 9.7/10.0 pt code, 9.0 pt tables, 8.2 pt labels;
- reader/PD2 9.2/9.5 pt code and accessible high-contrast default;
- 12 contrast-tested palettes and `editorial`/`technical` pairings, selectable
  through `style.palette` and `style.fonts`;
- style-pack schema 2 with allowed palette/font lists; paper is frozen;
- heading/mono font wiring and all palette roles;
- CLI `styles palettes`, `styles fonts`, `styles info/preview --palette --fonts`;
- full text-role contrast validation and an EN revision/organiser label test;
- paper/reader/clean/accessible role placeholders; `component_scale` removed;
- audit metrics: body/code size, code/body ratio, font histogram, line p50/p90,
  long-line pages; `make type-check` gate;
- palette contrast/grayscale proof (`make palette-check`);
- package archive and clean-directory `doctor` smoke check.

Still open:

- candidate font families (`academic`, `accessible`, `plex`) pending licence,
  glyph, embedding and x-height tests; they are intentionally not selectable.

## 5. Implementation phases

### Phase 0 — Baseline and fixtures

- [x] Record current font histograms for PD0–PD1, PD2, PD3, revision and
      organiser.
- [x] Add a typography fixture with body, H1–H3, lead, table, caption, label,
      source, code block, inline code and TOC.
- [x] Add a dev script `scripts/check-type-scale.py` (PyMuPDF) that reports
      histogram, code/body ratio and line-length p50/p90.
- **Exit:** measured baseline committed; no layout changes.

### Phase 1 — Shared scale and code roles

- [x] Add `TypographyScale` and placeholder replacement in `styles.rs`.
- [x] Add `code_block`/`code_inline` tokens and placeholders to paper/clean/
      revision.
- [x] Wire `heading` and `mono` font tokens; replace only explicit
      `__BODY_FONT__`/`__HEADING_FONT__`/`__MONO_FONT__` placeholders.
- [x] Wire all palette roles (`faint`, `cream`, `sand`, `zebra`, `headerfill`)
      in every template; no silent drop.
- [x] Remove `component_scale` from accessible; use the shared scale.
- [x] Keep `paper` legacy values (parity) unless the parity decision changes.
- [x] Add unit tests for ratios/floors and code/body ratio.
- **Exit:** all templates render; paper parity holds; accessible components
      scale without a special multiplier.

### Phase 2 — Retune revision

- [x] `styles/revision/tokens.json`: body 10.5, code 9.7, table 9.0,
      label/source 8.2, TOC 10.0/9.2/8.6.
- [x] Rebuild `output/PD2_Revizyon.pdf` and `style-preview-revision-{tr,en}.pdf`.
- [x] Measure line p90 ≤ 82 (revision is segmented, not a prose reader);
      inspect code blocks at 100% print size.
- [x] Update the pilot if the body still feels large: next candidate 10.0 with
      32–34 mm margins.
- **Exit:** revision body visibly smaller, code visibly larger, audit passes.

### Phase 3 — Retune reader/PD2 and PD3

- [x] `styles/reader/tokens.json`: code 9.2, inline 9.5, table 8.6,
      caption 8.6, label/source 8.0, note 9.0.
- [x] Rebuild PD2 and PD3; confirm code blocks are readable and no overflow
      or orphan headings.
- [x] Accept page-count changes; update `audit --expected-pages` and docs.
- [x] Do not change body 10.0 unless the measure review says so.
- **Exit:** PD2/PD3 code/body ratio ≥ 0.90; no code overflow; strict audit.

### Phase 4 — Palette and font library

- [x] Add `styles/palettes/` with 12 semantic palettes:
      `warm-paper`, `sepia-archive`, `clay`, `berry`, `cool-slate`,
      `academic-navy`, `cobalt`, `forest-sage`, `ocean-teal`, `graphite`,
      `mono-print`, `high-contrast`.
- [x] Add `styles/fonts/` with `editorial`, `technical`.
- [ ] Add `academic`, `accessible` and `plex` only after licence/glyph/x-height
      tests; intentionally not selectable yet.
- [x] `StylePack` schema 2: optional `palette`/`fonts`; `typography.scale` in
      tokens; default from
      layout.
- [x] Contrast tests for every text role; grayscale proof for every palette.
- [x] Add `palette list`/`palette preview` CLI (or `styles info` fields).
- **Exit:** all 12 palettes load; paper/reader/revision can share a palette;
      clean can use cool; accessible can use contrast/12 pt.

### Phase 5 — Audit and gates

- [x] `font_sizes` histogram and `body_font_size` in `LayoutAudit`.
- [x] Code font size and code/body ratio from `NotoSansMono` spans.
- [x] `line_chars_p50/p90`, `long_line_pages`.
- [x] `audit --guide <slug>` reads the style scale/floors.
- [x] `make type-check` and `make revision-check`.
- [x] Add EN revision/organiser label parity test (B13).
- [x] Validate all text-role contrast, not only `ink/paper` (B17).
- **Exit:** gates fail on 0.74 code ratio or 6.9 pt labels in new styles;
      paper legacy is explicitly exempt; EN labels contain no Turkish defaults.

### Phase 6 — Docs and release candidate

- [x] Update `docs/design-system.md` with the type-scale table, code roles,
      palettes and font pairings.
- [x] Update `docs/authoring.md`/`authoring-evidence.md`/`authoring-spec.md`
      links and values.
- [x] Add an ADR: one scale, multiple palettes/fonts, paper legacy exemption.
- [x] Rebuild previews, pilots, PD2/PD3, TOC, audits, package checksum and
      clean install.
- **Exit:** v0.10.1 release candidate.

## 6. Acceptance criteria

### Typography

- [x] Every layout uses the same role names and ratio rules.
- [x] Revision body is 10.5 pt Inter (or 10.0 with a recorded measure check).
- [x] Revision code block ≥ 9.5 pt; inline ≥ 9.8 pt.
- [x] Reader code block ≥ 9.0 pt; inline ≥ 9.3 pt.
- [x] Table ≥ 8.5 pt; label/source ≥ 8.0 pt in new styles.
- [x] Code/body ratio 0.90–0.95 in reader/revision/accessible.
- [x] No global `9.5pt` replacement; no `component_scale` exception.
- [x] Accessible 12 pt body scales all components through the shared scale.

### Palettes/fonts

- [x] Palette roles are semantic and contrast-tested.
- [x] All 12 palettes load and can be selected independently of genre; every
      text role passes 4.5:1 and a grayscale proof.
- [x] Font pairing is a token layer; only embedded families are allowed.
- [x] No fourth family without licence/x-height/package review.

### Artifacts

- [x] `PD2_Revizyon.pdf` body smaller, code larger; strict audit passes.
- [x] `PD2_Calisma_Ozeti.pdf` code blocks ≥ 9 pt; no overflow; audit passes.
- [x] `PD3_Calisma_Ozeti.pdf` code-heavy pages readable; audit passes.
- [x] `style-preview-revision-{tr,en}.pdf` pass.
- [x] Paper parity still passes, or an ADR retires exact parity.

### Regression

- [x] 135+ tests, clippy, fmt.
- [x] TOC links and production report.
- [x] Package checksum and clean install.
- [x] No stale docs: `line_height`, `component_scale`, 6.9 pt labels or 7.4 pt
      code claims must be removed or marked legacy.

## 7. Decisions requested

1. **Revision body:** 10.5 pt (recommended) or 10.0 pt with 32–34 mm margins?
2. **Code ratio:** 0.92 block / 0.95 inline (recommended) or 0.90/0.93?
3. **Paper parity:** keep `paper` legacy frozen and standardize `reader`/
   `clean`/`revision` (recommended), or migrate paper and retire exact parity?
4. **Palettes:** implement all 12 in v0.10.1, or start with warm/editorial +
   cool/technical + high-contrast and add the rest in v0.11?
5. **Fonts:** keep the current three embedded families for v0.10.1
   (recommended), or add a second pairing such as IBM Plex in v0.11?

No code will change until these are confirmed. The first implementation step
is Phase 0 (baseline) and Phase 1 (shared scale), with a hard stop if paper
parity cannot be preserved.
