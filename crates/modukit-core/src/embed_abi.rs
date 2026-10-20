//! The HostEmbedV1 Rust-side mirror (Stage-4 M1 lane #2, ruling C++-3).
//!
//! The contract original is the hand-authored C header
//! `contracts/modukit_embed.h` — this module is its Rust-side bookkeeping:
//! the verb list (the parity gate's second source) and the table version
//! constant. The parity gate (`scripts/check-embed-parity.sh`) diffs this
//! list against the header's verb declarations, so a header edit without
//! the mirror (or the reverse) goes red at the next gate run.
//!
//! NOT wired into kernel dispatch — M1 lane #3 (emitter) and the demo lane
//! consume this mirror; this lane establishes contract + mirror + gate only.
//! The two frozen v1 plugin tables (`abi::PluginAbiV1` / `abi::HostAbiV1`,
//! D87 STABLE) are untouched by this surface.

/// Embed table major version (`modukit_embed_abi_version()` returns this).
pub const EMBED_ABI_VERSION: u32 = 1;

/// The header's verbs, in declaration order (`contracts/modukit_embed.h`).
/// One entry per C function declared in the header's extern "C" block; the
/// parity gate greps the header for `int32_t modukit_*(` declarations and
/// diffs the sorted lists. Amend by D83 tail-add only.
pub const EMBED_VERBS: &[&str] = &[
    "modukit_embed_abi_version",
    "modukit_install",
    "modukit_uninstall",
    "modukit_resolve",
    "modukit_start",
    "modukit_stop",
    "modukit_state",
    "modukit_service_call",
    "modukit_bus_publish",
    "modukit_bus_subscribe",
    "modukit_bus_recv",
    "modukit_bus_unsubscribe",
    "modukit_clock_now",
    "modukit_clock_advance",
    "modukit_introspect_snapshot",
    "modukit_introspect_events",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_table_version_is_one() {
        assert_eq!(EMBED_ABI_VERSION, 1);
    }
}
