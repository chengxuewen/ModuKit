# Contracts (SDD) — ownership

The clause files are index entries, not a second test suite. The named gate or test
remains the authority; `scripts/spec-trace.sh` only proves that every clause has a
matching `spec:` tag and that no orphan tag points at a missing clause.

| Namespace | Clause | Owner | Proof |
|---|---|---|---|
| KERNEL | `KERNEL-01` | `modukit-core` (lifecycle) | `crates/modukit-core/tests/process_executor/lifecycle.rs` |
| EMBED | `EMBED-01` | `modukit-core` (embed ABI) | `crates/modukit-core/src/embed_abi.rs` + `scripts/check-embed-parity.sh` |
| WIRE | `WIRE-01` | `modukit-core` (proc_wire) | `crates/modukit-core/tests/process_executor/handshake.rs` + `scripts/check-wire-parity.sh` |
| BUS | `BUS-01` | `modukit-core` (bus_wire) | `crates/modukit-core/tests/abi_v2_cross.rs` |
| TRANSPORT | `TRANSPORT-01` | `modukit-transport` | `crates/modukit-transport/tests/fdpass.rs` |

Tag convention: `// spec: PREFIX-NN` in Rust/C/C++/JS, `# spec: PREFIX-NN` in
Python/shell. Scan roots: `crates/*/src`, `crates/*/tests`, `test-plugins/`,
`examples/`. A clause whose proof is a demo script is named here and stays out of
the tag scan for round 1.

## Out of round 1 (2026-10-30 doc-audit)

These surfaces have owners and existing proofs but no clause in round 1. They are
recorded here so the SDD index is not mistaken for full contract coverage; round 2
will add the clause plus its `spec:` tag only when the proof is stable.

| Proposed clause | Surface | Owner | Existing proof |
|---|---|---|---|
| `BUS-02` | HostAbiV2/V3 door negotiation, v3 timer slots | `modukit-core` (bus_wire) | `crates/modukit-core/tests/abi_v2_cross.rs`, `scripts/demo-stage5-n2.sh` |
| `WIRE-02` | `modukit_wire.h` constants <-> `proc_wire.rs` both ways | `modukit-core` (proc_wire) | `scripts/check-wire-parity.sh` (14 goldens, 19 constants) |
| `EMBED-02` | generated Python/JS skin fidelity to the C contract | `modukit-codegen` | `scripts/check-embed-parity.sh`, `scripts/demo-wave2-py.sh` / `-js.sh` |
