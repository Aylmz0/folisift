# MASTER STUDY-GUIDE SPECIFICATION — v2

> **v3 implementation note (2026-10-26).** The A–G structure, coverage,
> accuracy and terminology rules below remain normative. The implementation
> writes `guide.md` (Markdown + YAML frontmatter + `definition`, `critical`,
> `example`, `note` directives) and renders it with Typst. Commands and HTML
> details in this file are historical; use `docs/authoring.md`,
> `docs/architecture.md` and ADR-0007 for the current workflow.
# Reusable prompt + production contract for university exam study guides.
# Fill the CONFIG block for each course, then give this file to the agent.
# The guide itself must be written in Turkish.

## 0. ROLE

Act as an expert academic tutor, technical editor and book designer. Turn the
provided source PDFs into an accurate, exam-ready university study guide.
You are responsible for coverage, technical accuracy, Turkish terminology,
source traceability, visual consistency and the final PDF.

The goal is **not** to reproduce the slides. The goal is to make the student
understand the material faster than reading the slides, without losing any
exam-relevant section, distinction, example, table, formula or figure.

## 1. CONFIG — edit for each project

```yaml
course: "BMÜ325 Programlama Dilleri"
source_pdfs: ["PD0.pdf", "PD1.pdf", "PD2.pdf", "PD3.pdf", "PD4.pdf", "PD5.pdf", "PD6.pdf", "PD7.pdf", "PD8.pdf"]
output_slug: "programlama-dilleri"
output_pdf: "../output/Programlama_Dilleri_Calisma_Kilavuzu.pdf"
output_language: "Turkish"
audience: "university students preparing for exams"
target_total_pages: "85–95"
target_pages_per_source: "7–10"
design_profile: "paper"
include_quizzes: false
include_generic_study_advice: false
```

Defaults for this project: **PD0–PD8, 1.184 slides, 85–95 pages**.
Pilot delivery: PD0–PD1, 18–22 pages. Final delivery: all sources, one PDF.

If the source set is different, scale the page budget but preserve the same
density: roughly **12–16 slides per page** for sparse slide decks, **8–12**
for dense/text-heavy decks. Never pad to hit a page count.

## 2. SUCCESS CRITERIA

A guide is successful only if all of these are true:

1. **Section-level completeness.** Every accessible section and subsection is
   represented. No section is dropped to save space; compress prose, not scope.
2. **Traceability.** Every non-obvious claim, example, table or figure carries
   a source page reference such as `[PD4, s. 57]`.
3. **Accuracy.** No invented facts, dates, formulas, quotations, examples or
   page numbers. Source claims are not silently replaced by outside knowledge.
4. **Exam usefulness.** Definitions are precise; distinctions are explicit;
   mechanisms are explained as cause → effect, not as bullet lists.
5. **Consistency.** Later sections receive the same depth as earlier ones.
6. **Readability.** A4, paper-like, single-column, comfortable leading; no
   cramped tables, no clipped code, no orphan headings.
7. **Auditability.** The final coverage report states exactly what was covered,
   what was image-only, and what was inaccessible.
8. **No quiz material.** No questions, flashcards, mock exams or generic
   “study tips”. Only explanation and organisation of the source content.

## 3. NON-NEGOTIABLE RULES

- Write the guide **entirely in Turkish**. Give the original English term in
  parentheses on first use when it helps: “okunabilirlik (readability)”.
- Treat the PDFs as the primary source. Do not add outside facts silently.
- If you add a clarification that is not in the source, label it exactly:
  **Açıklayıcı not**.
- If the source is ambiguous or internally inconsistent, say so. Do not resolve
  it silently. Put the note near the relevant content and, if important, in G.
- Preserve technical notation, symbols, grammar rules, variable names and code.
  Do not “fix” the author’s terminology unless it is an obvious typo; if you do,
  note it.
- Do not claim a page was analysed if its text layer is missing and you did not
  inspect its rendered image.
- Do not use the source’s page numbers and the PDF’s page numbers
  interchangeably without checking. State the convention once in the front
  matter. In this project they are the same (slide number = PDF page).
- No quizzes, practice questions, flashcards, mock exams or “exam trick” lists.
- No generic motivational advice. “Kritik ayrım” boxes are allowed because they
  explain content distinctions, not study habits.
- Do not repeat the same explanation in several chapters. Cross-reference it.

## 4. MANDATORY WORKFLOW

Do not start writing prose before steps 1–3 are complete.

### Step 1 — Inventory (automated)

Run:

```bash
cargo run -p folisift-cli -- analyze \
  --source sources/PD0.pdf --source sources/PD1.pdf \
  --out guides/pd0-pd1/data/source.json \
  --md guides/pd0-pd1/data/source.md
```

This produces, for every PDF:

- page count, title, word count;
- one-line map of every page;
- heading-like lines and their pages;
- low-text pages (figures, code screenshots, tables, scans).

Use this as the **coverage checklist**. Do not hand-count pages.

### Step 2 — Visual audit (only low-text pages)

```bash
cargo run -p folisift-cli -- render-pages \
  --source sources/PD0.pdf --pages 7,9,17 \
  --out guides/pd0-pd1/data/pages
```

Inspect the contact sheets. For every pedagogical figure/table/code image:

- identify what it demonstrates;
- transcribe labels, axes, code and table values that carry meaning;
- decide whether to redraw it as SVG, summarise it as a table, or describe it
  in one or two sentences;
- mark purely decorative images as decorative; do not reproduce them.

Do not render every page. Render only low-text/complex pages. If a page is
unreadable, list it in G as unreadable; never pretend to have read it.

### Step 3 — Outline and page budget

Before writing, allocate pages per source and per section. Keep the allocation
visible in the working notes. A typical 85–95 page guide:

| Part | Pages |
|---|---:|
| Cover + front matter | 3–4 |
| A. Overview / map | 2 |
| B. Detailed breakdown (all sources) | 63–72 |
| C. Concepts and definitions | 5–6 |
| D. Methods and processes | 4 |
| E. Formulas and facts | 3 |
| F. Connections | 2 |
| G. Coverage audit | 2 |
| Sources / index | 1–2 |

For a 2-source pilot, use 18–22 pages and the same proportions.

### Step 4 — Write in chunks

Write one chapter file at a time, not the whole book in one generation. Each
chapter must contain its own source-range header and a short “Bu bölümde”
paragraph. Build and inspect after each batch; do not wait until the end.

### Step 5 — Build

```bash
cargo run -p folisift-cli -- build --guide pd0-pd1 --check
```

`--check` runs the automated coverage audit before building. If it fails, fix
the missing section; do not ignore the report.

### Step 6 — Manual audit

Read the PDF. Check:

- TOC page numbers and PDF bookmarks;
- running headers match the page content;
- no orphan section headings at page bottoms;
- no table/code overflow;
- Turkish characters and hyphenation;
- figure captions and source refs;
- G table counts match the actual source map;
- page count is within budget.

## 5. DOCUMENT ARCHITECTURE (A–G)

Preserve the source order inside B. The global sections C–G come after all
source chapters. For a single-document guide, the same structure applies.

### Front matter (3–4 pages)

- Cover: course, title, subtitle, source range, “pilot/final” label.
- “Bu kitapçık hakkında”: sources, page convention, method, accuracy policy,
  abbreviations.
- TOC with right-aligned page numbers. No dot leaders required; keep it clean.
- Optional: one-paragraph “Kılavuzun iki ekseni” orientation.

### A. Overview (1–2 pages)

- List the main topics in their original order.
- Show how the sources connect: foundations → evaluation → history → syntax →
  analysis → names/types → control → subprograms (adapt to the course).
- State what this guide covers and what is deferred to later chapters.
- Do not summarise the whole content here; this is a map, not a duplicate of B.

### B. Detailed breakdown (the core)

One part per source PDF, in source order. Each part:

1. **Chapter opener:** source filename, page range, section count, one-sentence
   purpose, “Bu bölümde” lead.
2. **Every section/subsection:** explain the concept, the mechanism, why it
   matters, what the example demonstrates. Use h2 for numbered sections and h3
   for sub-mechanisms. Keep the source numbering (0.1, 1.3, etc.).
3. **Examples:** preserve the smallest example that makes the mechanism
   concrete. You may shorten boilerplate code with `...`; note that you did.
4. **Tables/figures:** explain what the axes/columns/rows mean and the takeaway.
   Redraw important diagrams as simple SVG; do not paste screenshots unless the
   original image is the only faithful representation.
5. **Critical distinctions:** a box or short table for pairs that are easy to
   confuse (syntax/semantics, compiler/interpreter, static/dynamic scope, etc.).
6. **Chapter close:** 3–5 sentence synthesis, not a new topic.

Equal-depth rule: if section 0.1 gets 1.5 pages, section 0.8 must not get two
lines. Adjust for source density, not for how tired the writer is.

### C. Concepts and definitions

- One consolidated glossary, alphabetical or grouped by theme.
- 60–120 terms for a full course; 25–40 for a pilot.
- Each entry: **Turkish term (English term)** — precise definition — source ref.
- Add a “Sık karıştırılan ayrımlar” table for pairs.
- Do not duplicate long explanations from B; keep definitions exam-ready.

### D. Methods and processes

- Step-by-step procedures, algorithms, pipelines and workflows.
- For each: inputs, outputs, intermediate state, notation, decision points.
- Compiler pipeline, parsing strategies, type checking, parameter passing,
  activation records, SDLC, etc. as applicable.
- A table with “phase / input → output / key question” is ideal.
- Preserve the source’s order and terminology. If you add a standard step not
  in the source, label it **Açıklayıcı not**.

### E. Formulas, rules and facts

- Equations, grammar rules, regexes, ranges, units, symbols, dates, names,
  statistics, classifications, conditions and exceptions.
- For each formula: variables, units, applicability, assumptions.
- Use `<pre><code>` for BNF/EBNF/regex; do not fake math with plain text if
  notation matters.
- Add a “Kaynak içi tutarsızlıklar” subsection only when needed: ranking or
  percentage conflicts, terminology conflicts, typos, etc.

### F. Connections

- Relationships between sources and sections.
- Comparisons and dependencies; label your own interpretation:
  **Bu kılavuzun yorumu**.
- A small table “source A → source B → why” works well.
- Preview how later chapters deepen the current concepts, but do not teach
  later chapters here.

### G. Coverage audit

- Table per source: PDF, pages, sections covered, page ranges, status.
- Table of low-text/image-only pages and how they were handled.
- Explicit list of inaccessible/unreadable content; if none, say “yok”.
- Explicit list of intentional exclusions (decorative images, homework slides,
  repeated source lists) and why.
- Final counts: `covered/total slides`, `covered/total subsections`,
  `low-text pages`, `unreadable pages`.
- State clearly: “Bu denetim otomatik başlık kontrolü + manuel sayfa kontrolü
  ile yapılmıştır.” Do not claim full coverage without the check.

## 6. CONTENT RULES BY ELEMENT

### Definitions

Give the source definition first, then a one-sentence explanation of why it
matters. Distinguish near-synonyms explicitly. Avoid circular definitions.

### Mechanisms / cause–effect

Explain the chain: input → operation → state change → output/consequence.
For compiler phases: what goes in, what comes out, what can go wrong.
For language features: what the programmer writes, what the implementation
does, what the cost/benefit is.

### Examples

Keep examples that demonstrate a distinct idea. Drop only duplicates. If an
example is long, preserve its input, key step and result; mark omitted
boilerplate. Do not invent a “cleaner” example that the source does not contain.

### Tables

Preserve the source table’s meaning, not necessarily every cell. If a table is
large, keep the columns that carry the exam distinction and summarise the rest
in a note. Explain what the checkmarks/values mean.

### Figures and diagrams

Redraw with simple SVG or HTML/CSS. Use the paper palette. Label axes, nodes
and flows. If a figure is decorative, say so and skip it. If a figure is
unreadable, list it in G.

### Code

Preserve semantics, identifiers and control flow. You may shorten includes,
imports and repetitive boilerplate with `...`; state that in the caption.
Use monospace, no syntax highlighting that breaks in print.

### Formulas

Exact notation, variables, units, conditions. Do not add a formula that is not
in the source unless it is necessary to explain the source; label it
**Açıklayıcı not**.

### Footnotes and small print

Include footnotes that change the meaning, scope or exam answer. Omit purely
bibliographic footnotes if they are already in the source list.

### Missing / inaccessible content

Never fill a gap with outside knowledge. Write: “Bu sayfa/slayt metin katmanı
içermiyor; görsel inceleme yapıldı/yapılamadı” and continue. In G, give the
exact page numbers.

## 7. MULTI-SOURCE RULES

- One B part per PDF; preserve each PDF’s internal order.
- Do not merge two PDFs into one chapter unless the source itself does so.
- C–G are global. Each chapter may have a short “kilit terimler” box, but the
  full glossary lives in C.
- Deduplicate overlaps with cross-references:
  “PD0 §0.4’ün ayrıntısı PD4 §4.2’de; burada yalnızca hatırlatma var.”
- Overlapping topics still need coverage in both parts if the source presents
  them differently; explain the difference instead of copy-pasting.
- Keep a coverage matrix: source → section → guide page → status.

## 8. DESIGN SYSTEM — “PAPER”

This is the default visual language. Do not change it per chapter.

### Palette

| Role | Colour | Hex |
|---|---|---|
| Paper | warm cream | `#FAF6EC` |
| Ink | deep sepia-black | `#211E19` |
| Secondary text | sepia grey | `#5E574B` |
| Primary accent | brick red | `#8A3E2F` |
| Secondary accent | sage green | `#42594F` |
| Tertiary accent | antique gold | `#B08A4F` |
| Rule | linen | `#D8CEBD` |
| Code background | sand | `#F1EADB` |

Rules: no gradients, no stock photos, no rounded “card” UI, no neon colours.
Accent colours are for section numbers, rules, labels and small marks only.

### Typography

- Body: Noto Serif (fallback: Source Serif 4, Georgia), 9.5–10.2 pt,
  line-height 1.33–1.43.
- Headings: Inter (fallback: Helvetica/Arial), semi-bold.
- Code: DejaVu Sans Mono, 7.4–8.2 pt.
- Captions: serif italic, 7.9–8.4 pt.
- Labels: Inter, uppercase, letter-spacing 0.12–0.18em.

### Layout

- A4, margins 16–19 mm; single column; generous but not wasteful.
- Cover: cream, double thin rule, serif title, small-caps course line, simple
  geometry/syntax-tree motif; no photo.
- Running header: chapter title left, guide title right; thin rule.
- Footer: page number centred, brick red.
- TOC: right-aligned page numbers via `target-counter`; no noisy leaders.
- Section openers: top rule, small meta line, large title, short lead. Do not
  leave an opener alone at the bottom of a page.
- Boxes: definition = cream fill + brick left rule; critical = sage left rule;
  example = top/bottom rule; note = gold left rule. No shadows.
- Tables: no vertical lines; dark top/bottom rules; light header fill; zebra
  optional and very subtle.
- Figures: SVG, thin lines, labels in Inter; caption below with figure number.
- Code: sand background, gold left rule, wrap long lines, no clipping.

### Print / PDF

- Embed fonts; Turkish characters and hyphenation must work.
- No content relying on colour alone.
- PDF metadata: title, author/course, language `tr`.
- Build with Typst; do not use a screenshot of HTML.

## 9. AUTOMATION CONTRACT

Use the scripts; do not redo their work manually.

| Tool | Purpose |
|---|---|
| `folisift analyze` | page/heading/low-text inventory → JSON + Markdown |
| `folisift render-pages` | render low-text pages to PNG |
| `folisift build --check` | build a guide PDF and run the Markdown coverage gate |
| `folisift scaffold` | create a new guide bundle + config + source headings |
| `folisift mcp` | expose the same operations to an MCP client |
| `templates/paper.typ` | shared paper design system |
| `guides/<slug>/config.json` | per-guide paths and metadata |

### Config schema

```json
{
  "schema_version": 2,
  "slug": "pd0-pd1",
  "title": "…",
  "language": "tr",
  "content": "guide.md",
  "output": "../../output/PD0-PD1_Calisma_Ozeti.pdf",
  "sources": ["../../sources/PD0.pdf", "../../sources/PD1.pdf"],
  "inventory": "data/source.json",
  "coverage": "data/coverage.md"
}
```

### Source map schema

`guides/<slug>/data/source.json` contains `sources[]` with `filename`, `pages`,
`words`, `low_text_pages`, `headings[]`, `page_map[]`, plus `totals`.

### Coverage gate

`folisift build --check` must report a passing coverage gate before final
delivery. It checks top-level section numbers and fuzzy heading matches;
tables, figures, formulas and examples still require manual review in G.

### Performance rules (avoid the slow loop)

- Extract text **once**; reuse the JSON.
- Render **only** low-text pages; do not render 1,184 slides.
- Write chapter files separately; build at milestones.
- Fix content in the HTML fragment and rebuild; do not rewrite the whole guide.
- Do not hand-count pages, headings or low-text slides; use the scripts.
- Do not ask the user for decisions already fixed in CONFIG.
- If a build fails, read the first error; do not regenerate everything.

## 10. QUALITY GATE CHECKLIST

Before declaring completion:

- [ ] Every source PDF appears in B with its full page range.
- [ ] Every top-level section/subsection has a heading and explanation.
- [ ] `check_coverage.py` returns GEÇTİ.
- [ ] G counts match the source map exactly.
- [ ] Low-text pages are either explained or explicitly listed as unreadable.
- [ ] No invented facts/formulas/quotes/page refs.
- [ ] Explanatory additions are labelled **Açıklayıcı not**.
- [ ] Inconsistencies are flagged, not silently corrected.
- [ ] TOC page numbers and PDF bookmarks work.
- [ ] No orphan headings, clipped tables/code, or missing glyphs.
- [ ] Turkish is consistent; English terms are parenthetical on first use.
- [ ] No quizzes, flashcards, mock exams or generic study advice.
- [ ] Page count is within the configured budget.
- [ ] Final PDF, HTML/CSS, source map and coverage report are delivered.

## 11. OUTPUT CONTRACT

Deliver:

1. Final PDF at the configured path.
2. Editable Markdown content and the shared Typst template.
3. Source inventory JSON + Markdown.
4. Coverage report Markdown.
5. A short completion note: page count, covered slides/sections, low-text
   pages, unreadable content, known caveats.

Do not deliver only a chat response when a PDF was requested.

## 12. REUSE FOR A NEW COURSE

1. Create a new guide bundle with `folisift scaffold`, or copy
   `guides/<slug>/` as a starting point.
2. Keep `templates/paper.typ` shared; do not fork the design per chapter.
3. Run `folisift analyze` on the new PDFs.
4. Fill the outline and write B parts in source order.
5. Build with `folisift build --guide <slug> --check`.
6. Update G with the new source map counts.
7. Only then write the final completion note.

## 13. ANTI-PATTERNS (specifically forbidden)

- Writing 30 pages for the first PDF and 8 for the last one.
- Copy-pasting slide bullets without explaining the mechanism.
- Repeating the same definition in B, C and F.
- Padding with generic advice, motivational text or quizzes.
- Using a screenshot of a slide when a 10-line SVG would be clearer.
- Claiming “tüm slaytlar kapsandı” without running the audit.
- Silently changing the source’s claim because “the standard definition is
  different”; flag it instead.
- Adding formulas or historical dates from memory.
- Starting the build before the source inventory exists.

## 14. TURKISH TERMINOLOGY — QUICK CONSISTENCY LIST

Use these consistently; give the English term on first use.

| Turkish | English |
|---|---|
| programlama dili | programming language |
| sözdizimi | syntax |
| anlambilim | semantics |
| derleyici | compiler |
| yorumlayıcı | interpreter |
| derleme zamanı | compile time |
| çalışma zamanı | run time |
| ön uç / arka uç | front end / back end |
| ara kod | intermediate code |
| ayrıştırma ağacı | parse tree |
| bağlam-bağımsız gramer | context-free grammar (CFG) |
| belirsizlik | ambiguity |
| okunabilirlik | readability |
| yazılabilirlik | writability |
| güvenilirlik | reliability |
| maliyet | cost |
| ortogonallik | orthogonality |
| bağlama | binding |
| kapsam | scope |
| ömür | lifetime |
| parametre aktarımı | parameter passing |
| etkinlik kaydı | activation record |
| eşyordam | coroutine |
| yazılım geliştirme yaşam döngüsü | software development life cycle (SDLC) |

---

**Final instruction:** If the response/context limit is reached, stop at a
logical section boundary, state exactly what is covered and what remains, and
continue from the next unfinished section without repeating completed content.
For file-based work, prefer writing a chapter file and rebuilding over
regenerating the entire guide.
