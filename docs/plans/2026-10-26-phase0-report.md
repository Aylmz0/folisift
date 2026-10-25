# Phase 0 — Risk Spike Report

- **Date:** 2026-10-26
- **Status:** Complete — gate passed; two-file sidecar selected
- **Plan:** [`2026-10-26-rust-mcp-typst.md`](2026-10-26-rust-mcp-typst.md)
- **Spike:** throwaway crate at `/tmp/opencode/folisift-phase0` (not committed)

## Question

Can one Rust process parse source PDFs with `liteparse`, render the paper design
with Typst, and produce a Turkish PDF without Python, Poppler, WeasyPrint or a
system Typst installation?

A second question was implicit: how must the process be packaged so that a
clean machine needs no Python, Poppler, WeasyPrint or Typst installation?

## Environment

| Item | Value |
|---|---|
| OS | Linux x86_64 (glibc) |
| Rust | 1.98.1 |
| `liteparse` | 2.17.0, `default-features = false` (no Tesseract) |
| `typst` / `typst-pdf` | 0.15.1 |
| `rmcp` | 3.7.0 (metadata check only; not linked in the spike) |
| PDFium | `chromium/8058`, `libpdfium.so` 7.3 MB |
| Fonts | Noto Serif Regular, Noto Sans Mono (system files embedded in the spike) |

## Results

### Renderer — Typst

- One A4 page with the paper palette, Turkish headings/body, code and table.
- PDF: 20,922 bytes, PDF 1.7, tagged; `pdftotext` returns
  `ğüşıöç ĞÜŞİÖÇ` intact.
- Render time: ~0.06 s; peak RSS ~123 MB for a one-page document.
- A separate 3-page run with `#outline` produced:
  - a working table of contents;
  - PDF bookmarks: `1 Dil nedir?`, `1.1 Kritik ayrım`, `1.2 Kod`, `1.3 Tablo`;
  - `#set document(title: ..., author: ...)` populates PDF metadata.
- **Verdict:** Typst is viable for the paper design, TOC, bookmarks, metadata
  and Turkish glyphs.

### Parser — liteparse

`liteparse` uses PDFium through `libloading`; `liteparse-pdfium-sys` downloads
the shared library at build time and bakes its path into the binary. It does
**not** link PDFium statically on native targets. The runtime search order is:

1. `PDFIUM_LIB_PATH` (directory);
2. the compile-time cache path;
3. next to the running module;
4. next to the executable;
5. system loader paths.

Measured on the real sources:

| Source | Pages | Parse time | Words | Current inventory | Low-text pages |
|---|---:|---:|---:|---:|---:|
| `PD0.pdf` | 123 | ~117 ms | 5,386 | 5,334 | 40/40 exact |
| `PD1.pdf` | 137 | ~157 ms | 6,852 | 6,856 | 15/15 exact |

- Turkish characters were present in both documents.
- Low-text parity used the existing rule: fewer than 120 non-whitespace
  characters per page.
- A page screenshot of `PD0.pdf` page 7 rendered at 1500×1125 PNG (~245 KB).
- `is_complex` flagged 75/123 and 30/137 pages. That is a broader signal than
  the current low-text rule (it also flags sparse text and images). It should
  be used for routing/OCR, not as a drop-in replacement for `low_text_pages`.

### Packaging — sidecar selected

Three runs, each from a clean directory:

| Test | Result |
|---|---|
| Binary alone, PDFium cache removed | **Fails**: `failed to load pdfium shared library: LibraryUnavailable` |
| Binary + `libpdfium.so` in the same directory | **Works** |
| Binary with `libpdfium.so` embedded and extracted to a cache at startup | **Works**, but rejected as a product shape |

The maintainer selected the **sidecar** variant (B): the release archive
contains `folisift` and the platform PDFium shared library next to it. The
loader resolves the library next to the executable; there is no runtime
extraction, no executable written to a cache and no antivirus surprise.

Sizes:

| Build | Size |
|---|---:|
| Native binary, no PDFium payload, unstripped | 73 MB |
| Native binary, no PDFium payload, stripped | 56 MB |
| `libpdfium.so` sidecar | 7.3 MB |
| Embedded PDFium, stripped (rejected variant) | 63 MB |

`ldd` shows only glibc, `libgcc_s`, `libm`, `libdl`, `librt` and `libpthread`;
there is no link-time dependency on PDFium.

### Static/musl

Not verified. `cargo build --target x86_64-unknown-linux-musl` stopped because
`x86_64-linux-musl-gcc` is not installed. A true static build remains an open
experiment; it is not required for the one-file distribution test above.

## Gate verdict

**GO for the architecture, with the two-file sidecar.**

- Parser, renderer, Turkish text and sidecar distribution are feasible.
- “Single static binary” is **not** what liteparse provides. The selected
  target is **two files installed together**: `folisift` + `libpdfium.so`.
- The binary must fail with a clear message when the sidecar is missing or has
  the wrong platform/version.
- If a literal static binary becomes a hard requirement later, stop and revisit
  the parser choice rather than adding runtime extraction.

## Production sidecar check

The `folisift` binary was copied to a clean directory with `libpdfium.so` next
to it. `folisift analyze --source sources/PD0.pdf` produced 123 pages / 5,386
words. With the library and the build cache removed, the same command failed
cleanly:

```text
error: PDFium is not usable: pdfium shared library unavailable. Place
libpdfium.so next to the folisift executable or set FOLISIFT_PDFIUM_DIR.
```

No panic, no implicit extraction. The error is the intended installation
contract for the two-file release.

## Risks to carry into Phase 1

- The sidecar can be missing, stale, or from the wrong platform. Startup must
  resolve it next to the executable, verify it loads, and print an actionable
  error. Pin the PDFium build in the release notes.
- PDFium's licence and version must be recorded with the release.
- Binary size is ~56 MB stripped plus a 7.3 MB sidecar; this is acceptable for
  a desktop/MCP tool but must be measured again after `rmcp` and the real
  template are linked.
- `is_complex` over-flags compared with the current low-text rule; the
  inventory format needs both signals or an explicit mapping.
- The compile-time PDFium cache path leaks the build machine path into the
  binary. Runtime search must prefer the embedded extraction directory, not the
  baked path.
- `liteparse` pulls `reqwest`/`resvg` even with OCR disabled. The server must
  not expose them as network or conversion features by accident.

## Reproduction

The spike is intentionally outside the repository. Its shape:

```bash
cargo new folisift-phase0
cd folisift-phase0
# Cargo.toml: liteparse (default-features = false), typst, typst-pdf, tokio
cargo build --release
./target/release/folisift-phase0 render out.pdf
./target/release/folisift-phase0 parse /path/to/PD0.pdf
./target/release/folisift-phase0 shot /path/to/PD0.pdf 7 page7.png
```

The spike source is not a production artefact. Phase 1 starts from the
workspace in `crates/`; the production loader resolves `libpdfium.so` next to
the `folisift` executable.
