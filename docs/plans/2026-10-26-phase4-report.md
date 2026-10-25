# Phase 4 — MCP Server Report

- **Date:** 2026-10-26
- **Status:** Complete — MCP gate passed
- **Plan:** [`2026-10-26-rust-mcp-typst.md`](2026-10-26-rust-mcp-typst.md)

## Server

`folisift-mcp` uses `rmcp 3.7` over stdio. The CLI entry point is:

```bash
folisift mcp --root /path/to/workspace
```

`workspace_info` reports root, version, build, MCP tool schema version and
guide count. The negotiated MCP protocol in the probe was `2025-11-25`.

## Tools

| Tool | Purpose |
|---|---|
| `workspace_info` | root, version, build, guide count |
| `list_guides` | guide rows and validation status |
| `guide_status` | resolved config, content, output, sources |
| `analyze_source` | liteparse inventory JSON + Markdown |
| `render_pages` | PNG files for selected pages |
| `scaffold_guide` | schema-2 bundle with source section headings |
| `build_guide` | Typst PDF + optional coverage gate |
| `check_coverage` | Markdown coverage report |

## Resources and prompts

- `folisift://authoring-spec` — current authoring specification.
- `folisift://design-system` — current paper design system.
- `folisift://guide/{slug}` — guide config plus content.
- `write-study-guide` — the host-agent authoring prompt. It instructs the
  agent to use inventory headings, the four directives, source references and
  `build_guide(check=true)`.

## Gate evidence

A stdio probe completed:

1. `initialize` and `notifications/initialized`.
2. `tools/list` returned all eight tools.
3. `tools/call workspace_info` returned the workspace summary.
4. `analyze_source` read PD0 and wrote a 123-page inventory.
5. `scaffold_guide` created a temporary `mcp-test` bundle and generated
   headings from the source inventory.
6. `build_guide` with `check=true` rendered a 3-page PDF and returned
   coverage 5/5 PASS.
7. `check_coverage` returned 5/5 PASS with matched headings.
8. `resources/list`, `resources/read`, `prompts/list` and `prompts/get`
   returned the expected payloads.

The temporary guide and its output were removed after the probe. The server
uses the same workspace sandbox as the CLI: sources are read-only, writes stay
under `guides/`, `output/` and `data/`.

## Follow-ups

- MCP itself has no unit/integration test yet; the probe is manual. Add a
  stdio client test before the release gate.
- The protocol version negotiated by `rmcp` is whatever the client and SDK
  agree on; tool schemas are versioned separately with
  `mcp_tool_schema_version`.
- HTTP transport and release signing remain open questions from §16.

## Next

Phase 5: English docs, `AGENTS.md` working agreement, ADRs, versioning
authority and `docs/VERSIONS.md`.
