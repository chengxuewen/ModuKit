use modukit_core::bundle::BundleError;
use modukit_core::error::{KernelError, ManifestError};
use modukit_core::identity::IdGrammar;

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
