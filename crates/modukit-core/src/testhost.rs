//! The test host face (D24, modules/10 §3.1) — M6 floor 6.
//!
//! v1 surface discipline (D24): host construction, clock turning, and
//! snapshot/event reads ONLY — no mock services, no patching APIs. The
//! production code path runs with swapped water sources: a step-domain
//! clock (`advance()` is the dial), so nothing here ever sleeps on the wall
//! clock and every test that uses this face is deterministic.
//!
//! The fold checkers below are the §3 machinery: replay the merged event
//! stream into the live tables and compare — tables and events may never
//! disagree (the churn acceptance leans on exactly that).

use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;
use std::time::Duration;

use crate::clock::{Clock, ClockDomain, Timestamp};
use crate::events::{ObsEvent, ObsRecord, Tail};
use crate::introspect::{HostView, Snapshot};
use crate::kernel::Kernel;
use crate::registry::ServiceEvent;

/// Fold `Lifecycle` edges out of an event sequence into a
/// plugin-id → latest-state table (the `fold(events) -> snapshot` half of
/// the 10 §3 machinery).
#[must_use]
pub fn fold_states(events: &[ObsRecord]) -> HashMap<String, String> {
    let mut table = HashMap::new();
    for record in events {
        if let ObsEvent::Lifecycle { plugin, to, .. } = &record.event {
            table.insert(plugin.clone(), to.clone());
        }
    }
    table
}

/// Fold the registry presence pairs out of an event sequence into the set
/// of live canonical service ids (invariant 5's late-join rule: the events
/// alone determine the table).
#[must_use]
pub fn fold_services(events: &[ObsRecord]) -> BTreeSet<String> {
    let mut live = BTreeSet::new();
    for record in events {
        match &record.event {
            ObsEvent::Registry(ServiceEvent::Registered { service_id, .. }) => {
                live.insert(service_id.to_string());
            }
            ObsEvent::Registry(ServiceEvent::Unregistered { service_id, .. }) => {
                live.remove(&service_id.to_string());
            }
            _ => {}
        }
    }
    live
}

/// The D74 replay scenario (S2-M3b, paying D17 test-2): one fixed pair of
/// timer jobs both placements schedule identically, then step — the due-job
/// transcripts must come back byte-equal whether the dial was turned
/// in-process (`TestHost::advance`) or over the control socket
/// (`Supervisor::dial`). `(deadline_ns, job label)` in schedule order;
/// the tokens fall out as 1 and 2 on any fresh clock.
pub const REPLAY_MARKERS: &[(u64, &str)] = &[
    (1_000_000_000, "replay-alpha"),
    (3_000_000_000, "replay-beta"),
];

/// Schedule the replay markers on a clock — the scenario's one source of
/// truth: the runner embeds it, the `dl`-arm test schedules the same list.
pub fn schedule_replay_markers(clock: &Clock) {
    for (deadline_ns, job) in REPLAY_MARKERS {
        clock.timer(Timestamp::from(*deadline_ns), *job);
    }
}

/// The D24 face: a production kernel on the step-domain clock, plus the
/// dial, the reads, and the fold checker. Nothing here mocks, patches, or
/// sleeps — water sources are swapped, not code paths.
pub struct TestHost {
    kernel: Kernel,
    snapshot: Snapshot,
}

impl std::fmt::Debug for TestHost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TestHost(step-kernel)")
    }
}

impl Default for TestHost {
    fn default() -> Self {
        Self::new()
    }
}

impl TestHost {
    /// Boot a kernel on the step clock (`now()` starts at zero and time
    /// exists only where `advance` moved it).
    #[must_use]
    pub fn new() -> Self {
        let kernel = Kernel::with_clock(Clock::new(ClockDomain::Step));
        let snapshot = Snapshot::new(
            &kernel.registry,
            Arc::clone(&kernel.bus),
            Arc::clone(&kernel.plugins),
            Arc::clone(&kernel.obs),
            Arc::clone(&kernel.edges),
        );
        Self { kernel, snapshot }
    }

    /// The production kernel underneath (registry/bus/clock/ledger handles
    /// all reachable — the v1 discipline: construction, dial, reads).
    #[must_use]
    pub fn kernel(&self) -> &Kernel {
        &self.kernel
    }

    /// Turn the dial; returns the ring seqs of the timer jobs that became
    /// due (posted as events — D6 message-shaped, no callbacks).
    #[must_use]
    pub fn advance(&self, delta: Duration) -> Vec<u64> {
        self.kernel.advance_clock(delta)
    }

    /// The current head of the stream (for resumable reads).
    #[must_use]
    pub fn head(&self) -> u64 {
        self.kernel.observatory().last_seq()
    }

    /// Resumable event read: everything after `since_seq`.
    #[must_use]
    pub fn events_since(&self, since_seq: u64) -> Tail {
        self.kernel.observatory().tail(since_seq)
    }

    /// The object-table cut — the same answer the `org.modukit.snapshot`
    /// service gives (one source of truth, D17).
    #[must_use]
    pub fn snapshot(&self) -> HostView {
        self.snapshot.all()
    }

    /// The 10 §3 checker: fold(events since genesis) must equal the live
    /// tables — plugin states and the service set may never disagree.
    /// Panics with the first disagreement it finds.
    pub fn assert_consistent(&self) {
        let tail = self.kernel.observatory().tail(0);
        let states = fold_states(&tail.events);
        let live_services = fold_services(&tail.events);
        let view = self.snapshot.all();
        for plugin in &view.plugins {
            assert_eq!(
                states.get(&plugin.id),
                Some(&plugin.state),
                "plugin {} exists in the table; the fold says otherwise",
                plugin.id
            );
        }
        for (id, state) in &states {
            assert!(
                view.plugins.iter().any(|p| &p.id == id),
                "the fold knows plugin {id} ({state}) the table lost",
            );
        }
        let table_services: BTreeSet<String> = view
            .services
            .iter()
            .map(|s| s.service_id.to_string())
            .collect();
        assert_eq!(
            live_services, table_services,
            "service fold must equal the registry table",
        );
    }
}
