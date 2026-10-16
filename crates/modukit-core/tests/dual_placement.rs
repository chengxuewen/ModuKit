//! S2-M2 dual-placement contract-suite run (modules/01 §3: "switchability is
//! a tested property", modules/04 §4: "one suite, two placements"). The SAME
//! `example-plugin` cdylib fixture (read-only template per D23) is driven
//! through BOTH executor arms of the closed set:
//!
//! * `dl`  — the kernel with `DlopenExecutor` (the M3 path, assertions' shape
//!   mirrored from tests/dlopen_kernel.rs).
//! * `exe` — the kernel with `ProcessExecutor` + `Supervisor` driving the REAL
//!   `modukit-run` shadow-runner from `crates/modukit-runtime` (the M1 path,
//!   mirrored from tests/process_executor.rs, but with the runner instead of a
//!   stand-in executable).
//!
//! Mirror-case table (each case runs under both arms, kernel-observable):
//!  1. resolve/handshake green -> ACTIVE, identical lifecycle walk
//!  2. self-report tier: manifest id/version accepted by both arms
//!     (dl: descriptor, exe: strict HELLO cross-check)
//!  3. service call echo, D69 bytes-only: dl observes the `NativeService`
//!     round trip through the KERNEL registry; exe the kernel-side PROXY
//!     service (installed from the runner's REGISTER frames, S2-M3a)
//!     dispatches CALL/RESULT across the boundary through the SAME kernel
//!     door — byte-equal results and byte-equal unknown-method rejection
//!     (modules/01 §3 rule 2 stands: the host table never crosses; the wire
//!     carries message bytes, ids and method names only)
//!  4. cooperative stop lands RESOLVED, services gone
//!  5. bad-plugin fixture refused: dl `AbiMismatch(999)`, exe the runner
//!     refuses at its own dlopen guards and dies before HELLO — the parent
//!     parks FAILED promptly: the runner dies before HELLO, the parent reads
//!     EOF and classifies it as ProcessCrashed (F1 fixed in S2-M2 follow-up:
//!     the spawn-time descriptor retention is gone)
//!
//! Fixture builds follow the dlopen_kernel.rs in-test pattern (cdylibs are not
//! built by `cargo test`). A runner-binary build failure is FATAL here, not a
//! skip: it is a workspace member, so its inability to build is a real red.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use modukit_core::abi::NativeService;
use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::error::KernelError;
use modukit_core::executor::ProxyService;
use modukit_core::executor::Supervisor;
use modukit_core::kernel::Kernel;
use modukit_core::lifecycle::PluginState;
use modukit_core::manifest::Manifest;

const ID: &str = "com.example.modukit.demo";
const PROBE: &[u8] = b"dual-placement-probe";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace layout: crates/modukit-core")
        .to_path_buf()
}

fn target_dir() -> PathBuf {
    repo_root().join("out").join("dlopen-test-target")
}

/// Build one workspace package into the shared test target dir (the
/// dlopen_kernel.rs pattern). `None` = build environment unavailable (skip).
fn cargo_build(args: &[&str]) -> Option<bool> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let dir = target_dir();
    let status = Command::new(&cargo)
        .current_dir(repo_root())
        .args(args)
        .arg("--target-dir")
        .arg(&dir)
        .env("CARGO_TARGET_DIR", &dir)
        .status();
    match status {
        Ok(s) => Some(s.success()),
        Err(e) => {
            eprintln!("SKIPPING: cannot spawn cargo: {e}");
            None
        }
    }
}

/// The example-plugin fixture cdylib (built, never modified).
fn build_example_so(tag: &str) -> Option<PathBuf> {
    if std::env::var_os("MODUKIT_SKIP_DLOPEN").is_some() {
        eprintln!("SKIPPING: MODUKIT_SKIP_DLOPEN is set");
        return None;
    }
    let so = target_dir().join("debug").join("libexample_plugin.so");
    let stem = format!("dual-{tag}");
    match cargo_build(&["build", "-p", "example-plugin", "-p", "bad-plugin"]) {
        Some(true) => {}
        Some(false) => {
            eprintln!("SKIPPING: fixture build failed ({stem})");
            return None;
        }
        None => return None,
    }
    if so.is_file() {
        Some(so)
    } else {
        eprintln!("SKIPPING: expected artifact {} absent", so.display());
        None
    }
}

/// The shadow-runner binary. A workspace member that will not build is a
/// hard failure, not a skip (this IS the S2-M2 deliverable under test).
fn build_runner() -> PathBuf {
    let bin = target_dir().join("debug").join("modukit-run");
    // Always run cargo (incremental makes the no-op cheap); a stale-binary
    // shortcut here once served the pre-handshake skeleton as "green".
    match cargo_build(&["build", "-p", "modukit-runtime"]) {
        Some(true) => {}
        Some(false) => panic!("modukit-runtime must build (workspace member)"),
        None => panic!("cargo must be spawnable to build the runner"),
    }
    assert!(bin.is_file(), "runner binary missing at {}", bin.display());
    bin
}

fn dl_manifest_toml(artifact: &Path) -> String {
    format!(
        "id = \"{ID}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"dl\"\nartifact = \"{}\"\n",
        artifact.display()
    )
}

fn exe_manifest_toml(artifact: &Path, runner: &Path) -> String {
    format!(
        "id = \"{ID}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"exe\"\nartifact = \"{}\"\nrunner = \"{}\"\n",
        artifact.display(),
        runner.display()
    )
}

/// Lifecycle `from->to` strings for the tracked plugin (process_executor.rs
/// helper shape), taken from either arm's kernel.
fn lifecycle_walk(kernel: &Kernel) -> Vec<String> {
    kernel
        .observatory()
        .tail(0)
        .events
        .iter()
        .filter_map(|r| match &r.event {
            modukit_core::events::ObsEvent::Lifecycle {
                plugin, from, to, ..
            } if plugin == ID => Some(format!("{from}->{to}")),
            _ => None,
        })
        .collect()
}

/// Plugin-visible services only (D40 platform services filtered, the
/// dlopen_kernel.rs helper shape).
fn plugin_services(kernel: &Kernel) -> Vec<modukit_core::registry::RegisteredService> {
    kernel
        .registry()
        .snapshot()
        .into_iter()
        .filter(|s| !s.service_id.as_str().starts_with("org.modukit."))
        .collect()
}

/// Arm (a): the kernel with the `dl` executor walks the full lifecycle on the
/// real fixture. Returns the observable walk for the cross-arm comparison.
fn dl_arm(so: &Path) -> (Vec<String>, Vec<u8>, Vec<String>) {
    let kernel = Kernel::new();
    let manifest = Manifest::from_toml_str(&dl_manifest_toml(so)).expect("dl manifest parses");
    let id = manifest.id.clone();
    kernel.install(manifest).expect("dl install");
    kernel.resolve(&id).expect("dl resolve");
    kernel.start(&id).expect("dl start");

    let svc = kernel
        .registry()
        .acquire::<NativeService>()
        .expect("dl echo service in the kernel registry");
    let echo = svc.invoke("echo", PROBE).expect("dl echo round trip");
    let bad = svc
        .invoke("no-such-method", b"x")
        .err()
        .map(|e| e.code().to_string())
        .unwrap_or_default();
    kernel.registry().release(&svc).expect("dl release pair");

    kernel.stop(&id).expect("dl stop");
    kernel.uninstall(&id).expect("dl uninstall");
    (lifecycle_walk(&kernel), echo, vec![bad])
}

/// Arm (b): the kernel with `ProcessExecutor` + `Supervisor` driving the real
/// `modukit-run`. Returns the observable walk plus start/stop outcomes.
fn exe_arm(so: &Path, runner: &Path) -> (Vec<String>, Vec<u8>, Vec<String>) {
    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
    let manifest =
        Manifest::from_toml_str(&exe_manifest_toml(so, runner)).expect("exe manifest parses");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("exe install");
    sup.kernel().resolve(&id).expect("exe spawn + HELLO");
    sup.kernel()
        .start(&id)
        .expect("START ack 0: the runner round-tripped the echo service");
    // S2-M3a: the runner announced its plugin's service via REGISTER and
    // the executor installed a kernel-observable proxy. The host table
    // never crossed (01 §3 rule 2) — CALL/RESULT ride message bytes only.
    let svc = plugin_services(sup.kernel());
    assert_eq!(svc.len(), 1, "the proxy service is kernel-observable");
    assert_eq!(svc[0].service_id.as_str(), ID, "canonical id (D9)");
    let proxy = sup
        .kernel()
        .registry()
        .acquire::<ProxyService>()
        .expect("exe proxy installed through the registry door");
    assert_eq!(
        proxy.methods(),
        ["echo"],
        "REGISTER carried the method list"
    );
    let echo = proxy
        .invoke("echo", PROBE)
        .expect("exe echo round trip across the boundary");
    let bad = proxy
        .invoke("no-such-method", b"x")
        .err()
        .map(|e| e.code().to_string())
        .unwrap_or_default();
    if sup.kernel().registry().release(&proxy).is_err() {
        panic!("proxy release pair");
    }
    sup.kernel()
        .stop(&id)
        .expect("STOP acked, graceful teardown (proxy unregistered)");
    sup.kernel().uninstall(&id).expect("exe uninstall");
    assert_eq!(
        plugin_services(sup.kernel()),
        Vec::<modukit_core::registry::RegisteredService>::new(),
        "proxy gone after stop — same service lifecycle as the dl arm"
    );
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "the runner exited cooperatively, no supervised crash"
    );
    (lifecycle_walk(sup.kernel()), echo, vec![bad])
}

#[test]
fn full_lifecycle_walk_is_identical_under_both_placements() {
    let Some(so) = build_example_so("walk") else {
        return;
    };
    let runner = build_runner();
    let (dl_walk, dl_echo, dl_notes) = dl_arm(&so);
    let (exe_walk, exe_echo, exe_notes) = exe_arm(&so, &runner);

    assert_eq!(dl_echo, PROBE, "dl: bytes round trip via NativeService");
    assert_eq!(
        dl_echo, exe_echo,
        "F2 closed: byte-equal echo across placements — dl direct, exe via the proxy"
    );
    assert_eq!(
        dl_walk,
        vec![
            "UNINSTALLED->INSTALLED",
            "INSTALLED->RESOLVING",
            "RESOLVING->STARTING",
            "STARTING->ACTIVE",
            "ACTIVE->STOPPING",
            "STOPPING->RESOLVED",
            "RESOLVED->UNINSTALLED",
        ],
        "dl walk (mirrored from dlopen_kernel.rs)"
    );
    assert_eq!(dl_walk, exe_walk, "dual-placement lifecycle-walk equality");
    // D51 v0 mapping: ServiceCallFailed codes as CONTRACT_VIOLATION (the 14-name
    // freeze is the S2-M5 hearing agenda), so mirror the typed variant, not a
    // code string — same check shape as dlopen_kernel.rs.
    assert_eq!(dl_notes, vec!["CONTRACT_VIOLATION".to_string()]);
    assert_eq!(
        dl_notes, exe_notes,
        "unknown-method rejection byte-equal too (typed passthrough, as the dl arm)"
    );
}

#[test]
fn self_report_tier_agrees_with_the_manifest_under_both_placements() {
    let Some(so) = build_example_so("selfreport") else {
        return;
    };
    let runner = build_runner();

    // dl: the descriptor's own self-report (D7 tier) is what the M7
    // cross-check reads; probe it through a plain DlopenExecutor load.
    use modukit_core::executor::{DlopenExecutor, Executor};
    let manifest = Manifest::from_toml_str(&dl_manifest_toml(&so)).expect("dl manifest parses");
    let handle = DlopenExecutor::new().load(&manifest).expect("dl load");
    let dl_report = handle.self_report();
    assert_eq!(
        dl_report,
        (Some(ID.to_string()), Some("0.1.0".to_string())),
        "dl descriptor self-report"
    );
    assert_eq!(handle.plugin_name(), "example-plugin");
    drop(handle);

    // exe: the same strings must arrive in the HELLO the runner sends — a
    // manifest mismatch would abort the handshake (strict cross-check), so
    // reaching ACTIVE is the equality proof.
    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
    let em =
        Manifest::from_toml_str(&exe_manifest_toml(&so, &runner)).expect("exe manifest parses");
    let id = em.id.clone();
    sup.kernel().install(em).expect("install");
    sup.kernel()
        .resolve(&id)
        .expect("HELLO accepted: id+version agree");
    assert_eq!(sup.kernel().counters().load_ok(), 1);
    sup.kernel().start(&id).expect("start ack 0");
    sup.kernel().stop(&id).expect("stop");
}

#[test]
fn wrong_version_manifest_is_refused_under_both_placements() {
    let Some(so) = build_example_so("badver") else {
        return;
    };
    let runner = build_runner();

    // dl: the D7 cross-check lives in the bundle-install door (M7); at the
    // executor level a wrong manifest version is NOT refused for `dl`
    // placement (handshake checks ABI, not semver). Mirror that exact
    // asymmetry: exe refuses (strict HELLO), dl accepts — the contract suite
    // documents it so the switch is tested, not assumed.
    let bad_dl = format!(
        "id = \"{ID}\"\nversion = \"9.9.9\"\n[entry]\nkind = \"dl\"\nartifact = \"{}\"\n",
        so.display()
    );
    let kernel = Kernel::new();
    let m = Manifest::from_toml_str(&bad_dl).expect("parses");
    let id = m.id.clone();
    kernel.install(m).expect("install");
    kernel.resolve(&id).expect("dl handshake checks ABI only");
    assert_eq!(kernel.state(&id).expect("state"), PluginState::Starting);

    // exe: the HELLO self-report says 0.1.0, the manifest says 9.9.9 — the
    // strict cross-check must refuse with the typed disagreement.
    let bad_exe = format!(
        "id = \"{ID}\"\nversion = \"9.9.9\"\n[entry]\nkind = \"exe\"\nartifact = \"{}\"\nrunner = \"{}\"\n",
        so.display(),
        runner.display()
    );
    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
    let m = Manifest::from_toml_str(&bad_exe).expect("parses");
    let id = m.id.clone();
    sup.kernel().install(m).expect("install");
    let err = sup.kernel().resolve(&id).expect_err("version lie refused");
    assert!(
        matches!(&err, KernelError::IdentityMismatch { what, .. } if *what == "version"),
        "expected typed IdentityMismatch(version), got {err}"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("tracked"),
        PluginState::Failed
    );
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "the rejected child was reaped at load, no zombie, no phantom"
    );
}

#[test]
fn cooperative_stop_unregisters_services_under_both_placements() {
    let Some(so) = build_example_so("stop") else {
        return;
    };
    let runner = build_runner();

    // dl arm: service appears between start and stop, is gone after both.
    let kernel = Kernel::new();
    let m = Manifest::from_toml_str(&dl_manifest_toml(&so)).expect("parses");
    let id = m.id.clone();
    kernel.install(m).expect("install");
    kernel.resolve(&id).expect("resolve");
    assert!(plugin_services(&kernel).is_empty(), "no code ran yet");
    kernel.start(&id).expect("start");
    assert_eq!(plugin_services(&kernel).len(), 1, "echo registered");
    kernel.stop(&id).expect("stop");
    assert!(plugin_services(&kernel).is_empty(), "services gone");
    assert_eq!(
        kernel.state(&id).expect("state"),
        PluginState::Resolved,
        "cooperative stop lands RESOLVED (03 §2)"
    );

    // exe arm (S2-M3a): the proxy mirrors the same invariant — present
    // while ACTIVE (installed from the runner's REGISTER frames), gone
    // after a cooperative stop, which lands RESOLVED with a reaped child.
    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
    let m = Manifest::from_toml_str(&exe_manifest_toml(&so, &runner)).expect("parses");
    let id = m.id.clone();
    sup.kernel().install(m).expect("install");
    sup.kernel().resolve(&id).expect("resolve");
    sup.kernel()
        .start(&id)
        .expect("start: the runner REGISTERed its service before the ack");
    assert_eq!(plugin_services(sup.kernel()).len(), 1, "proxy installed");
    sup.kernel()
        .stop(&id)
        .expect("stop acked, proxy unregistered");
    assert_eq!(
        sup.kernel().state(&id).expect("state"),
        PluginState::Resolved,
        "exe cooperative stop lands RESOLVED too"
    );
    assert!(plugin_services(sup.kernel()).is_empty(), "proxy gone");
    assert_eq!(sup.poll(), Vec::<String>::new());
}

#[test]
fn bad_abi_fixture_is_refused_under_both_placements() {
    let Some(_fixture) = build_example_so("badabi") else {
        return;
    };
    // The bad-plugin fixture (ABI v999) — built alongside example-plugin in
    // build_example_so; locate it in the same debug dir.
    let bad_so = target_dir().join("debug").join("libbad_plugin.so");
    assert!(bad_so.is_file(), "bad fixture cdylib missing");
    let runner = build_runner();

    // dl arm: typed AbiMismatch at RESOLVING (mirrors dlopen_kernel.rs).
    let kernel = Kernel::new();
    let m = Manifest::from_toml_str(&{
        let mut t = dl_manifest_toml(&bad_so);
        t = t.replace(
            "id = \"com.example.modukit.demo\"",
            "id = \"com.example.modukit.bad\"",
        );
        t
    })
    .expect("parses");
    let id = m.id.clone();
    kernel.install(m).expect("install");
    let err = kernel.resolve(&id).expect_err("v999 never accepted");
    assert!(
        matches!(err, KernelError::AbiMismatch { found, .. } if found == 999),
        "dl: {err}"
    );
    assert_eq!(kernel.state(&id).expect("state"), PluginState::Failed);

    // exe arm: the runner's OWN guards refuse the artifact before it can
    // speak a HELLO truth — the child dies at setup, the parent observes the
    // closed control as a typed crash and lands FAILED. Both arms: zero
    // plugin code executed, no supervised run, no zombie.
    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
    let m = Manifest::from_toml_str(&{
        let mut t = exe_manifest_toml(&bad_so, &runner);
        t = t.replace(
            "id = \"com.example.modukit.demo\"",
            "id = \"com.example.modukit.bad\"",
        );
        t
    })
    .expect("parses");
    let id = m.id.clone();
    sup.kernel().install(m).expect("install");
    let err = sup
        .kernel()
        .resolve(&id)
        .expect_err("runner refuses v999 before HELLO");
    // Post-F1-fix parent behavior: the injected child control-end is
    // dropped with the Command at spawn, so the silent death closes the
    // last peer reference and the parent reads EOF -> the ProcessCrashed
    // arm, promptly (the regression bound is proven by silent_death_*).
    assert!(
        matches!(err, KernelError::ProcessCrashed { .. }),
        "exe: {err}"
    );
    assert_eq!(sup.kernel().state(&id).expect("state"), PluginState::Failed);
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "reaped at load, never entered the live table"
    );
}

/// Loose CI bound for the EOF classification — the F1 retention bug made
/// the read end only at `HANDSHAKE_TIMEOUT` (5s), the fixed path returns in
/// microseconds. The bound doubles as the fd-retention probe: a stray
/// parent-side copy of the child control-end keeps the pair open for the
/// WHOLE timeout, and it is invisible to the count-based hygiene probes
/// because the leaked descriptor number is recycled by the next `pair()`.
const SILENT_DEATH_BOUND: Duration = Duration::from_secs(1);

#[test]
fn silent_death_before_hello_is_eof_processcrashed_not_handshake_timeout() {
    // F1 regression (S2-M2 escalation): the parent must drop the injected
    // child control-end (Stdio-held descriptor) BEFORE reading HELLO, so a
    // child that dies without ever speaking yields EOF -> the ProcessCrashed
    // arm promptly. The silent child here is a shebang script that exits
    // immediately (no runner involved: the defect is parent-side). M1 never
    // reached the EOF arm because every stand-in speaks HELLO first.
    let dir = std::env::temp_dir().join(format!("mk-silent-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let script = dir.join("silent-child");
    std::fs::write(&script, "#!/bin/sh\nexit 4\n").expect("write script");
    std::fs::set_permissions(&script, std::os::unix::fs::PermissionsExt::from_mode(0o755))
        .expect("chmod");

    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
    let manifest = Manifest::from_toml_str(&format!(
        "id = \"com.example.proc.silent\"\nversion = \"0.1.0\"\n[entry]\nkind = \"exe\"\nartifact = \"{}\"\n",
        script.display()
    ))
    .expect("manifest parses");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("install");

    let at = Instant::now();
    let err = sup
        .kernel()
        .resolve(&id)
        .expect_err("a silent child must be refused");
    let waited = at.elapsed();

    assert!(matches!(err, KernelError::ProcessCrashed { .. }), "{err}");
    assert!(
        waited < SILENT_DEATH_BOUND,
        "EOF classification waited {waited:?}: the read ended at the timeout, meaning the parent still held the child control-end copy (F1 regression)"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("tracked"),
        PluginState::Failed,
        "FAILED edge via the resolve error arm"
    );
    assert_eq!(sup.kernel().counters().load_fail(), 1);
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "killed+reaped at the load doorway, never entered the live table"
    );
}
