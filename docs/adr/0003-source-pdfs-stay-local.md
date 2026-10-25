# ADR-0003: Source PDFs stay local

- **Status:** Accepted
- **Date:** 2026-10-25
- **Deciders:** Maintainer

## Context

The repository is private today but may become public later. The source PDFs
are course material with their own copyright and distribution terms. They are
also large and not needed to run the engine.

## Decision

- `sources/*.pdf` and `sources/**/*.pdf` are gitignored.
- `sources/README.md` explains where to place them locally.
- Generated guides live in `output/`; their publication is reviewed separately.
- Guide configs point at `../../sources/<name>.pdf`; no absolute paths.

## Consequences

- A fresh clone needs the PDFs added manually before building.
- The repository can be made public without exposing the source PDFs.
- The generated `output/` PDF still contains derived course content; review it
  separately before publishing.
- Do not add source PDFs “temporarily”; history rewrites are expensive.
