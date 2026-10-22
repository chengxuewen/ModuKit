#!/usr/bin/env bash
# Wave-2 Python volume M1 embed-demo gate (D100 R3-A leg 3): regenerate the
# generated ctypes floor through the codegen door, build the M0 cdylib, then
# run the Python 16-verb walk (examples/py/host_demo.py, a one-for-one mirror
# of examples/cpp/host_demo.cc scenes [1]-[11]) against the real built C++
# fixture .so and require its PY-DEMO PASS line. cmake-free precedent:
# scripts/demo-wave2-clock.sh; the cargo legs route through pixi per D65.
# Auto-SKIP only when an instrument is absent (no python3, no build
# toolchain at all, no pinned-env protoc - the cpp-band instrument contract,
# or the cpp fixture band was never built) - a present toolchain that fails
# to build is RED, never a silent skip (C0).
#
# C0 observed-RED lever: DEMO_PY_SABOTAGE=1 (read at exactly ONE named point
# in examples/py/host_demo.py, scene [2] manifest assembly) corrupts the
# manifest placement so the install door must RED:
#   DEMO_PY_SABOTAGE=1 bash scripts/demo-wave2-py.sh   # expect RED, exit 1
#   Observed 2026-10-22: script RED (exit 1, "PY-DEMO FAIL (1, install door)"
#   printed), restored green the same run.
set -uo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.pixi/bin:$PATH"
if ! command -v python3 >/dev/null 2>&1; then
  echo "demo-wave2-py: SKIP (no python3: the floor has no interpreter to run in)"
  exit 0
fi
if command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
elif command -v cargo >/dev/null 2>&1; then
  CARGO=(cargo)
else
  echo "demo-wave2-py: SKIP (no pixi and no cargo: no build toolchain on this box)"
  exit 0
fi
# Protoc pin (the instrument the cpp band auto-skips on). The floor itself
# needs no protoc at runtime (the checked-in fdsets carry the corpus and the
# step-0 payload probe measured RED - the demo carries its own fixture
# codec), but the leg keeps the cpp-band instrument contract so both skins
# rise and fall together.
protoc_found=0
[ -x ".pixi/envs/default/bin/protoc" ] && protoc_found=1
command -v protoc >/dev/null 2>&1 && protoc_found=1
if [ "$protoc_found" -eq 0 ]; then
  echo "demo-wave2-py: SKIP (no pinned-env protoc: cpp-band instrument contract unmet)"
  exit 0
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT

"${CARGO[@]}" build -p modukit-core >>"$log" 2>&1 || {
  echo "demo-wave2-py: RED (modukit-core cdylib build failed; last lines:)"
  tail -10 "$log"
  exit 1
}
core_so=""
for c in target/debug/libmodukit_core.so out/cargo-target/debug/libmodukit_core.so; do
  if [ -f "$c" ]; then core_so="$c"; break; fi
done
[ -n "$core_so" ] || { echo "demo-wave2-py: RED (no libmodukit_core.so after a green build)"; exit 1; }

# The live skin is REGENERATED here through the emission door - nothing is
# hand-committed under bindings/py/ (the byte-pinned face is the golden at
# crates/modukit-codegen/testdata/golden/modukit_embed.py).
scratch=$(mktemp -d)
"${CARGO[@]}" run -q -p modukit-codegen --bin protoc-gen-modukit -- \
  "@contracts/org.modukit.runtime.proto.fdset" \
  "@contracts/com.modukit.demo.proto.fdset" \
  --embed_header contracts/modukit_embed.h \
  --output_dir "$scratch" --py_out bindings/py >>"$log" 2>&1 || {
  echo "demo-wave2-py: RED (skin regeneration failed; last lines:)"
  tail -10 "$log"
  rm -rf "$scratch"
  exit 1
}
rm -rf "$scratch"
[ -f bindings/py/modukit_embed.py ] || { echo "demo-wave2-py: RED (bindings/py/modukit_embed.py not regenerated)"; exit 1; }

# The fixture: the same real C++ plugin .so the cpp M1 demo drives. It is
# built by the stage4 cmake lanes; when neither tree carries it the demo has
# nothing to install - the host_demo.cc skip rule (exit 77), kept honest.
so=""
for c in out/build-stage4/test-plugins/cpp/cpp-example-plugin/libcpp-example-plugin.so \
         out/build/test-plugins/cpp/cpp-example-plugin/libcpp-example-plugin.so; do
  if [ -f "$c" ]; then so="$c"; break; fi
done
if [ -z "$so" ]; then
  echo "demo-wave2-py: SKIP (cpp fixture band not built: no libcpp-example-plugin.so under out/)"
  exit 0
fi

python3 examples/py/host_demo.py "$core_so" "$so" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -eq 77 ]; then
  echo "demo-wave2-py: SKIP (demo reported a missing input - see the transcript above)"
  exit 0
fi
if [ "$status" -ne 0 ]; then
  echo "demo-wave2-py: RED (demo exited $status)"
  exit 1
fi
if ! grep -qx 'PY-DEMO PASS' "$log"; then
  echo "demo-wave2-py: RED (no PY-DEMO PASS line in the transcript)"
  exit 1
fi
echo "demo-wave2-py: PASS"
