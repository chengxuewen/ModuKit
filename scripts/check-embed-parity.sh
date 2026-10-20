#!/usr/bin/env bash
# leg: embed-contract parity (Stage-4 M1 lane #2, ruling C++-3). The C header
# contracts/modukit_embed.h IS the HostEmbedV1 contract original; this gate
# pins the Rust-side mirror (crates/modukit-core/src/embed_abi.rs EMBED_VERBS)
# to it verb-for-verb: the sorted lists must be identical, empty diff or red.
# Amendment policy is D83 tail-add - a verb added to one side without the
# other fails here before anything downstream can drift.
set -uo pipefail
cd "$(dirname "$0")/.."

HEADER=contracts/modukit_embed.h
MIRROR=crates/modukit-core/src/embed_abi.rs

if [ ! -f "$HEADER" ] || [ ! -f "$MIRROR" ]; then
  echo "check-embed-parity: RED (missing $HEADER or $MIRROR)"
  exit 1
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Header side: every verb function declaration (int32_t modukit_*() in the
# extern "C" block).
grep -oE '^int32_t modukit_[a-z_0-9]+\(' "$HEADER" \
  | sed 's/^int32_t //; s/($//' | sort > "$tmp/header.txt"

# Mirror side: the EMBED_VERBS entries (the parity gate's second source).
grep -oE '"modukit_[a-z_0-9]+"' "$MIRROR" | tr -d '"' | sort > "$tmp/mirror.txt"

if diff -u "$tmp/header.txt" "$tmp/mirror.txt" > "$tmp/diff.txt"; then
  echo "check-embed-parity: ok ($(wc -l < "$tmp/header.txt") verbs match header <-> mirror)"
  exit 0
fi
echo "check-embed-parity: RED (header <-> EMBED_VERBS mismatch)"
cat "$tmp/diff.txt"
exit 1
