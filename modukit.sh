#!/usr/bin/env bash
# modukit.sh -- execution front door (human convenience; agents/CI keep using `pixi run`).
#
# Pure passthrough:  ./modukit.sh <task> [args...]   ==   pixi run <task> [args...]
# There is no command table here on purpose: pixi.toml is the single source of
# task names, so this file cannot drift when a task is added or renamed.
#
# Not source-able -- use `source pixi.sh` to activate a shell instead.
#
# Exit codes owned by this file: -h/--help -> 0, no args -> 2, pixi missing -> 127.
# Everything else is the task's own exit code.

set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
cd "$ROOT"

USAGE='usage: ./modukit.sh <task> [args...]

common:
  ./modukit.sh mcp status      MCP: pin/installed versions + index state
  ./modukit.sh mcp setup       install/repair MCP packages at the pin
  ./modukit.sh mcp sync        incremental MCP sync
  ./modukit.sh mcp index       build (first run) or rebuild .codegraph
  ./modukit.sh agents sync     agent config sync (rules/mcp/memory/skills)
  ./modukit.sh agents check    verify every generated layer (non-zero on drift)
  ./modukit.sh gate-agents     single-source consistency gate
  ./modukit.sh ci              the whole local gate

docs (the book, output in www/book/en):
  ./modukit.sh book-build      build the site (--public drops internal pages)
  ./modukit.sh book-serve      serve it on :3000 and rebuild on change
  ./modukit.sh book-check      static checks on the built site
  ./modukit.sh book-graph      refresh graph.json/meta.json only
  ./modukit.sh spec-trace      SDD clauses <-> spec tags

  pixi task list               every task this workspace defines

`pixi run <task>` is always equivalent; Windows: modukit.bat <task>.'

case "${1-}" in
    '') echo "$USAGE"; exit 2 ;;
    -h|--help) echo "$USAGE"; exit 0 ;;
esac

# Resolve pixi the same way scripts/mcp-launch.sh does: PATH first, then ~/.pixi/bin.
PIXI="$(command -v pixi || true)"
[ -z "$PIXI" ] && [ -x "$HOME/.pixi/bin/pixi" ] && PIXI="$HOME/.pixi/bin/pixi"
[ -z "$PIXI" ] && { echo "modukit.sh: pixi not found (run bootstrap.sh)" >&2; exit 127; }

exec "$PIXI" run "$@"
