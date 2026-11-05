# Learning-science and genre review — 2026-11-02

- **Status:** Design evidence for the output-intent layer
- **Updated:** 2026-11-05 — added §6, visual learning evidence and
  source-visual pipeline implications
- **Scope:** exam summaries, concept summaries, long-form readers, reference
  sheets, practice workbooks and source visuals
- **Claim policy:** evidence-informed, not “proven”. Effect sizes describe
  average research findings; they do not guarantee an individual learner’s
  result.

## 1. Why one “summary” is not enough

A summary is a task, not a layout. Two readers can ask for the same file and
need different documents:

- An **exam student** needs fast scanning, discrimination, recall and
  self-checking under time pressure.
- A **reader trying to understand a paper** needs a coherent explanation:
  question, mechanism, evidence, example, comparison and limits.

The learning-science literature supports different design choices for these
tasks. It does **not** support matching instruction to “visual/auditory
learning styles” (Pashler et al., 2008). Use visuals because the content has a
structure worth showing, not because a learner is labelled “visual”.

## 2. What the evidence supports

| Principle | Evidence | What it means for Folisift |
|---|---|---|
| Retrieval practice | Roediger & Karpicke (2006); Dunlosky et al. (2013) high utility; Donoghue & Hattie (2021) practice testing *d* ≈ 0.74 | Exam summaries need recall prompts and separated answer keys, not just highlighted prose. |
| Distributed practice | Cepeda et al. (2006); Dunlosky et al. (2013) high utility; Donoghue & Hattie *d* ≈ 0.85 | Add a short review schedule; do not imply a PDF can enforce spacing. |
| Elaborative interrogation / self-explanation | Dunlosky et al. (2013) moderate utility; Donoghue & Hattie *d* ≈ 0.54–0.56 | Concept summaries need “why does this work?” prompts and explanations. |
| Summarization | Dunlosky et al. (2013) low utility; Donoghue & Hattie *d* ≈ 0.44; Graham & Hebert (2011) shows writing about text can support comprehension | A summary needs structure and retrieval; copious prose or copied slides is not enough. |
| Graphic organizers | Ponce, Mayer & Méndez (2025) meta-analysis; Nesbit & Adesope (2006); Schroeder et al. (2018) | Matrices, timelines and concept maps are appropriate when they encode relations; they must be labelled and explained. |
| Worked examples | Sweller & Cooper (1985); Dunlosky et al. (2013); Barbieri et al. (2023) | Procedural topics need full → faded → independent practice, especially in workbooks and exam sheets. |
| Multimedia / dual coding | Mayer’s coherence, signalling, segmenting and spatial-contiguity principles | Diagrams must be adjacent to the text they explain, captioned with the takeaway and stripped of decoration. |
| Cognitive load | Sweller (1988); Sweller et al. (2019) | Segment complex material; remove extraneous load; do not split related text and figures. |
| Note-taking | Kiewra (1989); Kobayashi (2006) | Cues, blanks and review prompts matter more than verbatim capture. |
| Accessibility / typography | WCAG 2.2; PDF/UA; UKAAF clear print | 4.5:1 contrast, tagged PDF, left-aligned text, ≤80 characters per line for prose, 8–9 pt floors for tables/references. |

### Important limitations

- Donoghue & Hattie’s meta-analysis is dominated by **surface/factual outcomes**
  and near transfer; it cautions against generalising to deep understanding.
- Most studies test a technique, not a finished PDF. Folisift can cue a
  technique; it cannot prove a learning outcome without product evaluation.
- Summarization is a family of behaviours; training and structure matter.
- “More visuals” is not a principle. Decorative or redundant visuals increase
  extraneous load (Mayer’s coherence principle).

## 3. Two summary grammars

### 3.1 Exam summary

**Reader task:** scan, recall, discriminate, self-test.

**Required structure:**

1. Scope and map (what is in/out).
2. High-yield must-know facts and definitions.
3. Comparison matrices and traps.
4. One worked or faded example per difficult procedure.
5. Recall questions with a separate answer key.
6. Short review plan.

**Layout rules:**

- Cards and tables; full-width maps; short bullets.
- Paragraphs ≤60 words; bullets ≤25 words; tables ≤5 columns and ≤8 data rows.
- No long explanatory prose; a “why” is one sentence plus a source.
- Visual ratio high; every diagram must encode a relation, not decorate.
- Page budget: 2–4 pages for a focused summary, 6–10 for a full exam revision.

**Why:** retrieval practice, graphic organizers, signalling and segmenting;
summarization alone is low utility, so the sheet must prompt recall and
self-checking.

### 3.2 Concept summary

**Reader task:** understand a paper, mechanism or argument.

**Required structure:**

1. Big picture / question and why it matters.
2. Key concepts and definitions.
3. Mechanisms: step → why → consequence.
4. Examples, comparisons and boundary cases.
5. Connections to prior/next topics.
6. Self-check prompts.

**Layout rules:**

- Explanatory prose with short paragraphs, supported by tables and figures.
- Every key concept gets a claim → mechanism → example → implication chain.
- At least one explanatory diagram or comparison table with a takeaway caption.
- Prose ratio higher than an exam sheet; bullet-only pages are a failure.
- Page budget: 4–8 pages for a focused concept summary; 10–18 for a study
  reader; 16–28 for deep study.

**Why:** coherence, signalling, dual coding, worked examples and
self-explanation; the reader needs causal and conceptual links, not only facts.

## 4. Decision rules

The agent must ask one clarifying question when the request is ambiguous:

> “Bunu sınavda hızlı tekrar için mi, konuyu anlamak için mi istiyorsun?”

| User language | Intent | Default |
|---|---|---|
| “sınav özeti”, “kısa özet”, “son bakış” | `exam-summary` | revision / 2–4 |
| “sınav için tüm konular” | `exam-revision` | revision / 6–10 |
| “özet ama anlamak istiyorum”, “makaleyi anla” | `concept-summary` | reader / 4–8 |
| “çalışma kılavuzu”, “konuyu öğren” | `study` | reader / 10–18 |
| “derin çalışma”, “makale gibi” | `deep` | reader / 16–28 |
| “formül”, “başvuru” | `reference` | reference / 2–8 |
| “soru çöz”, “pratik” | `practice` | workbook / 6–12 |
| “tek sayfa” | `one-page` | organiser / 1 |

A page budget is a constraint, not a target to pad. If a guide is outside the
budget:

- below: add a **meaningful** example, comparison, diagram or recall prompt;
- above: compress prose and remove redundancy; never drop scope or shrink type
  below the floor;
- if a source cannot fit, split it into volumes.

## 5. Gate implications

- `summary`/`concept-summary` use `source_scope`, not full A–G coverage.
- `target_pages` must be enforced after rendering.
- Exam gates: required map, recall/answer, table/visual minimum, paragraph and
  bullet limits.
- Concept gates: required explanatory prose, example and table/diagram;
  headings alone are not enough.
- Accessibility floors apply to both: body, table, label and reference sizes;
  no colour-only meaning; tagged PDF and alt text.

## 6. Visual learning evidence (2026-11-05 update)

This section extends the review for the source-visual pipeline. It is
evidence-informed, not a claim that a PDF can guarantee a learning outcome.
Effect sizes describe average findings across studies and materials.

### 6.1 What the evidence supports

| Principle | Evidence | Implication for Folisift |
|---|---|---|
| **Multimedia principle** | People generally learn better from words plus relevant pictures than from words alone; explanatory pictures outperform decorative ones (Mayer, 2021; Fletcher & Tobias, 2005; Butcher, 2014). | Include a visual when it explains a relation, mechanism, comparison or data pattern. Do not add pictures merely to fill space. |
| **Dual coding / picture superiority** | Pictures are often remembered better than words, and concrete concepts benefit from verbal plus imaginal encoding (Paivio & Csapo, 1973; Paivio, 1986). | Pair a visual with a concise verbal takeaway. The memory advantage is not a substitute for explanation or transfer. |
| **Coherence / seductive details** | Interesting but irrelevant details can reduce retention and transfer; irrelevant static images on paper can be especially harmful (Harp & Mayer, 1998; Rey, 2012; Sundararajan & Adesope, 2020). Graphics can improve liking without improving learning (Sung & Mayer, 2012). | Filter out decorative and “seductive” visuals. A visually pleasant guide is not automatically a better guide. |
| **Graph comprehension** | Graph comprehension depends on visual features (format, colour, legend, size), graph knowledge and content knowledge (Shah & Hoeffner, 2002; Cleveland & McGill, 1984; Heer & Bostock, 2010). | Preserve exact charts when values, axes or patterns matter. Do not redraw a quantitative chart with distorted proportions. |
| **Data visualization literacy** | Reading, interpreting and producing visualizations is a measurable literacy with its own framework (Börner, Bueckle & Ginda, 2019). | In data-mining and statistics courses, charts are domain content, not decoration. Embedding them can be part of the learning objective. |
| **Multiple representations** | Multiple representations help when learners are supported in connecting them; they are not automatically beneficial (Ainsworth, 2006; Rau, 2017). | Pair text, table and visual deliberately. Captions should state the connection; avoid three redundant versions of the same fact. |
| **Emotional design** | Warm colours and rounded shapes can induce positive affect and improve comprehension in some studies; transfer effects are mixed (Um et al., 2012; Plass et al., 2014). | Use palette, typography and layout for affect. Do not add cartoons or decorative images as “emotional design”. |
| **Generative drawing** | Learner-generated drawings can support comprehension and transfer, especially with training and accurate drawing (Van Meter & Garner, 2005; Fiorella & Mayer, 2016). | Where appropriate, a caption can invite self-explanation or a quick sketch. This is optional, not a substitute for a clear provided visual. |
| **Cognitive load** | Segmenting, signalling, spatial contiguity and coherence reduce extraneous load (Sweller, 1988; Mayer, 2021). | Place the figure near the paragraph it explains; label key parts; keep one concept per figure; avoid split attention. |
| **Learning styles** | There is no adequate evidence that matching instruction to “visual/auditory/kinesthetic” styles improves learning (Pashler et al., 2008). | Do not claim “visual learners learn better from visuals”. Use visuals because the content has structure worth showing. |

### 6.2 What the evidence does not support

- **“More visuals is always better.”** Extraneous visuals can increase
  extraneous cognitive load and reduce learning (coherence principle;
  seductive-details effect).
- **“This learner is a visual learner.”** Learning-style matching is not
  supported; the benefit comes from the representation and the task, not a
  learner label (Pashler et al., 2008).
- **“A picture guarantees understanding.”** Picture superiority is strongest
  for recognition/recall; deep understanding and transfer require explanation,
  connection and practice.
- **“D2 can redraw any chart.”** D2 is a diagram engine, not a plotting
  library. It can show qualitative relations, not exact coordinates or data
  values.
- **“Emotional design means adding fun pictures.”** The emotional-design
  evidence uses design features of the essential material, not extra
  seductive content.

### 6.3 Design rules for Folisift

1. **Relevance before decoration.** Classify each candidate as `core`,
   `supporting`, `decorative` or `seductive`. Skip the last two. A visual that
   only makes the page less boring but carries no content is a cost, not a
   benefit.
2. **Exactness decides the representation.**
   - Exact values, coordinates, thresholds, axes or trends → source crop,
     native table or describe.
   - Qualitative relations (separation, order, flow, hierarchy) → D2 concept
     or D2 schematic, clearly labelled as schematic. Only when the relation is
     representable with shapes (circles, ovals, boxes, arrows); smooth curves
     and exact XY plots stay source visuals.
   - Both exact and qualitative → prefer the source visual; an additional D2
     schematic is optional and must not invent data.
3. **Caption states the takeaway, not just the label.** “Şekil 3” is not a
   caption. A caption should say what to notice and why it matters, with the
   source/page citation.
4. **Place the visual where it is needed.** Insert it next to the paragraph it
   explains, not in a figure dump at the end. Spatial contiguity is part of the
   evidence base.
5. **Signal the important part.** Label axes, key regions and thresholds.
   Avoid colour-only meaning; use text labels or patterns as well.
6. **Avoid redundancy.** Do not repeat the same sentence in the body, caption
   and alt text. Body = explanation; caption = takeaway; alt = access
   description.
7. **Respect load.** One concept per figure; segment complex multi-panel
   figures; keep a readable width; do not shrink type to fit a screenshot.
8. **Treat data-mining charts as domain content.** A clustering scatter, ROC
   curve or quantile plot may be the skill being taught, not an illustration.
   Preserve it when exact reading matters; use a D2 schematic only for the
   qualitative idea.
9. **Accessibility is part of accuracy.** Alt text, contrast, tagged PDF and
   non-colour-only encoding are required. A visual that excludes a reader is
   not “visually rich”.
10. **Do not overclaim.** Write “evidence-informed”, not “proven”. Effect
    sizes are averages, not guarantees for an individual reader.

### 6.4 Implications for the visual pipeline

- The candidate report needs a `relevance` field (`core`, `supporting`,
  `decorative`, `seductive`) in addition to `material`.
- The reconstruction classifier needs `d2_concept`, `d2_schematic`, `table`,
  `text` and `unique`; “unique” includes charts, photos and screenshots that
  must keep their source pixels.
- A D2 schematic must be explicitly labelled “şematik” and must not assert
  values that are not in the source. If exact values matter, the source crop
  or a table must accompany it.
- The quality gate should reject decorative/seductive candidates even when
  they are visually attractive.
- The host still makes the final decision. The filter saves the host from
  reading every page; it does not replace judgment.

### 6.5 References for this section

- Ainsworth, S. (2006). DeFT: A conceptual framework for considering learning
  with multiple representations. *Learning and Instruction*, 16(3), 183–198.
- Börner, K., Bueckle, A., & Ginda, M. (2019). Data visualization literacy:
  Definitions, conceptual frameworks, exercises, and assessments.
  *Proceedings of the National Academy of Sciences*, 116(6), 1857–1864.
  <https://doi.org/10.1073/pnas.1807180116>
- Butcher, K. R. (2014). The multimedia principle. In R. E. Mayer (Ed.),
  *The Cambridge Handbook of Multimedia Learning* (2nd ed., pp. 174–205).
  Cambridge University Press.
- Cleveland, W. S., & McGill, R. (1984). Graphical perception: Theory,
  experimentation, and application to the development of graphical methods.
  *Journal of the American Statistical Association*, 79(387), 531–554.
  <https://doi.org/10.1080/01621459.1984.10478080>
- Fiorella, L., & Mayer, R. E. (2016). *Learning as a Generative Activity:
  Eight Learning Strategies That Promote Understanding*. Cambridge University
  Press.
- Fletcher, J. D., & Tobias, S. (2005). The multimedia principle. In
  R. E. Mayer (Ed.), *The Cambridge Handbook of Multimedia Learning*
  (pp. 117–133). Cambridge University Press.
- Harp, S. F., & Mayer, R. E. (1998). How seductive details do their damage:
  A theory of cognitive interest in science learning. *Journal of Educational
  Psychology*, 90(3), 414–434. <https://doi.org/10.1037/0022-0663.90.3.414>
- Heer, J., & Bostock, M. (2010). Crowdsourcing graphical perception: Using
  mechanical turk to assess visualization design. *CHI '10*, 2039–2042.
  <https://doi.org/10.1145/1753326.1753633>
- Mayer, R. E. (2021). *Multimedia Learning* (3rd ed.). Cambridge University
  Press.
- Mayer, R. E., & Fiorella, L. (2022). Principles for reducing extraneous
  processing in multimedia learning: Coherence, signaling, redundancy,
  spatial contiguity, and temporal contiguity principles. In R. E. Mayer
  (Ed.), *The Cambridge Handbook of Multimedia Learning* (3rd ed.). Cambridge
  University Press.
- Paivio, A. (1986). *Mental Representations: A Dual Coding Approach*. Oxford
  University Press.
- Paivio, A., & Csapo, K. (1973). Picture superiority in free recall: Imagery
  or dual coding? *Cognitive Psychology*, 5(2), 176–206.
- Pashler, H., McDaniel, M., Rohrer, D., & Bjork, R. (2008). Learning styles:
  Concepts and evidence. *Psychological Science in the Public Interest*,
  9(3), 105–119.
  <https://doi.org/10.1111/j.1539-6053.2009.01038.x>
- Plass, J. L., Heidig, S., Hayward, E. O., Homer, B. D., & Um, E. (2014).
  Emotional design in multimedia learning: Effects of shape and color on
  affect and learning. *Learning and Instruction*, 29, 128–140.
  <https://doi.org/10.1016/j.learninstruc.2013.02.006>
- Rau, M. A. (2017). Conditions for the effectiveness of multiple visual
  representations in enhancing STEM learning. *Educational Psychology Review*,
  29(4), 717–761. <https://doi.org/10.1007/s10648-016-9365-3>
- Rey, G. D. (2012). A review of research and a meta-analysis of the seductive
  detail effect. *Educational Research Review*, 7(3), 216–237.
  <https://doi.org/10.1016/j.edurev.2012.05.003>
- Shah, P., & Hoeffner, J. (2002). Review of graph comprehension research:
  Implications for instruction. *Educational Psychology Review*, 14(1),
  47–69. <https://doi.org/10.1023/A:1013180410169>
- Sundararajan, N., & Adesope, O. (2020). Keep it coherent: A meta-analysis
  of the seductive details effect. *Educational Psychology Review*, 32(3),
  707–734. <https://doi.org/10.1007/s10648-020-09522-4>
- Sung, E., & Mayer, R. E. (2012). When graphics improve liking but not
  learning from online lessons. *Computers in Human Behavior*, 28(5),
  1618–1625. <https://doi.org/10.1016/j.chb.2012.03.026>
- Sweller, J. (1988). Cognitive load during problem solving: Effects on
  learning. *Cognitive Science*, 12(2), 257–285.
- Sweller, J., Ayres, P., & Kalyuga, S. (2011). *Cognitive Load Theory*.
  Springer.
- Um, E., Plass, J. L., Hayward, E. O., & Homer, B. D. (2012). Emotional
  design in multimedia learning. *Journal of Educational Psychology*, 104(2),
  485–498. <https://doi.org/10.1037/a0026609>
- Van Meter, P., & Garner, J. (2005). The promise and practice of
  learner-generated drawing: Literature review and synthesis. *Educational
  Psychology Review*, 17, 285–325.
  <https://doi.org/10.1007/s10648-005-8136-3>

## 7. References

- Cepeda, N. J., Pashler, H., Vul, E., Wixted, J. T., & Rohrer, D. (2006).
  Distributed practice in verbal recall tasks. *Psychological Bulletin*.
- Chi, M. T. H., et al. (1994). Eliciting self-explanations improves
  understanding. *Cognitive Science*.
- Dunlosky, J., Rawson, K. A., Marsh, E. J., Nathan, M. J., & Willingham, D. T.
  (2013). Improving students’ learning with effective learning techniques.
  *Psychological Science in the Public Interest*, 14(1), 4–58.
  <https://doi.org/10.1177/1529100612453266>
- Donoghue, G. M., & Hattie, J. A. C. (2021). A meta-analysis of ten learning
  techniques. *Frontiers in Education*, 6, 581216.
  <https://doi.org/10.3389/feduc.2021.581216>
- Graham, S., & Hebert, M. (2011). Writing to read: A meta-analysis of the
  impact of writing and writing instruction on reading. *Harvard Educational
  Review*, 81(4), 710–744.
- Kiewra, K. A. (1989). A review of note-taking: The encoding-storage paradigm
  and beyond. *Educational Psychology Review*.
- Kobayashi, K. (2006). Conditional effects of interventions in note-taking
  procedures on learning. *Journal of Educational Psychology*.
- Mayer, R. E. (2021). *Multimedia Learning* (3rd ed.). Cambridge University
  Press.
- Pashler, H., McDaniel, M., Rohrer, D., & Bjork, R. (2008). Learning styles:
  Concepts and evidence. *Psychological Science in the Public Interest*.
- Ponce, H. R., Mayer, R. E., & Méndez, E. E. (2025). Learner-generated and
  instructor-provided graphic organizers as aids to learning from text: A
  meta-analysis. *Educational Psychology Review*.
- Roediger, H. L., & Karpicke, J. D. (2006). Test-enhanced learning: Taking
  memory tests improves long-term retention. *Psychological Science*, 17(3),
  249–255.
- Sweller, J., & Cooper, G. A. (1985). The use of worked examples as a
  substitute for problem solving in learning algebra. *Cognition and
  Instruction*.
- WCAG 2.2, W3C; PDF/UA-1 (ISO 14289-1); UKAAF clear print guidance.
