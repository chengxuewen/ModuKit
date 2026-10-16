//! Kernel boot wiring for the M6 observability floors.
//!
//! The kernel builds its own observation parts at boot (clock, ring, logger,
//! edge ledger, gate counters) and registers them as ORDINARY services under
//! the reserved `org.modukit.*` namespace (D40: the service table is the one
//! extension mechanism; D17: tools are downstream consumers, no back doors).
//! Kept out of `kernel.rs` so the lifecycle spine stays readable.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::bus::Bus;
use crate::clock::Clock;
use crate::edge_ledger::EdgeLedger;
use crate::events::Observatory;
use crate::executor::{DlopenExecutor, Executor};
use crate::introspect::{EventStream, LedgerDump, Snapshot};
use crate::kernel::{Kernel, PluginRecord};
use crate::logger::Logger;
use crate::registry::{Properties, PropertyValue, Registry};

/// Register one kernel-owned platform service. Kernel-authored props always
/// carry `service.id`; anything else is a platform bug (debug surface).
pub(crate) fn register_platform<S: Any + Send + Sync>(
    registry: &Registry,
    instance: Arc<S>,
    id: &str,
) {
    let mut props = Properties::new();
    props.insert("service.id", PropertyValue::Str(id.to_string()));
    let outcome = registry.register(instance, props);
    debug_assert!(
        outcome.is_ok(),
        "platform service {id} must register: {outcome:?}"
    );
}

impl Kernel {
    /// Kernel on an explicit clock domain (D20 one water source; the
    /// testhost dials `ClockDomain::Step`). Boots the observability floors
    /// and registers the platform services under `org.modukit.*`.
    #[must_use]
    pub fn with_clock(clock: Clock) -> Self {
        Self::with_clock_and_executor(clock, Box::new(DlopenExecutor::new()))
    }

    /// Kernel on an explicit clock and executor seam (S2-M1: the
    /// supervisor boots with `ProcessExecutor`; each executor answers
    /// exactly one placement kind and polices it inside `load`).
    #[must_use]
    pub fn with_clock_and_executor(clock: Clock, executor: Box<dyn Executor>) -> Self {
        let obs = Arc::new(Observatory::new(clock.clone()));
        let registry = Registry::with_observability(Arc::clone(&obs));
        let logger = Arc::new(Logger::new(Arc::clone(&obs)));
        let edges = Arc::new(EdgeLedger::new());
        obs.attach_edges(Arc::clone(&edges));
        let bus = Arc::new(Bus::new());
        let plugins: PluginTable = Arc::new(Mutex::new(HashMap::new()));
        // M7 boot seam: the contract-ledger (fdset) bytes, shared with the
        // `LedgerDump` service so a post-boot set is visible to consumers.
        let contract_ledger: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(Vec::new()));
        // Introspect trio: read-only services in the reserved namespace
        // (D17 — the kernel publishes itself; every tool is a consumer).
        let snapshot = Snapshot::new(
            &registry,
            Arc::clone(&bus),
            Arc::clone(&plugins),
            Arc::clone(&obs),
            Arc::clone(&edges),
        );
        register_platform(&registry, Arc::new(snapshot), "org.modukit.snapshot");
        register_platform(
            &registry,
            Arc::new(EventStream::new(Arc::clone(&obs))),
            "org.modukit.event_stream",
        );
        register_platform(
            &registry,
            Arc::new(LedgerDump::new(
                Arc::clone(&plugins),
                Arc::clone(&contract_ledger),
            )),
            "org.modukit.ledger_dump",
        );
        register_platform(&registry, Arc::new(clock.clone()), "org.modukit.clock");
        register_platform(&registry, Arc::clone(&logger), "org.modukit.logger");
        Self {
            registry,
            bus,
            clock,
            obs,
            logger,
            edges,
            executor,
            plugins,
            contract_ledger,
        }
    }

    /// The unified clock (D20) — the kernel's and every plugin's only
    /// sanctioned time source.
    #[must_use]
    pub fn clock(&self) -> &Clock {
        &self.clock
    }

    /// The shared observatory ring (D17 merged event stream backing).
    #[must_use]
    pub fn observatory(&self) -> &Observatory {
        &self.obs
    }

    /// The kernel logger handle (D19). Plugins may equally
    /// `acquire::<Logger>()` from the registry — same instance either way.
    #[must_use]
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    /// The live edge ledger (D18) — queryable through `Snapshot.edges()`
    /// or directly.
    #[must_use]
    pub fn edge_ledger(&self) -> &EdgeLedger {
        &self.edges
    }

    /// The kernel-side gate counters (D21) — read-only getters; the
    /// numbers live where the enforcement doorways already run.
    #[must_use]
    pub fn counters(&self) -> &crate::counters::GateCounters {
        self.obs.counters()
    }

    /// Folded `Snapshot.counters()` read: kernel gates plus the bus's
    /// per-topic publish/deliver/drop tallies, one plain-data view.
    #[must_use]
    pub fn counters_view(&self) -> crate::counters::CountersView {
        let g = self.obs.counters();
        let mut view = crate::counters::CountersView {
            registry_acquires: g.registry_acquires(),
            registry_releases: g.registry_releases(),
            lifecycle_transitions: g.lifecycle_transitions(),
            load_ok: g.load_ok(),
            load_fail: g.load_fail(),
            process_crashes: g.process_crashes(),
            process_restarts: g.process_restarts(),
            ..Default::default()
        };
        for st in self.bus.topic_stats() {
            view.bus_published += st.published;
            view.bus_delivered += st.delivered;
            view.bus_dropped += st.dropped + st.retention_dropped;
        }
        view
    }

    fn post_jobs(&self, due: Vec<crate::clock::DueJob>) -> Vec<u64> {
        due.into_iter()
            .map(|j| {
                self.obs.emit(crate::events::ObsEvent::TimerDone {
                    token: j.token.value(),
                    job: j.job,
                })
            })
            .collect()
    }

    /// Post timer jobs that became due (the `Real` domain's poll doorway):
    /// the kernel's half of "timers fire by posting" (D6/D20 — nothing
    /// here calls a user closure). Returns the seqs of the posted events.
    pub fn post_due_jobs(&self) -> Vec<u64> {
        self.post_jobs(self.clock.due_jobs())
    }

    /// Turn the dial and post what became due (the deterministic timer
    /// doorway used by the testhost, D24). Step/Sim domains only; `Real`
    /// collects via `post_due_jobs`.
    pub fn advance_clock(&self, delta: std::time::Duration) -> Vec<u64> {
        self.post_jobs(self.clock.advance(delta))
    }

    /// Turn the dial and return the due batch (D74 over-the-wire dial,
    /// S2-M3b): the same posting as `advance_clock` PLUS the fired jobs —
    /// the `DUE` frame's answer data (timers-as-jobs: a return of content,
    /// never a callback, D6/D20).
    pub fn dial_clock(&self, delta: std::time::Duration) -> Vec<crate::clock::DueJob> {
        let fired = self.clock.advance(delta);
        self.post_jobs(fired.clone());
        fired
    }
}

/// The plugin table handle shared with the introspect trio (kernel-owned,
/// read-only from there — D17). Kept here so `PluginRecord` visibility
/// never widens beyond this crate.
pub(crate) type PluginTable = Arc<Mutex<HashMap<crate::identity::PluginId, PluginRecord>>>;
