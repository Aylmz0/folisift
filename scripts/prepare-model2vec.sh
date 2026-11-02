#!/usr/bin/env bash
# Prepare the release semantic model once, on a maintainer machine.
set -euo pipefail

cd "$(dirname "$0")/.."
OUT="${FOLISIFT_MODEL_DIR:-models/potion-multilingual-128M-int8}"
mkdir -p "$OUT"

uv run --with model2vec python - "$OUT" <<'PY'
import sys
from model2vec import StaticModel

out = sys.argv[1]
model = StaticModel.from_pretrained(
    "minishlab/potion-multilingual-128M",
    quantize_to="int8",
)
model.save_pretrained(out)
print(f"wrote {out}")
PY

curl -fsSL https://raw.githubusercontent.com/MinishLab/model2vec/main/LICENSE \
  -o "${OUT}/LICENSE"

python3 - "$OUT" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

root = Path(sys.argv[1])
files = {}
for name in ["config.json", "model.safetensors", "tokenizer.json", "modules.json"]:
    path = root / name
    files[name] = {
        "bytes": path.stat().st_size,
        "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
    }
(root / "manifest.json").write_text(
    json.dumps(
        {
            "model": "minishlab/potion-multilingual-128M",
            "quantization": "int8",
            "files": files,
        },
        indent=2,
    )
    + "\n"
)
PY
