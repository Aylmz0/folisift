#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/sources" "$tmp/guides/d2-pdf" "$tmp/output"
cp "$root/tests/fixtures/pdf/orphan-heading.pdf" "$tmp/sources/PD0.pdf"
cp "$root/tests/fixtures/d2-pdf/config.json" "$tmp/guides/d2-pdf/config.json"
cp "$root/tests/fixtures/d2-pdf/guide.md" "$tmp/guides/d2-pdf/guide.md"
cp -R "$root/tests/fixtures/d2-pdf/diagrams" "$tmp/guides/d2-pdf/diagrams"

binary="${FOLISIFT_BIN:-$root/target/debug/folisift}"
if [[ ! -x "$binary" ]]; then
  cargo build -q -p folisift-cli --manifest-path "$root/Cargo.toml"
  binary="$root/target/debug/folisift"
fi

if [[ -z "${FOLISIFT_D2:-}" ]]; then
  echo "check-d2-pdf: set FOLISIFT_D2 to a d2 v0.9.0 binary" >&2
  exit 2
fi

"$binary" diagram build --root "$tmp" --guide d2-pdf --all >/dev/null
"$binary" build --root "$tmp" --guide d2-pdf >/dev/null

if [[ "${UPDATE_D2_PDF_BASELINE:-0}" == "1" ]]; then
  cp "$tmp/output/d2-pdf.pdf" "$root/tests/baselines/d2/pdf-pilot.pdf"
  echo "updated tests/baselines/d2/pdf-pilot.pdf"
  exit 0
fi

cp "$root/tests/baselines/d2/pdf-pilot.pdf" "$tmp/baseline.pdf"
"$binary" visual compare \
  --root "$tmp" \
  --reference baseline.pdf \
  --current output/d2-pdf.pdf \
  --pages 2,3,4 \
  --out data/visual/d2/pdf-pilot \
  --threshold 1.0 --ratio-threshold 0.001 --fail-on-diff
