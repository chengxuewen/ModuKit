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
//! cross-process host-table dispatch is S2-M3a wire transport: the child's
//! REGISTER frames install kernel-side proxies (`proc_proxy`), and calls ride
//! as CALL/RESULT message bytes — `start` still carries no authority through
//! the process boundary (01 §3 rule 2).

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
use crate::identity::ServiceId;
use crate::manifest::{Manifest, PlacementKind};
use crate::proc_proxy::ProxyService;
use crate::proc_wire::MSG_HELLO;
use crate::proc_wire::{self, MSG_ACK, MSG_REGISTER, MSG_START, MSG_STOP};
use crate::registry::{Properties, PropertyValue, RegistrationId, Registry, RegistryError};

/// The fd number the child's control end gets in M1: std cannot hand an
/// arbitrary fd across exec (that is the documented-unsafe libc-shim work of
/// S2-M3), so the child end of the socketpair is injected as stdin = fd 0 —
/// the stdio form of the modules/01 §1 "wire over socketpair/stdio" row.
/// The `--fd=N` argv convention is already the runner's; M2/M3 keep the
/// convention, not the number. E2 disposition: the control channel STAYS
/// fd-0/stdio (01 §1 shape) — SCM_RIGHTS passes BUFFER fds riding the
/// channel, it never renumbers this control fd.
const CHILD_CONTROL_FD: u32 = 0;

/// Read bound for the HELLO handshake (a peer that never speaks is a
/// crash, not an infinite wait).
pub const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(5);
/// Read bound for the START ack and the REGISTER-before-ack exchange.
pub const START_ACK_TIMEOUT: Duration = Duration::from_secs(5);
/// Read bound for one proxied service CALL's RESULT (S2-M3a): the child
/// runs plugin code synchronously, so the budget matches the handshake/
/// START family — a peer slower than that is hung, not busy, and is
/// killed on the timeout edge (the `dl` arm cannot time out at all; this
/// is the honest cross-process cost of 01 §3 rule 5 teardown).
pub const CALL_RESULT_TIMEOUT: Duration = Duration::from_secs(5);
/// Read bound for one dial turn's DUE answer (D74, S2-M3b): the plan's
/// timeout guard against a blocked runner — a child that never answers the
/// dial is killed at this bound, never held open.
pub const DIAL_TIMEOUT: Duration = Duration::from_secs(5);
/// Grace between the STOP message and SIGKILL (modules/01 §3 rule 5: `exe`
/// teardown is `kill`), and between a clean ack and the reap.
pub const STOP_KILL_GRACE: Duration = Duration::from_secs(1);
/// Linux `ETXTBSY` (`Text file busy`): execve refuses an inode while a write
/// reference it has not yet dropped still exists.
const LINUX_ETXTBSY: i32 = 26;
/// Extra attempts at the spawn doorway when execve returns ETXTBSY. The
/// kernel's close() returns before the delayed __fput drops the last write
/// reference on the executable's inode (installer copy/chmod/rename a few
/// milliseconds earlier), and execve inside that window fails ETXTBSY with
/// NO fd to the inode visible anywhere in /proc - probe-proven 2026-10-19
/// (holders=0 at failure, same execve succeeds ~20ms later). Go's os/exec
/// retries the same race at spawn; same bounded posture adopted here.
const SPAWN_ETXTBSY_RETRY_BOUND: u32 = 3;
/// Sleep between the bounded ETXTBSY retries - the observed __fput-lag
/// window is single-digit milliseconds; 50ms per step clears it with margin.
const SPAWN_ETXTBSY_RETRY_DELAY: Duration = Duration::from_millis(50);
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
    /// The kernel's registry door (S2-M3a): set once at Supervisor boot;
    /// proxies for REGISTERed services install through it. A standalone
    /// `ProcessExecutor::new()` has no door — a child that REGISTERs there
    /// is refused at start with a typed crash (no silent half-run).
    pub(crate) registry: Option<Registry>,
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

pub(crate) fn is_timeout(err: &io::Error) -> bool {
    matches!(
        err.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
    )
}

/// Crash observation carrying the reaped child's honest exit description.
pub(crate) fn crashed(id: &str, exit: impl Into<String>) -> KernelError {
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
pub(crate) fn kill_and_reap(p: &mut LiveProc) {
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
        // F1 fix (S2-M2): the descriptor passed to `Stdio::from(OwnedFd)`
        // stays open until the Command binding drops. The old shape kept the
        // builder alive across `read_hello`, so the parent retained a second
        // copy of the child control-end: a child that died before HELLO
        // never yielded EOF (the last write reference was the parent's own
        // leaked descriptor) and every silent death was misclassified at
        // HANDSHAKE_TIMEOUT. Spawning inside a block drops the builder -
        // and the injected copy with it - before the handshake read, so
        // peer death closes the last reference and EOF is observable.
        // Regression proof: dual_placement.rs silent_death_* (the sub-second
        // EOF bound doubles as the retention probe: count-based hygiene
        // checks cannot see this leak because the number is recycled).
        let mut child = {
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
            // The retry loop stays inside this block so the F1 invariant
            // holds: the builder - with the injected child_end - is dropped
            // before the handshake read whether the first attempt or the
            // last one succeeded.
            let mut attempts = 0u32;
            loop {
                match cmd.spawn() {
                    Ok(child) => break child,
                    Err(e)
                        if e.raw_os_error() == Some(LINUX_ETXTBSY)
                            && attempts < SPAWN_ETXTBSY_RETRY_BOUND =>
                    {
                        attempts += 1;
                        std::thread::sleep(SPAWN_ETXTBSY_RETRY_DELAY);
                    }
                    Err(e) => {
                        return Err(KernelError::LoadFailed {
                            path: exe.clone(),
                            reason: e.to_string(),
                        });
                    }
                }
            }
        };

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
            proxy_rids: Vec::new(),
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
    let (ver, kind, payload) = match proc_wire::read_frame(parent) {
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
    // A foreign envelope version is a protocol disagreement, checked before
    // anything is decoded from the payload (S2-M3 frame versioning).
    if ver != proc_wire::WIRE_VERSION {
        return Err(KernelError::WireVersionMismatch {
            expected: proc_wire::WIRE_VERSION,
            found: ver,
        });
    }
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
    /// Registration ids of the kernel-side proxies installed from the
    /// child's REGISTER frames (S2-M3a) — torn down with the run.
    proxy_rids: Vec<RegistrationId>,
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
        // inherited). S2-M3a: the child's REGISTER frames install kernel-side
        // PROXIES; calls ride as CALL/RESULT message bytes (D69, 04 §2).
        let _ = host;
        // Clone the table handle before locking: the proxy installs below
        // need `&mut self` while the guard is alive (a guard would borrow
        // `self.live` for its whole region).
        let live = self.live.clone();
        let mut shared = live.lock();
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
        // S2-M3a: the child answers with zero or more REGISTER frames and
        // then the START ack. Each read is bounded by START_ACK_TIMEOUT
        // (a REGISTER-streaming child is throttled by FRAME_MAX per frame).
        // shortcut: per-frame bound, no total exchange budget yet — the
        // M3b host loop owns the pump that can bound the whole exchange.
        let status;
        let mut registrations = Vec::new();
        loop {
            match proc_wire::read_frame(
                p.control
                    .as_mut()
                    .ok_or_else(|| crashed(&self.id, "control closed"))?,
            ) {
                Ok((proc_wire::WIRE_VERSION, MSG_ACK, payload)) => {
                    status = proc_wire::parse_ack(&payload).unwrap_or(i32::MIN);
                    break;
                }
                Ok((proc_wire::WIRE_VERSION, MSG_REGISTER, payload)) => {
                    let Some(reg) = proc_wire::parse_register(&payload) else {
                        kill_and_reap(p);
                        return Err(crashed(&self.id, "register: malformed payload"));
                    };
                    registrations.push(reg);
                }
                Ok((proc_wire::WIRE_VERSION, _, _)) => {
                    kill_and_reap(p);
                    return Err(crashed(&self.id, "start reply: wrong frame type"));
                }
                Ok((found, _, _)) => {
                    kill_and_reap(p);
                    return Err(KernelError::WireVersionMismatch {
                        expected: proc_wire::WIRE_VERSION,
                        found,
                    });
                }
                Err(err) if is_timeout(&err) => {
                    kill_and_reap(p);
                    return Err(crashed(&self.id, "start ack timeout, killed"));
                }
                Err(err) => {
                    kill_and_reap(p);
                    return Err(crashed(&self.id, format!("start read: {err}")));
                }
            }
        }
        if status != 0 {
            // The child refused the start but is alive and cooperative —
            // same typed channel as the `dl` non-zero start (03 §2). Any
            // announcements die with the refused start: nothing installs.
            return Err(KernelError::PluginStartFailed { status });
        }
        if !registrations.is_empty() {
            self.install_proxies(&mut shared, registrations)?;
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<(), KernelError> {
        let live = self.live.clone();
        let mut shared = live.lock();
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
        let outcome = match proc_wire::read_frame(
            p.control
                .as_mut()
                .ok_or_else(|| crashed(&self.id, "control closed"))?,
        ) {
            Ok((proc_wire::WIRE_VERSION, MSG_ACK, payload)) => {
                let status = proc_wire::parse_ack(&payload).unwrap_or(i32::MIN);
                reap_with_grace(p, STOP_KILL_GRACE);
                if status != 0 {
                    // Cooperative-but-failed stop: same typed channel as dl.
                    Err(KernelError::PluginStartFailed { status })
                } else {
                    Ok(())
                }
            }
            Ok((proc_wire::WIRE_VERSION, _, _)) => {
                kill_and_reap(p);
                Err(crashed(&self.id, "stop reply: wrong frame type"))
            }
            Ok((found, _, _)) => {
                kill_and_reap(p);
                Err(KernelError::WireVersionMismatch {
                    expected: proc_wire::WIRE_VERSION,
                    found,
                })
            }
            Err(err) if is_timeout(&err) => {
                kill_and_reap(p);
                Err(crashed(&self.id, "stop timeout, killed"))
            }
            Err(err) => {
                kill_and_reap(p);
                Err(crashed(&self.id, format!("stop read: {err}")))
            }
        };
        // S2-M3a: the proxied services leave the kernel registry with the
        // run — whatever the ack outcome (the crash arms are also covered
        // by Drop; dl bookkeeping parity, 03 §2 "services gone").
        self.drop_proxies(&mut shared);
        outcome
    }
}

impl ProcPlugin {
    /// This generation's live entry, reporting crash observations the
    /// crash-watch already reaped (the STOPPING -> FAILED memory, 01 §4:
    /// bounded and observable, never a second reap or socket ping-pong).
    /// A handle whose generation was superseded never touches the table.
    fn entry<'s>(&self, shared: &'s mut Shared) -> Result<&'s mut LiveProc, KernelError> {
        live_entry(shared, &self.id, self.run_gen)
    }

    /// Install one kernel-side proxy per REGISTER frame through the SAME
    /// registry door the dl trampoline uses (`Registry::register`, D69:
    /// message bytes in and out, no pointer crosses). On any refusal the
    /// partial install is rolled back — no half-registered service.
    fn install_proxies(
        &mut self,
        shared: &mut Shared,
        registrations: Vec<proc_wire::ServiceRegistration>,
    ) -> Result<(), KernelError> {
        let Some(registry) = shared.registry.clone() else {
            if let Ok(p) = live_entry(shared, &self.id, self.run_gen) {
                kill_and_reap(p);
            }
            return Err(crashed(
                &self.id,
                "service REGISTER with no kernel registry door (boot via Supervisor)",
            ));
        };
        for reg in registrations {
            if reg.service_id.is_empty() {
                self.drop_proxies(shared);
                return Err(KernelError::Registry(RegistryError::MissingServiceId));
            }
            let mut props = Properties::new();
            props.insert("service.id", PropertyValue::Str(reg.service_id.clone()));
            let proxy = Arc::new(ProxyService::new(
                ServiceId::new(reg.service_id.clone()),
                reg.methods,
                self.id.clone(),
                self.run_gen,
                self.live.clone(),
            ));
            match registry.register(proxy, props) {
                Ok(rid) => self.proxy_rids.push(rid),
                Err(e) => {
                    self.drop_proxies(shared);
                    return Err(e.into());
                }
            }
        }
        Ok(())
    }

    /// Remove this run's proxies from the kernel registry. Idempotent:
    /// an already-removed registration is not an error (the kernel's own
    /// stop bookkeeping uses the same tolerance, 03 §2 services gone).
    fn drop_proxies(&mut self, shared: &mut Shared) {
        let Some(registry) = shared.registry.clone() else {
            self.proxy_rids.clear();
            return;
        };
        for rid in self.proxy_rids.drain(..) {
            let _ = registry.unregister(rid);
        }
    }
}

/// Turn one child's dial (D74, S2-M3b): write `DIAL`, await the `DUE`
/// answer under `DIAL_TIMEOUT`. The lane serializes under the table mutex
/// (v0: one in-flight verb per child). A refusal (non-zero status) keeps
/// the child ALIVE — a Real-domain runner is cooperative, just undialable;
/// protocol death gets the same kill-and-reap treatment as the verbs.
pub(crate) fn dial_child(
    live: &Control,
    id: &str,
    delta: Duration,
) -> Result<proc_wire::DueTurn, KernelError> {
    let mut shared = live.lock();
    let p = shared
        .live
        .get_mut(id)
        .ok_or_else(|| crashed(id, "already gone"))?;
    if let Some(status) = &p.status {
        return Err(crashed(id, status.to_string()));
    }
    let sock = p
        .control
        .as_mut()
        .ok_or_else(|| crashed(id, "control closed"))?;
    sock.set_read_timeout(Some(DIAL_TIMEOUT))
        .map_err(|e| crashed(id, format!("set timeout: {e}")))?;
    if proc_wire::write_frame(sock, proc_wire::MSG_DIAL, &proc_wire::dial_payload(delta)).is_err() {
        kill_and_reap(p);
        return Err(crashed(id, "dial write: control closed"));
    }
    let turn = match proc_wire::read_frame(sock) {
        Ok((proc_wire::WIRE_VERSION, proc_wire::MSG_DUE, payload)) => {
            let Some(turn) = proc_wire::parse_due(&payload) else {
                kill_and_reap(p);
                return Err(crashed(id, "dial reply: malformed DUE payload"));
            };
            turn
        }
        Ok((proc_wire::WIRE_VERSION, _, _)) => {
            kill_and_reap(p);
            return Err(crashed(id, "dial reply: wrong frame type"));
        }
        Ok((found, _, _)) => {
            kill_and_reap(p);
            return Err(KernelError::WireVersionMismatch {
                expected: proc_wire::WIRE_VERSION,
                found,
            });
        }
        Err(err) if is_timeout(&err) => {
            kill_and_reap(p);
            return Err(crashed(id, "dial timeout, killed"));
        }
        Err(err) => {
            kill_and_reap(p);
            return Err(crashed(id, format!("dial read: {err}")));
        }
    };
    if turn.status != 0 {
        // Typed refusal on a live, cooperative child — no kill, no reap.
        return Err(KernelError::ClockDialRefused {
            status: turn.status,
        });
    }
    Ok(turn)
}

/// The generation-pinned lookup shared by the verbs and the proxies:
/// `id` alone would let a stale handle touch the respawned run.
pub(crate) fn live_entry<'s>(
    shared: &'s mut Shared,
    id: &str,
    run_gen: u64,
) -> Result<&'s mut LiveProc, KernelError> {
    let p = shared
        .live
        .get_mut(id)
        .ok_or_else(|| crashed(id, "already gone"))?;
    if p.run_gen != run_gen {
        return Err(crashed(id, "superseded generation"));
    }
    if let Some(status) = &p.status {
        return Err(crashed(id, status.to_string()));
    }
    Ok(p)
}

impl Drop for ProcPlugin {
    fn drop(&mut self) {
        // The leak-proof last line: an `exe` handle leaving the kernel's
        // table without a stop (uninstall path) kills its child and
        // releases the control half — but only while its own generation
        // still owns the entry (a dropped predecessor at respawn must
        // leave the new run alone). Idempotent when already reaped.
        let live = self.live.clone();
        let mut shared = live.lock();
        // S2-M3a: proxies leave with the handle whatever killed the run
        // (stop removed them already — unregister is idempotent).
        self.drop_proxies(&mut shared);
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
#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_dir(label: &str) -> std::path::PathBuf {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("mk-etxtbsy-{label}-{}-{ts}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    fn true_copy(dir: &Path) -> std::path::PathBuf {
        let exe = dir.join("fake-plugin");
        std::fs::copy("/bin/true", &exe).expect("copy /bin/true");
        exe
    }

    fn manifest_for(exe: &Path) -> Manifest {
        Manifest::from_toml_str(&format!(
            "id = \"com.example.etxtbsy.probe\"\nversion = \"0.0.1\"\n\
             [entry]\nkind = \"exe\"\nartifact = \"{}\"\n",
            exe.display()
        ))
        .expect("manifest parses")
    }

    #[test]
    fn spawn_door_retries_transient_etxtbsy_then_reaches_handshake() {
        // A held write fd makes the first spawn attempt deterministically
        // ETXTBSY. Released inside the retry budget, the load must get
        // past the spawn doorway and fail later at the handshake EOF of
        // the instantly-exiting `/bin/true` - never as "Text file busy".
        let dir = scratch_dir("transient");
        let exe = true_copy(&dir);
        let keep = std::fs::OpenOptions::new()
            .write(true)
            .open(&exe)
            .expect("hold write fd");
        let releaser = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(30));
            drop(keep);
        });
        let err = ProcessExecutor::new()
            .load(&manifest_for(&exe))
            .err()
            .expect("/bin/true never speaks hello");
        releaser.join().expect("releaser");
        let msg = err.to_string();
        assert!(
            !msg.contains("Text file busy"),
            "bounded retry should have absorbed the transient ETXTBSY: {msg}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn spawn_door_reports_etxtbsy_after_retry_bound() {
        // A write fd held past the whole retry budget: the doorway must
        // still surface the typed LoadFailed with the ETXTBSY text, having
        // spent its bound doing so.
        let dir = scratch_dir("bound");
        let exe = true_copy(&dir);
        let keep = std::fs::OpenOptions::new()
            .write(true)
            .open(&exe)
            .expect("hold write fd");
        let t0 = Instant::now();
        let err = ProcessExecutor::new()
            .load(&manifest_for(&exe))
            .err()
            .expect("write fd held");
        let waited = t0.elapsed();
        drop(keep);
        let KernelError::LoadFailed { reason, .. } = &err else {
            panic!("held write fd must stay in the LoadFailed family: {err:?}");
        };
        assert!(reason.contains("Text file busy"), "reason was: {reason}");
        assert!(
            waited >= SPAWN_ETXTBSY_RETRY_DELAY * 2,
            "retry budget was not spent: {waited:?}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
