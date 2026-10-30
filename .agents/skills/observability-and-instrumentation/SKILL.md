---
name: observability-and-instrumentation
type: tooling
notes: "kernel event ring, logger, debug-reader"
description: "Observability and instrumentation for ModuKit: the kernel Observatory ring, structured logging, counters/events, the debug-reader, and symptom-based alerting. Use when adding instrumentation, debugging runtime behavior, or extending a diagnostic gate."
---

# Observability and Instrumentation

## Overview

ModuKit's observability floor is in the kernel, not in a web page:

- `crates/modukit-core/src/observability.rs` owns the bounded event ring and the
  counter/event records.
- `crates/modukit-core/src/logger.rs` is the structured logger that feeds the ring.
- `crates/modukit-runtime` ships `modukit-debug-reader`, a read-only consumer of
  snapshot / events / ledger data.
- `scripts/demo-wave2-reader.sh` is the executable proof that the reader sees kernel
  state after the host has run.

The single source for gate commands is `pixi.toml` and `scripts/check.sh`; do not
invent a parallel telemetry gate.

## When to Use

- Adding a counter, event, or log field at the kernel/service/bus boundary.
- Debugging "the plugin ran but I cannot see it" reports.
- Extending the debug-reader or a reader-style gate.
- Designing an alert around an observable symptom.

## Process (steps + checkpoints + exit)

1. **Name the symptom** the operator will see; do not start from "add a metric".
2. **Choose the seam**: kernel state transition, registry call, bus publish/deliver,
   or process-wire frame. Instrument once at the owning seam.
3. **Keep the ring bounded**: every event has a fixed-size record; no unbounded Vec.
4. **Expose it read-only**: snapshot, `events_since`, or ledger dump; never a writer.
5. **Add the proof**: extend `scripts/demo-wave2-reader.sh` or a focused unit test;
   assert a count or a field, not "it logged".
6. **Run the gate**: `pixi run ci` (the reader demo is a leg); for wire/ABI changes
   also run `pixi run gate-abi`.
7. **Exit**: the symptom is reproducible, the event/counter is visible, and the
   gate output is quoted as evidence.

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "I'll add a println" | stdout is not the observability surface; use the logger/ring. |
| "the counter can grow" | The ring is bounded by design; an unbounded path is a bug. |
| "the reader test is slow" | It is the proof the reader works; skipping it hides stale state. |
| "logs are enough" | A log line without a stable field/count cannot support an alert. |

## Red Flags

- A new event that bypasses the ring or writes to stdout.
- A debug-reader path that mutates state.
- A metric with no owner seam and no gate assertion.

## Verification (evidence)

- `pixi run ci` green, with the reader leg visible in the output.
- For a new event: a unit test or demo assertion that observes the exact count/field.

## References (read on demand)

- `crates/modukit-core/src/observability.rs` — ring, counters, snapshot/events.
- `crates/modukit-core/src/logger.rs` — structured logger.
- `crates/modukit-runtime/src/debug_reader.rs` — read-only reader.
- `scripts/demo-wave2-reader.sh` — executable reader proof.
- `docs/modules/09-observability-debug.md` — the design note.
