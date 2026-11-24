#!/usr/bin/env bash
# Stage-4 M1 embed-demo acceptance gate: drive the cmake graph (cpp shelf +
# fixture + HostEmbedV1 staticlib lane), run the C++ embed demo against the
# real built fixture .so, and require its CPP-DEMO PASS line. NO
# skip-when-absent on the demo leg itself - a run that cannot build or
# cannot print CPP-DEMO PASS exits 1 (the ctest entry carries the 77-skip,
# this gate never does). Cargo/cmake legs route through pixi per D65 (same
# rule as demo-stage1/2).
#
# Build tree note: this gate runs in a DEDICATED build/presets/demo Ninja
# tree (target-scoped build), not the shared out/build - the shared tree is
# also reconfigured by IDE tooling (the PIT-7 class) and a generator flip
# mid-run would make this gate undeterministic. Warm incremental cost after
# the first run is seconds.
#
# C0 observed-RED lever: DEMO_STAGE4_SABOTAGE=1 (documented at exactly ONE
# named point in examples/cpp/host_demo.cc, scene [2] manifest assembly)
# corrupts the manifest placement so the install door must RED. Kept behind
# the env on purpose - the falsifiability proof stays re-runnable:
#   DEMO_STAGE4_SABOTAGE=1 bash scripts/demo-stage4-m1.sh   # expect RED, exit 1
set -uo pipefail
cd "$(dirname "$0")/.."

# repo-internal TMPDIR (MediaServo smoke lesson 2026-11-23): sandbox hosts
# sometimes mount /tmp read-only and mktemp then silently returns junk; pin
# scratch into the out/ territory instead (ignored, self-cleaning).
export TMPDIR="$(pwd)/build/tmp-scratch"
mkdir -p "$TMPDIR"

export PATH="$HOME/.pixi/bin:$PATH"
if ! command -v pixi >/dev/null 2>&1; then
  echo "demo-stage4-m1: RED (pixi absent - the cpp graph has no unpinned fallback)"
  exit 1
fi

bd="build/presets/demo"
log=$(mktemp)
trap 'rm -f "$log"' EXIT

# stamp-vs-artifact self-heal: `cargo clean` removes the lane artifacts but
# leaves the mediator stamp fresh, and the warm target then reports "no
# work" with the .a still missing (the artifact is not a ninja node, only a
# plain input). Deleting the stale stamp forces cargo to re-materialize.
[ -f build/cargo-target/debug/libmodukit_core.a ] ||
  rm -f "$bd/.polyorch-rust-stamps/modukit-core-build.stamp"

{
  pixi run cmake -S . -B "$bd" -G Ninja &&
  # Warm the shared cargo tree first: the .a is a plain ninja input with no
  # producer edge (cargo rides a stamp), so a cold single-target build dies at
  # ninja parse time. Found when the territory moved out/ -> build/ and no
  # stale artifact could silently satisfy the edge (2026-11-23).
  pixi run cmake --build "$bd" --target modukit-core-build &&
  pixi run cmake --build "$bd" --target modukit-cpp-host-demo &&
  # E4 deepening (D2): the D96 v2-door twin rides the same graph; its seed
  # is asserted by the host demo's optional argv[2] scene.
  pixi run cmake --build "$bd" --target cpp-busv2-plugin &&
  # E2 C-band ride (onboarding-plan, ruling A1): the C fixture + C host demo
  # ride THIS leg as a content extension (zero new legs) - their host-demo
  # target pulls the C fixture and the core mediator through its own
  # dependency edges.
  pixi run cmake --build "$bd" --target c-host-demo
} >"$log" 2>&1 || {
  echo "demo-stage4-m1: RED (cmake build failed; last lines:)"
  tail -10 "$log"
  exit 1
}

so="$bd/test-plugins/cpp/cpp-example-plugin/libcpp-example-plugin.so"
demo="$bd/examples/cpp/modukit-cpp-host-demo"
# The ONLY legit skip: the cpp band auto-skipped in this graph (pinned env
# carries no protoc/libprotobuf, or the cargo lanes are off) -- mirror the
# bindings/cpp + test-plugins/cpp auto-skip. Everything else is RED.
if [ ! -x "$demo" ] && grep -qE "cpp shelf/env incomplete|cpp fixture band skipped|bindings/cpp: no protoc|shelf skipped" "$log"; then
  echo "demo-stage4-m1: SKIP (cpp band auto-skipped: no env protoc / lanes off)"
  exit 0
fi
[ -x "$demo" ] || { echo "demo-stage4-m1: RED (demo binary missing: $demo)"; exit 1; }
[ -f "$so" ]   || { echo "demo-stage4-m1: RED (fixture .so missing: $so)"; exit 1; }

"$demo" "$so" "$bd/test-plugins/cpp/cpp-busv2-plugin/libcpp-busv2-plugin.so" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  echo "demo-stage4-m1: RED (demo exited $status)"
  exit 1
fi
if ! grep -qx 'CPP-DEMO PASS' "$log"; then
  echo "demo-stage4-m1: RED (no CPP-DEMO PASS line in the transcript)"
  exit 1
fi

# E2 C-band ride: same falsifiable grammar - the C host demo must print
# C-DEMO PASS against the real C fixture (its own sabotage lever,
# DEMO_C_SABOTAGE=1, corrupts the manifest at its scene [2]).
cso="$bd/test-plugins/c/c-example-plugin/libc-example-plugin.so"
cdemo="$bd/examples/c/c-host-demo"
[ -x "$cdemo" ] || { echo "demo-stage4-m1: RED (c demo binary missing: $cdemo - MODUKIT_BUILD_C off?)"; exit 1; }
[ -f "$cso" ]   || { echo "demo-stage4-m1: RED (c fixture .so missing: $cso)"; exit 1; }
"$cdemo" "$cso" 2>&1 | tee "$log"
cstatus=${PIPESTATUS[0]}
if [ "$cstatus" -ne 0 ]; then
  echo "demo-stage4-m1: RED (c demo exited $cstatus)"
  exit 1
fi
if ! grep -qx 'C-DEMO PASS' "$log"; then
  echo "demo-stage4-m1: RED (no C-DEMO PASS line in the transcript)"
  exit 1
fi
echo "demo-stage4-m1: PASS (cpp + c)"
