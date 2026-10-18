#!/usr/bin/env bash
# S2-M5 dual-placement acceptance gate (plan §7 S2-M5 row): run the
# end-to-end stage2 demo and require its PASS line. NO skip-when-absent -
# a run that cannot build or cannot print STAGE2-DEMO PASS exits 1. Cargo
# legs route through pixi per D65 (same rule as demo-stage1.sh).
#
# C0 observed-RED lever: DEMO_STAGE2_SABOTAGE=1 (documented at exactly ONE
# named point in examples/rs/stage2_demo.rs, scene [S7] bundle assembly)
# corrupts the runner path so the gate must RED. Kept behind the env on
# purpose - the falsifiability proof stays re-runnable:
#   DEMO_STAGE2_SABOTAGE=1 bash scripts/demo-stage2.sh   # expect RED, exit 1
set -uo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.pixi/bin:$PATH"
CARGO=(cargo)
if [ -f pixi.toml ] && command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT
"${CARGO[@]}" run -p modukit-examples --example stage2_demo 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  echo "demo-stage2: RED (demo exited $status)"
  exit 1
fi
if ! grep -qx 'STAGE2-DEMO PASS' "$log"; then
  echo "demo-stage2: RED (no STAGE2-DEMO PASS line in the transcript)"
  exit 1
fi
echo "demo-stage2: PASS"
