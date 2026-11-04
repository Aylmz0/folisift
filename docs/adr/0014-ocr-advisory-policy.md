# ADR-0014: OCR stays advisory and opt-in

- **Status:** Accepted
- **Date:** 2026-11-05
- **Deciders:** Maintainer

## Context

`source_insights` already reports `needs_ocr`, low-text pages, text-layer
coverage and candidate pages. The v3 feedback asks for an OCR workflow, but
automatic OCR is expensive, non-deterministic across engines and can turn a
blurred chart label into an authoritative-looking number. The MCP boundary also
forbids hidden network or model calls.

## Decision

1. OCR is **off by default**. `ocr=true` or `--ocr` is an explicit host choice.
2. `needs_ocr` is evidence, never a gate. It identifies pages that need human
   inspection; it does not assert that OCR would be correct.
3. Full-PDF OCR is not part of the visual A+B contract. A source crop remains
   the fidelity path for charts and drawn tables.
4. Any future OCR integration must:
   - be local and deterministic;
   - report engine, language, confidence and page coverage;
   - keep manual review mandatory for numbers, units, dates and legal text;
   - never make an OCR-derived value authoritative without a source-page
     reference.
5. `qa.environment` and `qa.determinism` record that OCR/reranker are off unless
   explicitly enabled.

## Consequences

- Low-text pages stay visible through `visual_brief` and `visual_evidence`.
- Hosts can opt in per source; the default build remains reproducible.
- OCR quality work becomes a separate, testable project rather than an
  implicit side effect of parsing.
