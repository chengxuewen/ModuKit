#!/usr/bin/env bash
# modukit.sh — execution front door (human convenience; agents/CI keep using `pixi run`).
#
# Pure passthrough:  ./modukit.sh <task> [args...]   ==   pixi run <task> [args...]
# There is no command table here on purpose: pixi.toml is the single source of
# task names, so this file cannot drift when a task is added or renamed.
#
# Not source-able -- use `source pixi.sh` to activate a shell instead.
# Exit codes owned by this file: -h/--help -> 0, no args -> 2, pixi missing -> 127.

set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
cd "$ROOT"

USAGE='usage: ./modukit.sh <task> [args...]

common:
  ./modukit.sh ci            the local gate (== bash scripts/check.sh)
  ./modukit.sh agents sync   regenerate the agent layers
  ./modukit.sh agents check  verify every generated layer
  ./modukit.sh mcp status    MCP pin/index state
  ./modukit.sh docs-gen      regenerate llms.txt
  pixi task list             every task this workspace defines

`pixi run <task>` is always equivalent; Windows: modukit.bat <task>.'

case "${1-}" in
    '') echo "$USAGE"; exit 2 ;;
    -h|--help) echo "$USAGE"; exit 0 ;;
esac

PIXI="$(command -v pixi || true)"
[ -z "$PIXI" ] && [ -x "$HOME/.pixi/bin/pixi" ] && PIXI="$HOME/.pixi/bin/pixi"
[ -z "$PIXI" ] && { echo "modukit.sh: pixi not found (run bootstrap.sh)" >&2; exit 127; }

exec "$PIXI" run "$@"
