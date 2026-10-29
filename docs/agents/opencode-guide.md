# OpenCode guide production runbook

This is the host-side loop for OpenCode. The MCP server is the hands; OpenCode
is the brain. The host may run shell commands listed here, but the MCP server
itself never runs a shell, D2 or a network request.

## Preconditions

- `libpdfium` is available to the binary (`folisift doctor`).
- D2 **v0.9.0** is available for `folisift diagram build`
  (`FOLISIFT_D2`, next to the binary, or on `PATH`).
- The workspace has `sources/*.pdf` and a writable `guides/`.

## Loop

```text
1. workspace_info
2. doctor
3. prepare_guide (analyze once, scaffold, outline)
4. read_source / search_source in 10–20 page chunks
5. write guide.md with covers, audit and source refs
6. for each meaningful mechanism:
     diagram validate → diagram save
7. shell: folisift diagram build --guide <slug> --all
8. diagram status
9. build_guide check=true
10. folisift audit --pdf <output> --expected-pages <n>
11. make visual (when a baseline exists)
12. human review: accuracy, G audit, table drafts, visual quality
13. release only after approval
```

Do not skip the build/audit gates. Do not lower a threshold to make a run pass.

## Diagram rules

- Use a diagram for a process, hierarchy, relationship map or message flow.
- Do not use a diagram for a single fact, unrelated list or decoration.
- 0–3 diagrams per chapter; one idea per diagram.
- 1–12 nodes, max 20 edges, tree depth max 4.
- The host writes `diagrams/<id>.json`; it never writes D2, SVG or Typst.
- Every spec needs `title`, `caption`, `alt`, `source`.
- Reference the spec in `guide.md` as:

  ```markdown
  ::: diagram id="pipeline"
  :::
  ```

- After saving, run the returned CLI command. A build without the generated
  SVG is a hard error; there is no native fallback.

## Shell allowlist

Only these commands are needed in the production loop:

```bash
folisift diagram build --guide <slug> --id <id>
folisift diagram build --guide <slug> --all
folisift diagram status --guide <slug>
folisift build --guide <slug> --check
folisift audit --pdf <output> --expected-pages <n>
make visual
```

Do not call `d2`, `dot`, `python`, `node`, a browser or a network command
directly. The Folisift CLI owns D2 and its version check.

## Failure policy

| Failure | Action |
|---|---|
| D2 missing/version mismatch | stop; install/pin v0.9.0 or set `FOLISIFT_D2` |
| `diagram status` missing/stale | run the exact `diagram build` command |
| diagram too wide/tall | split the spec; do not shrink below 6.5 pt |
| coverage failure | fix the section; do not lower the threshold |
| visual diff | inspect the diff; update baseline only after review |
| source text ambiguous | inspect the page with `render_pages`; do not invent |
| table extraction draft | review against the source page before use |

Stop after two automatic retries on the same item and report the exact command
and error. Do not silently fall back to another diagram engine.

## Human gates

The following are never auto-approved:

- factual accuracy and source page references;
- the guide’s G/coverage-audit section;
- draft table cells;
- visual/pedagogical quality of figures;
- release tag and push.

## MCP surface

| Tool/resource | Use |
|---|---|
| `diagram` `validate` | validate a spec before saving |
| `diagram` `save` | write spec + generated D2 source |
| `diagram` `status` | check manifest/SVG freshness |
| `diagram` `render_plan` | get the exact CLI argv/command |
| `diagram` `list` | list specs and states |
| `folisift://diagram-schema` | JSON contract |
| `folisift://diagram-rules` | visual/caption rules |
| `write-diagram` prompt | short spec-writing prompt |
| `write-study-guide` prompt | full production prompt |
