#!/usr/bin/env bash
# Rewrite the golden corpus under crates/modukit-codegen/testdata/golden/ from
# the checked-in contract fdsets AND both C contract headers (embed+plugin faces). Review the diff: a golden change means
# either the .proto changed (intended) or the generator drifted (a bug).
# The gate itself is `cargo test -p modukit-codegen --test golden`.
set -euo pipefail
cd "$(dirname "$0")/.."

if [ ! -f contracts/org.modukit.runtime.proto.fdset ] || [ ! -f contracts/com.modukit.demo.proto.fdset ]; then
  echo "FAIL regen-golden: checked-in fdsets missing - run scripts/regen-fdset.sh first" >&2
  exit 1
fi

# D65: the generator runs on the pinned pixi toolchain, never the system or
# mise shim. If we are not already inside a pixi env, re-exec under it.
if [ -z "${CONDA_PREFIX:-}" ] && [ -z "${MODUKIT_REGEN_INNER:-}" ] && [ -f pixi.toml ]; then
  export PATH="$HOME/.pixi/bin:$PATH"
  if command -v pixi >/dev/null 2>&1; then
    export MODUKIT_REGEN_INNER=1
    exec pixi run bash "$0"
  fi
fi

GOLDEN=crates/modukit-codegen/testdata/golden
mkdir -p "$GOLDEN"
# Keep only the generator's own outputs (no stale files from a renamed contract).
rm -f "$GOLDEN"/*.rs "$GOLDEN"/*.h "$GOLDEN"/*.hpp "$GOLDEN"/*.py "$GOLDEN"/*.js "$GOLDEN"/*_bridge.rs
cargo run -q -p modukit-codegen --bin protoc-gen-modukit -- \
  "@contracts/org.modukit.runtime.proto.fdset" \
  "@contracts/com.modukit.demo.proto.fdset" \
  --embed_header contracts/modukit_embed.h \
  --py_out "$GOLDEN" \
  --js_out "$GOLDEN" \
  --plugin_header contracts/modukit_plugin.h \
  --output_dir "$GOLDEN"
echo "ok regen-golden -> $GOLDEN"
