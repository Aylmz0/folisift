# Revision-sheet format: evidence, layout rules and a document grammar

- **Status:** Research note for the guide-quality and style work
- **Date:** 2026-10-30
- **Scope:** exam-preparation / revision documents for university students;
  graphic-heavy where useful, segment-oriented, table-rich, concept-map oriented,
  concise, designed for retrieval and review
- **Related:** [`authoring-evidence.md`](authoring-evidence.md),
  [`document-genres-and-layout-systems.md`](document-genres-and-layout-systems.md),
  [`authoring-spec.md`](authoring-spec.md), [`design-system.md`](design-system.md)
- **Claim policy:** evidence-informed, not a claim that a particular PDF improves
  exam scores. Strong evidence supports *techniques*; the page grammar below is a
  design synthesis, not a validated treatment.
- **Scope note:** this note is about the **revision-sheet genre only**. The full
  genre system (reader, knowledge organiser, guided notes, workbook, reference,
  academic paper, brief, clinical/legal note) is in
  [`document-genres-and-layout-systems.md`](document-genres-and-layout-systems.md).

## 1. Key findings

1. **Retrieval and distributed practice are the only techniques Dunlosky et al.
   rate high utility.** Practice-testing effects are moderate-to-large (Adesope
   et al. 2017, *g* = 0.61; Rowland 2014, *g* ≈ 0.50); transfer is smaller
   (*d* = 0.40; Pan & Rickard 2018). **Strong.**
2. **Interleaving is conditional.** Overall *g* = 0.42; mathematics 0.34;
   expository texts ambiguous; word lists reverse (*g* = −0.39; Brunmair &
   Richter 2019). Use for similar categories and “which method?” choices, not as
   a universal shuffle. **Moderate.**
3. **Worked examples help novices in procedural domains** (*g* = 0.48; Barbieri
   et al. 2023), but need fading and decision-point annotations. Expertise
   reversal applies; self-explanation prompts can reduce the effect in some
   formats. **Strong for novices, moderate for transfer.**
4. **Concept maps: construction ≠ study.** Overall *g* = 0.58; constructing
   *g* = 0.72, studying premade maps *g* = 0.43 (Schroeder et al. 2018). A PDF
   can supply maps for study; it cannot force construction. **Moderate.**
5. **Matrix notes beat conventional notes for recall** (Kiewra et al. 1991);
   contrasting cases improve transfer (Schwartz & Bransford 1998). Tables work
   for discrimination, not for storing prose. **Moderate.**
6. **Cornell and guided notes are weaker than their reputations.** Guided notes
   are supported mainly in school-age/special-education settings; Cornell lacks a
   strong university experimental base. The components (cues, provided framework,
   review) are better supported than the branded systems. **Mixed / weak.**
7. **Summarization is low utility as a standalone technique** (Dunlosky et al.
   2013), but structured summary writing can support comprehension (*g* = 0.40;
   Graham & Hebert 2011). It is not a substitute for retrieval. **Mixed.**
8. **Typography evidence is weaker than learning-technique evidence.** Line
   length, type size, white space and contrast are mostly standards/heuristics.
   WCAG 1.4.8 AAA recommends ≤ 80 characters and non-justified text; Section 508
   calls running text below 9 pt a barrier; UKAAF clear print starts at 12 pt.
   **Standard, with moderate reading-comfort evidence.**
9. **`paper`/`reader` conflict with scannable revision design.** A4 single-column
   at 16.5–17 mm margins and 9.5–10 pt yields long lines (body-size lines:
   median 92–105, p90 105–112 characters); text is justified with hyphenation off; table
   text is 6.9–7.9 pt; refs are 6.9 pt at 3.86:1 contrast. These are reader
   layouts, not revision-sheet layouts. **High.**
10. **A revision sheet needs its own grammar, not a shortened textbook.** Unit:
    map → definitions → matrix → worked example → traps → recall prompts →
    one-page synthesis. A chapter unit is 2–4 pages; a one-source revision sheet
    is 8–12 pages for a dense deck; a long-form course guide is 16–28 pages.
    These are observations/house rules, not fixed targets.
11. **Decoration, duplication and answer-free quizzes are the main failure
    modes.** Coherence/removing seductive details is robust; decorative images,
    colour-only meaning, repeated definitions and answer-free quizzes add load
    without retrievability.
12. **The “no quiz material” rule contradicts the evidence note and renderer.**
    `authoring-spec.md` forbids practice questions; the evidence note recommends
    optional recall prompts; the renderer supports `recall`/`explain`/
    `review-plan`. Distinguish a padded quiz book from optional retrieval prompts
    with an answer key.

## 2. Method and evidence labels

Meta-analyses and systematic reviews first, experiments second, accessibility
standards third, professional typography guidance last. Effect sizes appear only
where the source states them. **Strong** = consistent meta-analytic evidence;
**moderate** = experimental evidence with boundary conditions or heterogeneous
meta-analysis; **mixed** = depends on task, learner, implementation or
measurement; **standard** = professional/accessibility convention, not a
learning-outcome claim; **heuristic** = house rule or project hypothesis.

A technique can be strong while a *document format* built around it is untested.
There are no RCTs of “the one-page revision sheet” as a genre. A standard can be
worth following without being an empirical learning claim.

## 3. Technique evidence

| Principle | Finding | Strength | Design use |
|---|---|---|---|
| Retrieval practice | Practice tests beat restudying (Roediger & Karpicke 2006; Adesope 2017); transfer *d* = 0.40 (Pan & Rickard 2018) | strong | Specific Q&A after explanation; answers + feedback (Butler & Roediger 2008); low stakes |
| Distributed practice | Spacing beats massing; optimal gap ~20–40% of a 1-week delay, 5–10% of a 1-year delay | strong | Review prompts at 24 h, 3 d, 7 d; cumulative later |
| Interleaving | *g* = 0.42 overall; math 0.34; texts ambiguous; words −0.39 | moderate | Mixed problem types; “which method applies?”; cross-topic matrices |
| Self-explanation | Explaining why improves understanding; depends on prior knowledge and prompt quality | moderate | “Why is this valid?”, “What assumption breaks it?”; model answers |
| Worked examples | *g* = 0.48 in maths; correct > erroneous; prompts can reduce effect | strong/moderate | Full → faded → independent; annotate decisions; add variation |
| Multimedia/dual coding | 11 principles positive; largest: contiguity, signaling, captioning; coherence robust | strong for principles | Adjacent labels; headings; segmenting; no decoration; diagrams for mechanism |
| Cognitive load | Segment complex material; remove extraneous load; avoid split attention | strong | Short units; direct labels; no duplicate text; novice scaffolding |

Caveats: static PDF cannot use modality, temporal contiguity or animation; do not
generalise video findings to print. “Dual coding” is not “add a picture”, and it
is not evidence for learning-style matching (Pashler et al. 2008). Interleaving
feels harder and increases initial errors; label it as desirable difficulty.
Worked examples reverse for experts (Kalyuga et al. 2003). Self-explanation can
backfire for novices; pair prompts with a model answer.

## 4. Note-taking and revision formats

| Format | Evidence | Strength | Best use | Failure mode |
|---|---|---|---|---|
| Verbatim/slide copies | Capture without processing | weak | Archiving | Illusion of competence |
| Outline notes | Organises hierarchy | moderate/standard | Linear topics | Hides cross-links |
| Matrix notes | Greater recall than conventional notes (Kiewra 1991) | moderate | Contrasts, parameters | Oversimplifies mechanisms |
| Guided notes | Note accuracy + performance, mainly school-age (Konrad 2009) | moderate (secondary), weak (university) | Provided framework | Fill-in passivity |
| Cornell | Cue + summary fits retrieval, but direct evidence thin | mixed/heuristic | Student notes | Ritual without review |
| Concept map | *g* = 0.58; construct 0.72, study 0.43 | moderate | Structure/dependencies | Node spaghetti |
| Q&A/recall | Retrieval is strong; embedded format is moderate | strong/moderate | Definitions, mechanisms | Answer visible; no feedback |
| Bullet summary | Indirect signaling/segmenting support | heuristic | Parallel lists, steps | Fragmentation; lost causality |
| One-page synthesis | Structured summary + organisation | heuristic | Chapter consolidation | New content; overcrowding |

- **Concept maps:** use when the goal is structure. Label every edge with a verb
  (“causes”, “requires”, “is compiled to”). 8–12 nodes, one page, no crossing
  edges if avoidable. More than 12 nodes → split overview/detail or use a matrix.
- **Matrices:** rows = cases, columns = dimensions. Kiewra’s result supports the
  *discrimination task*, not any grid. Do not put a process in a matrix; use a
  numbered flow. A takeaway sentence is mandatory.
- **Guided notes:** borrow the principle (provided structure, active completion),
  not the worksheet aesthetic. A matrix with one column blanked, or a worked
  example with one step removed, is a better university variant.
- **Cornell:** treat it as a layout heuristic: cues left, explanation right,
  synthesis bottom. Do not claim it is proven.
- **Bullets:** one list principle; parallel items only. Mechanisms, arguments and
  causal chains need a numbered flow or diagram.

## 5. Summarization: low utility, useful as structured writing

Dunlosky et al. (2013) rate summarization **low utility**: students copy rather
than select; summaries can be long and vague; generalisable evidence is limited.
Brown and Day (1983) describe the macrorules expert summarisers use: delete
trivial/redundant material, generalise, construct a topic sentence. Those rules
are teachable. Graham and Hebert (2011) find writing about text improves reading
comprehension (*g* = 0.40), which supports structured summary *writing* as a
comprehension task — not “make a summary” as a revision strategy.

Use a **one-page synthesis** with selection rules: one core claim, five must-know
items, one comparison, one example, three traps. Then add recall prompts. The
synthesis is the map; the prompts are the workout.

## 6. Typography and layout: separate evidence from standards

| Claim | Status | Source | Rule |
|---|---|---|---|
| 45–90 characters is comfortable | moderate | Dyson & Haselgrove 2001; Bringhurst; Butterick | 55–80 characters |
| > 80 characters is an accessibility barrier | standard | WCAG 2.2 SC 1.4.8 (AAA) | ≤ 80; 55–75 preferred |
| Justified text creates uneven spacing | standard/mixed | WCAG 1.4.8 | Left-align; if justified, hyphenate |
| Running text < 9 pt is a barrier | standard | Section 508 | Tables ≥ 9 pt; refs ≥ 8 pt |
| Clear print starts at 12 pt | standard | UKAAF | Accessible variant 12–14 pt |
| Contrast ≥ 4.5:1 normal text | standard | WCAG 1.4.3 | Check every pair |
| Colour not sole carrier | standard | WCAG 1.4.1 | Add label/shape/text |
| White space aids scannability | heuristic/standard | Plain-language practice | 1.5–2× paragraph spacing |
| Headings/signals reduce search cost | moderate | Signaling; Noetel 2022 | 3 levels; descriptive labels |
| Few-column tables are easier | heuristic/standard | Table-design practice | ≤ 6 columns, ≤ 10 rows |
| Captions state the takeaway | standard/moderate | Contiguity; APA/Chicago | What to notice + source |

Two honesty notes:

- **WCAG 2.2 is a web standard.** 1.4.12 Text Spacing and 1.4.10 Reflow are
  web-oriented and are not a good fit for a fixed-layout PDF. WCAG is a useful
  benchmark; PDF/UA is the PDF-specific standard.
- **PDF/UA is structural, not typographic.** ISO 14289 covers tags, reading
  order, alt text, table structure and navigation — not font size, line length or
  contrast. A PDF can be PDF/UA-conformant and still be hard to read.

Concrete rules:

- Body 10.5–11.5 pt, 1.4–1.5 line height, left-aligned; measure 55–80 chars.
- Margins ≥ 18 mm; single-column A4 needs 28–32 mm side margins or two columns.
- Tables 9–10 pt, ≤ 6 columns, ≤ 10 rows, header row, units in header.
- Captions 8.5–9 pt; source refs 8 pt minimum; contrast ≥ 4.5:1.
- Three colours + ink; colour plus label/shape; no colour-only meaning.
- Horizontal rules only in tables; no heavy fills, shadows or decorative icons.
- Short uppercase labels only; never all-caps body text.
- Figure: label axes/units/nodes/edges directly; caption below with takeaway,
  source and alt text.

## 7. Recommended document grammar

This is a genre proposal built from the techniques above. The sequence is a
**heuristic** to test, not a validated format.

### 7.1 Unit hierarchy

```
Course
├── How to use this guide (1 page)
├── Course map (1 page)
├── Chapter units (2–4 pages each)
│   ├── Unit opener + chapter map
│   ├── Must-know definitions
│   ├── Comparison matrix (1–3)
│   ├── Worked example (1–2)
│   ├── Exam traps (3–6)
│   ├── Recall prompts (4–8) + answer key
│   └── One-page synthesis
├── Master reference tables (1–3 pages)
├── Answer key (1–2 pages)
└── Coverage and source note (1 page)
```

### 7.2 Page recipe (typical 4-page chapter)

| Page | Content | Rule |
|---|---|---|
| 1 | Opener: big question, 3 outcomes, source range, prerequisite | ≤ 80 words; no motivation prose |
| 1 | Chapter map: 8–12 nodes, labeled edges | One caption with takeaway |
| 2 | Must-know definitions: 5–8 | Term + 1 sentence + example + source |
| 2 | Comparison matrix #1 | ≤ 6 columns; takeaway sentence |
| 3 | Worked example: full + faded | Annotate decisions; variation at end |
| 3 | Exam traps: 3–6 | “If you see X → do Y → because Z” |
| 4 | Recall prompts: 4–8 | Answers at chapter end; source refs |
| 4 | One-page synthesis: 5 bullets + 2 comparisons + 3 traps | No new content |

Small chapter → 2 pages: map + definitions; matrix + one example + traps +
prompts. Dense chapter → 6 pages, split by source section, never by arbitrary
page count.

### 7.3 Component rules

**Definitions (5–8).** One precise sentence, one concrete example, one source.
English term in parentheses on first use in a Turkish guide. No duplicate
glossary entries; cross-reference. Distinguish near-synonyms in a matrix.

**Matrix (1–3).** Rows = cases, columns = dimensions; 3–6 columns, 4–8 rows. Use
for “same/different”, “when to use which”, “before/after”. Not for processes. End
with “Choose A when X; choose B when Y.”

**Worked example (1–2).** Problem → plan → steps → answer → why → variation.
Annotate decision points. Second example is faded: one step removed. For code:
input, trace, output, complexity, one edge case. For non-procedural domains:
claim → evidence → inference → limitation.

**Exam traps (3–6).** “If you see X → do Y → because Z.” A trap is a
misconception, not a trick; no guessing strategies. A critical distinction
explains a boundary; a trap explains a common error. Label added inference
**Açıklayıcı not**.

**Recall prompts (4–8).** Specific, answerable in 1–2 sentences. Mix define,
compare, explain, predict, apply. Answers in a separate key; “attempt first, then
check”. Interleave one prompt from a previous chapter.

**One-page synthesis (1).** Top: one sentence (≤ 25 words). Left: 5 must-know
bullets (≤ 2 lines). Right: one small matrix/map. Bottom: 3 traps + 3 prompts.
No new content, no duplicate definitions.

**Review schedule.** 24 h: recall prompts, book closed. 3 d: reconstruct the
matrix/map. 7 d: redo the faded example + one mixed problem. Optional 14 d:
cumulative mixed set. Label as a prompt, not a guarantee.

### 7.4 Bullet and paragraph rules (house style)

- One idea per bullet; max 25 words and max 2 lines; max 7 bullets per list.
- No nested bullets deeper than one level; use a table or split instead.
- Paragraphs: max 3 sentences or 60 words; one claim per paragraph.
- Claim → evidence → consequence. Number the steps in a process.
- If a bullet needs “because”, use a short paragraph or a claim/reason table.

### 7.5 Page and visual budget (observational, not fixed)

Built guides in `output/` are 20–23 pages: PD0–PD1 22 pages for two decks, PD2
23 pages for 103 slides, PD3 20 pages. These are observations, not norms.

| Output | Typical range | Notes |
|---|---:|---|
| Revision sheet, one short source (~40 slides) | 4–8 pages | proposed; validate |
| Revision sheet, one dense source (~100 slides) | 6–12 pages | proposed; validate |
| Long-form reader, one dense source (~100 slides) | 16–24 pages | observed 20–23; not a revision sheet |
| Long-form course guide (6–10 chapters) | 16–28 pages | includes front matter + key |
| Two-source long-form pilot | 20–24 pages | observed 22 |
| One-page synthesis | 1 per chapter | not per course |
| Master tables / answer key | 1–3 / 1–2 pages | formulas, units, answers |

Length is an output, not a target. Under range → add matrices/examples/prompts,
not prose. Over range → cut duplicates, decoration and verbatim source; never
shrink below the type floor. If a chapter cannot fit, split it.

**Visual budget:** one concept map per chapter (8–12 nodes); 0–2 mechanism
diagrams per chapter; 1–3 matrices; at most one “hero” visual per page; no
decorative images. If removing a visual changes nothing, cut it.

### 7.6 Answer-key placement

The evidence favours retrieval, but a PDF cannot hide answers reliably.
Recommended default: **question page + answer key at the end of the chapter**,
with a clear marker (`Cevaplar: s. 14`). For screen use, a separate answer file is
better; for print, a fold-under flap or cover sheet works. Never put the answer
on the same line as the question and call it retrieval.

## 8. What not to do

| Anti-pattern | Why it fails | Better move |
|---|---|---|
| Long textbook paragraphs | Load; hides structure | Claim-first paragraph; table/bullets |
| Decorative images/stock photos | Coherence cost; no function | Remove; mechanism diagram only |
| Duplicate definitions | Redundancy; drift | One canonical entry; cross-reference |
| Answer-free quizzes | No feedback; can encode errors | Answer key + source; low stakes |
| Colour-only meaning | Fails colour-blind/print users | Label + shape + text |
| Tiny table type (7–8 pt) | Below accessibility guidance | 9–10 pt; split table |
| Justified text without hyphenation | Uneven spacing; rivers | Left-align; hyphenate if justified |
| All-caps body text | Slower reading; barrier | Short labels only |
| Learning-style matching | Not supported | Match representation to content |
| Highlighting/rereading as main method | Low utility (Dunlosky) | Retrieval + spacing |
| Verbatim slide bullets | Capture without mechanism | Cause → effect; source refs |
| Page-count padding | Noise; length ≠ learning | Content budget; cut/split |
| Overloaded maps (> 12 nodes) | Illegible; no hierarchy | Split overview/detail |
| > 6 columns / rotated headers | Hard to scan; breaks print | Split by dimension; appendix |
| “Figure 1” captions | No takeaway; no access | Label axes; state what to notice |
| Generic study advice | Wastes space | Content-specific prompt |
| “Scientifically proven” | Overstates evidence | “Designed around evidence-based principles” |

## 9. Conflicts with current `paper` / `reader` styles

The styles are good **long-form readers**, not revision-sheet styles. The conflict
is about tradeoffs, not “wrong” design.

| Area | Current implementation | Evidence/standard | Severity | Recommendation |
|---|---|---|---|---|
| Genre | Continuous readers; no matrix/map/trap/synthesis grammar | Revision needs segment components | High | Add `revision` style/structure; keep readers long-form |
| Measure | A4 minus 16.5–17 mm at 9.5–10 pt → body-size lines median 92–105 chars, p90 105–112 | 45–90 comfortable; WCAG AAA ≤ 80 | High | 28–30 mm margins or two columns |
| Justification | `justify: true`, `hyphenate: false` | WCAG 1.4.8 AAA: not justified | High | Left-align; if justified, hyphenate + narrow measure |
| Table type | Table text 6.9–7.9 pt; captions 7.9; refs 6.9; code 7.4 | Section 508: < 9 pt barrier; UKAAF 12 pt | High | Tables ≥ 9; captions ≥ 8.5; refs ≥ 8; code ≥ 8 |
| Contrast | `faint` #8A7A63 on #FAF6EC = 3.86:1 at 6.9 pt; gold #B08A4F = 2.95:1 as text | WCAG 1.4.3 AA 4.5:1 | High | Darken faint to ≥ 4.5 (e.g. #75664E); gold only for rules/fills |
| Body type | 9.5 pt `paper`; 10 pt `reader` | 11–12 pt typical; UKAAF 12 pt clear print | Medium | 10.5–11.5 revision; 12 accessible |
| Density | `reader`: 10 pt, 0.70 em leading, 2.6 mm paragraph gap | Comfortable but low density | Medium | Keep 10.5; paragraph gap 1.8–2.2; density via tables/boxes |
| Components | `recall`/`explain`/`review_plan` alias `definition`/`note` | Retrieval/spacing need distinct semantics | Medium | `recall` = question/answer/source; `review-plan` = table |
| Policy | `authoring-spec.md` forbids quizzes/practice questions | Evidence recommends optional recall; renderer supports it | High | Allow optional prompts + key; ban quiz-book padding |
| Page targets | `guides/*/config.json` still has `target_pages` | v0.9 policy: `page_range`; length is output | Medium | Replace with `page_range` + content budget |
| Accessibility | `accessible` 12 pt but Noto Serif + inherited justification | UKAAF: sans-serif, left-aligned, 12–14 pt | Medium | Test sans/left; add PDF/UA tag check |
| Tables | No vertical lines; dark top/bottom; light header | Consistent with table design | — | Keep |
| Palette | Cream, sepia, brick/sage/gold | Good hierarchy; colour not sole carrier if labelled | — | Keep; fix faint/gold |

Component gaps: no `map` semantic (D2 graph is not a concept map); no `matrix`
(row/column roles, takeaway, limits); no `trap` (`critical` is a distinction, not
a common error); no synthesis page (a compact chapter + `synthesis` block would
do); no answer-key mode (`recall` renders as a definition box).

Keep: palette/paper metaphor, restrained tables, source refs, captions, coverage
audit. `paper`/`reader` remain right for long-form course readers; do not retrofit
them into revision sheets.

## 10. Evaluation plan

A revision format is a hypothesis until measured. Small study: 5–15 target
students; revision sheet vs. source slides vs. plain summary; immediate
comprehension quiz; delayed recall at 2–7 days; time-to-answer; NASA-TLX;
preference interview. Layout sub-studies: one vs. two columns; 9 vs. 10 pt tables;
answer key at chapter end vs. separate file; map vs. outline. Report means, effect
sizes and confidence intervals; report null results. Measure **time-on-task**:
maps, matrices and prompts all take longer than reading, and may look effective
simply because they make students work more.

## 11. Implementation checklist

- [ ] Add `revision` style: single-column 28–30 mm margins (or two-column),
      10.5–11 pt, ragged-right, 9–10 pt tables, 8.5–9 pt captions, 8 pt refs,
      high-contrast faint.
- [ ] Add `map`, `matrix`, `trap`, `recall` (question/answer/source),
      `synthesis`, `review-plan` (table).
- [ ] Schema limits: 8–12 map nodes; ≤ 6 table columns; ≤ 10 rows; 1 map +
      0–2 diagrams + 1–3 matrices per chapter.
- [ ] Allow optional recall prompts with an answer key; keep the ban on quiz-book
      padding and generic advice.
- [ ] Replace `target_pages` with `page_range` + content budget.
- [ ] Add layout gates: characters/line, min type size, contrast, table
      dimensions, answer-key presence.
- [ ] Add PDF/UA checks if accessibility is claimed: tags, reading order, alt
      text, table headers, language, bookmarks.
- [ ] Run the §10 evaluation before any product-level claim.

## References

- Adesope, O. O., Trevisan, D. A., & Sundararajan, N. (2017). Rethinking the use of tests. *Review of Educational Research, 87*(3), 659–701. <https://doi.org/10.3102/0034654316689306>
- Barbieri, C. A., Miller-Cotto, D., Clerjuste, S. N., & Chawla, K. (2023). A meta-analysis of the worked examples effect on mathematics performance. *Educational Psychology Review, 35*, 11. <https://doi.org/10.1007/s10648-023-09745-1>
- Bringhurst, R. (2013). *The elements of typographic style* (4th ed.). Hartley & Marks.
- Brown, A. L., & Day, J. D. (1983). Macrorules for summarizing texts. *Journal of Verbal Learning and Verbal Behavior, 22*(1), 1–14. <https://www.ideals.illinois.edu/items/17782>
- Brunmair, M., & Richter, T. (2019). Similarity matters: A meta-analysis of interleaved learning. *Psychological Bulletin, 145*(11), 1029–1052. <https://doi.org/10.1037/bul0000209>
- Butler, A. C., & Roediger, H. L. (2008). Feedback enhances the positive effects of multiple-choice testing. *Memory & Cognition, 36*(3), 604–616. <https://doi.org/10.3758/mc.36.3.604>
- Butterick, M. *Practical typography: Line length*. <https://practicaltypography.com/line-length.html>
- Cepeda, N. J., Pashler, H., Vul, E., Wixted, J. T., & Rohrer, D. (2006). Distributed practice in verbal recall tasks. *Psychological Bulletin, 132*(3), 354–380. <https://doi.org/10.1037/0033-2909.132.3.354>
- Cepeda, N. J., Vul, E., Rohrer, D., Wixted, J. T., & Pashler, H. (2008). Spacing effects in learning. *Psychological Science, 19*(11), 1095–1102. <https://doi.org/10.1111/j.1467-9280.2008.02209.x>
- Chi, M. T. H., De Leeuw, N., Chiu, M.-H., & LaVancher, C. (1994). Eliciting self-explanations improves understanding. *Cognitive Science, 18*(3), 439–477. <https://doi.org/10.1207/s15516709cog1803_3>
- Cornell University Learning Strategies Center. *The Cornell note-taking system*. <https://lsc.cornell.edu/how-to-study/taking-notes/cornell-note-taking-system/>
- Dunlosky, J., Rawson, K. A., Marsh, E. J., Nathan, M. J., & Willingham, D. T. (2013). Improving students’ learning with effective learning techniques. *Psychological Science in the Public Interest, 14*(1), 4–58. <https://doi.org/10.1177/1529100612453266>
- Dyson, M. C., & Haselgrove, M. (2001). The influence of reading speed and line length on reading from screen. *International Journal of Human-Computer Studies, 54*(4), 585–612. <https://doi.org/10.1006/ijhc.2001.0458>
- Graham, S., & Hebert, M. (2011). Writing to read. *Harvard Educational Review, 81*(4), 710–744. <https://doi.org/10.17763/haer.81.4.t2k0m13756113566>
- ISO. (2014). *ISO 14289-1:2014 — PDF/UA-1*. <https://www.iso.org/standard/64599.html>
- ISO. (2024). *ISO 14289-2:2024 — PDF/UA-2*. <https://www.iso.org/standard/82278.html>
- Kalyuga, S., Ayres, P., Chandler, P., & Sweller, J. (2003). The expertise reversal effect. *Educational Psychologist, 38*(1), 23–31. <https://doi.org/10.1207/S15326985EP3801_4>
- Kiewra, K. A., DuBois, N. F., Christian, D., McShane, A., Meyerhoffer, M., & Roskelley, D. (1991). Note-taking functions and techniques. *Journal of Educational Psychology, 83*(2), 240–245. <https://doi.org/10.1037/0022-0663.83.2.240>
- Kobayashi, K. (2006). Combined effects of note-taking/-reviewing on learning. *Educational Psychology, 26*(3), 459–477. <https://doi.org/10.1080/01443410500342070>
- Konrad, M., Joseph, L. M., & Eveleigh, E. (2009). A meta-analytic review of guided notes. *Education and Treatment of Children, 32*(3), 421–444. <https://doi.org/10.1353/etc.0.0066>
- Morehead, K., Dunlosky, J., & Rawson, K. A. (2019). Note-taking habits of 21st century college students. *Memory, 27*(6), 770–780. <https://doi.org/10.1080/09658211.2019.1569694>
- Nesbit, J. C., & Adesope, O. O. (2006). Learning with concept and knowledge maps. *Review of Educational Research, 76*(3), 413–448. <https://doi.org/10.3102/00346543076003413>
- Noetel, M., Griffith, S., Delaney, O., Sanders, T., Parker, P., del Pozo Cruz, B., & Lonsdale, C. (2022). Multimedia design for learning. *Review of Educational Research, 92*(5), 733–771. <https://doi.org/10.3102/00346543211052329>
- Pan, S. C., & Rickard, T. C. (2018). Transfer of test-enhanced learning. *Psychological Bulletin, 144*(7), 710–756. <https://doi.org/10.1037/bul0000151>
- Pashler, H., McDaniel, M., Rohrer, D., & Bjork, R. (2008). Learning styles: Concepts and evidence. *Psychological Science in the Public Interest, 9*(3), 105–119. <https://doi.org/10.1111/j.1539-6053.2009.01038.x>
- Renkl, A., & Atkinson, R. K. (2003). Structuring the transition from example study to problem solving. *Educational Psychologist, 38*(1), 15–22. <https://doi.org/10.1207/S15326985EP3801_3>
- Rittle-Johnson, B., Loehr, A. M., & Durkin, K. (2017). Promoting self-explanation. *Psychonomic Bulletin & Review, 24*, 1073–1084. <https://doi.org/10.3758/s13423-016-1079-5>
- Roediger, H. L., & Karpicke, J. D. (2006). Test-enhanced learning. *Psychological Science, 17*(3), 249–255. <https://doi.org/10.1111/j.1467-9280.2006.01693.x>
- Rohrer, D., & Taylor, K. (2007). The shuffling of mathematics problems improves learning. *Instructional Science, 35*, 481–498. <https://doi.org/10.1007/s11251-007-9015-8>
- Rowland, C. A. (2014). The effect of testing versus restudy on retention. *Psychological Bulletin, 140*(6), 1432–1463. <https://doi.org/10.1037/a0037559>
- Schroeder, N. L., Nesbit, J. C., Anguiano, C. J., & Adesope, O. O. (2018). Studying and constructing concept maps. *Educational Psychology Review, 30*, 431–455. <https://doi.org/10.1007/s10648-017-9403-9>
- Schwartz, D. L., & Bransford, J. D. (1998). A time for telling. *Cognition and Instruction, 16*(4), 475–522. <https://doi.org/10.1207/s1532690xci1604_4>
- Section508.gov. *Understanding accessible fonts and typography*. <https://www.section508.gov/develop/fonts-typography/>
- Sweller, J., & Cooper, G. A. (1985). The use of worked examples as a substitute for problem solving in learning algebra. *Cognition and Instruction, 2*(1), 59–89. <https://doi.org/10.1207/s1532690xci0201_3>
- Sweller, J., van Merriënboer, J. J. G., & Paas, F. (2019). Cognitive architecture and instructional design: 20 years later. *Educational Psychology Review, 31*, 261–292. <https://doi.org/10.1007/s10648-019-09465-5>
- UK Association for Accessible Formats. (2024). *Creating clear print and large print documents* (G003, v4). <https://www.ukaaf.org/wp-content/uploads/2024/12/G003-UKAAF-Creating-clear-print-and-large-print-documents-v4.pdf>
- W3C. (2023). *Web Content Accessibility Guidelines (WCAG) 2.2*. <https://www.w3.org/TR/WCAG22/>
- W3C WAI. (2026). *Understanding SC 1.4.8: Visual presentation*. <https://www.w3.org/WAI/WCAG22/Understanding/visual-presentation.html>
