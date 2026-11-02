#!/usr/bin/env bash
# Render every style preview in TR/EN and enforce its minimum font/audit gate.
set -euo pipefail

cd "$(dirname "$0")/.."
BIN="target/debug/folisift"
cargo build -q -p folisift-cli

fail=0
for style in paper reader clean journal brief accessible revision notes workbook reference; do
  case "$style" in
    paper) min=6.5 ;;
    revision) min=7.0 ;;
    accessible) min=9.0 ;;
    *) min=8.0 ;;
  esac
  for lang in tr en; do
    "$BIN" styles preview "$style" --language "$lang" \
      --out "output/style-preview-$style-$lang.pdf" >/dev/null
    "$BIN" audit --pdf "output/style-preview-$style-$lang.pdf" --strict >/tmp/folisift-style-audit.json
    python3 - "$style" "$lang" "$min" <<'PY'
import json, sys
style, lang, minimum = sys.argv[1], sys.argv[2], float(sys.argv[3])
report = json.load(open('/tmp/folisift-style-audit.json'))
if not report['passed']:
    raise SystemExit(f'{style}/{lang}: audit failed: {report["issues"]}')
if report['min_font_size'] < minimum:
    raise SystemExit(f'{style}/{lang}: min font {report["min_font_size"]:.1f} < {minimum}')
print(f'{style}/{lang}: {report["pages"]} pages, min {report["min_font_size"]:.1f} pt')
PY
  done
done
