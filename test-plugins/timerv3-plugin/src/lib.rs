//! timerv3-plugin — Wave-2 T1 v3 fixture (busv2-plugin pattern, D99). The
//! descriptor is the frozen `PluginAbiV1` shape declaring `ABI_VERSION_V3`;
//! the kernel then hands a `HostAbiV3` table at `start` (the v2 prefix
//! byte-identical in front plus the two timer slots at the tail).
//!
//! The whole proof runs INSIDE `start` (01 §3 rule 2: the table is borrowed
//! for the call): the v2-prefix mailbox answers on the v3 table (the
//! prefix-read law of the door), `clock_now` reads the kernel's one water
//! source (D20) twice identically on the step clock, `timer_schedule`
//! stamps one job a second ahead of the dialed now, and the three refusal
//! cells answer the pinned negated D51 shape (SCHEMA_INVALID(11)) for a
//! NULL out-param or NULL label. Any failed step returns a distinct
//! negative code so the driving test names it. The DUE bell itself rings
//! after `start` returns - the kernel posts it on
//! `org.modukit.timer.<plugin-id>` (D99 R4-A); `tests/abi_v3_door.rs`
//! reads it there.

use std::ffi::{CStr, c_char};

use modukit_core::abi::{ABI_VERSION_V3, HostAbiV1, HostAbiV2, HostAbiV3, PluginAbiV1};

/// The fixture's own v2-prefix mailbox topic (proof the prefix reads).
const TOPIC: &CStr = c"com.example.modukit.timerv3";
const SERVICE_ID: &CStr = c"com.example.modukit.timerv3.echo";
/// The message-shaped job label (D6) the kernel echoes back in the bell.
const JOB_LABEL: &CStr = c"tick-v3";

/// QoS ints ride the embed `modukit_qos` vocabulary (D96 ruling 5).
const QOS_LATEST: i32 = 0;
/// The pinned D51 number the refusal cells answer (modukit_embed.h).
const NEG_SCHEMA_INVALID: i32 = -11;
/// One second in nanos - the deadline offset from the dialed stamp.
const ONE_SECOND_NS: u64 = 1_000_000_000;

fn cstr_len(ptr: *const c_char) -> usize {
    let mut len = 0usize;
    // SAFETY: callers pass a readable NUL-terminated C string.
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    len
}

/// The one service's message-shaped echo (v1 grammar, unchanged) - proof
/// the HostAbiV3 table's v1 prefix still registers and answers calls.
extern "C" fn echo_call(
    method: *const c_char,
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_cap: usize,
) -> i32 {
    if method.is_null() || output.is_null() || (input_len > 0 && input.is_null()) {
        return -1;
    }
    let len = cstr_len(method);
    // SAFETY: cstr_len walked the NUL-terminated string; the slice covers
    // only the bytes before the terminator.
    let name = unsafe { std::slice::from_raw_parts(method.cast::<u8>(), len) };
    if name != b"echo" {
        return -1;
    }
    if input_len > output_cap {
        return -2;
    }
    if input_len > 0 {
        // SAFETY: input has input_len readable bytes, output has capacity;
        // the buffers were created by the host, disjoint from input.
        unsafe { std::ptr::copy_nonoverlapping(input, output, input_len) };
    }
    input_len as i32
}

/// The clock-door walk; `Err(code)` names the step that disagreed.
fn timer_walk(h2: &HostAbiV2, h3: &HostAbiV3) -> Result<(), i32> {
    // 1. The v2 prefix is live on the v3 table: subscribe -> publish ->
    //    self-echo recv -> unsubscribe, all through the prefix slots.
    let qid = h2.bus_subscribe(TOPIC).map_err(|_| -101)?;
    if h2.bus_publish(TOPIC, QOS_LATEST, b"v3-prefix-echo") != 0 {
        return Err(-102);
    }
    let mut buf = [0u8; 32];
    let written = h2.bus_recv(qid, &mut buf).map_err(|_| -103)?;
    if written != 14 || &buf[..written] != b"v3-prefix-echo" {
        return Err(-103);
    }
    if h2.bus_unsubscribe(qid) != 0 {
        return Err(-104);
    }
    // 2. clock_now reads the kernel's one clock (D20); on the step domain
    //    the answer is stable between dials - the door test owns the dial.
    let now = h3.clock_now().map_err(|_| -105)?;
    if h3.clock_now() != Ok(now) {
        return Err(-106);
    }
    let deadline = u64::try_from(now)
        .map_err(|_| -105)?
        .saturating_add(ONE_SECOND_NS);
    // 3. Schedule the one job the census and the bell ride on. The token is
    //    the kernel clock's monotonic counter; the door test decodes the
    //    exact value from the bell payload, this cell proves it came back.
    let token = h3.timer_schedule(deadline, JOB_LABEL).map_err(|_| -107)?;
    if token == 0 {
        return Err(-108);
    }
    // 4. Refusal cells (raw slot calls, the contract shapes unchanged): a
    //    NULL out-param and a NULL label answer negated SCHEMA_INVALID and
    //    schedule nothing - no cancel verb exists (D99 §3), so the door
    //    must never half-schedule on bad input.
    let null_now = unsafe { (h3.clock_now)(h3.user, std::ptr::null_mut()) };
    if null_now != NEG_SCHEMA_INVALID {
        return Err(-109);
    }
    let mut throwaway = 0u64;
    let null_label = unsafe {
        (h3.timer_schedule)(h3.user, ONE_SECOND_NS, std::ptr::null(), &raw mut throwaway)
    };
    if null_label != NEG_SCHEMA_INVALID {
        return Err(-110);
    }
    let null_out = unsafe {
        (h3.timer_schedule)(
            h3.user,
            ONE_SECOND_NS,
            JOB_LABEL.as_ptr(),
            std::ptr::null_mut(),
        )
    };
    if null_out != NEG_SCHEMA_INVALID || throwaway != 0 {
        return Err(-111);
    }
    Ok(())
}

/// Activation: take the declared-v3 host table, register one echo service
/// through the v1 prefix slot, then run the clock-door walk (all assertions
/// land on distinct negative codes; 0 = every clock-door law held).
extern "C" fn demo_start(host: *const HostAbiV1) -> i32 {
    if host.is_null() {
        return -1;
    }
    // SAFETY: a descriptor declaring major 3 is handed a HostAbiV3 whose
    // v2 prefix is byte-identical (offsets pinned by tests/abi_v3_door.rs),
    // valid for this call only (01 §3 rule 2).
    let h1 = unsafe { &*host };
    if h1.size < HostAbiV3::expected_size() {
        return -2;
    }
    // SAFETY: as above - the widened table is the kernel's own for this
    // call; the v2 and v3 views share its prefix and never alias-write.
    let h2 = unsafe { &*(host as *const HostAbiV2) };
    let h3 = unsafe { &*(host as *const HostAbiV3) };
    let rid = unsafe { (h3.register_service)(h3.user, SERVICE_ID.as_ptr(), echo_call) };
    if rid < 0 {
        return -3;
    }
    match timer_walk(h2, h3) {
        Ok(()) => 0,
        Err(code) => code,
    }
}

extern "C" fn demo_stop() -> i32 {
    0
}

static DESCRIPTOR: PluginAbiV1 = PluginAbiV1 {
    abi_version: ABI_VERSION_V3,
    size: PluginAbiV1::expected_size(),
    name: c"timerv3-plugin".as_ptr(),
    id: TOPIC.as_ptr(),
    version: c"0.1.0".as_ptr(),
    start: Some(demo_start),
    stop: Some(demo_stop),
};

/// The single exported entry symbol (04 §3).
#[unsafe(no_mangle)]
pub extern "C" fn modukit_plugin_init() -> *const PluginAbiV1 {
    &raw const DESCRIPTOR
}
