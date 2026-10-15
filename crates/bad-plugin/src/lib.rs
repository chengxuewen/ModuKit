//! bad-plugin — negative fixture: a deliberately mismatched handshake
//! (architecture.md §10: "bad plugin (negative fixture, ABI mismatch)").
//!
//! The descriptor reports ABI v999 with an otherwise well-formed struct, so
//! the DlopenExecutor version guard must reject it with a typed D27-shaped
//! error — NEVER a segfault and NEVER a silent accept. The `start`/`stop`
//! pointers are present (a null pointer would trip the *incomplete* guard
//! instead of the version guard, hiding what this fixture proves); they
//! carry poison-returning statuses so any partial-start attempt is visible.

use modukit_core::abi::{HostAbiV1, PluginAbiV1};

/// Unknown major version (04 §3: "unknown major version -> FAILED").
const BAD_ABI_VERSION: u32 = 999;

extern "C" fn lie_start(_host: *const HostAbiV1) -> i32 {
    // Running at all is the bug the version guard exists to prevent;
    // reporting failure keeps the story honest if it ever happens.
    -113
}

extern "C" fn lie_stop() -> i32 {
    -113
}

static DESCRIPTOR: PluginAbiV1 = PluginAbiV1 {
    abi_version: BAD_ABI_VERSION,
    size: PluginAbiV1::expected_size(),
    name: c"bad-plugin".as_ptr(),
    id: c"com.example.modukit.bad".as_ptr(),
    version: c"0.1.0".as_ptr(),
    start: Some(lie_start),
    stop: Some(lie_stop),
};

#[unsafe(no_mangle)]
pub extern "C" fn modukit_plugin_init() -> *const PluginAbiV1 {
    &raw const DESCRIPTOR
}
