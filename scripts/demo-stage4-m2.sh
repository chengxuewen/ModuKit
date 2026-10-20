#!/usr/bin/env bash
# Stage-4 M2 exe-rung gate: the C++ host walks sup_install/ACTIVE/step/
# dial-refusal over a REAL runner child, then gets SIGKILLed and the
# runner must self-reap (ORPHAN-PROOF transplanted from S2-M5a, now under
# a C++-authored host). NO skip-when-absent on the gate itself; cargo and
# cmake legs route through pixi per D65. Reuses the dedicated
# out/build-stage4 tree from demo-stage4-m1 (warm).
#
# C0 observed-RED lever: DEMO_STAGE4_M2_SABOTAGE=1 at exactly ONE named
# point in examples/cpp/host_sup_demo.cc (parent stops instead of kills):
# the census MUST then find the live runner and the gate goes RED without
# printing any PASS line:
#   DEMO_STAGE4_M2_SABOTAGE=1 bash scripts/demo-stage4-m2.sh   # expect RED
set -uo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.pixi/bin:$PATH"
if ! command -v pixi >/dev/null 2>&1; then
  echo "demo-stage4-m2: RED (pixi absent - the cpp graph has no unpinned fallback)"
  exit 1
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT

# runner first (the demo's argv[2] default points at this exact path)
pixi run cargo build -p modukit-runtime --target-dir out/cargo-target \
  >/dev/null 2>&1 || { echo "demo-stage4-m2: RED (runner build failed)"; exit 1; }
runner="$PWD/out/cargo-target/debug/modukit-run"

{
  pixi run cmake -S . -B out/build-stage4 -G Ninja &&
  pixi run cmake --build out/build-stage4 --target modukit-cpp-host-sup-demo
} >"$log" 2>&1 || {
  echo "demo-stage4-m2: RED (cmake build failed; last lines:)"; tail -10 "$log"; exit 1
}

so="out/build-stage4/test-plugins/cpp/cpp-example-plugin/libcpp-example-plugin.so"
demo="out/build-stage4/examples/cpp/modukit-cpp-host-sup-demo"
if [ ! -x "$demo" ] && grep -qE "cpp shelf/env incomplete|cpp fixture band skipped|bindings/cpp: no protoc|shelf skipped" "$log"; then
  echo "demo-stage4-m2: SKIP (cpp band auto-skipped: no env protoc / lanes off)"
  exit 0
fi
[ -x "$demo" ] || { echo "demo-stage4-m2: RED (demo binary missing: $demo)"; exit 1; }
[ -f "$so" ]   || { echo "demo-stage4-m2: RED (fixture .so missing: $so)"; exit 1; }

"$demo" --proof "$so" "$runner" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  echo "demo-stage4-m2: RED (demo exited $status)"
  exit 1
fi
if ! grep -qx 'SUP-DEMO PASS' "$log"; then
  echo "demo-stage4-m2: RED (no SUP-DEMO PASS line in the transcript)"
  exit 1
fi
echo "demo-stage4-m2: PASS"
