# ADR-0009: Resolve every path through a workspace sandbox

- **Status:** Accepted
- **Date:** 2026-10-26
- **Deciders:** Maintainer

## Context

The MCP server can be called by an agent with arbitrary path strings. A tool
that accepts a free path could read or overwrite files outside the guide
workspace. The same server also needs a read-only source area and a small set
of writable output areas.

## Decision

- Every tool and CLI command opens a `Workspace` rooted at `--root`
  (`FOLISIFT_ROOT`).
- Paths are normalized lexically, canonicalized where possible, and rejected
  if they escape the root.
- `sources/` is read-only.
- Writes are allowed only under `guides/`, `output/` and `data/`.
- Symlinks that escape the root are rejected.
- The MCP server performs no shell execution and no network access. It does not
  call an LLM.

## Consequences

- A tool call cannot delete or read an arbitrary file on the machine.
- Every new tool must resolve its inputs and outputs through `Workspace`; do not
  accept `PathBuf` directly from a tool argument.
- Tests cover traversal, absolute paths outside the root, read-only sources and
  symlink escapes.
- The rules are enforced in code, not only documented in prompts.
