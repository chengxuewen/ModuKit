#!/usr/bin/env bash
# mcp-browser-probe.sh -- prove that local-playwright can actually open a page.
#
# The gate only reads the resolution record (never launches a browser); this probe
# is the one C0-closable evidence that the resolved browser WORKS end to end:
# it drives the pinned server through the real launcher with a JSON-RPC handshake
# and one `browser_navigate` call, then asserts the response carries a page
# snapshot. Plan of record: docs/mcp-browser-plan.md (W5).
#
# Usage: bash scripts/mcp-browser-probe.sh [url]     (default: about:blank)
# Exit:  0 = the browser opened the page; 1 = no usable browser / no snapshot.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 1

URL="${1:-about:blank}"
OUT_DIR="${MODUKIT_MCP_PROBE_OUT:-out/mcp-probe}"
TIMEOUT="${MODUKIT_MCP_PROBE_TIMEOUT:-120}"
mkdir -p "$OUT_DIR"
LOG="$OUT_DIR/probe.log"
rm -f "$LOG"

# Node resolution: the repo's pixi env first (the pinned Node the MCP packages were
# installed for), then PATH. A system node that predates ESM would fail confusingly.
NODE=""
if [ -x ".pixi/envs/default/bin/node" ]; then
    NODE=".pixi/envs/default/bin/node"
else
    NODE="$(command -v node 2>/dev/null || true)"
fi
if [ -z "$NODE" ]; then
    echo "PROBE ✗ node not found (run via: pixi run mcp-probe)" >&2
    exit 127
fi

{
  printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"mcp-browser-probe","version":"1"}}}'
  printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized"}'
  printf '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"browser_navigate","arguments":{"url":"%s"}}}\n' "$URL"
  # Hold stdin open long enough for the browser to launch and answer, then close so
  # the server exits cleanly (its own idle timeout is far longer than this probe).
  sleep 20
} | timeout "$TIMEOUT" "$NODE" scripts/agents/mcp-serve.mjs \
        --from scripts/agents/mcp-servers.json --name local-playwright \
        >"$LOG" 2>&1
CODE=$?

if grep -q '"id":2' "$LOG" 2>/dev/null && grep -q 'Snapshot' "$LOG" 2>/dev/null; then
    echo "PROBE ✓ local-playwright opened $URL (snapshot captured)"
    echo "       log: $LOG"
    exit 0
fi

echo "PROBE ✗ no page snapshot from local-playwright (exit $CODE)" >&2
if grep -q 'no browser resolution' "$LOG" 2>/dev/null; then
    echo "       fix: pixi run mcp setup" >&2
elif grep -q '"isError":true' "$LOG" 2>/dev/null; then
    echo "       the server reported an error; first lines:" >&2
    grep -o '"text":"[^"]\{0,200\}' "$LOG" | head -3 >&2
else
    echo "       last lines:" >&2
    tail -5 "$LOG" >&2
fi
exit 1