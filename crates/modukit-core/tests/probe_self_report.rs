//! Tests for the two additive tail-probe doors (D83 additive policy):
//!
//! * `Manifest::dl_internal` — a grammar-legal internal/probe manifest whose
//!   non-entry fields are exactly what a minimal `dl` TOML parse yields.
//! * `DlopenExecutor::probe_self_report` — reads the descriptor self-report
//!   through the SAME guard sequence as `Executor::load` (single source of
//!   truth) and releases the handle.
//!
//! Written before the implementation (C0: red observed first). Fixture builds
//! follow the dlopen_kernel.rs in-test pattern (cdylibs are not built by
//! `cargo test`); when the build cannot run the fixture test skips green.

use std::path::{Path, PathBuf};
use std::process::Command;

use modukit_core::error::KernelError;
use modukit_core::executor::{DlopenExecutor, Executor};
use modukit_core::identity::PluginId;
use modukit_core::manifest::Manifest;

const EXAMPLE_ID: &str = "com.example.modukit.demo";

/// Build one fixture crate into a private target dir, return the .so path.
/// `None` = cannot build here (test skips with the printed reason).
fn build_fixture(crate_name: &str, lib_stem: &str) -> Option<PathBuf> {
    if std::env::var_os("MODUKIT_SKIP_DLOPEN").is_some() {
        eprintln!("SKIPPING: MODUKIT_SKIP_DLOPEN is set");
        return None;
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()? // crates/
        .parent()?; // repo root
    let target_dir = repo.join("out").join("dlopen-test-target");
    let status = Command::new(&cargo)
        .current_dir(repo)
        .args(["build", "-p", crate_name, "--target-dir"])
        .arg(&target_dir)
        .env("CARGO_TARGET_DIR", &target_dir)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("SKIPPING: fixture build failed: {crate_name} exited {s}");
            return None;
        }
        Err(e) => {
            eprintln!("SKIPPING: cannot spawn cargo for fixture build: {e}");
            return None;
        }
    }
    let so = target_dir.join("debug").join(format!("lib{lib_stem}.so"));
    if so.exists() {
        Some(so)
    } else {
        eprintln!("SKIPPING: expected artifact {} absent", so.display());
        None
    }
}

/// Minimal grammar-legal `dl` manifest TOML pointing at `artifact`.
fn minimal_dl_toml(artifact: &Path) -> String {
    format!(
        "id = \"{EXAMPLE_ID}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"dl\"\nartifact = \"{}\"\n",
        artifact.display()
    )
}

// (a) Round-trip equivalence: the lightweight constructor must produce a
// manifest byte-equal to a minimal TOML parse, defaults and all. The id and
// artifact carry through; every other field rides the RawManifest serde
// defaults (team "default", start_level 0, stop_timeout 2s, RestartPolicy::None,
// empty requires/provides/capabilities, no runner/min_host/start_timeout).
#[test]
fn dl_internal_matches_a_minimal_toml_parse_field_for_field() {
    let minimal = "id = \"com.internal.probe\"\nversion = \"\"\n[entry]\nkind = \"dl\"\nartifact = \"/tmp/probe.so\"\n";
    let parsed = Manifest::from_toml_str(minimal).expect("minimal dl TOML must parse");
    let built = Manifest::dl_internal(
        PluginId::new("com.internal.probe".to_string()),
        "/tmp/probe.so",
    );
    assert_eq!(
        built, parsed,
        "dl_internal must equal the minimal-parse default"
    );
    // The entry pair the executor reads is carried verbatim.
    assert_eq!(built.entry.artifact, "/tmp/probe.so");
}

// (b) Guard parity: probe_self_report must reject exactly what load rejects,
// with the same typed KernelError variant — for both the missing-file and the
// wrong-ABI fixture arms.
#[test]
fn probe_rejects_missing_artifact_with_the_same_variant_as_load() {
    let missing = Path::new("/nonexistent/modukit/probe-of-nothing.so");
    let manifest = Manifest::from_toml_str(&minimal_dl_toml(missing))
        .expect("manifest grammar is placement-agnostic");
    let load_err = match DlopenExecutor::new().load(&manifest) {
        Ok(_) => panic!("missing artifact must not load"),
        Err(e) => e,
    };
    let probe_err =
        DlopenExecutor::probe_self_report(missing).expect_err("missing artifact must not probe");
    assert!(
        matches!(load_err, KernelError::ArtifactMissing { .. }),
        "{load_err}"
    );
    assert_eq!(
        load_err, probe_err,
        "probe must reject a missing artifact with the load door's exact variant"
    );
}

#[test]
fn probe_rejects_bad_abi_with_the_same_variant_as_load() {
    let Some(so) = build_fixture("bad-plugin", "bad_plugin") else {
        return;
    };
    let manifest =
        Manifest::from_toml_str(&minimal_dl_toml(&so)).expect("bad fixture manifest parses");
    let load_err = match DlopenExecutor::new().load(&manifest) {
        Ok(_) => panic!("ABI v999 must never load"),
        Err(e) => e,
    };
    let probe_err = DlopenExecutor::probe_self_report(&so).expect_err("ABI v999 must never probe");
    assert!(
        matches!(load_err, KernelError::AbiMismatch { found: 999, .. }),
        "expected typed AbiMismatch(found=999), got: {load_err}"
    );
    assert_eq!(
        load_err, probe_err,
        "probe must reject a bad-ABI artifact with the load door's exact variant"
    );
}

// (c) Happy path: a real built fixture reports its descriptor name and the
// self-report id/version tuple through the probe door, handle released.
#[test]
fn probe_reads_self_report_for_a_good_artifact() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let report = DlopenExecutor::probe_self_report(&so).expect("probe must read the good fixture");
    assert_eq!(report.plugin_name, "example-plugin");
    assert_eq!(report.id.as_deref(), Some(EXAMPLE_ID));
    assert_eq!(report.version.as_deref(), Some("0.1.0"));
}
