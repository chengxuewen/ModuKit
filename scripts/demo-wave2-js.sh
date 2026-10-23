#!/usr/bin/env bash
# Wave-2 JS volume M1 embed-demo gate (D102 R3-A leg 3): regenerate the two
# generated faces through the codegen door (bridge -> crates/modukit-js/src/
# lib.rs, facade -> target/js/), build the kernel cdylib and the napi addon,
# then run the JS 16-verb walk (examples/js/host_demo.js, a one-for-one
# mirror of examples/cpp/host_demo.cc scenes [1]-[11]) against the real built
# C++ fixture .so and require its JS-DEMO PASS line. cmake-free precedent:
# scripts/demo-wave2-py.sh; the cargo legs route through pixi per D65.
# Auto-SKIP only when an instrument is absent (no node, no build toolchain at
# all, no pinned-env protoc - the cpp-band instrument contract, or the cpp
# fixture band was never built) - a present toolchain that fails to build is
# RED, never a silent skip (C0).
#
# Node pin (plan R5): the built .node matches modules ABI 127 (node 22.x,
# M0 probe). A node upgrade re-arms R2 - rebuild the addon, do not swap it.
#
# C0 observed-RED lever: DEMO_JS_SABOTAGE=1 (read at exactly ONE named point
# in examples/js/host_demo.js, scene [2] manifest assembly) corrupts the
# manifest placement so the install door must RED:
#   DEMO_JS_SABOTAGE=1 bash scripts/demo-wave2-js.sh   # expect RED, exit 1
#   Observed 2026-10-23: script RED (exit 1, "JS-DEMO FAIL (1, install door)"
#   printed), restored green the same run.
set -uo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.pixi/bin:$PATH"
# T3.1 runner-origin probe (wave-2 benchmark plan §4): pixi default env
# interpreter first, PATH fallback; neither present keeps the loud SKIP
# (marker string byte-identical). One RUNNER line per invocation, version
# read from the interpreter itself.
if [ -x ".pixi/envs/default/bin/node" ]; then
  NBIN=".pixi/envs/default/bin/node"
  NSRC="pixi-env"
elif command -v node >/dev/null 2>&1; then
  NBIN="node"
  NSRC="PATH fallback - pixi env not warmed"
else
  echo "demo-wave2-js: SKIP (no node: the bridge has no addon runtime to load into)"
  exit 0
fi
echo "RUNNER: node $("$NBIN" --version 2>&1) ($NSRC)"
if command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
elif command -v cargo >/dev/null 2>&1; then
  CARGO=(cargo)
else
  echo "demo-wave2-js: SKIP (no pixi and no cargo: no build toolchain on this box)"
  exit 0
fi
# Protoc pin (the instrument the cpp band auto-skips on). The bridge itself
# needs no protoc at runtime (the checked-in fdsets carry the corpus and the
# demo carries its own fixture codec, the python-era precedent), but the leg
# keeps the cpp-band instrument contract so both skins rise and fall together.
protoc_found=0
[ -x ".pixi/envs/default/bin/protoc" ] && protoc_found=1
command -v protoc >/dev/null 2>&1 && protoc_found=1
if [ "$protoc_found" -eq 0 ]; then
  echo "demo-wave2-js: SKIP (no pinned-env protoc: cpp-band instrument contract unmet)"
  exit 0
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT

"${CARGO[@]}" build -p modukit-core >>"$log" 2>&1 || {
  echo "demo-wave2-js: RED (modukit-core cdylib build failed; last lines:)"
  tail -10 "$log"
  exit 1
}
core_so=""
for c in target/debug/libmodukit_core.so out/cargo-target/debug/libmodukit_core.so; do
  if [ -f "$c" ]; then core_so="$c"; break; fi
done
[ -n "$core_so" ] || { echo "demo-wave2-js: RED (no libmodukit_core.so after a green build)"; exit 1; }

# The live faces are REGENERATED here through the emission door - nothing is
# hand-committed under target/js/ (the byte-pinned faces are the goldens at
# crates/modukit-codegen/testdata/golden/modukit_embed_bridge.rs +
# modukit_embed.js). The bridge face lands in the tracked crate source; the
# facade live path is target/js/ because .gitignore covers target/ and does
# NOT cover bindings/js/ (bindings/py/ landed 2026-10-22, the js sibling
# never did - the facade must stay out of git without editing the ignore).
scratch=$(mktemp -d)
"${CARGO[@]}" run -q -p modukit-codegen --bin protoc-gen-modukit -- \
  "@contracts/org.modukit.runtime.proto.fdset" \
  "@contracts/com.modukit.demo.proto.fdset" \
  --embed_header contracts/modukit_embed.h \
  --js_out "$scratch" \
  --output_dir "$scratch" >>"$log" 2>&1 || {
  echo "demo-wave2-js: RED (face regeneration failed; last lines:)"
  tail -10 "$log"
  rm -rf "$scratch"
  exit 1
}
mkdir -p target/js
cp "$scratch/modukit_embed_bridge.rs" crates/modukit-js/src/lib.rs || { echo "demo-wave2-js: RED (bridge face not copied)"; rm -rf "$scratch"; exit 1; }
cp "$scratch/modukit_embed.js" target/js/modukit_embed.js || { echo "demo-wave2-js: RED (facade not copied)"; rm -rf "$scratch"; exit 1; }
rm -rf "$scratch"
[ -f crates/modukit-js/src/lib.rs ] || { echo "demo-wave2-js: RED (crates/modukit-js/src/lib.rs not regenerated)"; exit 1; }
[ -f target/js/modukit_embed.js ] || { echo "demo-wave2-js: RED (target/js/modukit_embed.js not regenerated)"; exit 1; }
# Live bridge face must equal its golden, or the checked-in corpus drifted.
cmp -s crates/modukit-js/src/lib.rs crates/modukit-codegen/testdata/golden/modukit_embed_bridge.rs || {
  echo "demo-wave2-js: RED (crates/modukit-js/src/lib.rs drifted from the golden - regen + review)"
  exit 1
}

# The addon: the cdylib is the .node by another suffix (the M0 probe grammar).
"${CARGO[@]}" build -p modukit-js --offline >>"$log" 2>&1 || {
  echo "demo-wave2-js: RED (modukit-js addon build failed; last lines:)"
  tail -10 "$log"
  exit 1
}
addon_js="target/debug/libmodukit_js.so"
[ -f "$addon_js" ] || { echo "demo-wave2-js: RED (no libmodukit_js.so after a green build)"; exit 1; }
cp "$addon_js" target/js/modukit_js.node

# The fixture: the same real C++ plugin .so the cpp M1 demo drives. It is
# built by the stage4 cmake lanes; when neither tree carries it the demo has
# nothing to install - the host_demo.cc skip rule (exit 77), kept honest.
so=""
for c in out/build-stage4/test-plugins/cpp/cpp-example-plugin/libcpp-example-plugin.so \
         out/build/test-plugins/cpp/cpp-example-plugin/libcpp-example-plugin.so; do
  if [ -f "$c" ]; then so="$c"; break; fi
done
if [ -z "$so" ]; then
  echo "demo-wave2-js: SKIP (cpp fixture band not built: no libcpp-example-plugin.so under out/)"
  exit 0
fi
# Runtime libstdc++ fix (measured 2026-10-23): the cpp fixture's dependency
# chain (gencode -> pinned-env libprotobuf 36.2) needs GLIBCXX_3.4.29, but
# node links the SYSTEM libstdc++ (3.4.28) at startup and glibc dedupes by
# soname - the fixture then refuses to load inside the kernel's dlopen.
# Preloading the pinned-env libstdc++ (backward compatible, a superset of
# the 3.4.28 symbols node itself needs) lets the whole chain resolve; the
# cpp band's own binaries get the same runtime through the pixi env.
cxxlib="$PWD/.pixi/envs/default/lib/libstdc++.so.6"
if [ -f "$cxxlib" ]; then
  export LD_PRELOAD="$cxxlib${LD_PRELOAD:+:$LD_PRELOAD}"
fi

"$NBIN" examples/js/host_demo.js "$core_so" target/js/modukit_js.node "$so" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -eq 77 ]; then
  echo "demo-wave2-js: SKIP (demo reported a missing input - see the transcript above)"
  exit 0
fi
if [ "$status" -ne 0 ]; then
  echo "demo-wave2-js: RED (demo exited $status)"
  exit 1
fi
if ! grep -qx 'JS-DEMO PASS' "$log"; then
  echo "demo-wave2-js: RED (no JS-DEMO PASS line in the transcript)"
  exit 1
fi
echo "demo-wave2-js: PASS"
