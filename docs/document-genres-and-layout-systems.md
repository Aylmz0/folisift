# Document genres and layout systems

- **Status:** Research note for the genre-first revision of Folisift
- **Date:** 2026-10-30
- **Scope:** why `paper`, `reader`, `journal`, `brief` and `accessible` are not
  different document genres; what the actual genres are; what each genre's
  layout logic, content grammar and evidence base should be
- **Related:** [`revision-format-research.md`](revision-format-research.md),
  [`authoring-evidence.md`](authoring-evidence.md),
  [`plans/2026-10-30-revision-format-and-style-taxonomy.md`](plans/2026-10-30-revision-format-and-style-taxonomy.md)
- **Claim policy:** evidence-informed, not “scientifically proven”. Genre
  conventions are professional/social standards; only some components have
  experimental learning evidence.

## 1. The diagnosis

The current Folisift “styles” are **typographic presets**, not genres:

| Pack | What actually changes | What does not change |
|---|---|---|
| `paper` → `reader` | body 9.5→10 pt, margins 16.5→17 mm, leading 0.62→0.70 em, spacing 1.8/3.2→2.6/4.0 mm | layout, palette, fonts, cover, structure, component grammar |
| `paper` → `journal` | body 10.5 pt, margins 18 mm | same paper layout and components; no abstract, keywords or journal-specific moves |
| `clean` → `brief` | body 10.5 pt, margins 17 mm | same clean layout; no BLUF box, decision table or risk register |
| `paper` → `accessible` | body 12 pt, margins 20 mm | same component sizes; tables and labels remain 6.9–7.9 pt |

That is a **density/accessibility difference**, not a different information
architecture. A student revising for an exam and a researcher reading an IMRAD
paper do not need the same page logic with a different body size. They need
different **genres**.

Genre theory is explicit about this: genres are typified communicative
responses to recurring situations, defined by purpose, audience and community
conventions (Swales, 1990; Hyland, 2004). A genre is not a font. A revision
sheet, a knowledge organiser, a guided-notes page, a workbook, a formula sheet,
an academic paper and an executive brief have different purposes, different
reader tasks and therefore different layout logic.

## 2. Three layers, not one

| Layer | Question | Examples |
|---|---|---|
| **Genre** | What is this document *for*, and who is the reader? | revision sheet, reader, knowledge organiser, guided notes, workbook, reference sheet, academic paper, brief, clinical/legal note |
| **Structure** | What content units and sequence does the genre require? | map → definitions → matrix → example → traps → recall → synthesis; IMRAD; IRAC; SOAP; BLUF |
| **Style** | How is it dressed? | body size, leading, margins, palette, font, density, accessibility |

The correct dependency is:

```text
purpose / audience
      ↓
genre  →  structure + layout logic  →  typography / palette / density
```

Folisift currently does:

```text
style pack  →  typography / palette  →  same layout for every purpose
```

That is why the names feel like font sizes. The fix is not more token skins; it
is a **genre layer**.

## 3. What the psychology evidence says about visual, segmented documents

The revision-format note covers retrieval, spacing, concept maps, worked
examples and summarization in detail. This note adds the visual/document-design
evidence.

| Principle | Evidence | Strength | Design consequence | Caveat |
|---|---|---|---|---|
| Dual coding | Words + pictures can be processed in verbal and imaginal channels (Paivio, 1986); multimedia effect (Mayer, 2021) | strong for principles | Pair a diagram with a short verbal takeaway; label directly | “Dual coding” is not “add a decorative picture” |
| Coherence | Removing irrelevant words/pictures improves transfer (Mayer’s median 1.11 in his programme) | moderate/strong | No stock photos, no decorative icons, no “seductive details” | Lab effects; do not overgeneralise |
| Signalling | Headings, labels, colour cues and arrows guide attention (Mayer median 0.60) | moderate | Use a small number of consistent signals; label what matters | Too many colours/signals become noise |
| Spatial contiguity | Related words and pictures should be near each other (Mayer median 1.12) | moderate/strong | Caption below the figure; label nodes/axes directly; no split attention | Static PDF cannot use narration/temporal contiguity |
| Segmenting | Chunk complex material into learner-paced units | moderate | One concept per block; 2–4 page units; no wall of text | Chunking can fragment causal chains if overdone |
| Cognitive load | Extraneous load competes with schema construction (Sweller, 1988) | strong | Remove duplicate text, decoration and split attention; use worked examples for novices | Expertise reversal: advanced learners need less scaffolding |
| Generative drawing | Drawing to-be-learned content improves memory and comprehension; drawing effect is robust for free recall (Wammes et al., 2016); learning-by-drawing has boundary conditions (Fiorella & Zhang, 2018) | moderate | Give blank/partial visual templates for the learner to complete | A pre-made drawing is not the same as learner-generated drawing |
| Graphic organizers | Learner-generated organizers: g≈0.59 memory, g≈0.63 comprehension; instructor-provided: g≈0.70 memory, g≈0.53 comprehension (2025 meta-analysis) | moderate | Use matrices, timelines, flowcharts and partial maps; require completion | Heterogeneous; organizer type and instruction matter |
| Concept maps | g≈0.58 overall; constructing 0.72 vs studying 0.43 (Schroeder et al., 2018) | moderate | One map per unit; labelled edges; “redraw/fill” prompt | A finished map alone is weaker than constructing one |
| ICAP | Interactive > constructive > active > passive engagement (Chi & Wylie, 2014) | moderate | Ask the learner to complete, compare, explain, trace or answer | A beautifully designed but passive page is still passive |
| Learning styles | No evidence for matching instruction to “visual/auditory” learners (Pashler et al., 2008) | strong negative | Do not sell “visual learner” personalisation | Use visuals for the content, not the person |

### 3.1 “No long paragraphs” is right for revision, but not a universal law

The user’s target — graphic-heavy, bullet/segment-oriented, table-rich,
concept-map-oriented, short — is a good **revision-sheet** grammar. But “no
paragraphs at all” is too strong as a universal rule.

- **Bullets** are good for enumerations, features, steps and scan-and-check
  lists. They reduce search cost and support segmenting.
- **Tables** are good for contrasts, parameters and decisions. They make
  dimensions explicit.
- **Diagrams** are good for structure, process and mechanism. They make
  relations visible.
- **Micro-prose (1–3 sentences)** is still needed for causal chains, arguments,
  assumptions and “why this works”. A bullet list can fragment a mechanism into
  disconnected claims.
- **Long prose** is appropriate in a reader/academic genre, not in a revision
  sheet.

So the rule should be **“no long paragraphs”**, not **“no paragraphs”**. In a
revision sheet, a paragraph is a 1–3 sentence micro-explanation between visual
blocks; it is not a 300-word textbook section.

## 4. Genre profiles

Each profile states the reader task, the content unit, the layout logic, the
visual grammar and the evidence. “Standard” means a professional/genre
convention, not a learning effect.

### 4.1 Long-form reader

| Dimension | Specification |
|---|---|
| Purpose | Learn a subject over time; build a coherent model; read continuously |
| Reader task | Read, reflect, annotate, return |
| Unit | Chapter/section, 4–12 pages |
| Layout | Single column, 55–75 characters, section openers, running header, TOC |
| Content | Prose with micro-paragraphs, definitions, examples, figures, tables, chapter summary |
| Visual grammar | Explanatory figures, maps, timelines; captions with takeaway; no decoration |
| Evidence | Signalling, coherence, dual coding; genre convention |
| Avoid | Bullet-only fragmentation; slide dump; decorative visuals |
| Folisift | `paper` / `reader` + `a-g` or `imrad`; typography only is not enough to call it a genre |

### 4.2 Revision sheet (visual study guide)

| Dimension | Specification |
|---|---|
| Purpose | Prepare for an exam in a short, focused session |
| Reader task | Scan, recall, compare, practise, self-check |
| Unit | Source/part, 2–4 pages; whole sheet 8–12 pages for a 100-slide deck |
| Layout | Modular grid; full-width maps/tables; 2-column definition cards; recall block; answer key |
| Content | Map → must-know → comparisons → worked example → traps → recall → one-page synthesis → review plan |
| Visual grammar | Concept map, comparison matrix, process diagram, timeline, annotated example |
| Evidence | Retrieval, spacing, concept maps, worked examples, graphic organizers; summarization only as consolidation |
| Avoid | Long paragraphs, slide dump, decorative mind maps, answer-free quizzes, tiny tables |
| Folisift | new `revision` layout + `revision-sheet` structure |

### 4.3 Knowledge organiser (one-pager)

| Dimension | Specification |
|---|---|
| Purpose | Reference/display one unit's essential knowledge on a single page |
| Reader task | Look up, self-quiz, connect vocabulary and big ideas |
| Unit | One page per topic; 10–15 terms; 1 timeline/map/diagram |
| Layout | Quadrant/grid: vocabulary, timeline, key people/terms, big questions, diagram |
| Content | Definitions, dates, places, formulas, core distinctions; no worked examples (house standard, not an empirical constant) |
| Visual grammar | One dominant map/timeline; labelled; colour-coded by category (with text labels) |
| Evidence | Graphic organizers, concept maps, coherence; practitioner genre (Chartered College, 2018) |
| Avoid | Cramming a whole chapter; tiny type; unlabelled icons; “all facts” |
| Folisift | `revision` layout + `knowledge-organiser` structure (one page per unit); not a replacement for a revision sheet |

### 4.4 Guided notes / lecture notes

| Dimension | Specification |
|---|---|
| Purpose | Accompany a lecture/reading; capture and review |
| Reader task | Fill blanks, add cues, summarise, recite |
| Unit | Lecture/topic, 2–6 pages |
| Layout | Cornell-like: cue column (25–30%), notes column (55–65%), summary band (15–20%) |
| Content | Provided framework, blanks, key terms, cues, summary |
| Visual grammar | Simple diagrams/timelines with blank labels; no finished answers |
| Evidence | Guided notes moderate (d≈0.55); note-taking mixed; review matters |
| Avoid | Fully completed notes; passive fill-in; no review cue |
| Folisift | `notes` layout + `lecture-notes` structure |

### 4.5 Workbook / problem set

| Dimension | Specification |
|---|---|
| Purpose | Practise procedures and retrieval; reveal misconceptions |
| Reader task | Solve, check, reflect, redo |
| Unit | Problem set, 2–8 pages |
| Layout | Problem block → workspace → solution key; faded examples |
| Content | Worked example, faded example, independent problems, mixed review, answers |
| Visual grammar | Step diagrams, traces, tables of cases; space for handwriting |
| Evidence | Worked examples, retrieval, self-explanation, ICAP |
| Avoid | Solutions on the same page; no variation; only easy problems |
| Folisift | `workbook` layout + `workbook` structure |

### 4.6 Formula / reference sheet

| Dimension | Specification |
|---|---|
| Purpose | Fast lookup during practice or exam (where permitted) |
| Reader task | Find, verify, apply |
| Unit | Topic/course, 2–8 pages |
| Layout | Multi-column tables/cards; units; symbols; conditions |
| Content | Formulas, definitions, units, assumptions, common errors |
| Visual grammar | Formula cards, parameter tables, small diagrams for geometry/process |
| Evidence | Information design/typography; not a first-learning genre |
| Avoid | Derivation prose; unexplained symbols; missing units/conditions |
| Folisift | `reference` layout + `formula-sheet` structure |

### 4.7 Academic paper / report

| Dimension | Specification |
|---|---|
| Purpose | Make an argument, report research, join a discourse community |
| Reader task | Evaluate claims, methods, evidence, contribution |
| Unit | Article/report, 8–40+ pages |
| Layout | IMRAD or disciplinary structure; abstract; citations; figures/tables |
| Content | Claim → evidence → method → analysis → limitation; prose |
| Visual grammar | Data figures, tables, diagrams; captions with source and interpretation |
| Evidence | Genre convention (Swales, Hyland); not a study technique |
| Avoid | Bullet-only “executive summary” masquerading as a paper; unsourced claims |
| Folisift | `paper` / `journal` + `imrad` or `literature-matrix`; `journal` is a preset, not a full journal template |

### 4.8 Executive brief / decision memo

| Dimension | Specification |
|---|---|
| Purpose | Support a decision quickly |
| Reader task | Understand the recommendation, risks and next action |
| Unit | 1–4 pages |
| Layout | BLUF/answer-first; findings; recommendation; risks; owner/deadline |
| Content | One governing thought, 2–4 supporting arguments, evidence, caveats |
| Visual grammar | Decision matrix, risk table, one chart; no decorative imagery |
| Evidence | Professional standard (Minto Pyramid/BLUF); not learning science |
| Avoid | Chronological narrative; buried recommendation; no owner |
| Folisift | `clean` / `brief` + `executive-brief`; `brief` is a preset, not a full memo system |

### 4.9 Clinical / legal note

| Dimension | Specification |
|---|---|
| Purpose | Record a professional assessment/decision |
| Reader task | Follow facts, rules, assessment, plan; audit |
| Unit | Case/encounter, 1–3 pages |
| Layout | SOAP or IRAC fields; structured headings; evidence and uncertainty |
| Content | Subjective/objective/assessment/plan or issue/rule/application/conclusion |
| Visual grammar | Timeline, decision tree, red-flag table; minimal decoration |
| Evidence | Professional genre conventions; not exam revision |
| Avoid | Missing jurisdiction/uncertainty; invented facts; diagnosis without evidence |
| Folisift | `paper`/`clean` + `soap`/`irac`; `clinical`/`casebook` remain future |

### 4.10 Visual summary / sketchnote

| Dimension | Specification |
|---|---|
| Purpose | Personal visual encoding of a topic |
| Reader task | Draw, connect, recall |
| Unit | One page per topic |
| Layout | Free-form canvas with a clear hierarchy; 1 big idea, 5–9 branches |
| Content | Key words, icons, arrows, one example, one question |
| Visual grammar | Hand-drawn or clean vector; labels; no decorative noise |
| Evidence | Drawing effect, dual coding, generative learning; weak as a pre-made document |
| Avoid | Pre-made “sketchnote” that the learner only reads; artistic skill as a barrier |
| Folisift | provide a **blank/partial template** for learner completion, not a finished sketchnote |

## 5. Decision matrix

| User task | Genre | Folisift layout + structure | Do not use |
|---|---|---|---|
| Learn a course over weeks | Long-form reader | `paper`/`reader` + `a-g` | `brief`, one-pager |
| Revise before an exam | Revision sheet | `revision` + `revision-sheet` | `reader` with smaller font |
| Display a unit's core facts | Knowledge organiser | `revision` + `knowledge-organiser` | full revision sheet |
| Take notes in class | Guided notes | `notes` + `lecture-notes` | finished reader |
| Practise problems | Workbook | `workbook` + `workbook` | revision sheet without workspace |
| Look up formulas | Reference sheet | `reference` + `formula-sheet` | reader |
| Write a paper/report | Academic paper | `paper`/`journal` + `imrad` | revision sheet |
| Decide in a meeting | Executive brief | `clean`/`brief` + `executive-brief` | academic paper |
| Record a case | Clinical/legal note | `paper`/`clean` + `soap`/`irac` | revision sheet |
| Encode personally | Visual summary | blank/partial template | pre-made sketchnote |

## 6. Standards and sources by layer

### 6.1 Psychological / learning science

- Retrieval practice, distributed practice, interleaving: Dunlosky et al.
  (2013); Adesope et al. (2017); Rowland (2014); Cepeda et al. (2006);
  Brunmair & Richter (2019).
- Worked examples and expertise reversal: Sweller & Cooper (1985); Kalyuga et
  al. (2003); Barbieri et al. (2023).
- Concept maps and graphic organizers: Nesbit & Adesope (2006); Schroeder et
  al. (2018); 2025 graphic-organizer meta-analysis.
- Generative drawing: Wammes et al. (2016); Fiorella & Zhang (2018).
- Multimedia, coherence, signalling, contiguity: Mayer (2021); Noetel et al.
  (2022).
- Cognitive load: Sweller (1988); Sweller et al. (2019).
- Engagement: Chi & Wylie (2014).
- Learning styles are not supported: Pashler et al. (2008).

### 6.2 Academic genre and writing

- Genre as purpose/audience/community: Swales (1990); Hyland (2004, 2007).
- IMRAD: standard scientific report structure.
- IRAC/CRAC: legal analysis.
- SOAP: clinical note structure.
- Literature matrix / annotated bibliography: research synthesis.
- Academic phraseology and move analysis: Swales & Feak (2012).

### 6.3 Professional information design

- Document design and audience modeling: Schriver (1997).
- Information-design principles: structure, clarity, simplicity, emphasis,
  unity, harmony (Pettersson, 2018).
- Answer-first/BLUF: Minto (2009).
- Typography: Bringhurst (2012); Butterick’s practical typography.
- Plain language: federal plain-language guidelines.

### 6.4 Accessibility and typography standards

- WCAG 2.2: contrast 4.5:1, 80-character measure (AAA), non-justified text
  (AAA), text-spacing resilience (AA).
- PDF/UA-1 (ISO 14289-1): tags, reading order, alt text, table structure,
  language metadata.
- Section 508: running text below 9 pt is a barrier; typical body 11–12 pt.
- UKAAF clear print: 12 pt+ for accessible print; sans-serif; left-aligned;
  1.5 line spacing.
- BDA dyslexia style guide: 12–14 pt, 1.5 spacing, left-aligned, sans-serif.

## 7. What this means for Folisift

**Implementation status (v0.10):** the reader layout, revision-sheet genre and
one-page knowledge organiser are implemented. Guided notes, workbook and
reference sheet are specified for v0.11. `accessible` is a modifier with a
`component_scale` that now scales the paper components; `paper` remains the
legacy parity layout.

1. **Stop calling typographic presets genres.** `paper`, `reader`, `clean`,
   `journal`, `brief` and `accessible` are layouts/presets/modifiers.
2. **Add a genre layer.** At minimum: reader, revision sheet, knowledge
   organiser, guided notes, workbook, reference sheet, academic paper, brief,
   clinical/legal note.
3. **Give each genre its own gate.** A revision sheet is not “a reader with
   more tables”; it must fail if it has long paragraphs, no recall prompts, no
   answer key, or no concept map. A reader must fail if it has no coherent
   prose. A brief must fail if it buries the recommendation.
4. **Keep style modifiers separate.** Density and accessibility change
   typography; they do not change the genre.
5. **Do not generate finished sketchnotes.** Pre-made visual summaries are
   passive; provide partial templates for learner completion.
6. **Use micro-prose, not zero prose.** The revision sheet should be
   bullet/table/map-heavy, but 1–3 sentence explanations are still needed for
   causality and “why”.

## 8. References

- Adesope, O. O., Trevisan, D. A., & Sundararajan, N. (2017). Rethinking the
  use of tests: A meta-analysis of practice testing. *Review of Educational
  Research, 87*(3), 659–701. <https://doi.org/10.3102/0034654316689306>
- Barbieri, C. A., Miller-Cotto, D., Clerjuste, S. N., & Chawla, K. (2023). A
  meta-analysis of the worked examples effect on mathematics performance.
  *Educational Psychology Review, 35*, 11.
  <https://doi.org/10.1007/s10648-023-09745-1>
- Bringhurst, R. (2012). *The elements of typographic style* (4th ed.).
  Hartley & Marks.
- British Dyslexia Association. (2023). *Dyslexia style guide*.
- Brunmair, M., & Richter, T. (2019). Similarity matters: A meta-analysis of
  interleaved learning and its moderators. *Psychological Bulletin, 145*(11),
  1029–1052. <https://doi.org/10.1037/bul0000209>
- Cepeda, N. J., Pashler, H., Vul, E., Wixted, J. T., & Rohrer, D. (2006).
  Distributed practice in verbal recall tasks. *Psychological Bulletin,
  132*(3), 354–380. <https://doi.org/10.1037/0033-2909.132.3.354>
- Chartered College of Teaching. (2018). Organising knowledge: The purpose and
  pedagogy of knowledge organisers.
  <https://my.chartered.college/impact_article/organising-knowledge-the-purpose-and-pedagogy-of-knowledge-organisers>
- Chi, M. T. H., & Wylie, R. (2014). The ICAP framework. *Educational
  Psychologist, 49*(4), 219–243.
  <https://doi.org/10.1080/00461520.2014.965823>
- Cornell University Learning Strategies Center. *The Cornell note-taking
  system*. <https://lsc.cornell.edu/notes.html>
- Dunlosky, J., Rawson, K. A., Marsh, E. J., Nathan, M. J., & Willingham, D. T.
  (2013). Improving students’ learning with effective learning techniques.
  *Psychological Science in the Public Interest, 14*(1), 4–58.
  <https://doi.org/10.1177/1529100612453266>
- Fiorella, L., & Zhang, Q. (2018). Drawing boundary conditions for learning by
  drawing. *Educational Psychology Review, 30*, 1115–1137.
  <https://doi.org/10.1007/s10648-018-9444-8>
- Ponce, H. R., Mayer, R. E., & Méndez, E. E. (2025). Learner-generated and
  instructor-provided graphic organizers as aids to learning from text: A
  meta-analysis. *Educational Psychology Review, 37*, 107.
  <https://doi.org/10.1007/s10648-025-10084-6>
- Hyland, K. (2004). *Genre and second language writing*. University of
  Michigan Press.
- Kalyuga, S., Ayres, P., Chandler, P., & Sweller, J. (2003). The expertise
  reversal effect. *Educational Psychologist, 38*(1), 23–31.
  <https://doi.org/10.1207/S15326985EP3801_4>
- Kiewra, K. A., DuBois, N. F., Christian, D., McShane, A., Meyerhoffer, M., &
  Roskelley, D. (1991). Note-taking functions and techniques. *Journal of
  Educational Psychology, 83*(2), 240–245.
  <https://doi.org/10.1037/0022-0663.83.2.240>
- Kobayashi, K. (2006). Combined effects of note-taking/-reviewing on learning
  and the enhancement through interventions: A meta-analytic review.
  *Educational Psychology, 26*(3), 459–477.
  <https://doi.org/10.1080/01443410500342070>
- Konrad, M., Joseph, L. M., & Eveleigh, E. (2009). A meta-analytic review of
  guided notes. *Education and Treatment of Children, 32*(3), 421–444.
  <https://doi.org/10.1353/etc.0.0066>
- Larwin, K. H., & Larwin, D. A. (2013). The impact of guided notes on
  post-secondary student achievement. *International Journal of Teaching and
  Learning in Higher Education, 25*(1), 47–58.
  <https://files.eric.ed.gov/fulltext/EJ1016535.pdf>
- Mayer, R. E. (2021). *Multimedia learning* (3rd ed.). Cambridge University
  Press.
- Minto, B. (2009). *The pyramid principle: Logic in writing and thinking*
  (3rd rev. ed.). Financial Times Prentice Hall.
- Nesbit, J. C., & Adesope, O. O. (2006). Learning with concept and knowledge
  maps: A meta-analysis. *Review of Educational Research, 76*(3), 413–448.
  <https://doi.org/10.3102/00346543076003413>
- Noetel, M., Griffith, S., Delaney, O., Sanders, T., Parker, P., del Pozo
  Cruz, B., & Lonsdale, C. (2022). Multimedia design for learning: An overview
  of reviews with meta-meta-analysis. *Review of Educational Research, 92*(3),
  413–454. <https://doi.org/10.3102/00346543211052329>
- Paivio, A. (1986). *Mental representations: A dual coding approach*. Oxford
  University Press.
- Pashler, H., McDaniel, M., Rohrer, D., & Bjork, R. (2008). Learning styles:
  Concepts and evidence. *Psychological Science in the Public Interest, 9*(3),
  105–119. <https://doi.org/10.1111/j.1539-6053.2009.01038.x>
- Pettersson, R. (2018). *It depends: Information design principles*.
  <https://edunorth.wordpress.com/wp-content/uploads/2019/10/it-depends-rune-pettersson-2018.pdf>
- Schriver, K. A. (1997). *Dynamics in document design*. Wiley.
- Schroeder, N. L., Nesbit, J. C., Anguiano, C. J., & Adesope, O. O. (2018).
  Studying and constructing concept maps: A meta-analysis. *Educational
  Psychology Review, 30*(2), 431–455.
  <https://doi.org/10.1007/s10648-017-9403-9>
- Section508.gov. (2026). *Understanding accessible fonts and typography*.
  <https://www.section508.gov/develop/fonts-typography/>
- Swales, J. M. (1990). *Genre analysis: English in academic and research
  settings*. Cambridge University Press.
- Swales, J. M., & Feak, C. B. (2012). *Academic writing for graduate
  students* (3rd ed.). University of Michigan Press.
- Sweller, J. (1988). Cognitive load during problem solving. *Cognitive
  Science, 12*(2), 257–285.
  <https://doi.org/10.1207/s15516709cog1202_4>
- UK Association for Accessible Formats. (2024). *Creating clear print and
  large print documents* (G003, v4).
  <https://www.ukaaf.org/wp-content/uploads/2024/12/G003-UKAAF-Creating-clear-print-and-large-print-documents-v4.pdf>
- Wammes, J. D., Meade, M. E., & Fernandes, M. A. (2016). The drawing effect:
  Evidence for reliable and robust memory benefits in free recall. *Quarterly
  Journal of Experimental Psychology, 69*(9), 1752–1776.
  <https://doi.org/10.1080/17470218.2015.1094494>
- W3C. (2024). *Web Content Accessibility Guidelines (WCAG) 2.2*.
  <https://www.w3.org/TR/WCAG22/>
