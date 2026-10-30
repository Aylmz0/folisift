# Typography standardization: evidence, measurements and a type system

- **Status:** Research note for the v0.10.1 typography plan
- **Date:** 2026-10-31
- **Scope:** body, table, code and caption scale across every Folisift layout;
  multiple palettes and font pairings without breaking the genre system
- **Related:** [`document-genres-and-layout-systems.md`](document-genres-and-layout-systems.md),
  [`revision-format-research.md`](revision-format-research.md),
  [`authoring-evidence.md`](authoring-evidence.md)
- **Claims policy:** typography standards and reading research are not claims
  that a particular PDF improves exam scores.

## 1. The user-visible problem

Two concrete complaints are correct:

1. **The PD2 revision sheet body is too large.** It uses 11 pt Inter with
   32 mm side margins. Inter has a large x-height, so 11 pt Inter looks larger
   than 11 pt Noto Serif.
2. **The PD2 summary code is too small relative to the body.** Code blocks are
   7.4 pt against a 10 pt body (0.74 ratio); revision code is 8.9 pt against
   11 pt (0.81). Both are below the 0.90–0.95 range recommended for code that
   must be read, not merely scanned.

The root cause is not one bad number. It is a **typographic system with no
single scale**: paper/clean hard-code sizes, revision uses placeholders, and
`accessible` uses a `component_scale` multiplier. The same role can be 6.9,
8.0, 8.4, 9.0 or 9.5 pt depending on the layout.

## 2. Measurements from the current PDFs

Method: PyMuPDF text spans; character counts per font size; body-size lines
only for line length. These are observations, not universal targets.

### 2.1 PD2 revision pilot (`output/PD2_Revizyon.pdf`, 4 pages)

| Role | Size | Ratio to body |
|---|---:|---:|
| Body | 11.0 pt Inter | 1.00 |
| Table / caption | 9.5 pt | 0.86 |
| Code block | 8.9 pt Noto Sans Mono | **0.81** |
| Label | 8.4 pt | 0.76 |
| Source ref | 8.4 pt | 0.76 |
| TOC page | 9.0 pt | 0.82 |

Line length: median 51 characters, p90 79. The body feels large because
Inter's x-height is 0.546 em versus 0.536 em for Noto Serif: 11 pt Inter has an
x-height of about 6.01 pt, while 10 pt Noto Serif is about 5.36 pt (12% larger
visual body).

### 2.2 PD2 summary (`output/PD2_Calisma_Ozeti.pdf`, 23 pages)

| Role | Size | Ratio to body |
|---|---:|---:|
| Body | 10.0 pt Noto Serif | 1.00 |
| Table | 7.9 pt | 0.79 |
| Code block | 7.4 pt Noto Sans Mono | **0.74** |
| Inline code | 8.4 pt | 0.84 |
| Caption | 7.9 pt | 0.79 |
| Label | 6.9 pt | 0.69 |
| Source ref | 6.9 pt | 0.69 |
| Note | 8.9 pt | 0.89 |

Line length: body-size lines median 95, p90 106. The summary is a long-form
reader; its measure is already too long, but the code-size problem is separate:
code is visibly smaller than the prose it explains.

### 2.3 PD3 summary (`output/PD3_Calisma_Ozeti.pdf`, 20 pages)

PD3 is code-heavy: 5,114 characters at 7.4 pt Noto Sans Mono and 2,334
characters at 8.4 pt inline code against a 10 pt body. The code is the content;
at 0.74 body ratio it is the least readable element on the page.

### 2.4 Font metrics

| Face | Units/em | x-height | x/em |
|---|---:|---:|---:|
| Inter Regular | 2048 | 1118 | 0.546 |
| Noto Serif Regular | 1000 | 536 | 0.536 |
| Noto Sans Mono Regular | 1000 | 536 | 0.536 |

Inter is not a “bigger font” in em terms; it has a larger lowercase body. A
10.5 pt Inter body is visually closer to a 10.7 pt Noto Serif body than 11 pt
Inter is to 10 pt Noto Serif. That is why 11 pt Inter feels oversized in a
revision sheet.

## 3. What the standards say

| Claim | Status | Source | Practical rule |
|---|---|---|---|
| 55–80 characters per line | moderate/standard | Dyson & Haselgrove (2001); Bringhurst; WCAG 2.2 AAA 1.4.8 | Continuous prose ≤ 80; cards/tables can be shorter |
| Normal text contrast ≥ 4.5:1 | standard | WCAG 2.2 AA 1.4.3 | Check every text role, including source refs and code |
| Running text below 9 pt is a barrier | standard | Section 508 | No body/table/code below 9 pt; 8 pt only for non-essential metadata |
| Clear print starts at 12 pt | standard | UKAAF clear print | Large-print body 12 pt; components scale too |
| Line spacing 1.4–1.6 for body; 1.4–1.6 for code | heuristic/standard | Typographic practice; WCAG 1.4.8 AAA | Do not use 0.5 em leading in code |
| Max 2–3 font families | professional heuristic | Typographic practice | Serif/sans + mono is enough; avoid near-duplicate sans faces |
| Match x-height when pairing | professional heuristic | Typographic practice | Inter + Noto Serif work; check visual size, not only point size |
| Code should be 0.90–0.95 of body | heuristic | Professional code typography; Section 508 floor | Code is read character-by-character; do not shrink it to fit |
| Tables 0.85–0.90 of body, floor 8.5 | heuristic | Information design practice | Table text is read in cells; below 8.5 becomes cramped |
| Labels 0.75–0.80 of body, floor 8 | heuristic | Information design practice | Uppercase labels can be smaller, not tiny |
| Colour is never the only signal | standard | WCAG 1.4.1 | Pair colour with label, rule or shape |

Two honesty notes:

- **WCAG is a web standard.** For fixed-layout PDF, PDF/UA is the structural
  standard; WCAG remains a useful readability/contrast benchmark.
- **Print point sizes are not CSS pixels.** A 10 pt print body is not “too
  small” in the way 10 px web text is. The relevant test is x-height, measure,
  contrast and reading distance.

## 4. Proposed Typography Standard v1

### 4.1 One scale, every layout

The body size is the only genre-specific base. Every other role is a ratio
with a floor.

| Role | Ratio | Floor | Notes |
|---|---:|---:|---|
| Body | 1.00 | 9.0 | genre/density chooses the base |
| H1 | 2.00 | 18.0 | chapter/section band |
| H2 | 1.40 | 14.0 | section heading |
| H3 | 1.10 | 11.0 | subheading/label |
| Lead | 1.12 | 11.0 | front/section lead |
| Table | 0.86 | 8.5 | header and body same size |
| Caption | 0.86 | 8.5 | italic; takeaway text |
| Label | 0.78 | 8.0 | uppercase; tracking 0.08–0.12 em |
| Source ref | 0.78 | 8.0 | `faint` must pass 4.5:1 |
| Code block | 0.92 | 9.0 | monospace; line height 1.45–1.5 |
| Inline code | 0.95 | 9.0 | slightly larger than block code |
| TOC title | 0.95 | 9.0 | |
| TOC sub | 0.88 | 8.5 | |
| TOC page | 0.82 | 8.0 | |

At a 10.0 pt reader body: table 8.6, code 9.2, inline 9.5, labels 8.0.
At a 10.5 pt revision body: table 9.0, code 9.7, inline 10.0, labels 8.2.
At a 12.0 pt accessible body: table 10.3, code 11.0, inline 11.4, labels 9.4.

### 4.2 Body sizes by layout

| Layout/genre | Body | Why |
|---|---:|---|
| `paper` legacy | 9.5 Noto Serif | v0.9 parity fixture; frozen |
| `reader` | 10.0 Noto Serif | new default long-form reading |
| `clean` | 9.5 Inter | technical reports; dense |
| `journal` | 10.5 Noto Serif | academic reader |
| `brief` | 10.5 Inter | decision memo |
| `revision` | **10.5 Inter** | smaller than today's 11; still readable at 32 mm |
| `accessible` | 12.0 Noto Serif | large print; all components scale |

Revision body recommendation: **10.5 pt**, not 11. If the preview still feels
large, the next candidate is 10.0 with 32–34 mm margins; do not shrink code or
tables to compensate.

### 4.3 Revision specific values (proposed)

| Role | Today | Proposed |
|---|---:|---:|
| Body | 11.0 | **10.5** |
| H1 | 22.0 | 21.0 |
| H2 | 15.4 | 14.7 |
| H3 | 12.1 | 11.6 |
| Lead | 12.3 | 11.8 |
| Table/caption | 9.5 | 9.0 |
| Label/source | 8.4 | 8.2 |
| Code block | 8.9 | **9.7** |
| Inline code | 8.9 | 10.0 |
| TOC | 10.5/9.7/9.0 | 10.0/9.2/8.6 |

This directly fixes both complaints: body down, code up.

### 4.4 Reader / PD2 specific values (proposed)

| Role | Today | Proposed |
|---|---:|---:|
| Body | 10.0 | 10.0 (unchanged) |
| Table | 7.9 | **8.6** |
| Caption | 7.9 | 8.6 |
| Code block | 7.4 | **9.2** |
| Inline code | 8.4 | **9.5** |
| Label | 6.9 | **8.0** |
| Source ref | 6.9 | **8.0** |
| Note | 8.9 | 9.0 |

The reader body stays 10 pt because the complaint is code size, not prose
size. If the reader measure is later shortened, body size can be revisited.

## 5. Palettes: separate semantic layer

A palette is a set of semantic roles, not a style name. Proposed roles:

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

Candidate palettes:

| Palette | Paper | Ink | Accent | Use |
|---|---|---|---|---|
| `warm` | `#FAF6EC` | `#211E19` | `#8A3E2F` | paper, reader, journal, revision |
| `cool` | `#FFFFFF` | `#111827` | `#1D4ED8` | clean, technical |
| `contrast` | `#FFFFFF` | `#111111` | `#8A1C1C` | accessible/high-contrast |

Rules:

- Validate contrast for every text role against every actual background where
  it is used, not only `ink/paper`.
- `faint` must pass 4.5:1; gold is decoration/rules only unless a darker text
  variant is defined.
- Palette choice is independent of genre and layout; a revision sheet can be
  warm or cool.

## 6. Fonts: pairings, not random families

Current trio is coherent:

- **Noto Serif** — long-form body; generous print texture.
- **Inter** — headings, revision body, UI labels; large x-height.
- **Noto Sans Mono** — code; x-height matches Noto Serif.

Recommended **font pairings**:

| Pairing | Body | Heading | Mono | Use |
|---|---|---|---|---|
| `editorial` | Noto Serif | Inter | Noto Sans Mono | paper, reader, journal |
| `technical` | Inter | Inter | Noto Sans Mono | clean, revision, brief |
| `accessible` | Noto Serif | Inter | Noto Sans Mono | accessible (12 pt) |

Do **not** add more families until the three above are correctly tokenized.
If a new pairing is added later, require:

- OFL/compatible licence,
- embedded in the PDF,
- x-height within ~10% of the body face,
- a full glyph/ Turkish diacritic check,
- package-size and licence review.

## 7. Architecture fix

### 7.1 Current problem

`styles.rs` still uses string replacement for body size and a
`component_scale` multiplier for accessible. This is why a single role can
have different sizes in different layouts.

### 7.2 Target

```text
style pack
  ├── layout: paper | clean | revision
  ├── palette: warm | cool | contrast
  ├── fonts: editorial | technical | accessible
  ├── scale: compact | comfortable | spacious
  └── accessibility: standard | large-print
```

`TypographyScale` computes every role once:

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

Templates use placeholders only:

```typst
#set text(size: __BODY_SIZE__)
#set raw(... size: __CODE_SIZE__)
#set table(... size: __TABLE_SIZE__)
```

No global `9.5pt` replacement. No `component_scale` exception. The accessible
layout is just body 12 with the same ratios and floors.

### 7.3 Validation

- `body_size` 9–18 pt.
- `table`, `caption`, `code_block`, `code_inline`, `label`, `source` ≥ 8.0;
  table/code ≥ 9.0 in revision/accessible.
- `body / smallest_component ≤ 1.45` (revision/accessible ≤ 1.35).
- `code_block / body` 0.90–0.95.
- `table / body` 0.84–0.90.
- Every text palette role ≥ 4.5:1 on its actual backgrounds.
- `audit` reports font histogram, code/body ratio and line-length p50/p90.

## 8. What not to do

- Do not fix “body too large” by shrinking margins or code. Fix body first.
- Do not make code smaller than 9 pt; code is read character-by-character.
- Do not add a fourth font family to look “designed”.
- Do not use a different scale per style; use one scale with a different base.
- Do not use `component_scale` as a permanent exception; migrate to the shared
  scale.
- Do not force paper parity if it keeps 6.9 pt labels and 7.4 pt code forever;
  keep a `paper-legacy` fixture and move new guides to the standard.

## 9. References

- Bringhurst, R. (2012). *The elements of typographic style* (4th ed.).
- Dyson, M. C., & Haselgrove, M. (2001). The influence of reading speed and
  line length. *International Journal of Human-Computer Studies, 54*(4),
  585–612. <https://doi.org/10.1006/ijhc.2001.0458>
- Inter font metrics: `OS/2.sxHeight` / `head.unitsPerEm` from the embedded
  `templates/fonts/Inter-Regular.ttf` (0.546).
- Noto Serif / Noto Sans Mono metrics: same method (0.536).
- Section508.gov. (2026). *Understanding accessible fonts and typography*.
  <https://www.section508.gov/develop/fonts-typography/>
- UK Association for Accessible Formats. (2024). *Creating clear print and
  large print documents* (G003, v4).
- W3C. (2024). *Web Content Accessibility Guidelines (WCAG) 2.2*.
  <https://www.w3.org/TR/WCAG22/>
- ISO 14289-1:2014. *PDF/UA-1*.
