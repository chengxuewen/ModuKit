#!/usr/bin/env bash
# C5 batch-close memory disposition gate.
#
# Usage: bash scripts/check-memory.sh <base>..<head>
#
# Pass when the range either:
#   - touches no durable path, or
#   - changes .agents/memorys/, or
#   - carries a ` - Memory: none (<reason>)` bullet in a commit body.
# A batch that changes code/docs but leaves memory untouched is red.
#
# Deliberately NOT part of scripts/check.sh: per-commit memory churn is not
# required (status.md is capped at 8 KB); run this at batch/PR close.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."

RANGE="${1:-}"
if [ -z "$RANGE" ]; then
    echo "usage: bash scripts/check-memory.sh <base>..HEAD" >&2
    exit 2
fi
if ! git rev-list -n1 "$RANGE" >/dev/null 2>&1; then
    echo "check-memory: invalid range: $RANGE" >&2
    exit 2
fi

DURABLE=(
    crates contracts bindings scripts book docs
    .agents/rules .agents/skills
    pixi.toml CMakeLists.txt cmake .github .vscode README.md
)

durable=$(git diff --name-only "$RANGE" -- "${DURABLE[@]}" 2>/dev/null || true)
if [ -z "$durable" ]; then
    echo "MEMORY ✓ no durable paths in $RANGE"
    exit 0
fi

memory=$(git diff --name-only "$RANGE" -- .agents/memorys 2>/dev/null || true)
if [ -n "$memory" ]; then
    echo "MEMORY ✓ memory updated ($(printf '%s\n' "$memory" | grep -c .) file(s))"
    exit 0
fi

if git log --format=%B "$RANGE" 2>/dev/null | grep -Eq '^ - Memory: none \(.+\)$'; then
    echo "MEMORY ✓ explicit none disposition"
    exit 0
fi

echo "MEMORY ✗ $RANGE touches durable paths but has no memory change"
printf '%s\n' "$durable" | head -5
echo "  add a .agents/memorys/ entry or a ' - Memory: none (<reason>)' bullet"
exit 1
