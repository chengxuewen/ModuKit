#!/usr/bin/env bash
# demo.sh - one key for the demo gallery (onboarding plan E1, ruling A3).
#
# usage: scripts/demo.sh [leg] [args...]
#   (bare)            -> stage1, the 60-second full lifecycle
#   <leg> [args...]   -> scripts/demo-<leg>.sh (e.g. stage4-m1, wave2-py)
#   -l|--list         -> list every gallery leg
# An unknown leg lists the gallery on stderr and exits 2 (usage-code shape).
set -euo pipefail
cd "$(dirname "$0")/.."

legs() {
  for f in scripts/demo-*.sh; do
    basename "$f" .sh | sed 's/^demo-//'
  done
}

case "${1-}" in
  '')
    exec bash scripts/demo-stage1.sh
    ;;
  -l | --list)
    echo "gallery legs:"
    legs | sed 's/^/  /'
    ;;
  *)
    leg=$1
    shift
    if [ -f "scripts/demo-$leg.sh" ]; then
      exec bash "scripts/demo-$leg.sh" "$@"
    fi
    echo "demo: unknown leg '$leg' - the gallery:" >&2
    legs | sed 's/^/  /' >&2
    exit 2
    ;;
esac
