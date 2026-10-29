# Evidence base for Folisift guide design

- **Status:** Normative draft for the v0.9 quality plan
- **Date:** 2026-10-29
- **Scope:** learning science, note-taking, summarization, document design,
  accessibility and academic/professional genre conventions
- **Related:** `docs/plans/2026-10-29-guide-quality-visual-coverage-and-style-selection.md`,
  `docs/authoring-spec.md`, `docs/design-system.md`

## 1. Purpose and claims policy

Folisift should be **evidence-informed**, not “scientifically proven”. The
research below supports design principles; it does not guarantee that a
particular PDF will improve every learner’s exam score. Product-level claims
require product-level evaluation (Section 6).

Rules:

- Cite the principle, not a guaranteed outcome.
- Do not say “proven to improve learning”; say “designed around evidence-based
  principles” and report the actual evaluation.
- Do not treat summarization as a high-utility study technique by itself.
- Do not add visuals, quizzes or review schedules as decoration. Each component
  must have a learning purpose and a measurable place in the guide.
- Prefer a short, clear guide over a padded one. Length is an output, not a
  target.

## 2. Evidence summary

Strength labels: **strong** = meta-analysis or systematic review with consistent
effects; **moderate** = experimental evidence with boundary conditions;
**mixed** = evidence depends on task, learner or implementation; **standard** =
professional/accessibility convention, not a learning-outcome claim;
**heuristic** = project hypothesis to calibrate.

| Principle | Evidence | Strength | Design implication | Caveat |
|---|---|---|---|---|
| Retrieval practice | Roediger & Karpicke (2006); Dunlosky et al. (2013) rate practice testing **high utility** | strong | Optional recall prompts, self-testing, “close the book and explain” cues; answers separated from prompts | Tests must be low-stakes and feedback should follow; avoid turning the guide into a quiz book by default |
| Distributed practice | Cepeda et al. (2006); Dunlosky et al. (2013) rate distributed practice **high utility** | strong | A short review schedule (e.g. same day, 2–3 days, 1 week) and cumulative review cues | A PDF cannot enforce spacing; it can only prompt it |
| Interleaved practice | Rohrer & Taylor (2007); Dunlosky et al. (2013) rate interleaving **moderate utility** | moderate | Mixed comparison examples, “which method applies?” prompts, cross-topic tables | Strongest for problem-solving domains; not a universal rule |
| Elaborative interrogation / self-explanation | Chi et al. (1994); Dunlosky et al. (2013) rate self-explanation **moderate utility** | moderate | “Why does this work?”, “What changes if…?”, worked-example annotations, critical distinctions | Prompts must be content-specific, not generic study advice |
| Summarization | Brown & Day (1983) macrorules; Graham & Hebert (2011) summary-writing effect; Dunlosky et al. (2013) rate summarization **low utility** as a study technique | mixed | Use structured summary rules: select, delete, generalize, construct; preserve source claims and examples | Summarization can support comprehension when taught/structured, but it is weak as a standalone memory strategy |
| Note-taking | Kiewra (1989); Kobayashi (2006) meta-analysis; review matters more than capture | moderate (small effects) | Generative notes, cue boxes, external-storage design, review prompts; avoid verbatim slide copies | The act of note-taking alone is not enough; reviewing notes is the larger benefit |
| Multimedia / dual coding | Mayer’s cognitive theory of multimedia learning; Paivio’s dual coding | strong for principles, mixed by context | Meaningful diagrams with adjacent captions; coherence (no decoration), signaling, segmenting, spatial contiguity | Static PDF cannot use all multimedia principles (modality, temporal contiguity); do not overgeneralize |
| Cognitive load | Sweller (1988); cognitive load theory | strong | Segment complex material, remove extraneous load, use worked examples, avoid split attention and redundancy | Novice/expert reversal: advanced learners may need less scaffolding |
| Worked examples | Sweller & Cooper (1985); Dunlosky et al. (2013) | strong/moderate | For procedural/math/code domains: full worked example, then faded practice, then independent problem | Fading and self-explanation matter; a bare example is not enough |
| Document design / accessibility | WCAG 2.2; PDF/UA (ISO 14289) | standard | Contrast, type size, tagged PDF, headings, alt text, captions, no colour-only meaning, readable tables | Accessibility is a floor, not a guarantee of comprehension |
| Academic explanation | Composition and genre research; academic style guides | standard | Claim → evidence → explanation → implication; define terms; distinguish source claim from interpretation; cite | Not every guide is an argument; formula sheets and glossaries need different genres |
| Plain language | Federal plain language guidelines; professional style guides | standard | Short sentences, active voice, common words, explicit structure; define jargon | Plain does not mean simplistic; technical terms must remain precise |
| Academic/professional genre | APA, IEEE, Chicago, OSCOLA; IMRAD, IRAC, SOAP, BLUF | standard | Structure and citation conventions appropriate to domain; do not force one genre onto every subject | Genre conventions are not empirical learning effects; they are professional expectations |

## 3. What the evidence does and does not support

### 3.1 It supports

- **Structure and signaling** (headings, chapter maps, advance organizers) reduce
  search cost and help learners build a coherent model.
- **Retrieval and spacing** are the strongest general-purpose learning
  techniques in the Dunlosky review.
- **Self-explanation and elaborative interrogation** help when prompts are
  specific and content-rich.
- **Worked examples** reduce cognitive load for novices in procedural domains.
- **Meaningful visuals** can support dual coding when they explain structure or
  process, not when they decorate.
- **Generative note-taking** and review are more useful than verbatim capture.

These are general learning findings. Their transfer to a static PDF study guide
must be tested with the evaluation in Section 6; a PDF can cue retrieval and
spacing but cannot enforce them.

### 3.2 It does not support

- “More pages = more learning.”
- “More diagrams = better learning.” Decorative or redundant visuals add
  cognitive load.
- “Summarization alone is a proven study method.” Dunlosky rates it low
  utility; use it as a comprehension/organization task, then add retrieval and
  spacing.
- “One style fits every subject.” Genre, domain, prior knowledge and task
  change the appropriate structure and typography.
- “A PDF can replace deliberate practice.” It can organize and cue practice; it
  cannot perform the learner’s retrieval.

## 4. Component map

| Guide component | Evidence basis | Rule |
|---|---|---|
| Front matter / “Bu kitapçık hakkında” | Advance organizer, pre-training, signaling | State scope, source convention, method, limitations, how to use the guide |
| Chapter opener / “Bu bölümde” | Signaling, segmenting | One short advance organizer; no generic motivation |
| Definitions | Precision, elaboration | One precise definition; English term in parentheses on first use; source ref |
| Critical distinction | Elaborative interrogation, contrast | Explain the difference and the consequence; not just a label |
| Worked example | Worked-example effect | Show input → step → result → why; fade support in practice |
| Diagram | Multimedia, dual coding, coherence | Only for mechanism/structure; caption with takeaway, source, alt |
| Table | Spatial contiguity, comparison | Use for contrasts and parameters; explain what the columns mean |
| Recall prompt (optional) | Retrieval practice | Ask a specific question; answer separated; low-stakes |
| Review schedule (optional) | Distributed practice | Short, concrete schedule; cumulative review; no generic advice |
| Mixed comparison (optional) | Interleaving | Mix problem types or cases; ask “which method and why?” |
| Synthesis / connections | Elaboration, organization | Connect to prior/next chapters; label interpretation |
| G coverage audit | Metacognition, traceability | Machine counts + human caveats; no unsupported “all inspected” claims |
| Source refs | Traceability, academic integrity | Every non-obvious claim, table, figure, example or formula |

## 5. Dynamic adaptation

The system must not hard-code one course, one page count or one style. It needs
three profiles and a rule layer. The host LLM may override a default, but must
record a rationale.

### 5.1 Source profile

Detected deterministically where possible:

```json
{
  "source_type": "slides | textbook | paper | scanned | code | math | legal | clinical | mixed",
  "units": {"pages": 103, "words": 6448, "sections": 20},
  "text_layer": "native | partial | none | OCR",
  "low_text_pages": 9,
  "visual_load": "low | medium | high",
  "structure_clarity": "explicit | implicit | none",
  "language": "tr",
  "domain_hint": "cs",
  "difficulty_hint": "intro | intermediate | advanced"
}
```

### 5.2 Learner profile

```json
{
  "study_mode": "exam | deep | reference | teach",
  "prior_knowledge": "novice | intermediate | advanced",
  "time_budget": "quick-review | standard | full-course",
  "accessibility": "standard | large-print | high-contrast",
  "output_language": "tr"
}
```

### 5.3 Output profile

```json
{
  "structure": "a-g | imrad | irac | soap | formula-sheet | literature-matrix | lecture-notes | executive-brief",
  "style": "paper | clean | journal | brief | clinical | casebook | accessible",
  "page_range": [18, 26],
  "visual_budget": "0-3 per chapter; high-priority targets mandatory",
  "component_mix": {
    "definitions": "high",
    "worked_examples": "high",
    "tables": "medium",
    "diagrams": "high",
    "recall_prompts": "optional",
    "review_schedule": "optional"
  },
  "citation": "page-ref | apa | ieee | chicago | oscola"
}
```

### 5.4 Decision rules

Source-type rules:

- **Native text / slide deck:** text extraction + targeted visual audit.
- **Scanned / image-only:** every page is a visual target; OCR/vision is a
  recorded method; no text-only coverage claim. Unreadable pages go to G.
- **Mixed text/visual:** text extraction plus low-text page audit.
- **No explicit structure:** build a manual canonical outline from page ranges
  or the source's own questions; never invent section numbers.
- **Uncertain domain:** default to `general` and ask; do not auto-force a
  clinical/legal/business structure from weak signals.
- **Very large source:** split by source/part into volumes; do not shrink type.
- **Very small source:** choose a short genre (brief, formula sheet, lecture
  notes); do not pad to a fixed page count.
- **Prior knowledge / expertise reversal:** novices need definitions, worked
  examples and more scaffolding; advanced learners need synthesis, edge cases
  and fewer redundant explanations. Do not add beginner filler to an advanced
  guide or omit scaffolding from a novice guide.

Study-mode implications:

| `study_mode` | Emphasis | Retrieval/spacing | Length tendency |
|---|---|---|---|
| `exam` | high-yield definitions, comparisons, worked examples, common errors | optional recall prompts + short review plan | medium |
| `deep` | mechanisms, derivations, assumptions, limitations, connections | self-explanation prompts; spaced review optional | longer |
| `reference` | definitions, formulas, tables, units, index | minimal prompts; quick lookup | compact |
| `teach` | leads, concrete examples, visual explanation, optional self-check | optional recall prompts | medium–long |

| Input | Default | Override rule |
|---|---|---|
| `domain=cs`, slides, exam | `a-g` or `lecture-notes`; `paper`/`clean`; visual budget medium–high | Host may choose `clean` for code-heavy material; record why |
| `domain=humanities`, paper/textbook | `imrad`/`literature-matrix`; `journal`; visual budget low–medium | Use `paper` if the output is a course reader, not a research report |
| `domain=business` | `executive-brief`; `brief`; visual budget low–medium | Use `clean` for data-heavy reports; `brief` for decision memos |
| `domain=medicine` | `soap`; `clinical`; visual budget medium | No diagnosis without evidence; red flags and uncertainty required |
| `domain=law` | `irac`; `casebook`; visual budget low | Jurisdiction and citations required; no invented outcomes |
| `domain=math` | `formula-sheet`/`a-g`; `paper`/`clean`; visual budget medium | Worked examples and units are mandatory; diagrams only for geometry/process |
| Accessibility request | `accessible`; 12 pt+; high contrast | Any structure is allowed; tables/figures must reflow |
| Short output request | `executive-brief`/`formula-sheet`/`lecture-notes` | Do not compress a full course into a brief; choose the right genre |
| Long multi-source course | one guide with one B part per source, or a volume set | Split at source/part boundaries; do not shrink type below the floor |

### 5.5 Length policy

**Length is an output of content, genre and learner task — not a target to
pad.**

- Replace fixed `target_pages` with `page_range`.
- Compute a content budget from canonical sections, concept density, worked
  examples, tables and high-value visuals.
- Allocate a range per section (for example 0.5–2.5 pages for a normal
  section); the total plus front/back matter gives the guide range.
- Use the existing source-density heuristic as a starting point, not a law:
  sparse slide decks around 12–16 slides/page; dense decks around 8–12; written
  sources need section-level budgeting.
- If output is below the range, add **meaningful** worked examples, diagrams,
  comparisons or retrieval prompts — never filler prose.
- If output is above the range, compress prose and remove redundancy — never
  drop scope or shrink type below the accessibility floor.
- Split into volumes when a source set cannot fit without violating the floor.
- Validate with the underfilled-page gate and a readability review.

Short/medium/long examples:

| Genre | Typical range | What changes |
|---|---:|---|
| Executive brief | 4–8 pages | BLUF, findings, recommendation, risks; no full A–G |
| Formula sheet | 4–10 pages | Definitions, formulas, units, pitfalls, worked examples |
| Lecture notes | 8–16 pages | Learning goals, concepts, examples, summary |
| Course guide (2 sources) | 18–30 pages | A–G, coverage, glossary, methods, facts, connections |
| Full course (9 sources) | 60–100+ pages or volumes | One B part per source; split volumes; index |

These are **project heuristics**; calibrate them against expert review and user
tests. Do not present them as universal scientific constants.

## 6. Evaluation

A guide is evidence-informed only if the output is measured.

### 6.1 Expert rubric

- factual accuracy and source traceability;
- canonical coverage;
- visual disposition and diagram quality;
- clarity and terminology;
- accessibility and typography;
- style/genre compliance;
- length appropriateness (no padding, no omission).

### 6.2 Small user evaluation (optional but recommended)

- 5–15 target students; pre/post comprehension quiz;
- delayed recall after 2–7 days;
- time-to-answer for exam-style questions;
- perceived cognitive load (for example NASA-TLX);
- preference/usability interview.
- Compare guide vs source slides or a plain summary; report effect sizes with
  confidence intervals, not just means.

### 6.3 Claims

Only after a product evaluation may Folisift say “improved recall in our
sample”. Until then: “designed around evidence-based principles”.

## 7. References

- Brown, A. L., & Day, J. D. (1983). Macrorules for summarizing texts: The
  development of expertise. *Journal of Verbal Learning and Verbal Behavior,
  22*(1), 1–14. <https://www.ideals.illinois.edu/items/17782>
- Cepeda, N. J., Pashler, H., Vul, E., Wixted, J. T., & Rohrer, D. (2006).
  Distributed practice in verbal recall tasks: A review and quantitative
  synthesis. *Psychological Bulletin, 132*(3), 354–380.
  <https://pubmed.ncbi.nlm.nih.gov/16719566/>
- Chi, M. T. H., De Leeuw, N., Chiu, M.-H., & LaVancher, C. (1994). Eliciting
  self-explanations improves understanding. *Cognitive Science, 18*(3),
  439–477. <https://doi.org/10.1207/s15516709cog1803_3>
- Dunlosky, J., Rawson, K. A., Marsh, E. J., Nathan, M. J., & Willingham, D. T.
  (2013). Improving students’ learning with effective learning techniques:
  Promising directions from cognitive and educational psychology.
  *Psychological Science in the Public Interest, 14*(1), 4–58.
  <https://doi.org/10.1177/1529100612453266>
- Graham, S., & Hebert, M. (2011). Writing to read: A meta-analysis of the
  impact of writing and writing instruction on reading. *Harvard Educational
  Review, 81*(4), 710–744. <https://eric.ed.gov/?id=EJ961480>
- Kiewra, K. A. (1989). A review of note-taking: The encoding-storage paradigm
  and beyond. *Educational Psychology Review, 1*(2), 147–172.
  <https://doi.org/10.1007/BF01326640>
- Kobayashi, K. (2006). Combined effects of note-taking/-reviewing on learning
  and the enhancement through interventions: A meta-analytic review.
  *Educational Psychology, 26*(3), 459–477.
  <https://doi.org/10.1080/01443410500342070>
- Mayer, R. E. (2021). *Multimedia learning* (3rd ed.). Cambridge University
  Press. See also Mayer’s evidence-based principles:
  <https://psycnet.apa.org/record/2021-42922-001>
- Roediger, H. L., & Karpicke, J. D. (2006). Test-enhanced learning: Taking
  memory tests improves long-term retention. *Psychological Science, 17*(3),
  249–255. <https://doi.org/10.1111/j.1467-9280.2006.01693.x>
- Rohrer, D., & Taylor, K. (2007). The shuffling of mathematics problems
  improves learning. *Instructional Science, 35*, 481–498.
  <https://psycnet.apa.org/record/2008-00329-002>
- Sweller, J. (1988). Cognitive load during problem solving: Effects on
  learning. *Cognitive Science, 12*(2), 257–285.
  <https://doi.org/10.1207/s15516709cog1202_4>
- Sweller, J., & Cooper, G. A. (1985). The use of worked examples as a
  substitute for problem solving in learning algebra. *Cognition and
  Instruction, 2*(1), 59–89.
  <https://doi.org/10.1207/s1532690xci0201_3>
- Paivio, A. (1986). *Mental representations: A dual coding approach*. Oxford
  University Press.
- Swales, J. M. (1990). *Genre analysis: English in academic and research
  settings*. Cambridge University Press.
- W3C (2023). *Web Content Accessibility Guidelines (WCAG) 2.2*.
  <https://www.w3.org/TR/WCAG22/>
- ISO 14289-1:2014. *Document management applications — Electronic document
  file format enhancement for accessibility — Part 1: Use of ISO 32000-1
  (PDF/UA-1)*.
- Plain Language Action and Information Network. *Federal plain language
  guidelines*. <https://www.plainlanguage.gov/guidelines/>
- APA Style. <https://apastyle.apa.org/>
- IEEE Author Center. <https://ieeeauthorcenter.ieee.org/>
- Chicago Manual of Style. <https://www.chicagomanualofstyle.org/>
- OSCOLA. <https://www.law.ox.ac.uk/oscola>
- IMRAD. <https://writingcenter.gmu.edu/writing-resources/imrad/writing-an-imrad-report>
- IRAC/CRAC/CREAC. <https://www.law.columbia.edu/sites/default/files/2024-10/WC%20Handout%20IRAC%2C%20CRAC%2C%20CREAC.revised%209.24.pdf>
