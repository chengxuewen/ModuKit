#!/usr/bin/env bash
# warm-lanes.sh - the IDE/cold-build warm step WITH stamp-vs-artifact
# self-heal (pixi task `warm`; the durable home of the m1/m2 leg heal).
#
# Why this exists: the cargo .a files are plain ninja inputs with no
# producer edge (cargo rides per-tree stamps), so a single-target build
# dies at ninja PARSE time when an artifact is missing - and `cargo
# clean` (or a wiped build/cargo-target, e.g. a true-cold rehearsal)
# removes the artifacts while leaving the stamps FRESH. A plain
# `cmake --build --target modukit-rust-build` then reports "no work to
# do" with the .a still missing. The heal: a missing artifact makes its
# tree's stamps stale lies - remove them and let the cargo rule
# re-materialize the artifacts.
#
# Side mapping (the shared cargo cache, one CARGO_TARGET_DIR for all
# trees): release/ is produced by the RelWithDebInfo preset trees
# (default, release, system-release); debug/ by the bare/debug trees
# (demo, debug, system, system-debug).
set -uo pipefail
cd "$(dirname "$0")/.."

bust() { # <artifact> <tree...>
  local art=$1
  shift
  if [ ! -f "$art" ]; then
    echo "warm-lanes: $art missing - busting stale lane stamps"
    local t
    for t in "$@"; do
      [ -d "build/presets/$t" ] && rm -f "build/presets/$t"/.polyorch-rust-stamps/*.stamp
    done
  fi
}

bust build/cargo-target/release/libmodukit_core.a default release system-release
bust build/cargo-target/debug/libmodukit_core.a demo debug system system-debug

pixi run cmake --build --preset default --target modukit-rust-build
