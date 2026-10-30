#!/usr/bin/env bash
# cmake-smoke (B4b-2): three-state cold-tree gate.
#   no cmake            -> SKIP (exit 0)
#   default             -> configure + build + ctest + two negative paths
#   --configure-only    -> configure + two negative paths (the ci leg)
# Negative paths: (1) missing PolyOrch submodule; (2) stale PolyOrch version
# floor. The `MODUKIT_USE_PIXI=OFF`/no-cargo probe is not falsifiable here because
# PolyOrch resolves the pixi cargo by absolute path, so the version-floor probe
# takes its place (recorded in docs/ve-parity-plan.md B4b-2).
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."

MODE=full
[ "${1:-}" = "--configure-only" ] && MODE=configure

ENV_BIN="$PWD/.pixi/envs/default/bin"
CM="$ENV_BIN/cmake"
[ -x "$CM" ] || CM=$(command -v cmake || true)
if [ -z "$CM" ] || [ ! -x "$CM" ]; then
    echo "CMAKE-SMOKE SKIP (no cmake; run pixi install first)"
    exit 0
fi
CT="$(dirname "$CM")/ctest"
export PATH="$ENV_BIN:$HOME/.pixi/bin:$PATH"
# Mirror pixi activation: the conda compiler is required by the abseil headers
# that ship with protobuf 7.x; the system g++ on Ubuntu 20.04 is too old.
if [ -x "$ENV_BIN/x86_64-conda-linux-gnu-cc" ]; then
    export CC="$ENV_BIN/x86_64-conda-linux-gnu-cc"
    export CXX="$ENV_BIN/x86_64-conda-linux-gnu-c++"
fi

B=out/cmake-smoke
NEG1=out/cmake-smoke-neg-polyorch
NEG2=out/cmake-smoke-neg-floor
HOST=out/cmake-smoke-neg-floor-host
rm -rf "$B" "$NEG1" "$NEG2" "$HOST"

configure() {  # $1 = build dir, $2 = log
    "$CM" -S . -B "$1" -G Ninja -DCMAKE_BUILD_TYPE=Debug >"$2" 2>&1
}

if ! configure "$B" out/cmake-smoke-configure.log; then
    echo "CMAKE-SMOKE ✗ bare configure"
    tail -5 out/cmake-smoke-configure.log
    exit 1
fi

if [ "$MODE" = "full" ]; then
    if ! "$CM" --build "$B" >out/cmake-smoke-build.log 2>&1; then
        echo "CMAKE-SMOKE ✗ build"
        tail -5 out/cmake-smoke-build.log
        exit 1
    fi
    if ! "$CT" --test-dir "$B" --output-on-failure >out/cmake-smoke-ctest.log 2>&1; then
        echo "CMAKE-SMOKE ✗ ctest"
        tail -8 out/cmake-smoke-ctest.log
        exit 1
    fi
fi

# Negative 1: the submodule must be a hard, readable failure.
if [ -d PolyOrch ]; then
    mv PolyOrch PolyOrch.smoke-hidden
    trap 'mv "$PWD/PolyOrch.smoke-hidden" "$PWD/PolyOrch" 2>/dev/null || true' EXIT INT TERM
    configure "$NEG1" out/cmake-smoke-neg-polyorch.log
    rc1=$?
    mv PolyOrch.smoke-hidden PolyOrch 2>/dev/null || true
    trap - EXIT INT TERM
    rm -rf "$NEG1"
    if [ "$rc1" -eq 0 ] || ! grep -q "PolyOrch submodule missing" out/cmake-smoke-neg-polyorch.log; then
        echo "CMAKE-SMOKE ✗ missing-PolyOrch negative path did not state the fix"
        exit 1
    fi
fi

# Negative 2: a stale PolyOrch sentinel must trip the 0.7.1 floor with the
# actionable message (the supplier CTEST keyword is what the lanes depend on).
mkdir -p "$HOST"
cat > "$HOST/CMakeLists.txt" <<'EOF'
cmake_minimum_required(VERSION 3.25)
project(neg_floor LANGUAGES CXX)
add_library(PolyOrch::PolyOrch INTERFACE IMPORTED GLOBAL)
set_property(TARGET PolyOrch::PolyOrch PROPERTY INTERFACE_POLYORCH_VERSION "0.1.1")
function(polyorch_rust_setup)
endfunction()
add_subdirectory("${CMAKE_CURRENT_LIST_DIR}/../.." modukit)
EOF
"$CM" -S "$HOST" -B "$NEG2" -G Ninja >out/cmake-smoke-neg-floor.log 2>&1
rc2=$?
rm -rf "$NEG2" "$HOST"
if [ "$rc2" -eq 0 ] || ! grep -q "needs PolyOrch >= 0.7.1" out/cmake-smoke-neg-floor.log; then
    echo "CMAKE-SMOKE ✗ stale-PolyOrch floor negative path did not fire"
    exit 1
fi

if [ "$MODE" = "full" ]; then
    echo "CMAKE-SMOKE ✓ (cold configure/build/ctest + missing-PolyOrch + version-floor)"
else
    echo "CMAKE-SMOKE ✓ (configure-only + missing-PolyOrch + version-floor)"
fi
