//! The supervisor loop over the `exe` executor seam (S2-M1): crash-watch,
//! the D3 restart policy, and FAILED edges driven through the EXISTING
//! state machine. No phantom reloads: a respawn rides only the legal
//! `FAILED -> RESOLVING` edge, and the crash surfaces as the ordinary
//! `STOPPING -> FAILED` pair the kernel's stop path already emits (03 §2).
//!
//! D21 posture: `poll` and `revive` ARE the new doorways (they are code
//! that had to exist), and the `process_crashes` / `process_restarts`
//! counters increment inside them where the work already runs — nothing
//! else grows to observe this.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use crate::clock::{Clock, Timestamp};
use crate::error::KernelError;
use crate::executor_proc::{Control, ProcessExecutor};
use crate::identity::PluginId;
use crate::kernel::Kernel;
use crate::lifecycle::{PluginState, RestartPolicy};

/// The supervisor's boot answer: one production `Kernel` wired with the
/// `exe` executor, the shared live table, and the per-plugin budget memory.
pub struct Supervisor {
    kernel: Kernel,
    live: Control,
    watch: Mutex<HashMap<String, Watch>>,
}

/// Per-plugin supervision memory: D3 budget spent so far, and when the
/// crash-watch last reaped a child (the backoff/cooldown window anchor).
#[derive(Default)]
struct Watch {
    attempts: u32,
    last_crash: Option<Timestamp>,
}

/// The answer of one `revive` request (05 §2 `restart{}` semantics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Revive {
    /// Budget and window allowed it: the child respawned and re-started.
    Revived,
    /// Backoff/cooldown still open — ask again after the window (or the
    /// clock dial crosses it; Step-domain hosts do exactly that, zero-sleep).
    Deferred,
    /// Policy exhausted its budget (or is `none`): parked in FAILED,
    /// operator-visible, never a silent loop (03 §2 "stopped trying").
    Parked,
}

impl Supervisor {
    /// Production posture: real clock, `exe` executor (modules/01 §1 row 2).
    #[must_use]
    pub fn new() -> Self {
        Self::with_clock(Clock::new(crate::clock::ClockDomain::Real))
    }

    /// Test/replay posture (D24 face, D74 direction): the boot dial is the
    /// host's, so the backoff windows gate deterministically on `advance`.
    #[must_use]
    pub fn with_clock(clock: Clock) -> Self {
        let (executor, live) = ProcessExecutor::paired();
        let kernel = Kernel::with_clock_and_executor(clock, Box::new(executor));
        // S2-M3a: the exe executor installs kernel-side proxies for the
        // child's REGISTER announcements through the kernel's own registry
        // door — one shared storage, wired once here.
        live.lock().registry = Some(kernel.registry().shared());
        Self {
            kernel,
            live,
            watch: Mutex::new(HashMap::new()),
        }
    }

    /// The kernel underneath — registry/bus/events reads and the lifecycle
    /// doors stay the ordinary kernel face (D24: construction, dial, reads).
    #[must_use]
    pub fn kernel(&self) -> &Kernel {
        &self.kernel
    }

    /// Crash-watch, wait-style: `try_wait` each supervised child (reaping
    /// exactly once, so a dead child never becomes a zombie and never gets
    /// waited twice), count the observation at this doorway, and drive the
    /// FAILED edge through the existing state machine by taking the
    /// cooperative-stop path against the dead control channel.
    ///
    /// Returns the ids whose child newly died. A plugin still in STARTING
    /// keeps its crash for the ordinary `start` doorway (the handle reports
    /// it there) — no invented transition either way.
    pub fn poll(&self) -> Vec<String> {
        let mut dead: Vec<String> = Vec::new();
        {
            let mut shared = self.live.lock();
            let mut vanished: Vec<String> = Vec::new();
            for (id, p) in shared.live.iter_mut() {
                if p.status.is_some() {
                    continue; // already reaped at some doorway this generation
                }
                match p.child.try_wait() {
                    Ok(Some(status)) => {
                        p.status = Some(status);
                        let _ = p.control.take(); // fd hygiene: release now
                        dead.push(id.clone());
                    }
                    Ok(None) => {}
                    Err(_) => vanished.push(id.clone()), // reaped elsewhere
                }
            }
            for id in vanished {
                shared.live.remove(&id);
                dead.push(id);
            }
        }
        if dead.is_empty() {
            return dead;
        }
        let now = self.kernel.clock().now();
        {
            let mut watch = self.lock_watch();
            for id in &dead {
                self.kernel.counters().note_process_crash();
                watch.entry(id.clone()).or_default().last_crash = Some(now);
            }
        }
        for id in &dead {
            // The FAILED edge rides the existing doorway: `Kernel::stop`
            // takes ACTIVE -> STOPPING, the handle reports the reaped
            // status, and the error arm moves STOPPING -> FAILED. States
            // that cannot take `stop` (STARTING, already FAILED) return
            // InvalidTransition here and surface the crash at their own
            // existing doorway instead — zero phantom events by omission.
            let _ = self.kernel.stop(&PluginId::new(id.clone()));
        }
        dead
    }

    /// Restart one FAILED `exe` plugin per its manifest policy (D3 table:
    /// `allows_restart` budget + backoff window grown per attempt inside
    /// the `min..max` range, floored by `cooldown`, measured on the boot
    /// clock). The respawn rides `FAILED -> RESOLVING` — the edge the
    /// machine already legalizes for the restart policy.
    ///
    /// # Errors
    ///
    /// `KernelError::UnknownPlugin` / `InvalidTransition` (not FAILED);
    /// resolve/start failures of the fresh attempt propagate as their own
    /// typed errors (a liar on respawn lands on FAILED again by itself).
    pub fn revive(&self, id: &PluginId) -> Result<Revive, KernelError> {
        let (policy, state) = {
            let table = self
                .kernel
                .plugins
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            let record = table.get(id).ok_or_else(|| KernelError::UnknownPlugin {
                id: id.as_str().to_string(),
            })?;
            (record.manifest.restart.clone(), record.machine.state())
        };
        if state != PluginState::Failed {
            return Err(KernelError::InvalidTransition {
                from: state.to_string(),
                to: "RESTART (the supervisor revives only FAILED)".to_string(),
            });
        }
        let mut watch = self.lock_watch();
        let w = watch.entry(id.as_str().to_string()).or_default();
        if !policy.allows_restart(w.attempts) {
            return Ok(Revive::Parked);
        }
        if let Some(crash_ts) = w.last_crash {
            let waited = self.kernel.clock().now().delta_since(crash_ts);
            if waited < window(&policy, w.attempts) {
                return Ok(Revive::Deferred);
            }
        }
        w.attempts += 1;
        drop(watch);
        self.kernel.counters().note_process_restart();
        // The budget is spent when the attempt is MADE, not when it ends:
        // a respawn that fails again lands on FAILED through its own
        // resolve/start error arms and the bounded loop keeps counting.
        self.kernel.resolve(id)?;
        self.kernel.start(id)?;
        Ok(Revive::Revived)
    }

    fn lock_watch(&self) -> MutexGuard<'_, HashMap<String, Watch>> {
        self.watch.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl Default for Supervisor {
    fn default() -> Self {
        Self::new()
    }
}

/// The D3 wait before restart attempt number `attempts`: the backoff range
/// `min..max` (05 §2) grown exponentially and capped at `max`, floored by
/// `cooldown`. `None` policy never reaches here (budget check parks first).
fn window(policy: &RestartPolicy, attempts: u32) -> Duration {
    let RestartPolicy::OnFailure {
        backoff, cooldown, ..
    } = policy
    else {
        return Duration::ZERO;
    };
    let step = backoff
        .min
        .saturating_mul(1u32 << attempts.min(20))
        .min(backoff.max);
    step.max(*cooldown)
}
