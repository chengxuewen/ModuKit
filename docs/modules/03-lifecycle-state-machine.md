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
  STOPPING drain semantics (D40, the industry-converged three-step): new calls are
  REFUSED (handle-table marked), in-flight calls complete naturally (bounded by the
  plugin's `stop_timeout`, D34 expiry rules on overrun) - refuse new, finish old, with
  an alarm.
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

**Failover pairs (D47, named - mechanism at its bell)**: hot standby is FREE today -
two providers of the same service with different ranks (D4's reserved ranking was
designed for exactly this; consumers select by rank, it is not a permission). Cold
standby (start the backup on primary Failed) is named and trigger-gated on the first
real degraded-topology requirement; when it rings, it assembles ONLY from stock parts:
D3 Failed event -> 03 §3.1's sibling gate (start approval) -> lazy activation in its
purpose-specific form (D33's deferred general knob), with every switch landing on the
ring (D35/D36 audit discipline). A general condition-startup DSL is rejected (D38: no
logic in config, twice-ruled) - nobody may hand-roll a launch rules engine in a plugin.

## 3.1 Start-approval hook (named, trigger-gated - D40)

A runtime policy checkpoint at RESOLVING ("this id may not start until audited") is named
but not built: the gate exists (resolution order), the policy author is undefined -
trigger-gated on the first governance-policy requirement. Fits the D35 gate/audit split
(decisions land on the ring).

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

## 7. The host's own lifecycle (D34)

The host is a long-lived process; its own lifecycle is three statements, all mirrors of
existing plugin-side rulings:

1. **Cold start**: bring up the observation ring first (D19 - the host's own logs ride it),
   read deploy-TOML (D29), then load bundles along the boot DAG (03 §3).
2. **Graceful shutdown**: on SIGTERM/SIGINT, stop bundles in REVERSE start_level order;
   each plugin gets its `stop_timeout` (05 §2, D34) with the per-placement expiry outcome;
   the host itself has a total shutdown budget - exceeded, it exits with a code that says
   "unclean".
3. **Self-healing belongs to the OS**: host crash/keepalive is the OS service manager's job
   (systemd `Restart=`, vehicle power-mode supervisor - general knowledge). The host
   cooperates with two courtesies: an exit-code vocabulary (clean / config error / fatal,
   for Restart policies to key on) and best-effort crash testament (last N ring events
   persisted for post-mortem; implementation trigger-gated on the first field crash
   analysis need). External watchdogs stay out of scope (03 §4 boundary unchanged).

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

## 8. The control panel (D44)

Manual lifecycle operations, a ledger-native service: `start / stop / restart / reap`.
Every button is a hand-cranked twin of an already-adjudicated automatic mechanism - no
new machinery:

| Op | Automatic twin |
|---|---|
| start / stop / restart | D3 restart family |
| reap | D34 placement-specific expiry (SIGKILL / epoch-cancel / mark-and-move) |
| pause-publish | trigger-gated (D21 storm observation gives it reason; first incident builds it) |

Gates: every ctl call passes identity (D37 bridge/local declarations) + capability word
`org.modukit.control` (D26 vocabulary); accepted AND denied actions land as ring events
(D35 audit). Monitors wanting a button press this panel through the same gates - there
is no second control channel.

Honest limits: dl per-plugin resource remains n/a (D22 physics, not laziness - panels
show aggregates, not illusions); the monitor binary itself obeys this section's sibling
(§7 host lifecycle) - the oxmgr lesson: a watcher that ignores shutdown blinds the house.
