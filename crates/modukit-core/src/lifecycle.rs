//! Six-state lifecycle machine + Failed edge + restart policy types (D3).
//!
//! Names and edges mirror `docs/modules/03-lifecycle-state-machine.md` §1-2:
//! UNINSTALLED -> INSTALLED -> RESOLVING -> STARTING -> ACTIVE -> STOPPING ->
//! RESOLVED/UNINSTALLED, with FAILED entered from RESOLVING/STARTING (setup
//! error) and STOPPING (stop timeout). `dl` placement has no ACTIVE->FAILED
//! edge (01 §4: host corruption leaves nothing to report to). Restart policy
//! types ship here; the supervisor that acts on them is `exe` placement
//! (Stage-2) — under `dl` the fields are recorded, not hidden (05 §2).

use std::fmt;
use std::time::Duration;

use crate::error::KernelError;

/// The plugin states of the machine (modules/03 §1 diagram).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginState {
    /// Not known to the kernel yet (initial and terminal state).
    Uninstalled,
    /// Artifact known, identity + manifest parsed, not yet validated.
    Installed,
    /// Contract/link/placement prerequisites being checked; required
    /// dependencies validated here (deterministic boot failure, before any
    /// plugin code executes). Transient: passes on to STARTING or FAILED.
    Resolving,
    /// `start()` in progress.
    Starting,
    /// Running.
    Active,
    /// Cooperative `stop()` in progress (drain: refuse new, finish old).
    Stopping,
    /// Stopped-but-resolved, cheap restart origin.
    Resolved,
    /// Setup error or (process placement only) supervised death.
    Failed,
}

impl fmt::Display for PluginState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PluginState::Uninstalled => "UNINSTALLED",
            PluginState::Installed => "INSTALLED",
            PluginState::Resolving => "RESOLVING",
            PluginState::Starting => "STARTING",
            PluginState::Active => "ACTIVE",
            PluginState::Stopping => "STOPPING",
            PluginState::Resolved => "RESOLVED",
            PluginState::Failed => "FAILED",
        };
        f.write_str(name)
    }
}

/// Transition rule from the modules/03 §2 table. `from` -> `to` must appear
/// in the table; every other pair is invalid. `UNINSTALLED` is reachable from
/// any state (uninstall guard, last row of the table).
#[must_use]
pub fn is_valid_transition(from: PluginState, to: PluginState) -> bool {
    if to == PluginState::Uninstalled {
        return true; // any -> UNINSTALLED (uninstall)
    }
    matches!(
        (from, to),
        (PluginState::Uninstalled, PluginState::Installed)
            | (PluginState::Installed, PluginState::Resolving)
            | (PluginState::Resolving, PluginState::Starting)
            | (PluginState::Resolving, PluginState::Failed)
            | (PluginState::Starting, PluginState::Active)
            | (PluginState::Starting, PluginState::Failed)
            | (PluginState::Active, PluginState::Stopping)
            | (PluginState::Stopping, PluginState::Resolved)
            | (PluginState::Stopping, PluginState::Failed)
            | (PluginState::Failed, PluginState::Resolving) // restart policy allows
    )
}

/// One plugin's state machine. Transitions are checked against §2; an
/// invalid move returns `KernelError::InvalidTransition` and keeps the state.
#[derive(Debug)]
pub struct StateMachine {
    state: PluginState,
}

impl StateMachine {
    /// Fresh machine; plugins start `UNINSTALLED` until the kernel installs them.
    #[must_use]
    pub fn new() -> Self {
        Self {
            state: PluginState::Uninstalled,
        }
    }

    #[must_use]
    pub fn state(&self) -> PluginState {
        self.state
    }

    /// Move to `to` if the §2 table allows it; error otherwise (state kept).
    pub fn transition(&mut self, to: PluginState) -> Result<(), KernelError> {
        if !is_valid_transition(self.state, to) {
            return Err(KernelError::InvalidTransition {
                from: self.state.to_string(),
                to: to.to_string(),
            });
        }
        self.state = to;
        Ok(())
    }

    /// Move to FAILED (the edge every guard failure lands on, 03 §2).
    /// Legal only from RESOLVING/STARTING/STOPPING under the table.
    pub fn fail(&mut self) -> Result<(), KernelError> {
        self.transition(PluginState::Failed)
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

/// Restart policy from the manifest `restart{}` block (05 §2, D3):
/// `{ none | on-failure, max_retries, backoff, cooldown }`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RestartPolicy {
    /// Never restart (`policy = "none"` or no `restart{}` block).
    None,
    /// Restart on failure while the budget lasts.
    OnFailure {
        max_retries: u32,
        backoff: Backoff,
        cooldown: Duration,
    },
}

/// Backoff window `"<min>..<max>"` (05 §2 example `1s..30s`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Backoff {
    pub min: Duration,
    pub max: Duration,
}

impl Default for Backoff {
    fn default() -> Self {
        Self {
            min: Duration::from_secs(1),
            max: Duration::from_secs(30),
        }
    }
}

impl RestartPolicy {
    /// Guard of the FAILED -> RESOLVING edge: does the budget still allow a
    /// restart after `attempts_so_far` failures? Budget exhaustion parks in
    /// FAILED and is operator-visible (03 §2 "stopped trying"), never a
    /// silent loop.
    #[must_use]
    pub fn allows_restart(&self, attempts_so_far: u32) -> bool {
        match self {
            RestartPolicy::None => false,
            RestartPolicy::OnFailure { max_retries, .. } => attempts_so_far < *max_retries,
        }
    }
}
