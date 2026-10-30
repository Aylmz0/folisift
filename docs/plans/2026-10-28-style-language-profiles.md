# Folisift v0.4 — Style, Language and Study-Profile Plan

- **Status:** v0.4.1 — S0–S7 landed; screenshots and signing remain for v0.5
- **Date:** 2026-10-28
- **Related:** `docs/plans/2026-10-28-roadmap.md`,
  `docs/plans/2026-10-28-visual-parity-and-production.md`,
  `docs/plans/2026-10-26-mcp-v0.2-plan.md`, ADR-0002, ADR-0007
- **Goal:** one engine, many standards. The default `paper` style reproduces the
  old PDF; other styles serve different audiences, languages, domains and study
  goals without forking the engine or the content model.

## 1. Why this plan exists

Folisift currently has **one aesthetic** (`paper`) and one output language
(`tr`). That is correct for the PD0–PD1 pilot, but not for an MCP product:

- a law student needs IRAC/case-brief structure, not A–G;
- a medical student needs SOAP, red flags and clinical cautions;
- a computer-science student may prefer a clean technical layout with code and
  complexity tables;
- an English-speaking user needs English labels, typography and citation rules;
- a business reader needs an executive brief, not a 22-page course reader;
- an accessibility user may need larger type and high contrast.

The server must not fork into N engines. The content model, coverage gate,
source handling and MCP flow stay shared. **Style changes presentation and
structure guidance; it never changes facts or coverage.**

## 2. Design principles

1. **Content and presentation are separate.** `guide.md` is the same semantic
   content regardless of style.
2. **A style is a versioned data pack, not a fork.** No guide may inject raw
   Typst or CSS.
3. **Default is stable.** `paper` is the parity style; new styles are marked
   `experimental` until they pass their gates.
4. **Coverage is style-independent.** Every source section must be mapped once,
   even when the style uses IMRAD, IRAC or SOAP.
5. **Language is an axis, not a style.** A Turkish paper guide and an English
   paper guide share the layout but not the labels or hyphenation.
6. **No fabricated standards.** Citation styles only apply when the metadata
   exists; page references remain the default traceability mechanism.
7. **Same content hash across styles.** A style must not silently delete or
   rewrite content to fit a layout.
8. **Accessibility is a gate, not a theme.** Contrast, font size, reading order
   and tagged PDF apply to every style.

## 3. Research: what different domains actually use

### 3.1 Professional structures

| Domain / genre | Common structure | What makes it good | Avoid |
|---|---|---|---|
| Research / lab | IMRAD (Introduction, Methods, Results, Discussion) | question → method → findings → meaning; reproducible methods | results mixed with discussion |
| Law | IRAC / CRAC / CREAC | issue, rule, application, conclusion; jurisdiction explicit | rule without facts or conclusion |
| Medicine / clinical | SOAP (Subjective, Objective, Assessment, Plan) | symptoms, findings, differential, plan; red flags | diagnosis without evidence/caveat |
| Business / policy | Executive brief / memo | BLUF, context, findings, implications, recommendation | burying the recommendation |
| Humanities | Thesis + argument + evidence | claim, textual evidence, counterargument, synthesis | chronology without argument |
| Literature review | Question → sources → methods → findings → gaps | synthesis across sources, not source-by-source summary | annotated bibliography disguised as synthesis |
| Technical docs | Task → prerequisites → steps → result | testable steps, examples, error cases | marketing prose |
| Language learning | Vocabulary/grammar → examples → usage → exceptions | examples, contrast, frequency | rules without usage |

Sources:

- IMRAD: <https://writingcenter.gmu.edu/writing-resources/imrad/writing-an-imrad-report>
- IRAC/CRAC/CREAC: <https://www.law.columbia.edu/sites/default/files/2024-10/WC%20Handout%20IRAC%2C%20CRAC%2C%20CREAC.revised%209.24.pdf>
- Professional genres and memos:
  <https://owl.purdue.edu/owl/subject_specific_writing/professional_technical_writing/index.html>
- Literature review: <https://owl.purdue.edu/owl/research_and_citation/conducting_research/writing_a_literature_review.html>
- Plain language: <https://www.plainlanguage.gov/guidelines/>
- Google developer documentation style:
  <https://developers.google.com/style>
- Microsoft Writing Style Guide:
  <https://learn.microsoft.com/en-us/style-guide/>
- APA: <https://apastyle.apa.org/>
- IEEE: <https://ieeeauthorcenter.ieee.org/>
- Chicago: <https://www.chicagomanualofstyle.org/>
- OSCOLA: <https://www.law.ox.ac.uk/oscola>

### 3.2 Study principles

Six strategies with strong evidence:

1. **Spaced practice** — distribute study over time.
2. **Interleaving** — mix related problem types.
3. **Retrieval practice** — recall before rereading.
4. **Elaboration** — explain why/how and connect to prior knowledge.
5. **Concrete examples** — instantiate abstract principles.
6. **Dual coding** — pair verbal explanation with meaningful visuals.

Sources:

- “Teaching the science of learning”:
  <https://pmc.ncbi.nlm.nih.gov/articles/PMC5780548/>
- “Considerations for Applying Six Strategies…”:
  <https://pmc.ncbi.nlm.nih.gov/articles/PMC8368916/>
- Worked examples / cognitive load:
  <https://www.mdpi.com/2227-7102/14/8/813>

Implications for a **static PDF summary**:

- We cannot enforce spacing or retrieval in a PDF, but we can design for them:
  - concise segments, clear headings, “self-check” prompts optional;
  - worked examples for problem-solving domains;
  - concrete examples and dual coding for abstract concepts;
  - elaboration via “why/how/compare” notes, not generic advice.
- Do not add quizzes/flashcards by default; the product is an exam-oriented
  summary, not a learning-management system.
- Cognitive load: segment long sections, signal structure, avoid decorative
  images, keep tables readable.

### 3.3 Community/MCP context

The v0.2 plan already covers PDF/MCP context lessons. Style work adds:

- Style previews must be cheap: render one sample page, not a full guide.
- Tool schemas are context tax: do not expose 20 style tools; expose one
  `style` tool with `list` / `info` / `preview` actions.
- The host LLM writes prose; the server only validates structure and renders.

## 4. Axes, not one “theme”

A guide has independent axes:

| Axis | Values (initial) | Default |
|---|---|---|
| `aesthetic` | `paper`, `clean`, `journal`, `brief`, `clinical`, `casebook`, `accessible` | `paper` |
| `structure` | `a-g`, `imrad`, `irac`, `soap`, `formula-sheet`, `literature-matrix`, `lecture-notes`, `executive-brief` | `a-g` |
| `language` | `tr`, `en`; more later | `tr` |
| `source_language` | `tr`, `en`, `mixed` | source-derived |
| `voice` | `academic`, `technical`, `plain`, `legal`, `clinical`, `business` | `academic` |
| `study_mode` | `exam`, `deep`, `reference`, `teach` | `exam` |
| `domain` | `cs`, `math`, `medicine`, `law`, `business`, `humanities`, `language`, `general` | `general` |
| `citation` | `page-ref`, `apa`, `ieee`, `chicago`, `oscola` | `page-ref` |

Not every combination is valid. The registry declares compatibility and
fallbacks. Example: `soap` + `humanities` is rejected; `irac` + `medicine` is
rejected.

## 5. Guide config

```json
{
  "schema_version": 3,
  "slug": "pd0-pd1",
  "title": "Programlama Dilleri — Çalışma Kılavuzu",
  "content": "guide.md",
  "output": "../../output/PD0-PD1_Calisma_Ozeti.pdf",
  "sources": ["../../sources/PD0.pdf", "../../sources/PD1.pdf"],
  "style": {
    "aesthetic": "paper",
    "structure": "a-g",
    "language": "tr",
    "source_language": "tr",
    "voice": "academic",
    "study_mode": "exam",
    "domain": "cs",
    "citation": "page-ref"
  },
  "inventory": "data/source.json",
  "coverage": "data/coverage.md",
  "target_pages": "18-23"
}
```

Compatibility rule: the old flat fields (`language`, `theme`) are accepted as
aliases during migration. `schema_version` 2 remains readable; new guides are
schema 3.

## 6. Style pack contract

```text
styles/<id>/
├── style.json          # metadata, compatibility, layout family
├── tokens.json         # palette, type, spacing, rules, density
├── strings/
│   ├── tr.json
│   └── en.json
└── preview.md          # canonical preview content
```

`style.json`:

```json
{
  "schema_version": 1,
  "id": "paper",
  "name": "Paper exam reader",
  "status": "stable",
  "layout": "paper",
  "aesthetic": "paper",
  "supported_structures": ["a-g", "lecture-notes"],
  "supported_languages": ["tr", "en"],
  "supported_domains": ["general", "cs", "math", "humanities"],
  "supported_voices": ["academic", "technical", "plain"],
  "tokens": "tokens.json",
  "strings": ["tr", "en"],
  "engine_min": "0.4.0",
  "page_target": "22",
  "preview": "preview.md"
}
```

Rules:

- `layout` names a **layout family implemented by the engine**, not arbitrary
  Typst from a user. This keeps consistency and security.
- `tokens.json` may change colors, type scale, margins, table/box styles, cover
  mode, density.
- `strings/*.json` owns every visible label: cover, TOC, box labels, audit,
  page labels. No Turkish labels in an English style.
- `preview.md` is a small canonical document used by the `style` preview action and visual
  snapshots.
- Style packs are versioned independently; `engine_min` guards compatibility.
- No style may remove the audit section or source refs.

## 7. Layout families

Initial families (code, not arbitrary Typst):

| Family | Character | Use |
|---|---|---|
| `paper` | serif body, cream, rules, boxes | default; old PDF parity |
| `clean` | sans body, white, tighter grid, code blocks | technical/CS |
| `journal` | serif, two-column optional, citation emphasis | academic/humanities |
| `brief` | one-page/short, BLUF, strong summary tables | business/policy |
| `clinical` | high-signal tables, red-flag boxes, SOAP | medicine |
| `casebook` | IRAC blocks, wide margins, case tables | law |
| `accessible` | 12 pt+, high contrast, no color-only meaning | accessibility |

The engine owns these layouts. A style pack selects one and supplies tokens.
Later, a trusted plugin API can add layouts; not in v0.4.

## 8. Structures and coverage mapping

A–G is one structure. Other structures must still cover every source section.

Add a content-level coverage marker:

```markdown
<!-- covers: PD0:0.1, PD0:0.2 -->
## Foundations

...
```

or a directive:

```markdown
::: section covers="PD0:0.1, PD0:0.2"
## Foundations
:::
```

Rules:

- `coordinator` (coverage tool) reads `covers` first; if absent, falls back to
  `## <number> <title>` headings.
- Every source subsection must be claimed exactly once.
- Unknown source id or section number → build error.
- A style cannot omit the audit section; its label is localized.
- `section` metadata is style-neutral; it survives style changes.

Structure templates:

| Structure | Required sections |
|---|---|
| `a-g` | A overview, B by source, C concepts, D methods, E facts, F connections, G audit |
| `imrad` | Introduction, Methods, Results, Discussion, Audit |
| `irac` | Issue, Rule, Application, Conclusion (per case), Audit |
| `soap` | Subjective, Objective, Assessment, Plan, Audit |
| `formula-sheet` | Definitions, Formulas, Units/Conditions, Examples, Audit |
| `literature-matrix` | Question, Sources, Methods, Findings, Gaps, Audit |
| `lecture-notes` | Agenda, Concepts, Examples, Summary, Audit |
| `executive-brief` | Context, Key findings, Implications, Recommendation, Audit |

## 9. Domain profiles

A domain profile is a prompt/validation pack, not a layout:

| Domain | Emphasis | Required checks |
|---|---|---|
| `cs` | algorithms, complexity, code, traces, architecture | code compiles mentally; complexity stated; edge cases |
| `math` | definitions, theorems, derivations, units | notation consistent; assumptions; no missing steps |
| `medicine` | clinical reasoning, red flags, guidelines | caveats; no diagnosis without evidence; source years |
| `law` | jurisdiction, rule, application | IRAC complete; citations/jurisdiction; no outcome invention |
| `business` | decision context, frameworks, numbers | assumptions; recommendation; sensitivity |
| `humanities` | thesis, evidence, counterargument | claims tied to text; chronology not a substitute for argument |
| `language` | vocabulary, grammar, usage | examples; exceptions; frequency/register |

Profiles supply:

- prompt fragment;
- preferred structure defaults;
- terminology table;
- table/figure conventions;
- forbidden claims / cautions;
- validation checklist.

The server validates structure and references; the host LLM judges content.
No automatic “medical advice” or “legal advice” claims.

## 10. Language and localization

- `language` controls:
  - UI strings;
  - hyphenation and line breaking;
  - date/number formats;
  - font fallback;
  - TOC/audit labels.
- `source_language` records the input; output may differ.
- The server does not translate. The host LLM writes; the server validates
  headings, refs and structure.
- Terminology policy:
  - preserve source term on first use;
  - provide target-language equivalent;
  - maintain a per-guide glossary;
  - never silently replace a technical term.
- Initial languages: `tr`, `en`. Add `de`, `fr`, `ar` only after font/RTL
  tests.
- English style check: no Turkish labels (`Tanım`, `Kritik ayrım`, `Kapsam
  denetimi`) unless the guide language is Turkish.
- RTL is out of scope for v0.4; it needs bidi layout, fonts and table tests.

## 11. Voice and citation standards

| Voice | Rules |
|---|---|
| `academic` | evidence, hedging, definitions, citations, no chatty tone |
| `technical` | imperative steps, prerequisites, examples, error cases |
| `plain` | short sentences, active voice, common words, no jargon without definition |
| `legal` | issue/rule/application/conclusion; jurisdiction; precise terms |
| `clinical` | SOAP; red flags; uncertainty; guideline years |
| `business` | BLUF; findings; implications; recommendation; assumptions |

Citation:

- `page-ref` default: `[PD0, s. 3]`; always traceable.
- `apa`, `ieee`, `chicago`, `oscola` only when bibliographic metadata exists.
- No fabricated authors, years, DOIs or page numbers.
- A style may change the visual form of a reference, not its truth.

## 12. Study modes

| Mode | Content strategy | Avoid |
|---|---|---|
| `exam` | high-yield definitions, comparisons, common mistakes, worked examples, source refs | long historical digressions |
| `deep` | mechanisms, derivations, assumptions, limitations, connections | bullet-only summaries |
| `reference` | definitions, formulas, tables, units, index | prose padding |
| `teach` | lead, concrete examples, visual explanation, optional self-check | quizzes by default |

Research-backed features:

- worked examples for problem-solving domains;
- concrete examples for abstract ideas;
- dual coding: meaningful figures with captions, not decoration;
- elaboration: “why/how/compare” notes;
- optional retrieval prompts, off by default;
- segmenting and signaling to manage cognitive load.

No style adds spaced repetition scheduling or flashcards unless the user asks
for a study-aid profile; those are separate products, not the default summary.

## 13. Style tooling

### CLI

```bash
folisift styles list
folisift styles info paper
folisift styles preview paper --language tr --structure a-g
folisift scaffold --slug pd2 --style clean --domain cs --structure imrad --language en
folisift guide-status pd2
```

### MCP tools

| Tool | Purpose |
|---|---|
| `style` | `action: list \| info \| preview`; ids, tokens, compatibility, sample PDF/PNG |
| `prepare_guide` | accepts style/structure/language/domain; one-call prepare |
| `scaffold_guide` | accepts style/structure/language/domain; granular fallback |
| `build_guide` | uses config style; no style override in build |
| `guide_status` | reports style and compatibility |

Resources: `folisift://styles/{id}`, `folisift://styles/{id}/strings/{lang}`.

Do not expose 20 style tools. One `style` tool with three actions is enough:
v0.2 has 10 core tools; v0.3 adds `prepare_guide` (11); v0.4 adds `style`
(12 total).

## 13.1 Caching and no repeated work

- Source text/inventory cache from v0.2 is reused; style does not reparse.
- `prepare_guide` is the default entry point; granular tools exist for recovery.
- `style` preview renders one sample page, not a full guide.
- `build_guide` returns `content_hash`; unchanged content/style returns cache.
- A style change must not rewrite prose; it re-renders the same content with a
  different layout family/labels.
- Cross-style test builds reuse the same source cache.

## 14. Consistency gates

Every style build must pass:

1. **Content hash parity** — normalized prose hash equals the `paper` build,
   except localized labels and TOC/audit headings.
2. **Coverage parity** — same source sections claimed; 0 missing, 0 empty.
3. **Asset parity** — same 24 tables, 4 figures and directive counts.
4. **Reference parity** — every subsection has source refs; no invented refs.
5. **Localization lint** — no missing strings; no Turkish UI labels in `en`.
6. **Accessibility lint** — contrast, font size, tagged PDF, bookmarks.
7. **Visual snapshot** — selected pages per style; threshold review.
8. **Layout audit** — the same empty-page, overflow, orphan, table/figure/box
   split and bookmark checks from the visual parity plan.
9. **Compatibility** — style supports the chosen structure/language/domain.
10. **No raw Typst/CSS** in guide content.
11. **Deterministic build** — same input → same content hash/page structure.
12. **Page target** — every style declares a range; deviations are explained.

Cross-style fixture:

```text
tests/fixtures/style-sample.md
```

Build it in every stable style; compare text hash and coverage.

## 15. Style matrix (initial)

| Id | Aesthetic | Structure | Best for | Status |
|---|---|---|---|---|
| `paper` | paper | a-g, lecture | exam reader, old-PDF parity | stable |
| `clean` | clean | imrad, lecture | CS/engineering, code-heavy | experimental |
| `journal` | journal | imrad, literature-matrix | humanities/social science | experimental |
| `brief` | brief | executive-brief | business/policy | experimental |
| `clinical` | clinical | soap | medicine/health | experimental |
| `casebook` | casebook | irac | law | experimental |
| `formula` | clean | formula-sheet | math/engineering exam | experimental |
| `accessible` | accessible | any | accessibility/high contrast | experimental |

Only `paper` is stable until its visual parity gate passes. New styles cannot
be marked stable without snapshots and cross-style content parity.

## 16. Phases

### Phase S0 — Registry and schema — landed (1–2 days)

- `styles/` directory, `style.json`/`tokens.json`/`strings/`.
- Schema validation and compatibility matrix.
- `paper` manifest extracted from `templates/paper.typ`.
- Exit: registry loads; invalid style/config errors are line-numbered.

### Phase S1 — Style tooling — landed (1–2 days)

- `styles list/info/preview`; MCP `style` with `list`/`info`/`preview`.
- `scaffold` accepts style/structure/language/domain.
- `guide_status` shows style and compatibility.
- Exit: preview renders a small PDF for every style.

### Phase S2 — Paper parity (parallel with visual plan)

- Make `paper` pass the visual parity plan.
- Exit: 22-page old-PDF parity; `paper` becomes stable.

### Phase S3 — Second style (`clean`) — landed (2–3 days)

- Sans layout, code/table emphasis, IMRAD/lecture structure.
- Exit: content hash parity, snapshot review.

### Phase S4 — Domain structures — landed (3–4 days)

- `irac`, `soap`, `formula-sheet`, `literature-matrix`, `executive-brief`.
- Coverage `covers` metadata.
- Exit: a fixture source can be rendered in each structure with coverage pass.

### Phase S5 — English/localization — landed (1–2 days)

- English strings, hyphenation, date/number, fonts.
- `en` style preview and localization lint.
- Exit: no Turkish UI labels in English builds.

### Phase S6 — Accessibility/CI (1–2 days) — landed

- Contrast, tagged PDF, bookmarks, font size.
- Cross-style fixture and snapshots in CI.
- Exit: gates pass on Linux; macOS/Windows next.

### Phase S7 — Release v0.4.0 (1 day) — landed

- Archive, docs, tag, release notes.
- Exit: `styles list` shows stable/experimental; default unchanged.

## 17. Acceptance criteria

- [x] `paper` still reproduces old PDF (22-page visual plan gate).
- [x] `clean` builds the same content with a different aesthetic.
- [x] Every structure can map source sections via `covers` metadata.
- [x] English build has no Turkish UI labels.
- [x] `styles list/info/preview` and MCP `style` actions exist.
- [x] `prepare_guide` accepts the style profile and returns a style-aware plan.
- [x] Same content hash across styles except localized labels.
- [x] Coverage/table/figure/reference parity across paper and clean.
- [x] Accessibility gate passes for paper/clean: tagged PDF, bookmarks, min 6 pt,
  contrast >= 4.5:1 and body size >= 9 pt.
- [x] No raw Typst/CSS in guide content.
- [x] No style can remove the audit section.
- [x] Clean install + `doctor` works.

## 18. Risks

| Risk | Mitigation |
|---|---|
| Style proliferation becomes unmaintainable | Layout families + registry; stable/experimental; only stable shipped. |
| Content forks per style | Content hash parity; one `guide.md`; `covers` metadata. |
| Localization drift | Strings linter; no labels in templates. |
| Coverage breaks in IMRAD/IRAC | `covers` metadata; fallback to numbered headings. |
| Legal/medical claims | Domain cautions; no advice claims; source years. |
| Visual snapshots brittle | Selected semantic pages + threshold, not full pixel equality. |
| RTL scope creep | Deferred; explicit non-goal for v0.4. |
| Font licensing | Embedded OFL fonts; per-style licence manifest. |
| Tool schema bloat | One `style` tool with three actions; style data via resources. |

## 19. Open questions

1. Is `clean` the right second style, or should `accessible` come first?
2. Should `style` be a nested object or flat config fields?
3. Do domain profiles ship as JSON packs or as prompts only?
4. Is `covers` metadata syntax a comment or a directive?
5. Do we support `en` only, or `tr` + `en` in v0.4?
6. Should style previews be PDF, PNG or both?
7. Do we keep the old `paper` name or call it `paper-exam`?
8. Who approves a style as “stable”: maintainer only, or snapshot + review?

## 20. Definition of done

A user can say:

> “Bunu İngilizce, temiz teknik düzende, IMRAD olarak, bilgisayar bilimi
> öğrencisi için hazırla.”

and Folisift:

1. scaffolds a schema-3 guide with the requested style/structure/language;
2. reads bounded source chunks;
3. lets the host LLM write `guide.md`;
4. builds in `clean` with IMRAD labels;
5. maps every source section via `covers`;
6. passes coverage and content parity;
7. renders a style preview and a final PDF;
8. reports the style, structure, language and audit result.

The default `paper` style remains the old-PDF parity standard. Other styles are
additive, not replacements.
