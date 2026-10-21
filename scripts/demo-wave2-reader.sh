#!/usr/bin/env bash
# Wave-2 debug-reader gate (closes the D97 re-review trigger): run the
# reader-only self-check face - modukit-debug-reader boots TestHost, reads
# the already-shipped introspect trio read-only, and prints
# 'debug-reader: PASS' as its final line. The gate is that PASS line
# (n2 grammar); cargo legs route through pixi per D65. Auto-SKIP only when
# NO build toolchain exists at all (fresh box without pixi and without
# cargo) - a present toolchain that fails to build is RED, never a silent
# skip (C0).
#
# C0 observed-RED lever: MODUKIT_DEBUG_READER_SABOTAGE=1 (read at exactly
# ONE named point in crates/modukit-runtime/src/debug_reader.rs) forces the
# pre-PASS sabotage branch: the face must exit 1 and no PASS line may print:
#   MODUKIT_DEBUG_READER_SABOTAGE=1 bash scripts/demo-wave2-reader.sh   # expect RED, exit 1
#   Observed 2026-10-21: script RED (exit 1, no PASS line) and the check.sh
#   leg takes its FAIL branch; restored green. Three-row evidence in
#   docs/wave-2-closeout-plan.md §5.
set -uo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.pixi/bin:$PATH"
if command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
elif command -v cargo >/dev/null 2>&1; then
  CARGO=(cargo)
else
  echo "demo-wave2-reader: SKIP (no pixi and no cargo: no build toolchain on this box)"
  exit 0
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT
"${CARGO[@]}" run -p modukit-runtime --bin modukit-debug-reader 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  echo "demo-wave2-reader: RED (reader exited $status)"
  exit 1
fi
if ! grep -qx 'debug-reader: PASS' "$log"; then
  echo "demo-wave2-reader: RED (no debug-reader: PASS line in the transcript)"
  exit 1
fi
echo "demo-wave2-reader: PASS"
