# Diagram rules

- 0-3 diagrams per chapter; one idea per diagram.
- 1-24 nodes, max 40 edges, tree/decision depth max 8.
- Use flow for processes, concept for relationships, tree/decision for hierarchy, sequence for messages, class/erd for structural models.
- Layouts: tala, dagre, elk. Record the layout in the manifest.
- `aspect_ratio` is a direction hint when `direction` is absent.
- `label_lines` is the preferred multiline label; escaped `\n` also renders.
- `class_styles` + `node_meta.class` colour legal outcome branches; `grid` builds matrix containers; `vars` avoids repeated literals; local `imports` are under data/d2.
- Node/group `width`/`height` and spec-level `width`/`max_height`/`fit` are validated, not ignored.
- `allow_page_span` is required when the geometric fit exceeds one page.
- Sketch rendering is opt-in and off by default for legal/medical guides.
- No decorative images, stock photos or AI illustrations.
- Every diagram needs a takeaway caption, an alt text and provenance.
- The host writes JSON only; D2, SVG and Typst are engine-owned.
- Builds read pre-generated SVG; the CLI `folisift diagram build` is the only D2 caller.
