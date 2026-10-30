# Design system — multi-genre

The guide is a study object, not a slide deck. The engine now separates
**genre** (reader, revision sheet, knowledge organiser), **structure** (A–G,
revision-sheet, organiser) and **style** (layout, density, accessibility).
The evidence for these choices is in
[`document-genres-and-layout-systems.md`](document-genres-and-layout-systems.md)
and [`revision-format-research.md`](revision-format-research.md). The visual language is warm,
editorial and print-friendly. The design system is shared by every guide and
lives in `templates/paper.typ` and `templates/fonts/`. See
`docs/adr/0002-paper-design-system.md` and `docs/adr/0007-typst-renderer.md`.

## Palette

| Role | Colour | Hex |
|---|---|---|
| Paper | warm cream | `#FAF6EC` |
| Ink | deep sepia-black | `#211E19` |
| Secondary text | sepia grey | `#5E574B` |
| Primary accent | brick red | `#8A3E2F` |
| Secondary accent | sage green | `#42594F` |
| Tertiary accent | antique gold | `#B08A4F` |
| Rule | linen | `#D8CEBD` |
| Code background | sand | `#F1EADB` |

Rules: no gradients, no stock photos, no neon colours, no heavy shadows, no
rounded “card” interface. Colour marks sections and rules; it never carries
meaning alone. Source references use `faint` `#75664E` (5.17:1 on paper,
4.75:1 on cream); the old `#8A7A63` failed 4.5:1 at small sizes.

## Typography

- Body: Noto Serif (embedded), 9.5 pt, Typst leading 0.62 em (≈1.3 line
  spacing). The old “line-height 1.33” label was ambiguous; Typst `leading` is
  the extra space between lines, not CSS line-height.
- Headings: Inter (embedded, Regular/Medium/SemiBold/Bold), semibold.
- Code: Noto Sans Mono (embedded), 7.4 pt in blocks; DejaVu is only a fallback
  and requires its own licence if shipped.
- Captions: serif italic, 7.9–8.4 pt.
- Labels: Inter, uppercase, letter-spacing 0.12–0.18em.
- Revision layout: Inter 11 pt body, 9.5 pt tables/captions, 8.4 pt
  labels/source refs; 30–32 mm side margins. The legacy paper layout keeps its
  7.9/6.9 pt table components for the v0.9 parity fixture.

Turkish characters and hyphenation must be checked in the final PDF.

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
17 mm margins, 2.6 mm paragraph spacing and a 4.0 mm list rhythm. TOC rows are
slightly tighter so a 20-section guide still fits on one contents page.

It is selected with `aesthetic: "reader"`; `paper` remains the stable
old-PDF parity style. Reader is experimental until its cross-style and
accessibility snapshots pass, but it is the preferred readable default for new
guides; MCP `prepare_guide` reports `style_source: "default-reader"` when it
auto-selects it; revision/organiser structures default to `revision` and report
`"default-revision"`.

## Revision style

`revision` is the exam-revision layout: A4 portrait, 11 pt Inter, 30–32 mm
side margins, left-aligned, 0.72 em leading, 9.5 pt tables/captions, 8.4 pt
labels/source refs. It implements `revision-sheet` (multi-page) and
`knowledge-organiser` (one page per unit). It is not a shortened reader:
the grammar is map → must-know → comparison → worked example → traps →
recall/answer key → one-page synthesis → review plan. See
[`revision-format-research.md`](revision-format-research.md) §7 and
[`document-genres-and-layout-systems.md`](document-genres-and-layout-systems.md)
§4 for the evidence and limits.

## Clean style

`clean` is the second engine-owned layout. It keeps the same content model and
directives but uses an Inter body, white paper, a blue accent, a single cover
rule, denser tables and square technical boxes. It is selected by the style
pack `clean`; it is experimental until its snapshots and cross-style gates
pass. `paper` remains the default and the old-PDF parity standard.

## Presets and accessibility modifiers

`journal`, `brief` and `accessible` are **not genres**:

- `journal` is a `paper` preset (10.5 pt, 18 mm) for IMRAD/literature reviews.
- `brief` is a `clean` preset (10.5 pt, 17 mm) for executive briefs.
- `accessible` is a **large-print modifier**; its `component_scale` (1.3)
  scales body, headings, tables, captions, labels and TOC, not only prose.

They are experimental until their snapshots pass.

## Components

| Component | Use | Style |
|---|---|---|
| Definition | Key term or formal definition | Cream fill, brick left rule |
| Critical distinction | Exam-relevant contrast | Sage left rule, light fill |
| Example | Worked example / code | Top/bottom rule, slightly smaller text |
| Note | Source caveat or added clarification | Gold left rule, small text |
| Data table | Comparisons, criteria, facts | No vertical lines; dark top/bottom; light header |
| Figure | Diagram | Simple SVG; caption below; figure number |
| Code block | Code / grammar / regex | Sand background, gold left rule, wrap long lines |
| Source ref | Traceability | Small caps, `[PD0, s. 3]` |

## Figures and tables

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
