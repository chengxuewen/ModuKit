//! The shared observatory ring (D17, modules/10 §1) — M6 floor 2a.
//!
//! One bounded, monotonic, append-only ring merges the event families the
//! introspect trio exposes: lifecycle transitions, registry event pairs,
//! `ModuleLoaded`, timer-done postings, edge retirements and log records.
//! Same discipline as the M5 bus ring (06 §2): a full ring sheds its oldest
//! entry and COUNTS the eviction — emit never blocks, drops are visible
//! (observation, never enforcement; §6 commandments).
//!
//! `tail(since_seq)` is resumable: seq is monotonic and the returned view
//! reports `first_seq` + `dropped` so a consumer can tell a gap from genesis.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::clock::{Clock, Timestamp};
use crate::registry::ServiceEvent;

/// Ring capacity: enough history that the M6 churn acceptance (200 cycles
/// from genesis) folds without a gap; retention/quota policy is trigger-
/// gated later (year 1 keeps this one constant, no knobs).
pub const RING_CAP: usize = 8192;

/// One merged event on the ring (modules/10 §1: lifecycle + registry +
/// QoS-visible + the §2 `ModuleLoaded` + D6 timer postings + D19 logs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObsEvent {
    /// A plugin lifecycle transition, emitted at the kernel's own doorway
    /// (03 §2 table). Names are the `PluginState` display forms.
    Lifecycle {
        plugin: String,
        from: String,
        to: String,
    },
    /// The registry's presence pair (invariant 5) mirrored onto the ring so
    /// one stream answers "what exists and what changed".
    Registry(ServiceEvent),
    /// 10 §2: the module became live in the address space — the pending-
    /// breakpoint trigger for off-the-shelf debug adapters. `load_base` is
    /// a stable address inside the mapped image (v0 hint: the descriptor).
    ModuleLoaded {
        plugin: String,
        artifact: String,
        load_base: u64,
        symbol_hint: String,
    },
    /// A timer job became due (D6/D20): the token is POSTED here — nothing
    /// called a closure to get this event.
    TimerDone { token: u64, job: String },
    /// An edge-ledger entry retired with its provider (D18).
    EdgeRemoved { subject: String },
    /// Reserved test/consumer marker for ring exercises (never emitted by
    /// the kernel itself).
    Marker(String),
    /// A logger record (D19, 10 §5): the same bounded ring as frames and
    /// lifecycle events, so logs replay on the one axis.
    Log(crate::logger::LogEvent),
}

/// One ring entry with its monotonic seq and the unified-clock stamp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObsRecord {
    pub seq: u64,
    pub ts: Timestamp,
    pub event: ObsEvent,
}

/// The result of `Observatory::tail`: retained records plus the honest
/// gap report (a consumer cannot be fooled about missing history).
#[derive(Debug, Clone)]
pub struct Tail {
    /// First seq still retained (genesis is seq 1; `> since_seq+1` = gap).
    pub first_seq: u64,
    /// Evictions this ring has ever shed.
    pub dropped: u64,
    pub events: Vec<ObsRecord>,
}

#[derive(Debug, Default)]
struct Ring {
    entries: VecDeque<ObsRecord>,
    next_seq: u64,
    dropped: u64,
}

/// The kernel-owned ring. Cheap to share (the kernel hands out one `Arc`).
#[derive(Debug)]
pub struct Observatory {
    clock: Clock,
    ring: Mutex<Ring>,
    /// Live edge ledger projection (D18, floor 2b): attached at boot; the
    /// emit doorway feeds both from the one event.
    edges: Mutex<Option<Arc<crate::edge_ledger::EdgeLedger>>>,
    /// D21 gate counters carried by the shared handle (floor 5): the
    /// registry and kernel doorways bump them through this one object.
    counters: crate::counters::GateCounters,
}

fn lock_ring(obs: &Observatory) -> MutexGuard<'_, Ring> {
    obs.ring.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Observatory {
    /// A ring stamped by `clock`, no ledger projection yet.
    #[must_use]
    pub fn new(clock: Clock) -> Self {
        Self {
            clock,
            ring: Mutex::new(Ring::default()),
            edges: Mutex::new(None),
            counters: crate::counters::GateCounters::new(),
        }
    }

    /// The gate counters that ride this shared handle (D21).
    #[must_use]
    pub fn counters(&self) -> &crate::counters::GateCounters {
        &self.counters
    }

    /// Attach the edge-ledger projection (kernel boot; one-shot).
    pub fn attach_edges(&self, ledger: Arc<crate::edge_ledger::EdgeLedger>) {
        let mut slot = self.edges.lock().unwrap_or_else(PoisonError::into_inner);
        *slot = Some(ledger);
    }

    /// Append one event: stamp with the unified clock, assign the next seq,
    /// shed + count the oldest when full. Never blocks on a consumer.
    pub fn emit(&self, event: ObsEvent) -> u64 {
        let ts = self.clock.now();
        let mut ring = lock_ring(self);
        ring.next_seq += 1;
        let seq = ring.next_seq;
        if ring.entries.len() >= RING_CAP {
            ring.entries.pop_front();
            ring.dropped += 1;
        }
        ring.entries.push_back(ObsRecord { seq, ts, event });
        let edges = self.edges.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(ledger) = &*edges {
            // Same doorway, two projections: ring entry and edge record
            // share seq + stamp, so they can never disagree (D18/D17).
            // just pushed above, so back() is Some; if-let keeps the
            // invariant read panic-free (D104: no expect in production).
            if let Some(last) = ring.entries.back() {
                ledger.record(seq, ts, &last.event);
            }
        }
        seq
    }

    /// Resumable read: all retained events with `seq > since_seq`.
    #[must_use]
    pub fn tail(&self, since_seq: u64) -> Tail {
        let ring = lock_ring(self);
        let kept: Vec<ObsRecord> = ring
            .entries
            .iter()
            .filter(|r| r.seq > since_seq)
            .cloned()
            .collect();
        Tail {
            first_seq: ring.entries.front().map_or(0, |r| r.seq),
            dropped: ring.dropped,
            events: kept,
        }
    }

    /// Seq of the most recently emitted event (0 when the ring is empty).
    #[must_use]
    pub fn last_seq(&self) -> u64 {
        lock_ring(self).next_seq
    }

    /// Entries currently retained.
    #[must_use]
    pub fn retained(&self) -> usize {
        lock_ring(self).entries.len()
    }

    /// Total evictions — the "drops must be visible" commandment, ring-side.
    #[must_use]
    pub fn drops(&self) -> u64 {
        lock_ring(self).dropped
    }
}
