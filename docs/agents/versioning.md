# Versioning and Release Authority

This file is the authority for build numbers, SemVer, guide schema versions and
release ownership. `AGENTS.md` only points here.

## Three independent layers

| Layer | Source | Owner |
|---|---|---|
| Build | `git rev-list --count HEAD`, embedded at compile time | derived |
| SemVer | annotated git tag `vX.Y.Z` | maintainer |
| Guide schema | `schema_version` in each guide config | content owner |

`FOLISIFT_BUILD` may override the build number for a reproducible build. A
binary must never read `.git` at runtime.

The MCP tool contract has its own `mcp_tool_schema_version` (currently `1`).
A breaking tool-schema change requires at least a MINOR SemVer bump.

## SemVer rules

- `MAJOR`: incompatible CLI, MCP tool schema or guide schema change.
- `MINOR`: new tool, new guide capability, new renderer feature.
- `PATCH`: bug fix, documentation, visual correction without contract change.
- `schema_version` is independent of SemVer. A guide schema migration is not a
  reason to bump the binary if the old schema remains accepted.
- Tags are annotated and pushed by the maintainer. Agents may prepare a release
  note and a local commit; they do not push branches or tags unless explicitly
  asked.
- A release that cannot produce a non-zero build number or a `.git`-free binary
  is not releasable.
- Shallow clones are not a release environment. CI must fail if the build
  number cannot be derived.

## Release checklist

1. `cargo test --workspace`
2. `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. `cargo fmt --all -- --check`
4. `cargo run -p folisift-cli -- build --guide pd0-pd1 --output output/PD0-PD1_v2.pdf --check`
5. Verify the sidecar `libpdfium.so` is next to the binary.
6. Verify `pdfinfo`: page count, title, author, PDF version and bookmarks.
7. Record the tag, build, commit, PDFium build and what changed in
   `docs/VERSIONS.md`.
8. Attach the platform archive (binary + PDFium sidecar) to the release.

## Commit rules

- Conventional Commits (`feat:`, `fix:`, `docs:`, `refactor:`, `test:`).
- One logical change per commit; do not mix a migration with a visual tweak.
- Do not commit `sources/*.pdf`.
- Do not hand-edit generated inventories or coverage reports.
