//! modukit-run — the shadow-runner (S2-M2; modules/01 §2).
//!
//! `exe` placement does not require a separate executable per plugin: the
//! host spawns this generic runner with the SAME artifact the `dl` executor
//! would load plus the inherited control fd —
//!
//! ```text
//! modukit-run <plugin.so> --fd=N [--clock=step|sim|real]
//! ```
//!
//! (N = 0 today: `ProcessExecutor` injects the child end of the socketpair as
//! stdin, executor_proc.rs `CHILD_CONTROL_FD`; the `--fd=N` convention is
//! honored, not the number.) The runner `dlopen`s the artifact through
//! `DlopenExecutor` — one source of truth for the ABI v1 guards — embeds a
//! kernel of its own as the in-process host, and pumps the contract over the
//! fd with the `proc_wire` envelope: HELLO (the plugin's OWN self-report,
//! D7 tier) on connect, START -> `kernel.start` -> REGISTER -> ACK,
//! CALL -> dispatch -> RESULT (S2-M3a proxy lane), DIAL -> clock turn ->
//! DUE (D74 replay track; refused typed on `--clock=real`), STOP ->
//! `kernel.stop` -> ACK, clean exit when the control socket EOFs (the
//! parent is gone).
//!
//! Error posture (bin, application-level): every fallible setup step is
//! handled explicitly and lands on a nonzero exit code with a one-line
//! stderr note; a rejected artifact dies BEFORE it can speak a HELLO, so the
//! parent observes the typed crash family, never a lie. Exit codes: 2 usage,
//! 3 artifact, 4 dlopen guards, 5 self-report, 6 hello write, 7..9 embedded-
//! kernel setup, 11 control read, 12 foreign frame version, 13/14 malformed
//! CALL/DIAL frame.

use std::env;
use std::io;
use std::os::fd::FromRawFd;
use std::os::unix::net::UnixStream;
use std::path::Path;

use modukit_core::abi::{ABI_VERSION, NativeService};
use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::error::KernelError;
use modukit_core::executor::{DlopenExecutor, Executor};
use modukit_core::identity::PluginId;
use modukit_core::kernel::Kernel;
use modukit_core::lifecycle::RestartPolicy;
use modukit_core::manifest::{Entry, Manifest, PlacementKind};
use modukit_core::proc_wire::{
    self, DIAL_STATUS_REFUSED_REAL, DueTurn, MSG_ACK, MSG_CALL, MSG_DIAL, MSG_DUE, MSG_HELLO,
    MSG_REGISTER, MSG_RESULT, MSG_START, MSG_STOP, ServiceRegistration, due_payload, hello_payload,
    parse_call, parse_dial, read_frame, register_payload, result_payload, write_frame,
};
use modukit_core::testhost::schedule_replay_markers;

/// One handshake string must fit the wire's one-byte length prefix
/// (`proc_wire::parse_hello` reads `u8` counts — the cap is the format).
const HELLO_STR_MAX: usize = u8::MAX as usize;

/// The `NativeService` probe the runner round-trips inside its own process
/// after `start` (D69: bytes only) — kept as an INTERNAL guard: a plugin
/// whose template contract broke never acks START. The kernel-observable
/// path is no longer gated on this: S2-M3a announces the verified service
/// via REGISTER and dispatches CALL/RESULT across the boundary (F2
/// closed; friction log item 5 dissolved — the CALL frame carries the
/// caller's method name).
const PROBE_METHOD: &str = "echo";
const PROBE_BYTES: &[u8] = b"shadow-runner-selfcheck";

fn die(code: i32, msg: &str) -> ! {
    eprintln!("modukit-run: {msg}");
    std::process::exit(code)
}

fn parse_args() -> (String, i32, ClockDomain) {
    let mut artifact: Option<String> = None;
    let mut fd_num: i32 = 0;
    // D74 (S2-M3b): the runner boots the test/replay track — the embedded
    // clock defaults to Step so the supervisor's wire dial is the sole time
    // authority (zero-sleep discipline across the boundary). Production
    // placements opt out with `--clock=real`; there the dial is refused,
    // never silently swallowed.
    let mut domain = ClockDomain::Step;
    for arg in env::args().skip(1) {
        if let Some(rest) = arg.strip_prefix("--fd=") {
            fd_num = match rest.parse::<i32>() {
                Ok(n) if n >= 0 => n,
                _ => die(2, &format!("bad --fd value: {rest}")),
            };
        } else if let Some(rest) = arg.strip_prefix("--clock=") {
            domain = match rest {
                "step" => ClockDomain::Step,
                "sim" => ClockDomain::Sim,
                "real" => ClockDomain::Real,
                other => die(2, &format!("bad --clock value: {other}")),
            };
        } else if arg.starts_with("--") {
            die(2, &format!("unknown option {arg}"));
        } else if artifact.is_none() {
            artifact = Some(arg);
        } else {
            die(
                2,
                "usage: modukit-run <plugin.so> --fd=N [--clock=step|sim|real]",
            );
        }
    }
    match artifact {
        Some(path) => (path, fd_num, domain),
        None => die(
            2,
            "usage: modukit-run <plugin.so> --fd=N [--clock=step|sim|real]",
        ),
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

/// START handling: run the embedded kernel's start, round-trip the
/// plugin's own service through the SAME `NativeService` path the dl
/// placement uses (D69 bytes only, now the internal guard), then
/// announce the verified service with a REGISTER frame BEFORE the ack —
/// the parent installs a kernel-side proxy from it (S2-M3a). Returns the
/// ACK status. A plugin that registers no native service is not failed
/// by the runner — there is nothing here to verify or announce.
fn handle_start(kernel: &Kernel, id: &PluginId, control: &mut UnixStream) -> i32 {
    if let Err(err) = kernel.start(id) {
        return verb_status(err);
    }
    let Some(svc) = kernel.registry().acquire::<NativeService>() else {
        return 0;
    };
    let announced = svc.service_id().as_str().to_string();
    let echo = svc.invoke(PROBE_METHOD, PROBE_BYTES);
    let unknown_rejected = svc
        .invoke("no-such-method", b"x")
        .err()
        .is_some_and(|e| matches!(e, KernelError::ServiceCallFailed { .. }));
    if kernel.registry().release(&svc).is_err() {
        eprintln!("modukit-run: release pair failed");
    }
    match echo {
        Ok(bytes) if bytes == PROBE_BYTES && unknown_rejected => {
            let reg = ServiceRegistration {
                service_id: announced,
                // shortcut: ABI v1 has no method introspection — advertise
                // exactly what the self-check round-tripped (the template's
                // echo); generated contracts own the list (04 §1).
                methods: vec![PROBE_METHOD.to_string()],
            };
            if write_frame(control, MSG_REGISTER, &register_payload(&reg)).is_err() {
                eprintln!("modukit-run: control closed before the register was read");
                return -1;
            }
            0
        }
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

fn run(artifact: &str, fd_num: i32, domain: ClockDomain) {
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
    let kernel = Kernel::with_clock(Clock::new(domain));
    let manifest = internal_manifest(artifact, id, version);
    let pid = match kernel.install(manifest) {
        Ok(id) => id,
        Err(err) => die(7, &format!("embedded install: {err}")),
    };
    if let Err(err) = kernel.resolve(&pid) {
        die(8, &format!("embedded resolve: {err}"));
    }

    // The D74 replay scenario (D17 test-2): the v0 marker pair both
    // placements schedule identically, dialed by the testhost in-process on
    // the `dl` arm and over the wire on this one. shortcut: ABI v1 has no
    // clock verbs for plugins — the markers stand in for plugin-side timer
    // jobs until the surface hearing (same missing-face class as the bus).
    if domain != ClockDomain::Real {
        schedule_replay_markers(kernel.clock());
    }

    loop {
        let (ver, kind, _payload) = match read_frame(&mut control) {
            Ok(frame) => frame,
            // Parent hung up: leave cleanly (the supervised run is over).
            Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => return,
            Err(err) => die(11, &format!("control read: {err}")),
        };
        // A foreign envelope version is a protocol disagreement: refuse it
        // typed (exit 12) and die here — never reinterpret a v2 frame as v1
        // (S2-M3 frame versioning, mirrors the parent's read_hello check).
        if ver != proc_wire::WIRE_VERSION {
            die(
                12,
                &format!("control frame wire version {ver}, runner frames v1"),
            );
        }
        match kind {
            MSG_START => {
                let status = handle_start(&kernel, &pid, &mut control);
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
            MSG_CALL => {
                // Kernel-side proxy dispatch (S2-M3a): answer exactly one
                // CALL, echoing the seq back (04 §2 request/response). The
                // plugin's own code is the method authority — the runner
                // never pre-rejects, so the rejection is byte-equal with the
                // dl arm; -1 is reserved for "no native service here".
                let Some(call) = parse_call(&_payload) else {
                    die(13, "malformed CALL frame");
                };
                let (status, output) = match kernel.registry().acquire::<NativeService>() {
                    Some(svc) => {
                        let outcome = svc.invoke(&call.method, &call.input);
                        if kernel.registry().release(&svc).is_err() {
                            eprintln!("modukit-run: release pair failed");
                        }
                        match outcome {
                            Ok(bytes) => (0i32, bytes),
                            Err(KernelError::ServiceCallFailed { code }) => (code, Vec::new()),
                            Err(err) => {
                                eprintln!("modukit-run: call {err}");
                                (-1, Vec::new())
                            }
                        }
                    }
                    None => {
                        eprintln!("modukit-run: CALL but no native service registered");
                        (-1, Vec::new())
                    }
                };
                if write_frame(
                    &mut control,
                    MSG_RESULT,
                    &result_payload(call.seq, status, &output),
                )
                .is_err()
                {
                    return;
                }
            }
            MSG_DIAL => {
                // D74: the supervisor owns the dial on this track. Posting
                // rides the embedded kernel's own doorway (timers-as-jobs);
                // the fired batch returns as answer DATA, never callbacks.
                let Some(delta) = parse_dial(&_payload) else {
                    die(14, "malformed DIAL frame");
                };
                let turn = if kernel.clock().domain() == ClockDomain::Real {
                    DueTurn {
                        status: DIAL_STATUS_REFUSED_REAL,
                        fired_at_ns: 0,
                        jobs: Vec::new(),
                    }
                } else {
                    // The stamp is read AFTER the turn — struct field order
                    // is evaluation order, and the answer must carry the
                    // post-dial clock, not the pre-dial one.
                    let jobs = kernel.dial_clock(delta);
                    DueTurn {
                        status: 0,
                        fired_at_ns: kernel.clock().now().raw(),
                        jobs,
                    }
                };
                if write_frame(&mut control, MSG_DUE, &due_payload(&turn)).is_err() {
                    return;
                }
            }
            // Forward-compatible silence: unknown frame types are ignored
            // (no speculative handling before a speaker exists for them).
            _ => {}
        }
    }
}

fn main() {
    let (artifact, fd_num, domain) = parse_args();
    run(&artifact, fd_num, domain);
}
