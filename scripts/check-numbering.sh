#!/usr/bin/env bash
# Numbering gate (B0, D109): the decision ledger's max D is the only high-water
# source. The script asserts:
#   1. every "D1-D<n>" claim in AGENTS.md and status.md equals that max;
#   2. AGENTS.md's "high-water today D<n>" equals that max;
#   3. docs/architecture.md section 11 carries a row for that max.
# --self-test copies the four files to a temp tree, plants D<max+1>, expects red,
# restores, expects green (C0: the gate is observed failing once).
set -uo pipefail
cd "$(dirname "$0")/.."
ROOT="${NUMBERING_ROOT:-$PWD}"
fail=0

if [ "${1:-}" = "--self-test" ]; then
  mkdir -p "$ROOT/out"
  tmp=$(mktemp -d "$ROOT/out/numbering.XXXXXX") || {
    echo "NUMBERING ✗ self-test: cannot create a temp dir under $ROOT/out"
    exit 1
  }
  mkdir -p "$tmp/.agents/memorys" "$tmp/docs"
  cp "$ROOT/.agents/memorys/decisions.md" "$tmp/.agents/memorys/"
  cp "$ROOT/.agents/memorys/status.md" "$tmp/.agents/memorys/"
  cp "$ROOT/AGENTS.md" "$tmp/"
  cp "$ROOT/docs/architecture.md" "$tmp/docs/"
  max=$(grep -oE '^## D[0-9]+:' "$tmp/.agents/memorys/decisions.md" | grep -oE '[0-9]+' | sort -n | tail -1)
  printf '\n## D%d: planted self-test\n' "$((max + 1))" >> "$tmp/.agents/memorys/decisions.md"
  if NUMBERING_ROOT="$tmp" bash "$0" >/dev/null 2>&1; then
    echo "NUMBERING ✗ self-test: planted D$((max + 1)) did not red"
    rm -rf "$tmp"
    exit 1
  fi
  cp "$ROOT/.agents/memorys/decisions.md" "$tmp/.agents/memorys/decisions.md"
  if ! NUMBERING_ROOT="$tmp" bash "$0" >/dev/null 2>&1; then
    echo "NUMBERING ✗ self-test: clean copy did not green"
    rm -rf "$tmp"
    exit 1
  fi
  rm -rf "$tmp"
  echo "NUMBERING ✓ self-test (planted D$((max + 1)) red, clean green)"
  exit 0
fi

max=$(grep -oE '^## D[0-9]+:' "$ROOT/.agents/memorys/decisions.md" | grep -oE '[0-9]+' | sort -n | tail -1)
if [ -z "${max:-}" ]; then
  echo "NUMBERING ✗ no '## D<n>:' entries in decisions.md"
  exit 1
fi

dups=$(grep -oE '^## D[0-9]+:' "$ROOT/.agents/memorys/decisions.md" | sort | uniq -d)
if [ -n "$dups" ]; then
  echo "NUMBERING ✗ duplicate D headings: $dups"
  fail=1
fi

check_claims() {
  local file="$1" hits
  hits=$(grep -oE 'D1-D[0-9]+' "$ROOT/$file" 2>/dev/null | sort -u)
  if [ -z "$hits" ]; then
    echo "NUMBERING ✗ $file: no D1-D<n> claim found"
    fail=1
    return
  fi
  while IFS= read -r claim; do
    [ -n "$claim" ] || continue
    if [ "${claim#D1-D}" != "$max" ]; then
      echo "NUMBERING ✗ $file: $claim != D1-D$max"
      fail=1
    fi
  done <<< "$hits"
}

check_claims AGENTS.md
check_claims .agents/memorys/status.md

hw=$(grep -oE 'high-water today D[0-9]+' "$ROOT/AGENTS.md" | grep -oE '[0-9]+' | head -1)
if [ "${hw:-}" != "$max" ]; then
  echo "NUMBERING ✗ AGENTS.md: high-water today D${hw:-?} != D$max"
  fail=1
fi

if ! grep -qE "^\| D$max \|" "$ROOT/docs/architecture.md"; then
  echo "NUMBERING ✗ docs/architecture.md: section 11 has no D$max row"
  fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "NUMBERING ✓ max D$max; AGENTS/status claims and the section-11 row agree"
fi
exit "$fail"
