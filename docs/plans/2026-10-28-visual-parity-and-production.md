# Folisift v0.3 — Visual Parity and Production Readiness Plan

- **Status:** Draft for approval
- **Date:** 2026-10-28
- **Reference:** `output/old-python.pdf` (22 pages, WeasyPrint, 209 KB)
- **Current:** `output/PD0-PD1_Calisma_Ozeti.pdf` (23 pages, Typst, 420 KB)
- **Related:** `docs/plans/2026-10-26-mcp-v0.2-plan.md`,
  `docs/plans/2026-10-26-rust-mcp-typst.md`, ADR-0002, ADR-0005–0009

## 1. Goal

Make the Rust/Typst output **visually and structurally indistinguishable** from
the old Python/WeasyPrint output while keeping the v0.2 MCP architecture:

- same 22-page structure;
- same cover, front matter, TOC, chapter openers and running header/footer;
- same C two-column glossary;
- same table rules, header treatment, widths and zebra rows;
- same figures, captions, SVG sizing and source references;
- same Turkish content and source coverage;
- same ease of installation: binary + `libpdfium.so` sidecar;
- no broken install, unclear MCP flow or false coverage pass.

“Perfect” here means **all parity gates pass**, not a claim that every
anti-aliased pixel is identical. The old PDF is the reference, not a license to
stop thinking.

## 2. Reference audit

### 2.1 Method

The old PDF was inspected with `pdfinfo`, `pdftotext`, and `pdftoppm` page
renders at 100–200 DPI. The model cannot ingest a PDF directly; page images and
extracted text were read instead.

The current PDF was inspected the same way. Both are A4, 595.276 × 841.89 pt.

### 2.2 Page map

| Old page | Current page | Section | Status |
|---:|---:|---|---|
| 1 | 1 | Cover | Different treatment |
| 2 | 3 | “Bu kitapçık hakkında” | Order reversed |
| 3 | 2 | “Kılavuzun haritası” TOC | Order/style different |
| 4–10 | 4–10 | Bölüm 0 | Flow differs |
| 11–17 | 11–17 | Bölüm 1 | Flow differs |
| 18 | 18–19 | C — Kavramlar ve tanımlar | **C split into 2 pages** |
| 19 | 20 | D — Yöntemler ve süreçler | Table styling differs |
| 20 | 21 | E — Formüller, kurallar ve olgular | Table styling differs |
| 21 | 22 | F — Bağlantılar ve karşılaştırmalar | Table styling differs |
| 22 | 23 | G — Kapsam denetimi | Same content, later page |

The extra page is **C**, not G. C lost its two-column glossary and its
“Sık Karıştırılan Ayrımlar” table was pushed to a second page.

### 2.3 What the old PDF does

The old stylesheet is the authority for values:

| Token | Old value |
|---|---|
| Page margins | `16mm 16.5mm 14mm 16.5mm` |
| Body | Noto Serif 9.5 pt, line-height 1.33, justified |
| Chapter title | Inter 21 pt, semibold, `-0.02em` |
| Chapter kicker | Inter 8 pt, uppercase, `0.22em`, brick |
| H2 | Inter 12 pt, semibold, linen bottom rule |
| H3 | Inter 8.2 pt, uppercase, `0.12em`, sage |
| Definition box | cream `#F4ECDE`, 2 pt brick left rule |
| Critical box | `#F1EEE4`, 2 pt sage left rule |
| Example box | linen top/bottom rule, 9.5 pt |
| Note box | gold left rule, 8.9 pt muted |
| Table | 7.9 pt; header 6.9 pt uppercase; top/bottom dark rules; row rule linen; zebra `#F7F1E5` |
| Figure caption | 7.9 pt italic; figure number 6.8 pt uppercase brick |
| Code | DejaVu Sans Mono 7.4 pt; sand background; gold left rule |
| Source ref | Inter 6.9 pt, `[PD0, s. 3]`, nowrap |
| Two-column glossary | 2 columns, 7 mm gutter, 0.35 pt rule |
| Footer | page number centered, 8 pt, brick |

The old cover has:

- warm `#F3EBDD` background;
- double frame: 0.8 pt gold outer, 0.4 pt linen inner;
- 33 pt serif title, 17 pt italic subtitle;
- course line in small uppercase;
- pilot badge “PİLOT SÜRÜM · PD0–PD1”;
- syntax-tree motif near the bottom;
- source/deck note at the bottom.

The old chapter opener has:

- left part label (`B · AYRINTILI İNCELEME`);
- right source label (`KAYNAK: PD0 · 123 SLAYTI`);
- letter/number kicker (`BÖLÜM 0`, `C`, `D`, …);
- 21 pt title, 10.8 pt lead, source range line;
- a 1.4 pt brick rule above.

The old TOC has:

- kicker `İÇİNDEKİLER`;
- title `Kılavuzun haritası`;
- note with pilot scope and page-number convention;
- part rows `A · ÇERÇEVE`, `B · AYRINTILI İNCELEME`, `C–G · ORTAK BÖLÜMLER`;
- chapter entries and B subentries, not C–G subsections;
- right-aligned page numbers with thin row rules, no dotted leaders.

### 2.4 What the current PDF does differently

| Area | Current | Required |
|---|---|---|
| Cover | sans title, no motif, no pilot badge | restore old cover |
| Front/TOC order | TOC then front matter | front matter then TOC |
| TOC | generic dots, no part rows, includes G subsections | custom part-aware TOC |
| Chapter openers | brick H1 only; no meta/kicker/lead/source | full old opener |
| Running header | “Folisift” left + guide title right | chapter left + guide title right |
| Footer | right-aligned gray page number | centered brick page number |
| Body | 10.2 pt, 22/20 mm margins | 9.5 pt, old margins |
| C | single column, table spills to page 19 | 2 columns + table on one page |
| Tables | `stroke: none`; no header rules, cramped widths | old table rules/header/zebra/widths |
| Figures | captions lowercase/italic; source numbers kept | old caption + uppercase figure number |
| Boxes | sentence-case labels | uppercase letter-spaced labels with colors |
| Page count | 23 | 22 |
| Content | text/directives/figures/tables count matches | keep, restore metadata/layout |

## 3. Findings and bugs

### 3.1 Rendering findings

1. **Table stroke is disabled.** The renderer emits `stroke: none` for every
   table, overriding the template’s row rule. Remove this and let the template
   own table rules.
2. **No table header treatment.** `table.header` is emitted but the header is
   not uppercase, small, tracked or filled. Add a template show rule and keep
   the semantic header.
3. **No table width metadata.** Markdown tables cannot express the old
   `width: 5% / 20% / 35% / 40%` columns. D, E and F tables become cramped.
   Add an optional table metadata directive.
4. **C is one column.** The glossary content is a normal Markdown flow. Add a
   two-column region and a column-break marker.
5. **Chapter metadata was dropped by the HTML→Markdown converter.** The old
   `div.meta` and `.chapter-open` kicker/lead/source are absent from
   `guide.md`. Add a `::: chapter` directive and port the data.
6. **Front/TOC order is hard-coded.** The template emits cover → TOC → body.
   Old order is cover → front matter → TOC → chapters. Add a `::: toc` marker
   and let content place it.
7. **Running header is static.** It cannot know the current chapter. Use
   Typst `context` + `query(heading.where(level: 1).before(here()))`.
8. **Footer alignment/color.** Current footer is right/gray; old is
   center/brick.
9. **Cover has no motif/pilot badge.** Add `templates/assets/cover-motif.svg`
   from the old bundle and a pilot/badge field in frontmatter.
10. **Page count is 23.** Fixing C’s two-column layout and tightening the
    body/margins should recover 22 pages; verify, do not shrink type blindly.
11. **G is one page, but F/C tables are unstyled.** G is not the extra page;
    C is.
12. **Content text parity is already strong.** HTML→Markdown word overlap was
    98.6%; exact section/directive/figure/table counts. The remaining problem
    is structure and metadata, not missing prose.

### 3.2 Code/UX bugs found during the audit

Fixed on `main` during v0.2:

- `read_source` default `1-20` failed on documents shorter than 20 pages.
  Fixed: default clamps to page count; explicit out-of-range still errors.
- `guide_status` treated G.1 (table-only, 304 chars) as empty. Fixed: empty
  section detection only applies to numeric source sections.
- `SourceCache` key did not include the liteparse version. Fixed: key includes
  crate + liteparse version.
- `render_pages` rendered every page when `pages` was omitted. Fixed: 8-page
  cap, `next_pages`, contact sheet, DPI cap.
- Coverage was heading-only and scaffold could pass. Fixed: body char gate,
  empty/weak sections, scaffold now fails.
- MCP `build_guide` defaulted to no check. Fixed: MCP default `check=true`.
- `workspace_info` lacked PDFium/sidecar/cache fields. Fixed.
- `guide_status`/`list_guides` lacked content metrics. Fixed.
- Tool annotations and MCP stdio test were absent. Added.

Still open:

- `read_source` `chars` accounting uses bytes in `used`, while the field is
  character-oriented. Normalize to characters and test Turkish text.
- `search_source` returns the full low-text page list on every hit; cap it to
  count + first N pages.
- `workspace_info.sidecar_path` is `null` in a dev build that uses the
  compile-time PDFium cache. Release builds are fine; make dev reporting
  explicit.
- `PDFIUM_VERSION` is a hard-coded `chromium/8058`; generate it from the same
  build input as the sidecar or assert it at startup.
- `guide_status.section_count` includes G sections; source sections and all
  sections should be separate fields.
- `list_guides.content_words` counts Markdown syntax, not prose.
- No visual golden test, no OpenCode real-model smoke, no macOS/Windows/musl
  build, no signing.
- `source.txt` cache is correct but not yet tested against a full 260-page
  cache invalidation cycle.

### 3.3 Research: what other document MCPs and PDF tools do

The v0.2 plan already covers MCP/PDF context findings. The relevant constraints
for visual parity are:

- Typst `query` supports custom TOCs and running headers from headings:
  <https://typst.app/docs/reference/introspection/query/>
- Typst `outline` can be styled but is heading-only; part rows require a custom
  query/TOC:
  <https://typst.app/docs/reference/model/outline/>
- Typst `columns` does not balance columns automatically; explicit
  `colbreak`/page-level columns are needed:
  <https://typst.app/docs/reference/layout/columns/>
- Typst table styling uses `table.header`, `table.cell`, `where(y: 0)` and
  `show table` rules; column widths are explicit:
  <https://typst.app/docs/reference/model/table/>
- PDF MCP/community lessons: bounded page reads, explicit truncation,
  page-linked evidence and good tool responses matter more than raw text dumps:
  <https://pdfvision.dev/guide/mcp-server>,
  <https://github.com/jztan/pdf-mcp>,
  <https://www.reddit.com/r/mcp/comments/1v937uv/what_i_learned_building_a_pdf_readwrite_mcp/>
- OpenCode can read PDFs and write files natively; the MCP should not duplicate
  host file tools:
  <https://opencode.ai/v2/docs/tools>

## 4. Target design specification

### 4.1 Page geometry and type

- A4, margins `16mm 16.5mm 14mm 16.5mm`.
- Body Noto Serif 9.5 pt / 1.33 line-height, justified, hyphenated.
- Chapter title Inter 21 pt semibold, `-0.02em`; kicker 8 pt uppercase.
- H2 Inter 12 pt semibold with linen bottom rule.
- H3 Inter 8.2 pt uppercase, `0.12em`, sage.
- Source refs Inter 6.9 pt, muted, brackets, nowrap.
- Footer page number Inter 8 pt brick, centered.
- Header Inter 7 pt uppercase left chapter, right guide title, 2.5 mm above.

### 4.2 Cover

- Warm `#F3EBDD`, double frame.
- Course line small uppercase, gold rule.
- 33 pt serif title, 17 pt italic subtitle, chapter line.
- Pilot badge only when `pilot: true` in frontmatter.
- Motif from `templates/assets/cover-motif.svg`; width 95 mm, centered.
- Bottom source/deck note and method note.

### 4.3 Front matter and TOC

- Front page: kicker `ÖN KISIM · OKUMA NOTU`, 19 pt title, 11.2 pt lead,
  two-column about-grid with uppercase left labels and rules.
- TOC page: kicker, `Kılavuzun haritası`, note, part rows, chapter entries,
  B subentries. No C–G subsections. Right-aligned page numbers, thin row rules,
  no dotted leaders.
- Explicit `::: toc` marker after front matter.

### 4.4 Chapter opener

- 1.4 pt brick top rule.
- Meta: left part, right source; 7 pt uppercase.
- Kicker (number/letter) 8 pt uppercase brick.
- 21 pt title; 10.8 pt lead; 7.4 pt source range.
- Page break before every chapter; keep opener with following content.

### 4.5 Boxes, tables, figures, code

- Boxes: labels uppercase, 6.9 pt, tracked, per-type colors; body 9.5/8.9.
- Tables: 7.9 pt; header uppercase 6.9 pt, fill `#EFE7D8`, 1 pt top/bottom,
  0.35 pt row rules, zebra `#F7F1E5`; no vertical rules; explicit widths where
  the old table had them.
- Figures: full width, or 78% for narrow; caption 7.9 pt italic with 0.35 pt
  top rule; figure number 6.8 pt uppercase brick.
- Code: DejaVu/Noto Mono 7.4 pt, sand background, gold left rule; no clipping.
- Source refs: `[PD0, s. 3]`, no wrap, 6.9 pt.

### 4.6 Two-column glossary

- C page: kicker/meta/title/lead; then a two-column region with 7 mm gutter and
  a 0.35 pt column rule.
- Terms bold; H3 uppercase sage; source refs smaller.
- Explicit `::: columnbreak` before the second half so the old balance is
  reproducible.
- The “Sık Karıştırılan Ayrımlar” table follows in the same page.

## 5. Content model changes

Add generic directives, not course-specific hacks:

```markdown
::: front
kicker: Ön kısım · Okuma notu
title: Bu kitapçık hakkında
:::

::: toc
:::

::: chapter
part: B · Ayrıntılı inceleme
meta-right: Kaynak: PD0 · 123 slayt
kicker: Bölüm 0
title: Hazırlık
lead: ...
source: PD0, s. 1–123 · Alt bölümler: 0.1–0.5
:::

::: columns
:::

::: columnbreak
:::

::: table
widths: 5% 20% 35% 40%
header: true
:::
```

Rules:

- Directives are parsed by `folisift-render`, not by Typst.
- Unknown directive/attribute → line-numbered error.
- `::: chapter` emits a heading with `outlined: true` so TOC/bookmarks work.
- `::: table` wraps one Markdown table; widths/alignment/header metadata are
  renderer inputs, not raw Typst.
- `::: toc` emits the custom TOC at that point.
- `::: columns` / `::: columnbreak` are generic layout, not “C page” special
  cases.
- Old content without these directives still renders in the fallback style.

Port the old chapter metadata from `guide.html` (available in git history) into
these directives:

| Chapter | Part | Meta right | Kicker |
|---|---|---|---|
| 0 | B · Ayrıntılı inceleme | Kaynak: PD0 · 123 slayt | Bölüm 0 |
| 1 | B · Ayrıntılı inceleme | Kaynak: PD1 · 137 slayt | Bölüm 1 |
| C | C · Ortak bölümler | Sözlük · PD0–PD1 | C |
| D | D · Ortak bölümler | Süreç kartları | D |
| E | E · Ortak bölümler | Sınav bilgileri | E |
| F | F · Ortak bölümler | Bölümler arası ağ | F |
| G | G · Ortak bölümler | Pilot denetimi | G |

## 6. Renderer and template architecture

Use codebase-design terms:

- `folisift-render` is a **module** with a small interface:
  `render(markdown, options) -> {pdf, pages, warnings}`.
- The Markdown parser is an implementation detail; the MCP handler is a thin
  **adapter** and never emits Typst.
- `templates/paper.typ` is the shared design system; guides cannot inject Typst.
- The TOC, running header and chapter metadata are Typst introspection
  queries, not string templates assembled by callers.
- No per-course branches in the template. If C needs columns, content says
  `::: columns`; the template does not know “C”.

Implementation pieces:

1. **Parser**
   - frontmatter (title, subtitle, course, pilot, sources, language);
   - block directives;
   - inline source refs;
   - table metadata;
   - headings, lists, quotes, code, images.
2. **TOC**
   - `query(heading.where(level: 1, outlined: true))` for chapters;
   - part metadata from `::: chapter` / frontmatter;
   - B subentries from level 2 headings;
   - page numbers via `counter(page).display(at: loc)`;
   - skip C–G subsections.
3. **Running header**
   - `context` + `query(selector(heading).before(here()))` (verify exact
     selector in implementation);
   - fall back to guide title on front matter.
4. **Tables**
   - renderer emits `table.header` and cell content;
   - template sets rules/header/zebra;
   - optional widths/alignment from `::: table`.
5. **Columns**
   - `#columns(2, gutter: 7mm)[...]`;
   - `#colbreak()` for explicit break;
   - page-level behavior tested on C.
6. **Cover**
   - `image("assets/cover-motif.svg")`;
   - badge conditional;
   - metadata from frontmatter.
7. **Page flow**
   - cover no header/footer;
   - front page no header/footer;
   - TOC no header/footer;
   - chapters pagebreak;
   - C/D/E/F/G each one page in the pilot;
   - G one page.
8. **Fonts**
   - Noto Serif, Inter, Noto Sans Mono (or DejaVu with license);
   - verify Turkish glyphs and hyphenation.

## 7. Reference assets and SVGs

- Copy `engine/assets/cover-motif.svg` from git history into
  `templates/assets/cover-motif.svg`; keep it shared.
- Audit all `guides/pd0-pd1/assets/*.svg`:
  - no external fonts or raster dependencies;
  - `viewBox` and intrinsic size set;
  - text converted to outlines or using a shared font;
  - labels readable at 78%/100% width;
  - no clipping in Typst.
- The four reference figures:
  - `pipeline.svg` (compiler pipeline);
  - `parse-tree.svg` (expression tree);
  - `von-neumann.svg` (architecture);
  - `sdlc.svg` (SDLC loop).
- Add a golden screenshot for each at the same page width.
- If a figure cannot be made faithful, redraw it in Typst/SVG; do not accept a
  raster screenshot unless it is the only faithful source.

## 8. Content port corrections

1. Re-port the old chapter meta/kicker/lead/source into `::: chapter`.
2. Insert `::: toc` after front matter; remove auto TOC from the template.
3. Wrap the C glossary in `::: columns` / `::: columnbreak`; restore the
   “Sık Karıştırılan Ayrımlar” table on the same page.
4. Add table width metadata for D, E, F and G tables.
5. Restore box labels (`TANIM`, `KRİTİK AYRIM`, `ÖRNEK`, `KAYNAK NOTU`) as
   directive labels, not bold prose.
6. Set figure numbers/captions to the old style; keep source refs.
7. Restore “Kılavuzun haritası” TOC title/note and part rows.
8. Verify G remains one page; do not split it.
9. Rebuild and compare page count; target 22, never shrink body type below 9.3
   pt just to hit the count.

## 9. MCP and LLM flow

The host LLM remains the writer. The visual work must not turn the server into
an LLM or a content generator.

- `read_source` gives bounded text chunks; `search_source` finds terms.
- `scaffold_guide` creates chapter directives and section headings.
- `build_guide(check=true)` renders and enforces content-aware coverage.
- `render_pages` returns page/contact-sheet paths for visual checks.
- `write-study-guide` prompt tells the host to write `guide.md` with its native
  `write`/`edit` tool, then build and audit.
- Add a **visual QA prompt**: after build, inspect cover, TOC, chapter opener,
  C glossary, D/E/F tables and G; report differences.

## 10. Installation and doctor

Add a first-class `folisift doctor`:

```text
workspace root
config schema
sidecar present/loadable
PDFium version
Ingen fonts loaded
Typst compile smoke
coverage file writable
MCP stdio handshake
```

Add `folisift mcp-config` that prints the OpenCode V2 snippet with absolute
paths. Add `scripts/install.sh` that verifies SHA-256 and keeps the sidecar
next to the binary. Do not use `curl | sh` until signing exists.

Release archives:

```text
folisift
libpdfium.so
INSTALL.txt
LICENSES/
```

## 11. Testing and parity gates

### 11.1 Golden reference

- Store the old PDF as a test fixture: `tests/golden/pd0-pd1-python.pdf`.
- Store page renders at 150 DPI for the pages that define the design:
  1 (cover), 2 (front), 3 (TOC), 4 (chapter 0), 11 (chapter 1), 18 (C),
  19 (D), 20 (E), 21 (F), 22 (G).
- Store text extraction for content parity.

### 11.2 Visual gate

- Render current pages at the same DPI.
- Compare each selected page with:
  - absolute pixel difference ratio;
  - SSIM or a small perceptual threshold;
  - text-region bounding boxes (title/lead/table/caption).
- Accept `<= 2%` pixel difference for text pages and `<= 5%` for pages with
  SVG anti-aliasing; review any outlier by eye.
- No acceptance if a table, figure, caption or page number is missing.

### 11.3 Content gate

- 13/13 source sections, 0 empty, 0 weak.
- 24/24 tables, 4/4 figures, 4/4 definition, 4/4 critical, 3/3 example,
  2/2 note.
- 22 pages (old reference), not 23.
- TOC entries and PDF bookmarks match the old map.
- Turkish glyphs and hyphenation.
- Source refs per subsection.

### 11.4 MCP gate

- stdio handshake, 10 tools, `read_source`/`search_source`, `build_guide`.
- OpenCode config smoke test with `mcp.servers`.
- The host writes a fixture section with `write` and builds.

### 11.5 Install gate

- Extract archive in a clean dir.
- Hide PDFium cache; sidecar must load.
- `folisift doctor` passes.
- `build --check` produces 22 pages and coverage pass.
- No Python/Poppler/WeasyPrint/system Typst.

## 12. Phases

### Phase 0 — Golden capture (0.5 day)

- Add old PDF fixture and selected page images.
- Freeze old page map and design tokens.
- Exit: golden images and text are reproducible.

### Phase 1 — Template parity (2–3 days)

- Cover, front, TOC, header/footer, chapter opener.
- Body margins/type, boxes, figures, code, source refs.
- Exit: first 4 pages and chapter opener side-by-side look right.

### Phase 2 — Content model directives (2 days)

- `front`, `toc`, `chapter`, `columns`, `columnbreak`, `table` metadata.
- Parser errors and tests.
- Exit: current guide can express the old structure without Typst hacks.

### Phase 3 — C/D/E/F/G corrections (1–2 days)

- Two-column C + table.
- Table widths for D/E/F/G.
- Chapter metadata, labels, captions.
- Exit: 22 pages; C on one page; G on one page.

### Phase 4 — Visual/content gates (1–2 days)

- Golden comparison harness.
- Page/table/figure/bookmark tests.
- Exit: gates pass; only explained anti-aliasing differences remain.

### Phase 5 — Install/doctor/CI (1–2 days)

- `doctor`, `mcp-config`, install script.
- CI matrix Linux; macOS/Windows next.
- Exit: clean install and MCP smoke pass.

### Phase 6 — Release v0.3.0 (1 day)

- Archive, sidecar, checksum, tag, release notes.
- Exit: published release with parity report.

## 13. Acceptance criteria

- [ ] 22 pages, same section order as old PDF.
- [ ] Cover visually matches: frame, title/subtitle, pilot badge, motif, notes.
- [ ] Front matter before TOC; old about-grid and kicker.
- [ ] TOC has part rows and correct page numbers; no C–G subsections.
- [ ] Chapter openers have meta/kicker/lead/source.
- [ ] Running header shows current chapter; footer centered brick.
- [ ] C is two columns on one page, including the comparison table.
- [ ] D/E/F/G tables have old rules, header style, zebra and widths.
- [ ] Four SVG figures render at old size; captions/numbers match.
- [ ] Boxes have uppercase letter-spaced labels and correct colors.
- [ ] Source refs match `[PD0, s. 3]` style.
- [ ] 24/24 tables, 4/4 figures, 13/13 coverage, 0 empty sections.
- [ ] Visual diff under threshold for selected pages.
- [ ] `folisift doctor` and clean-install test pass.
- [ ] MCP stdio and OpenCode config smoke pass.
- [ ] Release archive checksum verifies.

## 14. Risks

| Risk | Mitigation |
|---|---|
| Typst cannot reproduce CSS column balancing exactly | Use explicit `columnbreak`; accept ±5% visual diff on C. |
| Table widths vary with content | Add per-table metadata; test D/E/F/G. |
| TOC query self-reference | Use `outlined: true`, stable heading labels; test compile. |
| Running header query instability | Use a simple `before(here())` chapter query; test on all pages. |
| Font metrics differ | Compare at 150 DPI; adjust size/margins; never below 9.3 pt. |
| 22-page target forces cramped text | Prefer old 9.5 pt and old margins; accept 23 only with explicit approval. |
| Cover motif licensing | It is an original project asset; keep in `templates/assets/`. |
| Visual tests brittle | Use selected semantic pages + threshold, not full-document pixel equality. |
| Golden PDF copyright | Private repo; local fixture only, not a public release asset. |

## 15. Open questions

1. Should the old PDF be a committed test fixture or a downloaded local artifact?
2. Is a 22-page exact match mandatory, or is 23 acceptable if C is one page and
   type is 9.5 pt?
3. Do we keep DejaVu Mono or switch to Noto Sans Mono for the old look?
4. Should table width metadata use `::: table` or an HTML comment?
5. Do we need a “pilot: true” frontmatter flag or a generic badge field?
6. Is a 2% pixel threshold realistic with different font rasterizers?
7. Do we add macOS/Windows builds in this release or a follow-up?
8. Do we keep the old PDF in `output/` as a reference or move it under
   `tests/golden/`?

## 16. Definition of done

The plan is done when a clean install can:

```bash
folisift doctor
folisift build --guide pd0-pd1 --check
```

and produce a **22-page** guide whose selected pages pass the visual gate,
whose content passes coverage and table/figure counts, whose MCP flow lets the
host LLM read chunks and write `guide.md`, and whose archive verifies with a
checksum. Until then, the system is not “perfect”; it is a working v0.2 with a
documented visual gap.
