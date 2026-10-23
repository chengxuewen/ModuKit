//! busv2-plugin — Stage-5 N1 v2 fixture (template standing per D23, grown to
//! the D96 ruling-2 door). The descriptor is the frozen `PluginAbiV1` shape
//! declaring `ABI_VERSION_V2`; the kernel then hands a `HostAbiV2` table at
//! `start` (the v1 prefix plus the four mailbox slots).
//!
//! The whole proof runs INSIDE `start` (01 §3 rule 2: the table is borrowed
//! for the call): subscribe -> publish -> self-echo visible to the publisher
//! (ruling 6), recv into a too-small cap reports the need and does NOT
//! consume, late-join subscribe receives the retained snapshot, unsubscribe
//! cleans up, and the two refusal channels answer their pinned D51 numbers
//! (DEFERRAL_DECLINED(26), SCHEMA_INVALID(11)). Any failed step returns a
//! distinct negative code so the driving test names it.

use std::ffi::{CStr, c_char};

use modukit_core::abi::{ABI_VERSION_V2, HostAbiV1, HostAbiV2, PluginAbiV1};

const TOPIC: &CStr = c"com.example.modukit.busv2";
const DEFERRED_TOPIC: &CStr = c"com.example.modukit.busv2.deferred";
const BAD_TOPIC: &CStr = c"Not.A.Valid-Topic";
const SERVICE_ID: &CStr = c"com.example.modukit.busv2.echo";

/// QoS ints ride the embed `modukit_qos` vocabulary (D96 ruling 5).
const QOS_LATEST: i32 = 0;
const QOS_KEEP_ALL: i32 = 1;
/// The pinned D51 numbers this door answers (modukit_embed.h dictionary).
const NEG_SCHEMA_INVALID: i32 = -11;
const NEG_QUOTA_EXCEEDED: i32 = -19;
const NEG_NOT_FOUND: i32 = -14;
const NEG_DEFERRAL_DECLINED: i32 = -26;

fn cstr_len(ptr: *const c_char) -> usize {
    let mut len = 0usize;
    // SAFETY: callers pass a readable NUL-terminated C string.
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    len
}

/// The one service's message-shaped echo (v1 grammar, unchanged) - proof
/// the HostAbiV2 table's v1 prefix still registers and answers calls.
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

/// The mailbox walk; `Err(code)` names the step that disagreed.
fn mailbox_walk(h: &HostAbiV2) -> Result<(), i32> {
    // 1. subscribe first: the publisher must hear its own publish (ruling 6).
    let qid = h.bus_subscribe(TOPIC).map_err(|_| -101)?;
    if qid <= 0 {
        return Err(-101);
    }
    // 2. publish latest-value bytes -> 0 (broadcast, never blocks).
    if h.bus_publish(TOPIC, QOS_LATEST, b"self-echo") != 0 {
        return Err(-102);
    }
    // 3. self-echo is visible on the same call, no second publisher anywhere.
    let mut buf = [0u8; 32];
    let written = h.bus_recv(qid, &mut buf).map_err(|_| -103)?;
    if written != 9 || &buf[..written] != b"self-echo" {
        return Err(-103);
    }
    // 4. too-small cap: reports the need, answers QUOTA_EXCEEDED, does NOT
    //    consume - the embed-face recv discipline, verb for verb.
    if h.bus_publish(TOPIC, QOS_LATEST, b"quota-probe-1234") != 0 {
        return Err(-104);
    }
    let mut tiny = [0u8; 4];
    let (code, need) = h.bus_recv(qid, &mut tiny).err().ok_or(-105)?;
    if code != NEG_QUOTA_EXCEEDED || need != 16 {
        return Err(-105);
    }
    let written = h.bus_recv(qid, &mut buf).map_err(|_| -106)?;
    if written != 16 || &buf[..written] != b"quota-probe-1234" {
        return Err(-106);
    }
    // 5. retained-latest snapshot lands on a LATE subscribe (ruling 6 again,
    //    in-process column of the wall-transparency suite).
    if h.bus_publish(TOPIC, QOS_LATEST, b"late-snapshot") != 0 {
        return Err(-107);
    }
    let qid2 = h.bus_subscribe(TOPIC).map_err(|_| -108)?;
    let written = h.bus_recv(qid2, &mut buf).map_err(|_| -108)?;
    if written != 13 || &buf[..written] != b"late-snapshot" {
        return Err(-108);
    }
    // 6. unsubscribe cleans up: the id is gone (NOT_FOUND on later recv),
    //    a second unsubscribe is likewise NOT_FOUND; qid2 keeps its queue.
    if h.bus_unsubscribe(qid) != 0 {
        return Err(-109);
    }
    let (code, _) = h.bus_recv(qid, &mut buf).err().ok_or(-109)?;
    if code != NEG_NOT_FOUND {
        return Err(-109);
    }
    if h.bus_unsubscribe(qid) != NEG_NOT_FOUND || h.bus_unsubscribe(qid2) != 0 {
        return Err(-110);
    }
    // 7. refusal channels answer their pinned numbers: a deferred QoS
    //    publishes (recorded) but refuses the subscribe with 26; an
    //    ungrammatical topic and an out-of-vocabulary qos answer 11.
    if h.bus_publish(DEFERRED_TOPIC, QOS_KEEP_ALL, b"declared") != 0 {
        return Err(-111);
    }
    let code = h.bus_subscribe(DEFERRED_TOPIC).err().ok_or(-111)?;
    if code != NEG_DEFERRAL_DECLINED {
        return Err(-111);
    }
    if h.bus_publish(BAD_TOPIC, QOS_LATEST, b"x") != NEG_SCHEMA_INVALID
        || h.bus_subscribe(BAD_TOPIC) != Err(NEG_SCHEMA_INVALID)
        || h.bus_publish(TOPIC, 99, b"x") != NEG_SCHEMA_INVALID
    {
        return Err(-112);
    }
    // 8. an empty queue answers 0 with need 0 (no wait, no fake error).
    let qid3 = h
        .bus_subscribe(c"com.example.modukit.busv2.empty")
        .map_err(|_| -113)?;
    if h.bus_recv(qid3, &mut buf) != Ok(0) || h.bus_unsubscribe(qid3) != 0 {
        return Err(-113);
    }
    Ok(())
}

/// Activation: take the declared-v2 host table, register one echo service
/// through the v1 prefix slot, then run the mailbox walk (all assertions
/// land on distinct negative codes; 0 = every mailbox law held).
extern "C" fn demo_start(host: *const HostAbiV1) -> i32 {
    if host.is_null() {
        return -1;
    }
    // SAFETY: a descriptor declaring major 2 is handed a HostAbiV2 whose
    // v1 prefix is byte-identical (offsets pinned by tests/abi_v2_door.rs),
    // valid for this call only (01 §3 rule 2).
    let h1 = unsafe { &*host };
    if h1.size < HostAbiV2::expected_size() {
        return -2;
    }
    // SAFETY: as above - the widened table is the kernel's own for this call.
    let h2 = unsafe { &*(host as *const HostAbiV2) };
    let rid = unsafe { (h2.register_service)(h2.user, SERVICE_ID.as_ptr(), echo_call) };
    if rid < 0 {
        return -3;
    }
    match mailbox_walk(h2) {
        Ok(()) => 0,
        Err(code) => code,
    }
}

extern "C" fn demo_stop() -> i32 {
    0
}

static DESCRIPTOR: PluginAbiV1 = PluginAbiV1 {
    abi_version: ABI_VERSION_V2,
    size: PluginAbiV1::expected_size(),
    name: c"busv2-plugin".as_ptr(),
    id: SERVICE_ID.as_ptr(),
    version: c"0.1.0".as_ptr(),
    start: Some(demo_start),
    stop: Some(demo_stop),
};

/// The single exported entry symbol (04 §3).
#[unsafe(no_mangle)]
pub extern "C" fn modukit_plugin_init() -> *const PluginAbiV1 {
    &raw const DESCRIPTOR
}
