#!/usr/bin/env bash
# leg: lane-table reconciliation (benchmark plan T2.3, batch 2).
#
# cmake/ModuKitPolyOrchCrates.cmake registers the cargo lanes as the
# MODUKIT_RUST_LANES table, one row per "<package>@<folder>@<kind>@<libname>"
# (rows live at :22-27); root Cargo.toml owns the workspace membership. Two
# tables, one truth - this gate reconciles them:
#  (a) every lane row's folder must be a workspace member and its package
#      name must match the member directory - a ghost lane is fatal;
#  (b) every crates/* member must have a lane OR sit on the LANE_EXCEPTIONS
#      list below with a one-line reason - an unregistered member is fatal.
# RED iff either side drifts; anchor drift (either extraction empty) is RED
# too, same grammar as the sibling check scripts.
#
# Self-test (T2.3): copies the cmake lane table and Cargo.toml to a scratch
# dir, plants a ghost lane row and an unregistered crates/* member, re-runs
# against the scratch copy, and requires BOTH defects in the red output.
# Repo files are never touched. MODUKIT_GATE_INPUT_DIR redirects only the
# two parsed inputs; the repo root stays the working dir either way.
set -uo pipefail
SELF=$(realpath "$0") # absolute for the self-test re-exec below
cd "$(dirname "$0")/.."
IN="${MODUKIT_GATE_INPUT_DIR:-}"; [ -n "$IN" ] && IN="${IN%/}/"

# crates/* members that ride no lane by design (lookup matches the leading
# "<pkg> " token; the tail is the one-line reason).
LANE_EXCEPTIONS=(
  "modukit-runtime # CLI bins (modukit-run + modukit-host), no lane by design"
  "modukit-js # bin-like node-side face, D103 coverage-boundary exemption"
)

if [ "${1:-}" = "--self-test" ]; then
  s=$(mktemp -d)
  trap 'rm -rf "$s"' EXIT
  mkdir -p "$s/cmake"
  cp cmake/ModuKitPolyOrchCrates.cmake "$s/cmake/"
  cp Cargo.toml "$s/"
  # ghost lane: folder is not a workspace member
  sed -i '/^set(MODUKIT_RUST_LANES/,/)/{s/^\(.*"\))$/\1,\n    "ghost-lane@crates\/ghost-lane@lib@ghost_lane")/;}' "$s/cmake/ModuKitPolyOrchCrates.cmake"
  # unregistered member: crates/* with no lane and no exception row
  sed -i '/^members = /s/"\]$/", "crates\/sneaky-unlisted"]/' "$s/Cargo.toml"
  out=$(MODUKIT_GATE_INPUT_DIR="$s" bash "$SELF" 2>&1)
  rc=$?
  if [ "$rc" -eq 0 ]; then
    echo "self-test FAIL (check-lane-table.sh): planted defects NOT caught"
    exit 1
  fi
  if ! printf '%s\n' "$out" | grep -qF "ghost lane: ghost-lane" \
     || ! printf '%s\n' "$out" | grep -qF "no lane or exception: crates/sneaky-unlisted"; then
    echo "self-test FAIL (check-lane-table.sh): only one planted defect caught"
    printf '%s\n' "$out"
    exit 1
  fi
  echo "self-test OK (check-lane-table.sh): planted defects caught"
  exit 0
fi

LANES_CMAKE="${IN}cmake/ModuKitPolyOrchCrates.cmake"
CARGO_TOML="${IN}Cargo.toml"

for f in "$LANES_CMAKE" "$CARGO_TOML"; do
  if [ ! -f "$f" ]; then
    echo "check-lane-table: RED (missing $f)"
    exit 1
  fi
done

lanes=$(sed -n '/^set(MODUKIT_RUST_LANES/,/)/p' "$LANES_CMAKE" | grep -oE '"[^"]+"' | tr -d '"')
members=$(sed -n '/^members = /p' "$CARGO_TOML" | grep -oE '"[^"]+"' | tr -d '"')

if [ -z "$lanes" ] || [ -z "$members" ]; then
  echo "check-lane-table: RED (lane or member extraction empty - anchor drift?)"
  exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
printf '%s\n' "$members" > "$tmp/members.txt"
: > "$tmp/folders.txt"
fail=0

# ---- (a) lane rows -> workspace members ----
while IFS='@' read -r pkg folder kind lib; do
  [ -n "$pkg" ] || continue
  echo "$folder" >> "$tmp/folders.txt"
  if ! grep -qxF "$folder" "$tmp/members.txt"; then
    echo "check-lane-table: RED (ghost lane: $pkg -> $folder is not a workspace member)"
    fail=1
  fi
  base=${folder##*/}
  if [ "$pkg" != "$base" ]; then
    echo "check-lane-table: RED (lane name drift: pkg $pkg != member dir $base)"
    fail=1
  fi
done <<< "$lanes"

# ---- (b) crates/* members -> lane row or exception ----
while IFS= read -r member; do
  case "$member" in crates/*) ;; *) continue ;; esac
  if grep -qxF "$member" "$tmp/folders.txt"; then
    continue
  fi
  pkg=${member##*/}
  exception=""
  for row in "${LANE_EXCEPTIONS[@]}"; do
    case "$row" in "$pkg "*) exception="$row"; break ;; esac
  done
  if [ -z "$exception" ]; then
    echo "check-lane-table: RED (no lane or exception: $member)"
    fail=1
  fi
done <<< "$members"

if [ "$fail" -eq 0 ]; then
  echo "check-lane-table: ok ($(wc -l < "$tmp/folders.txt") lanes <-> $(wc -l < "$tmp/members.txt") members reconciled, ${#LANE_EXCEPTIONS[@]} exception rows)"
fi
exit "$fail"
