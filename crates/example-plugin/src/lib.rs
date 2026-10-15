//! example-plugin — positive dlopen fixture, template standing per D23.
//!
//! The kernel-facing surface is exactly the ABI v1 handshake plus ONE
//! service registration (modules/04 §3 symbol discipline; the template
//! standing keeps the fixture to nothing more). It exports the single entry
//! symbol `modukit_plugin_init`; `start` registers one message-shaped echo
//! service through the host table; `stop` is a cooperative no-op.

use std::ffi::{CStr, c_char};

use modukit_core::abi::{ABI_VERSION, HostAbiV1, PluginAbiV1};

/// Canonical service id (D9: v0 hand-writes the const; codegen owns it at
/// M4). `c""` literal: NUL-terminated by construction for the C ABI.
const SERVICE_ID: &CStr = c"com.example.modukit.demo";

fn cstr_len(ptr: *const c_char) -> usize {
    let mut len = 0usize;
    // SAFETY: callers pass a readable NUL-terminated C string.
    while unsafe { *ptr.add(len) } != 0 {
        len += 1;
    }
    len
}

/// The one service's message-shaped call (04 §2: method name + owned
/// bytes; returns bytes written, negative = plugin-side error code, D27
/// author code space). `echo` answers; anything else is -1.
extern "C" fn demo_call(
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

/// Activation: register exactly one service through the host table, keep
/// nothing (01 §3 rule 2: no ambient authority beyond this call).
extern "C" fn demo_start(host: *const HostAbiV1) -> i32 {
    if host.is_null() {
        return -1;
    }
    // SAFETY: the kernel passes a live HostAbiV1 table, valid for this call.
    let h = unsafe { &*host };
    if h.size < HostAbiV1::expected_size() {
        return -2;
    }
    // SAFETY: as above; `user` is the kernel's own context, handed back
    // unchanged.
    let rid = unsafe { (h.register_service)(h.user, SERVICE_ID.as_ptr(), demo_call) };
    if rid < 0 {
        return -3;
    }
    0
}

extern "C" fn demo_stop() -> i32 {
    0
}

static DESCRIPTOR: PluginAbiV1 = PluginAbiV1 {
    abi_version: ABI_VERSION,
    size: PluginAbiV1::expected_size(),
    name: c"example-plugin".as_ptr(),
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
