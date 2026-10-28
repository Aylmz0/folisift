#!/usr/bin/env bash
# Capture the golden reference PDF for visual review.
# Development-only: uses Poppler; the product does not.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
pdf="${1:-$root/output/old-python.pdf}"
out="${2:-$root/target/parity/reference}"

command -v pdfinfo >/dev/null
command -v pdftotext >/dev/null
command -v pdftoppm >/dev/null

mkdir -p "$out"
pdfinfo "$pdf" > "$out/pdfinfo.txt"
pdftotext -layout "$pdf" "$out/text.txt"
pdftoppm -png -r 150 "$pdf" "$out/page" >/dev/null 2>&1
sha256sum "$pdf" > "$out/sha256.txt"

echo "reference: $pdf"
echo "capture:   $out"
echo "pages:     $(pdfinfo "$pdf" | awk '/^Pages:/ {print $2}')"
