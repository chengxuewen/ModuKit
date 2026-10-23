# ModuKit Capability Ledger (Stage-4 M1 lane #7)

The matrix of what each language surface actually ships, verb by verb.
Adopted by `docs/stage-4-plan.md` §4 ("capability ledger gate", trunk-first
doctrine, ruled 2026-10-19); shipped with the M1 surfaces per §5 lane #7.

Semantics (the adjudicated rule, copied):
- **One row = one header verb.** Section A rows mirror `EMBED_VERBS`
  (`crates/modukit-core/src/embed_abi.rs`), itself pinned verb-for-verb to
  `contracts/modukit_embed.h` by `scripts/check-embed-parity.sh`.
- **RED iff a header verb lacks a row** — enforced by
  `scripts/check-capability-ledger.sh` (a `scripts/check.sh` leg).
- **Unshipped languages ride a `planned` column, never a red condition**
  (green from day one; tightens as skins ship).
- **The row updates in the SAME change that adds the verb** (D83 tail-add:
  new verbs append at the section tail; existing rows never move).

Machine-readable row format: every Section A row begins
`` | `verb_name` | `` — the backticked name is exactly the EMBED_VERBS
string. The gate greps that shape; keep it when editing rows.

Cell vocabulary — **contract existence vs implementation existence** is the
distinction the tokens encode (the embed face ships contract + wrapper +
mirror; the Rust `extern "C"` EXPORT landed with the M1 demo lane —
`crates/modukit-core/src/embed.rs`, consumed by `examples/cpp/host_demo.cc`):

| Token | Meaning |
|---|---|
| `header` | declared in the C contract original — the CONTRACT exists, no impl claim |
| `generated` | shipped generated wrapper (artifact cited in the row's note) |
| `native†` | RETIRED token (pre-M1): op existed in modukit-core without the `extern "C"` export; the M1 demo lane shipped the export, Section A rows now read `native` |
| `native` | Rust implementation shipped and consumed as-is (v1 frozen surface, D87) |
| `planned` | language skin not shipped; never a red condition |
| `n-a` | the row does not apply to that surface |

Citations (legend to keep rows short):
- `header` (A) = `contracts/modukit_embed.h`; `header` (B) = `contracts/modukit_plugin.h`.
- `generated` embed wrapper = `crates/modukit-codegen/testdata/golden/modukit_embed.hpp`
  (one `Kernel` forwarding method per verb, lane #3 face B).
- `generated` plugin dispatch = `crates/modukit-codegen/testdata/golden/com_modukit_demo.hpp`
  (C++ service skeleton + `Dispatch` byte door, lane #3 face A).

- `generated` Python embed floor = `crates/modukit-codegen/testdata/golden/modukit_embed.py`
  (enum mirrors + one ctypes prototype per verb across both rungs - the 16
  rung-0 verbs since M1 and the 8 `modukit_sup_*` verbs since M2; emitter
  `crates/modukit-codegen/src/py.rs`, D100 R2-A; proof chain:
  `examples/py/host_demo.py` + `scripts/demo-wave2-py.sh` (rung-0) and
  `examples/py/host_sup_demo.py` + `scripts/demo-wave2-py-sup.sh` (sup).
- `generated` JS embed bridge + facade = `crates/modukit-codegen/testdata/golden/
  `modukit_embed_bridge.rs` + `modukit_embed.js` (one napi forwarder per verb across
  both rungs over the dlopened C door + the CommonJS facade with enum/error mirrors -
  the 16 rung-0 verbs since M1 and the 8 `modukit_sup_*` exe-rung verbs since M2, the
  python line's shape; emitter `crates/modukit-codegen/src/js.rs`, D102 R2-A; proof
  chain: `examples/js/pin_test.js` + `examples/js/host_demo.js` +
  `scripts/demo-wave2-js.sh` (rung-0) and `examples/js/host_sup_demo.js` +
  `scripts/demo-wave2-js-sup.sh` (sup)).
- `embed.rs` ops per verb-group: lifecycle `kernel.rs`; service call
  `kernel.rs`/`registry.rs`; bus `bus.rs` (`recv` = `Receiver::try_recv` + the
  embed queue table, `unsubscribe` = table removal + `Receiver` drop); clock
  `clock.rs`; introspection `introspect.rs`/`events.rs` (opaque codecs
  documented in `embed.rs`/`embed_codec.rs`); `modukit_embed_abi_version`
  returns the bookkeeping const `EMBED_ABI_VERSION` (`embed_abi.rs`); the C++
  proof chain: `examples/cpp/host_demo.cc` + `scripts/demo-stage4-m1.sh`.
- `planned` Python: wave-2 binding candidate (`docs/stage-4-plan.md` §6).

Last verified: 2026-10-20 by `scripts/check-capability-ledger.sh`
(check.sh leg "capability-ledger").

## A. HostEmbedV1 embed verbs (24 rows = EMBED_VERBS)

| Capability | C | C++ | Rust | Python | JS |
|---|---|---|---|---|---|
| `modukit_embed_abi_version` | header | generated | native | generated | generated |
| `modukit_install` | header | generated | native | generated | generated |
| `modukit_uninstall` | header | generated | native | generated | generated |
| `modukit_resolve` | header | generated | native | generated | generated |
| `modukit_start` | header | generated | native | generated | generated |
| `modukit_stop` | header | generated | native | generated | generated |
| `modukit_state` | header | generated | native | generated | generated |
| `modukit_service_call` | header | generated | native | generated | generated |
| `modukit_bus_publish` | header | generated | native | generated | generated |
| `modukit_bus_subscribe` | header | generated | native | generated | generated |
| `modukit_bus_recv` | header | generated | native | generated | generated |
| `modukit_bus_unsubscribe` | header | generated | native | generated | generated |
| `modukit_clock_now` | header | generated | native | generated | generated |
| `modukit_clock_advance` | header | generated | native | generated | generated |
| `modukit_introspect_snapshot` | header | generated | native | generated | generated |
| `modukit_introspect_events` | header | generated | native | generated | generated |
| `modukit_sup_install` | header | generated | native | generated | generated |
| `modukit_sup_stop` | header | generated | native | generated | generated |
| `modukit_sup_state` | header | generated | native | generated | generated |
| `modukit_sup_step` | header | generated | native | generated | generated |
| `modukit_sup_dial` | header | generated | native | generated | generated |
| `modukit_sup_restart` | header | generated | native | generated | generated |
| `modukit_sup_uninstall` | header | generated | native | generated | generated |
| `modukit_sup_service_call` | header | generated | native | generated | generated |

## B. Plugin-side surface (contracts/modukit_plugin.h <-> abi::PluginAbiV1/HostAbiV1)

The generated C++ plugin face today is the service dispatch skeleton only
(golden cited above); entry-symbol and descriptor shims are hand-written by
the plugin author, so those rows are `planned` until an emitter ships them.
The Rust column here is the frozen v1 truth: `abi.rs` layout, consumed by
`executor.rs` handshake guards (`open_validated`/`validate`).

| Capability | C | C++ | Rust | Python | JS |
|---|---|---|---|---|---|
| `modukit_plugin_init` (entry symbol) | header | planned | native | planned | planned |
| `abi_version` (PluginAbiV1) | header | planned | native | planned | planned |
| `size` (PluginAbiV1) | header | planned | native | planned | planned |
| `name` (PluginAbiV1) | header | planned | native | planned | planned |
| `id` (PluginAbiV1) | header | planned | native | planned | planned |
| `version` (PluginAbiV1) | header | planned | native | planned | planned |
| `start` (PluginAbiV1) | header | planned | native | planned | planned |
| `stop` (PluginAbiV1) | header | planned | native | planned | planned |
| `register_service` (HostAbiV1 slot) | header | planned | native | planned | planned |
| `modukit_service_call_fn_t` (service-call slot) | header | generated | native | planned | planned |


## C. Plugin-side v2 door - HostAbiV2 slots (Stage-5 N1, D96 ruling 2/5)

The SECOND host table, negotiated by declared ABI major; the v1 tables in
section B stay frozen above (D87). Row keys mirror the `abi::HostAbiV2`
field names (`HOST2_ABI_FIELDS` in `embed_abi.rs`);
`scripts/check-capability-ledger.sh` requires one row per HOST2 field
(same existence-only rule as section A - the ordered layout truth lives in
the parity gate). Rust column: `abi.rs` table + `host_v2.rs` wiring over
the SAME in-process `Bus` (`bus.rs`). C++ `generated` =
`crates/modukit-codegen/testdata/golden/modukit_plugin.hpp` (one
forwarding method per mailbox slot, behind the declared major; `size`/`user`
are the guard's data, not verbs, so they read `header`). Proof chain:
`test-plugins/busv2-plugin` + `crates/modukit-core/tests/abi_v2_door.rs`.

| Capability | C | C++ | Rust | Python | JS |
|---|---|---|---|---|---|
| `size` (HostAbiV2 slot) | header | header | native | planned | planned |
| `user` (HostAbiV2 context) | header | header | native | planned | planned |
| `register_service` (HostAbiV2 slot) | header | generated | native | planned | planned |
| `bus_publish` (HostAbiV2 verb) | header | generated | native | planned | planned |
| `bus_subscribe` (HostAbiV2 verb) | header | generated | native | planned | planned |
| `bus_recv` (HostAbiV2 verb) | header | generated | native | planned | planned |
| `bus_unsubscribe` (HostAbiV2 verb) | header | generated | native | planned | planned |
## D. Plugin-side v3 clock door - HostAbiV3 slots (Wave-2 T1, D99 R2/R3/R4)

The THIRD host table, negotiated by declared ABI major: the byte-identical
v2 prefix (section C) plus the two timer slots at the tail; the v1/v2 tables
stay frozen above (D87/D96). Row keys mirror the `abi::HostAbiV3` field
names (`HOST3_ABI_FIELDS` in `embed_abi.rs`);
`scripts/check-capability-ledger.sh` requires one row per HOST3 field
(same existence-only rule as sections A/C - the ordered layout truth lives
in the parity gate). Rust column: `abi.rs` table + `host_v3.rs` door
trampolines over the kernel's `Clock` (`clock.rs`), with the due bell
posted at the ONE due-funnel `wiring.rs::post_jobs` onto the per-plugin
topic `org.modukit.timer.<plugin-id>` (token LE ++ label, `Lossy(16)`,
late-join replay pinned) and the authority-law token release at
stop/uninstall (`kernel.rs`). The seven prefix rows repeat here because
they are HostAbiV3's fields too - the C++ `generated` column rides the
same v2 trampolines. C++ `generated` =
`crates/modukit-codegen/testdata/golden/modukit_plugin.hpp` (one forwarding
method per parsed slot, the timer pair behind the v3 size guard; `size`/
`user` are the guard's data, not verbs, so they read `header`). Proof
chain: `test-plugins/timerv3-plugin` +
`crates/modukit-core/tests/abi_v3_door.rs`.

| Capability | C | C++ | Rust | Python | JS |
|---|---|---|---|---|---|
| `size` (HostAbiV3 slot) | header | header | native | planned | planned |
| `user` (HostAbiV3 context) | header | header | native | planned | planned |
| `register_service` (HostAbiV3 slot) | header | generated | native | planned | planned |
| `bus_publish` (HostAbiV3 slot) | header | generated | native | planned | planned |
| `bus_subscribe` (HostAbiV3 slot) | header | generated | native | planned | planned |
| `bus_recv` (HostAbiV3 slot) | header | generated | native | planned | planned |
| `bus_unsubscribe` (HostAbiV3 slot) | header | generated | native | planned | planned |
| `timer_schedule` (HostAbiV3 verb) | header | generated | native | planned | planned |
| `clock_now` (HostAbiV3 verb) | header | generated | native | planned | planned |
