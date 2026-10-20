#!/usr/bin/env bash
# leg: capability ledger coverage (Stage-4 M1 lane #7, plan §4 "capability
# ledger gate" adoption note, ruling C++ lineage).
#
# One row = one header verb: every verb in EMBED_VERBS
# (crates/modukit-core/src/embed_abi.rs - the mirror pinned to
# contracts/modukit_embed.h by scripts/check-embed-parity.sh) must appear
# as a row in docs/capability-ledger.md. RED iff a header verb lacks a
# row. Unshipped languages ride the `planned` column, never a red
# condition - this leg checks ROW EXISTENCE only, never cell values.
#
# Extraction is deliberately dumb (documented in the ledger header): a
# Section-A row is a markdown table line starting `| \`verb\` |`. If the
# row grammar changes, change the grep below in the same commit.
set -uo pipefail
cd "$(dirname "$0")/.."

LEDGER=docs/capability-ledger.md
MIRROR=crates/modukit-core/src/embed_abi.rs

for f in "$LEDGER" "$MIRROR"; do
  if [ ! -f "$f" ]; then
    echo "check-capability-ledger: RED (missing $f)"
    exit 1
  fi
done

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Mirror side: the EMBED_VERBS entries (same scoped extraction as the
# parity gate's mirror_fields helper - block-bounded, so other quoted
# modukit_* strings in the file never leak in).
sed -n '/^pub const EMBED_VERBS:/,/^\];/p' "$MIRROR" \
  | grep -oE '"modukit_[a-z_0-9]+"' | tr -d '"' | sort > "$tmp/verbs.txt"

if [ ! -s "$tmp/verbs.txt" ]; then
  echo "check-capability-ledger: RED (EMBED_VERBS extraction empty - anchor drift?)"
  exit 1
fi

# Ledger side: the backticked row key of every table row, once per line.
grep -oE '^\| `[^`]+`' "$LEDGER" | sed 's/^| `//; s/`$//' | sort -u > "$tmp/rows.txt"

missing=0
while IFS= read -r verb; do
  grep -qxF "$verb" "$tmp/rows.txt" || { echo "check-capability-ledger: RED (verb row missing: $verb)"; missing=1; }
done < "$tmp/verbs.txt"

if [ "$missing" -eq 0 ]; then
  echo "check-capability-ledger: ok ($(wc -l < "$tmp/verbs.txt") rows cover EMBED_VERBS; $(grep -cE '^\| `' "$LEDGER") ledger rows total)"
fi
exit "$missing"
