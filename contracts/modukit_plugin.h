/* modukit_plugin.h - the plugin-side self-report contract ORIGINAL
 * (gap ruling A, maintainer-adjudicated 2026-10-20: one contract original
 * per face; this header IS the contract). The plugin side of the C ABI -
 * the only cross-language exit: a `dl`-placed plugin exports the single
 * entry symbol `modukit_plugin_init`, which publishes a
 * `modukit_plugin_abi_v1` descriptor; the plugin's start() is then called
 * once with a borrowed `modukit_host_abi_v1` host table.
 *
 * Layout mirror of crates/modukit-core/src/abi.rs (PluginAbiV1 /
 * HostAbiV1, D87 STABLE). Every struct member keeps the EXACT snake name
 * of the Rust field it mirrors, in declaration order - the #[repr(C)]
 * field order IS the contract, and scripts/check-embed-parity.sh diffs
 * the ordered name lists (abi.rs <-> this header <-> the embed_abi.rs
 * field consts). A rename on one side must be a rename on all sides;
 * abi.rs is frozen, so renames here go through a D83 amendment hearing.
 *
 * Hand-authored; D83 tail-add amendment policy: new slots append to the
 * TABLE TAILS and `size` grows, existing entries never move or change
 * type. v1 tables are frozen (D87): a break is a hearing, never a silent
 * edit. (File order note: HostAbiV1 is declared first because the
 * PluginAbiV1 start slot names it; the Rust declaration order in abi.rs
 * is plugin-then-host and the per-field orders still mirror 1:1.)
 *
 * Coexists with contracts/modukit_embed.h (the application<->kernel
 * embed face): a TU including both compiles clean; no symbol declared by
 * embed.h is re-declared here.
 *
 * C shade conventions follow the codegen cabi precedent and embed.h:
 * NUL-terminated const char* names, byte buffers with explicit len/cap,
 * i32/i64 status on the int channel (D27 author code space).
 */
#ifndef MODUKIT_PLUGIN_H
#define MODUKIT_PLUGIN_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Host-side ABI major; mirrors `abi::ABI_VERSION` (u32 = 1, 04 §3: one
 * integer compare is the whole negotiation). The guard requires the
 * descriptor's abi_version to EQUAL this exactly - no range, no
 * "newer accepted". */
#define MODUKIT_ABI_VERSION 1u

/* The single exported entry symbol name; mirrors `abi::ENTRY_SYMBOL`
 * ("modukit_plugin_init", 04 §3 symbol discipline). */
#define MODUKIT_ENTRY_SYMBOL "modukit_plugin_init"

/* ------------------------------------------------------------------ */
/* Table tags forward-declared at file scope so the slot typedefs below
 * can name them (C parameter-list tag scoping bites otherwise). */
struct modukit_host_abi_v1;
struct modukit_plugin_abi_v1;

/* ------------------------------------------------------------------ */
/* Slot types - one typedef per abi.rs fn-pointer type. The Rust option
 * wrappers (Option<Fn>) are exactly "may be NULL" in C. */

/* Service call shape (mirrors `abi::ServiceCallFn`): message-shaped -
 * method name + owned bytes in, bytes out (04 §2 / 01 §3 rule 3).
 * Returns the number of bytes written to `output`, or a negative
 * plugin-side error code (D27). `method`/`input` are borrowed for the
 * call only; `output` is host-allocated and written only during the
 * call. Needing more than `output_cap` is a negative return, never a
 * truncation (v0 cap is max(input,64)+64; real sizing is the codegen
 * lane's job). */
typedef int32_t (*modukit_service_call_fn_t)(const char *method,
                                             const uint8_t *input,
                                             size_t input_len,
                                             uint8_t *output,
                                             size_t output_cap);

/* Host callback (mirrors `abi::RegisterServiceFn`): register one
 * service; returns the RegistrationId as a monotonic i64, or negative
 * on failure. `user` is the HostAbiV1 user context returned verbatim;
 * `service_id` is read during the call; `call` is retained by the host
 * for the registration's lifetime. */
typedef int64_t (*modukit_register_service_fn_t)(void *user,
                                                 const char *service_id,
                                                 modukit_service_call_fn_t call);

/* Activation (mirrors `abi::PluginStartFn`): called once with the host
 * context; non-zero = start failure (03 §2 STARTING -> FAILED). The
 * host table is borrowed for the duration of the call - plugins keep no
 * ambient authority past it (01 §3 rule 2). */
typedef int32_t (*modukit_plugin_start_fn_t)(const struct modukit_host_abi_v1 *host);

/* Cooperative stop (mirrors `abi::PluginStopFn`): must be
 * idempotent-friendly (01 §3 rule 6). Returns non-zero on failure. */
typedef int32_t (*modukit_plugin_stop_fn_t)(void);

/* Entry function (mirrors `abi::PluginInitFn`): reads the descriptor,
 * runs no service logic. */
typedef const struct modukit_plugin_abi_v1 *(*modukit_plugin_init_fn_t)(void);

/* ------------------------------------------------------------------ */
/* HostAbiV1 - the host table handed to start (CLAP `host_t` precedent;
 * mirrors `abi::HostAbiV1`). Valid only for the duration of the call.
 * The host fills `size` with the host-compiled size of this table. */
typedef struct modukit_host_abi_v1 {
    /* abi::HostAbiV1.size - table size, same tail-add guard style as the
     * descriptor's size (a plugin checks it is >= what it uses). */
    uint32_t size;
    /* abi::HostAbiV1.user - opaque kernel-side context, returned to the
     * callbacks verbatim. Never inspect it. */
    void *user;
    /* abi::HostAbiV1.register_service - never NULL on a v1 host. */
    modukit_register_service_fn_t register_service;
} modukit_host_abi_v1;

/* ------------------------------------------------------------------ */
/* PluginAbiV1 - the self-report descriptor (04 §3, D7 self-report tier;
 * mirrors `abi::PluginAbiV1`). Handshake guards the host runs before ANY
 * call (executor.rs `open_validated`/`validate` - the single source of
 * truth for the ABI v1 guards):
 *   1. the entry symbol resolves and init() returns non-NULL
 *      (missing symbol or NULL descriptor -> refused at RESOLVING,
 *       never a partial start);
 *   2. abi_version == MODUKIT_ABI_VERSION exactly (else ABI_MISMATCH);
 *   3. size >= the host's compiled sizeof(modukit_plugin_abi_v1)
 *      (else DESCRIPTOR_TOO_SMALL; a larger size is legal only as a
 *       D83 tail-add extension);
 *   4. start and stop are non-NULL (else HANDSHAKE_INCOMPLETE).
 * The struct is read-only publication data: the C-string members target
 * static bytes inside the plugin's own image, valid while the library
 * stays mapped; the host copies what it reads during the guards. */
typedef struct modukit_plugin_abi_v1 {
    /* abi::PluginAbiV1.abi_version - set to MODUKIT_ABI_VERSION (guard 2). */
    uint32_t abi_version;
    /* abi::PluginAbiV1.size - the plugin's compiled
     * sizeof(modukit_plugin_abi_v1); set exactly that (guard 3). */
    uint32_t size;
    /* abi::PluginAbiV1.name - human name (04 §3 shape). Read after the
     * guards pass; NULL degrades to a "<null>" placeholder, so provide
     * it. NUL-terminated static bytes in the plugin image. */
    const char *name;
    /* abi::PluginAbiV1.id - canonical reverse-DNS id for the future
     * cross-check (D7); not policed until M7, NULL allowed. */
    const char *id;
    /* abi::PluginAbiV1.version - the plugin's own semver string (05 §1),
     * independent of the ABI major; NULL allowed. */
    const char *version;
    /* abi::PluginAbiV1.start - Option<PluginStartFn>: must be non-NULL
     * (guard 4). */
    modukit_plugin_start_fn_t start;
    /* abi::PluginAbiV1.stop - Option<PluginStopFn>: must be non-NULL
     * (guard 4). */
    modukit_plugin_stop_fn_t stop;
} modukit_plugin_abi_v1;

/* The plugin's export, declared here so a conforming implementation is
 * signature-checked at its definition. The host resolves it by name
 * (MODUKIT_ENTRY_SYMBOL) and calls it at most during the handshake. */
const modukit_plugin_abi_v1 *modukit_plugin_init(void);

/* ------------------------------------------------------------------ */
/* Stage-5 N1 v2 door (D96 ruling 2) - append-only below. A plugin that
 * declares abi_version = MODUKIT_ABI_VERSION_V2 in its PluginAbiV1
 * descriptor is handed a `modukit_host_abi_v2` table at start (the v1
 * prefix plus the four mailbox slots, mirroring the embed face grammar
 * exactly - zero reverse-direction function pointers). HostAbiV1 and the
 * guards above are untouched; a v1 host never publishes this table. */

/* Host-side ABI major v2; mirrors `abi::ABI_VERSION_V2`. The door accepts
 * declared majors 1 and 2; anything else stays ABI_MISMATCH at RESOLVING
 * (the liar guard is unchanged, D87/D96). */
#define MODUKIT_ABI_VERSION_V2 2u

/* Mailbox slots (mirrors `abi::BusPublishFn` etc., embed-face grammar):
 * `user` is the HostAbiV2 user context returned verbatim; `qos` rides the
 * `modukit_qos` int vocabulary of contracts/modukit_embed.h; status is
 * the negated-D51 int channel (D27/D51). The queue ids a subscribe answers
 * live only while the host table does (the start-call authority law above). */
typedef int32_t (*modukit_bus_publish_fn_t)(void *user, const char *topic,
                                            int32_t qos, const uint8_t *value,
                                            size_t value_len);
typedef int32_t (*modukit_bus_subscribe_fn_t)(void *user, const char *topic,
                                              int32_t *out_queue_id);
/* recv: oldest buffered value out; NULL `out_value` is the size query and
 * does NOT consume; too-small `value_cap` reports the need through
 * `*out_len` and returns negated QUOTA_EXCEEDED (never a truncation). */
typedef int32_t (*modukit_bus_recv_fn_t)(void *user, int32_t queue_id,
                                         uint8_t *out_value, size_t value_cap,
                                         size_t *out_len);
typedef int32_t (*modukit_bus_unsubscribe_fn_t)(void *user, int32_t queue_id);

/* HostAbiV2 - the v2 host table (mirrors `abi::HostAbiV2`). The three v1
 * fields keep their exact order and types, so the table doubles as a valid
 * `modukit_host_abi_v1` prefix; a plugin reads the bus slots only after
 * checking `size >= sizeof(modukit_host_abi_v2)`. */
typedef struct modukit_host_abi_v2 {
    /* abi::HostAbiV2.size - table size (>= the v1 size for a real v2). */
    uint32_t size;
    /* abi::HostAbiV2.user - opaque kernel-side context, verbatim to slots. */
    void *user;
    /* abi::HostAbiV2.register_service - the v1 slot, unchanged semantics. */
    modukit_register_service_fn_t register_service;
    /* abi::HostAbiV2.bus_publish */
    modukit_bus_publish_fn_t bus_publish;
    /* abi::HostAbiV2.bus_subscribe */
    modukit_bus_subscribe_fn_t bus_subscribe;
    /* abi::HostAbiV2.bus_recv */
    modukit_bus_recv_fn_t bus_recv;
    /* abi::HostAbiV2.bus_unsubscribe */
    modukit_bus_unsubscribe_fn_t bus_unsubscribe;
} modukit_host_abi_v2;

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* MODUKIT_PLUGIN_H */
