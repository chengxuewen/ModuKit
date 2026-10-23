#!/usr/bin/env bash
# Wave-2 Python volume M2 sup-rung gate (D100 plan §3-M2): regenerate the
# generated ctypes floor through the codegen door (it now carries all 24
# verbs incl. the 8 sup_*), build the M0 cdylib and the shadow runner, then
# run the Python sup walk (examples/py/host_sup_demo.py, a one-for-one
# mirror of examples/cpp/host_sup_demo.cc): sup_install -> dial -> READY
# walk, the FIDELITY stanza over sup_service_call, then the SIGKILL-orphan
# proof via the /proc census. PASS-line grammar mirrors scripts/
# demo-wave2-py.sh; the cargo legs route through pixi per D65.
# Auto-SKIP only when an instrument is absent (no python3, no build
# toolchain at all, no pinned-env protoc - the cpp-band instrument contract,
# or the cpp fixture band was never built) - a present toolchain that fails
# to build is RED, never a silent skip (C0).
#
# C0 observed-RED levers (two named points, still ONE leg - the cpp
# demo-stage4-m2.sh pattern):
#   DEMO_PY_M2_SABOTAGE=1 - parent SIGSTOPs the walk child instead of
#     killing it: the /proc census MUST find the live runner and the gate
#     goes RED without printing any PASS line:
#       DEMO_PY_M2_SABOTAGE=1 bash scripts/demo-wave2-py-sup.sh  # expect RED
#   DEMO_PY_M3_SABOTAGE=1 - corrupts the fidelity stanza's expected bytes:
#     the walk child never prints the FIDELITY PASS marker, the parent goes
#     RED with no PY-SUP-DEMO PASS line:
#       DEMO_PY_M3_SABOTAGE=1 bash scripts/demo-wave2-py-sup.sh  # expect RED
set -uo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.pixi/bin:$PATH"
if ! command -v python3 >/dev/null 2>&1; then
  echo "demo-wave2-py-sup: SKIP (no python3: the floor has no interpreter to run in)"
  exit 0
fi
if command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
elif command -v cargo >/dev/null 2>&1; then
  CARGO=(cargo)
else
  echo "demo-wave2-py-sup: SKIP (no pixi and no cargo: no build toolchain on this box)"
  exit 0
fi
# Protoc pin (the instrument the cpp band auto-skips on) - same contract as
# scripts/demo-wave2-py.sh so both skins rise and fall together.
protoc_found=0
[ -x ".pixi/envs/default/bin/protoc" ] && protoc_found=1
command -v protoc >/dev/null 2>&1 && protoc_found=1
if [ "$protoc_found" -eq 0 ]; then
  echo "demo-wave2-py-sup: SKIP (no pinned-env protoc: cpp-band instrument contract unmet)"
  exit 0
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT

"${CARGO[@]}" build -p modukit-core >>"$log" 2>&1 || {
  echo "demo-wave2-py-sup: RED (modukit-core cdylib build failed; last lines:)"
  tail -10 "$log"
  exit 1
}
core_so=""
for c in target/debug/libmodukit_core.so out/cargo-target/debug/libmodukit_core.so; do
  if [ -f "$c" ]; then core_so="$c"; break; fi
done
[ -n "$core_so" ] || { echo "demo-wave2-py-sup: RED (no libmodukit_core.so after a green build)"; exit 1; }

# The shadow runner (the exe manifest's `runner` = the demo's argv[3]) -
# the dedicated out/cargo-target tree, same as scripts/demo-stage4-m2.sh:35.
"${CARGO[@]}" build -p modukit-runtime --target-dir out/cargo-target >>"$log" 2>&1 || {
  echo "demo-wave2-py-sup: RED (runner build failed; last lines:)"
  tail -10 "$log"
  exit 1
}
runner="$PWD/out/cargo-target/debug/modukit-run"
[ -x "$runner" ] || { echo "demo-wave2-py-sup: RED (runner binary missing: $runner)"; exit 1; }

# The live skin is REGENERATED here through the emission door - the sup
# prototypes the demo calls are generated, never hand-written (D63).
scratch=$(mktemp -d)
"${CARGO[@]}" run -q -p modukit-codegen --bin protoc-gen-modukit -- \
  "@contracts/org.modukit.runtime.proto.fdset" \
  "@contracts/com.modukit.demo.proto.fdset" \
  --embed_header contracts/modukit_embed.h \
  --output_dir "$scratch" --py_out bindings/py >>"$log" 2>&1 || {
  echo "demo-wave2-py-sup: RED (skin regeneration failed; last lines:)"
  tail -10 "$log"
  rm -rf "$scratch"
  exit 1
}
rm -rf "$scratch"
[ -f bindings/py/modukit_embed.py ] || { echo "demo-wave2-py-sup: RED (bindings/py/modukit_embed.py not regenerated)"; exit 1; }

# The fixture: the same real C++ plugin .so the cpp M2 demo drives; when
# neither tree carries it the demo has nothing to install - the host_demo.cc
# skip rule (exit 77), kept honest.
so=""
for c in out/build-stage4/test-plugins/cpp/cpp-example-plugin/libcpp-example-plugin.so \
         out/build/test-plugins/cpp/cpp-example-plugin/libcpp-example-plugin.so; do
  if [ -f "$c" ]; then so="$c"; break; fi
done
if [ -z "$so" ]; then
  echo "demo-wave2-py-sup: SKIP (cpp fixture band not built: no libcpp-example-plugin.so under out/)"
  exit 0
fi

python3 examples/py/host_sup_demo.py --proof "$core_so" "$so" "$runner" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -eq 77 ]; then
  echo "demo-wave2-py-sup: SKIP (demo reported a missing input - see the transcript above)"
  exit 0
fi
if [ "$status" -ne 0 ]; then
  echo "demo-wave2-py-sup: RED (demo exited $status)"
  exit 1
fi
if ! grep -q '^FIDELITY PASS' "$log"; then
  echo "demo-wave2-py-sup: RED (no FIDELITY PASS line - the fidelity door did not prove byte-exactness)"
  exit 1
fi
if ! grep -qx 'PY-SUP-DEMO PASS' "$log"; then
  echo "demo-wave2-py-sup: RED (no PY-SUP-DEMO PASS line in the transcript)"
  exit 1
fi
echo "demo-wave2-py-sup: PASS"
