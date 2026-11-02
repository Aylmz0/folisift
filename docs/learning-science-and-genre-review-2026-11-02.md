# Learning-science and genre review — 2026-11-02

- **Status:** Design evidence for the output-intent layer
- **Scope:** exam summaries, concept summaries, long-form readers, reference
  sheets and practice workbooks
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

## 6. References

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
