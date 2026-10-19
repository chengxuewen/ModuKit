use modukit_core::error::{KernelError, ManifestError, RetryHint};
use modukit_core::registry::RegistryError;

// ---------------------------------------------------------------------------
// KernelError::code() — the D85-mapped machine channel.
// ---------------------------------------------------------------------------

#[test]
fn kernel_error_code_map_is_the_frozen_d85_mapping() {
    let rows: Vec<(KernelError, &str)> = vec![
        (
            KernelError::InvalidTransition {
                from: "Stopped".into(),
                to: "Running".into(),
            },
            "CONTRACT_VIOLATION",
        ),
        (
            KernelError::Manifest(ManifestError::Toml("missing field".into())),
            "MANIFEST_INVALID",
        ),
        (
            KernelError::Registry(RegistryError::MissingServiceId),
            "CONTRACT_VIOLATION",
        ),
        (
            KernelError::Registry(RegistryError::UnknownRegistration { id: 3 }),
            "NOT_FOUND",
        ),
        (
            KernelError::Registry(RegistryError::AlreadyReleased { id: 3 }),
            "NOT_FOUND",
        ),
        (
            KernelError::ArtifactMissing {
                path: "/opt/p/lib.so".into(),
            },
            "MANIFEST_INVALID",
        ),
        (
            KernelError::LoadFailed {
                path: "/opt/p/lib.so".into(),
                reason: "cannot open".into(),
            },
            "NOT_FOUND",
        ),
        (
            KernelError::SymbolMissing {
                path: "/opt/p/lib.so".into(),
            },
            "ABI_MISMATCH",
        ),
        (KernelError::HandshakeNull, "ABI_MISMATCH"),
        (
            KernelError::AbiMismatch {
                expected: 1,
                found: 2,
            },
            "ABI_MISMATCH",
        ),
        (
            KernelError::DescriptorTooSmall {
                expected: 64,
                found: 16,
            },
            "ABI_MISMATCH",
        ),
        (
            KernelError::HandshakeIncomplete { what: "tick" },
            "ABI_MISMATCH",
        ),
        (
            KernelError::DependencyMissing {
                service: "svc.clock".into(),
            },
            "DEPENDENCY_MISSING",
        ),
        (
            KernelError::PluginStartFailed { status: 7 },
            "CONTRACT_VIOLATION",
        ),
        (
            KernelError::ServiceCallFailed { code: -3 },
            "SERVICE_CALL_FAILED",
        ),
        (
            KernelError::UnsupportedPlacement {
                kind: "wasm".into(),
            },
            "CONTRACT_VIOLATION",
        ),
        (KernelError::UnknownPlugin { id: "p".into() }, "NOT_FOUND"),
        (
            KernelError::ProcessCrashed {
                id: "p".into(),
                exit: "signal 11".into(),
            },
            "PROCESS_CRASHED",
        ),
        (
            KernelError::WireVersionMismatch {
                expected: 1,
                found: 9,
            },
            "ABI_MISMATCH",
        ),
        (
            KernelError::ClockDialRefused { status: -2 },
            "CLOCK_DIAL_REFUSED",
        ),
        (
            KernelError::RoutingDeferred {
                plugin: "p".into(),
                service: "svc.ui".into(),
            },
            "DEFERRAL_DECLINED",
        ),
        (
            KernelError::IdentityMismatch {
                what: "version",
                expected: "1.0.0".into(),
                found: "2.0.0".into(),
            },
            "IDENTITY_MISMATCH",
        ),
    ];
    for (err, want) in rows {
        assert_eq!(err.code(), want, "code() for {err:?}");
    }
}

// ---------------------------------------------------------------------------
// KernelError::hint() — the D27 policy-engine channel (RetryHint IS behavior).
// ---------------------------------------------------------------------------

#[test]
fn kernel_error_hint_retry_after_family_is_exactly_three_variants() {
    for err in [
        KernelError::DependencyMissing {
            service: "svc.clock".into(),
        },
        KernelError::LoadFailed {
            path: "/opt/p/lib.so".into(),
            reason: "cannot open".into(),
        },
        KernelError::ProcessCrashed {
            id: "p".into(),
            exit: "signal 11".into(),
        },
    ] {
        assert_eq!(err.hint(), RetryHint::RetryAfter, "hint for {err:?}");
    }
}

#[test]
fn kernel_error_hint_every_other_variant_is_do_not_retry() {
    let input_shaped: Vec<KernelError> = vec![
        KernelError::InvalidTransition {
            from: "Stopped".into(),
            to: "Running".into(),
        },
        KernelError::Manifest(ManifestError::BadDuration("5x".into())),
        KernelError::Registry(RegistryError::MissingServiceId),
        KernelError::Registry(RegistryError::UnknownRegistration { id: 1 }),
        KernelError::Registry(RegistryError::AlreadyReleased { id: 1 }),
        KernelError::ArtifactMissing {
            path: "/opt/p/lib.so".into(),
        },
        KernelError::SymbolMissing {
            path: "/opt/p/lib.so".into(),
        },
        KernelError::HandshakeNull,
        KernelError::AbiMismatch {
            expected: 1,
            found: 2,
        },
        KernelError::DescriptorTooSmall {
            expected: 64,
            found: 16,
        },
        KernelError::HandshakeIncomplete { what: "on_stop" },
        KernelError::PluginStartFailed { status: 1 },
        KernelError::ServiceCallFailed { code: -1 },
        KernelError::UnsupportedPlacement {
            kind: "wasm".into(),
        },
        KernelError::UnknownPlugin { id: "p".into() },
        KernelError::WireVersionMismatch {
            expected: 1,
            found: 2,
        },
        KernelError::ClockDialRefused { status: -2 },
        KernelError::RoutingDeferred {
            plugin: "p".into(),
            service: "svc.ui".into(),
        },
        KernelError::IdentityMismatch {
            what: "id",
            expected: "com.example.a".into(),
            found: "com.example.b".into(),
        },
    ];
    for err in input_shaped {
        assert_eq!(err.hint(), RetryHint::DoNotRetry, "hint for {err:?}");
    }
}

#[test]
fn retry_hint_values_are_distinct_and_clonable() {
    let a = RetryHint::RetryAfter;
    let b = a;
    assert_eq!(a, b);
    assert_ne!(RetryHint::RetryNow, RetryHint::RetryAfter);
    assert_ne!(RetryHint::RetryNow, RetryHint::DoNotRetry);
    assert_ne!(RetryHint::RetryAfter, RetryHint::DoNotRetry);
}

// ---------------------------------------------------------------------------
// KernelError Display — the prose channel (exact rendered line per variant).
// ---------------------------------------------------------------------------

#[test]
fn kernel_error_display_renders_documented_prose_per_variant() {
    let rows: Vec<(KernelError, String)> = vec![
        (
            KernelError::InvalidTransition {
                from: "Stopped".into(),
                to: "Running".into(),
            },
            "lifecycle transition Stopped -> Running is not in the rules table".into(),
        ),
        (
            KernelError::Manifest(ManifestError::BadDuration("5x".into())),
            concat!(
                "manifest: duration must be `<count>s|m|h` (or `<min>..<max>` for backoff), ",
                "got \"5x\""
            )
            .into(),
        ),
        (
            KernelError::Registry(RegistryError::MissingServiceId),
            "registry: registration properties must carry `service.id` (D9)".into(),
        ),
        (
            KernelError::ArtifactMissing {
                path: "/opt/p/lib.so".into(),
            },
            "artifact not present: /opt/p/lib.so".into(),
        ),
        (
            KernelError::LoadFailed {
                path: "/opt/p/lib.so".into(),
                reason: "cannot open shared object".into(),
            },
            "dlopen of /opt/p/lib.so failed: cannot open shared object".into(),
        ),
        (
            KernelError::SymbolMissing {
                path: "/opt/p/lib.so".into(),
            },
            "entry symbol modukit_plugin_init missing in /opt/p/lib.so".into(),
        ),
        (
            KernelError::HandshakeNull,
            "modukit_plugin_init returned a null descriptor".into(),
        ),
        (
            KernelError::AbiMismatch {
                expected: 1,
                found: 2,
            },
            "ABI mismatch: host speaks v1, plugin reports v2".into(),
        ),
        (
            KernelError::DescriptorTooSmall {
                expected: 64,
                found: 16,
            },
            "PluginAbiV1 descriptor too small: expected 64 bytes, got 16".into(),
        ),
        (
            KernelError::HandshakeIncomplete { what: "on_stop" },
            "descriptor field on_stop is null".into(),
        ),
        (
            KernelError::DependencyMissing {
                service: "svc.clock".into(),
            },
            "required service missing at resolve: svc.clock".into(),
        ),
        (
            KernelError::PluginStartFailed { status: 7 },
            "plugin start() returned error status 7".into(),
        ),
        (
            KernelError::ServiceCallFailed { code: -3 },
            "service call failed with plugin-side code -3".into(),
        ),
        (
            KernelError::UnsupportedPlacement {
                kind: "wasm".into(),
            },
            "placement wasm has no executor in this kernel (dl only until S2-M1 wiring)".into(),
        ),
        (
            KernelError::UnknownPlugin { id: "p".into() },
            "no plugin record for p".into(),
        ),
        (
            KernelError::ProcessCrashed {
                id: "p".into(),
                exit: "signal 11".into(),
            },
            "supervised plugin p died outside stop (signal 11)".into(),
        ),
        (
            KernelError::RoutingDeferred {
                plugin: "p".into(),
                service: "svc.ui".into(),
            },
            concat!(
                "plugin p declares parallel_ok for svc.ui; the swap door was asked to co-host two ",
                "versions but version-affinity routing is deferred (D80) — stop the running ",
                "instance first: stop + install is the legal swap lane"
            )
            .into(),
        ),
        (
            KernelError::ClockDialRefused { status: -2 },
            concat!(
                "clock dial refused by the child (status -2): the dial turns Step/Sim clocks only ",
                "(D74) — a Real-domain placement has no dial to turn"
            )
            .into(),
        ),
        (
            KernelError::WireVersionMismatch {
                expected: 1,
                found: 9,
            },
            "frame wire version mismatch: host speaks v1, peer framed v9".into(),
        ),
        (
            KernelError::IdentityMismatch {
                what: "version",
                expected: "1.0.0".into(),
                found: "2.0.0".into(),
            },
            concat!(
                "handshake self-report version mismatch: manifest says \"1.0.0\", artifact says ",
                "\"2.0.0\""
            )
            .into(),
        ),
    ];
    for (err, want) in rows {
        assert_eq!(err.to_string(), want, "Display prose mismatch for {err:?}");
    }
}
