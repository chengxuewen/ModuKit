use modukit_core::error::ManifestError;
use modukit_core::registry::RegistryError;

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
