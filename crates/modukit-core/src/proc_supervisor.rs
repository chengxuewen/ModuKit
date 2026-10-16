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
use crate::manifest::PlacementKind;

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

/// What ONE `Supervisor::step` pass observed (S2-M3b host-loop seam).
/// Plain data, foldable: the S2-M5 demo host loops `step`/`drive_until_idle`
/// and reads the report; nothing else needs to know the loop exists.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct StepReport {
    /// Ids whose supervised child newly died this pass (the `poll` answer).
    pub crashed: Vec<String>,
    /// Ring seqs of timer jobs the boot clock found due (the `Real`-domain
    /// posting doorway; empty on a never-dialed Step domain).
    pub posted: Vec<u64>,
    /// Ids the restart policy respawned and re-started this pass.
    pub revived: Vec<String>,
    /// Ids FAILED with the backoff/cooldown window still closed — waiting
    /// on the boot clock's dial, not on the driver.
    pub deferred: Vec<String>,
    /// Ids the policy parked (budget exhausted or `none`): terminal until
    /// an operator decision, they never re-enter the drive.
    pub parked: Vec<String>,
}

impl StepReport {
    fn merge(&mut self, other: StepReport) {
        self.crashed.extend(other.crashed);
        self.posted.extend(other.posted);
        self.revived.extend(other.revived);
        self.deferred.extend(other.deferred);
        self.parked.extend(other.parked);
    }

    /// A pass counts as progress when it observed a death or performed a
    /// restart; deferrals wait on the dial and parkings are terminal.
    fn actionable(&self) -> bool {
        !self.crashed.is_empty() || !self.revived.is_empty()
    }
}

/// The aggregate of a `drive_until_idle` run: how many bounded passes it
/// took and everything those passes reported, merged.
#[derive(Debug, Default)]
pub struct DrivenReport {
    pub passes: u32,
    pub report: StepReport,
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

    /// One bounded host-loop pass (S2-M3b): crash-watch, due-job posting on
    /// the boot clock, and D3 restart-policy application to FAILED supervised
    /// runs — the pass the S2-M5 `demo-stage2` host calls in a loop (the
    /// E1 escalation finding from S2-M1 finally gets its home; no new binary,
    /// no thread, no sleep inside). Pacing stays with the caller: the driver
    /// observes and applies what the policy already decided.
    pub fn step(&self) -> StepReport {
        let mut report = StepReport {
            crashed: self.poll(),
            ..StepReport::default()
        };
        report.posted = self.kernel.post_due_jobs();
        // FAILED supervised runs are the policy's queue; the drive applies it.
        // Only `exe` records join: `dl` placement has no supervision budget
        // (01 §1 — the executor seam polices its kind, the driver mirrors it).
        let failed: Vec<PluginId> = {
            let table = self
                .kernel
                .plugins
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            table
                .iter()
                .filter(|(_, record)| {
                    record.machine.state() == PluginState::Failed
                        && record.manifest.entry.kind == PlacementKind::Exe
                })
                .map(|(id, _)| id.clone())
                .collect()
        };
        for id in failed {
            match self.revive(&id) {
                Ok(Revive::Revived) => report.revived.push(id.as_str().to_string()),
                Ok(Revive::Deferred) => report.deferred.push(id.as_str().to_string()),
                Ok(Revive::Parked) => report.parked.push(id.as_str().to_string()),
                // Raced away (uninstalled between scan and door) or legally
                // un-revivable: hands off, the next pass re-reads the truth.
                Err(_) => {}
            }
        }
        report
    }

    /// Drive `step` until a pass observes nothing actionable, bounded by
    /// `max_passes` — a host loop that can spin is a host loop that hides a
    /// bug. Deferred restarts (waiting on the clock dial) and parked runs
    /// (terminal until an operator acts) end the drive honestly: they are
    /// not the driver's to wait for.
    pub fn drive_until_idle(&self, max_passes: u32) -> DrivenReport {
        let mut driven = DrivenReport::default();
        for pass in 1..=max_passes.max(1) {
            let report = self.step();
            let actionable = report.actionable();
            driven.passes = pass;
            driven.report.merge(report);
            if !actionable {
                break;
            }
        }
        driven
    }

    /// Turn one supervised child's clock dial over the control socket (D74,
    /// S2-M3b): on the test/replay track the supervisor is the SOLE
    /// authority advancing the runner's Step/Sim clock. Returns the fired
    /// batch (`DueTurn`) the child crossed; a Real-domain child refuses with
    /// the typed `ClockDialRefused`, a silent child is killed at
    /// `DIAL_TIMEOUT`. The production Real placement stays untouched —
    /// there is simply no dial there to turn.
    ///
    /// # Errors
    ///
    /// The crash family for a dead/hung/mismatched child;
    /// `KernelError::ClockDialRefused` for a live but undialable one.
    pub fn dial(
        &self,
        id: &PluginId,
        delta: std::time::Duration,
    ) -> Result<crate::proc_wire::DueTurn, KernelError> {
        crate::executor_proc::dial_child(&self.live, id.as_str(), delta)
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
