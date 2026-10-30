#!/usr/bin/env bash
# MSRV gate (B4): the workspace `rust-version` and the pixi msrv pin are one
# promise. A mismatch is red; a missing msrv env is a loud SKIP, never a pass.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."

field=$(sed -n 's/^rust-version = "\([^"]*\)".*/\1/p' Cargo.toml | head -1)
pin=$(awk '/^\[feature\.msrv\.dependencies\]/{f=1; next} /^\[/{f=0} f && /^rust[ \t]*=/{print}' pixi.toml \
    | sed -E 's/.*"=?([0-9.]+)".*/\1/')

[ -n "$field" ] || { echo "MSRV ✗ Cargo.toml has no rust-version"; exit 1; }
[ -n "$pin" ] || { echo "MSRV ✗ pixi.toml has no msrv rust pin"; exit 1; }
[ "$field" = "$pin" ] || { echo "MSRV ✗ Cargo.toml rust-version=$field != msrv pin=$pin"; exit 1; }

if ! command -v pixi >/dev/null 2>&1; then
    echo "MSRV SKIP (no pixi)"
    exit 0
fi
if [ ! -x .pixi/envs/msrv/bin/cargo ]; then
    echo "MSRV SKIP (env not installed; pixi install -e msrv)"
    exit 0
fi
if pixi run -e msrv cargo check --workspace >/dev/null 2>&1; then
    echo "MSRV ✓ rust-version=$field == msrv pin; cargo check --workspace green"
else
    echo "MSRV ✗ cargo check --workspace failed under $pin"
    exit 1
fi
