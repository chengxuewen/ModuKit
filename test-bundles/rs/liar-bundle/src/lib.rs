//! liar-bundle — negative fixture for the Stage-5 N1 v2 door: the same
//! ABI-lie class as bad-bundle (ABI v999) re-declared at major 99, an
//! in-between value the widened {1, 2} accept-set must still refuse. The
//! guard must park it FAILED with ABI_MISMATCH, never a partial start
//! (04 §3, D96: "the v2 negotiation must keep that negative RED-worthy").

use modukit_core::abi::{BundleAbiV1, HostAbiV1};

/// Unknown major between the accepted v1 and v2 doors.
const LIE_ABI_VERSION: u32 = 99;

extern "C" fn lie_start(_host: *const HostAbiV1) -> i32 {
    // Running at all is the bug the version guard exists to prevent.
    -113
}

extern "C" fn lie_stop() -> i32 {
    -113
}

static DESCRIPTOR: BundleAbiV1 = BundleAbiV1 {
    abi_version: LIE_ABI_VERSION,
    size: BundleAbiV1::expected_size(),
    name: c"liar-bundle".as_ptr(),
    id: c"com.example.modukit.liar".as_ptr(),
    version: c"0.1.0".as_ptr(),
    start: Some(lie_start),
    stop: Some(lie_stop),
};

/// The single exported entry symbol (04 §3).
#[unsafe(no_mangle)]
pub extern "C" fn modukit_bundle_init() -> *const BundleAbiV1 {
    &raw const DESCRIPTOR
}
