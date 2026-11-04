# Design system — multi-genre

The guide is a study object, not a slide deck. The engine now separates
**genre** (reader, revision sheet, knowledge organiser), **structure** (A–G,
revision-sheet, organiser) and **style** (layout, density, accessibility).
The evidence for these choices is in
[`document-genres-and-layout-systems.md`](document-genres-and-layout-systems.md),
[`revision-format-research.md`](revision-format-research.md) and
[`typography-standardization-research.md`](typography-standardization-research.md).
The 12-palette/2-pairing library, the shared scale and the code-size standard
are implemented in `styles/palettes/`, `styles/fonts/` and
[`plans/2026-10-31-typography-standardization-and-font-system.md`](plans/2026-10-31-typography-standardization-and-font-system.md).
The default visual language is warm, editorial and print-friendly; palette and
font pairing are independent token layers. The design system is shared by
every guide and lives in `templates/paper.typ`, `templates/revision.typ` and
`templates/fonts/`. See `docs/adr/0002-paper-design-system.md`,
`docs/adr/0007-typst-renderer.md` and `docs/adr/0012-type-scale-and-design-layers.md`.

## Palette library

A palette is a semantic, contrast-tested layer, not a mood. It always defines
the same 14 roles: `paper`, `ink`, `muted`, `soft`, `faint`, `accent`,
`secondary`, `gold`, `rule`, `code_bg`, `cover_bg`, `cream`, `zebra` and
`headerfill`. Text roles (`ink`, `muted`, `soft`, `faint`, `accent`,
`secondary`) must reach 4.5:1 on `paper`; `ink` must also reach 4.5:1 on every
surface. The loader rejects a palette that fails either test.

| Id | Family | Paper | Ink | Accent | Use |
|---|---|---|---:|---:|---|
| `warm-paper` | warm/editorial | `#FAF6EC` | `#211E19` | `#8A3E2F` | default paper, reader, revision |
| `sepia-archive` | warm/archive | `#F4ECD8` | `#2B2118` | `#7B2D26` | archival or historical sources |
| `clay` | warm/earthy | `#FAF6F2` | `#2E211C` | `#9C4A2F` | humanities, fieldwork |
| `berry` | warm/plum | `#FBF7F8` | `#2B1E24` | `#7A2E4A` | literature, language |
| `cool-slate` | cool/technical | `#FFFFFF` | `#111827` | `#1D4ED8` | clean, brief |
| `academic-navy` | cool/academic | `#FBF8F1` | `#14213D` | `#1D3557` | journal, formal reports |
| `cobalt` | cool/strong | `#F8FAFC` | `#0F172A` | `#1E3A8A` | technical reports |
| `forest-sage` | natural | `#F7F5EF` | `#1F2A24` | `#2F5D50` | biology, environmental science |
| `ocean-teal` | natural/cool | `#F7FBFC` | `#0B2B33` | `#0F6E7A` | medicine, earth science |
| `graphite` | neutral | `#FAFAFA` | `#111111` | `#374151` | print-safe neutral |
| `mono-print` | grayscale | `#FFFFFF` | `#000000` | `#000000` | monochrome printing |
| `high-contrast` | accessibility | `#FFFFFF` | `#000000` | `#A40000` | large print, low vision |

`gold` is a rule/accent colour, not a text colour: it does not carry meaning
without a label, and it is not required to pass 4.5:1. Never use red/green as
the only distinction. A grayscale proof must keep every category distinct; if
it does not, add a label, rule or pattern.

The old warm palette is unchanged and remains the default for the existing
guides. A guide selects another palette explicitly:

```json
"style": {
  "aesthetic": "reader",
  "palette": "academic-navy",
  "fonts": "editorial"
}
```

Paper is frozen: it accepts only `warm-paper` with `editorial`, so the v0.9
parity fixture keeps its exact colours and fonts. Other layouts
accept the palettes and pairings listed in their `style.json`.

## Typography

One role vocabulary, one ratio rule; the layout chooses only the body base.
Body 9.5–12 pt is legal, component floors are absolute. `paper` is a labelled
legacy fixture and keeps its old 6.9/7.4/7.9 pt components; new styles use the
improved scale.

| Role | Ratio to body | Floor | Paper legacy | Reader | Clean/brief | Revision | Organiser |
|---|---:|---:|---:|---:|---:|---:|---:|
| Body | 1.00 | 9.0 | 9.5 | 10.0 | 9.5 / 10.5 | 10.5 | 10.5 |
| H1 | 2.00 | 8.0 | 19.0 | 20.0 | 19.0 / 21.0 | 21.0 | 21.0 |
| H2 | 1.40 | 8.0 | 12.0 | 14.0 | 14.0 / 14.7 | 14.7 | 14.7 |
| H3 | 1.10 | 8.0 | 8.2 | 11.0 | 11.0 / 11.6 | 11.6 | 11.6 |
| Lead | 1.12 | 8.0 | 10.8 | 11.2 | 11.2 / 11.8 | 11.8 | 11.8 |
| Table | 0.86 | 8.5 | 7.9 | 8.6 | 8.5 / 9.0 | 9.0 | 9.0 |
| Caption | 0.86 | 8.5 | 7.8 | 8.6 | 8.5 / 9.0 | 9.0 | 9.0 |
| Label/source | 0.78 | 8.0 | 6.9 | 8.0 | 8.0 / 8.4 | 8.2 | 8.2 |
| Code block | 0.92 | 9.0 | 7.4 | 9.2 | 9.2 | 9.7 | 9.7 |
| Inline code | 0.95 | 9.0 | 8.4 | 9.5 | 9.5 | 10.0 | 10.0 |
| TOC title | 0.95 | 8.0 | 9.6 | 9.6 | 9.6 | 10.0 | 10.0 |
| TOC sub | 0.88 | 8.0 | 8.9 | 8.9 | 8.9 | 9.2 | 9.2 |
| TOC page | 0.82 | 8.0 | 8.2 | 8.2 | 8.2 | 8.6 | 8.6 |

Paper is the only frozen legacy column. Reader, clean, brief, accessible,
revision and organiser now use the same role names; paper keeps its old
components for the parity fixture and is not the target scale.

- Body: Noto Serif in editorial/paper layouts, Inter in `technical` layouts.
- Headings: Inter (Regular/Medium/SemiBold/Bold), semibold.
- Code: Noto Sans Mono; DejaVu is only a fallback and requires its own licence
  if shipped.
- Captions: serif italic in editorial layouts; sans in technical layouts.
- Labels: uppercase, letter-spacing 0.12–0.18em.
- Turkish characters and hyphenation must be checked in the final PDF.
- Reader code/body ratio is 0.92–0.95; revision is 0.92–0.95. Accessible is a
  large-print modifier and keeps its component scaling. A new style below
  0.90 is a regression.

### Font pairings

| Id | Body | Heading | Mono | Status |
|---|---|---|---|---|
| `editorial` | Noto Serif | Inter | Noto Sans Mono | embedded, verified |
| `technical` | Inter | Inter | Noto Sans Mono | embedded, verified |

Only embedded families are allowed. `academic`, `accessible` and `plex` remain
candidates until they pass licence, Turkish glyph, embedding and x-height
tests; they are not selectable in this release.

## Page layout

- A4, margins `16mm 16.5mm 14mm 16.5mm`.
- Single-column body; the C glossary uses two columns with a 7–10 mm gutter.
- Running header: chapter left, “Programlama Dilleri · Çalışma Kılavuzu” right;
  no separate header rule. The brick chapter-opener rule follows it.
- Footer: page number centred, brick red.
- Cover: darker cream `#F3EBDD`, double thin rule, 33 pt serif title, 17 pt
  italic subtitle, pilot badge and syntax-tree motif. No photograph.
- Section opener: top rule, small meta line, large title, short lead. The
  opener must not sit alone at the bottom of a page.

## Reader style

`reader` is the spacious paper variant for new long-form guides. It keeps the
paper layout and palette but uses a 10.0 pt Noto Serif body, 0.70 em leading,
17 mm margins, 2.6 mm paragraph spacing and a 4.0 mm list rhythm, 9.2 pt code
blocks and 9.5 pt inline code. TOC rows are slightly tighter so a 20-section
guide still fits on one contents page.

It is selected with `aesthetic: "reader"`; `paper` remains the stable
old-PDF parity style. Reader is experimental until its cross-style and
accessibility snapshots pass, but it is the preferred readable default for new
guides; MCP `prepare_guide` reports `style_source: "default-reader"` when it
auto-selects it; revision/organiser structures default to `revision` and report
`"default-revision"`.

## Revision style

`revision` is the exam-revision layout: A4 portrait, 10.5 pt Inter, 30–32 mm
side margins, left-aligned, 0.72 em leading, 9.0 pt tables/captions, 8.2 pt
labels/source refs, 9.7 pt code blocks and 10.0 pt inline code. It implements `revision-sheet` (multi-page) and
`knowledge-organiser` (one page per unit). It is not a shortened reader:
the grammar is map → must-know → comparison → worked example → traps →
recall/answer key → one-page synthesis → review plan. See
[`revision-format-research.md`](revision-format-research.md) §7 and
[`document-genres-and-layout-systems.md`](document-genres-and-layout-systems.md)
§4 for the evidence and limits. The standardized type scale and code sizes are
specified in
[`plans/2026-10-31-typography-standardization-and-font-system.md`](plans/2026-10-31-typography-standardization-and-font-system.md).

## Clean style

`clean` is the second engine-owned layout. It keeps the same content model and
directives but uses an Inter body, white paper, a blue accent, a single cover
rule, denser tables and square technical boxes. It is selected by the style
pack `clean`; it is experimental until its snapshots and cross-style gates
pass. `paper` remains the default and the old-PDF parity standard.

## Output intent and page budget

The reader task comes before the layout. `style recommend` accepts an
`intent` and returns a style, structure and page budget:

| Intent | Output | Default budget | Primary evidence |
|---|---|---|---|
| `exam-summary` | revision sheet, visual/tabular | 2–4 pages | retrieval, graphic organizers, signaling |
| `exam-revision` | full revision sheet | 6–10 pages | retrieval, spacing, worked examples |
| `concept-summary` | explanatory prose + table/figure | 4–8 pages | coherence, dual coding, worked examples |
| `study` | reader/A–G | 10–18 pages | signaling, segmenting, prose coherence |
| `deep` | reader/A–G | 16–28 pages | mechanisms, assumptions, expertise reversal |
| `reference` | formula/reference sheet | 2–8 pages | information design, lookup speed |
| `practice` | workbook | 6–12 pages | worked examples, retrieval, ICAP |
| `one-page` | knowledge organiser | 1 page | graphic organizers, coherence |

An intent is not a font size. It changes the content grammar and the page
budget. `target_pages` is enforced after rendering; a guide outside its budget
fails the build. A summary must declare `source_scope` instead of claiming
A–G coverage.

## Review artifacts

The PDF stays the primary artifact. Two sidecars support review without
changing the layout:

- **Claim spot-check** (`claims`): high-signal prose claims with a source page
  or a missing-reference marker. It is a reading aid, not a truth verdict.
- **Recall cards** (`cards`): `::: recall` / `::: answer` pairs exported as
  `front,back,tags,source` CSV with `review-<days>d` tags. The schedule comes
  from `review_intervals`; the review app owns mutable per-card state.

Neither sidecar adds decorative UI to the PDF. Exam sheets keep recall and
answer separate; concept summaries may use self-check prompts without turning
into flashcard decks.

## Notes, workbook and reference

These are full genres, not density variants:

- **notes** (`lecture-notes`): cue/notes rows, a dedicated notes workspace and
  a summary band. Cues are on the left, notes on the right; the summary closes
  the page. No pre-filled answer key.
- **workbook** (`workbook`): numbered problem blocks, ruled workspace, a
  worked/faded example, practice and a separate answer key.
- **reference** (`formula-sheet`): formula cards with source lines, symbol and
  unit tables, and a pitfalls block. Prose is lookup-sized, not chapter prose.

All three use the shared role scale, palette/font libraries, source_scope
coverage and preview gates. `styles preview notes|workbook|reference` renders a
complete fixture; `make genre-visual` checks their PDF baselines.

## Presets and accessibility modifiers

`journal`, `brief` and `accessible` are **not genres**:

- `journal` is a `paper` preset (10.5 pt, 18 mm) for IMRAD/literature reviews.
- `brief` is a `clean` preset (10.5 pt, 17 mm) for executive briefs.
- `accessible` is a **large-print modifier**; its default palette is
  `high-contrast`, its 12 pt body drives the shared role scale, and its
  component floors are raised (9.0 pt text, 10.0 pt tables). It does not use a
  separate multiplier.

They are experimental until their snapshots pass.

## Components

| Component | Use | Style |
|---|---|---|
| Definition | Key term or formal definition | Cream fill, brick left rule |
| Critical distinction | Exam-relevant contrast | Sage left rule, light fill |
| Example | Worked example / code | Top/bottom rule, slightly smaller text |
| Note | Source caveat or added clarification | Gold left rule, small text |
| Answer key | Separated recall answer | Cream fill, sage left rule |
| Data table | Comparisons, criteria, facts | No vertical lines; dark top/bottom; light header |
| Glossary | Term/definition pairs | One four-column table; `###` groups are full-width pale-sage rows |
| Figure | Diagram | Simple SVG; caption below; figure number |
| Code block | Code / grammar / regex | Sand background, gold left rule, wrap long lines |
| Source ref | Traceability | Small caps, `[SOURCE, s. 3]` |

## Figures and tables

D2 now supports concept/flow, tree, sequence, UML class and SQL/ERD payloads,
edge arrowheads and solid/dashed lines, a small shape allowlist, deterministic
TALA layout and a scrubbed environment. Icons, links, imports, layers,
scenarios, animations and remote assets are rejected. See
[`plans/2026-10-31-d2-capability-and-codebase-hardening.md`](plans/2026-10-31-d2-capability-and-codebase-hardening.md).


- Redraw important diagrams as simple SVG. Do not paste screenshots unless the
  original image is the only faithful representation.
- Describe what the figure demonstrates in the caption or the surrounding text.
- Tables preserve meaning, not necessarily every cell. Explain checkmarks,
  values and columns.
- Large tables should move to an appendix or be summarised, not shrunk to
  unreadable type.

## Code and formulas

- Preserve identifiers, notation and control flow.
- Boilerplate may be shortened with `...`; say so in the caption.
- BNF, regex and grammar rules use monospace, not fake math.
- Any added formula or clarification is labelled **Açıklayıcı not**.

## Guide-specific assets

Put diagrams used by one guide in `guides/<slug>/assets/`. Put brand assets
(cover motif) in `templates/`.
Do not fork the shared stylesheet or template for one guide; if a visual
exception is needed, document it and keep the change local.

## Print checks

- Fonts embedded; Turkish glyphs present.
- No content overflows the page or the code block.
- No section opener alone at page bottom.
- TOC page numbers are internal links and PDF bookmarks are correct.
- G counts match the inventory.
