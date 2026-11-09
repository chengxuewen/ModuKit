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
# --self-test (adjudication #35 / backlog P11): scratch git repo via
# MODUKIT_MEMORY_REPO walks all four outcomes - non-durable green, durable
# without memory red, the ' - Memory: none (<reason>)' bullet green, and a
# memory change green. Repo files never touched.
set -uo pipefail
SELF=$(realpath "$0") # absolute for the self-test re-exec below
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."
REPO="${MODUKIT_MEMORY_REPO:-.}"

if [ "${1:-}" = "--self-test" ]; then
  s=$(mktemp -d)
  trap 'rm -rf "$s"' EXIT
  git init -q "$s"
  git -C "$s" config user.email probe@example.invalid
  git -C "$s" config user.name probe
  git -C "$s" commit --allow-empty -qm "chore: scratch root"
  # 1) non-durable-only range: green
  echo x > "$s/notes.txt"
  git -C "$s" add -A && git -C "$s" commit -qm "chore: scratch note"
  MODUKIT_MEMORY_REPO="$s" bash "$SELF" HEAD~1..HEAD >/dev/null 2>&1 \
    || { echo "self-test FAIL (check-memory.sh): non-durable range should be green"; exit 1; }
  # 2) durable change without memory and without the bullet: red
  mkdir -p "$s/scripts" && echo x > "$s/scripts/x.sh"
  git -C "$s" add -A && git -C "$s" commit -qm "feat: scratch script"
  out=$(MODUKIT_MEMORY_REPO="$s" bash "$SELF" HEAD~1..HEAD 2>&1); rc=$?
  [ "$rc" -ne 0 ] || { echo "self-test FAIL (check-memory.sh): missing disposition NOT caught"; exit 1; }
  case "$out" in *"no memory change"*) ;; *) echo "self-test FAIL (check-memory.sh): red face not reported"; exit 1;; esac
  # 3) durable change with the ' - Memory: none (<reason>)' bullet: green
  echo y >> "$s/scripts/x.sh"
  git -C "$s" add -A && git -C "$s" commit -qm "feat: scratch script again" -m " - Memory: none (self-test probe)"
  out=$(MODUKIT_MEMORY_REPO="$s" bash "$SELF" HEAD~1..HEAD 2>&1)
  case "$out" in *"explicit none"*) ;; *) echo "self-test FAIL (check-memory.sh): none-bullet not honoured"; exit 1;; esac
  # 4) durable change plus a memory entry: green
  mkdir -p "$s/.agents/memorys" && echo x > "$s/.agents/memorys/status.md"
  echo z >> "$s/scripts/x.sh"
  git -C "$s" add -A && git -C "$s" commit -qm "docs: scratch memory"
  out=$(MODUKIT_MEMORY_REPO="$s" bash "$SELF" HEAD~1..HEAD 2>&1)
  case "$out" in *"memory updated"*) ;; *) echo "self-test FAIL (check-memory.sh): memory change not honoured"; exit 1;; esac
  echo "self-test OK (check-memory.sh): four-way disposition walk all green/red as drawn"
  exit 0
fi

RANGE="${1:-}"
if [ -z "$RANGE" ]; then
    echo "usage: bash scripts/check-memory.sh <base>..HEAD" >&2
    exit 2
fi
if ! git -C "$REPO" rev-list -n1 "$RANGE" >/dev/null 2>&1; then
    echo "check-memory: invalid range: $RANGE" >&2
    exit 2
fi

DURABLE=(
    crates contracts bindings scripts book docs
    .agents/rules .agents/skills
    pixi.toml CMakeLists.txt cmake .github .vscode README.md
)

durable=$(git -C "$REPO" diff --name-only "$RANGE" -- "${DURABLE[@]}" 2>/dev/null || true)
if [ -z "$durable" ]; then
    echo "MEMORY ✓ no durable paths in $RANGE"
    exit 0
fi

memory=$(git -C "$REPO" diff --name-only "$RANGE" -- .agents/memorys 2>/dev/null || true)
if [ -n "$memory" ]; then
    echo "MEMORY ✓ memory updated ($(printf '%s\n' "$memory" | grep -c .) file(s))"
    exit 0
fi

if git -C "$REPO" log --format=%B "$RANGE" 2>/dev/null | grep -Eq '^ - Memory: none \(.+\)$'; then
    echo "MEMORY ✓ explicit none disposition"
    exit 0
fi

echo "MEMORY ✗ $RANGE touches durable paths but has no memory change"
printf '%s\n' "$durable" | head -5
echo "  add a .agents/memorys/ entry or a ' - Memory: none (<reason>)' bullet"
exit 1
