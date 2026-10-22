//! Kernel boot wiring for the M6 observability floors.
//!
//! The kernel builds its own observation parts at boot (clock, ring, logger,
//! edge ledger, gate counters) and registers them as ORDINARY services under
//! the reserved `org.modukit.*` namespace (D40: the service table is the one
//! extension mechanism; D17: tools are downstream consumers, no back doors).
//! Kept out of `kernel.rs` so the lifecycle spine stays readable.
//!
//! Wave-2 T2 amendment (D83 tail-add by record, NAMED here): the plugin
//! bell's QoS capacity is `embed_codec::LOSSY_RING` — the wire vocabulary's
//! decode of qos code 2 (64) — replacing T1's `Lossy(16)` literal so wall-
//! in and wall-out rings behave identically (ruling 6's mirror law beats the
//! hearing draft's round number), and every bell payload leads with the
//! clock's boot generation, u64 LE (`gen ++ token ++ label`, D99 §9).

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

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
            plugin_timers: Mutex::new(HashMap::new()),
            parent_bells: Mutex::new(None),
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
                let seq = self.obs.emit(crate::events::ObsEvent::TimerDone {
                    token: j.token.value(),
                    job: j.job.clone(),
                });
                self.post_timer_bell(&j);
                seq
            })
            .collect()
    }

    /// Wave-2 T1 (D99 R4-A/R5-A): the per-plugin bell at the ONE due-funnel.
    /// Every dial/poll doorway (`post_due_jobs`/`advance_clock`/
    /// `dial_clock`) passes through `post_jobs`, so this is the only place a
    /// plugin token can be answered on the bus. Host-owned timers keep the
    /// `TimerDone` ring posting above EXACTLY as today; the bus publish is
    /// additive and scoped to registered plugin tokens. `bus.publish` touches
    /// only bus internals and never re-enters the registry/kernel locks (the
    /// deadlock class plan §5 names), and it never blocks (bus commandment 3).
    /// T2 (D99 R5-A) grows the TWO wall outlets after this local publish,
    /// still behind the register check: the runner-side bridge FORWARD and
    /// the parent-side fan-out pump - bells only, at the one due-funnel.
    fn post_timer_bell(&self, job: &crate::clock::DueJob) {
        let Some(plugin_id) = self
            .plugin_timers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&job.token.value())
            .cloned()
        else {
            return; // host-owned timer: ring posting only, never a bell.
        };
        // The T1 test lever (plan §4.2): read at EXACTLY this one site; it
        // skips the bus post and nothing else, so the door suite's recv
        // assertion goes red while the TimerDone ring stays honest.
        if std::env::var_os("MODUKIT_TIMER_BELL_SABOTAGE").is_some() {
            return;
        }
        let topic =
            crate::identity::TopicId::new(format!("org.modukit.timer.{}", plugin_id.as_str()));
        let mut payload = self.clock.boot_gen().to_le_bytes().to_vec();
        payload.extend_from_slice(&job.token.value().to_le_bytes());
        payload.extend_from_slice(job.job.as_bytes());
        self.bus.publish(
            &topic,
            crate::topic::QoS::Lossy(crate::embed_codec::LOSSY_RING),
            payload.clone(),
        );
        // Outlet 1 (D99 R5-A, child side): this kernel runs inside a
        // capable runner - the bell rides the mailbox door verb's own call
        // (FWD_PUB), and the runner's timer capability gates it (the demo
        // lever drops exactly this bit). A dead parent socket is the
        // crash-watch's to report at its own doorway; the bell never
        // blocks (commandment 3), so the io answer is swallowed forward.
        if let Some(b) = crate::bus_wire::bridge()
            && b.timer_bells()
        {
            let _ = b.publish(topic.as_str(), 2, &payload);
        }
        // Outlet 2 (D99 R5-A, parent side): the named landed absence from
        // the hearing review, fixed at the due-funnel - pump the just-
        // published bell to timer-capable INTERESTED children only. The
        // Arc clone keeps the parent_bells lock a leaf (one-thread host
        // law: no kernel lock stays live across the fanout's maps).
        let bells = self
            .parent_bells
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        if let Some(f) = bells {
            f.pump_local_bell(&topic, 2, &payload);
        }
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

#[cfg(test)]
/// T2 leg DOWN at the REAL bell site, frame-level (the crossing plan's
/// named precedent set, landed where the observation surface reaches):
/// the supervisor kernel hosts no dl plugin, so no live child can see a
/// parent-originated bell end to end (finding F-2 in the execution
/// report) - these tests drive `post_timer_bell` itself against an
/// attached, wired fan-out and read the DELIVER bytes off the socket.
mod bell_crossing_tests {
    use std::io::Read;
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    use super::*;
    // Premise: no MODUKIT_TIMER_BELL_SABOTAGE in this process (the lever
    // is the T1 door suite's own cell; these legs need the bell live).
    use crate::bus_wire::ParentFanout;
    use crate::clock::{Clock, ClockDomain, Timestamp};
    use crate::identity::{PluginId, TopicId};
    use crate::proc_wire::{self, FWD_SUB, Forward, MSG_DELIVER};

    fn bell_topic(plugin: &str) -> TopicId {
        TopicId::new(format!("org.modukit.timer.{plugin}"))
    }

    /// Kernel + wired, timer-capable child "cap", interest established.
    fn wired_pair(plugin: &str) -> (Kernel, UnixStream, TopicId) {
        let kernel = Kernel::with_clock(Clock::new(ClockDomain::Step));
        let (wired, reader) = UnixStream::pair().expect("socketpair");
        reader
            .set_read_timeout(Some(Duration::from_millis(300)))
            .expect("read timeout");
        let topic = bell_topic(plugin);
        let fan = Arc::new(ParentFanout::new(Arc::clone(&kernel.bus)));
        fan.register_child("cap".into(), wired);
        fan.note_timer_child("cap".into());
        fan.handle_forward(
            "cap",
            &Forward {
                op: FWD_SUB,
                topic: topic.as_str().to_string(),
                qos: 0,
                value: Vec::new(),
            },
        )
        .expect("bell-topic interest");
        kernel.attach_parent_bells(fan);
        (kernel, reader, topic)
    }

    fn register(kernel: &Kernel, token: u64, plugin: &str) {
        kernel
            .plugin_timers
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(token, PluginId::new(plugin.to_string()));
    }

    /// The parent-side bell crosses its own wall: dial, and the DELIVER
    /// frame answers the child socket with the pinned payload (gen u64 LE
    /// ++ token u64 LE ++ label, qos code 2) - the on_publish grammar.
    #[test]
    fn parent_bell_site_pumps_deliver_to_the_timer_child() {
        let (kernel, mut reader, topic) = wired_pair("com.example.dlparent");
        let token = kernel.clock().timer(Timestamp::from(1_000_000_000), "tick");
        register(&kernel, token.value(), "com.example.dlparent");
        kernel.dial_clock(Duration::from_secs(2));
        let (ver, kind, payload) =
            proc_wire::read_frame(&mut reader).expect("the pumped bell is a DELIVER");
        assert_eq!(
            (ver, kind),
            (proc_wire::WIRE_VERSION, MSG_DELIVER),
            "no new frame type: the existing DELIVER rides"
        );
        let delivery = proc_wire::parse_deliver(&payload).expect("DELIVER grammar");
        assert_eq!(delivery.topic, topic.as_str());
        assert_eq!(delivery.qos, 2, "the Lossy-ring vocabulary code");
        let bytes = delivery.value;
        assert!(bytes.len() > 16, "gen(8) + token(8) + label");
        let tag = u64::from_le_bytes(
            bytes[..8]
                .try_into()
                .expect("the payload leads with the boot generation"),
        );
        assert_eq!(
            tag,
            kernel.clock().boot_gen(),
            "self-equality against the clock's own sample - never a literal"
        );
        assert_eq!(
            u64::from_le_bytes(bytes[8..16].try_into().expect("token slot")),
            token.value()
        );
        assert_eq!(&bytes[16..], b"tick", "the job label rides verbatim");
    }

    /// The T1 register law through the new outlets: a host-owned token
    /// (absent from the register) never bells and never pumps.
    #[test]
    fn host_owned_timer_still_never_bells_or_pumps() {
        let (kernel, mut reader, _topic) = wired_pair("com.example.hostown");
        kernel
            .clock()
            .timer(Timestamp::from(1_000_000_000), "hosted");
        kernel.dial_clock(Duration::from_secs(2));
        let mut byte = [0u8; 1];
        match reader.read(&mut byte) {
            Err(err)
                if matches!(
                    err.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) => {}
            other => panic!("host-owned firing must not pump a frame, got {other:?}"),
        }
    }

    /// D99 §9 self-equality, asserted the only allowed way: two firings
    /// on the SAME kernel share the generation; different tokens prove
    /// two distinct entries. No literal generation value anywhere.
    #[test]
    fn same_kernel_entries_share_the_generation() {
        let kernel = Kernel::with_clock(Clock::new(ClockDomain::Step));
        let topic = bell_topic("com.example.twin");
        let mut rx = kernel
            .bus()
            .subscribe(&topic)
            .expect("live subscriber on an undeclared topic");
        let t1 = kernel
            .clock()
            .timer(Timestamp::from(1_000_000_000), "first");
        let t2 = kernel
            .clock()
            .timer(Timestamp::from(2_000_000_000), "second");
        register(&kernel, t1.value(), "com.example.twin");
        register(&kernel, t2.value(), "com.example.twin");
        kernel.dial_clock(Duration::from_secs(3));
        let a = rx.try_recv().expect("first bell");
        let b = rx.try_recv().expect("second bell");
        let gen_a = u64::from_le_bytes(a[..8].try_into().expect("gen slot"));
        let gen_b = u64::from_le_bytes(b[..8].try_into().expect("gen slot"));
        assert_eq!(gen_a, gen_b, "same-kernel entries share the boot tag");
        assert_eq!(gen_a, kernel.clock().boot_gen());
        assert_ne!(
            &a[8..16],
            &b[8..16],
            "the two firings carry distinct tokens"
        );
    }
}
