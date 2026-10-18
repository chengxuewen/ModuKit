//! S2-M1 process-placement tests: the `exe` executor branch over a unix
//! socketpair control channel (modules/01 §1-2), the supervisor's crash-watch
//! with the D3 restart policy, and the graceful-STOP-then-kill teardown
//! (modules/01 §3 rule 5).
//!
//! Written before the implementation (C0: red observed first). The child
//! stand-ins are tiny std-only Rust executables compiled in-test with rustc
//! (the runner binary `modukit-run` is S2-M2; these are the M1 test doubles).
//! The child's behavior is selected by its file stem, so the executor's argv
//! stays exactly the documented `--fd=N` convention.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::error::KernelError;
use modukit_core::executor::{DIAL_TIMEOUT, Revive, STOP_KILL_GRACE, Supervisor};
use modukit_core::identity::PluginId;
use modukit_core::lifecycle::PluginState;
use modukit_core::manifest::Manifest;

const ID: &str = "com.example.proc.stand-in";
const VERSION: &str = "0.1.0";

/// Real-process settle bound for the crash-watch poll (the child exits on
/// its own right after the START ack; the loop only waits for that exit).
const CRASH_POLL_DEADLINE: Duration = Duration::from_secs(5);

/// The in-test stand-in: one binary, behavior chosen by its file stem.
/// Speaks the same wire envelope as `proc_wire` (u32-LE payload length,
/// one wire-version byte, one type byte, payload) on the inherited control
/// fd (`--fd=N` per modules/01 §2; the host delivers it as fd 0 via stdio
/// inheritance). S2-M3 frame versioning: `VER` rides every frame; the
/// `ver-liar` stem frames its HELLO with a foreign version.
const STANDIN_SRC: &str = r#"
use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::os::unix::net::UnixStream;

const HELLO: u8 = 1;
const START: u8 = 2;
const STOP: u8 = 3;
const ACK: u8 = 4;
const VER: u8 = 1;

fn frame(kind: u8, payload: &[u8]) -> Vec<u8> {
    frame_ver(VER, kind, payload)
}

fn frame_ver(ver: u8, kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = (payload.len() as u32).to_le_bytes().to_vec();
    out.push(ver);
    out.push(kind);
    out.extend_from_slice(payload);
    out
}

fn hello_payload(abi: u32, id: &str, ver: &str, name: &str) -> Vec<u8> {
    let mut p = abi.to_le_bytes().to_vec();
    for s in [id, ver, name] {
        p.push(s.len() as u8);
        p.extend_from_slice(s.as_bytes());
    }
    p
}

fn read_all(sock: &mut UnixStream, buf: &mut [u8]) -> bool {
    let mut done = 0;
    while done < buf.len() {
        match sock.read(&mut buf[done..]) {
            Ok(0) | Err(_) => return false,
            Ok(n) => done += n,
        }
    }
    true
}

fn main() {
    let stem = std::env::current_exe()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_default();
    let mut fd_num: i32 = 0;
    for a in std::env::args().skip(1) {
        if let Some(rest) = a.strip_prefix("--fd=") {
            fd_num = rest.parse().unwrap_or(0);
        }
    }
    // SAFETY: the supervising host injected the control socket at this
    // inherited descriptor (the --fd=N convention, modules/01 2).
    let mut sock = unsafe { UnixStream::from_raw_fd(fd_num) };

    let mut abi: u32 = 1;
    let mut id = "com.example.proc.stand-in".to_string();
    if stem.contains("abi-liar") {
        abi = 999;
    }
    if stem.contains("id-liar") {
        id.push_str(".lie");
    }
    let hello = hello_payload(abi, &id, "0.1.0", "stand-in");
    // ver-liar: a foreign envelope version rides the HELLO frame itself —
    // the host must refuse the frame FORMAT, not the self-report (S2-M3).
    let hello_ver = if stem.contains("ver-liar") { 9 } else { VER };
    if sock.write_all(&frame_ver(hello_ver, HELLO, &hello)).is_err() {
        std::process::exit(1);
    }
    let _ = sock.flush();
    if stem.contains("liar") {
        // The host reads the lie from the socket buffer; the process
        // exit below closes the child end.
        std::process::exit(0);
    }

    let mut buf = [0u8; 6];
    loop {
        if !read_all(&mut sock, &mut buf) {
            std::process::exit(0); // parent hung up: leave cleanly
        }
        let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        let kind = buf[5]; // buf[4] is the envelope wire version
        let mut payload = vec![0u8; len];
        if len > 0 && !read_all(&mut sock, &mut payload) {
            std::process::exit(0);
        }
        match kind {
            START => {
                let _ = sock.write_all(&frame(ACK, &0i32.to_le_bytes()));
                let _ = sock.flush();
                if stem.contains("crasher") {
                    std::process::exit(7); // supervised death mid-run
                }
            }
            STOP => {
                if stem.contains("hung") {
                    continue; // never cooperates: the kill path exercises
                }
                let _ = sock.write_all(&frame(ACK, &0i32.to_le_bytes()));
                let _ = sock.flush();
                std::process::exit(0);
            }
            _ => {}
        }
    }
}
"#;

/// Compile the stand-in once per test into a private temp dir, then copy it
/// under the mode stems. `None` = cannot build here (test skips green with
/// the printed reason, the dlopen-fixture pattern).
/// Sweep leftover scratch dirs from DEAD previous runs (the 237-dir /tmp
/// exhaustion class, 2026-10-16): dirs embed their owner pid in the name;
/// a live pid means a concurrent test binary - never touch those.
fn sweep_dead_standins() {
    let me = std::process::id().to_string();
    let Ok(rd) = std::fs::read_dir(std::env::temp_dir()) else {
        return;
    };
    for entry in rd.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(rest) = name.strip_prefix("mk-standin") else {
            continue;
        };
        // names: mk-standin-<pid>-<tag> (process_executor) or mk-standin-fd-<pid>
        let pid = rest
            .trim_matches('-')
            .split('-')
            .find(|t| t.chars().all(|c| c.is_ascii_digit()) && !t.is_empty());
        let Some(pid) = pid else { continue };
        if pid == me {
            continue;
        }
        if std::path::Path::new(&format!("/proc/{pid}")).exists() {
            continue;
        }
        let _ = std::fs::remove_dir_all(entry.path());
    }
}

fn build_standins(tag: &str) -> Option<PathBuf> {
    sweep_dead_standins();
    let dir = std::env::temp_dir().join(format!("mk-standin-{}-{tag}", std::process::id()));
    let exe = dir.join("standin-good");
    if exe.is_file() {
        return Some(dir);
    }
    std::fs::create_dir_all(&dir).expect("temp dir");
    let src = dir.join("standin.rs");
    let mut f = std::fs::File::create(&src).expect("write standin.rs");
    f.write_all(STANDIN_SRC.as_bytes()).expect("source");
    drop(f);
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let out = Command::new(&rustc)
        .args(["--edition", "2024", "-o"])
        .arg(dir.join("standin-bin"))
        .arg(&src)
        .output();
    match out {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            eprintln!(
                "SKIPPING: standin build failed: {}",
                String::from_utf8_lossy(&o.stderr)
            );
            return None;
        }
        Err(e) => {
            eprintln!("SKIPPING: cannot spawn rustc for standin build: {e}");
            return None;
        }
    }
    let base = dir.join("standin-bin");
    for stem in ["good", "abi-liar", "id-liar", "ver-liar", "crasher", "hung"] {
        let to = dir.join(format!("standin-{stem}"));
        std::fs::copy(&base, &to).expect("copy standin");
        std::fs::set_permissions(&to, std::os::unix::fs::PermissionsExt::from_mode(0o755))
            .expect("chmod +x");
    }
    Some(dir)
}

fn manifest_toml(exe: &Path, restart_toml: &str) -> String {
    format!(
        "id = \"{ID}\"\nversion = \"{VERSION}\"\n[entry]\nkind = \"exe\"\nartifact = \"{}\"\n{restart_toml}",
        exe.display()
    )
}

fn parse_manifest(exe: &Path, restart_toml: &str) -> Manifest {
    Manifest::from_toml_str(&manifest_toml(exe, restart_toml))
        .unwrap_or_else(|e| panic!("fixture manifest must parse: {e}"))
}

fn step_supervisor() -> Supervisor {
    Supervisor::with_clock(Clock::new(ClockDomain::Step))
}

fn install_resolve_start(sup: &Supervisor, exe: &Path, restart_toml: &str) -> PluginId {
    let manifest = parse_manifest(exe, restart_toml);
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("install");
    sup.kernel()
        .resolve(&id)
        .expect("spawn + handshake must pass");
    sup.kernel()
        .start(&id)
        .expect("start over the control socket");
    id
}

/// Lifecycle `from->to` strings for the tracked plugin, in ring order.
fn lifecycle_walk(sup: &Supervisor) -> Vec<String> {
    sup.kernel()
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

/// Poll the supervisor until it observes the child's self-exit (bounded:
/// the wait is on a real OS process, zero-sleep discipline exempt here).
fn wait_crash(sup: &Supervisor) -> Vec<String> {
    let deadline = Instant::now() + CRASH_POLL_DEADLINE;
    loop {
        let crashed = sup.poll();
        if !crashed.is_empty() {
            return crashed;
        }
        assert!(
            Instant::now() < deadline,
            "crash never observed within the poll deadline"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

/// Zombies among this test process's children must reach zero after every
/// supervision interaction (S2-M1 "no zombies ever"). Tests in this binary
/// run as threads of ONE process, so a sibling test's child may be briefly
/// a zombie inside its own crash-watch poll window; the scan therefore
/// retries until clean. Every reap path in the executor bounds itself by a
/// constant far below this deadline, so a leak here is a real one.
fn assert_no_zombie_children() {
    let me = std::process::id().to_string();
    let deadline = Instant::now() + Duration::from_secs(3);
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
            "zombie children survived the supervisor loop: {zombies:?}"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn exe_spawn_green_walks_full_lifecycle_over_the_control_socket() {
    // Arrange
    let Some(dir) = build_standins("green") else {
        return;
    };
    let sup = step_supervisor();
    let exe = dir.join("standin-good");

    // Act: the full dl-parity walk on an exe manifest.
    let id = install_resolve_start(&sup, &exe, "");

    // Assert: every state edge ran through the existing machine.
    assert_eq!(
        sup.kernel().state(&id).expect("started"),
        PluginState::Active
    );
    let walk = lifecycle_walk(&sup);
    assert_eq!(
        walk,
        vec![
            "UNINSTALLED->INSTALLED",
            "INSTALLED->RESOLVING",
            "RESOLVING->STARTING",
            "STARTING->ACTIVE",
        ]
    );
    assert_eq!(
        sup.kernel().counters().load_ok(),
        1,
        "handshake passed once"
    );
    sup.kernel().stop(&id).expect("graceful STOP");
    assert_eq!(
        sup.kernel().state(&id).expect("stopped"),
        PluginState::Resolved
    );
    sup.kernel().uninstall(&id).expect("uninstall");
    assert_no_zombie_children();
}

#[test]
fn handshake_abort_on_abi_lie_lands_failed_edge_no_panic() {
    // Arrange
    let Some(dir) = build_standins("abi-lie") else {
        return;
    };
    let sup = step_supervisor();
    let manifest = parse_manifest(&dir.join("standin-abi-liar"), "");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("install");

    // Act: resolve spawns the liar; the ABI v1 recheck must abort it.
    let err = sup
        .kernel()
        .resolve(&id)
        .expect_err("abi 999 must never be accepted silently");

    // Assert: typed failure, FAILED edge, the bad-plugin doorway counted.
    assert!(
        matches!(err, KernelError::AbiMismatch { found, .. } if found == 999),
        "expected typed AbiMismatch(found=999), got {err}"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("still tracked"),
        PluginState::Failed
    );
    assert_eq!(sup.kernel().counters().load_fail(), 1);
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "the liar was reaped at load, no phantom"
    );
    assert_no_zombie_children();
    // Reaching here without a crash IS the second half of the assertion.
}

#[test]
fn handshake_abort_on_id_lie_is_rejected_by_strict_self_report() {
    // Arrange
    let Some(dir) = build_standins("id-lie") else {
        return;
    };
    let sup = step_supervisor();
    let manifest = parse_manifest(&dir.join("standin-id-liar"), "");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("install");

    // Act
    let err = sup
        .kernel()
        .resolve(&id)
        .expect_err("a lying self-report must abort the handshake (D7 family)");

    // Assert
    assert!(
        matches!(&err, KernelError::IdentityMismatch { what, .. } if *what == "id"),
        "expected typed IdentityMismatch(id), got {err}"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("still tracked"),
        PluginState::Failed
    );
}

#[test]
fn crash_injection_keeps_host_alive_lands_failed_edge_and_restarts_bounded() {
    // Arrange: on-failure policy with a budget of exactly one restart.
    let Some(dir) = build_standins("crash") else {
        return;
    };
    let sup = step_supervisor();
    let restart = "[restart]\npolicy = \"on-failure\"\nmax_retries = 1\nbackoff = \"1s..4s\"\n";
    let id = install_resolve_start(&sup, &dir.join("standin-crasher"), restart);

    // Act: the child self-kills right after the START ack; the watcher sees it.
    assert_eq!(wait_crash(&sup), vec![ID.to_string()]);

    // Assert: FAILED edge through the existing machine, crash counter moved,
    // and the restart budget gates on the D3 policy BEFORE the dial allows.
    assert_eq!(
        sup.kernel().state(&id).expect("host alive"),
        PluginState::Failed
    );
    assert_eq!(sup.kernel().counters().process_crashes(), 1);
    assert_eq!(
        lifecycle_walk(&sup),
        vec![
            "UNINSTALLED->INSTALLED",
            "INSTALLED->RESOLVING",
            "RESOLVING->STARTING",
            "STARTING->ACTIVE",
            "ACTIVE->STOPPING",
            "STOPPING->FAILED",
        ]
    );
    assert!(
        matches!(sup.revive(&id), Ok(Revive::Deferred)),
        "backoff is a gate, not a suggestion: no restart before the window"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("still FAILED"),
        PluginState::Failed
    );
    sup.kernel().advance_clock(Duration::from_secs(1));
    assert!(
        matches!(sup.revive(&id), Ok(Revive::Revived)),
        "budget allows one restart"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("restarted"),
        PluginState::Active
    );
    assert_eq!(sup.kernel().counters().process_restarts(), 1);

    // Act: the flapped child crashes again; the budget is now spent.
    wait_crash(&sup);
    sup.kernel().advance_clock(Duration::from_secs(1));

    // Assert: parked in FAILED with the budget exhausted - bounded, visible.
    assert!(
        matches!(sup.revive(&id), Ok(Revive::Parked)),
        "max_retries=1 must bound"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("parked"),
        PluginState::Failed
    );
    assert_eq!(sup.kernel().counters().process_crashes(), 2);
    assert_eq!(sup.kernel().counters().process_restarts(), 1);
    assert_no_zombie_children();
}

#[test]
fn flapping_peer_hits_max_restarts_and_parks_in_failed() {
    // Arrange: budget of two restarts, crasher stand-in.
    let Some(dir) = build_standins("flap") else {
        return;
    };
    let sup = step_supervisor();
    let restart = "[restart]\npolicy = \"on-failure\"\nmax_retries = 2\nbackoff = \"1s..4s\"\n";
    let id = install_resolve_start(&sup, &dir.join("standin-crasher"), restart);

    for cycle in 1..=3usize {
        wait_crash(&sup);
        sup.kernel().advance_clock(Duration::from_secs(10));
        let outcome = sup.revive(&id).expect("revive answers, never errors");
        if cycle <= 2 {
            assert!(
                matches!(outcome, Revive::Revived),
                "cycle {cycle} must restart"
            );
        } else {
            assert!(matches!(outcome, Revive::Parked), "cycle {cycle} must park");
        }
    }

    // Assert: three crashes observed, exactly two restarts granted, the
    // reload doorway ran exactly three handshakes - zero phantom reloads.
    assert_eq!(sup.kernel().counters().process_crashes(), 3);
    assert_eq!(sup.kernel().counters().process_restarts(), 2);
    assert_eq!(
        sup.kernel().counters().load_ok(),
        3,
        "one load per supervised run"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("parked"),
        PluginState::Failed
    );
    let walk = lifecycle_walk(&sup);
    assert_eq!(
        walk.iter()
            .filter(|e| e.as_str() == "FAILED->RESOLVING")
            .count(),
        2,
        "restarts ride only the real FAILED->RESOLVING edge: {walk:?}"
    );
    assert_no_zombie_children();
}

#[test]
fn stop_message_drives_graceful_teardown_no_kill_needed() {
    // Arrange
    let Some(dir) = build_standins("graceful") else {
        return;
    };
    let sup = step_supervisor();
    let id = install_resolve_start(&sup, &dir.join("standin-good"), "");

    // Act: kernel.stop succeeds only if the child READ the STOP frame and
    // acked it - the timeout path would return Err(ProcessCrashed).
    sup.kernel().stop(&id).expect("graceful STOP message acked");

    // Assert: cooperative stop lands RESOLVED (03 §2), process reaped.
    assert_eq!(
        sup.kernel().state(&id).expect("stopped"),
        PluginState::Resolved
    );
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "nothing crashed on the way out"
    );
    assert_no_zombie_children();
}

#[test]
fn hung_peer_is_killed_after_named_timeout_constant() {
    // Arrange
    let Some(dir) = build_standins("hung") else {
        return;
    };
    let sup = step_supervisor();
    let id = install_resolve_start(&sup, &dir.join("standin-hung"), "");

    // Act: STOP is ignored; teardown must bound itself by the named constant.
    let at = Instant::now();
    let err = sup
        .kernel()
        .stop(&id)
        .expect_err("a peer that never acks STOP cannot stop cleanly");
    let waited = at.elapsed();

    // Assert: typed crash failure, FAILED edge, kill happened, bounded wait.
    assert!(matches!(err, KernelError::ProcessCrashed { .. }), "{err}");
    assert_eq!(
        sup.kernel().state(&id).expect("host alive"),
        PluginState::Failed
    );
    assert!(
        waited <= STOP_KILL_GRACE * 3,
        "teardown waited {waited:?}; the kill timeout constant is {STOP_KILL_GRACE:?}"
    );
    assert_no_zombie_children();
}

/// D80 (S2-M3b): the swap door asked to co-host two versions of a declared
/// parallel_ok service gets a TYPED DEFERRAL — never a silent ignore, never
/// the bare transition refusal that hides the author's request. Undeclared
/// co-host requests keep the M7 duplicate-identity path untouched.
#[test]
fn cohost_request_on_a_declared_parallel_ok_service_is_deferred_typed() {
    let Some(dir) = build_standins("d80") else {
        return;
    };
    let sup = step_supervisor();
    let declared = manifest_toml(
        &dir.join("standin-good"),
        "[[provides]]\nservice = \"com.example.proc.stand-in.echo\"\nparallel_ok = true\n",
    );
    let manifest = Manifest::from_toml_str(&declared).expect("declared manifest parses");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("install");
    sup.kernel().resolve(&id).expect("resolve");
    sup.kernel().start(&id).expect("start (ACTIVE)");

    // The co-host request: a new version installed while the old is ACTIVE.
    let swap = declared.replace("version = \"0.1.0\"", "version = \"0.2.0\"");
    let swap = Manifest::from_toml_str(&swap).expect("swap manifest parses");
    let err = sup
        .kernel()
        .install(swap)
        .expect_err("parallel_ok co-host must get a typed voice");
    assert!(
        matches!(&err, KernelError::RoutingDeferred { plugin, service }
            if plugin == ID && service == "com.example.proc.stand-in.echo"),
        "expected typed RoutingDeferred, got {err}"
    );
    // The deferral is a refusal at the door, not a teardown: the run lives.
    assert_eq!(
        sup.kernel().state(&id).expect("still ACTIVE"),
        PluginState::Active
    );

    // Stop-based swap stays the legal lane: stop, then the install replaces
    // the record (terminal state) exactly as before. (The swapped 0.2.0 is
    // not re-resolved here — the stand-in self-reports 0.1.0 and the strict
    // D7 cross-check would rightly refuse the version lie.)
    sup.kernel().stop(&id).expect("stop");
    let legal =
        Manifest::from_toml_str(&declared.replace("version = \"0.1.0\"", "version = \"0.2.0\""))
            .expect("parses");
    sup.kernel()
        .install(legal)
        .expect("stop-based swap accepted");
    sup.kernel()
        .uninstall(&id)
        .expect("uninstall swapped record");

    // Undeclared path unchanged: a live re-install with no parallel_ok
    // declaration stays the generic M7 refusal, not the deferral.
    let plain = manifest_toml(&dir.join("standin-good"), "");
    let plain_manifest = Manifest::from_toml_str(&plain).expect("parses");
    sup.kernel()
        .install(plain_manifest)
        .expect("install undeclared");
    sup.kernel().resolve(&id).expect("resolve undeclared run");
    sup.kernel().start(&id).expect("start undeclared run");
    let bump = plain.replace("version = \"0.1.0\"", "version = \"0.3.0\"");
    let err = sup
        .kernel()
        .install(Manifest::from_toml_str(&bump).expect("parses"))
        .expect_err("undeclared co-host stays the M7 policy refusal");
    assert!(
        matches!(err, KernelError::InvalidTransition { .. }),
        "undeclared path changed: {err}"
    );
    sup.kernel().stop(&id).expect("stop undeclared");
    sup.kernel().uninstall(&id).expect("uninstall");
    assert_no_zombie_children();
    std::fs::remove_dir_all(&dir).expect("cleanup scratch");
    assert!(!dir.exists(), "mk-standin scratch must not leak");
}

/// S2-M3b host-loop driver: ONE `step` pass replaces the hand-assembled
/// poll → classify → revive sequence (crash-watch, due-posting, and the
/// D3 restart policy in one doorway). The backoff window stays a gate —
/// the same pass defers instead of force-restarting.
#[test]
fn one_driver_step_pass_collapses_poll_and_restart_policy_application() {
    let Some(dir) = build_standins("driver-step") else {
        return;
    };
    let sup = step_supervisor();
    let restart = "[restart]\npolicy = \"on-failure\"\nmax_retries = 1\nbackoff = \"1s..4s\"\n";
    let id = install_resolve_start(&sup, &dir.join("standin-crasher"), restart);

    // The crasher self-exits right after the START ack; poll each driver
    // pass until it observes the death (real process timing, bounded).
    let deadline = Instant::now() + CRASH_POLL_DEADLINE;
    let first = loop {
        let report = sup.step();
        if !report.crashed.is_empty() {
            break report;
        }
        assert!(
            Instant::now() < deadline,
            "crash never observed within the poll deadline"
        );
        std::thread::sleep(Duration::from_millis(2));
    };
    assert_eq!(first.crashed, vec![ID.to_string()]);
    assert!(first.revived.is_empty());
    assert_eq!(
        first.deferred,
        vec![ID.to_string()],
        "the same pass applies the policy: the window is closed, so DEFER"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("tracked"),
        PluginState::Failed
    );

    // Turn the boot clock past the window; the next single pass restarts.
    sup.kernel().advance_clock(Duration::from_secs(4));
    let second = sup.step();
    assert_eq!(
        second.revived,
        vec![ID.to_string()],
        "the policy opened the window — one pass, respawned"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("restarted"),
        PluginState::Active
    );

    let _ = sup.poll();
    let _ = sup.kernel().stop(&id); // the respawned crasher may be gone; kill path is legal
    let _ = sup.poll();
    assert_no_zombie_children();
    std::fs::remove_dir_all(&dir).expect("cleanup scratch");
    assert!(!dir.exists(), "mk-standin scratch must not leak");
}

/// A quiet supervised run drives to idle after exactly one bounded pass —
/// the host-loop driver never spins (S2-M3b seam contract).
#[test]
fn drive_until_idle_returns_after_one_quiet_pass() {
    let Some(dir) = build_standins("driver-idle") else {
        return;
    };
    let sup = step_supervisor();
    let id = install_resolve_start(&sup, &dir.join("standin-good"), "");

    let driven = sup.drive_until_idle(8);
    assert_eq!(driven.passes, 1, "a quiet host is idle after one pass");
    assert!(driven.report.crashed.is_empty(), "{:?}", driven.report);
    assert!(driven.report.revived.is_empty(), "{:?}", driven.report);
    assert!(driven.report.deferred.is_empty(), "{:?}", driven.report);

    sup.kernel().stop(&id).expect("stop");
    sup.kernel().uninstall(&id).expect("uninstall");
    assert_eq!(sup.poll(), Vec::<String>::new());
    assert_no_zombie_children();
    std::fs::remove_dir_all(&dir).expect("cleanup scratch");
    assert!(!dir.exists(), "mk-standin scratch must not leak");
}

/// D74 timeout guard (S2-M3b): a child that never answers the dial is
/// killed at the named `DIAL_TIMEOUT` bound and reported through the
/// ordinary crash family — the dial can never be held open by a wedged
/// runner (the plan's timeout guard against a blocked runner).
#[test]
fn dial_of_a_silent_child_is_killed_at_the_named_dial_timeout() {
    let Some(dir) = build_standins("dial-timeout") else {
        return;
    };
    let sup = step_supervisor();
    let id = install_resolve_start(&sup, &dir.join("standin-hung"), "");

    let at = Instant::now();
    let err = sup
        .dial(&id, Duration::from_secs(1))
        .expect_err("a silent peer cannot hold the dial open");
    let waited = at.elapsed();

    assert!(matches!(err, KernelError::ProcessCrashed { .. }), "{err}");
    assert!(
        waited <= DIAL_TIMEOUT * 3,
        "dial waited {waited:?}; the named guard is {DIAL_TIMEOUT:?}"
    );
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "killed+reaped at the dial doorway"
    );
    assert_no_zombie_children();
    std::fs::remove_dir_all(&dir).expect("cleanup scratch");
    assert!(!dir.exists(), "mk-standin scratch must not leak");
}

/// S2-M3 frame versioning, codec doorways: the writer stamps the version
/// byte, the reader reports a foreign version as data for the caller to
/// type (parse, don't validate — the kernel doors answer with
/// `KernelError::WireVersionMismatch`, the codec itself stays io-level).
#[test]
fn frame_codec_stamps_the_wire_version_and_surfaces_a_foreign_one() {
    use modukit_core::proc_wire;
    use std::os::unix::net::UnixStream;
    let (mut a, mut b) = UnixStream::pair().expect("socketpair");

    // One honest round trip: the raw header is len(4) | ver=1 | kind.
    proc_wire::write_frame(&mut a, proc_wire::MSG_ACK, b"xy").expect("write");
    let mut raw = [0u8; 8];
    std::io::Read::read_exact(&mut b, &mut raw).expect("raw header+payload");
    assert_eq!(
        &raw,
        &[
            2,
            0,
            0,
            0,
            proc_wire::WIRE_VERSION,
            proc_wire::MSG_ACK,
            b'x',
            b'y'
        ],
        "envelope = len | ver | type | payload"
    );

    // A hand-framed foreign ver arrives intact: read_frame reports it,
    // it does not reinterpret or silently drop (the caller types it).
    let foreign = [4u8, 0, 0, 0, 9, 1, 0, 0, 0, 0]; // len 4, ver 9, kind 1
    std::io::Write::write_all(&mut b, &foreign).expect("write foreign");
    let (ver, kind, payload) = proc_wire::read_frame(&mut a).expect("read foreign");
    assert_eq!((ver, kind, payload.as_slice()), (9, 1, &[0u8, 0, 0, 0][..]));
}

/// The parent refuses a foreign envelope version on the HELLO with the
/// typed protocol error — never a decode, never a supervised run.
#[test]
fn handshake_abort_on_foreign_wire_version_is_typed_version_mismatch() {
    let Some(dir) = build_standins("ver-lie") else {
        return;
    };
    let sup = step_supervisor();
    let manifest = parse_manifest(&dir.join("standin-ver-liar"), "");
    let id = manifest.id.clone();
    sup.kernel().install(manifest).expect("install");

    let err = sup
        .kernel()
        .resolve(&id)
        .expect_err("a v9 frame must never be decoded as v1");
    assert!(
        matches!(
            err,
            KernelError::WireVersionMismatch {
                expected: 1,
                found: 9
            }
        ),
        "{err}"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("still tracked"),
        PluginState::Failed
    );
    assert_eq!(sup.kernel().counters().load_fail(), 1);
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "the mismatched child was reaped at load, no phantom"
    );
    assert_no_zombie_children();
}
