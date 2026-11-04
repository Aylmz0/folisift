# Final feedback v3 remediation plan — 2026-11-05

- **Status:** Active — M1/M2 implemented; A-V4 green; release pending
- **Source:** [`docs/FOLISIFT-GERI-BILDIRIM-v3.md`](../FOLISIFT-GERI-BILDIRIM-v3.md)
  (0.20.0, build 195, score 9.2/10)
- **Previous plans:**
  [`2026-11-04-production-feedback-v2-remediation.md`](2026-11-04-production-feedback-v2-remediation.md),
  [`2026-11-04-visual-evidence-and-chart-policy.md`](2026-11-04-visual-evidence-and-chart-policy.md)
- **Baseline:** Folisift 0.20.0, build 195, MCP schema 15, 37 tools,
  LiteParse 2.17, PDFium `chromium/8058`, D2 v0.9.0
- **Target:** `v0.21.0` for the P0/P1 tool contract, `v1.0.0` for
  determinism and the remaining P2 items. The report calls the first part a
  `0.20.x` hotfix; one `0.21.0` minor is cleaner because the fixes add response
  fields and change manifest selection.
- **Decision:** keep the deterministic/local boundary. No chart renderer, no
  reranker, no cloud call. Fix the tool contract before adding new surface.
  Do not ship an ambiguous `0.20.1` patch: all P0/P1 work lands in `0.21.0`.
- **Implementation note:** E1–E9 and the A-V4 harness are in the worktree;
  `make acceptance`, A-V2, A-V3 and A-V4 pass against the checkout. OCR is
  documented as advisory in ADR-0014; D2/graph limits are in the cookbook;
  determinism and topic-label provenance are in QA. Release/tag is pending.

## 0. Executive reading

The v3 report is mostly accurate. The 0.20.0 release closed the v2 coverage,
visual and model-discovery gaps; the remaining failures are concentrated in
four tool contracts:

1. generated coverage manifests still embed body text and miss recoverable
   canonical rows;
2. `qa` can treat an unadopted generated manifest as a hard gate;
3. `chapter_brief` returns a title, not a chapter;
4. brief and coverage-manifest responses do not honour one budget contract.

Two report claims need correction or refinement:

- `status:"excluded"` already exists in `CoverageStatus` and is treated as
  covered by `report()`; it is **not surfaced, counted or documented**. This
  plan finishes the feature rather than inventing it.
- `source_brief` reports `heading_count: 139` but `headings_total: 141` with
  `review_added: 2`. The two canonical headings are already recovered from the
  review manifest, but the counters and the summary array are inconsistent.
  The parser still misses them in the inventory.

The “combined > generated” statement in the report is also not the current
code. Actual precedence is **review → generated → combined → legacy**. This
plan deliberately changes it to **combined (adopted) → review → generated
(advisory) → legacy**, so `adopt` is meaningful and an edited combined file is
not shadowed by an older review file.

## 1. Verified evidence (0.20.0, fresh calls)

| ID | Check | Observed | Verdict |
|---|---|---|---|
| V3-01 | `coverage-manifest.generated.json` for `is-hukuku` | 138 rows; 15 headings longer than 120 chars; first rows contain body prose; `4.4.4` and `10.2` absent | **confirmed** |
| V3-02 | `coverage_manifest validate manifest=generated` | `total:138, claimed:17, missing:121, passed:false`; `missing` contains body prose | **confirmed** |
| V3-03 | `qa` when only generated exists | generated is loaded as a hard gate; failure writes `_draft.pdf` and preserves the output PDF | **confirmed behaviour; must become advisory/opt-in** |
| V3-04 | `chapter_brief heading=4 max_tokens=200` | `chars:41`, `summary` equals the title, `truncated:false`, no child body | **confirmed** |
| V3-05 | `section_brief 4.4.3 max_tokens=50` | `estimated_tokens:107`, `budget_tokens:50`, `truncated:true`, `next_offset:100` | **confirmed** |
| V3-06 | `heading_brief 4.4.3 max_tokens=20` | `summary:""`, `estimated_tokens:81`, `next_offset:0` | **confirmed** |
| V3-07 | `coverage_manifest validate` with `detail/max_tokens/fields` | full `per_source`/`missing` payload returned; no `truncated`, no `omitted`, `fields` ignored | **confirmed** |
| V3-08 | `coverage_manifest hint` with `max_tokens/fields` | full hints returned; no `truncated`; `fields` ignored | **confirmed** |
| V3-09 | `source_brief` `is-hukuku` | `heading_count:139`, `headings_total:141`, `review_added:2`, `headings_returned:141` while the summary array contains 4 rows + `{omitted:137}`; `4.4.4`/`10.2` are not in inventory headings | **partial; counters and summary contract wrong** |
| V3-10 | `hint` merged headings | `2.1–2.2` maps `2.1 → 0.95 covered`, `2.2 → 0.333 likely-missing`; same for `3.1–3.4` | **confirmed** |
| V3-11 | `status:"excluded"` | enum exists; `report()` treats it as covered; no `excluded` count, no review-tool output, no docs/tests | **claim partly false; finish feature** |
| V3-12 | IRAC scaffold | 10 cases and four `###` sections; placeholder citation `[PD0, s. 1]`; `source-headings.md` is written by scaffold code but not covered by acceptance | **placeholder confirmed; add regression** |
| V3-13 | `verify_citations` | only `references.first()` is verified; page ranges and multiple citations produce one snippet | **confirmed** |
| V3-14 | `section_brief.numbers` | empty at `max_tokens=50` because budget trimming clears the array | **confirmed budget artefact** |
| V3-15 | `weak_sections` | manifest path uses `MIN_SECTION_CHARS=400`; legacy path uses `MIN=400`/`WEAK=600`; neither is exposed in the report | **definition not documented** |
| V3-16 | `source_topics` cluster label | `term` is the first heading, not a semantic label | **confirmed; P2** |

## 2. Goals and non-goals

### Goals

1. A generated manifest is a clean, body-free candidate set derived from source
   structure; a reviewed manifest remains the human authority.
2. `qa` chooses one documented manifest source and never silently hard-gates an
   unadopted generated skeleton.
3. Briefs are actual briefs: chapter content is hierarchical, budgets are
   honest, and tiny budgets still return a useful minimum summary.
4. Every large MCP response obeys the same `detail`/`fields`/`max_tokens`
   contract, including coverage manifest actions.
5. `excluded` and merged headings are first-class, counted and auditable.
6. A reviewer can see why a section is weak and what threshold was applied.
7. IRAC scaffolds contain real source ids and source-headings output.
8. Remaining P2 work is explicit and does not block the 10/10 contract.

### Non-goals

- No chart renderer; visual evidence stays A+B.
- No reranker; the heading-term fallback remains.
- No OCR by default; `needs_ocr` stays advisory until a separate policy ADR.
- No LLM or network call inside the MCP server.
- No PDF-specific slide-grid heuristic.
- No automatic “truth” verdict for citations; only evidence and confidence.

## 3. Workstreams

### E1 — Generated manifest correctness and origin

**Findings:** V3-01, V3-02, V3-14.

**Change**

- Stop joining arbitrary TOC continuation lines into a title. A continuation
  is joined only when it looks like a title (short, no sentence terminator, no
  page-body marker), never when it is prose.
- Recover inline numbered headings from paragraph/list blocks when the pattern
  is unambiguous, e.g. `4.4.4. Asıl İşveren – Alt İşveren`. Emit a heading row
  and split the block text at that point. Do not recover bare references such as
  `bkz. 10.2`.
- Add `origin` to `ManifestRow` (`toc`, `heading`, `inline`, `review`, `manual`)
  so a generated row and a review-added row are distinguishable.
- Generated rows must not contain body prose. Reject a candidate title longer
  than a configurable title cap (default 160 chars) or containing sentence
  punctuation in the first 80 chars.
- Generated validation compares generated rows against the guide, not against
  the review file. `validate generated` must be self-consistent:
  `claimed == total`, `missing=[]`, `passed=true`.
- Review may add rows that the source never exposes as a heading (for example
  `10.2` if it is only a reference). Those rows carry `origin:"manual"` and are
  reported as `review_added`; they do not make generated validation fail.
- `canonical_total` is explicit in generated and review files:
  - generated: number of recoverable rows;
  - review: curated row count (for example 141).
- Keep `4.4.4` recoverable from the inline heading; `10.2` is review-added if
  the source has no heading block.

**Interface**

```text
ManifestRow {
  source, heading, normalized_heading, page,
  guide_heading?, guide_sections, status,
  origin: toc|heading|inline|review|manual,
  evidence?, confidence
}

coverage_manifest validate manifest=generated
→ { total, claimed, missing, passed, origin_counts, manifest_kind, ... }
```

**Acceptance:** A-V4-01, A-V4-02; generated keys contain no body prose and
generated validation is self-consistent. Review-added `10.2` is visible as
manual, not silently missing.

### E2 — Gate precedence, generated opt-in, draft contract

**Findings:** V3-03.

**Change**

- One shared manifest selector for CLI and MCP. Replace the duplicated
  `select_coverage_manifest` implementations.
- Precedence:
  1. explicit `manifest` argument;
  2. `data/coverage-manifest.json` (adopted combined);
  3. eligible `coverage-manifest.review.json`;
  4. `coverage-manifest.generated.json` only when explicitly selected or when
     `config.audit.manifest_gate="generated"` opts in;
  5. legacy heading gate with an explicit warning.
- Default generated handling is advisory:
  - `coverage_source="generated-advisory"` or `"generated-warn"`;
  - `coverage_warning` explains that the file is a skeleton;
  - `qa`/`build` do not fail on generated rows unless opted in.
- A generated file must never override combined. `adopt=true` still writes the
  review rows to combined.
- Document `_draft.pdf`:
  - a failed `--check` writes `output/<slug>_draft.pdf` and returns
    `draft_path`;
  - the committed output PDF is not overwritten;
  - the host may delete the draft after review;
  - generated v1.0 docs must show the cleanup step.
- Add `manifest_gate` to `AuditPolicy` (or the guide config) as a small enum:
  `legacy|generated|review|combined` with default `combined`.

**Acceptance:** A-V4-03; combined wins after adopt; generated-only guide still
builds with a warning; explicit generated gate fails with a draft path.

### E3 — Chapter brief and brief budget contract

**Findings:** V3-04, V3-05, V3-06.

**Change**

- Split brief resolution into three modes:
  - `section_brief`: one section, existing behaviour;
  - `heading_brief`: one heading, existing behaviour;
  - `chapter_brief`: parent heading + descendant sections, 300–600 content
    tokens by default.
- `chapter_brief` body includes:
  - parent heading and page range;
  - a short lead from the parent body, if present;
  - one bounded summary per immediate child;
  - a `sections` list with child heading, page range and content tokens.
- `source_sections` must not seed the section text with its heading. Strip the
  heading line before `summary`, `numbers` and `citations`; today the summary
  starts with `4. 4. 3. İşveren Vekili`.
- `numbers` and `citations` are extracted from body text only, not from the
  heading/title. `chapter_brief` must not report `4857` as a number merely
  because it appears in the title.
- Define one budget contract:
  - `max_tokens` is the **content** budget (summary + selected arrays), not the
    JSON envelope;
  - `estimated_tokens` is the content estimate and is guaranteed
    `<= budget_tokens`;
  - `response_tokens` is the full serialized estimate, reported separately;
  - `requested_tokens` and `budget_tokens` are explicit;
  - a minimum summary of 40 characters is kept at every budget;
  - `next_offset` is a source-character offset and is always `>=` the consumed
    summary length; `next_offset:0` while `truncated:true` is forbidden.
- Budget trimming order is deterministic:
  1. drop `table_rows`;
  2. drop `citations`;
  3. trim `numbers` to the first 4;
  4. shorten `summary` to the remaining budget;
  5. never return an empty summary when the section has text.
- Cache key includes the brief mode, query, budget and layout digest.

**Interface**

```text
chapter_brief { guide, source, heading, max_tokens? }
→ {
  heading, level:"chapter", page_start, page_end,
  summary, sections:[{heading,page_start,page_end,summary,estimated_tokens}],
  numbers, citations, chars, estimated_tokens, response_tokens,
  budget_tokens, requested_tokens, truncated, next_offset
}
```

**Acceptance:** A-V4-04, A-V4-05; chapter 4 has body and child summaries;
`section_brief 50` and `heading_brief 20` satisfy the content budget and never
return an empty summary.

### E4 — Coverage-manifest response envelope

**Findings:** V3-07, V3-08.

**Change**

- Apply `shape_response` to every `coverage_manifest` action: `build`,
  `validate`, `review`, `hint`, `read`.
- `validate` default `detail=summary`:
  - counts and `missing` names;
  - `per_source` array truncated with `{total, omitted}`;
  - `sections` arrays truncated per source;
  - `truncated`, `max_tokens`, `next` fields.
- `detail=full` returns the full report.
- `fields` supports dotted paths (`per_source.sections.number`) and projects
  nested objects/arrays. Top-level fields remain supported.
- `max_tokens` applies to the serialized response, with an explicit
  `truncated`/`omitted` envelope. Summarize arrays before shrinking strings.
- `hint` uses the same envelope and includes `merged_heading_hits`.
- `build` output includes `review_diff` counts and truncates the diff arrays;
  `build` must not dump full manifests.
- Add a shared `shape_response` unit test for nested `fields` and tiny budgets.

**Acceptance:** A-V4-06, A-V4-07; validate/hint with `fields` and `max_tokens`
return the requested projection and explicit omission counts.

### E5 — Excluded status, merged headings and hints

**Findings:** V3-10, V3-11.

**Change**

- Finish `excluded`:
  - `CoverageReport` adds `excluded: usize`;
  - `review` action returns `excluded` and `merged` counts;
  - `review_eligible()` remains true when only covered/merged/excluded rows
    remain;
  - `claimed` counts covered+merged; excluded is separate and never missing;
  - docs and tests use `covered|merged|excluded`.
- Add a shared heading-range parser:
  - `2.1–2.2`, `3.1–3.4`, `2.1-2.2`, comma lists and `covers` comments;
  - a source number inside the range maps to the merged guide heading with
    score 1.0;
  - `hint` uses this parser before fuzzy scoring;
  - `manifest.report` also accepts merged ranges for `guide_heading`
    verification.
- `hint` reports `merged_heading_hits` and never labels a covered row inside a
  merged range as `likely-missing`.
- `hint` reads the same selected manifest as `source_brief` (combined → review
  → generated) and marks review-only rows; it must not silently omit
  `4.4.4`/`10.2` when the manifest knows them.

**Acceptance:** A-V4-08, A-V4-09; `excluded` is counted; `2.2` inside
`2.1–2.2` is covered with score 1.0 in hint.

### E6 — Source heading recovery and `source_brief`

**Findings:** V3-09.

**Change**

- Feed the recovered inline headings from E1 into `SourceInventory.headings`.
- Recover missing TOC/manual headings from the selected manifest only for
  display; use the same combined → review → generated precedence as E2, not a
  stale review file. Mark them `review_added`.
- `source_brief` counters:
  - `inventory_heading_count`: parser headings only;
  - `review_heading_count`: review-only additions;
  - `headings_total = inventory + review`;
  - `headings_returned` equals the actual array length after pagination;
  - `truncated`/`next` reflect the returned array, not the pre-pagination total.
- `detail=summary` must not silently replace the headings array with 4 rows
  while claiming `headings_returned:141`. Either return the requested page of
  headings or report the omission explicitly.
- Add `heading_source` to each row: `inventory` or `review`.
- `4.4.4` and `10.2` are visible in the full list; if still absent, the response
  says `review_heading_count` and points to the manifest.

**Acceptance:** A-V4-10; `heading_count`, `headings_total`, returned length and
pagination agree; canonical headings are visible or explicitly review-only.

### E7 — IRAC scaffold and citation evidence

**Findings:** V3-12, V3-13.

**Change**

- IRAC scaffold:
  - use the real source id and first page in placeholder citations, e.g.
    `[is-hukuku, s. 1]`;
  - `sources: [is-hukuku]` stays;
  - `source-headings.md` is written and non-empty for every IRAC scaffold,
    including `prepare_guide` on an existing guide;
  - add a `--analyze`/`analyze:true` option or make analysis explicit when the
    source cache is missing;
  - no `PD0` placeholder remains anywhere.
- `verify_citations`:
  - verify each `ClaimRef` independently;
  - return `references: [{source,page,end_page,snippet,match_terms,confidence,
    verified,reason}]`;
  - keep a top-level aggregate `verified`, `confidence` and `snippet` for
    compatibility;
  - page ranges may return a per-page list;
  - budget the array with the shared envelope.

**Acceptance:** A-V4-11, A-V4-12; scaffold citations use the real source;
multi-reference claims expose one evidence row per page.

### E8 — Weak-section transparency and workflow docs

**Findings:** V3-15, V3-03 docs, V3-12 docs.

**Change**

- Define one weak-section contract for manifest and legacy coverage:
  - `empty`: `body_chars == 0`;
  - `weak`: `0 < body_chars < weak_section_chars` (default 400);
  - `healthy`: `body_chars >= weak_section_chars`;
  - `body_scope`: the guide heading or parent heading that supplied the body.
- Keep `MIN_SECTION_CHARS=400` as the default and allow a guide/config
  override; remove the silent 400/600 split. The current `3.x` rows at 398
  characters are weak by definition, not empty.
- `CoverageReport` exposes `weak_section_chars`, `empty_section_chars` and
  `body_scope` per row; `qa` and `validate` show the threshold.
- Docs:
  - review → validate → adopt → qa flow;
  - combined-first precedence;
  - generated advisory/opt-in;
  - `_draft.pdf` creation and cleanup;
  - weak-section threshold and parent-body scope;
  - merged heading syntax (`2.1–2.2`, comma/en-dash/range);
  - IRAC scaffold and `source-headings.md`.
- Add a regression that a 398-character parent-mapped section is `weak`, not
  `empty`, and that the response names `weak_section_chars:400`.

**Acceptance:** A-V4-13, A-V4-17.

### E9 — P2: determinism, OCR, D2, graph and topics

**Findings:** V3-16 and the v1 deferrals.

**Change**

- Determinism manifest: versions, source hashes, model signature, style pack,
  template hash, seed and build number in one `qa.determinism` object.
- OCR policy: keep `needs_ocr` advisory; add a separate opt-in plan/ADR for
  confidence and manual review. No OCR by default.
- D2: keep v0.9.0; document remaining limits (`decision` kind, 20+ nodes,
  vertical, `width_percent`). No new engine.
- Graph `path`/`explain`: apply the shared budget envelope and `fields`.
- `source_topics`: keep deterministic labels but add `label_source`; a
  semantic label is P2, not a 10/10 blocker.

**Acceptance:** A-V4-14..16; determinism/OCR/D2/graph/topics claims are
explicit and tested.

## 4. Data contracts

```text
ManifestRow {
  source, heading, normalized_heading, page,
  guide_heading?, guide_sections, status: covered|missing|merged|excluded,
  origin: toc|heading|inline|review|manual,
  evidence?, confidence
}

CoverageReport {
  passed, total, canonical_total, claimed, excluded, merged,
  gate_total, missing, missing_from_gate,
  empty_sections, weak_sections,
  min_section_chars, weak_section_chars,
  ...
}

BriefBudget {
  requested_tokens, budget_tokens, estimated_tokens, response_tokens,
  truncated, next_offset
}

ChapterBrief = BriefBudget + {
  heading, level:"chapter", page_start, page_end,
  summary, sections:[{heading,page_start,page_end,summary,estimated_tokens}]
}

SourceBrief = {
  inventory_heading_count, review_heading_count, headings_total,
  headings_returned, headings:[{heading,page,heading_source}],
  truncated, next
}
```

## 5. Milestones

### M1 — `v0.21.0` contract repair (P0/P1)

- E1 generated manifest correctness.
- E2 combined-first precedence, generated advisory/opt-in, draft docs.
- E3 chapter/brief budget contract.
- E4 response envelope.
- E5 excluded/merged/hint.
- E6 source_brief.
- E7 scaffold/citations.
- E8 weak-section transparency and workflow docs.

**Exit:** A-V4-01..13 and A-V4-17 pass; existing guides still pass; MCP schema
bump documented.

### M2 — `v1.0.0` confidence and determinism (P2)

- E9 determinism, OCR policy, D2/graph limits, source topics labels.
- Full A-V4 suite and v2/v3 acceptance remain green.

**Exit:** A-V4-14..16 pass; reranker and chart renderer remain explicit
non-goals.

## 6. Finding register

| ID | Finding | Workstream | Priority | Target |
|---|---|---|---|---|
| F-V4-1 | Generated manifest embeds body prose and misses canonical rows | E1/E6 | P0 | A-V4-01 |
| F-V4-2 | Generated manifest is a silent hard gate | E2 | P0 | A-V4-03 |
| F-V4-3 | `chapter_brief` is title-only | E3 | P0 | A-V4-04 |
| F-V4-4 | Brief budgets report above budget and can be empty | E3 | P0 | A-V4-05 |
| F-V4-5 | `coverage_manifest validate`/`hint` ignore `fields`/`max_tokens` | E4 | P0 | A-V4-06 |
| F-V4-6 | `source_brief` counters and summary pagination disagree | E6 | P1 | A-V4-10 |
| F-V4-7 | `excluded` exists but is not surfaced or documented | E5 | P1 | A-V4-08 |
| F-V4-8 | Hint misclassifies merged headings | E5 | P1 | A-V4-09 |
| F-V4-9 | `weak_sections` threshold/definition undocumented | E8 | P1 | A-V4-13 |
| F-V4-10 | IRAC scaffold hard-codes `PD0` | E7 | P1 | A-V4-11 |
| F-V4-11 | `verify_citations` checks only the first reference | E7 | P1 | A-V4-12 |
| F-V4-12 | `section_brief.numbers` silently cleared by budget | E3 | P1 | A-V4-05 |
| F-V4-13 | `source_topics` cluster labels are raw headings | E9 | P2 | A-V4-15 |
| F-V4-14 | Determinism manifest is still missing | E9 | P2 | A-V4-14 |
| F-V4-15 | OCR auto policy is not integrated | E9 | P2 | A-V4-16 |
| F-V4-16 | Graph `path`/`explain` budget parity | E9 | P2 | A-V4-16 |
| F-V4-17 | D2 limits are not documented in one place | E9 | P2 | A-V4-16 |
| F-V4-18 | Docs do not describe review/adopt/draft flow | E8 | P1 | A-V4-17 |

### 6.1 Feedback §6 item map

| Feedback item | Plan workstream | Release |
|---|---|---|
| P0.1 generated manifest | E1 | 0.21.0 |
| P0.2 qa generated gate | E2 | 0.21.0 |
| P0.3 chapter brief body | E3 | 0.21.0 |
| P0.4 validate budgeting | E4 | 0.21.0 |
| P0.5 brief budget invariant | E3 | 0.21.0 |
| P1.6 excluded status | E5 | 0.21.0 |
| P1.7 merged heading parser | E5 | 0.21.0 |
| P1.8 weak_sections definition | E8 | 0.21.0 |
| P1.9 source_brief canonical headings | E6 | 0.21.0 |
| P1.10 IRAC scaffold | E7 | 0.21.0 |
| P1.11 multi-reference citations | E7 | 0.21.0 |
| P1.12 hint merged/fields | E4/E5 | 0.21.0 |
| P2.13 determinism manifest | E9 | 1.0.0 |
| P2.14 D2 limits | E9 | 1.0.0 |
| P2.15 OCR policy | E9 | 1.0.0 |
| P2.16 graph path/explain | E9 | 1.0.0 |
| P2.17 workflow docs (limit docs in E9) | E8/E9 | 0.21.0 / 1.0.0 |

## 7. Acceptance suite

| ID | Test | Expected |
|---|---|---|
| A-V4-01 | Generated rows for `is-hukuku` | no body prose; title cap respected; `4.4.4` recovered inline; `10.2` visible as review/manual |
| A-V4-02 | `validate generated` | `claimed == total`, `missing=[]`, `passed:true`, `origin_counts` present |
| A-V4-03 | Manifest precedence | combined wins after adopt; review wins when eligible without combined; generated is advisory unless opted in; `is-hukuku` stays 141/141 and digital stays 41/41; failure writes `_draft.pdf` |
| A-V4-04 | `chapter_brief heading=4` | 300–600 content tokens, child summaries, body text, no title-only result |
| A-V4-05 | `section_brief 50` / `heading_brief 20` | `estimated_tokens <= budget_tokens`, non-empty summary, `next_offset` consistent |
| A-V4-06 | `validate` with `fields`/`max_tokens` | projection applied, `truncated`, `omitted`/`total`, no full dump |
| A-V4-07 | `hint` with `fields`/`max_tokens` | same envelope; merged hits and review-only rows visible |
| A-V4-08 | Excluded/merged counts | `excluded` counted; review eligible; no missing |
| A-V4-09 | Merged heading parser | `2.2` inside `2.1–2.2` is covered with score 1.0 |
| A-V4-10 | `source_brief` counters | `headings_returned == len(headings)`, `headings_total = inventory + review`, pagination honest |
| A-V4-11 | IRAC scaffold | real source id in citations, `source-headings.md` non-empty, no `PD0` placeholder |
| A-V4-12 | Multi-reference citations | one evidence row per reference/page; aggregate fields stable |
| A-V4-13 | Weak sections | the 398-character `3.x` rows are `weak`, not `empty`; `weak_section_chars:400` and `body_scope` are explicit |
| A-V4-14 | Determinism manifest | versions/hashes/model/template/seed/build in `qa.determinism` |
| A-V4-15 | `source_topics` label source | deterministic; labels documented; no false semantic claim |
| A-V4-16 | OCR/D2/graph limits | OCR off by default; graph path/explain budgeted; D2 limits documented |
| A-V4-17 | Docs and draft cleanup | review→validate→adopt→qa, combined precedence, draft steps, weak/merged syntax |

## 8. Risks and open questions

1. **Source recovery is heuristic.** Inline heading recovery can false-positive
   legal article numbers. Require a dotted multi-level number and a title-like
   suffix; mark recovered rows `origin:"inline"` and keep review authoritative.
2. **`10.2` may not exist as a heading.** If the source only references it, keep
   it review-only. Do not fabricate a canonical row to satisfy a number.
3. **Tiny budgets versus metadata.** `max_tokens=20` cannot contain the full
   JSON envelope. The contract therefore budgets content tokens and reports the
   envelope separately. This must be explicit in docs and tests.
4. **Combined-first precedence is a behaviour change.** Existing guides must
   still pass; run the full v2/v3 external suites before release.
5. **Chapter brief uses child summaries.** It is extractive, not generative; do
   not claim full chapter synthesis.
6. **Generated opt-in.** Existing generated files must not silently disappear;
   report them as advisory with a warning.
7. **Schema bump.** The P0/P1 work adds response fields and a guide-level
   manifest option; it ships as `0.21.0`, not an ambiguous patch. The MCP
   schema and release notes must be updated in the same commit.

## 9. Definition of done

- A-V4-01..17 pass; v2/v3 acceptance remains green.
- A generated manifest is body-free and self-consistent; review remains the
  human authority; combined-first precedence is documented and tested.
- `chapter_brief` returns a real hierarchical body; tiny briefs obey the
  content budget and never return an empty summary.
- `coverage_manifest validate`/`hint` honour `fields`, `max_tokens` and explicit
  omission counts.
- `excluded` and merged heading ranges are counted, reported and documented.
- `source_brief` counters, pagination and canonical headings agree.
- IRAC scaffold uses the real source id and writes `source-headings.md`.
- Multi-reference citations expose per-page evidence.
- Determinism, OCR, D2 and graph limits are documented; no hidden deferral.
- The only explicit deferrals remain the chart renderer and reranker.
