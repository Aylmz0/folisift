#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

json=()
record() {
  json+=("{\"check\":\"$1\",\"ok\":$2,\"detail\":\"$3\"}")
}

run() {
  local name="$1"
  shift
  if "$@" >"/tmp/folisift-acceptance-$name.log" 2>&1; then
    record "$name" true "pass"
  else
    record "$name" false "see /tmp/folisift-acceptance-$name.log"
    return 1
  fi
}

run tests cargo test --workspace
run fmt cargo fmt --all -- --check
run clippy cargo clippy --all-targets --all-features -- -D warnings
run exam make exam-check
run visual make visual
run type-check make type-check
run parity make parity
run genre make genre-check

if [[ -n "${FOLISIFT_D2:-}" ]]; then
  run d2-pdf make d2-pdf-check
else
  record d2-pdf true "skipped: FOLISIFT_D2 is not set"
fi

for guide in pd0-pd1 pd2 pd3 pd2-rev pd2-ko; do
  if cargo run -q -p folisift-cli -- qa --guide "$guide" >/tmp/folisift-acceptance-qa-"$guide".json 2>/tmp/folisift-acceptance-qa-"$guide".err; then
    record "qa:$guide" true "pass"
  else
    record "qa:$guide" false "see /tmp/folisift-acceptance-qa-$guide.err"
  fi
done

run doctor cargo run -q -p folisift-cli -- doctor
run coverage-hint cargo run -q -p folisift-cli -- coverage-hint --guide pd3
run diagram-status cargo run -q -p folisift-cli -- diagram status --guide pd3

printf '{"acceptance":true,"checks":[%s]}\n' "$(IFS=,; echo "${json[*]}")"
