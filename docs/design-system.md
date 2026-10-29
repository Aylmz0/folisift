# Design system — “paper”

The guide is a study object, not a slide deck. The visual language is warm,
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
meaning alone.

## Typography

- Body: Noto Serif (embedded), 9.5 pt, line-height 1.33.
- Headings: Inter (embedded, Regular/Medium/SemiBold/Bold), semibold.
- Code: Noto Sans Mono (embedded), 7.4 pt in blocks; DejaVu is only a fallback
  and requires its own licence if shipped.
- Captions: serif italic, 7.9–8.4 pt.
- Labels: Inter, uppercase, letter-spacing 0.12–0.18em.

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
guides.

## Clean style

`clean` is the second engine-owned layout. It keeps the same content model and
directives but uses an Inter body, white paper, a blue accent, a single cover
rule, denser tables and square technical boxes. It is selected by the style
pack `clean`; it is experimental until its snapshots and cross-style gates
pass. `paper` remains the default and the old-PDF parity standard.

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
