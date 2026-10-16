#!/usr/bin/env bash
# M8 Stage-1 acceptance gate (plan §8 DoD 2): run the end-to-end demo and
# require its PASS line. NO skip-when-absent here - a run that cannot build
# or cannot print STAGE1-DEMO PASS exits 1. Cargo legs route through pixi
# per D65 (same rule as check.sh).
set -uo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.pixi/bin:$PATH"
CARGO=(cargo)
if [ -f pixi.toml ] && command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT
"${CARGO[@]}" run -p modukit-core --example stage1_demo 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  echo "demo-stage1: RED (demo exited $status)"
  exit 1
fi
if ! grep -qx 'STAGE1-DEMO PASS' "$log"; then
  echo "demo-stage1: RED (no STAGE1-DEMO PASS line in the transcript)"
  exit 1
fi
echo "demo-stage1: PASS"
