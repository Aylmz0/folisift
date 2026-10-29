# Install and run

## Requirements

- Rust 1.92+ only when building from source.
- A PDFium shared library for source parsing:
  - Linux: `libpdfium.so`
  - macOS: `libpdfium.dylib`
  - Windows: `pdfium.dll`
- The release archive contains the binary, the matching Linux PDFium library,
  `styles/` and `domains/`.

## Linux release archive

```bash
tar -xzf folisift-v0.5.0-linux-x86_64.tar.gz
sha256sum -c folisift-v0.5.0-linux-x86_64.tar.gz.sha256
./folisift doctor
```

`folisift` loads PDFium from its own directory first, then
`FOLISIFT_PDFIUM_DIR`, `PDFIUM_LIB_PATH`, and the executable directory.

Archives are **not code-signed**. Verify the published SHA-256 before use;
signing certificates are not part of the current release process.

## MCP

stdio (default):

```bash
folisift mcp --root /path/to/workspace
```

Streamable HTTP:

```bash
folisift mcp --root /path/to/workspace --http 127.0.0.1:8000
# endpoint: http://127.0.0.1:8000/mcp
```

OpenCode V2 configuration uses `mcp.servers`; see the project docs for the
local command example. Do not expose the HTTP endpoint to an untrusted network:
v0.5 has no authentication or multi-user isolation.

## Sources and guides

`sources/*.pdf` are local inputs and are never committed. A workspace needs:

```text
sources/       # your PDFs
guides/        # guide bundles
styles/        # optional overrides (built-ins are embedded)
domains/       # optional overrides (built-ins are embedded)
output/        # generated PDFs
data/cache/    # generated cache
```

Run `folisift doctor` before the first build, then
`folisift build --guide <slug> --check`.
