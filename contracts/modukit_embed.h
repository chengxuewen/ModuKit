/* modukit_embed.h - the HostEmbedV1 contract ORIGINAL (ruling C++-3: this
 * header IS the contract). The application<->kernel embed face: a host
 * application embedding the ModuKit kernel in the same process binds these
 * verbs directly; wrappers (Rust/C++) are generated FROM this file later
 * (M1 codegen lane), never hand-grown around it.
 *
 * Hand-authored; D83 tail-add amendment policy: new verbs append to the
 * vtable tail and bump the minor bookkeeping, existing entries never move
 * or change type. The two frozen v1 plugin tables (PluginAbiV1/HostAbiV1,
 * D87 STABLE) are NOT this surface and are untouched by it.
 *
 * C shade conventions follow the codegen cabi precedent: opaque handle
 * pointers, byte buffers with explicit len/cap params, i32 status (>= 0 ok,
 * < 0 is a MODUKIT_EMBED_* code from the D51 dictionary below), NUL-
 * terminated const char* names.
 */
#ifndef MODUKIT_EMBED_H
#define MODUKIT_EMBED_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Embed table major version; `modukit_embed_abi_version()` returns this. */
#define MODUKIT_EMBED_ABI_VERSION 1u

/* ------------------------------------------------------------------ */
/* D51 kernel error dictionary mirror (the 19-name KernelCode enum of
 * contracts/org.modukit.runtime.proto; numeric values copied VERBATIM,
 * RESERVED_n slots keep their slot numbers frozen).
 * Return channel: every verb returns 0 on success, a negative value of
 * one of these codes on kernel-domain failure. */
typedef enum modukit_kernel_code {
    MODUKIT_KERNEL_CODE_UNSPECIFIED = 0,
    /* lifecycle domain */
    MODUKIT_MANIFEST_INVALID = 1,
    MODUKIT_DEPENDENCY_MISSING = 2,
    MODUKIT_DEPENDENCY_CYCLE = 3,
    MODUKIT_START_TIMEOUT = 4,
    MODUKIT_START_REJECTED = 5,
    MODUKIT_STOP_TIMEOUT = 6,
    MODUKIT_RESERVED_LIFECYCLE_7 = 7,
    MODUKIT_RESERVED_LIFECYCLE_8 = 8,
    /* contract domain */
    MODUKIT_ABI_MISMATCH = 9,
    MODUKIT_CONTRACT_VIOLATION = 10,
    MODUKIT_SCHEMA_INVALID = 11,
    MODUKIT_RESERVED_CONTRACT_12 = 12,
    MODUKIT_RESERVED_CONTRACT_13 = 13,
    /* runtime domain */
    MODUKIT_NOT_FOUND = 14,
    MODUKIT_EPOCH_EXPIRED = 15,
    MODUKIT_CARRIER_UNREACHABLE = 16,
    MODUKIT_RESERVED_RUNTIME_17 = 17,
    MODUKIT_RESERVED_RUNTIME_18 = 18,
    /* resource domain */
    MODUKIT_QUOTA_EXCEEDED = 19,
    MODUKIT_CAPABILITY_DENIED = 20,
    MODUKIT_RESERVED_RESOURCE_21 = 21,
    MODUKIT_RESERVED_RESOURCE_22 = 22,
    /* supervision domain (S2-M5b append-only tail) */
    MODUKIT_PROCESS_CRASHED = 23,
    MODUKIT_IDENTITY_MISMATCH = 24,
    MODUKIT_SERVICE_CALL_FAILED = 25,
    MODUKIT_DEFERRAL_DECLINED = 26,
    MODUKIT_CLOCK_DIAL_REFUSED = 27
} modukit_kernel_code;

/* Plugin lifecycle states - numeric mirror of `lifecycle::PluginState`
 * (the kernel returns these through `modukit_state` verbatim). */
typedef enum modukit_plugin_state {
    MODUKIT_STATE_UNINSTALLED = 0,
    MODUKIT_STATE_INSTALLED = 1,
    MODUKIT_STATE_RESOLVING = 2,
    MODUKIT_STATE_STARTING = 3,
    MODUKIT_STATE_ACTIVE = 4,
    MODUKIT_STATE_STOPPING = 5,
    MODUKIT_STATE_RESOLVED = 6,
    MODUKIT_STATE_FAILED = 7
} modukit_plugin_state;

/* Clock domains - numeric mirror of `clock::ClockDomain`. */
typedef enum modukit_clock_domain {
    MODUKIT_CLOCK_REAL = 0,
    MODUKIT_CLOCK_SIM = 1,
    MODUKIT_CLOCK_STEP = 2
} modukit_clock_domain;

/* Bus QoS kinds - numeric mirror of `topic::QoS`. */
typedef enum modukit_qos {
    MODUKIT_QOS_LATEST_VALUE = 0,
    MODUKIT_QOS_KEEP_ALL = 1,
    MODUKIT_QOS_LOSSY = 2,
    MODUKIT_QOS_BOUNDED = 3
} modukit_qos;

/* One merged observability event (a flat numeric mirror of the ring's
 * ObsEvent kind; payloads ride the serialized bytes). */
typedef enum modukit_event_kind {
    MODUKIT_EVENT_LIFECYCLE = 0,
    MODUKIT_EVENT_REGISTRY = 1,
    MODUKIT_EVENT_MODULE_LOADED = 2,
    MODUKIT_EVENT_TIMER_DONE = 3,
    MODUKIT_EVENT_EDGE_REMOVED = 4,
    MODUKIT_EVENT_MARKER = 5,
    MODUKIT_EVENT_LOG = 6
} modukit_event_kind;

/* One merged observability record: kind + seq + nanosecond stamp +
 * serialized payload. Byte discipline follows the kernel event ring. */
typedef struct modukit_event {
    int32_t kind;   /* modukit_event_kind */
    uint64_t seq;   /* ring sequence (resumable read cursor) */
    uint64_t ts_ns; /* unified-clock stamp */
    const uint8_t *payload;
    size_t payload_len;
} modukit_event;

/* ------------------------------------------------------------------ */
/* Table bookkeeping */

/* Kernel verb: (bookkeeping - the embed face's own handshake guard).
 * Returns MODUKIT_EMBED_ABI_VERSION. Hosts check it before calling any
 * other verb; version discipline is D83 tail-add, never renumber. */
int32_t modukit_embed_abi_version(void);

/* ------------------------------------------------------------------ */
/* Lifecycle verbs (kernel.rs public in-process face) */

/* Kernel verb: install (UNINSTALLED -> INSTALLED).
 * manifest_toml is a NUL-terminated manifest-v0 TOML document.
 * The kernel copies the document during the call; ownership stays with
 * the caller. out_plugin_id receives a NUL-terminated canonical bundle id
 * (D9) backed by kernel-owned static registry storage, valid until
 * uninstall of that plugin; the caller copies it out and never frees it. */
int32_t modukit_install(const char *manifest_toml, const char **out_plugin_id);

/* Kernel verb: uninstall (any state -> UNINSTALLED; drops all of the
 * plugin's registrations). `id` is a canonical bundle id previously
 * returned by install; borrowed for the call only. */
int32_t modukit_uninstall(const char *id);

/* Kernel verb: resolve (INSTALLED -> RESOLVING -> STARTING|FAILED).
 * Checks contract/link/placement prerequisites; required dependencies
 * validated deterministically before any plugin code executes. */
int32_t modukit_resolve(const char *id);

/* Kernel verb: start (RESOLVED -> STARTING -> ACTIVE|FAILED).
 * The kernel calls the plugin's start() and parks FAILED on refusal. */
int32_t modukit_start(const char *id);

/* Kernel verb: stop (ACTIVE -> STOPPING -> RESOLVED|FAILED). Cooperative
 * drain: new calls are refused while old ones finish. */
int32_t modukit_stop(const char *id);

/* Kernel verb: state. out_state receives one modukit_plugin_state value. */
int32_t modukit_state(const char *id, int32_t *out_state);

/* ------------------------------------------------------------------ */
/* Service verbs */

/* Kernel verb: the minimal synchronous call door (02 §4 message-shaped:
 * bytes in, bytes out). `service_id` is the canonical service id (D9);
 * `method` and `input`/`input_len` are the call; the kernel copies the
 * response into caller-owned memory. Returns the number of response
 * bytes written on success (>= 0), a negative D51 code on failure.
 * A provider needing more than response_cap room fails the call rather
 * than truncating. */
int32_t modukit_service_call(const char *service_id, const char *method,
                             const uint8_t *input, size_t input_len,
                             uint8_t *response, size_t response_cap);

/* ------------------------------------------------------------------ */
/* Bus verbs (host-side semantics of bus.rs: publish never blocks; a
 * subscription is topic-scoped; late-join seeds per the declared QoS) */

/* Kernel verb: bus publish (broadcast, never blocks the caller - a slow
 * subscriber sheds its own oldest value and counts the drop). Bytes are
 * copied during the call; ownership stays with the caller. `qos` declares
 * the topic's retention kind (modukit_qos). */
int32_t modukit_bus_publish(const char *topic, int32_t qos,
                            const uint8_t *value, size_t value_len);

/* Kernel verb: bus subscribe. Registers the caller as a topic-scoped
 * subscriber and returns a queue id (>= 0) for modukit_bus_recv; a
 * deferred QoS (KEEP_ALL/BOUNDED) refuses with MODUKIT_DEFERRAL_DECLINED.
 * The queue id is owned by the kernel until the host calls
 * modukit_bus_unsubscribe; a dropped (never-unsubscribed) queue id is
 * reclaimed by the kernel when its publisher side drops. */
int32_t modukit_bus_subscribe(const char *topic, int32_t *out_queue_id);

/* Kernel verb: bus receive (the receiver side of the subscription). If
 * out_value is non-NULL it receives the oldest buffered value, copied
 * into caller memory, bounded by value_cap; if out_value is NULL the
 * verb reports the required size through *out_len without copying.
 * Returns the byte count written, 0 when the queue is empty, negative
 * code on a bad queue id. */
int32_t modukit_bus_recv(int32_t queue_id, uint8_t *out_value,
                         size_t value_cap, size_t *out_len);

/* Kernel verb: bus unsubscribe. Releases the queue id; the id becomes
 * invalid for modukit_bus_recv immediately (negative code afterwards). */
int32_t modukit_bus_unsubscribe(int32_t queue_id);

/* ------------------------------------------------------------------ */
/* Clock verbs (D20 one water source; dial exists on Sim/Step only) */

/* Kernel verb: clock now. The unified-clock stamp in nanoseconds. */
int32_t modukit_clock_now(int64_t *out_now_ns);

/* Kernel verb: clock advance (Sim/Step domains only). Turns the dial by
 * delta_ns; returns MODUKIT_CLOCK_DIAL_REFUSED on the Real domain (no
 * dial exists - never a silent no-op). */
int32_t modukit_clock_advance(int64_t delta_ns);

/* ------------------------------------------------------------------ */
/* Introspection verbs (read-only: the observability floors) */

/* Kernel verb: introspect snapshot. Serializes the object-table cut (the
 * same answer the `org.modukit.snapshot` service gives - one source of
 * truth) into out_buf. If out_buf is non-NULL, at most buf_cap bytes are
 * written; if out_buf is NULL, *out_len receives the required size
 * without writing. Returns bytes written (>= 0) or a negative code. */
int32_t modukit_introspect_snapshot(uint8_t *out_buf, size_t buf_cap,
                                    size_t *out_len);

/* Kernel verb: introspect events (resumable merged ring read). Copies
 * records with seq > since_seq into out_buf (at most buf_cap bytes),
 * reports bytes written via *out_len and the new cursor via *out_seq.
 * Returns bytes written (>= 0), 0 when no records remain, negative code
 * on a bad cursor. */
int32_t modukit_introspect_events(uint64_t since_seq, uint8_t *out_buf,
                                  size_t buf_cap, size_t *out_len,
                                  uint64_t *out_seq);

/* ------------------------------------------------------------------ */
/* The vtable (HostEmbedV1). Same shape discipline as the codegen C shade:
 * handshake guards first (version + size as compiled by THIS header's
 * consumer), then one slot per verb, appended in declaration order. A
 * host fills the slots with the implementations above (or its own
 * equivalents); embedding consumers call through the table, never
 * assuming link-time symbols beyond the version probe. */
typedef struct modukit_host_embed_v1 {
    /* handshake guards first (04 §3 discipline) */
    uint32_t abi_version; /* MODUKIT_EMBED_ABI_VERSION as compiled */
    uint32_t size;        /* sizeof(struct modukit_host_embed_v1) */

    /* lifecycle */
    int32_t (*install)(const char *manifest_toml, const char **out_plugin_id);
    int32_t (*uninstall)(const char *id);
    int32_t (*resolve)(const char *id);
    int32_t (*start)(const char *id);
    int32_t (*stop)(const char *id);
    int32_t (*state)(const char *id, int32_t *out_state);

    /* service call door */
    int32_t (*service_call)(const char *service_id, const char *method,
                            const uint8_t *input, size_t input_len,
                            uint8_t *response, size_t response_cap);

    /* bus */
    int32_t (*bus_publish)(const char *topic, int32_t qos,
                           const uint8_t *value, size_t value_len);
    int32_t (*bus_subscribe)(const char *topic, int32_t *out_queue_id);
    int32_t (*bus_recv)(int32_t queue_id, uint8_t *out_value,
                        size_t value_cap, size_t *out_len);
    int32_t (*bus_unsubscribe)(int32_t queue_id);

    /* clock */
    int32_t (*clock_now)(int64_t *out_now_ns);
    int32_t (*clock_advance)(int64_t delta_ns);

    /* introspection (read-only) */
    int32_t (*introspect_snapshot)(uint8_t *out_buf, size_t buf_cap,
                                   size_t *out_len);
    int32_t (*introspect_events)(uint64_t since_seq, uint8_t *out_buf,
                                 size_t buf_cap, size_t *out_len,
                                 uint64_t *out_seq);
} modukit_host_embed_v1;

/* ------------------------------------------------------------------ */
/* M2 tail-add - the exe rung (D83 tail-add; v1 major unchanged).
 *
 * TWO RUNGS, TWO AMBIENT KERNELS: the 16 verbs above drive the
 * in-process (dl) kernel; the modukit_sup_* verbs below drive a
 * SEPARATE process-global Supervisor kernel (proc_supervisor.rs) that
 * owns child-process plugins. Services registered on one rung are NOT
 * visible to the other - registry-level unification is a future
 * ruling, deliberately not faked here. A dl manifest on sup_install
 * (or an exe manifest on modukit_install) is refused at the INSTALL
 * door (MANIFEST_INVALID) - the placement gate runs before any spawn
 * or dlopen. Child teardown machinery
 * (STOP_KILL_GRACE, spawn-door retries, fd ownership) stays
 * Rust-side: a C caller never sees a child fd or pid.
 */

/* Install one exe manifest (kind = "exe") and walk it in ONE call:
 * install -> resolve (spawn the runner + HELLO handshake) -> start.
 * Rung 0 is deliberately coarse: a failure returns the FIRST failing
 * door's code; the record's stage stays readable via modukit_sup_state.
 * A wrong-kind manifest is refused at this install door (see the rung
 * block above). out_plugin_id follows
 * the modukit_install string-table ownership rule (kernel-owned bytes;
 * copy, never free). */
int32_t modukit_sup_install(const char *manifest_toml,
                            const char **out_plugin_id);

/* Cooperative stop of a supervised plugin (grace window, then kill -
 * all Rust-side). Same 0 / negative-D51 channel as modukit_stop. */
int32_t modukit_sup_stop(const char *id);

/* State of a supervised plugin id; same int32_t modukit_plugin_state
 * codes as modukit_state. */
int32_t modukit_sup_state(const char *id, int32_t *out_state);

/* One supervisor pass: reap dead children, post due timers, apply the
 * restart policy. *out_actions counts children REAPED plus children
 * REVIVED this pass; timer postings and park decisions ride the event
 * ring instead (introspect_events). */
int32_t modukit_sup_step(int32_t *out_actions);

/* Dial wall time on a child's step clock (tests / replay): fires the
 * timers due after delta_ns. On success *out_status carries the
 * wire's DueTurn status code and *out_fired_at_ns the fired stamp;
 * individual due jobs ride the event ring. A silent child is killed
 * at DIAL_TIMEOUT with a typed refusal, never left half-answered. */
int32_t modukit_sup_dial(const char *id, int64_t delta_ns,
                         int32_t *out_status, uint64_t *out_fired_at_ns);

/* One restart request per manifest policy (D3). *out_revive: 0 =
 * Revived (respawned, re-started), 1 = Deferred (backoff window open;
 * step or dial past it), 2 = Parked (budget exhausted, FAILED - never
 * a silent loop). Unknown ids / wrong rung return the negative code. */
int32_t modukit_sup_restart(const char *id, int32_t *out_revive);

/* Uninstall a STOPPED supervised record (same door as modukit_uninstall
 * on the other rung's kernel). An ACTIVE id refuses with the typed
 * lifecycle code - stop it first; the child teardown belongs to
 * modukit_sup_stop, never to this door. */
int32_t modukit_sup_uninstall(const char *id);

#ifdef __cplusplus
} /* extern "C" */
#endif

#endif /* MODUKIT_EMBED_H */
