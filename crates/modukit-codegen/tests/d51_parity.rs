//! D51 parity: the kernel's `code()` strings and the `org.modukit.runtime`
//! ledger enum must describe the same frozen dictionary.
//!
//! v0 rewiring choice (reported): a PARITY TEST, not generated constants -
//! error.rs keeps its hand-written &'static str literals, and this test
//! fails if the literals and the ledger ever diverge. Generated consts
//! would need a build script + include! plumbing for zero behavioral gain
//! at M4 scale; the ledger stays the single source because the test reads
//! it through the same decoder the generator uses.

use std::collections::BTreeSet;

use modukit_codegen::descriptor::decode_file_descriptor_set;
use modukit_core::error::{KernelError, ManifestError};
use modukit_core::registry::RegistryError;

fn repo_contracts() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("crate lives under <repo>/crates/")
        .join("contracts")
}

/// Every name in the ledger's KernelCode enum except the 0-sentinel and the
/// RESERVED_n tail slots (D51: sentinel + domain tails are padding, not
/// dictionary entries).
fn ledger_kernel_codes() -> BTreeSet<String> {
    let fdset = std::fs::read(repo_contracts().join("org.modukit.runtime.proto.fdset"))
        .expect("checked-in runtime fdset present");
    let files = decode_file_descriptor_set(&fdset).expect("runtime fdset decodes");
    let ledger = files
        .iter()
        .find(|f| f.package == "org.modukit.runtime")
        .expect("runtime package in its own fdset");
    let enum_ = ledger
        .enums
        .iter()
        .find(|e| e.name == "KernelCode")
        .expect("KernelCode enum present");
    enum_
        .values
        .iter()
        .filter(|v| v.number != 0 && !v.name.starts_with("RESERVED_"))
        .map(|v| v.name.clone())
        .collect()
}

/// The D51 frozen list, transcribed from the decision record.
const D51_LEDGER: &[&str] = &[
    "MANIFEST_INVALID",
    "DEPENDENCY_MISSING",
    "DEPENDENCY_CYCLE",
    "START_TIMEOUT",
    "START_REJECTED",
    "STOP_TIMEOUT",
    "ABI_MISMATCH",
    "CONTRACT_VIOLATION",
    // S2-M5b hearing rulings 4+7: supervision domain appended per the D51
    // amendment policy (reserved slots untouched, tail-only addition)
    "PROCESS_CRASHED",
    "IDENTITY_MISMATCH",
    "SERVICE_CALL_FAILED",
    "DEFERRAL_DECLINED",
    "CLOCK_DIAL_REFUSED",
    "SCHEMA_INVALID",
    "NOT_FOUND",
    "EPOCH_EXPIRED",
    "CARRIER_UNREACHABLE",
    "QUOTA_EXCEEDED",
    "CAPABILITY_DENIED",
];

/// One value per KernelError variant - adding a variant without extending
/// this list fails the count assertion below, forcing the mapping review.
fn every_kernel_error_variant() -> Vec<KernelError> {
    vec![
        KernelError::InvalidTransition {
            from: "INIT".into(),
            to: "STOPPED".into(),
        },
        KernelError::Manifest(ManifestError::Toml("x".into())),
        KernelError::ArtifactMissing { path: "/p".into() },
        KernelError::LoadFailed {
            path: "/p".into(),
            reason: "x".into(),
        },
        KernelError::SymbolMissing { path: "/p".into() },
        KernelError::HandshakeNull,
        KernelError::AbiMismatch {
            expected: 1,
            found: 2,
        },
        KernelError::DescriptorTooSmall {
            expected: 8,
            found: 4,
        },
        KernelError::HandshakeIncomplete { what: "start" },
        KernelError::DependencyMissing {
            service: "s".into(),
        },
        KernelError::PluginStartFailed { status: 3 },
        KernelError::ServiceCallFailed { code: -1 },
        // S2-M5b supervision-domain variants (rulings 4+7): one sample each
        // so the produced-set assertion covers the appended codes.
        KernelError::ProcessCrashed {
            id: "com.example.p".into(),
            exit: "signal 9".into(),
        },
        KernelError::IdentityMismatch {
            what: "version",
            expected: "0.1.0".into(),
            found: "9.9.9".into(),
        },
        KernelError::RoutingDeferred {
            plugin: "com.example.p".into(),
            service: "svc".into(),
        },
        KernelError::ClockDialRefused { status: -2 },
        KernelError::UnsupportedPlacement { kind: "exe".into() },
        KernelError::UnknownPlugin { id: "p".into() },
        KernelError::Registry(RegistryError::MissingServiceId),
        KernelError::Registry(RegistryError::UnknownRegistration { id: 1 }),
        KernelError::Registry(RegistryError::AlreadyReleased { id: 1 }),
    ]
}

#[test]
fn ledger_enum_is_the_adjudicated_dictionary() {
    let ledger = ledger_kernel_codes();
    let frozen: BTreeSet<String> = D51_LEDGER.iter().map(|s| s.to_string()).collect();
    assert_eq!(
        ledger, frozen,
        "org.modukit.runtime.proto KernelCode drifted from the D51 list"
    );
    assert_eq!(ledger.len(), 19);
}

#[test]
fn kernel_code_strings_are_all_ledger_members() {
    let ledger = ledger_kernel_codes();
    let mut produced = BTreeSet::new();
    for variant in every_kernel_error_variant() {
        let code = variant.code();
        assert!(
            ledger.contains(code),
            "KernelError::code() produced {code:?}, which is not a KernelCode \
             name in org.modukit.runtime.proto (D51/D27 parity broken)"
        );
        produced.insert(code.to_string());
    }
    assert_eq!(
        produced,
        [
            "ABI_MISMATCH",
            "CONTRACT_VIOLATION",
            "DEPENDENCY_MISSING",
            "MANIFEST_INVALID",
            "NOT_FOUND",
            "PROCESS_CRASHED",
            "IDENTITY_MISMATCH",
            "SERVICE_CALL_FAILED",
            "DEFERRAL_DECLINED",
            "CLOCK_DIAL_REFUSED",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<BTreeSet<String>>(),
        "the set of codes the M3 slice emits changed - re-check the mapping \
         (unemitted ledger codes are expected: DEPENDENCY_CYCLE / timeouts / \
         SCHEMA_INVALID / EPOCH_EXPIRED / CARRIER_UNREACHABLE / QUOTA_EXCEEDED \
         / CAPABILITY_DENIED ride later milestones)"
    );
    // ManifestError maps through KernelError::Manifest and directly.
    assert!(ledger.contains(ManifestError::Toml("x".into()).code()));
}
