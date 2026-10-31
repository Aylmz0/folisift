# Folisift v0.10 — Genre-first revision format and layout systems

- **Status:** v0.10 revision/organiser complete; v0.11 notes/workbook/reference genres complete; metadata migration remains a follow-on
- **Date:** 2026-10-30
- **Baseline:** `v0.9.0` (`f7f4e400`, build 101; PD0–PD1 22, PD2 23, PD3 20
  pages; 124 tests)
- **Trigger:** the user review of `paper` vs `reader`, the lack of true
  purpose-specific output genres, and the typography inconsistencies found in
  the production PDFs
- **Follow-on:** [`2026-10-31-typography-standardization-and-font-system.md`](2026-10-31-typography-standardization-and-font-system.md)
- **Research notes:**
  [`docs/revision-format-research.md`](../revision-format-research.md),
  [`docs/document-genres-and-layout-systems.md`](../document-genres-and-layout-systems.md)
- **Decision in one sentence:** stop treating typographic presets as genres;
  make **genre** the primary layer, give each genre its own structure, layout
  logic and gate, and keep density/accessibility as modifiers.

> **Genre correction.** The user's critique is correct: `paper` and `reader`
> are the same layout at different density; `journal`, `brief` and
> `accessible` are presets/modifiers, not different information architectures.
> This plan now starts from **purpose and reader task**, not from font size.
> See `docs/document-genres-and-layout-systems.md` for the full genre matrix.

## 1. What is actually wrong

### 1.1 Product and taxonomy

The current `aesthetic` field mixes four different things:

1. a **layout family** (`paper`, `clean`);
2. a **density variant** (`reader`);
3. an **accessibility variant** (`accessible`);
4. a **professional genre preset** (`journal`, `brief`).

That is why `paper` and `reader` look nearly identical: they are the same
layout, palette, fonts, cover and structure, with different body size, margins,
leading and spacing. `journal`, `brief` and `accessible` are token-only
variants too; they do not add the components their names promise.

The current packs are therefore not a style system. They are **presets with
genre-sounding names**. A student looking for a revision sheet has no genre to
select.

The deeper problem is not the names. **No pack changes the page logic.** Every
pack uses the same section flow, the same component grammar and the same visual
hierarchy. A revision sheet needs cards, comparison matrices, maps, worked
examples, traps and recall prompts; a reader needs continuous prose and
chapter-level argument; a workbook needs workspace and an answer key; a
knowledge organiser needs a one-page quadrant. Those are different information
architectures, not different font sizes.

### 1.2 Typography harmony

The production PDFs contain real, measurable violations:

| ID | Violation | Evidence | Effect | Severity |
|---|---|---|---|---|
| T1 | `body_size` is applied by replacing every `9.5pt` in the template | `styles.rs:315-320`; `paper.typ:252,295,438,525,556` | body size leaks into cover/quote/front matter; fragile and unpredictable | high |
| T2 | Component sizes are hard-coded | `paper.typ`: table 7.9, table header 6.9, caption 7.9, label 6.9, note 8.9, source 6.9, table caption 7.8, figure label 6.8, cover notes 6.6 | components do not scale with the body or the accessibility pack | high |
| T3 | `accessible` has 12 pt body but 6.9–7.9 pt tables/labels | generated `style-preview-accessible-tr.pdf` | “large print” is only true for prose; tables and labels remain small | high |
| T4 | Heading hierarchy collapses in `accessible` | `paper.typ:503` (h2 12 pt), `:516` (h3 8.2 pt), `:281` (TOC page 8.2 pt) | h2 is the same size as the 12 pt body; h3 and TOC are smaller than body | high |
| T5 | `line_height` tokens are ignored | `paper/tokens.json:16`, `clean`, `journal`, `brief`, `accessible`; `styles.rs:322-327` only reads `leading_em` | two names for one property; docs say 1.33 but template uses 0.62 em leading | medium |
| T6 | `faint` text fails contrast | `#8A7A63` on `#FAF6EC` = **3.86:1**; on cream `#F4ECDE` = **3.55:1**; used for source refs, TOC page numbers, cover notes | source traceability text is below WCAG AA 4.5:1 | high |
| T7 | Body is justified with hyphenation disabled | `paper.typ:443`: `justify: true`, `hyphenate: false` | rivers and loose lines, especially in Turkish and narrow table cells | medium |
| T8 | Line measure is too long | measured body-size lines in production PDFs: median **92–105** characters, p90 **105–112** | outside 45–75 (Bringhurst), 55 (Dyson & Haselgrove), and 80 (WCAG AAA) guidance | high |
| T9 | Table headers are 6.9 pt uppercase with 0.09em tracking | `paper.typ:114-120` | hard to read at print size; all-caps reduces word shape | medium |
| T10 | Near-duplicate type sizes | 6.6, 6.8, 6.9, 7.0, 7.2, 7.4, 7.8, 7.9, 8.2, 8.4, 8.9, 9.4, 9.5, 9.6, 10.8, 11.2, 12 | no modular scale; hierarchy is accidental | medium |
| T11 | Docs and template disagree | `design-system.md` says 9.5 pt / line-height 1.33; template uses 0.62em leading; accessible uses 0.80em | maintainers cannot tell which value is authoritative | medium |
| T12 | `note` body is 8.9 pt while definition body inherits 12 pt in `accessible` | `paper.typ:230` | recall/explain components are inconsistent | high |
| T13 | TOC tokens are missing for `journal`, `brief`, `accessible` | `styles.rs:349-363`; no `toc` in those token files | TOC stays at paper sizes while body grows | high |
| T14 | `facts_table` is 9.4/7.4 fixed | `paper.typ:149-166` | component does not scale | medium |
| T15 | `audit` accepts 6 pt | `audit.rs:317` | a 6.9 pt label passes even in the large-print pack | high |

Measured production evidence:

```text
PD0–PD1: body-size lines median 105 chars, p90 112; font sizes 9.5
         (35,698 chars), 7.9 (10,835), 8.2 (3,775), 6.9 (3,020)
PD2:     body-size lines median 95 chars, p90 106; 10.0 (38,421),
         7.9 (4,291), 6.9 (2,632), 8.9 (2,380)
PD3:     body-size lines median 92 chars, p90 105
```

Measurement method: PyMuPDF text lines; only lines whose largest span is the
body size (9.5 or 10.0 pt); 388–495 lines per guide.

### 1.3 Gate and documentation bugs

| ID | Bug | Where | Fix | Severity |
|---|---|---|---|---|
| G1 | `audit` uses hard-coded paper margins (46 pt sides, 52 pt top, 45 pt bottom) | `audit.rs:164-166` | read margins from the guide/style; add `audit --guide` | high |
| G2 | `audit` cannot see text that intrudes into the larger `accessible` margins | same | style-aware content box | high |
| G3 | `audit` treats visual pages as underfilled | `audit.rs:360+` | skip pages with figures/tables, or count visual area | medium |
| G4 | `figure_captions` counts “Şekil” anywhere in prose | `audit.rs` helper | count caption-like lines only | low |
| G5 | No line-length or font-size-distribution audit | `audit.rs` | add p50/p90/max and font histogram | medium |
| G6 | Contrast validation checks only ink/paper | `styles.rs:177-199` | validate every text role against its backgrounds | high |
| G7 | `docs/authoring-evidence.md` still lists APA/IEEE/Chicago/OSCOLA | section 5.3 | only `page-ref` is implemented | medium |
| G8 | `docs/authoring-evidence.md` lists `clinical`/`casebook` styles that do not exist | section 5.3 | mark as future or remove | medium |
| G9 | `docs/authoring-spec.md` says “No quiz material” | section 3 | make the rule genre-specific; revision sheets need retrieval | medium |
| G10 | `design-system.md` has the journal/brief/accessible paragraph before the “Clean style” heading | `design-system.md` | rewrite the style section | low |
| G11 | `page_target` mixes `"22"`, `"18-26"` and `"dynamic"` | `styles/*/style.json` | one `page_range` concept; `paper` is a legacy parity fixture | medium |
| G12 | `aesthetic` duplicates `id` in every pack | `styles/*/style.json` | replace with `role`/`preset_of` metadata | medium |
| G13 | CLI help says `--style` is “paper or clean” | `folisift-cli` scaffold help | list all packs or point to `styles list` | low |

### 1.4 What is not wrong

These should not be “fixed”:

- The paper palette, fonts, cover, running header and overall editorial
  character are good and distinctive.
- The A–G long-form structure works for course guides.
- D2 diagrams, canonical coverage, visual audit, linked TOC and the production
  report are solid foundations.
- PDF/UA tagging, bookmarks and glyph coverage pass today.
- `paper` remains the legacy 22-page parity reference until an ADR replaces
  exact page parity with structural parity.

## 2. Evidence-based revision format

The full evidence table and references are in
[`docs/revision-format-research.md`](../revision-format-research.md). The short
version:

| Design choice | Evidence | Strength | What it means for Folisift |
|---|---|---|---|
| Retrieval prompts with feedback | Adesope et al. 2017 (g≈0.61); Rowland 2014 (g≈0.50); Pan & Rickard 2018 (transfer d≈0.40); Dunlosky et al. 2013 | strong | Required recall section, answers separated |
| Spaced review plan | Cepeda et al. 2006; Dunlosky et al. 2013 | strong | Small schedule block, concrete tasks |
| Concept map, constructed not just viewed | Schroeder et al. 2018 (g≈0.58; construct 0.72; study 0.43); Nesbit & Adesope 2006 | moderate | One map per source, plus “redraw/fill” prompt |
| Graphic organizers / matrices | 2025 meta-analysis: learner-generated g≈0.59 memory / 0.63 comprehension; instructor-provided g≈0.70 / 0.53; Kiewra et al. 1991 | moderate | Partial matrices/maps that the learner completes |
| Generative drawing | Wammes et al. 2016 (drawing effect); Fiorella & Zhang 2018 (boundary conditions) | moderate | Blank/partial visual templates, not finished sketchnotes |
| Guided notes / partial structure | Larwin & Larwin 2013 (d≈0.55); Konrad et al. 2009 (school-age) | moderate | Fill-in prompts, not blank pages or finished prose |
| Worked examples, then fading | Barbieri et al. 2023 (g≈0.48); Sweller & Cooper 1985; Kalyuga et al. 2003 | strong/moderate | Full example → faded step → independent task |
| Comparison tables / matrix notes | Kiewra et al. 1991; Dexter & Hughes 2011 (heterogeneous); Mayer spatial contiguity | moderate/mixed | Tables for contrast; captions state the takeaway |
| Summarization | Dunlosky et al. 2013 (low); Graham & Hebert 2011 (g≈0.40 comprehension) | mixed/low | One-page synthesis as consolidation, not the main method |
| Coherence / signalling | Mayer (median 1.11 coherence, 0.60 signalling); Noetel et al. 2022 | moderate | Remove decoration; use headings and labels as signals |
| Constructive engagement | Chi & Wylie 2014 (ICAP) | moderate | Ask the learner to complete, compare, explain, answer |
| Typography / accessibility | WCAG 2.2; PDF/UA; Dyson & Haselgrove 2001; Section 508; UKAAF 2024 | standard/empirical | 55–80 chars, 4.5:1 contrast, tables ≥9 pt, refs ≥8 pt |

Claims policy: use “evidence-informed”, never “proven to raise exam scores”.
A PDF can cue retrieval and spacing; it cannot enforce either. The research
note’s 6–12 page range is the outer envelope; this plan recommends 8–12 pages
for a 100-slide revision sheet and 5–7 for a 40-slide source.

## 3. Genre-first model

### 3.1 Current packs are typographic presets

| Id | Actual role | What it really is | Intended use | Verdict |
|---|---|---|---|---|
| `paper` | legacy layout | 9.5 pt Noto Serif, cream, 16.5 mm margins, 1.8 mm paragraph spacing, 22-page parity | legacy long-form course guide; parity reference | keep as **legacy layout** |
| `reader` | density preset | `paper` + 10.0 pt, 17 mm, 0.70 em leading, 2.6/4.0 mm spacing | long-form reading, new guides | **not a genre**; `paper` + spacious |
| `clean` | layout | Inter body, white, blue, compact, single cover rule | code/technical reports, data-heavy briefs | keep as **layout** |
| `journal` | preset | `paper` + 10.5 pt, 18 mm, 0.72 em leading | humanities IMRAD/literature review | **paper preset**, not a journal template |
| `brief` | preset | `clean` + 10.5 pt, 17 mm, compact | business decision memo | **clean preset**, not a memo system |
| `accessible` | modifier | `paper` + 12 pt, 20 mm, 0.80 em leading | large-print reading | **accessibility modifier**, incomplete |

The honest conclusion: **there is no genre difference here**. The user is
right. A font-size change is not a design logic.

### 3.2 Three layers

| Layer | Question | Folisift examples |
|---|---|---|
| **Genre** | What is this document for, and who is the reader? | reader, revision sheet, knowledge organiser, guided notes, workbook, reference sheet, academic paper, brief, clinical/legal note |
| **Structure** | What content units and sequence does the genre require? | map → definitions → matrix → example → traps → recall → synthesis; IMRAD; IRAC; SOAP; BLUF |
| **Style** | How is it dressed? | body size, leading, margins, palette, font, density, accessibility |

Dependency:

```text
purpose / audience → genre → structure + layout logic → typography / palette
```

No more `style pack → same layout for every purpose`.

### 3.3 Genre matrix

| Genre | Reader task | Layout logic | Content grammar | Evidence | Folisift implementation |
|---|---|---|---|---|---|
| **Long-form reader** | Read continuously, build a model, return | Single column 55–75 chars, chapter openers, TOC, running header | Prose + micro-paragraphs, definitions, examples, figures, tables, chapter summary | Signaling, coherence, dual coding; genre convention | `paper` / `reader` + `a-g` or `imrad` |
| **Revision sheet** | Scan, recall, compare, practise, self-check | Modular grid, full-width maps/tables, compact card stack, recall block, answer key | Map → must-know → comparisons → worked example → traps → recall → one-page synthesis → review plan | Retrieval, spacing, concept maps, worked examples, graphic organizers | new `revision` layout + `revision-sheet` structure |
| **Knowledge organiser** | Look up and self-quiz one unit on one page | Single page, quadrant/grid: vocabulary, timeline, key terms, big questions, one diagram | 10–15 terms, 5–8 dates/people, 1 map/timeline, 5 big questions; no worked examples | Graphic organizers, concept maps, coherence; practitioner genre | `revision` layout + `knowledge-organiser` structure |
| **Guided notes** | Fill blanks, add cues, summarise, recite | Cornell-like cue/notes/summary zones; blanks | Provided framework, key terms, cues, summary; no finished answers | Guided notes d≈0.55; note-taking mixed | `notes` layout + `lecture-notes` structure |
| **Workbook** | Solve, check, reflect, redo | Problem block → workspace → solution key; faded examples | Worked example, faded example, independent problems, mixed review, answers | Worked examples, retrieval, self-explanation, ICAP | `workbook` layout + `workbook` structure |
| **Reference sheet** | Find, verify, apply | Multi-column tables/cards; units; symbols; conditions | Formulas, definitions, units, assumptions, common errors | Information design/typography; not first-learning | `reference` layout + `formula-sheet` structure |
| **Academic paper/report** | Evaluate claims, methods, evidence, contribution | IMRAD/disciplinary structure, abstract, citations, figures/tables | Claim → evidence → method → analysis → limitation; prose | Genre convention (Swales, Hyland) | `paper`/`journal` + `imrad`/`literature-matrix` |
| **Executive brief** | Decide quickly | BLUF/answer-first; findings; recommendation; risks; owner | One governing thought, 2–4 arguments, evidence, caveats | Professional standard (Minto/BLUF) | `clean`/`brief` + `executive-brief` |
| **Clinical/legal note** | Follow facts, rules, assessment, plan; audit | SOAP/IRAC fields, structured headings, uncertainty | Subjective/objective/assessment/plan or issue/rule/application/conclusion | Professional genre conventions | `paper`/`clean` + `soap`/`irac` |

### 3.4 Genre gates

A genre is only real if it can fail for the right reason.

| Genre | Must fail if |
|---|---|
| Long-form reader | No coherent prose; only bullets; no section model; underfilled pages |
| Revision sheet | Paragraph >60 words; no concept map; no recall prompts/answer key; no traps; no synthesis; >12–14 pages without reason |
| Knowledge organiser | More than one page per unit; >15 terms; no map/timeline; tiny type; worked examples crammed in |
| Guided notes | No blanks/cues/summary; answers already filled in; no review cue |
| Workbook | No workspace; no answer key; no faded example; solutions on the same page |
| Reference sheet | Unexplained symbols/units; prose >2 sentences; missing conditions/assumptions |
| Academic paper | No citations/method/limitations; bullet-only argument; unsourced claims |
| Executive brief | Recommendation buried; no risks/owner; >4 pages; chronology instead of answer |
| Clinical/legal note | Missing facts/jurisdiction/uncertainty; invented diagnosis/outcome |

### 3.5 What to build first

Do not build all genres at once. The user’s priority is the visual revision
genre.

| Release | Genres | Why |
|---|---|---|
| **v0.10** | Long-form reader (existing), **revision sheet**, **knowledge organiser** | directly addresses the exam-revision request; reuses maps/tables/cards |
| **v0.11** | Guided notes, workbook, reference sheet | distinct layouts and gates; needs user testing |
| **existing** | Academic paper, brief, clinical/legal note | keep as structures/presets; do not pretend they are full genre systems |

### 3.6 Axes

| Axis | Values | What it controls |
|---|---|---|
| `genre` | `reader`, `revision-sheet`, `knowledge-organiser`, `guided-notes`, `workbook`, `reference-sheet`, `academic-paper`, `brief`, `clinical-note`, `legal-note` | purpose, structure, layout logic, gate |
| `layout` | `paper`, `clean`, `revision`, `notes`, `workbook`, `reference` | template, palette, component shapes |
| `structure` | `a-g`, `revision-sheet`, `knowledge-organiser`, `lecture-notes`, `workbook`, `formula-sheet`, `imrad`, `irac`, `soap`, `literature-matrix`, `executive-brief` | required sections and content grammar |
| `density` | `compact`, `comfortable`, `spacious` | body size, leading, margins, spacing |
| `accessibility` | `standard`, `large-print`, `high-contrast` | minimum sizes, contrast, reflow rules |
| `voice` | `academic`, `plain`, `technical`, `legal`, `clinical`, `business` | tone and sentence style |
| `citation` | `page-ref` | traceability format |

`reader` is `paper` + `spacious`; `journal` is `paper` + `comfortable` +
`imrad`; `brief` is `clean` + `compact` + `executive-brief`; `accessible` is
`paper` + `large-print`. The old ids remain **compatibility aliases** for one
release and are marked `preset_of` / `role: "modifier"` in `style.json`.

### 3.7 What to tell users

| User need | Genre | Do not choose |
|---|---|---|
| Full course guide, exam preparation over weeks | reader + `a-g` | brief, one-pager |
| Short revision before an exam | **revision sheet** (`revision-sheet`) | reader with smaller font |
| Display a unit's core facts | **knowledge organiser** | full revision sheet |
| Take notes in class | guided notes (`lecture-notes`) | finished reader |
| Practise problems | workbook | revision sheet without workspace |
| Look up formulas | reference sheet (`formula-sheet`) | reader |
| Write a paper/report | academic paper (`imrad`) | revision sheet |
| Decide in a meeting | executive brief (`executive-brief`) | academic paper |
| Record a case | clinical/legal note (`soap`/`irac`) | revision sheet |
| Large-print version of any genre | same genre + `accessible` modifier | `accessible` as a genre |

### 3.8 Backward compatibility

- Schema 3 guide configs keep loading. `style.aesthetic` remains the pack id.
- New optional `style.genre`, `style.density` and `style.accessibility` fields
  default from the selected pack.
- `style.json` schema 2 adds `role`, `preset_of`, `genre`, `density`,
  `accessibility`; existing schema 1 packs load with inferred defaults.
- `styles list` shows role and preset relationship, so users can see that
  `reader` is not a genre.
- No existing guide is rewritten. The revision pilot is a **new bundle**.

## 4. Revision-sheet and organiser specification

### 4.1 Structure

New `structures/revision-sheet.json`, with optional sections:

| # | Section (TR / EN) | Required | Purpose |
|---|---|---|---|
| 1 | Kapsam ve harita / Scope and map | yes | source range, 5–8 must-know ideas, concept map |
| 2 | Bilmen gerekenler / Must-know | yes | precise definitions and distinctions |
| 3 | Karşılaştırma / Comparisons | yes | 1–3 contrast tables |
| 4 | Çözümlü örnekler / Worked examples | optional* | procedure, step, why, faded practice |
| 5 | Sınav tuzakları / Exam traps | yes | common errors and corrections |
| 6 | Hatırlama / Recall | yes | 8–12 questions with separated answers |
| 7 | Tek sayfa sentez / One-page synthesis | yes | 5–8 bullets, 2 comparisons, 3 traps, 5 terms, 1 map |
| 8 | Tekrar planı / Review plan | yes | same day, 2–3 days, 1 week, 2 weeks |

\* Required for `cs`, `math`, `business` and other procedural domains; optional
for source sets with no procedures. The structure pack gets a `required` flag
per section; `build --check` fails only on missing required sections and reports
optional omissions as warnings.

### 4.2 New directives

| Directive | Renders | Rules |
|---|---|---|
| `::: must-know` | full-width compact card stack of definitions | ≤ 8 cards per page; each ≤ 25 words; source ref |
| `::: synthesis` | full-page summary with 2 columns and a final checklist | one per guide; 5–8 bullets, 2 comparisons, 3 traps, 5 terms; no new facts |
| `::: answer` | answer-key block at the end of the section/guide | muted, smaller, top rule; never adjacent to the question |
| existing `::: critical label="SINAV TUZAĞI"` | trap box | “if you see X, do Y, not Z” |
| existing `::: example label="ÇÖZÜMLÜ ÖRNEK"` | worked example | step → why → result; last step faded |
| existing `::: recall` | retrieval prompt | specific, answerable from the source |
| existing `::: review-plan` | spaced-review block | concrete tasks, not generic advice |
| existing `::: diagram id=...` | concept map / process diagram | D2 only; caption with takeaway; source ref |

### 4.3 Layout and typography

A4 portrait, 30–32 mm side margins (target 55–80 characters at 11 pt; verify
with the line-length audit), 18 mm top, 16 mm bottom.

| Element | Size | Notes |
|---|---|---|
| Body | 11.0 pt Inter | left-aligned, `leading_em: 0.72`, no justification |
| H1 / section band | 22 pt | 2.0× body |
| H2 | 15.4 pt | 1.4× body |
| H3 | 12.1 pt | 1.1× body |
| Lead | 12.3 pt | 1.12× body |
| Table body | 9.5 pt | 0.86× body; minimum 9 pt |
| Table header | 9.5 pt | sentence case or small caps; tracking 0.04em |
| Caption | 9.5 pt | italic; takeaway, not “Table 1” |
| Label / meta | 8.4 pt | uppercase; tracking 0.08em |
| Source ref | 8.4 pt | `faint` must pass 4.5:1 |
| Code | 8.9 pt | monospace |
| TOC title / sub / page | 10.5 / 9.7 / 9.0 pt | real links; page numbers match |

Rules:

- No component below **8 pt**; tables/captions at least **9 pt**; large print
  no component below **9 pt**.
- No paragraph over **60 words / 3 sentences**; 1–3 sentence micro-prose is
  allowed for causal chains and “why”, not a 300-word textbook block.
- No bullet over **25 words**; no list over **7 items**.
- No table over **5 columns** or **8 rows** per page.
- Every section has at least one visual, table or recall prompt.
- Every figure has a takeaway caption and source page.
- No decorative images; no colour-only meaning.
- No duplicate definitions; cross-reference.
- No answer-free quiz; every question has an answer in a separate key, not on
  the same line or facing page.

### 4.4 Page flow

```text
p1  title band | how to use (3 bullets) | mini TOC
    concept map (full width, takeaway caption)
p2  must-know cards (2 columns, 6–8 cards)
p3  comparison table 1 | exam traps
p4  comparison table 2 | worked example
p5  worked example (faded step) | what-if variation
p6  recall questions (8–12)
p7  answers | review plan
p8  one-page synthesis (full page)
```

A 100-slide source should land around **8–12 pages**, not 22; a 40-slide source
around **5–7 pages**. If a procedural source needs more room for worked
examples, 12–14 pages is acceptable; if it exceeds that, cut prose and duplicate
content, never type size. These are observational ranges; the gate checks
content and density, not a fixed count.

### 4.5 What a revision sheet must not become

- A textbook chapter with headings and long paragraphs.
- A slide dump with one bullet per slide.
- A deck of decorative mind maps.
- A quiz book with no answers.
- A “cheat sheet” that omits traceability.
- A large-print guide whose tables are still 6.9 pt.

### 4.6 Knowledge organiser (one page per unit)

This is a separate genre, not a short revision sheet.

| Section | Content | Rule |
|---|---|---|
| Büyük sorular | 5 questions the unit answers | answerable from the source; no generic motivation |
| Anahtar sözcükler | 10–15 terms | one-line definition + source; no duplicate glossary |
| Kronoloji / zaman çizelgesi | 5–8 dates/people/events | only if the source has a chronology |
| Harita veya şema | one dominant map/timeline/process | labelled edges; one takeaway caption |
| Kritik ayrımlar | 3–5 contrasts | “X vs Y → consequence” |
| Kaynak aralığı | source pages | footer, not a paragraph |

Rules: exactly one page per unit; no worked examples; no prose over two
sentences; type floor and contrast still apply; if it does not fit, cut scope or
split the unit, never shrink type.

## 5. Typography harmony fixes

### 5.1 Replace string replacement with a type scale

Add a `TypographyScale` in `folisift-render`:

```rust
struct TypographyScale {
    body: f64,
    h1: f64,
    h2: f64,
    h3: f64,
    lead: f64,
    caption: f64,
    label: f64,
    table: f64,
    code: f64,
    source: f64,
    toc_title: f64,
    toc_sub: f64,
    toc_page: f64,
}
```

Computed from tokens:

```json
{
  "typography": {
    "body_size": 11.0,
    "leading_em": 0.72,
    "scale": {
      "h1": 2.0,
      "h2": 1.4,
      "h3": 1.1,
      "lead": 1.12,
      "caption": 0.86,
      "label": 0.76,
      "table": 0.86,
      "code": 0.81,
      "source": 0.76,
      "toc_title": 0.95,
      "toc_sub": 0.88,
      "toc_page": 0.82
    },
    "floor": {
      "component_pt": 8.0,
      "table_pt": 9.0,
      "accessible_component_pt": 9.0
    }
  }
}
```

Rules:

- `component = max(floor, body × ratio)`.
- `body_size` must be 9–18 pt.
- `body / smallest_component ≤ 1.45`; for accessible ≤ 1.35.
- `line_height` is removed; `leading_em` is the only vertical-rhythm token.
  Docs explain that Typst `leading` is extra space between lines, not CSS
  `line-height`.
- If `paper` must keep exact v0.9 parity, it gets a `legacy_scale` flag and is
  exempt from the new floors; every new style uses the scale.
- Templates use placeholders (`__BODY_SIZE__`, `__TABLE_SIZE__`, …), not
  hard-coded point sizes and not global `9.5pt` replacement.

### 5.2 Measure, alignment and hyphenation

- `paper` stays frozen for the v0.9 parity fixture.
- `reader` uses 26–28 mm side margins; `revision` uses 30–32 mm. Their measured
  p90 must be ≤ 80 characters.
- Body text is left-aligned, not justified, in `reader` and `revision`.
- `paper` keeps justification only if an ADR accepts the parity trade-off;
  otherwise it moves to left-aligned in the same release that retires exact
  page parity.
- Table cells are never justified.
- Add an audit metric: `line_chars_p50`, `line_chars_p90`, `line_chars_max`,
  `long_line_pages`. Strict mode fails above the style limit.

### 5.3 Contrast

- Darken `faint` from `#8A7A63` to `#75664E` (5.17:1 on paper, 4.75:1 on cream,
  4.71:1 on cover).
- Keep `clean` `faint` at `#6B7280` (4.83:1 on white).
- Validate every text role against every background where it is used:
  `ink`, `muted`, `soft`, `faint`, `accent`, `secondary`; not just `ink/paper`.
- Rules are allowed to use `gold`; text is not, unless it passes 4.5:1.
- Add a unit test with the actual palette pairs.

### 5.4 Template architecture

- `templates/paper.typ`, `clean.typ`, `revision.typ` share the same type-scale
  placeholders.
- `styles.rs` computes the scale once and replaces placeholders.
- Add a `templates/README` or comments explaining which sizes are tokens and
  which are fixed (cover display type, running header, footer).
- Add a fixture `tests/fixtures/typography-all.md` containing H1/H2/H3, lead,
  body, definition, note, example, table, caption, code, TOC and cover notes.
- Add a test that renders every style and asserts the computed scale; add a
  local `scripts/check-type-scale.py` (PyMuPDF, dev-only) that verifies the
  emitted PDFs.

## 6. Implementation plan

### Phase 0 — Baseline and regression harness

- [ ] Record v0.9.0 baselines: page counts, TOC links, font histograms, line
      lengths, audit results, visual hashes.
- [ ] Add `tests/fixtures/typography-all.md` and a style render test.
- [ ] Add a dev script `scripts/check-type-scale.py` that reports font
      histogram and line-length percentiles for a PDF.
- **Exit:** baseline report committed; no production changes yet.

### Phase 1 — Typography token system

- [ ] Add `TypographyScale` and placeholder replacement in
      `crates/folisift-render/src/styles.rs`.
- [ ] Replace hard-coded sizes in `templates/paper.typ` and `clean.typ` with
      placeholders.
- [ ] Remove `line_height` from built-in tokens; document `leading_em`.
- [ ] Darken `faint` in the paper-family palettes (`paper`, `reader`,
      `journal`, `accessible`); add all-text-role contrast tests.
- [ ] Add unit tests for scale floors, ratios, and contrast.
- [ ] Verify `paper` still builds; if parity breaks, either stop for an ADR or
      keep a `legacy_scale` flag for `paper` and apply the new scale to
      `reader`, `revision`, `clean` and `accessible`.
- **Exit:** paper/clean render with the new scale; paper parity either holds or
      has an approved ADR / legacy flag.

### Phase 2 — Revision sheet and knowledge organiser

- [ ] Add `styles/revision/{style.json,tokens.json,strings,preview.md}`.
- [ ] Add `structures/revision-sheet.json` and
      `structures/knowledge-organiser.json`; add `required` flags and optional
      section handling in `folisift-core/src/structures.rs`.
- [ ] Add `templates/revision.typ` with two modes:
      `revision` (multi-page) and `organiser` (one page per unit).
- [ ] Revision mode: compact title band, concept-map page, card grid, table
      block, trap/worked/recall/answer components and one-page synthesis.
- [ ] Organiser mode: quadrants for vocabulary, timeline, key terms, big
      questions and one dominant map/diagram; no worked examples.
- [ ] Add `must-know`, `synthesis`, `answer` and `organiser` directives in
      `crates/folisift-render/src/markdown.rs`.
- [ ] Add embedded assets/string arms in `styles.rs`.
- [ ] Add `revision` and `knowledge-organiser` to `STRUCTURES` and `LAYOUTS`
      in `folisift-core/src/config.rs`.
- [ ] Tests: render every new directive in TR/EN; structure pass/fail; optional
      sections; one-page organiser; no raw Typst; D2 diagram asset.
- **Exit:** `styles preview revision` builds a 4-page revision preview and a
      1-page organiser; `scaffold --structure revision-sheet` and
      `--structure knowledge-organiser` produce the required headings.

### Phase 2B — Guided notes, workbook and reference sheet (v0.11)

These are specified now but not part of the v0.10 release.

- [x] `notes` layout: Cornell-like cue/notes/summary zones; blanks; no answers.
- [x] `workbook` layout: problem block, workspace, faded example, answer key.
- [x] `reference` layout: formula cards, units, conditions, comparison tables.
- [x] Structures: `workbook`, `formula-sheet` refinement; `lecture-notes`
      already exists.
- [x] Add genre fixtures and visual baselines for each.
- **Exit:** each genre has a preview, a gate and at least one fixture.

### Phase 3 — Genre grammar gates

- [x] Add `crates/folisift-parse/src/genre.rs` with per-genre checks:
      - revision: paragraphs > 60 words, bullets > 25 words, lists > 7,
        tables > 5 columns/8 rows, missing map, recall answers or synthesis;
      - knowledge organiser: more than one page per unit, > 15 terms, no
        map/timeline, worked examples crammed in;
      - guided notes: no blanks/cues/summary, answers filled in;
      - workbook: no workspace, no answer key, no faded example;
      - reference: unexplained symbols/units, prose > 2 sentences, missing
        conditions.
- [x] Add `build --check` integration by genre; existing A–G guides are not
      affected. For v0.10, implement revision + organiser gates; the
      notes/workbook/reference gates are specified now and enforced in v0.11.
- [ ] Add `prepare_guide` genre checklist and `build_guide` genre report.
- [ ] Add good/bad fixtures per genre.
- **Exit:** a deliberately bad revision sheet fails with actionable messages;
      a good revision sheet and a good organiser pass.

### Phase 4 — Genre and style taxonomy compatibility

- [ ] Add `genre`, `role`, `preset_of`, `density`, `accessibility` to
      `StylePack` (schema 2; schema 1 inferred).
- [ ] Add optional `genre`/`density`/`accessibility` to `StyleProfile`; keep
      `aesthetic` as pack alias.
- [ ] Update `styles list`, `style recommend`, `prepare_guide` and docs to show
      genre + layout + preset + modifiers.
- [ ] Mark `reader`, `journal`, `brief`, `accessible` as presets/modifiers;
      no new token-only genre packs.
- [ ] Migration tests: all v0.9 configs load; invalid combinations rejected
      with a clear message.
- **Exit:** `style recommend` explains *why* a genre fits; no user is told that
      `reader` is a different genre.

### Phase 5 — Audit and gates

- [x] Add `AuditOptions` with margins, min font, max line chars.
- [x] Add `audit --guide <slug>` to derive margins/floor from the guide style.
- [x] Add font histogram, line-length percentiles, long-line pages and
      style-aware min font to `LayoutAudit`.
- [x] Skip visual pages in `underfilled_pages`; count figure captions only on
      caption-like lines.
- [x] Add `make type-check` and `make genre-check`.
- [ ] Update `scripts/check-production-report.py` for the new report fields.
- **Exit:** `audit --guide pd2` reports the real margins and line lengths;
      `audit --guide pd2-rev --strict` passes only when the revision grammar
      passes.

### Phase 6 — Pilots, docs and release

- [x] Create `guides/pd2-rev/` and `guides/pd3-rev/` revision sheets from PD2
      (103 slides) and PD3 (dense semantics).
- [x] Create `guides/pd2-ko/` as a one-page knowledge organiser for one PD2
      unit; test the one-page constraint.
- [x] Build, audit, TOC-check and visual-audit all pilots.
- [ ] Expert review with the rubric in `docs/authoring-evidence.md` §6.1.
- [x] Rewrite `docs/design-system.md` as a multi-layout design system:
      palette, type scale, measure, component table, genre roles,
      accessibility modifiers.
- [ ] Update `docs/authoring.md`, `docs/authoring-spec.md`,
      `docs/authoring-evidence.md` (citation/clinical/quiz contradictions).
- [ ] Add an ADR: exact page parity → structural parity + page-range gate.
- [ ] Update `docs/VERSIONS.md`, README and release notes.
- [x] Run the full gate: tests, clippy, fmt, paper/clean/revision previews,
      PD0–PD1/PD2/PD3 builds, TOC links, strict audits, package checksum,
      clean install.
- **Exit:** v0.10.0 release candidate; no known typography or taxonomy bug.

## 6.5 Implementation status (v0.10)

| Area | Status | Evidence |
|---|---|---|
| `revision` layout + `revision-sheet` structure | done | `templates/revision.typ`, `structures/revision-sheet.json` |
| One-page `knowledge-organiser` structure | done | `structures/knowledge-organiser.json`, `templates/revision.typ` organiser mode |
| `must-know`, `synthesis`, `answer`, `organiser` directives | done | `crates/folisift-render/src/markdown.rs` |
| Revision grammar gate | done | `crates/folisift-core/src/genre.rs`; `build --check` |
| Revision/organiser pilots | done | `guides/pd2-rev/`, `guides/pd2-ko/` |
| Source-reference contrast | done | `faint` changed to `#75664E`; 5.17:1 on paper |
| Audit body-font/orphan detection | done | style-aware body size in `audit.rs` |
| v0.11 guided notes/workbook/reference | done | `styles/{notes,workbook,reference}`, `templates/{notes,workbook,reference}.typ`, genre gates and PDF baselines |
| v0.11 metadata migration (`genre`/`role`/`density` fields) | deferred | structure + aesthetic + palette/font layers cover the current surface |
| Paper/reader full type-scale migration | done in v0.11 | paper is a pinned legacy exception; reader/clean/revision/accessible use shared role sizes |
| `role`/`preset_of` genre metadata | deferred | the old `aesthetic` field remains the pack id; docs now describe the real roles |

The v0.10 gate is `cargo test`, clippy, fmt, the revision pilot build and
strict audit, the organiser one-page audit, and the existing paper parity and
visual checks. v0.11 adds the remaining genre layouts.

## 7. Acceptance criteria

### Product and taxonomy

- [ ] Every style has a documented role: layout, preset or modifier.
- [ ] Every genre has a purpose, reader task, layout logic and gate.
- [ ] `paper` and `reader` are explicitly described as the same layout at
      different density; no claim that they are different genres.
- [ ] `journal` and `brief` are documented as presets, not full journal/memo
      templates, unless genre-specific components are added.
- [ ] `accessible` scales body, headings, tables, captions, labels, TOC and
      source refs; 12 pt body with 9 pt minimum components.
- [ ] `revision` is a distinct layout + structure, not a token skin.
- [ ] `knowledge-organiser` is a one-page genre, not a short revision sheet.
- [ ] No style can be selected with an incompatible structure/domain without a
      clear error.
- [ ] A genre gate fails for the right reason: revision on long paragraphs;
      reader on bullet-only content; organiser on multi-page sprawl; brief on a
      buried recommendation.

### Revision pilot

- [ ] 8–12 pages for a 100-slide source; 5–7 for a 40-slide source; no fixed
      page count.
- [ ] 0 paragraphs > 60 words; 0 bullets > 25 words; 0 lists > 7 items.
- [ ] At least 1 concept map, 2 comparison tables, 1 worked example (where
      applicable), 8 recall prompts with answers, 1 synthesis page, 1 review
      plan.
- [ ] Every non-obvious claim has a source page reference.
- [ ] `build --check` and `audit --strict` pass; no underfilled or overflowing
      pages.
- [ ] TOC links and page numbers match.
- [ ] Visual audit covers every low-text page.

### Knowledge-organiser pilot

- [ ] Exactly one page for the chosen unit.
- [ ] 10–15 key terms, 5–8 dates/people, 1 map/timeline, 5 big questions.
- [ ] No worked examples; no long prose; every term labelled and sourced.
- [ ] Type floor and contrast pass; no clipped content.

### Typography

- [ ] Body/h1/h2/h3/lead/caption/table/label/source ratios match the declared
      scale.
- [ ] No component below 8 pt; tables/captions at least 9 pt; no accessible
      component below 9 pt.
- [ ] No text colour below 4.5:1 on its actual background.
- [ ] Line-length p90 ≤ 80 characters in `reader` and `revision`.
- [ ] Body text is left-aligned in `reader` and `revision`; table cells are
      never justified.
- [ ] TOC sizes scale with body size; no 8.2 pt TOC in a 12 pt guide.
- [ ] Cover notes are at least 8 pt.

### Gates and regression

- [ ] `audit --guide` uses the guide's real margins, not hard-coded paper
      margins.
- [ ] Font histogram and line-length stats are reported.
- [ ] Underfilled-page logic does not flag legitimate visual pages.
- [ ] All v0.9 configs still load; old guides still build.
- [ ] Existing tests plus new tests pass; clippy and fmt pass.
- [ ] Package checksum and clean install pass.
- [ ] No stale “APA/IEEE/OSCOLA implemented”, “clinical/casebook exists” or
      “no quiz material in any genre” claims remain.

## 8. Non-goals

- No fixed page count. The 22-page paper guide remains a legacy parity fixture,
  not a quality target.
- No claim of proven learning gains. The revision format is evidence-informed
  and must be evaluated with students.
- No decorative diagrams, stock photos or “motivational” filler.
- No new token-only style that pretends to be a genre.
- No LLM call, network access or shell execution inside the MCP server.
- No rewrite of the existing PD0–PD1/PD2/PD3 guides; the pilot is additive.
- No change to the D2-only diagram pipeline.

## 9. Risks

| Risk | Mitigation |
|---|---|
| Revision sheets become long textbooks | Hard grammar gate: paragraph/bullet/table limits, page range, one-page synthesis |
| Hard limits produce shallow content | Domain override with a recorded reason; expert review; retrieval prompts require real answers |
| Concept maps become decoration | One map per source; labelled edges; takeaway caption; “redraw/fill” prompt |
| Accessibility pack still has small components | Token floor + contrast + font histogram gate; 9 pt minimum |
| Type-scale migration breaks paper parity | Phase 1 stops on parity failure; ADR decides between exact and structural parity before proceeding |
| Audit false positives on visual pages | Skip visual pages in underfilled logic; report separately |
| Too many style combinations | `style recommend` returns a small compatible set; invalid combos rejected |
| User preference for a different look | Pilot both serif and sans revision previews before locking the pack |

## 10. Decisions requested before implementation

1. **Parity:** may v0.10 replace exact 22-page parity with structural parity +
   page-range gates? Recommended: yes; keep the old PDF as a visual reference,
   not a fixed page target.
2. **Revision pilot:** create new `pd2-rev` / `pd3-rev` bundles instead of
   converting the existing guides? Recommended: yes.
3. **Revision body font:** Inter 11 pt for scannability, or Noto Serif 11 pt
   to stay closer to the paper family? Recommended: Inter for the revision
   sheet, because it is bullet/table-heavy and read on screen.
4. **Page range:** is 8–12 pages for a 100-slide source and 5–7 for a
   40-slide source the right target? Recommended: yes.
5. **Accessibility:** confirm `accessible` becomes a modifier (`paper` +
   large-print), not a peer style. Recommended: yes.
6. **Genre scope:** v0.10 = revision sheet + knowledge organiser; v0.11 =
   guided notes + workbook + reference sheet. Recommended: yes, so the visual
   revision genre is finished before adding more layouts.
7. **Micro-prose:** confirm the rule is “no long paragraphs”, not “no
   paragraphs”. Recommended: 1–3 sentence micro-prose is allowed for causal
   chains and “why”; bullets/tables/maps dominate.

No code will be changed until these are confirmed. The next step after approval
is Phase 0 (baseline) and Phase 1 (type scale), with a hard stop if paper parity
cannot be preserved.
