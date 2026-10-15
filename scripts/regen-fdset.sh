#!/usr/bin/env bash
# D12 ledger regen: compile every hand-written contracts/*.proto to its
# checked-in fdset (protoc-native --descriptor_set_out, D13: no exporter).
# OUT_DIR arg lets scripts/check.sh regen into a temp dir and diff against
# the checked-in copies (empty diff or red).
#
# protoc comes from the pinned pixi env (D65): run as `pixi run
# bash scripts/regen-fdset.sh`, or directly when the env is already active.
set -euo pipefail
cd "$(dirname "$0")/.."

OUT_DIR="${1:-contracts}"

# D65: protoc must come from the pinned pixi env, never the system one. If we
# are not already inside a pixi env, re-exec under it; only a machine with no
# pixi at all falls back to a system protoc (PROTOC=... overrides the name).
if [ -z "${CONDA_PREFIX:-}" ] && [ -z "${MODUKIT_REGEN_INNER:-}" ] && [ -f pixi.toml ]; then
  export PATH="$HOME/.pixi/bin:$PATH"
  if command -v pixi >/dev/null 2>&1; then
    export MODUKIT_REGEN_INNER=1
    exec pixi run bash "$0" "$OUT_DIR"
  fi
fi

PROTOC="${PROTOC:-protoc}"
if ! command -v "$PROTOC" >/dev/null 2>&1; then
  echo "FAIL regen-fdset: protoc unavailable (no pixi env, no system protoc)" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
shopt -s nullglob
found=0
for proto in contracts/*.proto; do
  base=$(basename "$proto" .proto)
  # Skip the option-vocabulary file: it declares only (modukit.*) extensions,
  # carries no contract, and its fdset would never be read by the generator.
  [ "$base" = "modukit" ] && continue
  found=1
  "$PROTOC" -I contracts --descriptor_set_out="$OUT_DIR/$base.proto.fdset" "$proto"
done
[ "$found" -eq 1 ] || { echo "FAIL regen-fdset: no contracts/*.proto found" >&2; exit 1; }
echo "ok regen-fdset ($OUT_DIR)"
