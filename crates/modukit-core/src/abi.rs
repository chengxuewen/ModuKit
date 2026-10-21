//! The `dl` ABI v1 handshake surface (modules/04 §3), shared by the kernel
//! and plugin fixtures as one source of layout truth.
//!
//! Contract honored here: one exported entry symbol (`modukit_plugin_init`)
//! returning `*const PluginAbiV1`; version integer + struct size are checked
//! before any call; unknown ABI -> `FAILED` at `RESOLVING`, never a partial
//! start. The service call shape is message-shaped (method name + owned
//! bytes, 04 §2 / 01 §3 rule 3) — full contract codegen replaces these raw
//! fn-pointer shapes at M4; nothing beyond the handshake and one service
//! registration is frozen here (D23 fixture standing, D67 export freeze is
//! its own later trigger).

use std::ffi::{c_char, c_void};
use std::mem::size_of;

use crate::error::KernelError;
use crate::identity::ServiceId;

/// Host-side ABI major (04 §3: one integer compare is the whole negotiation).
pub const ABI_VERSION: u32 = 1;

/// Host-side ABI major v2 (D96 ruling 2): a plugin declaring this major is
/// handed the `HostAbiV2` table (the v1 prefix plus the four mailbox slots).
/// `ABI_VERSION` and every v1 line above stay byte-identical (D87 STABLE).
pub const ABI_VERSION_V2: u32 = 2;

/// The single exported entry symbol (04 §3 symbol discipline).
pub const ENTRY_SYMBOL: &[u8] = b"modukit_plugin_init\0";

/// `extern "C" fn() -> *const PluginAbiV1` — reads the descriptor, runs no
/// service logic.
pub type PluginInitFn = extern "C" fn() -> *const PluginAbiV1;

/// Activation: called once with the host context; non-zero = start failure
/// (03 §2 STARTING -> FAILED).
pub type PluginStartFn = unsafe extern "C" fn(host: *const HostAbiV1) -> i32;

/// Cooperative stop; must be idempotent-friendly (01 §3 rule 6).
pub type PluginStopFn = unsafe extern "C" fn() -> i32;

/// Message-shaped service call: method name + owned bytes in, bytes out.
/// Returns the number of bytes written to `output`, or a negative
/// plugin-side error code (D27 author code space stays the int channel).
pub type ServiceCallFn = unsafe extern "C" fn(
    method: *const c_char,
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_cap: usize,
) -> i32;

/// Host callback: register one service; returns the `RegistrationId` as a
/// monotonic `i64`, or negative on failure.
pub type RegisterServiceFn =
    unsafe extern "C" fn(user: *mut c_void, service_id: *const c_char, call: ServiceCallFn) -> i64;

/// The self-report descriptor (04 §3, D7 self-report tier). `size` is the
/// plugin's compiled `size_of` of this struct; the host checks
/// `abi_version` and `size` before calling anything.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PluginAbiV1 {
    pub abi_version: u32,
    pub size: u32,
    /// Human name (04 §3 shape).
    pub name: *const c_char,
    /// Canonical reverse-DNS id for the future cross-check (D7); not
    /// policed until M7.
    pub id: *const c_char,
    /// Plugin semver string (05 §1: independent of the ABI version).
    pub version: *const c_char,
    pub start: Option<PluginStartFn>,
    pub stop: Option<PluginStopFn>,
}

// SAFETY: the descriptor is read-only publication data; its C-string
// pointers target static bytes inside the owner's image, and the struct is
// only stored in `static` descriptors (fixture template standing, D23).
unsafe impl Sync for PluginAbiV1 {}

impl PluginAbiV1 {
    /// The exact compiled size a conforming plugin must report.
    #[must_use]
    pub const fn expected_size() -> u32 {
        size_of::<PluginAbiV1>() as u32
    }
}

/// The host table handed to `start` (CLAP `host_t` precedent). Valid only
/// for the duration of the call — plugins keep no ambient authority
/// (01 §3 rule 2).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HostAbiV1 {
    pub size: u32,
    /// Opaque kernel-side context, returned to the callbacks verbatim.
    pub user: *mut c_void,
    pub register_service: RegisterServiceFn,
}

impl HostAbiV1 {
    #[must_use]
    pub const fn expected_size() -> u32 {
        size_of::<HostAbiV1>() as u32
    }
}

/// A service provided over the ABI v1 call table (the `dl` local dispatch of
/// M4's generated dispatch enum, hand-shaped at v0 scale). Stored behind
/// `Arc<NativeService>` in the registry; consumers acquire it by type.
#[derive(Debug)]
pub struct NativeService {
    service_id: ServiceId,
    call: ServiceCallFn,
}

impl NativeService {
    #[must_use]
    pub const fn new(service_id: ServiceId, call: ServiceCallFn) -> Self {
        Self { service_id, call }
    }

    #[must_use]
    pub fn service_id(&self) -> &ServiceId {
        &self.service_id
    }

    /// Synchronous message-shaped call (D6): bytes in, owned bytes out.
    /// The output buffer is host-allocated; a plugin needing more than
    /// `cap` returns a negative code, which surfaces as an error (the
    /// cap is `max(input, 64) + 64` at v0 — real sizing is M4's job).
    ///
    /// # Errors
    ///
    /// `KernelError::ServiceCallFailed` for negative plugin codes and for
    /// a reported write larger than the buffer (the `-2` code).
    pub fn invoke(&self, method: &str, input: &[u8]) -> Result<Vec<u8>, KernelError> {
        let mut method_z = method.as_bytes().to_vec();
        method_z.push(0);
        let cap = input.len().max(64) + 64;
        let mut output = vec![0u8; cap];
        // SAFETY: method_z is NUL-terminated and outlives the call; input
        // pointer is only read for `input_len` bytes (valid even when the
        // slice is empty: the callee contract says do not read at len 0).
        let written = unsafe {
            (self.call)(
                method_z.as_ptr().cast::<c_char>(),
                input.as_ptr(),
                input.len(),
                output.as_mut_ptr(),
                cap,
            )
        };
        if written < 0 {
            return Err(KernelError::ServiceCallFailed { code: written });
        }
        let n =
            usize::try_from(written).map_err(|_| KernelError::ServiceCallFailed { code: -2 })?;
        if n > cap {
            return Err(KernelError::ServiceCallFailed { code: -2 });
        }
        output.truncate(n);
        Ok(output)
    }
}

/// Copy a NUL-terminated C string into a Rust `String`; `None` on null or
/// non-UTF-8 bytes (lossy not needed for fixtures).
pub(crate) fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: caller guarantees a readable NUL-terminated byte string.
    let bytes = unsafe { std::ffi::CStr::from_ptr(ptr) };
    bytes.to_str().ok().map(str::to_string)
}

// --- Stage-5 N1 v2 door (D96 ruling 2): append-only surface below. --------

/// Mailbox verb (D96 ruling 5, mirroring the embed face `modukit_bus_publish`
/// exactly): broadcast, never blocks; returns 0 or a negated D51 code.
pub type BusPublishFn = unsafe extern "C" fn(
    user: *mut c_void,
    topic: *const c_char,
    qos: i32,
    value: *const u8,
    value_len: usize,
) -> i32;

/// Mailbox verb (mirrors `modukit_bus_subscribe`): answers the kernel-owned
/// queue id (positive) or a negated D51 code.
pub type BusSubscribeFn =
    unsafe extern "C" fn(user: *mut c_void, topic: *const c_char, out_queue_id: *mut i32) -> i32;

/// Mailbox verb (mirrors `modukit_bus_recv`): oldest buffered value into
/// caller memory; a too-small cap reports the need through `out_len` and
/// answers negated QUOTA_EXCEEDED, never a truncation.
pub type BusRecvFn = unsafe extern "C" fn(
    user: *mut c_void,
    queue_id: i32,
    out_value: *mut u8,
    value_cap: usize,
    out_len: *mut usize,
) -> i32;

/// Mailbox verb (mirrors `modukit_bus_unsubscribe`): releases the queue id;
/// a later recv on it answers negated NOT_FOUND.
pub type BusUnsubscribeFn = unsafe extern "C" fn(user: *mut c_void, queue_id: i32) -> i32;

/// The host table handed to a plugin that declared `ABI_VERSION_V2`
/// (D96 ruling 2: a SECOND table, negotiated by declared major; `HostAbiV1`
/// stays frozen). The three v1 fields keep their exact order and types so
/// the table doubles as a valid `HostAbiV1` prefix (offsets pinned by the
/// `abi_v2_door` layout test); a v2 plugin reads the four bus slots after
/// checking `size >= HostAbiV2::expected_size()`. Same borrowed-for-the-
/// `start`-call authority law as v1 (01 §3 rule 2).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HostAbiV2 {
    pub size: u32,
    /// Opaque kernel-side context, returned to the callbacks verbatim.
    pub user: *mut c_void,
    pub register_service: RegisterServiceFn,
    pub bus_publish: BusPublishFn,
    pub bus_subscribe: BusSubscribeFn,
    pub bus_recv: BusRecvFn,
    pub bus_unsubscribe: BusUnsubscribeFn,
}

impl HostAbiV2 {
    /// The exact compiled size a conforming v2 host must publish in `size`.
    #[must_use]
    pub const fn expected_size() -> u32 {
        size_of::<HostAbiV2>() as u32
    }

    // Sugar for Rust plugins only (D63: sugar lives plugin-side, never a
    // contract direction) - thin safe views over the table slots.

    /// Publish bytes on a topic (`qos` = the embed `modukit_qos` int).
    pub fn bus_publish(&self, topic: &std::ffi::CStr, qos: i32, value: &[u8]) -> i32 {
        // SAFETY: slots are kernel-published non-null on a v2 table; topic is
        // NUL-terminated; a zero-len slice is never dereferenced (callee rule).
        unsafe { (self.bus_publish)(self.user, topic.as_ptr(), qos, value.as_ptr(), value.len()) }
    }

    /// Subscribe to a topic; answers the queue id or a negated D51 code.
    pub fn bus_subscribe(&self, topic: &std::ffi::CStr) -> Result<i32, i32> {
        let mut queue_id = 0i32;
        // SAFETY: as above; `queue_id` is a live local for the out-param.
        let status = unsafe { (self.bus_subscribe)(self.user, topic.as_ptr(), &raw mut queue_id) };
        if status >= 0 { Ok(status) } else { Err(status) }
    }

    /// Receive the oldest buffered value into `buf`; `Err((code, need))`
    /// when the cap is too small (negated QUOTA_EXCEEDED, value NOT
    /// consumed) - the embed-face need-report through `out_len`.
    pub fn bus_recv(&self, queue_id: i32, buf: &mut [u8]) -> Result<usize, (i32, usize)> {
        let mut len = 0usize;
        // SAFETY: as above; `buf`/`len` are live locals for the out-params.
        let status = unsafe {
            (self.bus_recv)(
                self.user,
                queue_id,
                buf.as_mut_ptr(),
                buf.len(),
                &raw mut len,
            )
        };
        if status < 0 {
            Err((status, len))
        } else {
            Ok(len)
        }
    }

    /// Release a queue id.
    pub fn bus_unsubscribe(&self, queue_id: i32) -> i32 {
        // SAFETY: slot is kernel-published non-null on a v2 table.
        unsafe { (self.bus_unsubscribe)(self.user, queue_id) }
    }
}
