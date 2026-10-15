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
