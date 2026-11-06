# Embed Contract (SDD)

> Test annotation: `// spec: EMBED-01`; gate: `scripts/check-embed-parity.sh`.

## EMBED-01: 24-verb parity

`EMBED_VERBS` is the ordered C embedding surface. Every verb declared in
`contracts/modukit_embed.h` appears exactly once, in the same order, and the Rust
table is the only mirror used by the generated wrappers. A missing or extra verb is
a parity failure; the gate compares the ordered lists, not a sorted set.

## EMBED-02: The FFI panic wall

Every embed verb body runs inside `codec::guard`. A kernel-side panic is caught
at that boundary and answered with the D51 slot `NEG_KERNEL_BUG` (-10); an
unwind NEVER crosses into C frames or plugin code. The guard is the single
conversion point: a verb that returns normally passes its own code through
unchanged, and a new verb declared without the guard is a contract violation
the parity review must refuse.
