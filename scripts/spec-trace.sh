#!/usr/bin/env bash
# SDD clause numbers <-> test `spec:` tags, bidirectional coverage gate.
# Clause lines are `## PREFIX-NN: ...`; tags are `// spec:` (Rust/C/C++/JS) or
# `# spec:` (Python/shell). A prefix outside the whitelist is the third check:
# extending a namespace without extending this table must not produce a fake green.
set -uo pipefail
cd "$(dirname "${BASH_SOURCE[0]:-$0}")/.."

PREFIX_RE='KERNEL|EMBED|WIRE|BUS|TRANSPORT'
SCAN_ROOTS=(crates test-plugins examples)

sdds=$(grep -ohE "^## ($PREFIX_RE)-[0-9]{2}\b" docs/sdd/*.md 2>/dev/null | sed 's/^## //' | sort -u)
tags=$(grep -rhoE "(//|#) spec: ($PREFIX_RE)-[0-9]{2}\b" "${SCAN_ROOTS[@]}" 2>/dev/null \
    | sed -E 's|.*spec: ||' | sort -u)

ghost=$( {
    grep -ohE '^## [A-Z]+-[0-9]{2}\b' docs/sdd/*.md 2>/dev/null | sed 's/^## //;s/-[0-9][0-9]//'
    grep -rhoE '(//|#) spec: [A-Z]+-[0-9]{2}' "${SCAN_ROOTS[@]}" 2>/dev/null | sed -E 's|.*spec: ||;s/-[0-9][0-9]//'
} | sort -u | grep -vxE "$PREFIX_RE" || true )
if [ -n "$ghost" ]; then
    echo "ERROR: prefix outside the whitelist (extend PREFIX_RE or remove the ghost): $(echo $ghost)" >&2
    exit 1
fi

if [ -z "$sdds" ] && [ -z "$tags" ]; then
    echo "spec-trace OK: 0 clauses (empty set is legal during skeleton phase)"
    exit 0
fi
if [ "$sdds" != "$tags" ]; then
    echo "ERROR: SDD/spec trace mismatch (left = clause without test, right = test without clause):" >&2
    diff <(printf '%s\n' "$sdds") <(printf '%s\n' "$tags") >&2 || true
    exit 1
fi
echo "spec-trace OK: $(printf '%s\n' "$sdds" | grep -c .) pairs aligned"
