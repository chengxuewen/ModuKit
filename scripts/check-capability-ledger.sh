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
# Section C (Stage-5 N1, D96 ruling 2): one row per HostAbiV2 field -
# the HOST2_ABI_FIELDS mirror list, same existence-only rule.
#
# Extraction is deliberately dumb (documented in the ledger header): a
# Section-A row is a markdown table line starting `| \`verb\` |`. If the
# row grammar changes, change the grep below in the same commit.
set -uo pipefail
SELF=$(realpath "$0") # absolute for the self-test re-exec below
cd "$(dirname "$0")/.."
IN="${MODUKIT_GATE_INPUT_DIR:-}"; [ -n "$IN" ] && IN="${IN%/}/"

# ---- self-test (benchmark plan T2.2) ----
# Copy the ledger and the verb mirror to a scratch dir, delete one
# `| `verb` |` row from the scratch ledger, re-run against the scratch
# copy: the covering-row check MUST go red. Repo files are never touched.
if [ "${1:-}" = "--self-test" ]; then
  s=$(mktemp -d)
  trap 'rm -rf "$s"' EXIT
  mkdir -p "$s/docs" "$s/crates/modukit-core/src"
  cp docs/capability-ledger.md "$s/docs/"
  cp crates/modukit-core/src/embed_abi.rs "$s/crates/modukit-core/src/"
  verb=$(sed -n '/^pub const EMBED_VERBS:/,/^\];/p' "$s/crates/modukit-core/src/embed_abi.rs" \
    | grep -oE '"modukit_[a-z_0-9]+"' | tr -d '"' | head -1)
  grep -v "^| \`$verb\`" "$s/docs/capability-ledger.md" > "$s/ledger.tmp" && mv "$s/ledger.tmp" "$s/docs/capability-ledger.md"
  if MODUKIT_GATE_INPUT_DIR="$s" bash "$SELF" > /dev/null 2>&1; then
    echo "self-test FAIL (check-capability-ledger.sh): planted defect NOT caught"
    exit 1
  fi
  echo "self-test OK (check-capability-ledger.sh): planted defect caught"
  exit 0
fi

LEDGER="${IN}docs/capability-ledger.md"
MIRROR="${IN}crates/modukit-core/src/embed_abi.rs"

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
# Section C (N1): every HOST2_ABI_FIELDS entry needs a ledger row too.
sed -n '/^pub const HOST2_ABI_FIELDS:/,/^\];/p' "$MIRROR" \
  | grep -oE '"[a-z_][a-z0-9_]+"' | tr -d '"' | sort > "$tmp/host2.txt"
if [ ! -s "$tmp/host2.txt" ]; then
  echo "check-capability-ledger: RED (HOST2_ABI_FIELDS extraction empty - anchor drift?)"
  exit 1
fi
while IFS= read -r field; do
  grep -qxF "$field" "$tmp/rows.txt" || { echo "check-capability-ledger: RED (HostAbiV2 row missing: $field)"; missing=1; }
done < "$tmp/host2.txt"

if [ "$missing" -eq 0 ]; then
  echo "check-capability-ledger: ok ($(wc -l < "$tmp/host2.txt") rows cover HOST2_ABI_FIELDS)"
fi
# Section D (T1): every HOST3_ABI_FIELDS entry needs a ledger row too.
sed -n '/^pub const HOST3_ABI_FIELDS:/,/^\];/p' "$MIRROR" \
  | grep -oE '"[a-z_][a-z0-9_]+"' | tr -d '"' | sort > "$tmp/host3.txt"
if [ ! -s "$tmp/host3.txt" ]; then
  echo "check-capability-ledger: RED (HOST3_ABI_FIELDS extraction empty - anchor drift?)"
  exit 1
fi
while IFS= read -r field; do
  grep -qxF "$field" "$tmp/rows.txt" || { echo "check-capability-ledger: RED (HostAbiV3 row missing: $field)"; missing=1; }
done < "$tmp/host3.txt"

if [ "$missing" -eq 0 ]; then
  echo "check-capability-ledger: ok ($(wc -l < "$tmp/host3.txt") rows cover HOST3_ABI_FIELDS)"
fi
exit "$missing"
