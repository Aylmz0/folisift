# ADR-0015: Source visuals are filtered, classified and provenance-checked

- **Status:** Accepted
- **Date:** 2026-11-05
- **Deciders:** Maintainer

## Context

The A+B policy (ADR-0011 and the v0.20.0 plan) made source crops possible, but
the host still had to find every visual, guess a crop box and decide whether a
visual was worth using. The data-mining PDF shows why page-level evidence is
not enough: page 14 contains four different charts, page 45 mixes a table, a
ROC chart and a D2-rebuildable workflow, and page 71 draws a four-panel
clustering scatter as vector paths. A single page decision is wrong for all
three.

The evidence also distinguishes relevant visuals from decorative ones. Words
plus explanatory pictures generally support learning, while interesting but
irrelevant details can increase liking and reduce learning (Mayer, 2021;
Rey, 2012; Sundararajan & Adesope, 2020; Sung & Mayer, 2012). Data-mining
charts are domain content, not decoration. D2 can rebuild concept diagrams and
qualitative schematics, but it is not a plotting library.

## Decision

1. `visual_candidates` enumerates per-visual candidates with stable ids,
   bounding boxes, bounded context, **relevance** (`core`, `supporting`,
   `decorative`, `seductive`), **reconstruction** (`d2_concept`,
   `d2_schematic`, `table`, `text`, `unique`), an explainable salience score
   and a suggested action.
2. A candidate is `unique` when exact values, thresholds, coordinates or a
   faithful point cloud matter. A D2 **schematic** is allowed only for a
   qualitative relation, must say “şematik” in the caption, must not assert
   values absent from the source, and must carry `schematic_of` when exact
   data matters.
3. Native raster extraction is preferred when one image object is the figure;
   vector/mixed figures use an adaptive 200–500 DPI crop with a 40 MP guard.
   Effective DPI is evaluated at the natural bbox width; upscaling a low-res
   raster is not credited.
4. `visual_decision` materializes the asset, validates blank/resolution/caption
   /alt/hash quality, upserts `visual-audit.json` and returns the exact
   Markdown snippet. It never edits `guide.md`.
5. Assets stay under `data/crops/` or `data/figures/`, are gitignored and are
   local-only. `visual assets --check|materialize|refresh` verifies or
   regenerates them from the stored recipe.
6. The visual gate keeps `off|warn|required`; a soft budget is optional and
   warning-only by default. There is no hard default quota.
7. The MCP server stays LLM-free and network-free. Candidate reports are
   text-only; image bytes are never returned. The host decides, with or
   without vision.

## Consequences

- The host gets a ranked, bounded list instead of reading every page.
- Decorative and seductive visuals are filtered before they consume image
  tokens or page budget.
- Exact charts stay faithful; D2 schematics cannot silently invent data.
- A guide remains reproducible from the source + audit recipe on the same
  PDFium build; a different renderer may require `visual assets --refresh`.
- Visual quality is now part of the build gate, not only a visual review note.
- The remaining gap is semantic importance: the score is salience, not a
  claim that a visual is pedagogically essential. The manual calibration report
  records the reviewed top candidates and known disagreements.
