//! Gate counters on the existing doorways (D21, modules/10 §6.1) — M6
//! floor 5. "No new doorway — the existing ones get richer": every counter
//! here increments in code that ALREADY had to run (registry acquire/
//! release, the kernel lifecycle transition helper, the load handshake).
//! Relaxed atomic adds, read-only getters, zero enforcement policy —
//! observation year 1 (06 §6 keeps rate limits and caps trigger-gated).
//!
//! The per-topic bus family (`{published, delivered, dropped,
//! queue-high-water}`) lives on the bus's own `TopicState` — that is where
//! its doorway is; `Kernel::counters_view()` folds the two families into
//! the one `Snapshot.counters()` read.

use std::sync::atomic::{AtomicU64, Ordering};

/// Kernel-side gate counters (registry + lifecycle + load handshake).
#[derive(Debug, Default)]
pub struct GateCounters {
    registry_acquires: AtomicU64,
    registry_releases: AtomicU64,
    lifecycle_transitions: AtomicU64,
    load_ok: AtomicU64,
    load_fail: AtomicU64,
    /// S2-M1 supervision family: deaths the crash-watch reaped, restarts
    /// the D3 budget granted. Doorway: `Supervisor::poll` / `revive`.
    process_crashes: AtomicU64,
    process_restarts: AtomicU64,
}

/// The folded read of every gate family for `Snapshot.counters()` — plain
/// data, snapshotted at read time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CountersView {
    pub registry_acquires: u64,
    pub registry_releases: u64,
    pub lifecycle_transitions: u64,
    pub load_ok: u64,
    pub load_fail: u64,
    pub process_crashes: u64,
    pub process_restarts: u64,
    pub bus_published: u64,
    pub bus_delivered: u64,
    pub bus_dropped: u64,
}

impl GateCounters {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn note_acquire(&self) {
        self.registry_acquires.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn note_release(&self) {
        self.registry_releases.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn note_transition(&self) {
        self.lifecycle_transitions.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn note_load(&self, ok: bool) {
        if ok {
            self.load_ok.fetch_add(1, Ordering::Relaxed);
        } else {
            self.load_fail.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Crash observed by the supervisor's crash-watch (S2-M1 doorway:
    /// `Supervisor::poll`, where the reap actually happens).
    pub(crate) fn note_process_crash(&self) {
        self.process_crashes.fetch_add(1, Ordering::Relaxed);
    }

    /// Restart granted inside the D3 budget (S2-M1 doorway:
    /// `Supervisor::revive`, at the moment the respawn is executed).
    pub(crate) fn note_process_restart(&self) {
        self.process_restarts.fetch_add(1, Ordering::Relaxed);
    }

    /// Read-only getters (D21: no policy hangs off these numbers).
    #[must_use]
    pub fn registry_acquires(&self) -> u64 {
        self.registry_acquires.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn registry_releases(&self) -> u64 {
        self.registry_releases.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn lifecycle_transitions(&self) -> u64 {
        self.lifecycle_transitions.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn load_ok(&self) -> u64 {
        self.load_ok.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn load_fail(&self) -> u64 {
        self.load_fail.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn process_crashes(&self) -> u64 {
        self.process_crashes.load(Ordering::Relaxed)
    }

    #[must_use]
    pub fn process_restarts(&self) -> u64 {
        self.process_restarts.load(Ordering::Relaxed)
    }
}
