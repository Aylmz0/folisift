# ADR-0001: Separate the engine from guide bundles

- **Status:** Accepted
- **Date:** 2026-10-25
- **Deciders:** Maintainer

## Context

The first implementation mixed reusable code, design assets and the PD0–PD1
course content under a single `kitap/` directory. That made a second course
look like a fork of the first: paths, styles and generated data would be
duplicated, and the engine could drift per guide.

## Decision

Split the repository into three layers:

- `engine/` — domain-agnostic build code, tools, templates and design system.
- `guides/<slug>/` — one self-contained course/material bundle with `guide.html`,
  `config.json`, `assets/` and `data/`.
- `docs/` — architecture, authoring, design and ADRs.

A guide config uses paths relative to the guide directory. The engine contains
no course names, slide counts or source paths.

## Consequences

- A new course is a new `guides/<slug>/` directory, not a new engine.
- The engine can be tested against a minimal fixture guide.
- Shared changes (CSS, templates, coverage checks) reach every guide.
- A guide can be copied to another machine with its `sources/` inputs.
- Slight indirection: commands need `--guide`, and paths must be resolved
  relative to the guide config.
