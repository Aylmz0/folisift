# Documentation

| Document | Purpose |
|---|---|
| [Architecture](architecture.md) | Workspace layers, build flow, config schema |
| [Authoring](authoring.md) | Practical runbook for producing a guide |
| [Authoring specification](authoring-spec.md) | Normative content, coverage and design rules |
| [Design system](design-system.md) | Paper palette, typography and components |
| [Learning-science review](learning-science-and-genre-review-2026-11-02.md) | Evidence for exam summaries, concept summaries, readers and source visuals |
| [Font candidate review](font-candidate-review-2026-11-02.md) | Licence, glyph and package evidence for candidate pairings |
| [Code-graph tooling review](code-graph-tooling-review-2026-11-02.md) | Graphify and alternatives as developer tooling, not a runtime dependency |
| [Document-graph tooling review](document-graph-tooling-review-2026-11-02.md) | GraphRAG, LightRAG, Graphify and page-anchored document graphs for source PDFs |
| [Lightweight semantic linking](lightweight-semantic-linking-research-2026-11-03.md) | Small Rust/FOSS helper options for section-level semantic links |
| [Semantic link pilot](semantic-link-pilot-2026-11-03.md) | Twenty-pair review and threshold decision for review-only links |
| [Table parity](table-parity-2026-11-03.md) | Layout table blocks vs the legacy native-text extractor |
| [Accessibility review](accessibility-review-2026-11-03.md) | Tagged-PDF reading order, component contrast and the offline exam copy |
| [ADRs](adr/) | Architectural decisions and their consequences |
| [Plans](plans/) | Architecture, migration and remediation plans |
| [Source visual pipeline](plans/2026-11-05-source-visual-pipeline.md) | Proposed candidate, salience, D2/unique, crop/native and audit plan |
| [Source visual calibration](plans/2026-11-05-source-visual-calibration.md) | Manual ground truth from data-mining and PD2/PD3 pages |
| [ADR-0015](adr/0015-source-visual-pipeline.md) | Source visual candidates, relevance, D2/unique and local assets |
| [Production feedback remediation](plans/2026-11-03-production-feedback-remediation.md) | Completed remediation plan for coverage, layout, diagnostics, D2, MCP, context economics and engine depth |
| [Agent configuration](agents/) | Issue tracker, triage labels, domain-doc rules, versioning |
| [Versions](VERSIONS.md) | Released tags, builds and release notes |
| [Agent rules](../AGENTS.md) | Repo-specific operating rules and commands |
| [Domain context](../CONTEXT.md) | Ubiquitous language and invariants |

## Where to start

- New contributor: `README.md` → `docs/architecture.md` → `docs/authoring.md`.
- New guide: `docs/authoring.md`.
- Design change: `docs/design-system.md` + an ADR if the change is structural.
- Agent/issue work: `AGENTS.md`, `docs/agents/` and `docs/agents/versioning.md`.

## Invariants

- Source PDFs are local; never commit `sources/*.pdf`.
- `folisift build --guide <slug> --check` must pass before a guide is called
  complete.
- Engine changes must not introduce a course name, source filename or slide
  count into `crates/`.
- Do not fork `templates/paper.typ` per guide.
