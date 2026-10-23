# Wave-2 T1 plan - the v3 clock door (in-process)

> STATUS: LANDED 2026-10-21 - executed by the T1 worker, independently
> re-measured by the orchestrator (254/55, sabotage red observed twice,
> frozen audit deletions=0, check.sh green); evidence row in
> docs/stage-5-plan.md CLOCK lane. Reviewed OKAY by Momus beforehand (4
> corrections folded in - exe interim recast included). The CONTRACT
> shape is already ruled (D99
> hearing, rulings B,A,A,A,A,A in docs/wave-2-clock-hearing.md §8); this file
> only plans the landing of lane T1 (door half). T2 (crossing) is a separate
> go order and NOT in this plan. Execution starts only on maintainer word.

## 1. Scope (what T1 ships)

Per D99 R1-R6, in-process only:

- `HostAbiV3` in `abi.rs`: V2 fields byte-identical in front + TWO tail slots
  - `timer_schedule(user, deadline_ns u64, job_label cstr, out_token u64*) -> i32`
  - `clock_now(user, out_now_ns i64*) -> i32`
  plus `ABI_VERSION_V3: u32 = 3`, typedefs, safe-Rust sugar views (D63 rule:
  sugar mirrors v2's style, never a contract direction).
- Kernel-side door wiring (dl executor only):
  - `executor.rs`: v3 branch in `validate` mirroring `validate_v2` (same
    descriptor guards, declared-major equality).
  - `kernel.rs` `start`: v3 branch mirroring the v2 branch - new module
    `host_v3.rs::start_v3(&mut handle, &registry, &bus, clock-register hooks)`.
  - `host_v3.rs`: trampolines for the two new slots (mirror host_v2 grammar:
    out-param + status; negative D51 codes for bad input) reusing
    `trampoline_register_service` + the four bus trampolines from v2.
- The per-plugin timer register (NEW kernel state, D99 R4-A):
  - `kernel.rs` (insertion): `plugin_timers: Mutex<HashMap<u64 token, PluginId>>`.
  - `timer_schedule` trampoline: calls `clock.timer(deadline,label)`, records
    token -> plugin id in the register.
  - `clock.rs` (NOT frozen): new internal removal
    `remove_by_tokens(&[u64]) -> usize` (scan due buckets; the ONLY cancel path).
  - stop/uninstall release (kernel.rs insertions): on plugin stop/uninstall,
    remove the plugin's tokens from the clock + the register
    ("token never re-fires" census; authority-law grammar, kernel-side - NOT an
    ABI verb).
- The bell posting (D99 R4-A/R5-A, T1 half = local direction):
  - `wiring.rs::post_jobs` (THE single due-funnel - step dial, real poll, and
    wire dial all pass here): for each fired job that is in `plugin_timers`,
    additionally `bus.publish(TopicId("org.modukit.timer.<plugin-id>"),
    QoS::Lossy(16), token u64 LE ++ label UTF-8)`. Host-owned timers keep the
    TimerDone ring posting EXACTLY as today; the bus post is additive and
    scoped to registered plugin tokens.
- Mirror faces (N1 recipe, extended): `contracts/modukit_plugin.h` HostAbiV3
  struct + doc comments (slot order = abi.rs field order - "ORDER IS THE
  CONTRACT" per check-embed-parity.sh / D96-r2, NOT D54); codegen
  pluginface emitter gains the v3 Rust/C++ views behind the same size-guard;
  parity trio rows; capability-ledger rows for the two new plugin verbs;
  HOST2-style ledger rows extend to v3 tables (follow the landed HOST2 pattern).
- Tests (wall-in only; deterministic via TestHost step clock):
  - `tests/abi_v3_door.rs` mirroring abi_v2_door: layout/expected_size,
    v3 fixture declares major 3, walks `clock_now` -> schedules on the dialed
    stamp + 1s -> TestHost advance -> bus recv on own topic answers the token
    payload; registrations still land; v1/v2 plugins unchanged (byte-identical
    prefix read).
  - new fixture timerv3-plugin (busv2 pattern) [into the rs fixture band at
    test-plugins/rs/ 2026-10-23, maintainer-ruled A].
  - stop-census test: schedule, stop the plugin, dial past deadline, assert
    no TimerDone post, no bus publish, pending count drops.
  - liar extension: v3-declared against the v1 guard... NO - v3 validates;
    a FALSE major 3 (bad descriptor) keeps DescriptorTooSmall/HandshakeIncomplete
    shapes; unknown major 4 stays AbiMismatch (guard already returns it).
  - exe interim pin (recast by review - the child runner loads through the
    SAME shared executor.rs::validate T1 opens, so a v3 plugin in an exe
    child passes the child's door and schedules on the child's own D74
    clock BY CONSTRUCTION - exactly R5-A local semantics). The honest T1
    interim: child-local bells reach child-local subscribers only; they do
    NOT cross the wall (the due-site bridge routing is T2). PIN THAT AS
    BEHAVIOR (an assertion or a named note in the landing row); no refusal
    insertion anywhere; T2 adds the crossing and flips the pin. The HELLO
    abi field is the runner's own self-report (=1) and never carries the
    plugin's declared major - no wall-side gate exists or is built here.
  - late-join replay PINNED (D99 R4-A): subscribe AFTER a firing on the same
    topic receives the ring entry (asserted behavior, not accidental).

## 2. Non-scope (explicit)

- T2: the parent->child pump call site, capability bit HELLO_CAP_TIMER_V2=2,
  bridge/FWD_PUB due-routing, cross-wall suites, demo leg + sabotage lever.
- The cross-revive token-generation dedup field: aliasing needs the ring to
  span a process restart; T1 is one kernel, tokens monotonic for its lifetime
  - candidate lands with T2 where the problem physically exists (named at D99).
- Embed/plugin ABI v1 surface: byte-identical, zero touch. v2 tables unchanged.
- Recurring/cancel verbs, now()-on-embed changes: none (D99 §3).
- Wire/proto/fdset: untouched - no new talker, so regen-diff must stay clean.

## 3. Frozen-surface discipline (D87/D83)

`abi.rs` and `kernel.rs` accept PURE INSERTIONS ONLY (the N1 precedent
record: 111/0, 14/0). `proc_wire.rs`, `executor_proc.rs`, manifest grammar,
`executor.rs`... executor.rs was N1-insertion (39/0) - keep insertions there
too. After landing: `git diff` audit line-by-line that every frozen-family file
shows deletions=0 (except the v3 branches' pure insertions), and check.sh green.

## 4. Verification / done-when (C0 grammar)

1. New suites green on the FIRST run after implementation (list the counts,
   redirected file parsing per PIT-9).
2. One planted red, observed: sabotage env `MODUKIT_TIMER_BELL_SABOTAGE=1`
   read at the post_jobs bus-publish site (the ONE named point) skips the bus
   post - the door suite's recv assertion then goes RED; remove, green.
   (This is the T1 test lever; T2 owns the wall lever.)
3. Frozen audit (deletions=0 on the D87 set) printed and logged in the row.
4. check.sh all green including the six existing demo legs + regen-diff +
   parity trio (the v3 rows must satisfy the extended parity script).
5. Orchestrator re-measures test totals independently (metric-integrity rule);
   lane rows land in docs/stage-5-plan.md per the D99 §6 ruling (the bus
   precedent file), and docs/wave-2-clock-hearing.md gains a §9 pointer row
   to them (hearing file stays the contract, not the evidence ledger).
6. Docs: capability-ledger rows + architecture §11 note arrives WITH the code
   commit (D99 re-review trigger honored); no memory-only claims.

## 5. Risks & named uncertainties

- post_jobs runs under the kernel plugin-table lock context (call sites hold
  locks?) - if bus.publish inside post_jobs ever re-enters registry/kernel
  locks, deadlock class; plan: bus.publish touches only Bus internals (no
  re-entry; verified in bus.rs today) - first test exercises the dial path,
  a hang is RED by construction.
- TopicId grammar for the composed `org.modukit.timer.<id>`: pinned by a unit
  assert BEFORE wiring (if the grammar refuses it, that is a finding to walk,
  not to work around).
- The exe interim (post-T1 child-local v3) is a PINNED behavior, not a
  refusal - if observation shows anything beyond 'child-local schedule
  works, crossing absent', STOP and disposition (hearing walk), don't widen.
- v3 size-guard: old v2 plugins must keep reading the V2-shaped prefix from a
  V3 host table - the table handed depends on declared major (v2 plugins get
  HostAbiV2 exactly as today - NOT a v3 table truncated; the door picks the
  struct by major, so prefix tests assert v2 handoff unchanged).

## 6. Review checklist

Contract fidelity to D99 R1-R4 (no invented semantics); frozen insertion-only;
every new mechanism has an observed-red lever or is refused at landing;
register lifecycle census proven; post_jobs funnel is the ONLY bus-post site;
no T2 sneak-ins.

## 7. Review result

PENDING - Momus.
