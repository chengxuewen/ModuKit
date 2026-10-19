//! Contract tests for the error-vocabulary surfaces (D84 coverage debt).
//!
//! Every row asserts the adjudicated behavior verbatim: the D51/D85 `code()`
//! mapping, the D86 stable human `hint()` lines, and the `Display` prose arm
//! (exact rendered line, including the carried field values). No
//! touching-lines-without-asserting: one row per variant, pinning text.

use std::time::Duration;

use modukit_core::bundle::BundleError;
use modukit_core::clock::{ClockDomain, DueJob, JobToken};
use modukit_core::error::{KernelError, ManifestError, RetryHint};
use modukit_core::identity::IdGrammar;
use modukit_core::proc_wire::{
    Call, DIAL_STATUS_REFUSED_REAL, DueTurn, FRAME_MAX, Hello, MSG_ACK, MSG_CALL, MSG_DIAL,
    MSG_DUE, MSG_HELLO, MSG_REGISTER, MSG_RESULT, MSG_START, MSG_STOP, ServiceRegistration,
    ServiceResult, WIRE_VERSION, call_payload, dial_payload, due_payload, hello_payload, parse_ack,
    parse_call, parse_dial, parse_due, parse_hello, parse_register, parse_result, register_payload,
    result_payload,
};
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

// ---------------------------------------------------------------------------
// ManifestError Display + code — the per-variant grammar prose (D51 family).
// ---------------------------------------------------------------------------

#[test]
fn manifest_error_display_and_code_cover_every_grammar_arm() {
    let rows: Vec<(ManifestError, &str)> = vec![
        (
            ManifestError::Toml("bad table".into()),
            "TOML shape error: bad table",
        ),
        (
            ManifestError::BadDuration("5x".into()),
            concat!(
                "duration must be `<count>s|m|h` (or `<min>..<max>` for backoff), ",
                "got \"5x\""
            ),
        ),
        (
            ManifestError::UnknownPlacement("so".into()),
            "entry.kind must be dl / exe / wasm, got \"so\" (05 §1)",
        ),
        (
            ManifestError::BadDepPolicy("maybe".into()),
            concat!(
                "requires policy must be wait(<t>) / failfast / fallback, ",
                "got \"maybe\" (02 §2)"
            ),
        ),
        (
            ManifestError::BadOnDown("ignore".into()),
            "on_down must be restart-self / block / fail-self, got \"ignore\" (02 §2)",
        ),
        (
            ManifestError::BadRestartPolicy("always".into()),
            "restart policy must be none / on-failure, got \"always\" (05 §2)",
        ),
        (
            ManifestError::BadBackoff("1..".into()),
            "backoff must be `<min>..<max>` durations, got \"1..\" (05 §2)",
        ),
    ];
    for (err, want) in rows {
        assert_eq!(err.to_string(), want, "ManifestError Display for {err:?}");
        assert_eq!(err.code(), "MANIFEST_INVALID", "code for {err:?}");
    }
}

// ---------------------------------------------------------------------------
// RegistryError Display (feeds the KernelError::Registry display arm).
// ---------------------------------------------------------------------------

#[test]
fn registry_error_display_renders_documented_prose_per_variant() {
    let rows: Vec<(RegistryError, &str)> = vec![
        (
            RegistryError::MissingServiceId,
            "registration properties must carry `service.id` (D9)",
        ),
        (
            RegistryError::UnknownRegistration { id: 42 },
            "no registration with id 42",
        ),
        (
            RegistryError::AlreadyReleased { id: 42 },
            "handle for registration 42 was already released",
        ),
    ];
    for (err, want) in rows {
        assert_eq!(err.to_string(), want, "RegistryError Display for {err:?}");
    }
}

// ---------------------------------------------------------------------------
// BundleError::code() — frozen-name mapping incl. the INTEGRITY_FAILED tier.
// ---------------------------------------------------------------------------

#[test]
fn bundle_error_code_map_splits_the_integrity_tier_exactly() {
    let integrity_tier: Vec<BundleError> = vec![
        BundleError::HashMismatch {
            path: "lib/plugin.so".into(),
            expected: "aa".into(),
            found: "bb".into(),
        },
        BundleError::SizeMismatch {
            path: "lib/plugin.so".into(),
            expected: 10,
            found: 12,
        },
        BundleError::ArtifactNotListed {
            path: "lib/plugin.so".into(),
        },
    ];
    for e in integrity_tier {
        assert_eq!(e.code(), "INTEGRITY_FAILED", "code for {e:?}");
    }

    let contract_tier: Vec<BundleError> = vec![
        BundleError::DuplicateId {
            id: "com.example.a".into(),
        },
        BundleError::CrossCheck { what: "abi".into() },
    ];
    for e in contract_tier {
        assert_eq!(e.code(), "CONTRACT_VIOLATION", "code for {e:?}");
    }

    let others: Vec<(BundleError, &str)> = vec![
        (
            BundleError::Config("layout root missing".into()),
            "SCHEMA_INVALID",
        ),
        (
            BundleError::Manifest(ManifestError::Toml("x".into())),
            "MANIFEST_INVALID",
        ),
        (
            BundleError::ManifestField("bad checklist".into()),
            "MANIFEST_INVALID",
        ),
        (
            BundleError::Grammar {
                value: "bad id".into(),
                reason: IdGrammar::IllegalChar,
            },
            "MANIFEST_INVALID",
        ),
        (
            BundleError::Io {
                path: "/tmp/b".into(),
                reason: "permission denied".into(),
            },
            "NOT_FOUND",
        ),
        (
            BundleError::MissingFile {
                path: "lib/x.so".into(),
            },
            "NOT_FOUND",
        ),
        (
            BundleError::NotInstalled {
                id: "com.example.a".into(),
            },
            "NOT_FOUND",
        ),
    ];
    for (e, want) in others {
        assert_eq!(e.code(), want, "code for {e:?}");
    }

    // Kernel passthrough rides the inner code (pure function, no capture).
    let inner = KernelError::ServiceCallFailed { code: -1 };
    assert_eq!(
        BundleError::Kernel(inner.clone()).code(),
        "SERVICE_CALL_FAILED"
    );
    assert_eq!(
        BundleError::Kernel(KernelError::ProcessCrashed {
            id: "p".into(),
            exit: "signal".into(),
        })
        .code(),
        "PROCESS_CRASHED"
    );
    assert_eq!(inner.code(), "SERVICE_CALL_FAILED");
}

// ---------------------------------------------------------------------------
// BundleError::hint() — the D86 stable one-line-per-variant strings, verbatim.
// ---------------------------------------------------------------------------

#[test]
fn bundle_error_hint_lines_are_the_d86_stable_strings() {
    let tamper = "the bundle was tampered or corrupted in transit - do not install; rebuild \
                  and re-sign the bundle";
    let manifest_fix = "fix modukit.toml: id/version/entry fields must match the contract grammar";
    let disk = "verify the bundle contents on disk; a listed member is unreadable";
    let rows: Vec<(BundleError, &str)> = vec![
        (
            BundleError::Config("layout root missing".into()),
            "check the installer arguments and the bundle layout root",
        ),
        (
            BundleError::Manifest(ManifestError::Toml("x".into())),
            manifest_fix,
        ),
        (
            BundleError::ManifestField("bad checklist".into()),
            manifest_fix,
        ),
        (
            BundleError::Grammar {
                value: "bad id".into(),
                reason: IdGrammar::IllegalChar,
            },
            manifest_fix,
        ),
        (
            BundleError::Io {
                path: "/tmp/b".into(),
                reason: "permission denied".into(),
            },
            disk,
        ),
        (
            BundleError::MissingFile {
                path: "lib/x.so".into(),
            },
            disk,
        ),
        (
            BundleError::HashMismatch {
                path: "lib/plugin.so".into(),
                expected: "aa".into(),
                found: "bb".into(),
            },
            tamper,
        ),
        (
            BundleError::SizeMismatch {
                path: "lib/plugin.so".into(),
                expected: 10,
                found: 12,
            },
            tamper,
        ),
        (
            BundleError::ArtifactNotListed {
                path: "lib/plugin.so".into(),
            },
            "list the entry artifact in the [[files]] checklist, then rebuild the bundle",
        ),
        (
            BundleError::DuplicateId {
                id: "com.example.a".into(),
            },
            "a bundle with this id is already installed - uninstall it first or bump the version",
        ),
        (
            BundleError::CrossCheck { what: "abi".into() },
            "the artifact self-report disagrees with modukit.toml - rebuild the bundle from \
             matching sources",
        ),
        (
            BundleError::Kernel(KernelError::HandshakeNull),
            "the artifact failed the door probe - see the kernel error for the specific stage",
        ),
        (
            BundleError::NotInstalled {
                id: "com.example.a".into(),
            },
            "nothing is installed under this id - install the bundle first",
        ),
    ];
    for (err, want) in rows {
        assert_eq!(err.hint(), want, "hint for {err:?}");
    }
}

#[test]
fn bundle_error_display_renders_documented_prose_per_variant() {
    let rows: Vec<(BundleError, String)> = vec![
        (
            BundleError::Config("layout root missing".into()),
            "install config: layout root missing".into(),
        ),
        (
            BundleError::Manifest(ManifestError::Toml("bad table".into())),
            "bundle manifest: TOML shape error: bad table".into(),
        ),
        (
            BundleError::ManifestField("bad checklist".into()),
            "bundle TOML: bad checklist".into(),
        ),
        (
            BundleError::Grammar {
                value: "Bad Id".into(),
                reason: IdGrammar::EmptyLabel,
            },
            concat!(
                "id \"Bad Id\" violates the reverse-DNS grammar: id has an empty label ",
                "(leading/doubled/trailing dot)"
            )
            .into(),
        ),
        (
            BundleError::Io {
                path: "/tmp/b".into(),
                reason: "permission denied".into(),
            },
            "/tmp/b: permission denied".into(),
        ),
        (
            BundleError::MissingFile {
                path: "lib/x.so".into(),
            },
            "listed member file missing: lib/x.so".into(),
        ),
        (
            BundleError::HashMismatch {
                path: "lib/plugin.so".into(),
                expected: "aa".into(),
                found: "bb".into(),
            },
            "hash mismatch for lib/plugin.so: manifest says aa, file says bb".into(),
        ),
        (
            BundleError::SizeMismatch {
                path: "lib/plugin.so".into(),
                expected: 10,
                found: 12,
            },
            "size mismatch for lib/plugin.so: manifest says 10, file says 12".into(),
        ),
        (
            BundleError::ArtifactNotListed {
                path: "lib/plugin.so".into(),
            },
            "artifact lib/plugin.so carries no hash-manifest entry".into(),
        ),
        (
            BundleError::DuplicateId {
                id: "com.example.a".into(),
            },
            "plugin id com.example.a is already installed (D34: one identity per host)".into(),
        ),
        (
            BundleError::CrossCheck { what: "abi".into() },
            "D7 cross-check failed: abi".into(),
        ),
        (
            BundleError::Kernel(KernelError::HandshakeNull),
            "artifact door: modukit_plugin_init returned a null descriptor".into(),
        ),
        (
            BundleError::NotInstalled {
                id: "com.example.a".into(),
            },
            "no installed bundle for id com.example.a".into(),
        ),
    ];
    for (err, want) in rows {
        assert_eq!(err.to_string(), want, "Display for {err:?}");
    }
}

// ---------------------------------------------------------------------------
// proc_wire — frame-kind constants and envelope version (the frozen grammar).
// ---------------------------------------------------------------------------

#[test]
fn wire_constants_are_the_frozen_frame_grammar() {
    assert_eq!(WIRE_VERSION, 1);
    assert_eq!(MSG_HELLO, 1);
    assert_eq!(MSG_START, 2);
    assert_eq!(MSG_STOP, 3);
    assert_eq!(MSG_ACK, 4);
    assert_eq!(MSG_REGISTER, 5);
    assert_eq!(MSG_CALL, 6);
    assert_eq!(MSG_RESULT, 7);
    assert_eq!(MSG_DIAL, 8);
    assert_eq!(MSG_DUE, 9);
    assert_eq!(DIAL_STATUS_REFUSED_REAL, -2);
    assert_eq!(FRAME_MAX, 64 * 1024);
}

// ---------------------------------------------------------------------------
// proc_wire — HELLO codec: roundtrip + every refusal arm reachable without a
// socket (truncated header, truncated pstr, trailing bytes, non-UTF-8).
// ---------------------------------------------------------------------------

#[test]
fn hello_roundtrip_preserves_all_fields() {
    let hello = Hello {
        abi: 1,
        id: "com.example.plugin".into(),
        version: "1.2.3".into(),
        name: "Example".into(),
    };
    let parsed = parse_hello(&hello_payload(&hello)).expect("roundtrip must parse");
    assert_eq!(parsed, hello);
}

#[test]
fn hello_parse_refuses_truncated_and_lying_frames() {
    let hello = Hello {
        abi: 1,
        id: "com.example.plugin".into(),
        version: "1.2.3".into(),
        name: "Example".into(),
    };
    let full = hello_payload(&hello);

    // Shorter than the u32 abi header.
    assert_eq!(parse_hello(&full[..3]), None);
    // The last pstr declares more bytes than the frame carries.
    assert_eq!(parse_hello(&full[..full.len() - 1]), None);
    // Trailing bytes after the declared name: a lie, refused (strict tail).
    let mut trailing = full.clone();
    trailing.push(0);
    assert_eq!(parse_hello(&trailing), None);
    // Non-UTF-8 id bytes refuse the frame (silence and lies both disagree).
    let mut lying = full.clone();
    assert_eq!(lying[4] as usize, "com.example.plugin".len());
    lying[5] = 0xFF;
    assert_eq!(parse_hello(&lying), None);
}

// ---------------------------------------------------------------------------
// proc_wire — ACK codec: exactly one i32 LE or nothing.
// ---------------------------------------------------------------------------

#[test]
fn ack_codec_takes_exactly_one_i32_le() {
    assert_eq!(parse_ack(&(-7i32).to_le_bytes()), Some(-7));
    assert_eq!(parse_ack(&0i32.to_le_bytes()), Some(0));
    assert_eq!(parse_ack(&[0, 0, 0]), None);
    assert_eq!(parse_ack(&[0, 0, 0, 0, 0]), None);
}

// ---------------------------------------------------------------------------
// proc_wire — REGISTER codec.
// ---------------------------------------------------------------------------

#[test]
fn register_roundtrip_preserves_service_and_method_list() {
    let reg = ServiceRegistration {
        service_id: "com.example.svc".into(),
        methods: vec!["paint".into(), "resize".into(), "".into()],
    };
    let parsed = parse_register(&register_payload(&reg)).expect("roundtrip must parse");
    assert_eq!(parsed, reg);
}

#[test]
fn register_parse_refuses_malformed_frames() {
    // Empty payload: no service id at all.
    assert_eq!(parse_register(&[]), None);
    // Service id pstr declares more bytes than present.
    assert_eq!(parse_register(&[9, b'a']), None);
    // Count byte promises two methods but the frame carries none.
    let mut short = register_payload(&ServiceRegistration {
        service_id: "s".into(),
        methods: vec![],
    });
    short[1 + "s".len()] = 2;
    assert_eq!(parse_register(&short), None);
    // Trailing garbage after the last method is refused.
    let mut trailing = register_payload(&ServiceRegistration {
        service_id: "s".into(),
        methods: vec!["m".into()],
    });
    trailing.push(0xAB);
    assert_eq!(parse_register(&trailing), None);
    // Non-UTF-8 method bytes refuse.
    let mut lying = register_payload(&ServiceRegistration {
        service_id: "s".into(),
        methods: vec!["m".into()],
    });
    let m_pos = 1 + "s".len() + 1;
    lying[m_pos + 1] = 0xFF;
    assert_eq!(parse_register(&lying), None);
}

// ---------------------------------------------------------------------------
// proc_wire — CALL codec.
// ---------------------------------------------------------------------------

#[test]
fn call_roundtrip_preserves_seq_method_and_input() {
    let call = call_payload(9, "paint", &[1, 2, 3]);
    let parsed = parse_call(&call).expect("roundtrip must parse");
    assert_eq!(
        parsed,
        Call {
            seq: 9,
            method: "paint".into(),
            input: vec![1, 2, 3],
        }
    );
    // Empty input is a legal shape.
    let empty = parse_call(&call_payload(0, "m", &[])).expect("empty input parses");
    assert!(empty.input.is_empty());
}

#[test]
fn call_parse_refuses_malformed_frames() {
    // Shorter than the seq header.
    assert_eq!(parse_call(&[1, 2, 3]), None);
    // Method pstr truncated.
    assert_eq!(parse_call(&[0, 0, 0, 0, 5, b'a']), None);
    // Input-length prefix truncated (not even 4 bytes after the method).
    let mut p = call_payload(1, "m", &[7, 8]);
    let n = p.len();
    p.truncate(n - 2);
    assert_eq!(parse_call(&p), None);
    // Declared input length longer than the carried bytes (len lives at
    // offset 6: 4 seq + 1+1 method pstr).
    let mut p = call_payload(1, "m", &[7]);
    p[6] = 2;
    assert_eq!(parse_call(&p), None);
    // Declared input length shorter than the carried bytes.
    let mut p = call_payload(1, "m", &[7, 8]);
    p[6] = 0;
    assert_eq!(parse_call(&p), None);
}

// ---------------------------------------------------------------------------
// proc_wire — RESULT codec.
// ---------------------------------------------------------------------------

#[test]
fn result_roundtrip_preserves_seq_status_and_output() {
    for (seq, status, out) in [(1u32, 0i32, vec![9u8, 8]), (u32::MAX, i32::MIN, vec![])] {
        let parsed = parse_result(&result_payload(seq, status, &out)).expect("roundtrip");
        assert_eq!(
            parsed,
            ServiceResult {
                seq,
                status,
                output: out,
            }
        );
    }
}

#[test]
fn result_parse_refuses_malformed_frames() {
    // Shorter than seq+status.
    assert_eq!(parse_result(&[0; 7]), None);
    // Length prefix truncated.
    let mut p = result_payload(1, 0, &[5]);
    p.truncate(p.len() - 1);
    assert_eq!(parse_result(&p), None);
    // Trailing bytes past the declared output length are refused.
    let mut p = result_payload(1, 0, &[5]);
    p.push(0);
    assert_eq!(parse_result(&p), None);
    // Declared length disagrees downward (len lives at offset 8: 4 seq +
    // 4 status).
    let mut p = result_payload(1, 0, &[5, 6]);
    p[8] = 0;
    assert_eq!(parse_result(&p), None);
    // Negative status passes through verbatim (D27 author code space).
    assert_eq!(
        parse_result(&result_payload(3, -42, &[])).map(|r| r.status),
        Some(-42)
    );
}

// ---------------------------------------------------------------------------
// proc_wire — DIAL codec: u64 LE micros with named saturation.
// ---------------------------------------------------------------------------

#[test]
fn dial_roundtrip_and_saturation_at_u64_max() {
    assert_eq!(
        parse_dial(&dial_payload(Duration::from_micros(1500))),
        Some(Duration::from_micros(1500))
    );
    // Named saturation: a dial longer than u64 micros clamps, never wraps.
    assert_eq!(
        parse_dial(&dial_payload(Duration::from_secs(u64::MAX))),
        Some(Duration::from_micros(u64::MAX))
    );
    assert_eq!(parse_dial(&[0; 7]), None);
    assert_eq!(parse_dial(&[0; 9]), None);
}

// ---------------------------------------------------------------------------
// proc_wire — DUE codec: zero- and multi-job turns + every refusal arm.
// ---------------------------------------------------------------------------

#[test]
fn due_roundtrip_preserves_status_stamp_and_job_list() {
    let turn = DueTurn {
        status: 0,
        fired_at_ns: 1_234_567_890,
        jobs: vec![
            DueJob {
                token: JobToken::at(7),
                job: "tick".into(),
            },
            DueJob {
                token: JobToken::at(u64::MAX),
                job: "".into(),
            },
        ],
    };
    let parsed = parse_due(&due_payload(&turn)).expect("roundtrip must parse");
    assert_eq!(parsed, turn);
    let empty = DueTurn {
        status: -2,
        fired_at_ns: 0,
        jobs: vec![],
    };
    assert_eq!(parse_due(&due_payload(&empty)), Some(empty));
}

#[test]
fn due_parse_refuses_malformed_frames() {
    // Below the fixed 14-byte header.
    assert_eq!(parse_due(&[0; 13]), None);
    // Count promises one job but the token bytes are missing.
    let mut p = due_payload(&DueTurn {
        status: 0,
        fired_at_ns: 0,
        jobs: vec![],
    });
    p[12] = 1;
    assert_eq!(parse_due(&p), None);

    let one_job = || {
        due_payload(&DueTurn {
            status: 0,
            fired_at_ns: 0,
            jobs: vec![DueJob {
                token: JobToken::at(1),
                job: "job".into(),
            }],
        })
    };
    // Job pstr declares more bytes than remain (length byte at offset
    // 14 header + 8 token = 22).
    let mut p = one_job();
    p[22] = 9;
    assert_eq!(parse_due(&p), None);
    // Non-UTF-8 job label refuses the whole frame.
    let mut p = one_job();
    p[14 + 8 + 1] = 0xFF;
    assert_eq!(parse_due(&p), None);
    // Trailing garbage after the last job is refused (strict tail).
    let mut p = one_job();
    p.push(0);
    assert_eq!(parse_due(&p), None);
    // D74 domain discipline the due mapping rides on.
    assert_ne!(ClockDomain::Real, ClockDomain::Step);
}

// ---------------------------------------------------------------------------
// Error trait plumbing: std::error::Error impls and From conversions are
// adjudicated behavior too.
// ---------------------------------------------------------------------------

#[test]
fn kernel_and_bundle_errors_are_std_errors_and_cloneable() {
    fn assert_error<E: std::error::Error + Clone + PartialEq + std::fmt::Debug>(_e: &E) {}
    let k = KernelError::HandshakeNull;
    let b = BundleError::NotInstalled { id: "x".into() };
    assert_error(&k);
    assert_error(&b);
    assert_eq!(k.clone(), k);
    assert_eq!(b.clone(), b);
}

#[test]
fn from_impls_wrap_without_loss() {
    let m = ManifestError::BadDuration("9z".into());
    assert_eq!(
        KernelError::from(m.clone()),
        KernelError::Manifest(m.clone())
    );
    assert_eq!(
        BundleError::from(m),
        BundleError::Manifest(ManifestError::BadDuration("9z".into()))
    );
    let r = RegistryError::AlreadyReleased { id: 5 };
    assert_eq!(
        KernelError::from(r),
        KernelError::Registry(RegistryError::AlreadyReleased { id: 5 })
    );
}
