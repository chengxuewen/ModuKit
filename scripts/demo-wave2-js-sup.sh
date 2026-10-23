#!/usr/bin/env bash
# Wave-2 JS volume M2 sup-rung gate (D102 plan §3-M2): regenerate the two
# generated faces through the codegen door (they now carry all 24 verbs incl.
# the 8 sup_*), build the kernel cdylib + the shadow runner + the napi addon,
# then run the JS sup walk (examples/js/host_sup_demo.js, a one-for-one mirror
# of examples/cpp/host_sup_demo.cc): sup_install -> dial -> READY walk, the
# FIDELITY stanza over sup_service_call (the tri-language byte constant), then
# the SIGKILL-orphan proof via the /proc census. PASS-line grammar mirrors
# scripts/demo-wave2-js.sh + scripts/demo-wave2-py-sup.sh; cargo legs route
# through pixi per D65.
# Auto-SKIP only when an instrument is absent (no node, no build toolchain at
# all, no pinned-env protoc - the cpp-band instrument contract, or the cpp
# fixture band was never built) - a present toolchain that fails to build is
# RED, never a silent skip (C0).
#
# Node pin (plan R5): the built .node matches modules ABI 127 (node 22.x, M0
# probe). A node upgrade re-arms R2 - rebuild the addon, do not swap it.
#
# C0 observed-RED levers (two named points, still ONE leg - the cpp
# demo-stage4-m2.sh and the Python demo-wave2-py-sup.sh pattern):
#   DEMO_JS_M2_SABOTAGE=1 - parent SIGSTOPs the walk child instead of killing
#     it: the /proc census MUST find the live runner and the gate goes RED
#     without printing any PASS line:
#       DEMO_JS_M2_SABOTAGE=1 bash scripts/demo-wave2-js-sup.sh  # expect RED
#   DEMO_JS_M3_SABOTAGE=1 - corrupts the fidelity stanza's expected bytes: the
#     walk child never prints the FIDELITY PASS marker, the parent goes RED
#     with no JS-SUP-DEMO PASS line:
#       DEMO_JS_M3_SABOTAGE=1 bash scripts/demo-wave2-js-sup.sh  # expect RED
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
  echo "demo-wave2-js-sup: SKIP (no node: the bridge has no addon runtime to load into)"
  exit 0
fi
echo "RUNNER: node $("$NBIN" --version 2>&1) ($NSRC)"
if command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
elif command -v cargo >/dev/null 2>&1; then
  CARGO=(cargo)
else
  echo "demo-wave2-js-sup: SKIP (no pixi and no cargo: no build toolchain on this box)"
  exit 0
fi
# Protoc pin (the instrument the cpp band auto-skips on) - same contract as
# scripts/demo-wave2-js.sh so both JS skins rise and fall together.
protoc_found=0
[ -x ".pixi/envs/default/bin/protoc" ] && protoc_found=1
command -v protoc >/dev/null 2>&1 && protoc_found=1
if [ "$protoc_found" -eq 0 ]; then
  echo "demo-wave2-js-sup: SKIP (no pinned-env protoc: cpp-band instrument contract unmet)"
  exit 0
fi

log=$(mktemp)
trap 'rm -f "$log"' EXIT

"${CARGO[@]}" build -p modukit-core >>"$log" 2>&1 || {
  echo "demo-wave2-js-sup: RED (modukit-core cdylib build failed; last lines:)"
  tail -10 "$log"
  exit 1
}
core_so=""
for c in target/debug/libmodukit_core.so out/cargo-target/debug/libmodukit_core.so; do
  if [ -f "$c" ]; then core_so="$c"; break; fi
done
[ -n "$core_so" ] || { echo "demo-wave2-js-sup: RED (no libmodukit_core.so after a green build)"; exit 1; }

# The shadow runner (the exe manifest's `runner` = the demo's argv[4]) - the
# dedicated out/cargo-target tree, same as scripts/demo-wave2-py-sup.sh:67.
"${CARGO[@]}" build -p modukit-runtime --offline --target-dir out/cargo-target >>"$log" 2>&1 || {
  echo "demo-wave2-js-sup: RED (runner build failed; last lines:)"
  tail -10 "$log"
  exit 1
}
runner="$PWD/out/cargo-target/debug/modukit-run"
[ -x "$runner" ] || { echo "demo-wave2-js-sup: RED (runner binary missing: $runner)"; exit 1; }

# The live faces are REGENERATED here through the emission door - the sup_*
# forwarders the demo calls are generated, never hand-written (D63), and
# nothing is hand-committed under target/js/. The bridge face lands in the
# tracked crate source; the facade lives under target/ (git-ignored).
scratch=$(mktemp -d)
"${CARGO[@]}" run -q -p modukit-codegen --bin protoc-gen-modukit -- \
  "@contracts/org.modukit.runtime.proto.fdset" \
  "@contracts/com.modukit.demo.proto.fdset" \
  --embed_header contracts/modukit_embed.h \
  --js_out "$scratch" \
  --output_dir "$scratch" >>"$log" 2>&1 || {
  echo "demo-wave2-js-sup: RED (face regeneration failed; last lines:)"
  tail -10 "$log"
  rm -rf "$scratch"
  exit 1
}
mkdir -p target/js
cp "$scratch/modukit_embed_bridge.rs" crates/modukit-js/src/lib.rs || { echo "demo-wave2-js-sup: RED (bridge face not copied)"; rm -rf "$scratch"; exit 1; }
cp "$scratch/modukit_embed.js" target/js/modukit_embed.js || { echo "demo-wave2-js-sup: RED (facade not copied)"; rm -rf "$scratch"; exit 1; }
rm -rf "$scratch"
[ -f crates/modukit-js/src/lib.rs ] || { echo "demo-wave2-js-sup: RED (crates/modukit-js/src/lib.rs not regenerated)"; exit 1; }
[ -f target/js/modukit_embed.js ] || { echo "demo-wave2-js-sup: RED (target/js/modukit_embed.js not regenerated)"; exit 1; }
# Live bridge face must equal its golden, or the checked-in corpus drifted.
cmp -s crates/modukit-js/src/lib.rs crates/modukit-codegen/testdata/golden/modukit_embed_bridge.rs || {
  echo "demo-wave2-js-sup: RED (crates/modukit-js/src/lib.rs drifted from the golden - regen + review)"
  exit 1
}

# The addon: the cdylib is the .node by another suffix (the M0 probe grammar).
"${CARGO[@]}" build -p modukit-js --offline >>"$log" 2>&1 || {
  echo "demo-wave2-js-sup: RED (modukit-js addon build failed; last lines:)"
  tail -10 "$log"
  exit 1
}
addon_js="target/debug/libmodukit_js.so"
[ -f "$addon_js" ] || { echo "demo-wave2-js-sup: RED (no libmodukit_js.so after a green build)"; exit 1; }
cp "$addon_js" target/js/modukit_js.node

# The fixture: the same real C++ plugin .so the cpp M2 demo drives (the sup
# demo dlopens it through the SAME kernel); when neither tree carries it the
# demo has nothing to install - the host_sup_demo.cc skip rule (exit 77).
so=""
for c in out/build-stage4/test-plugins/cpp/cpp-example-plugin/libcpp-example-plugin.so \
         out/build/test-plugins/cpp/cpp-example-plugin/libcpp-example-plugin.so; do
  if [ -f "$c" ]; then so="$c"; break; fi
done
if [ -z "$so" ]; then
  echo "demo-wave2-js-sup: SKIP (cpp fixture band not built: no libcpp-example-plugin.so under out/)"
  exit 0
fi
# Runtime libstdc++ fix (carried from scripts/demo-wave2-js.sh): the cpp
# fixture's dependency chain needs GLIBCXX_3.4.29 but node links the SYSTEM
# libstdc++ at startup; preloading the pinned-env libstdc++ lets the whole
# chain resolve inside the kernel's dlopen.
cxxlib="$PWD/.pixi/envs/default/lib/libstdc++.so.6"
if [ -f "$cxxlib" ]; then
  export LD_PRELOAD="$cxxlib${LD_PRELOAD:+:$LD_PRELOAD}"
fi

"$NBIN" examples/js/host_sup_demo.js --proof "$core_so" target/js/modukit_js.node "$so" "$runner" 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
if [ "$status" -eq 77 ]; then
  echo "demo-wave2-js-sup: SKIP (demo reported a missing input - see the transcript above)"
  exit 0
fi
if [ "$status" -ne 0 ]; then
  echo "demo-wave2-js-sup: RED (demo exited $status)"
  exit 1
fi
if ! grep -q '^FIDELITY PASS' "$log"; then
  echo "demo-wave2-js-sup: RED (no FIDELITY PASS line - the fidelity door did not prove byte-exactness)"
  exit 1
fi
if ! grep -qx 'JS-SUP-DEMO PASS' "$log"; then
  echo "demo-wave2-js-sup: RED (no JS-SUP-DEMO PASS line in the transcript)"
  exit 1
fi
echo "demo-wave2-js-sup: PASS"
