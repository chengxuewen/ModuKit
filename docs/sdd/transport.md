# Transport Contract (SDD)

> Test annotation: `// spec: TRANSPORT-01`; gate: `scripts/spec-trace.sh`.

## TRANSPORT-01: fdpass handoff

The SCM_RIGHTS handoff sends exactly one memfd blob over a socketpair; the receiver
gets the sender's open file description, not a copy of the bytes. An oversized
ancillary block is refused as a typed `CTRUNC` error; a dead lane or receive timeout
surfaces the typed backend error. The same handoff must work across a real process
boundary, not only in-process.
