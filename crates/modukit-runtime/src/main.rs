//! modukit-run — the shadow-runner (S2-M2; modules/01 §2).
//!
//! `exe` placement does not require a separate executable per plugin: the
//! host spawns this generic runner with the SAME artifact the `dl` executor
//! would load plus the inherited control fd —
//!
//! ```text
//! modukit-run <plugin.so> --fd=N
//! ```
//!
//! (N = 0 today: `ProcessExecutor` injects the child end of the socketpair as
//! stdin, executor_proc.rs `CHILD_CONTROL_FD`; the `--fd=N` convention is
//! honored, not the number.) The runner `dlopen`s the artifact through
//! `DlopenExecutor` — one source of truth for the ABI v1 guards — embeds a
//! kernel of its own as the in-process host, and pumps the contract over the
//! fd with the `proc_wire` envelope: HELLO (the plugin's OWN self-report,
//! D7 tier) on connect, START -> `kernel.start` -> ACK, STOP -> `kernel.stop`
//! -> ACK, clean exit when the control socket EOFs (the parent is gone).
//!
//! Error posture (bin, application-level): every fallible setup step is
//! handled explicitly and lands on a nonzero exit code with a one-line
//! stderr note; a rejected artifact dies BEFORE it can speak a HELLO, so the
//! parent observes the typed crash family, never a lie. Exit codes: 2 usage,
//! 3 artifact, 4 dlopen guards, 5 self-report, 6 hello write, 7..9 embedded-
//! kernel setup, 11 control read.

use std::env;
use std::io;
use std::os::fd::FromRawFd;
use std::os::unix::net::UnixStream;
use std::path::Path;

use modukit_core::abi::{ABI_VERSION, NativeService};
use modukit_core::error::KernelError;
use modukit_core::executor::{DlopenExecutor, Executor};
use modukit_core::identity::PluginId;
use modukit_core::kernel::Kernel;
use modukit_core::lifecycle::RestartPolicy;
use modukit_core::manifest::{Entry, Manifest, PlacementKind};
use modukit_core::proc_wire::{
    self, MSG_ACK, MSG_HELLO, MSG_START, MSG_STOP, hello_payload, read_frame, write_frame,
};

/// One handshake string must fit the wire's one-byte length prefix
/// (`proc_wire::parse_hello` reads `u8` counts — the cap is the format).
const HELLO_STR_MAX: usize = u8::MAX as usize;

/// The `NativeService` probe the runner round-trips inside its own process
/// after `start` (D69: bytes only). shortcut: the M2 envelope carries no CALL
/// frame, so the runner verifies the template fixture's contract (modules/04
/// §4, example-plugin) process-locally and gates the START ack on it; S2-M3
/// cross-process dispatch replaces this self-check as the observable path.
const PROBE_METHOD: &str = "echo";
const PROBE_BYTES: &[u8] = b"shadow-runner-selfcheck";

fn die(code: i32, msg: &str) -> ! {
    eprintln!("modukit-run: {msg}");
    std::process::exit(code)
}

fn parse_args() -> (String, i32) {
    let mut artifact: Option<String> = None;
    let mut fd_num: i32 = 0;
    for arg in env::args().skip(1) {
        if let Some(rest) = arg.strip_prefix("--fd=") {
            fd_num = match rest.parse::<i32>() {
                Ok(n) if n >= 0 => n,
                _ => die(2, &format!("bad --fd value: {rest}")),
            };
        } else if arg.starts_with("--") {
            die(2, &format!("unknown option {arg}"));
        } else if artifact.is_none() {
            artifact = Some(arg);
        } else {
            die(2, "usage: modukit-run <plugin.so> --fd=N");
        }
    }
    match artifact {
        Some(path) => (path, fd_num),
        None => die(2, "usage: modukit-run <plugin.so> --fd=N"),
    }
}

/// An internal `dl` manifest for the embedded machinery: the runner declares
/// only WHERE the artifact runs in-process (executor `load` reads exactly
/// `entry.kind` + `entry.artifact`; id/version ride the HELLO, validated by
/// the parent's strict cross-check, executor_proc.rs).
fn internal_manifest(artifact: &str, id: String, version: String) -> Manifest {
    Manifest {
        id: PluginId::new(id),
        version,
        team: "modukit-runtime".to_string(),
        start_level: 0,
        start_timeout: None,
        stop_timeout: None,
        min_host: None,
        capabilities: Vec::new(),
        entry: Entry {
            kind: PlacementKind::Dl,
            artifact: artifact.to_string(),
            runner: None,
        },
        restart: RestartPolicy::None,
        requires: Vec::new(),
        provides: Vec::new(),
    }
}

/// Read the artifact's own self-report before anything is spoken: the guards
/// of `DlopenExecutor::load` run HERE first so a rejected artifact dies
/// before HELLO (fail-closed), then run again inside the embedded kernel.
fn self_report(artifact: &str) -> (String, String, String) {
    let empty = internal_manifest(artifact, String::new(), String::new());
    let probe = match DlopenExecutor::new().load(&empty) {
        Ok(handle) => handle,
        Err(err) => die(4, &format!("load {artifact}: {err}")),
    };
    let (id, version) = probe.self_report();
    let name = probe.plugin_name().to_string();
    drop(probe);
    let (Some(id), Some(version)) = (id, version) else {
        die(
            5,
            "plugin self-report is silent (id/version): refusing to speak a lie",
        );
    };
    for (what, s) in [("id", &id), ("version", &version), ("name", &name)] {
        if s.len() > HELLO_STR_MAX {
            die(
                5,
                &format!("self-report {what} over the wire cap ({} bytes)", s.len()),
            );
        }
    }
    (id, version, name)
}

/// START handling: run the embedded kernel's start, then round-trip the
/// plugin's own service through the SAME `NativeService` path the dl
/// placement uses (D69 bytes only). Returns the ACK status. A plugin that
/// registers no native service is not failed by the runner — there is
/// nothing here to verify; the kernel-side claim stays honest either way.
fn handle_start(kernel: &Kernel, id: &PluginId) -> i32 {
    if let Err(err) = kernel.start(id) {
        return verb_status(err);
    }
    let Some(svc) = kernel.registry().acquire::<NativeService>() else {
        return 0;
    };
    let echo = svc.invoke(PROBE_METHOD, PROBE_BYTES);
    let unknown_rejected = svc
        .invoke("no-such-method", b"x")
        .err()
        .is_some_and(|e| matches!(e, KernelError::ServiceCallFailed { .. }));
    if kernel.registry().release(&svc).is_err() {
        eprintln!("modukit-run: release pair failed");
    }
    match echo {
        Ok(bytes) if bytes == PROBE_BYTES && unknown_rejected => 0,
        Ok(_) => {
            eprintln!("modukit-run: service self-check mismatch");
            -1
        }
        Err(err) => {
            eprintln!("modukit-run: service self-check: {err}");
            -1
        }
    }
}

/// A non-zero plugin status rides the same int channel the dl placement uses
/// (D27 author code space); everything else is a runner-side -1 with a note.
fn verb_status(err: KernelError) -> i32 {
    match err {
        KernelError::PluginStartFailed { status } => status,
        other => {
            eprintln!("modukit-run: {other}");
            -1
        }
    }
}

fn ack(control: &mut UnixStream, status: i32) -> io::Result<()> {
    write_frame(control, MSG_ACK, &status.to_le_bytes())
}

fn run(artifact: &str, fd_num: i32) {
    if !Path::new(artifact).is_file() {
        die(3, &format!("artifact not found: {artifact}"));
    }
    // SAFETY: the `--fd=N` convention (modules/01 §2) — the supervising
    // parent injected the control-socket end as this inherited descriptor.
    let mut control = unsafe { UnixStream::from_raw_fd(fd_num) };

    let (id, version, name) = self_report(artifact);
    let hello = proc_wire::Hello {
        abi: ABI_VERSION,
        id: id.clone(),
        version: version.clone(),
        name,
    };
    if write_frame(&mut control, MSG_HELLO, &hello_payload(&hello)).is_err() {
        die(6, "control closed before the hello was read");
    }

    // The embedded host: the same kernel face the dl placement drives, in
    // this process. Nothing about the host table crosses the boundary
    // (01 §3 rule 2) — the wire carries lifecycle, not authority.
    let kernel = Kernel::new();
    let manifest = internal_manifest(artifact, id, version);
    let pid = match kernel.install(manifest) {
        Ok(id) => id,
        Err(err) => die(7, &format!("embedded install: {err}")),
    };
    if let Err(err) = kernel.resolve(&pid) {
        die(8, &format!("embedded resolve: {err}"));
    }

    loop {
        let (kind, _payload) = match read_frame(&mut control) {
            Ok(frame) => frame,
            // Parent hung up: leave cleanly (the supervised run is over).
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return,
            Err(err) => die(11, &format!("control read: {err}")),
        };
        match kind {
            MSG_START => {
                let status = handle_start(&kernel, &pid);
                if ack(&mut control, status).is_err() {
                    return;
                }
            }
            MSG_STOP => {
                let status = match kernel.stop(&pid) {
                    Ok(()) => 0,
                    Err(err) => verb_status(err),
                };
                // Best-effort reply: if the parent is gone the run ended anyway.
                let _ = ack(&mut control, status);
                return; // teardown complete: exit cooperatively (01 §3 rule 5)
            }
            // Forward-compatible silence: unknown frame types are ignored
            // until S2-M3 defines them (no speculative handling here).
            _ => {}
        }
    }
}

fn main() {
    let (artifact, fd_num) = parse_args();
    run(&artifact, fd_num);
}
