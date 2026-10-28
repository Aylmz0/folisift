#!/usr/bin/env bash
# Development-only page-map check for the paper style.
# Requires Poppler (pdfinfo, pdftotext); the product does not.
set -euo pipefail
unset LD_PRELOAD || true

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
pdf="${1:-$root/output/PD0-PD1_Calisma_Ozeti.pdf}"
expected_pages=22

command -v pdfinfo >/dev/null
command -v pdftotext >/dev/null
[[ -f "$pdf" ]] || { echo "missing PDF: $pdf" >&2; exit 1; }

pages="$(pdfinfo "$pdf" | awk '/^Pages:/ {print $2}')"
[[ "$pages" == "$expected_pages" ]] || {
  echo "page count $pages, expected $expected_pages" >&2
  exit 1
}

check_page() {
  local page="$1" needle="$2"
  local text
  text="$(pdftotext -f "$page" -l "$page" "$pdf" -)"
  if ! grep -qF "$needle" <<<"$text"; then
    echo "page $page is missing: $needle" >&2
    exit 1
  fi
}

check_page 4 "Hazırlık"
check_page 10 "Programlama dillerinin yazılım"
check_page 11 "Giriş"
check_page 17 "Programlama platformları"
check_page 18 "KARŞILAŞTIRMA"
check_page 19 "Yöntemler ve süreçler"
check_page 20 "Formüller, kurallar ve olgular"
check_page 21 "Bağlantılar ve karşılaştırmalar"
check_page 22 "Kapsam denetimi"

cargo run -q -p folisift-cli -- audit --pdf "$pdf" --expected-pages "$expected_pages"
echo "parity: $pages pages, page map and layout audit pass"
