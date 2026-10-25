# Phase 5 — Docs and Agent Contract Report

- **Date:** 2026-10-26
- **Status:** Complete
- **Plan:** [`2026-10-26-rust-mcp-typst.md`](2026-10-26-rust-mcp-typst.md)

## What changed

- `AGENTS.md` now states the one-sentence constraint, decision rules,
  invariants, working agreement and versioning authority. Commands cover both
  the Python reference and the Rust v2 workspace.
- `README.md` is English and documents the v2 layout, sidecar requirement and
  MCP entry point.
- `docs/architecture.md` describes the strangle, schema 1/2, generated
  artifacts, coverage model and test commands.
- `docs/authoring.md` is the v2 runbook: inventory, low-text review, Markdown
  directives, build/check, scaffold.
- `docs/design-system.md` points at `templates/paper.typ` and the embedded
  fonts; DejaVu is no longer the shipped code font.
- `docs/agents/domain.md` layout includes `crates/`, `templates/` and plans.
- `docs/plans/` reports for Phases 0, 3 and 4 and this file.
- ADR-0006 (Rust workspace), ADR-0007 (Typst), ADR-0008 (liteparse),
  ADR-0009 (workspace sandbox).
- `docs/agents/versioning.md` and `docs/VERSIONS.md`.

## Verification

- Local Markdown links checked; only two example `assets/x.svg` links inside
  code fences are intentionally not files.
- AGENTS commands exercised during Phases 2–4: `workspace-info`,
  `guide-status`, `analyze`, `render-pages`, `build --check`, `scaffold`,
  `mcp`, `cargo test`, `clippy`, `fmt`.
- Python reference build still passes (`22 pages`, coverage `13/13`).
- Rust v2 build still passes (`23 pages`, coverage `13/13`).

## Remaining docs work

- `docs/authoring-spec.md` still contains HTML/CSS-era details; the v3 note at
  the top scopes them to the Python reference.
- Phase 6 will remove the Python reference and then simplify the docs again.
- CI is not configured yet; the Phase 1 gate remains open.
