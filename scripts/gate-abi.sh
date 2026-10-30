#!/usr/bin/env bash
# gate-abi: the exported modukit_* symbol set must equal contracts/modukit_embed.h.
# Builds the core cdylib, reads dynamic symbols with nm, and compares exact sets
# (a missing symbol or an accidental extra is red). The real C++ demo run is owned
# by demo-stage4-m1; this gate owns the symbol set only.
# SKIP when nm is absent or the platform uses a different nm dialect.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
HEADER="${HEADER:-contracts/modukit_embed.h}"

if [ "${1:-}" = "--self-test" ]; then
  cp "$HEADER" out/embed-selftest.h
  printf '\nvoid modukit_self_test(void);\n' >> out/embed-selftest.h
  if HEADER=out/embed-selftest.h bash "$0" >/dev/null 2>&1; then
    rm -f out/embed-selftest.h
    echo "GATE-ABI ✗ self-test did not red"
    exit 1
  fi
  rm -f out/embed-selftest.h
  echo "GATE-ABI ✓ self-test (planted symbol red)"
  exit 0
fi

NM=$(command -v nm || command -v x86_64-conda-linux-gnu-nm || true)
if [ -z "$NM" ]; then
  echo "GATE-ABI SKIP (no nm)"
  exit 0
fi
case "$(uname -s)" in
  Linux) SO=target/debug/libmodukit_core.so ;;
  *) echo "GATE-ABI SKIP (non-Linux nm dialect)"; exit 0 ;;
esac

if command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
else
  CARGO=(cargo)
fi
if ! "${CARGO[@]}" build -p modukit-core >/dev/null 2>&1; then
  echo "GATE-ABI ✗ cargo build -p modukit-core failed"
  exit 1
fi
if [ ! -f "$SO" ]; then
  echo "GATE-ABI ✗ missing $SO"
  exit 1
fi

want=$(grep -oE 'modukit_[a-z0-9_]+\(' "$HEADER" | sed 's/(//' | sort -u)
have=$("$NM" -D --defined-only "$SO" | awk '{print $3}' | grep -E '^modukit_' | sort -u)
missing=$(comm -23 <(printf '%s\n' "$want") <(printf '%s\n' "$have"))
extra=$(comm -13 <(printf '%s\n' "$want") <(printf '%s\n' "$have"))

if [ -n "$missing" ] || [ -n "$extra" ]; then
  echo "GATE-ABI ✗ exported symbol set differs from $HEADER"
  [ -n "$missing" ] && printf 'missing:\n%s\n' "$missing"
  [ -n "$extra" ] && printf 'extra:\n%s\n' "$extra"
  exit 1
fi
count=$(printf '%s\n' "$want" | grep -c .)
echo "GATE-ABI ✓ ${count} exported modukit_* symbols match the header"
