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

# ---- self-test (benchmark plan T2.2; E-row plant D106 batch 1) ----
# Copy the ledger, the verb mirror and the wire.h ledger to a scratch
# dir, delete one `| `verb` |` row from the scratch ledger, re-run
# against the scratch copy: the covering-row check MUST go red. Restore
# the ledger, drop one section-E row, re-run: the E gap MUST go red too.
# Repo files are never touched.
if [ "${1:-}" = "--self-test" ]; then
  s=$(mktemp -d)
  trap 'rm -rf "$s"' EXIT
  mkdir -p "$s/docs" "$s/crates/modukit-core/src" "$s/contracts"
  cp docs/capability-ledger.md "$s/docs/"
  cp crates/modukit-core/src/embed_abi.rs "$s/crates/modukit-core/src/"
  cp contracts/modukit_wire.h "$s/contracts/"
  verb=$(sed -n '/^pub const EMBED_VERBS:/,/^\];/p' "$s/crates/modukit-core/src/embed_abi.rs" \
    | grep -oE '"modukit_[a-z_0-9]+"' | tr -d '"' | head -1)
  grep -v "^| \`$verb\`" "$s/docs/capability-ledger.md" > "$s/ledger.tmp" && mv "$s/ledger.tmp" "$s/docs/capability-ledger.md"
  if MODUKIT_GATE_INPUT_DIR="$s" bash "$SELF" > /dev/null 2>&1; then
    echo "self-test FAIL (check-capability-ledger.sh): planted defect NOT caught"
    exit 1
  fi
  # E-row plant: restore the ledger, drop one section-E row, re-run.
  cp docs/capability-ledger.md "$s/docs/"
  grep -v '^| `MSG_DUE`' "$s/docs/capability-ledger.md" > "$s/ledger.tmp" && mv "$s/ledger.tmp" "$s/docs/capability-ledger.md"
  if MODUKIT_GATE_INPUT_DIR="$s" bash "$SELF" > /dev/null 2>&1; then
    echo "self-test FAIL (check-capability-ledger.sh): E-row gap NOT caught"
    exit 1
  fi
  echo "self-test OK (check-capability-ledger.sh): both planted defects caught"
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
# Section E (wave-2 plugin door batch 1, D106 plan T1.4): every E-table row
# mirrors a "NAME = value" constant line in contracts/modukit_wire.h - row
# name AND decimal value pinned, checked BOTH directions (a row without its
# ledger constant and a ledger constant without its row are both red).
# E-scope = the MSG_*/FWD_*/HELLO_CAP_* wire.h constants; WIRE_VERSION,
# FRAME_MAX and DIAL_STATUS_REFUSED_REAL are envelope grammar, not rows -
# the pin of EVERY wire.h constant against the proc_wire.rs pub consts
# lives in scripts/check-wire-parity.sh. The `armed` token is this
# section's only addition to the cell vocabulary (this volume's batches
# 2/3 own the skins; cell values are never a red condition - the
# existence rule is unchanged).
WIRE="${IN}contracts/modukit_wire.h"
if grep -q '^## E\.' "$LEDGER" || [ -f "$WIRE" ]; then
  if [ ! -f "$WIRE" ]; then
    echo "check-capability-ledger: RED (section E rows but no $WIRE)"; missing=1
  elif ! grep -q '^## E\.' "$LEDGER"; then
    echo "check-capability-ledger: RED (wire ledger present but no section E rows)"; missing=1
  else
    sed -n '/^## E\./,$p' "$LEDGER" \
      | grep -oE '^\| `[A-Z0-9_]+` \| [0-9]+ ' | sed 's/^| `//; s/` | / /; s/ $//' | sort > "$tmp/e_rows.txt"
    grep -oE '^ \* (MSG_|FWD_|HELLO_CAP_)[A-Z0-9_]+ = [0-9]+' "$WIRE" \
      | sed 's/^ \* //; s/ = / /' | sort > "$tmp/e_ledger.txt"
    if [ ! -s "$tmp/e_rows.txt" ] || [ ! -s "$tmp/e_ledger.txt" ]; then
      echo "check-capability-ledger: RED (section E extraction empty - grammar drift?)"
      missing=1
    fi
    while IFS=' ' read -r name value; do
      lval=$(awk -v n="$name" '$1 == n { print $2 }' "$tmp/e_ledger.txt")
      if [ -z "$lval" ]; then
        echo "check-capability-ledger: RED (E row has no wire.h constant: $name)"; missing=1
      elif [ "$lval" != "$value" ]; then
        echo "check-capability-ledger: RED (E row value mismatch: $name row=$value wire.h=$lval)"; missing=1
      fi
    done < "$tmp/e_rows.txt"
    while IFS=' ' read -r name value; do
      grep -q "^$name " "$tmp/e_rows.txt" || {
        echo "check-capability-ledger: RED (wire.h constant has no E row: $name)"; missing=1; }
    done < "$tmp/e_ledger.txt"
    if [ "$missing" -eq 0 ]; then
      echo "check-capability-ledger: ok ($(wc -l < "$tmp/e_rows.txt") rows cover the wire E-scope)"
    fi
  fi
fi
exit "$missing"
