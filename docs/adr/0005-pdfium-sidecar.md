# ADR-0005: Ship PDFium as a sidecar library

- **Status:** Accepted
- **Date:** 2026-10-26
- **Deciders:** Maintainer

## Context

`liteparse` uses PDFium through `libloading`. Native builds do not link PDFium
statically; the build script downloads a platform shared library and loads it
at runtime. A literal static binary is therefore not available without
replacing the parser or changing its loading model.

Phase 0 tested three shapes: binary alone (fails), binary plus
`libpdfium.so` next to it (works), and PDFium embedded in the binary and
extracted to a cache at startup (works, but adds an executable-write and AV
surface). The maintainer selected the sidecar shape.

## Decision

Ship each release as two files installed in the same directory:

```text
folisift
libpdfium.so        # libpdfium.dylib on macOS, pdfium.dll on Windows
```

- The runtime resolves PDFium next to the executable first.
- `FOLISIFT_PDFIUM_DIR` (or `PDFIUM_LIB_PATH`) may override the directory for
  development and testing.
- The binary fails with a clear, actionable error if the library is missing or
  cannot be loaded. It does not fall back to writing an extracted copy.
- Each release records the PDFium build and licence next to the binary.
- Packaging is per platform; there is no universal archive that mixes
  incompatible shared libraries.

## Consequences

- Installation is two files, not one. Installers and MCP configuration must
  keep them together.
- No runtime extraction, no executable cache, and no antivirus surprise.
- The PDFium version is explicit and can be upgraded independently of the
  binary as long as the ABI matches.
- A future static build remains possible only by changing the parser or
  upstream PDFium packaging; it is not a release requirement.
