# Wire Contract (SDD)

> Test annotation: `// spec: WIRE-01`; gate: `scripts/check-wire-parity.sh`.

## WIRE-01: TL frame prefix

Every process-wire frame starts with `[u32 LE payload-length][u8 WIRE_VERSION=1]
[u8 kind][payload...]`. The length covers the payload only; the 6-byte header is not
counted. A payload above `FRAME_MAX` is refused before allocation. A foreign version
is read and surfaced to the caller, never silently accepted or reinterpreted.
