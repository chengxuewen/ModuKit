//! The `exe` arm of the closed executor set (S2-M1; modules/01 §1, D13 seam
//! extension). `ProcessExecutor` spawns the plugin executable as a
//! supervised child and speaks the `proc_wire` envelope over a socketpair
//! control channel; the crash-watch + restart-policy loop is `Supervisor`
//! (`proc_supervisor`).
//!
//! Scope honesty (stage-2-plan §4): the shadow-runner binary `modukit-run`
//! lands in S2-M2 — the M1 tests spawn stand-in executables directly. The
//! runner argv shape of modules/01 §2 is already honored: if the manifest
//! names a `runner`, it becomes the executable and `entry.artifact` rides as
//! the positional; otherwise the artifact itself is the executable. The
//! cross-process host-table dispatch is S2-M3 wire transport, so `start`
//! carries no authority through the process boundary (01 §3 rule 2).

use std::collections::HashMap;
use std::io;
use std::os::fd::OwnedFd;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use crate::abi::{ABI_VERSION, HostAbiV1};
use crate::error::KernelError;
use crate::executor::{Executor, LoadedPlugin};
use crate::manifest::{Manifest, PlacementKind};
use crate::proc_wire::MSG_HELLO;
use crate::proc_wire::{self, MSG_ACK, MSG_START, MSG_STOP};

/// The fd number the child's control end gets in M1: std cannot hand an
/// arbitrary fd across exec (that is the documented-unsafe libc-shim /
/// SCM_RIGHTS work of S2-M3), so the child end of the socketpair is
/// injected as stdin = fd 0 — the stdio form of the modules/01 §1 "wire
/// over socketpair/stdio" row. The `--fd=N` argv convention is already
/// the runner's; M2/M3 keep the convention, not the number.
const CHILD_CONTROL_FD: u32 = 0;

/// Read bound for the HELLO handshake (a peer that never speaks is a
/// crash, not an infinite wait).
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
/// Read bound for the START ack.
pub const START_ACK_TIMEOUT: Duration = Duration::from_secs(5);
/// Grace between the STOP message and SIGKILL (modules/01 §3 rule 5: `exe`
/// teardown is `kill`), and between a clean ack and the reap.
pub const STOP_KILL_GRACE: Duration = Duration::from_secs(1);
/// Spin interval of the bounded reap waits — only ever spent on a real OS
/// child process (zero-sleep discipline exemption, S2-M1 brief).
const REAP_POLL_INTERVAL: Duration = Duration::from_millis(2);

/// One supervised child plus its parent half of the control socketpair.
/// `run_gen` stamps the supervised RUN: a handle dropped at a respawn (the
/// kernel replaces `record.handle` on resolve) must never touch the new
/// generation's entry — id keys alone would erase it.
#[derive(Debug)]
pub(crate) struct LiveProc {
    pub(crate) child: Child,
    /// Parent half of the control socketpair. Taken the moment the child
    /// is reaped: a dead child's descriptor must not linger in the table
    /// (S2-M1 fd hygiene, plan §6 risk 1).
    pub(crate) control: Option<UnixStream>,
    pub(crate) run_gen: u64,
    /// `Some` once the child has been reaped — the crash-watch memory the
    /// handle reports back through the state machine. Reaping happens
    /// exactly once, at one doorway, which is what keeps zombies at zero.
    pub(crate) status: Option<ExitStatus>,
}

/// The shared supervision state: live children keyed by plugin id, plus
/// the monotonic run counter behind the generation stamps.
#[derive(Debug, Default)]
pub(crate) struct Shared {
    pub(crate) live: HashMap<String, LiveProc>,
    next_gen: u64,
}

/// The live-children table shared by `ProcessExecutor`, its handles and the
/// `Supervisor` (one table, three doors — the D21 posture).
#[derive(Debug, Clone, Default)]
pub(crate) struct Control(Arc<Mutex<Shared>>);

impl Control {
    pub(crate) fn lock(&self) -> MutexGuard<'_, Shared> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

fn is_timeout(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
    )
}

/// Crash observation carrying the reaped child's honest exit description.
fn crashed(id: &str, exit: impl Into<String>) -> KernelError {
    KernelError::ProcessCrashed {
        id: id.to_string(),
        exit: exit.into(),
    }
}

/// SIGKILL + blocking reap for a bare child (handshake-failure path: the
/// child never enters the live table, so it must die and be waited HERE).
fn kill_wait_child(child: &mut Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Kill + reap a table entry unless the crash-watch already reaped it.
fn kill_and_reap(p: &mut LiveProc) {
    if p.status.is_some() {
        return;
    }
    let _ = p.child.kill();
    p.status = p.child.wait().ok();
    let _ = p.control.take();
}

/// Bounded reap after a cooperative ack: poll `try_wait` until the grace
/// deadline, escalate to kill. The child is a real process — this is the
/// one sleep loop the M1 brief allows, and it never exceeds the constant.
fn reap_with_grace(p: &mut LiveProc, grace: Duration) {
    if p.status.is_some() {
        return;
    }
    let deadline = Instant::now() + grace;
    loop {
        match p.child.try_wait() {
            Ok(Some(status)) => {
                p.status = Some(status);
                let _ = p.control.take();
                return;
            }
            Ok(None) => {}
            Err(_) => return, // reaped elsewhere; nothing left to wait for
        }
        if Instant::now() >= deadline {
            kill_and_reap(p);
            return;
        }
        std::thread::sleep(REAP_POLL_INTERVAL);
    }
}

/// The `exe` executor (01 §1): placement-floor guard, spawn, handshake.
#[derive(Debug, Default)]
pub struct ProcessExecutor {
    live: Control,
}

impl ProcessExecutor {
    /// Standalone executor (kernel `with_executor` door). Crash-watch needs
    /// the shared table — boot through `Supervisor` for that half.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The supervised boot pair: the executor for the kernel seam plus the
    /// control table the `Supervisor` crash-watches (S2-M1 doorway).
    pub(crate) fn paired() -> (Self, Control) {
        let live = Control::default();
        (Self { live: live.clone() }, live)
    }
}

impl Executor for ProcessExecutor {
    fn load(&self, manifest: &Manifest) -> Result<Box<dyn LoadedPlugin + Send>, KernelError> {
        if manifest.entry.kind != PlacementKind::Exe {
            return Err(KernelError::UnsupportedPlacement {
                kind: manifest.entry.kind.to_string(),
            });
        }
        // Switchability preconditions (01 §3) enforceable from placement
        // data alone: a missing executable path is refused before any fork.
        // The remaining rules are tested properties of the contract suite —
        // the dual-placement runs land with the runner in S2-M2/M3; nothing
        // is checkable statically here and no machinery is built for them.
        let exe = match &manifest.entry.runner {
            Some(runner) => runner.clone(),
            None => manifest.entry.artifact.clone(),
        };
        if exe.is_empty() || !Path::new(&exe).is_file() {
            return Err(KernelError::ArtifactMissing { path: exe });
        }
        let (mut parent, child_end) = UnixStream::pair().map_err(|e| KernelError::LoadFailed {
            path: exe.clone(),
            reason: e.to_string(),
        })?;
        let mut cmd = Command::new(&exe);
        // modules/01 §2 command shape: `<runner> <plugin.so> --fd=N`; the
        // M1 stand-in executables are the artifact itself (no runner).
        if manifest.entry.runner.is_some() {
            cmd.arg(&manifest.entry.artifact);
        }
        cmd.arg(format!("--fd={CHILD_CONTROL_FD}"))
            .stdin(Stdio::from(OwnedFd::from(child_end)))
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = cmd.spawn().map_err(|e| KernelError::LoadFailed {
            path: exe.clone(),
            reason: e.to_string(),
        })?;

        // ABI v1 recheck over the wire + D7 cross-check, executed BEFORE
        // anything is marked live. Every failure path kills and reaps the
        // child right here: a rejected handshake never becomes a zombie or
        // a supervised run. Strict self-report: silence or a lie is
        // disagreement (bundle.rs precedent), never a partial start.
        let hello = match read_hello(&mut parent, manifest.id.as_str()) {
            Ok(hello) => hello,
            Err(err) => {
                kill_wait_child(&mut child);
                return Err(err);
            }
        };
        if hello.abi != ABI_VERSION {
            kill_wait_child(&mut child);
            return Err(KernelError::AbiMismatch {
                expected: ABI_VERSION,
                found: hello.abi,
            });
        }
        // Strict self-report, one field at a time (the `bundle.rs` liar
        // family): silence or a lie both land on the typed mismatch.
        let checks = [
            ("id", hello.id.as_str(), manifest.id.as_str()),
            ("version", hello.version.as_str(), manifest.version.as_str()),
        ];
        for (what, found, expected) in checks {
            if found != expected {
                kill_wait_child(&mut child);
                return Err(KernelError::IdentityMismatch {
                    what,
                    expected: expected.to_string(),
                    found: found.to_string(),
                });
            }
        }

        let mut shared = self.live.lock();
        // A respawn onto a still-watched id retires the previous generation
        // first: its child can never outlive its entry, and the OLD handle
        // (dropped when the kernel replaces it) sees the run_gen stamp moved
        // and keeps its hands off the new run in `Drop`.
        if let Some(old) = shared.live.get_mut(manifest.id.as_str()) {
            kill_and_reap(old);
        }
        shared.next_gen += 1;
        let run_gen = shared.next_gen;
        shared.live.insert(
            manifest.id.as_str().to_string(),
            LiveProc {
                child,
                control: Some(parent),
                run_gen,
                status: None,
            },
        );
        let handle = ProcPlugin {
            id: manifest.id.as_str().to_string(),
            name: hello.name,
            self_id: hello.id,
            self_version: hello.version,
            run_gen,
            live: self.live.clone(),
        };
        Ok(Box::new(handle))
    }
}

/// Read + decode the HELLO frame; timeout / EOF / malformation become typed
/// errors (the caller reaps).
fn read_hello(parent: &mut UnixStream, id: &str) -> Result<proc_wire::Hello, KernelError> {
    parent
        .set_read_timeout(Some(HANDSHAKE_TIMEOUT))
        .map_err(|e| crashed(id, format!("set timeout: {e}")))?;
    let (kind, payload) = match proc_wire::read_frame(parent) {
        Ok(frame) => frame,
        Err(err) if is_timeout(&err) => {
            return Err(KernelError::HandshakeIncomplete {
                what: "hello (timeout)",
            });
        }
        Err(err) => {
            return Err(crashed(id, format!("control closed before hello: {err}")));
        }
    };
    if kind != MSG_HELLO {
        return Err(KernelError::HandshakeIncomplete {
            what: "hello (wrong frame type)",
        });
    }
    proc_wire::parse_hello(&payload).ok_or(KernelError::HandshakeIncomplete {
        what: "hello (malformed payload)",
    })
}

/// One running `exe` plugin: verbs ride the control channel; the child
/// itself lives in the shared live table (one source for the supervisor).
/// `run_gen` pins the verbs to THIS supervised run.
struct ProcPlugin {
    id: String,
    name: String,
    self_id: String,
    self_version: String,
    run_gen: u64,
    live: Control,
}

impl LoadedPlugin for ProcPlugin {
    fn plugin_name(&self) -> &str {
        &self.name
    }

    fn self_report(&self) -> (Option<String>, Option<String>) {
        // The validated HELLO strings (04 §3 self-report tier, `exe` flavor).
        (Some(self.self_id.clone()), Some(self.self_version.clone()))
    }

    fn start(&mut self, host: *const HostAbiV1) -> Result<(), KernelError> {
        // The host table is the in-process `dl` shape and deliberately does
        // NOT cross the process boundary (01 §3 rule 2: nothing ambient is
        // inherited). Cross-process service dispatch rides the wire
        // transport of S2-M3 — no speculative machinery here.
        let _ = host;
        let mut shared = self.live.lock();
        let p = self.entry(&mut shared)?;
        let control_io = |e: io::Error| crashed(&self.id, format!("control io: {e}"));
        p.control
            .as_mut()
            .ok_or_else(|| crashed(&self.id, "control closed"))?
            .set_read_timeout(Some(START_ACK_TIMEOUT))
            .map_err(control_io)?;
        proc_wire::write_frame(
            p.control
                .as_mut()
                .ok_or_else(|| crashed(&self.id, "control closed"))?,
            MSG_START,
            &[],
        )
        .map_err(|e| {
            kill_and_reap(p);
            control_io(e)
        })?;
        let status = match proc_wire::read_frame(
            p.control
                .as_mut()
                .ok_or_else(|| crashed(&self.id, "control closed"))?,
        ) {
            Ok((MSG_ACK, payload)) => proc_wire::parse_ack(&payload).unwrap_or(i32::MIN),
            Ok(_) => {
                kill_and_reap(p);
                return Err(crashed(&self.id, "start reply: wrong frame type"));
            }
            Err(err) if is_timeout(&err) => {
                kill_and_reap(p);
                return Err(crashed(&self.id, "start ack timeout, killed"));
            }
            Err(err) => {
                kill_and_reap(p);
                return Err(crashed(&self.id, format!("start read: {err}")));
            }
        };
        if status != 0 {
            // The child refused the start but is alive and cooperative —
            // same typed channel as the `dl` non-zero start (03 §2).
            return Err(KernelError::PluginStartFailed { status });
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<(), KernelError> {
        let mut shared = self.live.lock();
        let p = self.entry(&mut shared)?;
        let control_io = |e: io::Error| crashed(&self.id, format!("control io: {e}"));
        p.control
            .as_mut()
            .ok_or_else(|| crashed(&self.id, "control closed"))?
            .set_read_timeout(Some(STOP_KILL_GRACE))
            .map_err(control_io)?;
        // Graceful STOP message first; a named grace, then SIGKILL
        // (modules/01 §3 rule 5: `exe` teardown is kill).
        proc_wire::write_frame(
            p.control
                .as_mut()
                .ok_or_else(|| crashed(&self.id, "control closed"))?,
            MSG_STOP,
            &[],
        )
        .map_err(|e| {
            kill_and_reap(p);
            control_io(e)
        })?;
        match proc_wire::read_frame(
            p.control
                .as_mut()
                .ok_or_else(|| crashed(&self.id, "control closed"))?,
        ) {
            Ok((MSG_ACK, payload)) => {
                let status = proc_wire::parse_ack(&payload).unwrap_or(i32::MIN);
                reap_with_grace(p, STOP_KILL_GRACE);
                if status != 0 {
                    // Cooperative-but-failed stop: same typed channel as dl.
                    return Err(KernelError::PluginStartFailed { status });
                }
                Ok(())
            }
            Ok(_) => {
                kill_and_reap(p);
                Err(crashed(&self.id, "stop reply: wrong frame type"))
            }
            Err(err) if is_timeout(&err) => {
                kill_and_reap(p);
                Err(crashed(&self.id, "stop timeout, killed"))
            }
            Err(err) => {
                kill_and_reap(p);
                Err(crashed(&self.id, format!("stop read: {err}")))
            }
        }
    }
}

impl ProcPlugin {
    /// This generation's live entry, reporting crash observations the
    /// crash-watch already reaped (the STOPPING -> FAILED memory, 01 §4:
    /// bounded and observable, never a second reap or socket ping-pong).
    /// A handle whose generation was superseded never touches the table.
    fn entry<'s>(&self, shared: &'s mut Shared) -> Result<&'s mut LiveProc, KernelError> {
        let p = shared
            .live
            .get_mut(&self.id)
            .ok_or_else(|| crashed(&self.id, "already gone"))?;
        if p.run_gen != self.run_gen {
            return Err(crashed(&self.id, "superseded generation"));
        }
        if let Some(status) = &p.status {
            return Err(crashed(&self.id, status.to_string()));
        }
        Ok(p)
    }
}

impl Drop for ProcPlugin {
    fn drop(&mut self) {
        // The leak-proof last line: an `exe` handle leaving the kernel's
        // table without a stop (uninstall path) kills its child and
        // releases the control half — but only while its own generation
        // still owns the entry (a dropped predecessor at respawn must
        // leave the new run alone). Idempotent when already reaped.
        let mut shared = self.live.lock();
        if shared
            .live
            .get(&self.id)
            .is_some_and(|p| p.run_gen == self.run_gen)
            && let Some(mut p) = shared.live.remove(&self.id)
            && p.status.is_none()
        {
            kill_and_reap(&mut p);
        }
    }
}
