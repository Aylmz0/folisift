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

Build the same archive locally with:

```bash
make package
# target/package/folisift-v<version>-linux-<arch>.tar.gz
# target/package/folisift-v<version>-linux-<arch>.tar.gz.sha256
```

## Platform status

| Platform | Binary | PDFium sidecar | Status |
|---|---|---|---|
| Linux x86_64 | archive | `libpdfium.so` | released |
| macOS arm64/x86_64 | source build | `libpdfium.dylib` | no archive yet |
| Windows x86_64 | source build | `pdfium.dll` | no archive yet |

The engine and CLI are platform-neutral; only the PDFium sidecar and archive
smoke test differ. A macOS/Windows archive is accepted only after
`folisift doctor`, a fixture build and `folisift audit` pass on that platform.

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

Non-loopback binds are refused unless a bearer token is supplied:

```bash
export FOLISIFT_HTTP_TOKEN="$(head -c 32 /dev/urandom | base64)"
folisift mcp --root /path/to/workspace --http 0.0.0.0:8000
# Every request must include: Authorization: Bearer $FOLISIFT_HTTP_TOKEN
```

OpenCode V2 configuration uses `mcp.servers`; see the project docs for the
local command example. Prefer loopback. The token is a single shared bearer
secret: there is no user account, rate limit or multi-user isolation yet.

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
