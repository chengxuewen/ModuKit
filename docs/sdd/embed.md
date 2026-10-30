# Embed Contract (SDD)

> Test annotation: `// spec: EMBED-01`; gate: `scripts/check-embed-parity.sh`.

## EMBED-01: 24-verb parity

`EMBED_VERBS` is the ordered C embedding surface. Every verb declared in
`contracts/modukit_embed.h` appears exactly once, in the same order, and the Rust
table is the only mirror used by the generated wrappers. A missing or extra verb is
a parity failure; the gate compares the ordered lists, not a sorted set.
