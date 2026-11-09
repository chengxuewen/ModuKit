#!/usr/bin/env bash
# MSRV gate (B4): the workspace `rust-version` and the pixi msrv pin are one
# promise. A mismatch is red; a missing msrv env is a loud SKIP, never a pass.
# --self-test (adjudication #35 / backlog P8): scratch root via MSRV_ROOT with
# a missing rust-version and a planted version mismatch (both must red), then
# a matching scratch pair - green via the loud env-absent SKIP, because a
# scratch root never carries .pixi/envs/msrv. Repo files never touched.
set -uo pipefail
SELF=$(realpath "$0") # absolute for the self-test re-exec below
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
ROOT="${MSRV_ROOT:-.}"

if [ "${1:-}" = "--self-test" ]; then
  s=$(mktemp -d)
  trap 'rm -rf "$s"' EXIT
  printf '[workspace]\n' > "$s/Cargo.toml"
  printf '[feature.msrv.dependencies]\nrust = "=9.9.9"\n' > "$s/pixi.toml"
  out=$(MSRV_ROOT="$s" bash "$SELF" 2>&1)
  case "$out" in *"has no rust-version"*) ;; *) echo "self-test FAIL (check-msrv.sh): missing rust-version not reported"; exit 1;; esac
  printf 'rust-version = "1.88"\n' >> "$s/Cargo.toml"
  out=$(MSRV_ROOT="$s" bash "$SELF" 2>&1); rc=$?
  [ "$rc" -ne 0 ] || { echo "self-test FAIL (check-msrv.sh): planted mismatch NOT caught"; exit 1; }
  case "$out" in *"!= msrv pin"*) ;; *) echo "self-test FAIL (check-msrv.sh): mismatch not reported"; exit 1;; esac
  printf '[feature.msrv.dependencies]\nrust = "=1.88"\n' > "$s/pixi.toml"
  MSRV_ROOT="$s" bash "$SELF" >/dev/null 2>&1 \
    || { echo "self-test FAIL (check-msrv.sh): matching pair should be green"; exit 1; }
  echo "self-test OK (check-msrv.sh): missing field and mismatch red; matching pair green"
  exit 0
fi

field=$(sed -n 's/^rust-version = "\([^"]*\)".*/\1/p' "$ROOT/Cargo.toml" | head -1)
pin=$(awk '/^\[feature\.msrv\.dependencies\]/{f=1; next} /^\[/{f=0} f && /^rust[ \t]*=/{print}' "$ROOT/pixi.toml" \
    | sed -E 's/.*"=?([0-9.]+)".*/\1/')

[ -n "$field" ] || { echo "MSRV ✗ Cargo.toml has no rust-version"; exit 1; }
[ -n "$pin" ] || { echo "MSRV ✗ pixi.toml has no msrv rust pin"; exit 1; }
[ "$field" = "$pin" ] || { echo "MSRV ✗ Cargo.toml rust-version=$field != msrv pin=$pin"; exit 1; }

if ! command -v pixi >/dev/null 2>&1; then
    echo "MSRV SKIP (no pixi)"
    exit 0
fi
if [ ! -x "$ROOT/.pixi/envs/msrv/bin/cargo" ]; then
    echo "MSRV SKIP (env not installed; pixi install -e msrv)"
    exit 0
fi
if pixi run -e msrv cargo check --workspace >/dev/null 2>&1; then
    echo "MSRV ✓ rust-version=$field == msrv pin; cargo check --workspace green"
else
    echo "MSRV ✗ cargo check --workspace failed under $pin"
    exit 1
fi
