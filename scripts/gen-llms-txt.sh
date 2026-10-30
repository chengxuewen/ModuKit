#!/usr/bin/env bash
# gen-llms-txt.sh — emit the agent-facing repository index (llms.txt).
# Committed output; regenerate with `pixi run docs-gen`.
# Missing sources are skipped loudly, never silently.
set -euo pipefail
cd "$(dirname "$0")/.."
OUT=llms.txt

# Rust API reference (VE docs-gen parity): target/doc is merged into the book
# by book-build when present. No doc warnings are promoted here.
cargo doc --workspace --no-deps --quiet

line() { printf '%s\n' "$*"; }
doc() {
  local path="$1" label="$2" note="$3"
  if [ -e "$path" ]; then
    line "- [$label]($path) — $note"
  else
    line "docs-gen: skip $path (absent)" >&2
  fi
}

{
  line "# ModuKit"
  line ""
  line "> Modular plugin host framework & polyglot extension toolkit: Rust kernel,"
  line "> C ABI as the only cross-language exit, generated bindings for C++/Python/JS."
  line ""
  line "## Docs"
  doc AGENTS.md "AGENTS.md" "project facts, conventions, commands, gates"
  doc docs/architecture.md "architecture" "adjudicated baseline (section 11 = decision index)"
  doc docs/whitepaper.md "whitepaper" "positioning and v1.0 canon"
  doc docs/modules/00-overview.md "modules" "reading order for the 00-11 deep dives"
  doc docs/capability-ledger.md "capability ledger" "embed/plugin verb map"
  doc docs/ve-parity-plan.md "VE-parity plan" "non-functional port, B0-B8"
  doc docs/agent-host-parity-plan.md "agent host plan" "host links, skill surface"
  doc docs/reference/README.md "reference roster" "external project profiles"
  doc docs/env.md "environment guide" "pixi, tasks, MCP lifecycle"
  doc target/doc/modukit_core/index.html "rustdoc reference" "generated API documentation (pixi run docs-gen)"
  doc book/toc.toml "book" "mdBook chapter order (B6)"
  doc docs/sdd/ "SDD contracts" "clause/test trace (B6)"
  line ""
  line "## Crates"
  line "- modukit-core — lifecycle state machine, registry, in-process bus, clock, observability"
  line "- modukit-runtime — modukit-run / modukit-host / modukit-debug-reader binaries"
  line "- modukit-transport — narrow transport seam, socketpair/memfd, SCM_RIGHTS fdpass, vendor-shm"
  line "- modukit-codegen — protoc plugin, Rust/C ABI/C++/Py/JS faces, wire goldens"
  line "- modukit-js — napi bridge over the C ABI"
  line ""
  line "## Contracts"
  doc contracts/modukit_embed.h "embed ABI" "the C embedding surface"
  doc contracts/modukit_plugin.h "plugin ABI" "modukit_plugin_init and HostAbi tables"
  doc contracts/modukit_wire.h "process wire" "TL frame prefix + payload grammar"
  doc contracts/modukit.proto "kernel proto" "proto2 ledger"
  doc contracts/golden/wire-v1 "wire goldens" "regression frames"
} > "$OUT"

echo "docs-gen: wrote $OUT ($(wc -l < "$OUT") lines)"
