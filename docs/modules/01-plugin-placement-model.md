# 01 — Plugin Placement Model

D2. Placement is a deployment property; language is orthogonal.

## 1. The closed executor set

| Executor | Where code runs | Artifact | Transport | Ships |
|---|---|---|---|---|
| `dl` | host address space | `plugin.so` | local (in-proc call) | Stage 1 |
| `exe` | supervised child process | same `plugin.so` | wire over socketpair/stdio | Stage 2 (interface in Stage 1) |
| `wasm` | sandboxed instance | `plugin.wasm` | wasmtime host calls | Stage 5 |

New placements are added as executors, never as new plugin concepts.

## 2. Shadow-runner

`exe` placement does not require a separate executable per plugin. The host spawns a generic
runner, hands it the artifact path and an inherited fd:

```
modukit-run <plugin.so> --fd=N
```

The runner `dlopen`s the same `.so` the `dl` executor would have loaded, and pumps the same
contract messages over the fd. One artifact, two placements: this is what makes switching a
configuration change instead of a rebuild.

Prior art: dora operator vs node under one topic contract (`docs/reference/dora.md:48-52`);
ipc-channel inprocess vs cross-process backends of one API (`docs/reference/ipc-channel.md:14,96`);
VST3 Gateway and ROS2 components (industry knowledge, not repo-verified).

## 3. Switchability preconditions

A plugin may switch `dl <-> exe` only if all hold. The first five are code-level; the last two
are lifecycle-level.

1. All host access goes through contract services — no `dlsym` on the host, no host singletons,
   no direct `stdout`/`syslog` assumptions.
2. No ambient authority: the plugin receives no fd, socket, or `Arc` it did not request by name.
3. No raw pointers crossing a service call — messages carry registration ids and handles
   (architecture.md §5, modules/02-service-model.md).
4. No process-global mutation: no signal handlers, no `setenv`, no locale or allocator changes.
   (The `dl` loader refuses known-unsafe shapes where detectable; the rest is review.)
5. No threads that outlive a `stop()` call without joining; `dl` teardown is cooperative,
   `exe` teardown is `kill`.
6. `start()` must not assume an event loop exists yet; `stop()` must be idempotent.
7. Any shared-memory handle it obtains must be released before reporting stopped.

> **Rules 3 + 7 carry the frame clause (D69):** the pool + descriptor channel (07 §3) is the only path frame bytes take to a plugin, any placement; a `dl` plugin mapping the pool is permitted but trusted-tier-only.

**Switchability is a tested property.** The contract suite (04 §4) runs against the plugin under
both placements; a plugin claims dl/exe-switch only after both passes. Documentation-only claims
are not accepted.

## 4. Asymmetries the abstraction never hides

| | `dl` | `exe` |
|---|---|---|
| Blast radius of UB/crash | host corruption; process dies whole | plugin dies; host survives, `Failed` edge fires (03) |
| Crash blast radius is | identical to the host's | bounded and observable |
| Cost per call | in-proc | wire hops, serialization |
| Hot placement switch | needs restart of host | supervisor respawns with new policy |
| fd/allocator/signal namespace | shared | private |
| Debug | one debugger | two, plus the wire |

The state machine, the registry and the contract deliberately do **not** paper over the
crash-radius difference. `dl` has no `Failed` edge: if the plugin corrupts the host there is
nothing left to report to.

## 5. Trust veto

Trust tier (architecture.md §7) sets the floor:

- untrusted third-party -> `exe` minimum, `wasm` default;
- `dl` is permitted only for reviewed internal code.

Switch direction is policed: `dl -> exe` (more isolated) is free at deploy time;
`exe -> dl` is vetoed by the host — reducing isolation is not a deployment decision, it is a
security decision.

## 6. Cluster identity (D35)

ModuKit is a single-machine framework with carrier-transparent reach. Three statements: (1) every machine's host is fully sovereign - it installs/runs/supervises its own bundles and keeps working through network partitions (autonomy IS partition resilience); (2) cross-machine service access rides the carrier (zenoh discovery/transport) with call syntax identical to local - distance is a deployment fact (D14); (3) fleet orchestration (what ships where, version consistency, rolling updates) is the orchestrator's job (vehicle SM / K8s-class); ModuKit's two cooperation surfaces are the deploy manifest (packaging unit) and the idempotent install protocol (D28 - safe to replay). No logical multi-machine host, no placement scheduler, no split-brain semantics - refused as identity-level scope.

## 7. Deliberately not in year 1

ProcessExecutor implementation, the shadow-runner binary, cgroup/rlimit supervision,
witness-based liveness. Interfaces exist (so the closed set is real); implementations land in
Stage 2 with a real untrusted plugin to justify them.
