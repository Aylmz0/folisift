# Domain Docs

How the engineering skills consume this repository's domain documentation.

## Before exploring, read

- **`CONTEXT.md`** at the repo root.
- **`docs/adr/`** — read ADRs that touch the area you are about to change.
- **`docs/architecture.md`** for the workspace layers and data flow.

If a file does not exist, proceed silently. Do not flag its absence; create
documentation lazily when a term or decision actually needs recording.

## Layout

Single-context repository:

```
/
├── CONTEXT.md
├── docs/
│   ├── adr/
│   ├── plans/
│   ├── agents/
│   ├── architecture.md
│   ├── authoring.md
│   └── design-system.md
├── crates/             # Rust workspace: core, parse, render, mcp, cli
├── templates/          # paper.typ + embedded fonts
├── guides/
└── sources/            # local only, gitignored
```

There is no `CONTEXT-MAP.md` and no per-package context. If the engine later
splits into independent packages, add a root `CONTEXT-MAP.md` and revisit this
file.

## Vocabulary

Use the terms defined in `CONTEXT.md`: **guide**, **guide bundle**, **inventory**,
**low-text page**, **coverage report**, **engine**, **template**, **design
system**, **output**. Do not drift to “document”, “project”, “deck” or “theme”
when one of those precise terms is meant.

## ADR conflicts

If an output contradicts an existing ADR, surface it explicitly:

> _Contradicts ADR-0001 (engine/guide separation) — but worth reopening because…_
