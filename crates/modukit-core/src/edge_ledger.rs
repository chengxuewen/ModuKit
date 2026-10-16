//! The live edge ledger (D18, modules/10 §6) — M6 floor 2b.
//!
//! One append-only ring of lifecycle/registry EDGES: who registered,
//! unregistered and transitioned, when (stamped by the floor-4 clock).
//! The ledger is fed by the SAME doorway the event ring uses
//! (`Observatory::attach_edges` + emit), so tables and events can never
//! disagree — the churn acceptance test leans on exactly that.
//!
//! Cap is the D18 verdict verbatim: long history belongs to recordings,
//! never to RAM (retention/quota policy is trigger-gated; year 1 keeps one
//! constant). `(caller, service, method)` CALL edges — the D18 client-stub
//! table — grow the doorway with M4/codegen consumer stubs; the v0 ledger
//! records the registry/lifecycle families the kernel can attribute without
//! inventing a caller identity across the ABI (see the M6 ambiguity list).

use std::collections::VecDeque;
use std::sync::Mutex;

use crate::clock::Timestamp;
use crate::events::ObsEvent;

/// Ledger capacity — the "~100-line, in-RAM, ring-capped" D18 shape.
pub const EDGE_CAP: usize = 4096;

/// Which edge family a record belongs to (modules/10 §6 event set).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    /// A service appeared on the table (invariant 5 pair, half one).
    Registered,
    /// A provider retired — its entries retire with it (the `EdgeRemoved`
    /// moment for this ledger; D18).
    Unregistered,
    /// A plugin lifecycle transition (03 §2 edge, floor-2a doorway).
    Transition,
    /// A module became live in the address space (10 §2).
    ModuleLoaded,
}

/// One edge record. `seq` mirrors the event ring's monotonic seq (the fold
/// checker pairs records by it); `subject` is the service id or plugin id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeRecord {
    pub seq: u64,
    pub ts: Timestamp,
    pub kind: EdgeKind,
    pub subject: String,
    /// Human-readable shape (`from->to` for transitions, methodless for
    /// presence pairs) — plain data, no formatting beyond Debug/Display.
    pub detail: String,
}

#[derive(Debug, Default)]
struct State {
    entries: VecDeque<EdgeRecord>,
    dropped: u64,
}

/// The ledger. Shared with the observatory; queryable snapshot only —
/// there is no mutation path besides the emit projection (read-only face
/// per D17).
#[derive(Debug)]
pub struct EdgeLedger {
    state: Mutex<State>,
}

impl Default for EdgeLedger {
    fn default() -> Self {
        Self {
            state: Mutex::new(State::default()),
        }
    }
}

impl EdgeLedger {
    /// An empty ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Project one ring event into an edge record (called at the emit
    /// doorway). Events outside the edge families are ignored, cheaply.
    pub fn record(&self, seq: u64, ts: Timestamp, event: &ObsEvent) {
        let (kind, subject, detail) = match event {
            ObsEvent::Registry(crate::registry::ServiceEvent::Registered {
                service_id, ..
            }) => (EdgeKind::Registered, service_id.to_string(), String::new()),
            ObsEvent::Registry(crate::registry::ServiceEvent::Unregistered {
                service_id, ..
            }) => (
                EdgeKind::Unregistered,
                service_id.to_string(),
                String::new(),
            ),
            ObsEvent::Lifecycle {
                plugin, from, to, ..
            } => (
                EdgeKind::Transition,
                plugin.clone(),
                format!("{from}->{to}"),
            ),
            ObsEvent::ModuleLoaded {
                plugin, artifact, ..
            } => (EdgeKind::ModuleLoaded, plugin.clone(), artifact.clone()),
            _ => return,
        };
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.entries.len() >= EDGE_CAP {
            state.entries.pop_front();
            state.dropped += 1;
        }
        state.entries.push_back(EdgeRecord {
            seq,
            ts,
            kind,
            subject,
            detail,
        });
    }

    /// All retained edges with `seq > since_seq`, oldest first.
    #[must_use]
    pub fn since_seq(&self, since_seq: u64) -> Vec<EdgeRecord> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .entries
            .iter()
            .filter(|e| e.seq > since_seq)
            .cloned()
            .collect()
    }

    /// Edges currently retained.
    #[must_use]
    pub fn retained(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entries
            .len()
    }

    /// Edges ever shed (retention is visible, never silent).
    #[must_use]
    pub fn drops(&self) -> u64 {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .dropped
    }
}
