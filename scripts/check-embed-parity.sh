#!/usr/bin/env bash
# leg: contract parity for the C contract originals (Stage-4 M1 lane #2,
# ruling C++-3; extended by gap ruling A 2026-10-20 for the plugin side).
#
# (a) Embed face: contracts/modukit_embed.h IS the HostEmbedV1 contract
#     original; pins the Rust-side mirror (embed_abi.rs EMBED_VERBS) verb-
#     for-verb against it. SORTED diff - the embed vtable order is not
#     itself a contract.
# (b) Plugin face: contracts/modukit_plugin.h mirrors the frozen #[repr(C)]
#     tables abi::PluginAbiV1 / abi::HostAbiV1 (crates/modukit-core/src/
#     abi.rs - read-only layout truth, D87 STABLE). Here ORDER IS THE
#     CONTRACT, so the field-name lists are diffed UNSORTED, three-way:
#     abi.rs <-> header <-> embed_abi.rs consts (PLUGIN_ABI_FIELDS /
#     HOST_ABI_FIELDS).
#     Stage-5 N1 (D96 ruling 2) adds the v2 trio: abi::HostAbiV2 <->
#     modukit_host_abi_v2 <-> HOST2_ABI_FIELDS, same ordered diff.
#     Mapping discipline: Rust field `x` <-> C member `x` - identical
#     snake_case names, no affix; one field per line in both sources.
#     Split of duties: the consts' Rust unit test asserts the lists are
#     non-empty + unique; the ordering truth lives in this script.
#     The version/entry pins are checked too: ABI_VERSION <->
#     MODUKIT_ABI_VERSION, ENTRY_SYMBOL <-> MODUKIT_ENTRY_SYMBOL.
# Amendment policy is D83 tail-add - a verb/field moved or added on one
# side without the other fails here before anything downstream can drift.
set -uo pipefail
SELF=$(realpath "$0") # absolute for the self-test re-exec below
cd "$(dirname "$0")/.."
IN="${MODUKIT_GATE_INPUT_DIR:-}"; [ -n "$IN" ] && IN="${IN%/}/"

# ---- self-test (benchmark plan T2.2) ----
# Copy the gate's four inputs to a scratch dir, drop one verb declaration
# from the scratch embed header, re-run against the scratch copy: the
# header <-> mirror diff MUST go red. Repo files are never touched.
if [ "${1:-}" = "--self-test" ]; then
  s=$(mktemp -d)
  trap 'rm -rf "$s"' EXIT
  mkdir -p "$s/contracts" "$s/crates/modukit-core/src"
  cp contracts/modukit_embed.h contracts/modukit_plugin.h "$s/contracts/"
  cp crates/modukit-core/src/embed_abi.rs crates/modukit-core/src/abi.rs "$s/crates/modukit-core/src/"
  sed -i '0,/^int32_t modukit_/{/^int32_t modukit_/d}' "$s/contracts/modukit_embed.h"
  if MODUKIT_GATE_INPUT_DIR="$s" bash "$SELF" > /dev/null 2>&1; then
    echo "self-test FAIL (check-embed-parity.sh): planted defect NOT caught"
    exit 1
  fi
  echo "self-test OK (check-embed-parity.sh): planted defect caught"
  exit 0
fi
EMBED_HEADER="${IN}contracts/modukit_embed.h"
PLUGIN_HEADER="${IN}contracts/modukit_plugin.h"
MIRROR="${IN}crates/modukit-core/src/embed_abi.rs"
ABI="${IN}crates/modukit-core/src/abi.rs"

for f in "$EMBED_HEADER" "$PLUGIN_HEADER" "$MIRROR" "$ABI"; do
  if [ ! -f "$f" ]; then
    echo "check-embed-parity: RED (missing $f)"
    exit 1
  fi
done

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
fail=0

# ---- (a) embed verbs: header declarations <-> EMBED_VERBS (sorted) ----
# Header side: every verb function declaration (int32_t modukit_*() in the
# extern "C" block).
grep -oE '^int32_t modukit_[a-z_0-9]+\(' "$EMBED_HEADER" \
  | sed 's/^int32_t //; s/($//' | sort > "$tmp/header.txt"

# Mirror side: the EMBED_VERBS entries (the parity gate's second source).
grep -oE '"modukit_[a-z_0-9]+"' "$MIRROR" | tr -d '"' | sort > "$tmp/mirror.txt"

if diff -u "$tmp/header.txt" "$tmp/mirror.txt" > "$tmp/verbs.diff"; then
  echo "check-embed-parity: ok ($(wc -l < "$tmp/header.txt") verbs match header <-> mirror)"
else
  echo "check-embed-parity: RED (header <-> EMBED_VERBS mismatch)"
  cat "$tmp/verbs.diff"
  fail=1
fi

# ---- (b) plugin tables: ordered field diffs ----
# Rust side: the `pub <name>: <type>,` field lines inside a struct block.
rust_fields() { sed -n "/^pub struct $1 {/,/^}/p" "$ABI" \
  | grep -oE 'pub [a-z_][a-z0-9_]*:' | sed 's/^pub //; s/:$//'; }

# C side: the identifier before each line-ending `;` inside the struct
# block; comment and closing-`}` lines are excluded, and the header keeps
# one member per line (documented convention).
c_fields() { sed -n "/^typedef struct $1 {/,/^} $1;/p" "$PLUGIN_HEADER" \
  | sed 's|/\*.*\*/||g' \
  | grep -vE '^[[:space:]]*(/\*|\*|})' \
  | grep -oE '[a-z_][a-z0-9_]*[[:space:]]*;[[:space:]]*$' \
  | sed 's/[[:space:]]*;[[:space:]]*$//'; }

# Mirror side: the quoted entries of a *_ABI_FIELDS const block. awk, not
# a sed range: a single-line const closes on its START line and sed only
# checks the range end from the FOLLOWING line - with a second
# *_ABI_FIELDS const after the first, a sed range would swallow the sibling.
mirror_fields() { awk -v c="pub const $1" '
  index($0, c ":") == 1 { inside = 1 }
  inside { print; if (index($0, "];") > 0) inside = 0 }
' "$MIRROR" | grep -oE '"[a-z_][a-z0-9_]*"' | tr -d '"'; }

check_table() { # $1 rust struct, $2 C tag, $3 mirror const
  rust_fields "$1" > "$tmp/$1.rust"
  c_fields "$2" > "$tmp/$1.c"
  mirror_fields "$3" > "$tmp/$1.mirror"
  if [ ! -s "$tmp/$1.rust" ] || [ ! -s "$tmp/$1.c" ] || [ ! -s "$tmp/$1.mirror" ]; then
    echo "check-embed-parity: RED ($1: empty extraction - anchor drift?)"
    fail=1
    return
  fi
  if diff -u "$tmp/$1.rust" "$tmp/$1.c" > "$tmp/$1.diff" \
     && diff -u "$tmp/$1.rust" "$tmp/$1.mirror" >> "$tmp/$1.diff"; then
    echo "check-embed-parity: ok ($1: $(wc -l < "$tmp/$1.rust") fields match abi.rs <-> header <-> mirror, order preserved)"
  else
    echo "check-embed-parity: RED ($1: abi.rs <-> header/mirror field order or names differ)"
    cat "$tmp/$1.diff"
    fail=1
  fi
}
check_table PluginAbiV1 modukit_plugin_abi_v1 PLUGIN_ABI_FIELDS
check_table HostAbiV1 modukit_host_abi_v1 HOST_ABI_FIELDS
# Stage-5 N1 v2 door (D96 ruling 2): the second host table joins the trio.
check_table HostAbiV2 modukit_host_abi_v2 HOST2_ABI_FIELDS
# Wave-2 T1 v3 clock door (D99 R2): the third host table joins the trio.
check_table HostAbiV3 modukit_host_abi_v3 HOST3_ABI_FIELDS

# Version pin: both sides must read the same major integer.
rust_ver=$(sed -n 's/^pub const ABI_VERSION: u32 = \([0-9]\{1,\}\);$/\1/p' "$ABI")
c_ver=$(sed -n 's/^#define MODUKIT_ABI_VERSION \([0-9]\{1,\}\)[a-z]*$/\1/p' "$PLUGIN_HEADER")
if [ -z "$rust_ver" ] || [ "$rust_ver" != "$c_ver" ]; then
  echo "check-embed-parity: RED (ABI_VERSION=$rust_ver vs MODUKIT_ABI_VERSION=$c_ver)"
  fail=1
fi

# v2 version pin (N1): the declared-major-2 constant mirrors the same way.
rust_ver2=$(sed -n 's/^pub const ABI_VERSION_V2: u32 = \([0-9]\{1,\}\);$/\1/p' "$ABI")
c_ver2=$(sed -n 's/^#define MODUKIT_ABI_VERSION_V2 \([0-9]\{1,\}\)[a-z]*$/\1/p' "$PLUGIN_HEADER")
if [ -z "$rust_ver2" ] || [ "$rust_ver2" != "$c_ver2" ]; then
  echo "check-embed-parity: RED (ABI_VERSION_V2=$rust_ver2 vs MODUKIT_ABI_VERSION_V2=$c_ver2)"
  fail=1
fi

# v3 version pin (T1): the declared-major-3 constant mirrors the same way.
rust_ver3=$(sed -n 's/^pub const ABI_VERSION_V3: u32 = \([0-9]\{1,\}\);$/\1/p' "$ABI")
c_ver3=$(sed -n 's/^#define MODUKIT_ABI_VERSION_V3 \([0-9]\{1,\}\)[a-z]*$/\1/p' "$PLUGIN_HEADER")
if [ -z "$rust_ver3" ] || [ "$rust_ver3" != "$c_ver3" ]; then
  echo "check-embed-parity: RED (ABI_VERSION_V3=$rust_ver3 vs MODUKIT_ABI_VERSION_V3=$c_ver3)"
  fail=1
fi

# Entry-symbol pin: same exported name on both sides (NUL stripped).
rust_sym=$(sed -n 's/^pub const ENTRY_SYMBOL: .*b"\(.*\)".*$/\1/p' "$ABI" | sed 's/\\0$//')
c_sym=$(sed -n 's/^#define MODUKIT_ENTRY_SYMBOL "\(.*\)".*$/\1/p' "$PLUGIN_HEADER")
if [ -z "$rust_sym" ] || [ "$rust_sym" != "$c_sym" ]; then
  echo "check-embed-parity: RED (ENTRY_SYMBOL=$rust_sym vs MODUKIT_ENTRY_SYMBOL=$c_sym)"
  fail=1
fi

exit "$fail"
