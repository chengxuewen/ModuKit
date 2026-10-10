# 03 — Lifecycle State Machine

D3. Six OSGi-style states plus a `Failed` edge; availability grade in-vehicle-B: tiered
parallel boot, supervised restart with backoff, consumer policies (02 §2) — and explicitly no
hard real-time and no functional-safety surface.

## 1. States

```
UNINSTALLED -> INSTALLED -> RESOLVING -> STARTING -> ACTIVE -> STOPPING -> RESOLVED/UNINSTALLED
                                  |           |          ^
                                  v           v          | restart (policy, backoff)
                               FAILED  -------------------+
```

- `INSTALLED`: artifact known, identity + manifest parsed, not yet validated.
- `RESOLVING`: contract/link/placement prerequisites checked; required dependencies validated
  (fail here = deterministic boot failure, before any plugin code executes).
- `STARTING` / `ACTIVE` / `STOPPING`: `start()` / running / cooperative `stop()`.
- `FAILED`: reached from `RESOLVING`/`STARTING` (setup error) or — process placement only —
  from `ACTIVE` on supervised death or heartbeat loss.
- `RESOLVED`: stopped-but-resolved, cheap restart origin.

`dl` placement has **no** transition out of host corruption (01 §4): if a native plugin
segfaults the host, no state machine survives to record it. `exe` placement is where
`Failed` + restart actually protects the system.

## 2. Transition rules

| From | To | Guard |
|---|---|---|
| `UNINSTALLED` | `INSTALLED` | manifest valid, artifact present |
| `INSTALLED` | `RESOLVING` | install policy / explicit |
| `RESOLVING` | `STARTING` | required deps present; ABI handshake passes (04 §3) |
| `RESOLVING` | `FAILED` | any validation error |
| `STARTING` | `ACTIVE` | `start()` returns ok within `start_timeout` |
| `STARTING` | `FAILED` | `start()` error or timeout |
| `ACTIVE` | `STOPPING` | stop / restart / placement switch |
| `STOPPING` | `RESOLVED` | clean stop |
| `STOPPING` | `FAILED` | stop timeout (process placement: kill path) |
| `FAILED` | `RESOLVING` | restart policy allows (`on-failure`, backoff, cooldown, `max_retries`) |
| any | `UNINSTALLED` | uninstall (from `RESOLVED`/`FAILED`/`INSTALLED` direct) |

Restart budget exhaustion parks in `FAILED` and emits an event — an operator-visible
"stopped trying", not a silent loop.

## 3. Boot orchestration

- **start levels**: integer in manifest; boot proceeds level by level, parallel within a level.
- **dependency edges** (02 §2) add ordering inside/between levels; the host computes a tiered
  DAG and starts it. A cycle at resolve = install-time error.
- **consumer policy at boot**: `wait(timeout)` parks the *dependent* in `RESOLVING`-like
  pending state — the boot graph reports who waits on whom; `failfast` fails the dependent.
- Boot is not a latency contract: high-rate hardware data planes bypass the bus entirely
  (architecture.md §6); no microsecond promises here.

## 4. Watchdog and safety neighbor (out of scope, boundary stated)

External safety monitors that want "HMI alive" heartbeats read the same event stream
(01 supervisor, 02 registry events); ModuKit does not own watchdog feeding, ASIL claims, or
boot-time deadlines — those belong to system layers (D3 rationale; if a hard boot-deadline
requirement ever lands on ModuKit, that is a D3 re-review trigger, not a kernel feature).

## 5. Blocking and timeouts (the honest asymmetry, D6)

Contract calls are synchronous at the boundary (04 §2), so a plugin may block inside a call:

| Placement | Caller timeout | Host defense |
|---|---|---|
| `dl` | none — native code cannot be preempted | review + contract rules + external watchdog; a blocked plugin blocks the host |
| `exe` | caller abandons the correlation id; supervisor kills/restarts per §2 policy | full: timeout -> `FAILED` -> restart path |

The job pattern (02 §4) removes the long-tail motivation to block: short sync `submit`,
completion by event.

## 6. Clock & timers (D20)

One water source for time: the built-in `org.modukit.clock` service —

```
trait Clock { fn now(&self) -> Timestamp; }      // domains: real | sim | step
fn timer(clock, deadline) -> JobToken            // fires as a `done` JobEvent (02 §4)
```

- Kernel-side timers — `start_timeout`, restart backoff (§2), future QoS deadlines — ride
  this same source; the replay virtual publisher (09 §3) gains the right to turn the dial
  under `step|burst`, so frames, logs (D19) and time stay one axis.
- `sim` domain accepts an external time base (test rigs/HIL; wiring field deferred to the
  tooling round).
- Convention: plugin time comes from the service; **high-rate loops use the message's own
  `capture_ts`** (authoritative at capture, zero lookups on hot paths); raw wall-clock use
  is reserved for host-owned binaries (CLI/daemons) that sit outside the bus.
- Rejected alternatives (D20): LD_PRELOAD-style interception (co-poisons the host under
  `dl`, unportable to win/mac, fights debuggers); test-only mock clocks (replay would keep
  lying — two clocks, two truths).

## 6. Update path (D8)

The whitepaper lifecycle includes update (§4.1, `whitepaper.md:111`); year-1 semantics are
**stop-based with a declared parallel escape hatch**:

| Placement | Mechanism | States traversed |
|---|---|---|
| `dl` | new artifact staged (pending marker at `INSTALLED`), activated on next host boot | none for the running instance — the host restart cycle does the swap (01 §4: dlclose is not a reload strategy) |
| `exe` | supervisor: `ACTIVE -> STOPPING -> (swap artifact) -> STARTING`, restart policy applies | existing §2 edges, no new states |

- Consumers observe exactly one `Unregistered -> Registered` event pair; surviving the gap is
  governed by the already-adjudicated `on_down` / `policy` fields (02 §2). No silent behavior
  change: a consumer that chose `wait(timeout)` waits; one that chose `fallback` serves stale.
- **`parallel_ok` escape hatch (declared, implementation Stage-2)**: an author may declare a
  provided service stateless/idempotent; only such services, only in `exe` placement, may run
  two versions concurrently — new registers (version property carries `major`), consumers
  re-acquire by version affinity, drained old instance unregisters. Year 1: the declaration
  field exists, is cross-checked (05 §6), and is inert — no multi-instance routing ships.
- OTA distribution (fetch/verify/rollback of bundles) remains outside the kernel
  (architecture.md §10 out-of-scope list).
