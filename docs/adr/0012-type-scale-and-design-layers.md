# ADR-0012: One type scale, independent palette and font layers

- **Status:** Accepted
- **Date:** 2026-10-31
- **Deciders:** Maintainer

## Context

The design system had accumulated three drift paths: a global `9.5pt`
replacement in `styles.rs`, a `component_scale` multiplier for accessible
guides, and revision-only literal sizes. The same role therefore meant a
different size in paper, reader, revision and accessible. Palette choices were
baked into a pack's `tokens.json`; there was no way to select a cool or
high-contrast palette without editing a pack.

The audit (A12, A15, B16, B17) and the typography plan called for one role
vocabulary, a small palette library and only embedded font families.

## Decision

1. **One role vocabulary.** Body, H1–H3, lead, table, caption, label, source,
   code block, inline code and TOC roles use ratios and absolute floors. A
   layout selects the body base, not a separate scale.
2. **Revision retune.** Revision body is 10.5 pt Inter, code block 9.7 pt,
   inline code 10.0 pt, table 9.0 pt, labels/source 8.2 pt.
   Reader/PD2 uses 10.0 pt body, 9.2 pt code and 9.5 pt inline code.
3. **Palettes are a library.** `styles/palettes/*.json` defines 14 semantic
   roles. `warm-paper` preserves the old default. A guide selects a palette
   through `style.palette`; packs declare the palettes they accept.
4. **Font pairing is a library.** `styles/fonts/*.json` defines body, heading
   and mono families. Only the embedded Noto Serif, Inter and Noto Sans Mono
   families are allowed. A guide selects a pairing through `style.fonts`.
5. **Paper is a labelled legacy fixture.** It keeps its exact old sizes,
   colours and `editorial` pairing for the parity baseline. New guides use
   reader, revision or organiser styles.
6. **No silent fallback.** Unknown palettes/fonts, unembedded families and
   unsupported combinations fail validation. Low-contrast palettes fail the
   loader, not the reviewer.
7. **No multiplier exception.** The paper table/label migration is complete;
   every layout, including accessible, uses role sizes or explicit pinned
   sizes. `component_scale` was removed.

## Consequences

- A revision sheet can be warm, cool, grayscale or high-contrast without
  changing its grammar or layout.
- A palette or pairing change invalidates the build cache through the resolved
  tokens.
- Paper parity remains measurable and intentional.
- Paper remains a labelled legacy fixture; reader/clean/revision/accessible
  use the shared role scale.
- Adding a new family requires a licence, Turkish glyph, embedding and
  x-height test before it enters `styles/fonts/`.
