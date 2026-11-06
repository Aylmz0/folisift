# MCP guidance contract

Folisift sends guidance to an agent in five layers. The server instruction is
short; detailed rules live in tool descriptions, schemas and on-demand
resources. Do not turn the instruction field into a second authoring manual.

| Layer | Format | Owner | Loaded |
|---|---|---|---|
| Server instruction | Short plain text, ≤600 characters | Server | Every session, if the client injects it |
| Tool descriptions + schemas | Natural language + JSON Schema | Server | On tool discovery |
| Tool results | `structuredContent` + serialized JSON text + concise summary | Server | On call |
| Prompts | Plain text workflow recipes | Server, invoked by user | On demand |
| Resources | Markdown / JSON Schema | Server, read by application | On demand |

## Server instruction

```text
Folisift turns source PDFs into Turkish study guides. Start with workspace_info, then prepare_guide.
The host writes guide.md; the server parses, renders and gates. Never edit sources/.
Read folisift://authoring-spec before authoring and folisift://design-system before layout changes.
Never claim coverage or a clean gate without build --check.
For the full workflow, use the start-here prompt or read folisift://capabilities.
```

## Prompt

`start-here` names the first three calls and the resources to read. The longer
`write-study-guide` and `write-diagram` prompts remain user-invoked recipes;
they are not a substitute for the server instruction or tool descriptions.

## Tool results

The four high-traffic tools (`coverage_manifest`, `guide_plan`, `qa_guide`,
`build_guide`) return:

1. a concise text summary;
2. the serialized JSON text block required for compatibility;
3. `structuredContent` with the exact fields.

Example:

```json
{
  "content": [
    {"type": "text", "text": "Coverage 138/138; 0 missing; next: build --check."},
    {"type": "text", "text": "{\"passed\":true,\"canonical_total\":138}"}
  ],
  "structuredContent": {"passed": true, "canonical_total": 138}
}
```

## Rules

- Tool descriptions say when to use a tool, what it returns, the next step and
  the actionable error.
- JSON Schema carries enums, ranges and required fields.
- The MCP server makes no LLM, network or shell call.
- Long normative prose belongs in `folisift://authoring-spec` or
  `folisift://design-system`, not in `instructions`.
- `folisift://capabilities` reports `guidance_version` and the current tool
  schema.
