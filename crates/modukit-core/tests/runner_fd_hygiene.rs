//! S2-M2 fd hygiene for the REAL shadow-runner path (extends the S2-M1
//! probe in `proc_fd_hygiene.rs` from stand-in executables to
//! `modukit-run` + the genuine example-plugin cdylib). Own test file = own
//! process (the M1 rationale), so the descriptor count is perturbed only by
//! this file's own tests.
//!
//! Coverage: spawn/handshake/start/stop/uninstall cycles against the real
//! runner (`/proc/self/fd` before == after), one silent-death cycle (the
//! bad-plugin: the child dies before HELLO, the parent reads EOF and
//! classifies ProcessCrashed (post-F1-fix), reaping via `kill_wait_child` —
//! zombie may survive that path either), and the zero-zombie scan for the
//! whole process.

// D104 T2.4: test-scaffolding file; the clippy.toml allow-*-in-tests valve
// exempts #[test] fn bodies only, not plain helper fns in a tests/ target
// (clippy 0.1.99), so the exemption lands here. Production stays denied.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::executor::Supervisor;
use modukit_core::manifest::Manifest;

const ID: &str = "com.example.modukit.demo";
const BAD_ID: &str = "com.example.modukit.bad";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace layout: crates/modukit-core")
        .to_path_buf()
}

fn build_cargo(args: &[&str]) -> Option<bool> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let dir = repo_root().join("out").join("dlopen-test-target");
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

/// (artifact cdylib, runner binary) — fixture builds skip green when the
/// environment cannot run cargo (the dlopen_kernel.rs discipline); a runner
/// build failure is a hard red (workspace member).
fn build_pairs() -> Option<(PathBuf, PathBuf)> {
    let debug = repo_root()
        .join("out")
        .join("dlopen-test-target")
        .join("debug");
    let so = debug.join("libexample_plugin.so");
    let bad = debug.join("libbad_plugin.so");
    let bin = debug.join("modukit-run");
    match build_cargo(&["build", "-p", "example-plugin", "-p", "bad-plugin"]) {
        Some(true) => {}
        Some(false) => {
            eprintln!("SKIPPING: fixture build failed");
            return None;
        }
        None => return None,
    }
    if !(so.is_file() && bad.is_file()) {
        eprintln!("SKIPPING: fixture artifacts absent");
        return None;
    }
    match build_cargo(&["build", "-p", "modukit-runtime"]) {
        Some(true) => {}
        Some(false) => panic!("modukit-runtime must build (workspace member)"),
        None => panic!("cargo must be spawnable to build the runner"),
    }
    assert!(bin.is_file(), "runner binary missing at {}", bin.display());
    Some((so, bin))
}

fn runner_manifest(artifact: &Path, runner: &Path, id: &str) -> Manifest {
    Manifest::from_toml_str(&format!(
        "id = \"{id}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"exe\"\nartifact = \"{}\"\nrunner = \"{}\"\n\
         [restart]\npolicy = \"on-failure\"\nmax_retries = 2\nbackoff = \"1s..4s\"\n",
        artifact.display(),
        runner.display()
    ))
    .expect("runner manifest parses")
}

fn open_fd_count() -> usize {
    std::fs::read_dir("/proc/self/fd")
        .expect("/proc/self/fd")
        .count()
}

/// Zero-zombie scan for this process's children (the process_executor.rs
/// probe, compact re-shape): the supervisor reaps at one doorway, so by the
/// scan deadline every child must be gone, not `Z`.
fn assert_no_zombie_children() {
    let me = std::process::id().to_string();
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let mut zombies = Vec::new();
        for entry in std::fs::read_dir("/proc").expect("/proc") {
            let name = entry
                .expect("proc entry")
                .file_name()
                .to_string_lossy()
                .into_owned();
            if !name.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let Ok(stat) = std::fs::read_to_string(format!("/proc/{name}/stat")) else {
                continue;
            };
            let Some(tail) = stat.rsplit_once(") ") else {
                continue;
            };
            let fields: Vec<&str> = tail.1.split_whitespace().collect();
            if fields.len() >= 2 && fields[1] == me && fields[0] == "Z" {
                zombies.push(name);
            }
        }
        if zombies.is_empty() {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "zombie children survived the runner cycles: {zombies:?}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn real_runner_cycles_leak_no_descriptors_or_zombies() {
    let Some((so, runner)) = build_pairs() else {
        return;
    };
    // Builds and the supervisor boot happen before the baseline: only the
    // spawn/teardown cycles below are measured by the fd delta.
    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
    let clean = runner_manifest(&so, &runner, ID);
    let id = clean.id.clone();
    let before = open_fd_count();

    for _ in 0..3 {
        // Real runner cycle: spawn + HELLO + START (in-runner dlopen,
        // embedded kernel start, echo self-check) + ack, then STOP + ack +
        // cooperative exit + reap + uninstall (handle drop).
        sup.kernel().install(clean.clone()).expect("install");
        sup.kernel().resolve(&id).expect("resolve: runner HELLO");
        sup.kernel()
            .start(&id)
            .expect("start: runner echo self-check acked 0");
        sup.kernel().stop(&id).expect("stop: runner teardown acked");
        sup.kernel().uninstall(&id).expect("uninstall");
    }

    // Silent-death cycle through the kill_wait_child reap: the runner
    // refuses bad-plugin at its own dlopen guards and dies before HELLO;
    // the parent reads EOF (the F1 fix dropped its copy of the child
    // control-end at spawn) and must release every descriptor anyway.
    // NOTE (limitation worth recording): the count-based probe below and
    // in proc_fd_hygiene.rs cannot see descriptor RETENTION - a leaked fd
    // number is recycled by the next socketpair, keeping the count flat.
    // The deterministic retention proof is the sub-second EOF bound in
    // dual_placement.rs silent_death_before_hello_is_eof_processcrashed_*.
    let bad = runner_manifest(
        &repo_root()
            .join("out")
            .join("dlopen-test-target")
            .join("debug")
            .join("libbad_plugin.so"),
        &runner,
        BAD_ID,
    );
    let bad_id = bad.id.clone();
    sup.kernel().install(bad).expect("install bad");
    let err = sup
        .kernel()
        .resolve(&bad_id)
        .expect_err("silent-death child must be refused");
    assert!(
        matches!(err, modukit_core::error::KernelError::ProcessCrashed { .. }),
        "{err}"
    );
    assert_eq!(
        sup.kernel().state(&bad_id).expect("tracked"),
        modukit_core::lifecycle::PluginState::Failed
    );

    let after = open_fd_count();
    assert_eq!(
        after, before,
        "descriptor count grew across real-runner cycles: {before} -> {after}"
    );
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "the silent-death child was reaped at the load doorway"
    );
    assert_no_zombie_children();
}
