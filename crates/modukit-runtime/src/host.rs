//! modukit-host — the deferred supervisor binary (S2-M5; modules/01 §1 row 2,
//! the S2-M1 E1 escalation finding whose named home is the host loop here).
//!
//! ```text
//! modukit-host <manifest.toml> [--clock=step|real]
//! ```
//!
//! Boots `Supervisor` with the `exe` executor pair (`with_clock` wires
//! `ProcessExecutor::paired()` + the kernel-side registry door for REGISTER
//! proxies — that IS the production pair constructor; no separate wiring
//! exists), installs/resolves/starts the ONE manifest plugin (V0 hosts
//! exactly one manifest — nothing more), then runs the documented host
//! loop: `drive_until_idle` each pass. On a Real clock the loop parks on
//! `IDLE_PARK` between passes (the loop's wall-time surface: due-job
//! posting and crash re-watch ride the cadence); on Step the loop turns
//! per pass with no sleep (zero-sleep discipline — a Step host has no
//! wall-time surface, and the dial is the supervisor's, S2-M3b).
//! SIGTERM/SIGINT flip a signal-safe flag; the loop exits and takes the
//! EXISTING graceful door (`Kernel::stop`: STOP frame, ack,
//! `STOP_KILL_GRACE` SIGKILL escalation), exit 0.
//!
//! Error posture (bin, application-level; mirrors `modukit-run`): every
//! fallible setup step lands on a nonzero exit code with a one-line stderr
//! note. Exit codes: 2 usage, 3 manifest read/parse, 4 install, 5 resolve,
//! 6 start.

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::executor::Supervisor;
use modukit_core::identity::PluginId;
use modukit_core::manifest::Manifest;

/// Real-clock idle park between supervisor passes (the host-loop contract's
/// named idle-park constant).
const IDLE_PARK: Duration = Duration::from_millis(100);

/// Flipped by SIGTERM/SIGINT (async-signal-safe: one lock-free store); the
/// main loop observes it and runs the graceful stop on its own thread.
static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn on_signal(_sig: i32) {
    RUNNING.store(false, Ordering::Relaxed);
}

// Minimal `signal(2)` binding: the vendor-lock floor forbids adding a
// signal-handling dependency, and the libc symbol needs no crate — the
// platform C runtime resolves it at link time. SAFETY: signum is one of the
// two constants below; the handler is a plain async-signal-safe store.
unsafe extern "C" {
    fn signal(signum: i32, handler: extern "C" fn(i32)) -> usize;
}

fn usage() -> ! {
    eprintln!("usage: modukit-host <manifest.toml> [--clock=step|real]");
    std::process::exit(2)
}

fn parse_args() -> (String, ClockDomain) {
    let mut manifest: Option<String> = None;
    let mut domain = ClockDomain::Real;
    for arg in std::env::args().skip(1) {
        if let Some(rest) = arg.strip_prefix("--clock=") {
            domain = match rest {
                "step" => ClockDomain::Step,
                "real" => ClockDomain::Real,
                other => {
                    eprintln!("modukit-host: bad --clock value: {other}");
                    usage()
                }
            };
        } else if arg == "--help" {
            println!("modukit-host <manifest.toml> [--clock=step|real]");
            println!("  Supervise ONE exe-placement plugin: install, resolve, start,");
            println!("  then drive the supervisor loop (crash-watch, D3 restarts,");
            println!("  due-job posting) until SIGTERM/SIGINT: graceful stop, exit 0.");
            std::process::exit(0);
        } else if arg.starts_with("--") {
            eprintln!("modukit-host: unknown option {arg}");
            usage()
        } else if manifest.is_none() {
            manifest = Some(arg);
        } else {
            usage()
        }
    }
    match manifest {
        Some(path) => (path, domain),
        None => usage(),
    }
}

fn run(path: &str, domain: ClockDomain) -> i32 {
    let text = match std::fs::read_to_string(Path::new(path)) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("modukit-host: read {path}: {err}");
            return 3;
        }
    };
    let manifest = match Manifest::from_toml_str(&text) {
        Ok(manifest) => manifest,
        Err(err) => {
            eprintln!("modukit-host: parse {path}: {err}");
            return 3;
        }
    };
    let id = manifest.id.as_str().to_string();
    let pid = PluginId::new(id.clone());

    // The production pair in one constructor (proc_supervisor.rs with_clock).
    let sup = Supervisor::with_clock(Clock::new(domain));
    if let Err(err) = sup.kernel().install(manifest) {
        eprintln!("modukit-host: install: {err}");
        return 4;
    }
    if let Err(err) = sup.kernel().resolve(&pid) {
        eprintln!("modukit-host: resolve (spawn + HELLO): {err}");
        return 5;
    }
    if let Err(err) = sup.kernel().start(&pid) {
        eprintln!("modukit-host: start (START ack / REGISTER): {err}");
        return 6;
    }
    println!("modukit-host: ACTIVE plugin={id}");

    // The host loop (S2-M1 E1's home): one bounded drive per pass; pacing
    // follows the clock domain.
    while RUNNING.load(Ordering::Relaxed) {
        let driven = sup.drive_until_idle(8);
        let r = &driven.report;
        if !r.posted.is_empty()
            || !r.crashed.is_empty()
            || !r.revived.is_empty()
            || !r.parked.is_empty()
        {
            println!(
                "modukit-host: pass passes={} posted={:?} crashed={:?} revived={:?} deferred={:?} parked={:?}",
                driven.passes, r.posted, r.crashed, r.revived, r.deferred, r.parked
            );
        }
        if domain == ClockDomain::Real {
            std::thread::sleep(IDLE_PARK);
        }
    }
    match sup.kernel().stop(&pid) {
        Ok(()) => {
            println!("modukit-host: stopped plugin={id}");
            0
        }
        Err(err) => {
            eprintln!("modukit-host: graceful stop failed: {err}");
            1
        }
    }
}

fn main() {
    let (path, domain) = parse_args();
    // SAFETY: see the extern declaration above — two fixed signums, one
    // async-signal-safe handler.
    unsafe {
        signal(15, on_signal); // SIGTERM
        signal(2, on_signal); // SIGINT
    }
    std::process::exit(run(&path, domain));
}
