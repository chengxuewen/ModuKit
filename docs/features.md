# Feature gallery

One page answers "what can ModuKit do, where do I SEE it run, and how do I
use it from my band?" Three columns per capability: **run it** (the standing
gate leg that proves it nightly - `pixi run demo <leg>`), **read it** (the
canon deep-dive), **use it** (the band face). Numbers and verb lists live in
the [capability ledger](capability-ledger.md), never here. Learning paths:
the [tutorials](tutorials/README.md).

## Plugin lifecycle

| Capability | Run it | Read it | Use it |
|---|---|---|---|
| Install protocol (D28: stage/verify/commit, tamper RED) | `demo stage1` | [modules 05](modules/05-manifest-and-deployment.md) | every band's host demo; tutorial 02 |
| Six-state lifecycle + FAILED edge (D3) | `demo stage1` | [module 03](modules/03-lifecycle-state-machine.md) | embed verbs `install/resolve/start/state/stop/uninstall` (25-verb face, all bands) |
| exe placement + supervisor + crash/revive budget | `demo stage2` | [module 01](modules/01-plugin-placement-model.md) §4 | `exe` manifests + the `sup_*` verbs |
| ORPHAN-PROOF (SIGKILL census, zero strays) | `demo stage2` / `demo stage4-m2` | [module 01](modules/01-plugin-placement-model.md) | automatic; the census is the proof |

## Service model (plugin-management-plan S1-S3)

| Capability | Run it | Read it | Use it |
|---|---|---|---|
| RFC1960-subset property filter query | `demo stage4-m1` (the C host drives `find_services`) | [module 02](modules/02-service-model.md); ledger section A | embed verb `modukit_embed_find_services` (C/C++/py/js skins) |
| Service events: Registered/Unregistered/Modified | (registry-level Rust API; `demo stage1` walks presence) | [module 02](modules/02-service-model.md) §3 | Rust `subscribe` / `subscribe_filtered` |
| Filtered subscription | (Rust API; the tracker rides it) | [module 02](modules/02-service-model.md) §5 | Rust `subscribe_filtered(filter)` |
| ServiceTracker (callback-driven, transition-only) | rides `demo stage4-m1` via the C band walk | [module 02](modules/02-service-model.md) §5 | Rust `tracker.rs` monitor |
| requires version ranges | manifest grammar; resolve RED on miss | [module 05](modules/05-manifest-and-deployment.md) §3 | manifest `version = ">=1.2,<2"` |
| DM-lite: on_down drives lifecycle (block/restart-self/fail-self) | (resolve + presence grammar; monitor in `dm.rs`) | [module 02](modules/02-service-model.md) §5 | manifest `on_down`; Rust `dm.rs` |
| ConfigAdmin hot lane (D29; S3-2) | **no demo leg yet** - canon is the honest path | [module 05](modules/05-manifest-and-deployment.md) §6 | Rust `set_config/get_config`; manifest `config_live` |

## Communication and time

| Capability | Run it | Read it | Use it |
|---|---|---|---|
| Topic bus v1 (LatestValue, late-join) | `demo stage1` | [module 06](modules/06-topics-and-bus.md) | embed verbs `bus_publish/subscribe/recv/unsubscribe` |
| Bus v2 mailbox doors (D96) | `demo stage5-n2` | [module 06](modules/06-topics-and-bus.md); ledger section C | v2 host table slots (plugin-side) |
| Clock v3 + timer bell (D99; no cancel - the kernel releases tokens) | `demo wave2-clock` | [module 03](modules/03-lifecycle-state-machine.md) clock section | v3 host table `timer_schedule/clock_now` |
| Parent-to-child push (D108/D110, bounded outbox) | wave-3 legs in the gate | [../wave-3-parent-push-plan.md](wave-3-parent-push-plan.md); ledger section D | parent fanout pump |

## Process wall and transport

| Capability | Run it | Read it | Use it |
|---|---|---|---|
| proc-wire TL frames (11 wire kinds, D87-frozen) | every exe leg | [module 07](modules/07-transport-architecture.md) | the skin per band (tutorial 06) |
| Transport seam: socketpair/memfd + SCM_RIGHTS + iceoryx2 + vendor-shm | gate transport legs | [module 07](modules/07-transport-architecture.md) §2-4 | `modukit-transport` crate |
| Script plugin doors (D106/D107: py/js children speak the wire) | `demo wave2-plugin-py` / `-js` | [module 04](modules/04-contract-and-codegen.md); tutorial 06 | `bindings/{py,js}/plugin_runtime.*` skins |

## Observability and ops

| Capability | Run it | Read it | Use it |
|---|---|---|---|
| Introspect trio (snapshot/events/ledger, documented codecs) | `demo stage4-m1` scenes [9]-[10] | [module 10](modules/10-observability-and-debugging.md) | embed verbs `introspect_snapshot/events`; platform services |
| Logger (ring; logs ride the event ring, D19) | every leg | [module 10](modules/10-observability-and-debugging.md) | the `modukit.logger` platform service |
| Debug reader (observability floor) | `demo wave2-reader` | [module 10](modules/10-observability-and-debugging.md) | the `modukit-debug-reader` bin |
| Ops face: lb / query / dm diagnose (read-only) | `modukit-host <manifest> --ops=lb` | [module 03](modules/03-lifecycle-state-machine.md); `kernel.rs` `dm_report` | the `modukit-host` binary |

## Build and toolchain

| Capability | Run it | Read it | Use it |
|---|---|---|---|
| Official preset family (2 routes x 3 types, hidden bases) | `cmake --preset default` (etc.) | [env.md](env.md) preset section | presets; `MODUKIT_BUILD_CPP` / `MODUKIT_BUILD_C` options |
| One-key demo gallery | `pixi run demo <leg>` | README Quickstart | `scripts/demo.sh` |
| Contract and codegen (proto single door, emitted faces + goldens) | gate legs (regen-fdset/parity/golden) | [module 04](modules/04-contract-and-codegen.md) | `modukit-codegen` |

## Deliberately NOT here (trigger-gated)

Update-in-place hot swap (A4), ServiceFactory (D4), RSA cross-machine (A6),
lazy activation (A3), Zenoh remote carrier / I420 media face (D89-K3), and
the video-frame walkthrough (module 08 is design-only). Each names its
trigger in the onboarding plan deviations or the plugin-management plan.
