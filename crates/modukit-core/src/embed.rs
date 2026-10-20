//! The HostEmbedV1 embed face: the Rust implementation of every contract
//! verb declared in `contracts/modukit_embed.h` (the header IS the
//! contract, ruling C++-3). The verb list stays pinned to the mirror by
//! `scripts/check-embed-parity.sh`; the generated C++ wrapper
//! (`modukit_embed.hpp`) forwards its methods to the free symbols below.
//!
//! DESIGN DECISIONS (settled here — the contract absorbed all sixteen
//! verbs; no header amendment was needed):
//!
//! 1. ONE AMBIENT KERNEL PER PROCESS. The contract carries no handle
//!    parameter and no create/destroy verbs (the wrapper's RAII note
//!    points a D83 tail-add as where ownership would grow), so the face
//!    binds ONE process-global kernel built by `Kernel::new()` — the
//!    standard in-process posture: real-clock domain + `DlopenExecutor`.
//!    Everything the verbs need is process-global by construction: the
//!    kernel, the plugin-id string table, the bus subscription table.
//! 2. THE BIG LOCK. `Kernel` is `!Send + !Sync` (measured by compile
//!    probe: the `Box<dyn Executor>` field, a trait without a `Send`
//!    bound). The ambient wraps it in a newtype carrying a minimal
//!    `unsafe impl Send` — justified because it is ONLY ever built here
//!    from `Kernel::new()`, whose executor is `DlopenExecutor`: a
//!    stateless unit, every other field `Arc`/`Mutex`-guarded. All verbs
//!    serialize through one outer Mutex (year-1 single-writer
//!    discipline, the posture the introspect trio documents). shortcut: a
//!    plugin calling back into these verbs from inside its `start`/`stop`
//!    would self-deadlock — the D6 sync boundary forbids host callbacks
//!    there; revisit with the M2 dial-verb tail-add.
//! 3. OWNERSHIP (the header's `modukit_install` comment): `out_plugin_id`
//!    rides a leaked process-lifetime string table — the bytes are
//!    kernel-owned and outlive "valid until uninstall" as a superset; the
//!    caller copies and never frees. Re-installing the same id returns
//!    the same stable pointer.
//! 4. ERROR CHANNEL: 0 (or the byte count) on success, a NEGATED D51 slot
//!    on failure. Names come from the existing `KernelError::code()`
//!    mapping, numbers from the header enum transcribed verbatim
//!    (`embed_codec::d51_number` — nothing re-derived). The header pins
//!    two overrides where kernel-side prose disagrees: deferred-QoS
//!    subscribe answers DEFERRAL_DECLINED(26) (not `BusError::code()`'s
//!    CONTRACT_VIOLATION) and the Real-domain dial answers
//!    CLOCK_DIAL_REFUSED(27), never a silent no-op. `codec::guard()` is
//!    the panic wall: a Rust panic NEVER unwinds into C (the repo's
//!    first C symbol with Rust behind it — no catch_unwind precedent
//!    existed to copy, so the discipline lands here; a caught panic
//!    answers CONTRACT_VIOLATION(10)).
//! 5. OPAQUE CODECS (the header fixes shapes only; the wrapper + demo are
//!    the pinning consumers — documented in `embed_codec.rs`): snapshot =
//!    UTF-8 TAB-separated lines, section-ordered (plugin/service/topic/
//!    edge, each sorted, then one counters line); events = little-endian
//!    self-delimiting records mirroring the `modukit_event` struct
//!    field-for-field. NULL buffer = size query through `*out_len` on
//!    every byte-out verb.
//! 6. NO VTABLE EXPORT: the header declares the `modukit_host_embed_v1`
//!    struct as bookkeeping but ships no table-accessor verb, and the
//!    wrapper golden calls the free functions only. Inventing an accessor
//!    would break the parity gate (the law), so no table symbol ships.
//! 7. SERVICE-LOOKUP LIMIT (honest gap, reported not hacked): the
//!    registry's public door is the type-keyed `acquire::<S>()` (top-
//!    ranked of its type); there is no by-id acquire. `service_call`
//!    composes `acquire::<NativeService>()` and matches the canonical id,
//!    so with more than one native provider a non-top-ranked id answers
//!    NOT_FOUND. A by-id accessor is an additive registry change for its
//!    own ruling.

use std::collections::HashMap;
use std::ffi::{CStr, CString, c_char};
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{LazyLock, Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::Duration;

use crate::abi::NativeService;
use crate::bus::{BusError, Receiver as BusReceiver};
use crate::clock::ClockDomain;
use crate::embed_abi::EMBED_ABI_VERSION;
use crate::embed_codec::{self as codec, bytes_arg, copy_out, cstr_arg, queue_out_len};
use crate::error::KernelError;
use crate::identity::{PluginId, TopicId};
use crate::introspect::Snapshot;
use crate::kernel::Kernel;
use crate::manifest::Manifest;

/// The ambient kernel newtype — the wrapper exists only because
/// `Box<dyn Executor>` makes `Kernel` `!Send` (design decision 2).
struct Ambient(Kernel);

// SAFETY: `Ambient` is constructed exactly once, here, exclusively from
// `Kernel::new()` — whose executor is `DlopenExecutor`, a stateless unit
// type. Every other `Kernel` field is `Send` by construction (Arc/Mutex
// guarded), and the outer Mutex serializes all access, so no two threads
// ever hold the kernel at once.
unsafe impl Send for Ambient {}

static KERNEL: OnceLock<Mutex<Ambient>> = OnceLock::new();

fn ambient() -> MutexGuard<'static, Ambient> {
    KERNEL
        .get_or_init(|| Mutex::new(Ambient(Kernel::new())))
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// Kernel-owned static string storage behind `out_plugin_id` (decision 3).
static PLUGIN_IDS: LazyLock<Mutex<HashMap<String, &'static CStr>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn registered_id(id: &str) -> Option<&'static CStr> {
    let mut table = PLUGIN_IDS.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(seen) = table.get(id) {
        return Some(seen);
    }
    let leaked: &'static CString = Box::leak(Box::new(CString::new(id.replace('\0', "")).ok()?));
    table.insert(id.to_string(), leaked.as_c_str());
    Some(leaked.as_c_str())
}

struct BusQueue {
    rx: BusReceiver,
    /// Oldest buffered value popped by a size query (NULL `out_value`)
    /// and restored here — a probe NEVER consumes; the next copy-out
    /// sees the same value first.
    pending: Option<Vec<u8>>,
}

static BUS_QUEUES: LazyLock<Mutex<HashMap<i32, BusQueue>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static NEXT_QUEUE: AtomicI32 = AtomicI32::new(1);

/// Shared plumbing of resolve/start/stop/uninstall: id borrow + big lock +
/// kernel door + negated-D51 channel.
fn lifecycle_move(
    id: *const c_char,
    door: impl FnOnce(&Kernel, &PluginId) -> Result<(), KernelError>,
) -> i32 {
    let Some(s) = cstr_arg(id) else {
        return codec::NEG_SCHEMA_INVALID;
    };
    let k = ambient();
    match door(&k.0, &PluginId::new(s.to_string())) {
        Ok(()) => 0,
        Err(err) => codec::neg_kernel(&err),
    }
}

#[unsafe(no_mangle)]
/// Embed handshake probe: answers `MODUKIT_EMBED_ABI_VERSION`.
pub extern "C" fn modukit_embed_abi_version() -> i32 {
    i32::try_from(EMBED_ABI_VERSION).unwrap_or(1)
}

#[unsafe(no_mangle)]
/// install (UNINSTALLED -> INSTALLED): manifest-v0 TOML in, canonical id
/// out. Parse failure and missing artifact both leave MANIFEST_INVALID(1)
/// on the channel; the swap door answers its D80 deferral.
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: the contract pins these signatures (C declarations, no Rust unsafe); every deref is null-checked.
pub extern "C" fn modukit_install(
    manifest_toml: *const c_char,
    out_plugin_id: *mut *const c_char,
) -> i32 {
    codec::guard(|| {
        let (Some(toml), false) = (cstr_arg(manifest_toml), out_plugin_id.is_null()) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        let manifest = match Manifest::from_toml_str(toml) {
            Ok(m) => m,
            Err(err) => return -codec::d51_number(err.code()),
        };
        let k = ambient();
        let id = match k.0.install(manifest) {
            Ok(id) => id,
            Err(err) => return codec::neg_kernel(&err),
        };
        match registered_id(id.as_str()) {
            Some(c) => {
                // SAFETY: `out_plugin_id` checked non-null above.
                unsafe { *out_plugin_id = c.as_ptr() };
                0
            }
            None => codec::NEG_SCHEMA_INVALID,
        }
    })
}

#[unsafe(no_mangle)]
/// uninstall (any -> UNINSTALLED; drops the plugin's registrations).
pub extern "C" fn modukit_uninstall(id: *const c_char) -> i32 {
    codec::guard(|| lifecycle_move(id, |k, pid| k.uninstall(pid)))
}

#[unsafe(no_mangle)]
/// resolve (INSTALLED -> RESOLVING -> STARTING|FAILED): contract/link/
/// placement guards, then the dl handshake; failures park FAILED.
pub extern "C" fn modukit_resolve(id: *const c_char) -> i32 {
    codec::guard(|| lifecycle_move(id, |k, pid| k.resolve(pid)))
}

#[unsafe(no_mangle)]
/// start (STARTING -> ACTIVE|FAILED): the plugin's own `start()`.
pub extern "C" fn modukit_start(id: *const c_char) -> i32 {
    codec::guard(|| lifecycle_move(id, |k, pid| k.start(pid)))
}

#[unsafe(no_mangle)]
/// stop (ACTIVE -> STOPPING -> RESOLVED|FAILED): cooperative drain.
pub extern "C" fn modukit_stop(id: *const c_char) -> i32 {
    codec::guard(|| lifecycle_move(id, |k, pid| k.stop(pid)))
}

#[unsafe(no_mangle)]
/// state: answers one `modukit_plugin_state` number through `out_state`.
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: the contract pins these signatures (C declarations, no Rust unsafe); every deref is null-checked.
pub extern "C" fn modukit_state(id: *const c_char, out_state: *mut i32) -> i32 {
    codec::guard(|| {
        if out_state.is_null() {
            return codec::NEG_SCHEMA_INVALID;
        }
        let Some(s) = cstr_arg(id) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        let k = ambient();
        match k.0.state(&PluginId::new(s.to_string())) {
            Ok(state) => {
                // SAFETY: `out_state` checked non-null above.
                unsafe { *out_state = codec::state_number(state) };
                0
            }
            Err(err) => codec::neg_kernel(&err),
        }
    })
}

#[unsafe(no_mangle)]
/// The synchronous message-shaped call door: bytes in, bytes out; answers
/// the response byte count or a negative D51 code. The cap is
/// fail-rather-than-truncate; a plugin-side failure surfaces as
/// SERVICE_CALL_FAILED(25) — the D27 two-code-space rule keeps the
/// plugin's own int out of the kernel channel (it lives in the prose).
pub extern "C" fn modukit_service_call(
    service_id: *const c_char,
    method: *const c_char,
    input: *const u8,
    input_len: usize,
    response: *mut u8,
    response_cap: usize,
) -> i32 {
    codec::guard(|| {
        let (Some(sid), Some(method), Some(bytes)) = (
            cstr_arg(service_id),
            cstr_arg(method),
            bytes_arg(input, input_len),
        ) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        if response.is_null() {
            return codec::NEG_SCHEMA_INVALID;
        }
        let k = ambient();
        let registry = k.0.registry();
        // Design decision 7: type-keyed acquire + canonical-id match.
        let Some(svc) = registry.acquire::<NativeService>() else {
            return codec::NEG_NOT_FOUND;
        };
        if svc.service_id().as_str() != sid {
            let _ = registry.release(&svc);
            return codec::NEG_NOT_FOUND;
        }
        let answer = match svc.invoke(method, bytes) {
            // This door has no out_len channel: copy_out with a NULL size
            // slot answers the byte count / QUOTA_EXCEEDED (decision 4).
            Ok(reply) => copy_out(&reply, response, response_cap, std::ptr::null_mut()),
            Err(err) => codec::neg_kernel(&err),
        };
        if registry.release(&svc).is_err() {
            return codec::NEG_KERNEL_BUG;
        }
        answer
    })
}

#[unsafe(no_mangle)]
/// bus publish: broadcast, never blocks the caller (a slow subscriber
/// sheds its own oldest value). `qos` is the header's `modukit_qos` int;
/// ungrammatical topic ids are refused at this parse door (M7 validator).
pub extern "C" fn modukit_bus_publish(
    topic: *const c_char,
    qos: i32,
    value: *const u8,
    value_len: usize,
) -> i32 {
    codec::guard(|| {
        let (Some(name), Some(q), Some(bytes)) = (
            cstr_arg(topic),
            codec::qos_from_i32(qos),
            bytes_arg(value, value_len),
        ) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        let Ok(tid) = TopicId::try_new(name.to_string()) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        ambient().0.bus().publish(&tid, q, bytes.to_vec());
        0
    })
}

#[unsafe(no_mangle)]
/// bus subscribe: topic-scoped subscription, answers the kernel-owned
/// queue id. A deferred QoS (KEEP_ALL/BOUNDED) refuses
/// DEFERRAL_DECLINED(26) — the header's pinned override of
/// `BusError::code()`.
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: the contract pins these signatures (C declarations, no Rust unsafe); every deref is null-checked.
pub extern "C" fn modukit_bus_subscribe(topic: *const c_char, out_queue_id: *mut i32) -> i32 {
    codec::guard(|| {
        let (Some(name), false) = (cstr_arg(topic), out_queue_id.is_null()) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        let Ok(tid) = TopicId::try_new(name.to_string()) else {
            return codec::NEG_SCHEMA_INVALID;
        };
        let rx = match ambient().0.bus().subscribe(&tid) {
            Ok(rx) => rx,
            Err(BusError::Deferred { .. }) => return codec::NEG_DEFERRAL_DECLINED,
        };
        let id = NEXT_QUEUE.fetch_add(1, Ordering::Relaxed);
        if id <= 0 {
            return codec::NEG_QUOTA_EXCEEDED;
        }
        BUS_QUEUES
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(id, BusQueue { rx, pending: None });
        // SAFETY: `out_queue_id` checked non-null above.
        unsafe { *out_queue_id = id };
        id
    })
}

#[unsafe(no_mangle)]
/// bus receive: oldest buffered value into caller memory with the
/// header's cap discipline. NULL `out_value` is the size query and does
/// NOT consume; an empty queue answers 0 with `*out_len = 0`; a bad
/// queue id is negative.
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: the contract pins these signatures (C declarations, no Rust unsafe); every deref is null-checked.
pub extern "C" fn modukit_bus_recv(
    queue_id: i32,
    out_value: *mut u8,
    value_cap: usize,
    out_len: *mut usize,
) -> i32 {
    codec::guard(|| {
        if out_len.is_null() {
            return codec::NEG_SCHEMA_INVALID;
        }
        let mut table = BUS_QUEUES.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(queue) = table.get_mut(&queue_id) else {
            return codec::NEG_NOT_FOUND;
        };
        let value = match queue.pending.take() {
            Some(v) => v,
            None => match queue.rx.try_recv() {
                Some(v) => v,
                None => {
                    queue_out_len(out_len, 0);
                    return 0;
                }
            },
        };
        let len = value.len();
        let Some(n) = i32::try_from(len).ok() else {
            queue.pending = Some(value);
            return codec::NEG_QUOTA_EXCEEDED;
        };
        if out_value.is_null() || len > value_cap {
            // Probe or too-small cap: the value goes back untouched.
            queue.pending = Some(value);
            queue_out_len(out_len, len);
            return if out_value.is_null() {
                n
            } else {
                codec::NEG_QUOTA_EXCEEDED
            };
        }
        // SAFETY: non-null `out_value` with cap >= len (branch above).
        unsafe { std::slice::from_raw_parts_mut(out_value, len) }.copy_from_slice(&value);
        queue_out_len(out_len, len);
        n
    })
}

#[unsafe(no_mangle)]
/// bus unsubscribe: releases the queue id (recv answers NOT_FOUND after).
pub extern "C" fn modukit_bus_unsubscribe(queue_id: i32) -> i32 {
    codec::guard(|| {
        let removed = BUS_QUEUES
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&queue_id);
        if removed.is_some() {
            0
        } else {
            codec::NEG_NOT_FOUND
        }
    })
}

#[unsafe(no_mangle)]
/// clock now: the unified-clock stamp in nanoseconds (D20 water source).
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: the contract pins these signatures (C declarations, no Rust unsafe); every deref is null-checked.
pub extern "C" fn modukit_clock_now(out_now_ns: *mut i64) -> i32 {
    codec::guard(|| {
        if out_now_ns.is_null() {
            return codec::NEG_SCHEMA_INVALID;
        }
        let raw = ambient().0.clock().now().raw();
        // SAFETY: `out_now_ns` checked non-null above.
        unsafe { *out_now_ns = i64::try_from(raw).unwrap_or(i64::MAX) };
        0
    })
}

#[unsafe(no_mangle)]
/// clock advance: the dial exists on Sim/Step domains ONLY. The ambient
/// kernel boots the Real posture (decision 1), so this face answers
/// CLOCK_DIAL_REFUSED(27) — the header's "never a silent no-op" pin. A
/// dial-carrying ambient domain is a D83 tail-add for a later lane.
pub extern "C" fn modukit_clock_advance(delta_ns: i64) -> i32 {
    codec::guard(|| {
        if delta_ns < 0 {
            return codec::NEG_SCHEMA_INVALID;
        }
        let k = ambient();
        if k.0.clock().domain() == ClockDomain::Real {
            return codec::NEG_CLOCK_DIAL_REFUSED;
        }
        k.0.advance_clock(Duration::from_nanos(delta_ns as u64));
        0
    })
}

#[unsafe(no_mangle)]
/// introspect snapshot: the `org.modukit.snapshot` object-table cut in
/// the documented line codec (decision 5). NULL buffer = size query.
pub extern "C" fn modukit_introspect_snapshot(
    out_buf: *mut u8,
    buf_cap: usize,
    out_len: *mut usize,
) -> i32 {
    codec::guard(|| {
        if out_len.is_null() {
            return codec::NEG_SCHEMA_INVALID;
        }
        let k = ambient();
        let Some(snap) = k.0.registry().acquire::<Snapshot>() else {
            return codec::NEG_KERNEL_BUG;
        };
        let bytes = codec::render_snapshot(&snap.all());
        if k.0.registry().release(&snap).is_err() {
            return codec::NEG_KERNEL_BUG;
        }
        copy_out(&bytes, out_buf, buf_cap, out_len)
    })
}

#[unsafe(no_mangle)]
/// introspect events: resumable merged-ring read — records with
/// `seq > since_seq`, whole records only up to `buf_cap` (the cursor
/// advances past exactly what was copied); negative code on a cursor
/// pointing past the ring's end. NULL buffer = size query (cursor held).
#[allow(clippy::not_unsafe_ptr_arg_deref)] // FFI: the contract pins these signatures (C declarations, no Rust unsafe); every deref is null-checked.
pub extern "C" fn modukit_introspect_events(
    since_seq: u64,
    out_buf: *mut u8,
    buf_cap: usize,
    out_len: *mut usize,
    out_seq: *mut u64,
) -> i32 {
    codec::guard(|| {
        if out_len.is_null() || out_seq.is_null() {
            return codec::NEG_SCHEMA_INVALID;
        }
        let k = ambient();
        let obs = k.0.observatory();
        if since_seq > obs.last_seq() {
            return codec::NEG_EPOCH_EXPIRED;
        }
        let tail = obs.tail(since_seq);
        let (bytes, last_seq) =
            codec::encode_events(&tail.events, buf_cap, since_seq, out_buf.is_null());
        // SAFETY: both out pointers checked non-null above.
        unsafe { *out_seq = last_seq };
        copy_out(&bytes, out_buf, buf_cap, out_len)
    })
}

#[cfg(test)]
#[path = "embed_tests.rs"]
mod tests;
