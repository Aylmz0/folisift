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

### 2.3 PD3 summary (`output/PD3_Calisma_Ozeti.pdf`, 21 pages after the code retune)

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

## 5. Palettes: a semantic library, not three skins

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

### 5.1 Palette library

All core text roles below pass 4.5:1 on `paper`; the “worst” column is the
lowest of ink/muted/faint/accent/secondary on paper. Full files also define
`cream`, `zebra`, `headerfill`, `code_bg` and `cover_bg`, and each is checked
against `ink`.

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

This is intentionally a **library**, not a free-for-all:

- four warm/editorial options for long reading and revision;
- four cool/academic options for technical and scientific material;
- one natural option for humanities/business;
- two neutral/print options including a grayscale proof;
- one high-contrast accessibility option.

### 5.2 Palette rules

- Palette is independent of genre. A revision sheet can be warm, cool or
  grayscale; a reader can be warm or academic navy.
- A palette must pass every text/background pair, not only `ink/paper`.
- Test grayscale/black-and-white print; if two categories are distinguishable
  only by hue, add a label, pattern, icon or rule.
- Do not use red/green as the only distinction. Colour-blind prevalence is
  about 1 in 12 men and 1 in 200 women of Northern European descent; labels
  and shapes carry the meaning.
- Colour psychology is context-dependent; do not claim “blue improves focus”.
  Use colour for structure and signalling, not as a learning intervention
  (Elliot & Maier, 2014).
- `gold` is a rule/accent colour; it is not used for small text unless a
  darker text variant passes 4.5:1.

### 5.3 What “proven style” means

There is no RCT showing that “warm paper” or “cobalt” raises exam scores.
“Proven” applies to **learning techniques** (retrieval, spacing, worked
examples, concept maps, graphic organizers) and to **accessibility standards**
(contrast, type size, tagged PDF). Palette and font choices are professional
design choices constrained by those standards, not learning-outcome claims.

## 6. Fonts: pairings, not random families

Current trio is coherent:

- **Noto Serif** — long-form body; generous print texture.
- **Inter** — headings, revision body, UI labels; large x-height.
- **Noto Sans Mono** — code; x-height matches Noto Serif.

### 6.1 Pairing library

| Pairing | Body | Heading | Mono | Status | Use |
|---|---|---|---|---|---|
| `editorial` | Noto Serif | Inter | Noto Sans Mono | embedded, verified | paper, reader, journal |
| `technical` | Inter | Inter | Noto Sans Mono | embedded, verified | clean, revision, brief |
| `academic` | Source Serif 4 | Source Sans 3 | Source Code Pro | candidate, OFL; Turkish glyph test required | academic reader/report |
| `accessible` | Atkinson Hyperlegible Next | Atkinson Hyperlegible Next | Noto Sans Mono | candidate, OFL; Latin Extended/Turkish glyph test required | large-print |
| `plex` | IBM Plex Serif | IBM Plex Sans | IBM Plex Mono | candidate, OFL; extended Latin/Turkish glyph test required | technical/academic |

Do **not** add all candidates at once. Keep the embedded trio as default for
v0.10.1; add one candidate pairing only after a glyph/licence/package test.

### 6.2 Pairing rules

- Maximum three families: body, heading, mono.
- A new pairing replaces a family, not adds a fourth.
- OFL/compatible licence; embed and subset in the PDF.
- Full Turkish glyph coverage: `ı İ ğ ş ç ö ü` in regular/bold/italic.
- x-height within ~10% of the body face; otherwise adjust body size visually.
- No near-duplicate sans as a “second heading” font.
- Test actual PDF output at 8–12 pt, not just specimen images.

### 6.3 Why not “one font for everything”

A serif body improves long-form reading texture; Inter gives structural
contrast and a large x-height for revision; Noto Sans Mono makes code literal
and aligned. Using one family for all roles collapses hierarchy and makes
code/body distinction weaker. Using four or more families adds package weight
and visual noise without a reading task.

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
- Elliot, A. J., & Maier, M. A. (2014). Color psychology: Effects of perceiving
  color on psychological functioning in humans. *Annual Review of Psychology,
  65*, 95–120. <https://doi.org/10.1146/annurev-psych-010213-115035>
- Colour Blind Awareness. *About colour blindness*.
  <https://www.colourblindawareness.org/colour-blindness/>
- Adobe. *Source Serif 4 / Source Sans 3 / Source Code Pro* (SIL OFL).
  <https://github.com/adobe-fonts>
- IBM. *IBM Plex* (SIL OFL). <https://github.com/ibm/plex>
- Braille Institute. *Atkinson Hyperlegible* (SIL OFL).
  <https://www.brailleinstitute.org/freefont/>
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
