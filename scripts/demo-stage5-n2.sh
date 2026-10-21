#!/usr/bin/env bash
# Stage-5 N2 cross-wall gate (D96 ruling 6): run the wall-transparency demo
# - the real shadow-runner hosts the busv2 fixture across the exe wall and
# the PARENT's own bus must carry the child's publishes + retained snapshot.
# The gate is the final 'STAGE5-N2-DEMO PASS' line (stage2 grammar); cargo
# legs route through pixi per D65. Auto-SKIP only when NO build toolchain
# exists at all (fresh box without pixi and without cargo) - a present
# toolchain that fails to build is RED, never a silent skip (C0).
#
# C0 observed-RED lever: DEMO_STAGE5_N2_SABOTAGE=1 (documented at exactly
# ONE named point in examples/rs/stage5_n2_demo.rs, scene [S2]) turns the
# runner's capability flag off: the child walks its local bus, the crossing
# scene [S4] must then RED and no PASS line may print:
#   DEMO_STAGE5_N2_SABOTAGE=1 bash scripts/demo-stage5-n2.sh   # expect RED, exit 1
set -uo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.pixi/bin:$PATH"
if command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
elif command -v cargo >/dev/null 2>&1; then
  CARGO=(cargo)
else
  echo "demo-stage5-n2: SKIP (no pixi and no cargo: no build toolchain on this box)"
  exit 0
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT
"${CARGO[@]}" run -p modukit-examples --example stage5_n2_demo 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  echo "demo-stage5-n2: RED (demo exited $status)"
  exit 1
fi
if ! grep -qx 'STAGE5-N2-DEMO PASS' "$log"; then
  echo "demo-stage5-n2: RED (no STAGE5-N2-DEMO PASS line in the transcript)"
  exit 1
fi
echo "demo-stage5-n2: PASS"
