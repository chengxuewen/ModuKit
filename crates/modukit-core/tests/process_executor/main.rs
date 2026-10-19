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
//!
//! Crate layout: this root holds the shared stand-in fixture builder and the
//! test helpers; the tests live in topic submodules (mod declarations below).

mod cohost;
mod driver;
mod handshake;
mod lifecycle;
mod restart;
mod teardown;

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
        // names: mk-standin-<pid>-<nanos>-<tag> (process_executor) or
        // mk-standin-fd-<pid>-<nanos> - pid is always the first numeric segment.
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
    // The dir is unique per invocation (pid + unix-nanos + tag) to isolate
    // concurrent and instrumented runs - but this is NOT a cure for the
    // ETXTBSY class: the 2026-10-19 hunt caught a sighting INSIDE such a
    // private dir. The race is the kernel's async-close window after the
    // landing writes below; the bounded retry at the spawn doorway
    // (executor_proc SPAWN_ETXTBSY_RETRY_BOUND) is what absorbs it.
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mk-standin-{}-{ts}-{tag}", std::process::id()));
    let exe = dir.join("standin-good");
    if exe.is_file() && dir.join("standin-hung").is_file() {
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
        .arg(dir.join("standin-bin-rustc-standin.tmp"))
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
    // atomic landings: writes go to .new/.tmp names and rename into place, so
    // a concurrent exec never sees a half-written file. (The separate
    // async-close ETXTBSY window is handled at the spawn doorway, not here.)
    let base = dir.join("standin-bin");
    std::fs::rename(dir.join("standin-bin-rustc-standin.tmp"), &base).expect("rename bin");
    for stem in ["good", "abi-liar", "id-liar", "ver-liar", "crasher", "hung"] {
        let to = dir.join(format!("standin-{stem}"));
        let stage = dir.join(format!("standin-{stem}.new"));
        std::fs::copy(&base, &stage).expect("copy standin");
        std::fs::set_permissions(&stage, std::os::unix::fs::PermissionsExt::from_mode(0o755))
            .expect("chmod +x");
        std::fs::rename(&stage, &to).expect("land standin");
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
