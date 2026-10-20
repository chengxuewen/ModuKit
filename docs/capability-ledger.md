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

## A. HostEmbedV1 embed verbs (16 rows = EMBED_VERBS)

| Capability | C | C++ | Rust | Python |
|---|---|---|---|---|
| `modukit_embed_abi_version` | header | generated | native | planned |
| `modukit_install` | header | generated | native | planned |
| `modukit_uninstall` | header | generated | native | planned |
| `modukit_resolve` | header | generated | native | planned |
| `modukit_start` | header | generated | native | planned |
| `modukit_stop` | header | generated | native | planned |
| `modukit_state` | header | generated | native | planned |
| `modukit_service_call` | header | generated | native | planned |
| `modukit_bus_publish` | header | generated | native | planned |
| `modukit_bus_subscribe` | header | generated | native | planned |
| `modukit_bus_recv` | header | generated | native | planned |
| `modukit_bus_unsubscribe` | header | generated | native | planned |
| `modukit_clock_now` | header | generated | native | planned |
| `modukit_clock_advance` | header | generated | native | planned |
| `modukit_introspect_snapshot` | header | generated | native | planned |
| `modukit_introspect_events` | header | generated | native | planned |

## B. Plugin-side surface (contracts/modukit_plugin.h <-> abi::PluginAbiV1/HostAbiV1)

The generated C++ plugin face today is the service dispatch skeleton only
(golden cited above); entry-symbol and descriptor shims are hand-written by
the plugin author, so those rows are `planned` until an emitter ships them.
The Rust column here is the frozen v1 truth: `abi.rs` layout, consumed by
`executor.rs` handshake guards (`open_validated`/`validate`).

| Capability | C | C++ | Rust | Python |
|---|---|---|---|---|
| `modukit_plugin_init` (entry symbol) | header | planned | native | planned |
| `abi_version` (PluginAbiV1) | header | planned | native | planned |
| `size` (PluginAbiV1) | header | planned | native | planned |
| `name` (PluginAbiV1) | header | planned | native | planned |
| `id` (PluginAbiV1) | header | planned | native | planned |
| `version` (PluginAbiV1) | header | planned | native | planned |
| `start` (PluginAbiV1) | header | planned | native | planned |
| `stop` (PluginAbiV1) | header | planned | native | planned |
| `register_service` (HostAbiV1 slot) | header | planned | native | planned |
| `modukit_service_call_fn_t` (service-call slot) | header | generated | native | planned |
