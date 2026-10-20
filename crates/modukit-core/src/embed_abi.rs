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
//! The HostEmbedV1 embed surface itself touches neither of the two frozen
//! v1 plugin tables (`abi::PluginAbiV1` / `abi::HostAbiV1`, D87 STABLE);
//! the `PLUGIN_ABI_FIELDS`/`HOST_ABI_FIELDS` constants below are read-only
//! field-name mirrors of those tables for the plugin-header parity gate -
//! the layout truth stays in `abi.rs`.

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
    // M2 tail-add: the exe rung (supervisor). Order = header declaration order.
    "modukit_sup_install",
    "modukit_sup_stop",
    "modukit_sup_state",
    "modukit_sup_step",
    "modukit_sup_dial",
    "modukit_sup_restart",
    "modukit_sup_uninstall",
];

/// `abi::PluginAbiV1` fields in declaration order - the third parity
/// source for `contracts/modukit_plugin.h` (gap ruling A). The parity gate
/// (`scripts/check-embed-parity.sh`) diffs the ORDERED name lists
/// abi.rs <-> header <-> this const: the `#[repr(C)]` layout is the
/// contract, so order matters here (unlike the sorted EMBED_VERBS diff
/// above). C member names are the identical Rust snake names - the
/// mapping discipline is documented in the C header and in the script.
///
/// Split of duties: the unit test below asserts these lists are non-empty
/// and unique; the truth of ordering against abi.rs and the header lives
/// in the script's ordered diffs (abi.rs is the frozen truth, D87).
pub const PLUGIN_ABI_FIELDS: &[&str] = &[
    "abi_version",
    "size",
    "name",
    "id",
    "version",
    "start",
    "stop",
];

/// `abi::HostAbiV1` fields in declaration order (same contract as
/// `PLUGIN_ABI_FIELDS`).
pub const HOST_ABI_FIELDS: &[&str] = &["size", "user", "register_service"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_table_version_is_one() {
        assert_eq!(EMBED_ABI_VERSION, 1);
    }

    #[test]
    fn plugin_table_field_lists_are_populated_and_unique() {
        // Ordering truth against abi.rs and the header lives in the parity
        // script's ordered diffs; this test guards the mirror list shape.
        for fields in [PLUGIN_ABI_FIELDS, HOST_ABI_FIELDS] {
            assert!(!fields.is_empty(), "field list must not be empty");
            for (i, name) in fields.iter().enumerate() {
                assert!(!fields[..i].contains(name), "duplicate field name");
            }
        }
    }
}
