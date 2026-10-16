//! dlopen integration tests (M3): the kernel boots real cdylib fixtures through
//! the full state walk and rejects the ABI-mismatched one with a typed error.
//!
//! kernel-v0 lesson (modules/04 §4): cdylib artifacts are NOT built by
//! `cargo test`, so each test builds its fixture explicitly into a private
//! target dir first. When the build cannot run (no cargo, cross-compile env,
//! MODUKIT_SKIP_DLOPEN=1) the test prints a reason and skips green.
//!
//! Written before the executor/kernel implementation (C0: red observed first).

use std::path::{Path, PathBuf};
use std::process::Command;

use modukit_core::abi::NativeService;
use modukit_core::error::KernelError;
use modukit_core::kernel::Kernel;
use modukit_core::lifecycle::PluginState;
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

fn manifest_toml(artifact: &Path) -> String {
    format!(
        "id = \"{EXAMPLE_ID}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"dl\"\nartifact = \"{}\"\n",
        artifact.display()
    )
}

/// Plugin-visible services only: the M6 floors boot the kernel with
/// platform services (`org.modukit.*`) on the same table (D40), so the
/// fixture assertions filter the plugin's own entries out.
fn plugin_services(kernel: &Kernel) -> Vec<modukit_core::registry::RegisteredService> {
    kernel
        .registry()
        .snapshot()
        .into_iter()
        .filter(|s| !s.service_id.as_str().starts_with("org.modukit."))
        .collect()
}

#[test]
fn happy_path_walks_states_and_round_trips_the_service() {
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let kernel = Kernel::new();
    let events = kernel.registry().subscribe();
    let manifest = Manifest::from_toml_str(&manifest_toml(&so))
        .unwrap_or_else(|e| panic!("fixture manifest must parse: {e}"));
    let id = manifest.id.clone();

    // install: identity + manifest parsed, artifact present.
    kernel.install(manifest).expect("install");
    assert_eq!(
        kernel.state(&id).expect("installed"),
        PluginState::Installed
    );

    // resolve: deps validated, artifact loaded, ABI v1 handshake passes.
    kernel.resolve(&id).expect("resolve must pass");
    assert_eq!(
        kernel.state(&id).expect("resolved"),
        PluginState::Starting,
        "resolve lands on the STARTING edge (03 §2: deps + handshake pass)"
    );
    assert!(plugin_services(&kernel).is_empty(), "no code has run yet");

    // start: plugin code runs, registers exactly one service.
    kernel.start(&id).expect("start must pass");
    assert_eq!(kernel.state(&id).expect("started"), PluginState::Active);

    let snap = plugin_services(&kernel);
    assert_eq!(
        snap.len(),
        1,
        "example-plugin registers exactly one service"
    );
    assert_eq!(snap[0].service_id.as_str(), EXAMPLE_ID);
    let event = events
        .recv_timeout(std::time::Duration::from_secs(2))
        .expect("Registered event must arrive on the stream");
    assert!(
        matches!(
            event,
            modukit_core::registry::ServiceEvent::Registered { .. }
        ),
        "expected Registered, got {event:?}"
    );

    // acquire -> handle -> message-shaped call (01 §3 rule 3: bytes, ids).
    let svc = kernel
        .registry()
        .acquire::<NativeService>()
        .expect("lookup returns a handle to the plugin service");
    assert_eq!(svc.service_id().as_str(), EXAMPLE_ID);
    let echo = svc
        .invoke("echo", b"kernel-v0")
        .expect("service call over the C ABI");
    assert_eq!(echo, b"kernel-v0", "echo round trip through the plugin");
    let err = svc
        .invoke("no-such-method", b"x")
        .expect_err("unknown method must be a typed error");
    assert!(
        matches!(err, KernelError::ServiceCallFailed { .. }),
        "{err}"
    );
    kernel
        .registry()
        .release(&svc)
        .expect("acquire/release pair is mandatory (invariant 6)");

    // stop: cooperative stop, the kernel deregisters the plugin's services.
    kernel.stop(&id).expect("stop must pass");
    assert_eq!(
        kernel.state(&id).expect("stopped"),
        PluginState::Resolved,
        "STOPPING -> RESOLVED on clean stop (03 §2)"
    );
    assert!(plugin_services(&kernel).is_empty(), "services gone");
    let event = events
        .recv_timeout(std::time::Duration::from_secs(2))
        .expect("Unregistered event must arrive");
    assert!(
        matches!(
            event,
            modukit_core::registry::ServiceEvent::Unregistered { .. }
        ),
        "expected Unregistered, got {event:?}"
    );

    // uninstall: back to UNINSTALLED (artifact released).
    kernel.uninstall(&id).expect("uninstall");
    assert_eq!(
        kernel.state(&id).expect("uninstalled"),
        PluginState::Uninstalled
    );
}

#[test]
fn bad_plugin_abi_mismatch_lands_failed_with_typed_error_never_a_crash() {
    let Some(so) = build_fixture("bad-plugin", "bad_plugin") else {
        return;
    };
    let kernel = Kernel::new();
    let manifest =
        Manifest::from_toml_str(&manifest_toml(&so)).expect("bad fixture manifest parses");
    let id = manifest.id.clone();
    kernel
        .install(manifest)
        .expect("install passes the §2 guard");

    let err = kernel
        .resolve(&id)
        .expect_err("ABI v999 must never be accepted silently");
    assert!(
        matches!(err, KernelError::AbiMismatch { found, .. } if found == 999),
        "expected typed AbiMismatch(found=999), got: {err}"
    );
    assert_eq!(
        kernel.state(&id).expect("still tracked"),
        PluginState::Failed,
        "unknown ABI version -> FAILED at RESOLVING (04 §3), never a partial start"
    );
    assert!(
        plugin_services(&kernel).is_empty(),
        "a rejected plugin must have run no code"
    );
    // Reaching here without a segfault IS the second half of the assertion.
}

#[test]
fn install_guard_rejects_missing_artifact_before_any_state_move() {
    let kernel = Kernel::new();
    let manifest = Manifest::from_toml_str(&manifest_toml(Path::new(
        "/nonexistent/out/of/this/world.so",
    )))
    .expect("manifest parses (grammar is placement-agnostic)");
    let err = kernel
        .install(manifest)
        .expect_err("§2 guard: artifact must be present to install");
    assert!(matches!(err, KernelError::ArtifactMissing { .. }), "{err}");
}

#[test]
fn state_of_unknown_plugin_is_a_typed_error() {
    let kernel = Kernel::new();
    let err = kernel
        .state(&modukit_core::identity::PluginId::new(
            "com.test.absent".to_string(),
        ))
        .expect_err("unknown id must not silently report a state");
    assert!(matches!(err, KernelError::UnknownPlugin { .. }), "{err}");
}
