# ADR-0011: Evidence-based visual audit and canonical coverage

- **Status:** Accepted (P0 implemented; P1 style work open)
- **Date:** 2026-10-30
- **Deciders:** Maintainer
- **Related:** ADR-0004, ADR-0010,
  `docs/plans/2026-10-29-guide-quality-visual-coverage-and-style-selection.md`,
  `docs/authoring-evidence.md`

## Context

The PD2/PD3 production run exposed three failures that a layout-only audit
could not see:

- the coverage gate saw 13/20 PD2 sections and 4/5 PD3 sections while the guide
  claimed 20/20 and 5/5;
- no low-text page had a persisted visual decision, yet G claimed all were
  visually inspected;
- the PDF contained underfilled section-tail pages that the audit did not
  report;
- D2 was installed and working, but the host hand-wrote SVGs instead of using
  the diagram contract.

The root cause is a contract gap, not a layout or D2 defect. A build must not
claim coverage or visual inspection without machine-checkable evidence.

## Decision

1. **The source's own topic list is the canonical coverage set.** The parser
   stores `SourceInventory.sections`; when present, coverage compares the guide
   against that list, not only against font-size-derived headings.
2. **Every low-text page needs one decision** in
   `guides/<slug>/visual-audit.json`: `redraw`, `table`, `text`, `decorative`
   or `unreadable`.
3. `redraw` requires an existing asset under the guide bundle; `text`,
   `decorative` and `unreadable` require a source-grounded reason.
4. `build --check` and MCP `build_guide check=true` fail on a missing or
   unresolved visual audit. `check_coverage` includes the visual result.
5. The layout audit reports `underfilled_pages` using body-character density;
   a build/audit with underfilled body pages fails. Reflow or `flow=true` for a compact chapter — not filler prose. A true
exception requires an explicit gate/ADR change, not a silent note.
6. D2 remains the only diagram engine. Manual SVG is allowed only as a
   registered exception; D2-compatible trees/flows/concepts use D2.
7. These gates are evidence requirements, not claims of learning efficacy.
   Product-level claims require the evaluation protocol in
   `docs/authoring-evidence.md`.

## Consequences

- `guides/<slug>/visual-audit.json` becomes a required host-owned artifact for
  guides with low-text pages.
- `SourceInventory` gains a backward-compatible `sections` field; cached
  inventories must be regenerated to get canonical coverage.
- A guide can no longer pass with “13/13 PASS” when the source topic list has
  20 sections.
- Underfilled pages become visible and must be fixed or explained.
- PD2/PD3 now have D2 diagrams and explicit visual decisions; the old manual
  `parse-tree.svg` and `operational.svg` are removed.
- The MCP response gains `visual` fields; this is additive to the tool
  contract, but the quality semantics are stricter.

## Alternatives considered

- **Trust the host's G prose.** Rejected: the PD2/PD3 run showed it can claim
  inspection without an artifact.
- **Only improve heading extraction.** Rejected: styled/wrapped headings are
  inherently lossy; the topic list is the source's own contract.
- **Make underfilled pages a warning only.** Rejected for production builds:
  the user sees the blank space; a warning does not fix the PDF. The normal
  audit now fails, and a justified exception must be explicit.
- **Keep manual SVG.** Rejected as the default because it bypasses D2
  determinism, manifests, tree-depth validation and caption/alt checks. It
  remains an exception only.

## Compliance

- `crates/folisift-parse/src/inventory.rs` — canonical sections.
- `crates/folisift-parse/src/coverage.rs` — canonical coverage set.
- `crates/folisift-parse/src/visual_audit.rs` — manifest and validation.
- `crates/folisift-parse/src/audit.rs` — underfilled pages.
- `crates/folisift-cli` and `crates/folisift-mcp` — build gates.
- `docs/authoring.md`, `docs/authoring-spec.md`,
  `docs/agents/opencode-guide.md` — authoring contract.
