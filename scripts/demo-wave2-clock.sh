#!/usr/bin/env bash
# Wave-2 T2 clock-crossing gate (D99 R5-A): run the crossing demo - the real
# shadow-runner hosts the timerv3 fixture across the exe wall, the supervisor
# dials it over the wire, and the child's plugin bell must ride the existing
# FWD_PUB frame into the PARENT's own bus (gen ++ token ++ label decoded
# there, token equal to the DUE answer). The gate is the final
# 'WAVE2-CLOCK-DEMO PASS' line (n2 grammar); cargo legs route through pixi
# per D65. Auto-SKIP only when NO build toolchain exists at all (fresh box
# without pixi and without cargo) - a present toolchain that fails to build
# is RED, never a silent skip (C0).
#
# C0 observed-RED lever: DEMO_WAVE2_CLOCK_SABOTAGE=1 (read at exactly ONE
# named point in examples/rs/wave2_clock_demo.rs, scene [S2]) drops ONLY the
# runner's timer capability bit: the child walks and schedules child-local,
# the dial still answers, but no bell crosses and scene [S4] must RED with
# no PASS line printed:
#   DEMO_WAVE2_CLOCK_SABOTAGE=1 bash scripts/demo-wave2-clock.sh   # expect RED, exit 1
#   Observed 2026-10-22: script RED (exit 1, no PASS line; scene [S4]
#   printed "crossing absent"), restored green the same run.
set -uo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.pixi/bin:$PATH"
if command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
elif command -v cargo >/dev/null 2>&1; then
  CARGO=(cargo)
else
  echo "demo-wave2-clock: SKIP (no pixi and no cargo: no build toolchain on this box)"
  exit 0
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT
"${CARGO[@]}" run -p modukit-examples --example wave2_clock_demo 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  echo "demo-wave2-clock: RED (demo exited $status)"
  exit 1
fi
if ! grep -qx 'WAVE2-CLOCK-DEMO PASS' "$log"; then
  echo "demo-wave2-clock: RED (no WAVE2-CLOCK-DEMO PASS line in the transcript)"
  exit 1
fi
echo "demo-wave2-clock: PASS"
