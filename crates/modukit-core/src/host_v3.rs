//! The wave-2 T1 v3 clock door (D99 ruling R2: a THIRD host table, the
//! byte-identical v2 prefix plus the two timer slots at the tail).
//!
//! A plugin that declares `ABI_VERSION_V3` at the handshake is handed this
//! table at `start`. The v1/v2 lines and the v2 start path in `kernel.rs`
//! are untouched (D87/D96); this module is pure addition. The five
//! prefix slots reuse the v2 trampolines VERBATIM (same `user` context
//! shape — `host_v2::v3_context`), so the mailbox grammar, the registry
//! door and the D51 refusal channels are the landed ones, not a second
//! implementation (D96 ruling 5 mirror law). The two new slots mirror the
//! landed host-side shapes (D99 R3-A): `timer_schedule` mirrors
//! `Clock::timer` (absolute deadline, message label, `JobToken` out),
//! `clock_now` mirrors the embed face `modukit_clock_now` (out-param i64,
//! status int channel, negated D51 codes for bad input).
//!
//! The due bell is NOT rung here: the kernel posts fired plugin timers onto
//! the per-plugin bus topic in `wiring.rs::post_jobs` (the ONE due-funnel,
//! D99 R4-A), and releases the plugin's tokens at stop/uninstall (the
//! authority-law register, kernel-side — not an ABI verb, no cancel slot).
//!
//! Authority law is the v1 law: the table and its queue ids live only for
//! the duration of the plugin's `start` call (01 §3 rule 2); the REGISTER,
//! by contrast, is kernel state and outlives the call — that is what makes
//! a post-start firing addressable, and what the release census reclaims.

use std::collections::HashMap;
use std::ffi::{c_char, c_void};
use std::sync::{Mutex, PoisonError};

use crate::abi::{
    BusPublishFn, BusRecvFn, BusSubscribeFn, BusUnsubscribeFn, ClockNowFn, HostAbiV1, HostAbiV3,
    RegisterServiceFn, TimerScheduleFn,
};
use crate::bus::Bus;
use crate::clock::{Clock, Timestamp};
use crate::embed_codec as codec;
use crate::error::KernelError;
use crate::executor::LoadedPlugin;
use crate::host_v2::{
    V2Context, trampoline_bus_publish, trampoline_bus_recv, trampoline_bus_subscribe,
    trampoline_bus_unsubscribe, trampoline_register_service, v3_context,
};
use crate::identity::PluginId;
use crate::registry::{RegistrationId, Registry};

/// The v3 `timer_schedule` slot (D99 R3-A): schedule on the kernel's one
/// clock (D20) and record token -> plugin in the register (D99 R4-A), the
/// pair that later lets `post_jobs` ring the per-plugin bell and the
/// release reclaim the token. Bad input answers the negated D51 codes on
/// the same channel as every landed verb; there is no cancel verb (§3).
///
/// # Safety
/// `user` must be a live `&V2Context` from `start_v3` (kernel-owned, valid
/// for the whole `start` call); `job_label` a readable NUL string;
/// `out_token` null-checked inside.
pub(crate) unsafe extern "C" fn trampoline_timer_schedule(
    user: *mut c_void,
    deadline_ns: u64,
    job_label: *const c_char,
    out_token: *mut u64,
) -> i32 {
    // SAFETY: the kernel only ever passes a `&mut V2Context` (the v3 hand
    // built by `v3_context`) as `user` and keeps it alive across the
    // `start` call it originated from.
    let ctx = unsafe { &*(user.cast::<V2Context<'_>>()) };
    if out_token.is_null() {
        return codec::NEG_SCHEMA_INVALID;
    }
    let (Some(clock), Some(plugin_id), Some(register)) =
        (ctx.clock.clone(), ctx.plugin_id.clone(), ctx.plugin_timers)
    else {
        // A v3 slot on a non-v3 hand is a kernel bug, not a plugin error -
        // the door can never build this shape; refuse on the input slot.
        return codec::NEG_SCHEMA_INVALID;
    };
    let Some(label) = codec::cstr_arg(job_label) else {
        return codec::NEG_SCHEMA_INVALID;
    };
    let token = clock.timer(Timestamp::from(deadline_ns), label.to_string());
    register
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(token.value(), plugin_id);
    // SAFETY: `out_token` checked non-null above.
    unsafe { *out_token = token.value() };
    0
}

/// The v3 `clock_now` slot (D99 R1-B): the plugin's read of the same water
/// source the deadlines ride (D20). Mirrors `modukit_clock_now` exactly -
/// saturating i64 out-param, negated SCHEMA_INVALID on a null out.
///
/// # Safety
/// `user` as above; `out_now_ns` null-checked inside.
pub(crate) unsafe extern "C" fn trampoline_clock_now(
    user: *mut c_void,
    out_now_ns: *mut i64,
) -> i32 {
    // SAFETY: as for `trampoline_timer_schedule`.
    let ctx = unsafe { &*(user.cast::<V2Context<'_>>()) };
    if out_now_ns.is_null() {
        return codec::NEG_SCHEMA_INVALID;
    }
    let Some(clock) = ctx.clock.clone() else {
        return codec::NEG_SCHEMA_INVALID;
    };
    let raw = clock.now().raw();
    // SAFETY: `out_now_ns` checked non-null above.
    unsafe { *out_now_ns = i64::try_from(raw).unwrap_or(i64::MAX) };
    0
}

/// The v3 half of `Kernel::start`: build the `HostAbiV3` table over the
/// kernel's real `Bus`, `Clock` and per-plugin timer register, run the
/// plugin's `start`, and hand back its status plus whatever services
/// registered (the same bookkeeping `start_v2` does).
pub(crate) fn start_v3(
    handle: &mut dyn LoadedPlugin,
    registry: &Registry,
    bus: &Bus,
    clock: Clock,
    plugin_id: PluginId,
    plugin_timers: &Mutex<HashMap<u64, PluginId>>,
) -> (Result<(), KernelError>, Vec<RegistrationId>) {
    let mut ctx = v3_context(registry, bus, clock, plugin_id, plugin_timers);
    let table = HostAbiV3 {
        size: HostAbiV3::expected_size(),
        user: &raw mut ctx as *mut c_void,
        // The byte-identical v2 prefix: the SAME trampolines, the SAME
        // context shape - a v3 plugin's mailbox reads are v2 reads.
        register_service: trampoline_register_service as RegisterServiceFn,
        bus_publish: trampoline_bus_publish as BusPublishFn,
        bus_subscribe: trampoline_bus_subscribe as BusSubscribeFn,
        bus_recv: trampoline_bus_recv as BusRecvFn,
        bus_unsubscribe: trampoline_bus_unsubscribe as BusUnsubscribeFn,
        timer_schedule: trampoline_timer_schedule as TimerScheduleFn,
        clock_now: trampoline_clock_now as ClockNowFn,
    };
    // The v3 table's first seven fields are the byte-identical v2 prefix
    // (`repr(C)` order; pinned by the abi_v3_door layout test), so passing
    // it through the frozen `*const HostAbiV1` start type is the honest
    // widening - a v2- or v1-shaped reader sees its own table.
    let started = handle.start(std::ptr::from_ref(&table).cast::<HostAbiV1>());
    (started, ctx.registered)
}
