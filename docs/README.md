# Documentation

| Document | Purpose |
|---|---|
| [Architecture](architecture.md) | Engine/guide layers, build flow, config schema |
| [Authoring](authoring.md) | Practical runbook for producing a guide |
| [Authoring specification](authoring-spec.md) | Normative content, coverage and design rules |
| [Design system](design-system.md) | Paper palette, typography and components |
| [ADRs](adr/) | Architectural decisions and their consequences |
| [Agent configuration](agents/) | Issue tracker, triage labels, domain-doc rules |

## Where to start

- New contributor: `README.md` → `docs/architecture.md` → `docs/authoring.md`.
- New guide: `docs/authoring.md`.
- Design change: `docs/design-system.md` + an ADR if the change is structural.
- Agent/issue work: `AGENTS.md` and `docs/agents/`.

## Invariants

- Source PDFs are local; never commit `sources/*.pdf`.
- `python3 engine/build.py --guide guides/<slug> --check` must pass before a
  guide is called complete.
- Engine changes must not introduce a course name, source filename or slide
  count into `engine/`.
- Do not fork `engine/style.css` per guide.
