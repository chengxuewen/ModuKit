# Wave-2 debug-reader plan

Status: MINIMAL LANDED on 2026-10-21. The `TestHost` self-check face shipped as one
`modukit-runtime` bin with integration tests. It is not a live-attach debugger and
no standalone `scripts/check.sh` demo leg is added.

## 1. Background

The 2026-10-21 adjudication walked five doors:

1. Remaining-feature adjudication only, no code.
2. Priority criterion: smallest adjudicable door.
3. Selected door: debug reader CLI / tooling floor.
4. Debug boundary: reader-only, no live attach.
5. Asset form: one bin under `modukit-runtime`, not a new `modukit-cli` crate.

Source anchors:

- `docs/stage-4-plan.md`: debug tooling floor is the missing face over the existing
  debug data plane.
- `docs/stage-4-plan.md` I6: reader-only CLI consumes existing artifacts and adds no
  kernel door.
- `docs/stage-5-plan.md`: debug reader remains a wave-2 door, not an open lane.
- `crates/modukit-runtime/Cargo.toml`: existing runtime bins already live in this crate.
- `crates/modukit-core/src/testhost.rs`: `TestHost::new()` / `snapshot()` / `head()` /
  `events_since()` are the existing in-process read path.
- `crates/modukit-core/src/introspect.rs`: `Snapshot`, `HostView`, and `LedgerDump` are
  the read-only objects the CLI should consume without creating a second source.

## 2. Scope

Open a minimal `modukit-debug-reader` face in `crates/modukit-runtime`.

The first version is a self-check reader:

- Boot the existing `TestHost` step-clock kernel.
- Read the existing introspect trio and edge data through ordinary in-process APIs:
  - `Snapshot`
  - `EventStream` / observatory tail
  - `LedgerDump`
  - edge-ledger projection exposed as `edges` on the `Snapshot` / `HostView` cut
- Print human-readable text to stdout.
- Exit `0` when the required PASS line is printed.

This proves the tooling face without adding a new ABI verb, socket, file format, or
runtime attach mechanism.

## 3. Non-scope

Deliberately excluded:

- Live attach to `modukit-host`.
- Unix socket / TCP reader protocol.
- mcap / dump-file reader.
- JSON CLI schema.
- `modukit-cli` crate.
- Plugin loading or production host configuration parsing.
- Changes to `HostAbiV1`, `HostAbiV2`, `HostEmbedV1`, `PluginAbiV1`, or wire version.
- C ABI or C++ emitter work.
- Capability-ledger rows, because no new header verb is introduced.

## 4. Proposed rulings

### P1 - Data source

A. Use `TestHost` and direct kernel service acquisition.  
B. Add a dump-file format now.  
C. Add live attach now.

Recommendation: A. It is the only option that stays inside the adjudicated reader-only
boundary and can be verified without protocol or ABI growth.

### P2 - Product form

A. Add `[[bin]] name = "modukit-debug-reader"` to `modukit-runtime`.  
B. Put the reader under `examples/`.  
C. Create `modukit-cli`.

Recommendation: A. B is not really a CLI; C is premature family expansion.

### P3 - Output grammar

A. Plain text with stable `debug-reader:` prefixes.  
B. Machine-readable JSON.  
C. Colored human dashboard.

Recommendation: A. It keeps the gate cheap and avoids schema commitments.

### P4 - Verification

A. `modukit-runtime` integration test only.  
B. Integration test plus immediate `scripts/check.sh` demo leg.  
C. Manual verification only.

Recommendation: A for the first landing. Add a `check.sh` leg later only if the CLI
output stabilizes enough to become a standing gate.

## 5. Implementation outline

1. Add a new binary target:
   - file: `crates/modukit-runtime/src/debug_reader.rs`
   - target name: `modukit-debug-reader`
   - dependency: existing `modukit-core`

2. Implement a tiny CLI:
   - support `--help`
   - default mode: boot `TestHost::new()`
   - no plugin loading in the first lane

3. Read the existing data plane:
   - `TestHost::snapshot()` returns `HostView`, the D18 object-table cut.
   - `TestHost::head()` returns the current observatory sequence.
   - `TestHost::events_since(0)` returns `Tail` with retained event records.
   - acquire `LedgerDump` through the existing registry service path; if that is not
     reachable from `TestHost`, resolve the smallest read-only accessor before coding.

4. Print at least:

   ```text
   debug-reader: ledger plugins=<count>
   debug-reader: snapshot plugins=<count> services=<count> topics=<count> edges=<count>
   debug-reader: events head=<seq> tail=<count>
   debug-reader: PASS
   ```

   The exact field list can be trimmed, but the PASS line is the stable assertion target.

5. Add an integration test:
   - file: `crates/modukit-runtime/tests/debug_reader.rs`
   - create `crates/modukit-runtime/tests/` if absent; this is the crate's first
     integration test directory
   - launch the binary via `env!("CARGO_BIN_EXE_modukit-debug-reader")`
   - assert stdout contains `debug-reader: PASS`
   - assert no panic and exit status is `0`

6. Optional sabotage hook:
   - environment: `MODUKIT_DEBUG_READER_SABOTAGE=1`
   - behavior: omit the PASS line so the negative path can be observed once
   - keep this out of `scripts/check.sh` until the output grammar is stable

## 6. Done-when

The first lane is done when:

- `modukit-debug-reader --help` prints usage.
- default run prints the PASS line and exits `0`.
- integration test passes under `pixi run cargo test --workspace`.
- sabotage path, if added, is observed once to fail and then removed from the green path.
- no ABI/wire/contract files change.
- no new crate is added.
- `docs/stage-5-plan.md` gets a pointer to this plan.

## 7. Risks

- The `TestHost` reader may look like a demo rather than a real debug tool. The plan
  should call it a self-check face until a dump-file or live-attach consumer appears.
- Output line stability is a future gate dependency; early churn can make later
  `check.sh` integration noisy.
- Exact data source for `LedgerDump` may need a small `TestHost` helper or existing
  registry acquisition path; implementation must reuse current read-only introspect
  services, not invent a second source of truth.

## 8. Review checklist

Before opening implementation, the reviewer should answer:

1. Is the `TestHost` data source honest enough to be called a debug reader?
2. Is the PASS-line grammar the smallest stable assertion target?
3. Should this lane add any `modukit-runtime` helper API, or is the existing
   `TestHost` surface enough?
4. Should the first landing include a sabotage hook even if it is not in `check.sh`?

## 9. Review result

Momus plan-critic verdict: OKAY on 2026-10-21. Mandatory fixes: none. Applied
optional fixes: cite `env!("CARGO_BIN_EXE_modukit-debug-reader")`, note the new
`crates/modukit-runtime/tests/` directory, and name the exact read surfaces:
`TestHost::snapshot() -> HostView`, `TestHost::events_since() -> Tail`, and the `edges`
projection on the object-table cut. The first landing shipped the `TestHost`
self-check face, `--help`, and the sabotage red path. Live attach and dump-file
readers remain separate doors.
