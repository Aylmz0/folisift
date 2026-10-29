# Install and run

## Linux release (recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/Aylmz0/folisift/main/scripts/install.sh | bash
folisift doctor
```

The installer downloads the latest Linux archive, verifies its SHA-256, and
installs under `~/.local/lib/folisift` with a link at `~/.local/bin/folisift`.
Set `FOLISIFT_PREFIX`, `FOLISIFT_VERSION` or `FOLISIFT_BASE_URL` to override
the defaults. Add `~/.local/bin` to `PATH` if the installer asks.

The archive contains:

```text
folisift              # CLI and MCP server
libpdfium.so          # source PDF/text extraction
d2                    # pinned v0.9.0, authoring only
D2-LICENSE.txt
THIRD_PARTY_NOTICES.md
styles/               # built-in style packs
domains/              # built-in domain profiles
templates/fonts/      # Inter and Noto fonts
```

A normal user does not need Rust, Python, Node, Graphviz or a separate D2
install. D2 is only used by `folisift diagram build`; `folisift build` embeds
committed SVG assets.

Manual archive verification:

```bash
tar -xzf folisift-v0.8.0-linux-x86_64.tar.gz
sha256sum -c folisift-v0.8.0-linux-x86_64.tar.gz.sha256
./folisift doctor
```

## OpenCode MCP

From the workspace root:

```bash
cd /path/to/workspace
opencode mcp add folisift -- folisift mcp
opencode mcp list
```

OpenCode V2 writes `mcp.servers` in the project config. The server inherits the
workspace as its `cwd`. The equivalent config is:

```jsonc
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "servers": {
      "folisift": {
        "type": "local",
        "command": ["folisift", "mcp"],
        "cwd": "/absolute/path/to/workspace"
      }
    },
  },
}
```

Use an absolute `command` path if `folisift` is not on `PATH`. Do not expose the
workspace to a server unless you trust the client. The MCP has no LLM call,
shell, network access or external D2 execution.

For a remote/HTTP client, use a loopback bind:

```bash
folisift mcp --root /path/to/workspace --http 127.0.0.1:8000
# endpoint: http://127.0.0.1:8000/mcp
```

Non-loopback binds require a bearer token:

```bash
export FOLISIFT_HTTP_TOKEN="$(head -c 32 /dev/urandom | base64)"
folisift mcp --root /path/to/workspace --http 0.0.0.0:8000
# Requests: Authorization: Bearer $FOLISIFT_HTTP_TOKEN
```

The HTTP token is a single shared secret; there is no user account, rate limit
or multi-user isolation yet.

## Source build

Requirements:

- Rust 1.92+
- PDFium for source parsing: `libpdfium.so`, `libpdfium.dylib` or `pdfium.dll`
- D2 v0.9.0 only for `diagram build`; `scripts/fetch-d2.sh` stages it

```bash
cargo build --release
cargo test
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Put the PDFium sidecar next to the binary or set `FOLISIFT_PDFIUM_DIR`. For a
development package:

```bash
make package
# target/package/folisift-v<version>-linux-<arch>.tar.gz
```

`make package` requires `FOLISIFT_D2` or a staged
`target/package/d2-v0.9.0/bin/d2`.

## First workspace

```text
sources/       # your PDFs; never committed
guides/        # guide bundles
output/        # generated PDFs
data/cache/    # generated cache
styles/        # optional overrides; built-ins are embedded
domains/       # optional overrides; built-ins are embedded
```

```bash
mkdir -p sources guides output
folisift doctor
folisift workspace-info
```

The quickest path is the MCP `prepare_guide` tool. It analyzes sources,
scaffolds the guide and returns the read/write plan in one call.

## Platform status

| Platform | Binary | PDFium sidecar | Status |
|---|---|---|---|
| Linux x86_64 | archive | `libpdfium.so` | released |
| macOS arm64/x86_64 | source build | `libpdfium.dylib` | no archive yet |
| Windows x86_64 | source build | `pdfium.dll` | no archive yet |

The engine is platform-neutral; only the sidecar and archive smoke test differ.
A macOS/Windows archive is accepted only after `folisift doctor`, a fixture
build and `folisift audit` pass on that platform.

## Troubleshooting

```bash
folisift doctor
```

It checks PDFium, cache/output paths, guides, fonts, structure packs and the
optional D2 sidecar. For MCP connection errors, run `opencode mcp list` and
check that the `cwd` is the workspace root.
