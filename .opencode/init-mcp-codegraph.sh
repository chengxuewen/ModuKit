#!/bin/sh
# init-mcp-codegraph.sh — auto install/init/serve CodeGraph MCP (replaces the
# old node .mjs wrapper, which crashed: this box's system node is v10 < ESM).
# Runs at every opencode session start as the mcp.local-codegraph command.
#
# Flow: ensure pinned CLI (.opencode/codegraph) -> ensure project graph
#       (.codegraph/) -> exec MCP server on stdio.
# Portable: Linux/macOS via this sh script. Windows-native: see
#           init-mcp-codegraph.bat twin (select via local opencode.json edit;
#           opencode config has no OS-conditionals).

set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BIN="$ROOT/.opencode/bin"
CG="$BIN/codegraph"
CODEGRAPH_VERSION="${CODEGRAPH_VERSION:-v1.6.2}"   # pinned; bump deliberately, then commit
INSTALLER_URL="https://raw.githubusercontent.com/colbymchenry/codegraph/main/install.sh"

if [ ! -x "$CG" ]; then
    echo "[codegraph] installing $CODEGRAPH_VERSION into .opencode/ ..." >&2
    curl -fsSL "$INSTALLER_URL" | \
        CODEGRAPH_VERSION="$CODEGRAPH_VERSION" \
        CODEGRAPH_INSTALL_DIR="$ROOT/.opencode/codegraph" \
        CODEGRAPH_BIN_DIR="$BIN" sh
fi

if [ ! -d "$ROOT/.codegraph" ]; then
    echo "[codegraph] no project graph yet — running init (empty repo = instant) ..." >&2
    ( cd "$ROOT" && "$CG" init ) >&2 || \
        echo "[codegraph] WARNING: init failed; serve will start with an empty/stale graph" >&2
fi

exec "$CG" serve --mcp
