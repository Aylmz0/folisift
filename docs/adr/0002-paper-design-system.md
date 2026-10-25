# ADR-0002: Paper design system is a shared contract

- **Status:** Accepted
- **Date:** 2026-10-25
- **Deciders:** Maintainer

## Context

The guide is a study object, not a slide deck. An earlier attempt used a
navy/teal interface palette; the maintainer preferred a paper-like, academic
look. Inconsistent colours or typography per chapter would make the guide feel
like a collection of documents rather than one book.

## Decision

Use one paper design system for every guide:

- warm cream paper `#FAF6EC`, ink `#211E19`;
- brick `#8A3E2F`, sage `#42594F`, antique gold `#B08A4F`;
- Noto Serif body, Inter headings, Noto Sans Mono code;
- A4, single column, thin rules, no gradients, no stock photos, no rounded
  “card” UI;
- boxes are defined by rules/fills, not shadows;
- figures are simple SVG; tables have no vertical lines.

The design system lives in `templates/paper.typ` and `templates/fonts/`.

## Consequences

- Every guide looks like part of the same product.
- A design change is a single-file change, reviewed once.
- Guide-specific visual exceptions require an ADR or a clearly documented
  reason; they must not fork the whole stylesheet.
- The design is print-friendly and does not depend on colour alone.
