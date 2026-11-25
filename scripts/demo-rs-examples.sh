#!/usr/bin/env bash
# demo-rs-examples.sh - the MERGED rs topic-example leg (bundle-management-plan
# example-volume sweep, ruling B1/B2): one script builds and runs the four
# Rust-face capability examples in order and greps each PASS line. A run that
# cannot build or cannot print a PASS line exits 1 - no skip-when-absent (the
# examples are workspace members; a failed build is a RED, never a silent pass).
#
# This is a NEW demo-family leg (the rs topic namespace), NOT a check.sh leg:
# B2's ceiling rule - the m1 leg stays under its scene budget; the topic
# namespace gets its own runner when it grows. Wired into check.sh by the
# example-volume sweep (single added line, counted).
set -uo pipefail
cd "$(dirname "$0")/.."
export PATH="$HOME/.pixi/bin:$PATH"

if ! command -v pixi >/dev/null 2>&1; then
  echo "demo-rs-examples: RED (pixi absent)"
  exit 1
fi

# repo-internal TMPDIR (the demo legs' convention).
export TMPDIR="$(pwd)/build/tmp-scratch"
mkdir -p "$TMPDIR"

run_example() {
  local ex=$1 pass_line=$2
  local log
  log=$(mktemp)
  if ! pixi run cargo run -q -p modukit-examples --example "$ex" >"$log" 2>&1; then
    echo "demo-rs-examples: RED ($ex exited non-zero; last lines:)"
    tail -6 "$log"
    rm -f "$log"
    exit 1
  fi
  if ! grep -qx "$pass_line" "$log"; then
    echo "demo-rs-examples: RED ($ex printed no $pass_line line)"
    tail -6 "$log"
    rm -f "$log"
    exit 1
  fi
  rm -f "$log"
  echo "  $ex: PASS"
}

echo "demo-rs-examples: building and running the topic examples"
run_example tracker_demo "TRACKER-DEMO PASS"
run_example dm_demo      "DM-DEMO PASS"
run_example config_demo  "CONFIG-DEMO PASS"
run_example ops_demo     "OPS-DEMO PASS"
echo "demo-rs-examples: PASS (tracker + dm + config + ops)"
