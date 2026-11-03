#!/usr/bin/env bash
# Build and freeze the five exam-ready PDFs, then print their SHA-256 hashes.
set -euo pipefail

cd "$(dirname "$0")/.."
BIN="target/debug/folisift"

cargo build -q -p folisift-cli

while read -r guide output pages; do
  "$BIN" build --guide "$guide" --check >/tmp/folisift-exam-build.json
  python3 - "$guide" <<'PY'
import json, sys
report = json.load(open("/tmp/folisift-exam-build.json"))
if not report.get("passed"):
    raise SystemExit(f"{sys.argv[1]}: build did not pass")
print(f"{sys.argv[1]}: {report['pages']} pages")
PY
  "$BIN" audit --pdf "$output" --guide "$guide" --expected-pages "$pages" --strict >/tmp/folisift-exam-audit.json
done <<'EOF'
pd0-pd1 output/PD0-PD1_Calisma_Ozeti.pdf 22
pd2 output/PD2_Calisma_Ozeti.pdf 24
pd3 output/PD3_Calisma_Ozeti.pdf 21
pd2-rev output/PD2_Revizyon.pdf 4
pd2-ko output/PD2_LISP_Bilgi_Organizatoru.pdf 1
EOF

python3 scripts/check-toc-links.py \
  output/PD0-PD1_Calisma_Ozeti.pdf \
  output/PD2_Calisma_Ozeti.pdf \
  output/PD3_Calisma_Ozeti.pdf
python3 scripts/check-reading-order.py \
  output/PD0-PD1_Calisma_Ozeti.pdf \
  output/PD2_Calisma_Ozeti.pdf \
  output/PD3_Calisma_Ozeti.pdf \
  output/PD2_Revizyon.pdf \
  output/PD2_LISP_Bilgi_Organizatoru.pdf
python3 scripts/check-component-contrast.py --strict \
  output/PD0-PD1_Calisma_Ozeti.pdf \
  output/PD2_Calisma_Ozeti.pdf \
  output/PD3_Calisma_Ozeti.pdf \
  output/PD2_Revizyon.pdf \
  output/PD2_LISP_Bilgi_Organizatoru.pdf
python3 scripts/check-production-report.py >/dev/null

sha256sum \
  output/PD0-PD1_Calisma_Ozeti.pdf \
  output/PD2_Calisma_Ozeti.pdf \
  output/PD3_Calisma_Ozeti.pdf \
  output/PD2_Revizyon.pdf \
  output/PD2_LISP_Bilgi_Organizatoru.pdf
