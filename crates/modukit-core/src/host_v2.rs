//! The Stage-5 N1 v2 door (D96 ruling 2): wiring behind `abi::HostAbiV2`.
//!
//! A plugin that declares `ABI_VERSION_V2` at the handshake is handed this
//! second host table at `start` — the v1 prefix (`size`/`user`/
//! `register_service`) plus the four mailbox slots. `HostAbiV1` and the v1
//! start path in `kernel.rs` are untouched (D87); this module is pure
//! addition. The verbs mirror the embed face grammar EXACTLY (D96 ruling 5,
//! `embed.rs::modukit_bus_*`): same shapes, same negated-D51 codes, same
//! need-reporting `recv` — implemented over the SAME in-process `Bus`
//! (`kernel.bus()`), no second bus, no new data structures beyond the
//! per-call queue table the id-returning mailbox grammar requires.
//!
//! Authority law is the v1 law: the table and its queue ids live only for
//! the duration of the plugin's `start` call (01 §3 rule 2 — no ambient
//! authority past it). Cross-process plugins keep their v1-only door here;
//! the HELLO/wire side of v2 belongs to lane N2 (stage-5-plan §2).

use std::collections::HashMap;
use std::ffi::{c_char, c_void};

use crate::abi::{
    BusPublishFn, BusRecvFn, BusSubscribeFn, BusUnsubscribeFn, HostAbiV1, HostAbiV2, NativeService,
    RegisterServiceFn, ServiceCallFn,
};
use crate::bus::Bus;
use crate::bus::Receiver as BusReceiver;
use crate::embed_codec as codec;
use crate::error::KernelError;
use crate::executor::LoadedPlugin;
use crate::identity::TopicId;
use crate::registry::{Properties, PropertyValue, RegistrationId, Registry};

/// One mailbox: the bus receiver plus the embed-face `pending` slot (a probe
/// or a too-small recv NEVER consumes the value — same rule as `embed.rs`).
struct Mailbox {
    rx: BusReceiver,
    pending: Option<Vec<u8>>,
}

/// The `user` context behind every v2 slot. Lives on `start_v2`'s stack for
/// the duration of the plugin's `start` call (the v1 `ActivationSink` law).
struct V2Context<'a> {
    registry: &'a Registry,
    bus: &'a Bus,
    registered: Vec<RegistrationId>,
    /// Queue id -> mailbox. Ids are positive, issued from 1 (embed-face
    /// rule `id <= 0` answers QUOTA_EXCEEDED).
    queues: HashMap<i32, Mailbox>,
    next_queue: i32,
}

/// Borrow a caller NUL-string and parse it as a canonical topic id
/// (ungrammatical names are refused at this door — the embed `bus_publish`
/// rule, `TopicId::try_new`).
fn topic_arg(ptr: *const c_char) -> Option<TopicId> {
    let name = codec::cstr_arg(ptr)?;
    TopicId::try_new(name.to_string()).ok()
}

/// The v2 `register_service` slot: identical semantics to the v1 arm in
/// `kernel.rs` (`trampoline_register_service` — kept separate because that
/// one casts `user` to the kernel-private `ActivationSink`).
///
/// # Safety
/// `user` must be a live `&mut V2Context` from `start_v2` (kernel-owned,
/// valid for the whole `start` call); `service_id` a readable NUL string.
unsafe extern "C" fn trampoline_register_service(
    user: *mut c_void,
    service_id: *const c_char,
    call: ServiceCallFn,
) -> i64 {
    // SAFETY: the kernel only ever passes a `&mut V2Context` as `user` and
    // keeps it alive across the `start` call it originated from.
    let ctx = unsafe { &mut *(user.cast::<V2Context<'_>>()) };
    let Some(id_str) = crate::abi::cstr_to_string(service_id) else {
        return -1;
    };
    let service_id = crate::identity::ServiceId::new(id_str.clone());
    let mut props = Properties::new();
    props.insert("service.id", PropertyValue::Str(id_str));
    let instance = std::sync::Arc::new(NativeService::new(service_id, call));
    match ctx.registry.register(instance, props) {
        Ok(rid) => {
            ctx.registered.push(rid);
            rid.value()
        }
        Err(_) => -1,
    }
}

/// The v2 `bus_publish` slot (mirrors `modukit_bus_publish`): broadcast,
/// never blocks; 0 or negated SCHEMA_INVALID on bad input shape.
unsafe extern "C" fn trampoline_bus_publish(
    user: *mut c_void,
    topic: *const c_char,
    qos: i32,
    value: *const u8,
    value_len: usize,
) -> i32 {
    // SAFETY: `user` is the live `&mut V2Context` (slot contract, above).
    let ctx = unsafe { &mut *(user.cast::<V2Context<'_>>()) };
    let (Some(tid), Some(q), Some(bytes)) = (
        topic_arg(topic),
        codec::qos_from_i32(qos),
        codec::bytes_arg(value, value_len),
    ) else {
        return codec::NEG_SCHEMA_INVALID;
    };
    ctx.bus.publish(&tid, q, bytes.to_vec());
    0
}

/// The v2 `bus_subscribe` slot (mirrors `modukit_bus_subscribe`): answers
/// the queue id; a deferred QoS refuses DEFERRAL_DECLINED(26).
unsafe extern "C" fn trampoline_bus_subscribe(
    user: *mut c_void,
    topic: *const c_char,
    out_queue_id: *mut i32,
) -> i32 {
    // SAFETY: as above; `out_queue_id` is null-checked before any write.
    let ctx = unsafe { &mut *(user.cast::<V2Context<'_>>()) };
    if out_queue_id.is_null() {
        return codec::NEG_SCHEMA_INVALID;
    }
    let Some(tid) = topic_arg(topic) else {
        return codec::NEG_SCHEMA_INVALID;
    };
    let rx = match ctx.bus.subscribe(&tid) {
        Ok(rx) => rx,
        Err(crate::bus::BusError::Deferred { .. }) => return codec::NEG_DEFERRAL_DECLINED,
    };
    let id = ctx.next_queue;
    ctx.next_queue = ctx.next_queue.wrapping_add(1);
    if id <= 0 {
        return codec::NEG_QUOTA_EXCEEDED;
    }
    ctx.queues.insert(id, Mailbox { rx, pending: None });
    // SAFETY: `out_queue_id` checked non-null above.
    unsafe { *out_queue_id = id };
    id
}

/// The v2 `bus_recv` slot (mirrors `modukit_bus_recv`): empty queue answers
/// 0 with `*out_len = 0`; NULL `out_value` is a size query that does NOT
/// consume; too-small cap reports the need through `*out_len` and answers
/// negated QUOTA_EXCEEDED; an unknown queue id is negated NOT_FOUND.
unsafe extern "C" fn trampoline_bus_recv(
    user: *mut c_void,
    queue_id: i32,
    out_value: *mut u8,
    value_cap: usize,
    out_len: *mut usize,
) -> i32 {
    // SAFETY: as above; `out_len` is null-checked, `out_value` per-branch.
    let ctx = unsafe { &mut *(user.cast::<V2Context<'_>>()) };
    if out_len.is_null() {
        return codec::NEG_SCHEMA_INVALID;
    }
    let Some(queue) = ctx.queues.get_mut(&queue_id) else {
        return codec::NEG_NOT_FOUND;
    };
    let value = match queue.pending.take() {
        Some(v) => v,
        None => match queue.rx.try_recv() {
            Some(v) => v,
            None => {
                codec::queue_out_len(out_len, 0);
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
        codec::queue_out_len(out_len, len);
        return if out_value.is_null() {
            n
        } else {
            codec::NEG_QUOTA_EXCEEDED
        };
    }
    // SAFETY: non-null `out_value` with cap >= len (branch above).
    unsafe { std::slice::from_raw_parts_mut(out_value, len) }.copy_from_slice(&value);
    codec::queue_out_len(out_len, len);
    n
}

/// The v2 `bus_unsubscribe` slot (mirrors `modukit_bus_unsubscribe`):
/// releases the queue id (a later recv answers negated NOT_FOUND).
unsafe extern "C" fn trampoline_bus_unsubscribe(user: *mut c_void, queue_id: i32) -> i32 {
    // SAFETY: `user` is the live `&mut V2Context` (slot contract).
    let ctx = unsafe { &mut *(user.cast::<V2Context<'_>>()) };
    if ctx.queues.remove(&queue_id).is_some() {
        0
    } else {
        codec::NEG_NOT_FOUND
    }
}

/// The v2 half of `Kernel::start`: build the `HostAbiV2` table over the
/// kernel's real `Bus`, run the plugin's `start`, and hand back its status
/// plus whatever services registered (the same bookkeeping the v1 path
/// does with its `ActivationSink`).
pub(crate) fn start_v2(
    handle: &mut dyn LoadedPlugin,
    registry: &Registry,
    bus: &Bus,
) -> (Result<(), KernelError>, Vec<RegistrationId>) {
    let mut ctx = V2Context {
        registry,
        bus,
        registered: Vec::new(),
        queues: HashMap::new(),
        next_queue: 1,
    };
    let table = HostAbiV2 {
        size: HostAbiV2::expected_size(),
        user: &raw mut ctx as *mut c_void,
        register_service: trampoline_register_service as RegisterServiceFn,
        bus_publish: trampoline_bus_publish as BusPublishFn,
        bus_subscribe: trampoline_bus_subscribe as BusSubscribeFn,
        bus_recv: trampoline_bus_recv as BusRecvFn,
        bus_unsubscribe: trampoline_bus_unsubscribe as BusUnsubscribeFn,
    };
    // The v2 table's first three fields are the byte-identical v1 prefix
    // (`repr(C)` order; pinned by the abi_v2_door layout test), so passing
    // it through the frozen `*const HostAbiV1` start type is the honest
    // widening — a v1-shaped reader sees a valid HostAbiV1.
    let started = handle.start(std::ptr::from_ref(&table).cast::<HostAbiV1>());
    (started, ctx.registered)
}
