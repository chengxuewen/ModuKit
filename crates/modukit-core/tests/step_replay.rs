//! S2-M3b D74: the supervisor-driven clock dial over the control socket, and
//! the D17 test-2 headline it pays for — step-replay CONTENT equality across
//! placements: the same replay scenario stepped by `TestHost::advance` on the
//! `dl` arm and by wire dials on the `exe` arm must produce byte-equal
//! due-job transcripts (tokens, labels, fire stamps). Production Real stays
//! undialable: the runner refuses a dial on a Real embedded clock with a
//! typed refusal (Verdicts A1 = "dial only valid when Step/Sim").
//!
//! Fixture-build discipline mirrors dual_placement.rs (cdylibs are built
//! inside the test; a runner build failure is a hard red). No /tmp scratch
//! besides the refusal-test wrapper, which is cleaned up in-test.

// D104 T2.4: test-scaffolding file; the clippy.toml allow-*-in-tests valve
// exempts #[test] fn bodies only, not plain helper fns in a tests/ target
// (clippy 0.1.99), so the exemption lands here. Production stays denied.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::error::KernelError;
use modukit_core::events::ObsEvent;
use modukit_core::executor::Supervisor;
use modukit_core::manifest::Manifest;
use modukit_core::testhost::{TestHost, schedule_replay_markers};

const ID: &str = "com.example.modukit.demo";

/// The replay transcript shape per dial turn: the clock stamp the turn
/// completed at, and the due jobs it fired (token, label).
type Turn = (u64, Vec<(u64, String)>);

/// Turn sequence: a no-due turn, one crossing `replay-alpha` (deadline 1 s),
/// one crossing `replay-beta` (deadline 3 s). Expected stamps: 0.5 s, 1.5 s,
/// 4.0 s — identical on both arms because both clocks are Step from zero.
const TURNS: &[Duration] = &[
    Duration::from_millis(500),
    Duration::from_millis(1000),
    Duration::from_millis(2500),
];

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

fn build_example_so(tag: &str) -> Option<PathBuf> {
    if std::env::var_os("MODUKIT_SKIP_DLOPEN").is_some() {
        eprintln!("SKIPPING: MODUKIT_SKIP_DLOPEN is set");
        return None;
    }
    let so = target_dir().join("debug").join("libexample_plugin.so");
    match cargo_build(&["build", "-p", "example-plugin"]) {
        Some(true) => {}
        Some(false) => {
            eprintln!("SKIPPING: fixture build failed ({tag})");
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

fn build_runner() -> PathBuf {
    let bin = target_dir().join("debug").join("modukit-run");
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

/// Arm (a): the `dl` placement stepped via the D24 testhost advance.
/// The transcript is folded out of the merged event ring (TimerDone
/// records), the same stream every consumer reads.
fn dl_transcript(so: &Path) -> Vec<Turn> {
    let host = TestHost::new();
    let kernel = host.kernel();
    let manifest = Manifest::from_toml_str(&dl_manifest_toml(so)).expect("dl manifest parses");
    let id = manifest.id.clone();
    kernel.install(manifest).expect("dl install");
    kernel.resolve(&id).expect("dl resolve");
    kernel.start(&id).expect("dl start");
    schedule_replay_markers(kernel.clock());

    let mut transcript = Vec::new();
    for turn in TURNS {
        let cursor = host.head();
        let _posted = host.advance(*turn);
        let fired: Vec<(u64, (u64, String))> = host
            .events_since(cursor)
            .events
            .iter()
            .filter_map(|r| match &r.event {
                ObsEvent::TimerDone { token, job } => Some((r.ts.raw(), (*token, job.clone()))),
                _ => None,
            })
            .collect();
        // Group under the stamp of the turn: same-clock records share the
        // crossed-dial stamp; take the fire list with the turn's stamp.
        let stamp = fired.last().map(|(ts, _)| *ts).unwrap_or_else(|| {
            // No-due turn: the dial still moved — read the clock back.
            kernel.clock().now().raw()
        });
        transcript.push((stamp, fired.into_iter().map(|(_, job)| job).collect()));
    }
    kernel.stop(&id).expect("dl stop");
    kernel.uninstall(&id).expect("dl uninstall");
    transcript
}

/// Arm (b): the `exe` placement stepped via supervisor-driven wire dials
/// (D74: the supervisor is the SOLE authority turning the child's clock).
fn exe_transcript(so: &Path, runner: &Path) -> Vec<Turn> {
    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
    let manifest =
        Manifest::from_toml_str(&exe_manifest_toml(so, runner)).expect("exe manifest parses");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("exe install");
    sup.kernel().resolve(&id).expect("exe resolve");
    sup.kernel().start(&id).expect("exe start");

    let mut transcript = Vec::new();
    for turn in TURNS {
        let reply = sup
            .dial(&id, *turn)
            .unwrap_or_else(|e| panic!("dial turn must answer: {e}"));
        transcript.push((
            reply.fired_at_ns,
            reply
                .jobs
                .into_iter()
                .map(|j| (j.token().value(), j.job().to_string()))
                .collect(),
        ));
    }

    sup.kernel().stop(&id).expect("exe stop");
    sup.kernel().uninstall(&id).expect("exe uninstall");
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "the runner exited cooperatively, no supervised crash"
    );
    transcript
}

#[test]
fn step_replay_transcripts_are_byte_equal_across_placements_via_the_wire_dial() {
    // Arrange
    let Some(so) = build_example_so("s2m3b-replay") else {
        return;
    };
    let runner = build_runner();

    // Act
    let dl = dl_transcript(&so);
    let exe = exe_transcript(&so, &runner);

    // Assert: the scenario is pinned concretely first — equality alone
    // could be two empty transcripts (a falsifiable D17 test-2, not a
    // tautology). Tokens 1/2 are the markers' schedule order on both clocks.
    let expected: Vec<Turn> = vec![
        (500_000_000, vec![]),
        (1_500_000_000, vec![(1u64, "replay-alpha".to_string())]),
        (4_000_000_000, vec![(2u64, "replay-beta".to_string())]),
    ];
    assert_eq!(dl, expected, "dl arm stepped via testhost advance");
    assert_eq!(
        dl, exe,
        "D17 test-2: step-replay content equality — exe stepped via the D74 wire dial"
    );
}

#[test]
fn dial_on_a_real_domain_child_is_refused_typed_not_silently_turned() {
    // Arrange: a runner wrapper that opts the embedded clock into the
    // production Real domain (the only shape where the dial is refused).
    let Some(so) = build_example_so("s2m3b-real") else {
        return;
    };
    let runner = build_runner();
    let dir = std::env::temp_dir().join(format!("mk-dial-real-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let wrapper = dir.join("real-runner");
    std::fs::write(
        &wrapper,
        format!("#!/bin/sh\nexec {} \"$@\" --clock=real\n", runner.display()),
    )
    .expect("write wrapper");
    std::fs::set_permissions(
        &wrapper,
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )
    .expect("chmod");

    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
    let manifest =
        Manifest::from_toml_str(&exe_manifest_toml(&so, &wrapper)).expect("exe manifest parses");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("install");
    sup.kernel().resolve(&id).expect("resolve");
    sup.kernel().start(&id).expect("start");

    // Act + Assert: the refusal is typed, stable (retrying the dial on a
    // Real clock cannot help — the dial does not exist there), and the
    // child stays cooperative (it is alive, just undialable).
    let err = sup
        .dial(&id, Duration::from_millis(500))
        .expect_err("Real domain has no dial (D74)");
    assert!(
        matches!(err, KernelError::ClockDialRefused { status: -2 }),
        "expected typed ClockDialRefused(-2), got {err}"
    );
    let again = sup
        .dial(&id, Duration::from_millis(500))
        .expect_err("refusal is stable, not transient");
    assert!(matches!(again, KernelError::ClockDialRefused { .. }));

    sup.kernel().stop(&id).expect("stop still cooperative");
    sup.kernel().uninstall(&id).expect("uninstall");
    assert_eq!(sup.poll(), Vec::<String>::new());
    let _ = std::fs::remove_dir_all(&dir);
    assert!(
        !dir.exists(),
        "scratch cleanup failed — tests must not leak /tmp dirs"
    );
}
