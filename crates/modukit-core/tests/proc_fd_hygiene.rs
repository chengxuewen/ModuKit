//! S2-M1 fd hygiene: the supervisor's spawn/handshake/teardown cycles must
//! return every descriptor they opened. Its own test file = own process, so
//! the count is not perturbed by unrelated parallel tests.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::executor::Supervisor;
use modukit_core::manifest::Manifest;

const ID: &str = "com.example.proc.stand-in";

/// Same stand-in source as tests/process_executor.rs, honest mode only
/// (plus the crasher for the supervised-death leg). Kept byte-identical in
/// protocol; duplicating a 60-line test double is cheaper than a shared
/// test-support crate that would widen the workspace.
const STANDIN_SRC: &str = r#"
use std::io::{Read, Write};
use std::os::fd::FromRawFd;
use std::os::unix::net::UnixStream;

fn frame(kind: u8, payload: &[u8]) -> Vec<u8> {
    let mut out = (payload.len() as u32).to_le_bytes().to_vec();
    out.push(kind);
    out.extend_from_slice(payload);
    out
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
    // SAFETY: the supervising host injected the control socket here.
    let mut sock = unsafe { UnixStream::from_raw_fd(fd_num) };
    let mut p = 1u32.to_le_bytes().to_vec();
    for s in ["com.example.proc.stand-in", "0.1.0", "stand-in"] {
        p.push(s.len() as u8);
        p.extend_from_slice(s.as_bytes());
    }
    let _ = sock.write_all(&frame(1, &p));
    let _ = sock.flush();
    let mut buf = [0u8; 5];
    loop {
        let mut done = 0;
        while done < buf.len() {
            match sock.read(&mut buf[done..]) {
                Ok(0) | Err(_) => std::process::exit(0),
                Ok(n) => done += n,
            }
        }
        let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        let mut junk = vec![0u8; len];
        if len > 0 {
            let _ = sock.read_exact(&mut junk);
        }
        match buf[4] {
            2 => {
                let _ = sock.write_all(&frame(4, &0i32.to_le_bytes()));
                let _ = sock.flush();
                if stem.contains("crasher") {
                    std::process::exit(7);
                }
            }
            3 => {
                let _ = sock.write_all(&frame(4, &0i32.to_le_bytes()));
                let _ = sock.flush();
                std::process::exit(0);
            }
            _ => {}
        }
    }
}
"#;

fn build_standins() -> Option<PathBuf> {
    let dir = std::env::temp_dir().join(format!("mk-standin-fd-{}", std::process::id()));
    if dir.join("standin-good").is_file() {
        return Some(dir);
    }
    std::fs::create_dir_all(&dir).expect("temp dir");
    let src = dir.join("standin.rs");
    std::fs::write(&src, STANDIN_SRC).expect("write source");
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
            eprintln!("SKIPPING: cannot spawn rustc: {e}");
            return None;
        }
    }
    for stem in ["good", "crasher"] {
        let to = dir.join(format!("standin-{stem}"));
        std::fs::copy(dir.join("standin-bin"), &to).expect("copy");
        std::fs::set_permissions(&to, std::os::unix::fs::PermissionsExt::from_mode(0o755))
            .expect("chmod");
    }
    Some(dir)
}

fn open_fd_count() -> usize {
    std::fs::read_dir("/proc/self/fd")
        .expect("/proc/self/fd")
        .count()
}

fn manifest_toml(exe: &Path) -> Manifest {
    Manifest::from_toml_str(&format!(
        "id = \"{ID}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"exe\"\nartifact = \"{}\"\n\
         [restart]\npolicy = \"on-failure\"\nmax_retries = 2\nbackoff = \"1s..4s\"\n",
        exe.display()
    ))
    .expect("manifest parses")
}

#[test]
fn spawn_and_teardown_cycles_leak_no_descriptors() {
    let Some(dir) = build_standins() else { return };
    // Build + Supervisor boot happen first: only the spawn/teardown cycles
    // below are timed by the before/after descriptor count.
    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
    let clean = manifest_toml(&dir.join("standin-good"));
    let crashy = manifest_toml(&dir.join("standin-crasher"));
    let id = clean.id.clone();
    let before = open_fd_count();

    for _ in 0..3 {
        // clean cycle: install -> spawn -> handshake -> start -> graceful stop
        sup.kernel().install(clean.clone()).expect("install");
        sup.kernel().resolve(&id).expect("resolve");
        sup.kernel().start(&id).expect("start");
        sup.kernel().stop(&id).expect("stop");
        sup.kernel().uninstall(&id).expect("uninstall");
    }

    // crash cycle: supervised death -> watcher reap -> revive -> stop
    sup.kernel()
        .install(crashy)
        .expect("re-install replaces record");
    sup.kernel().resolve(&id).expect("resolve crasher");
    sup.kernel().start(&id).expect("start crasher");
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while sup.poll().is_empty() && std::time::Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(2));
    }
    sup.kernel().advance_clock(Duration::from_secs(1));
    let _ = sup.revive(&id);
    if sup.kernel().state(&id).expect("state") == modukit_core::lifecycle::PluginState::Active {
        // crasher mode may self-exit before the STOP ack lands: the kill
        // path is the other legal teardown here.
        let _ = sup.kernel().stop(&id);
    }
    let _ = sup.poll();

    let after = open_fd_count();
    assert_eq!(
        after, before,
        "descriptor count grew across spawn/teardown cycles: {before} -> {after}"
    );
}
