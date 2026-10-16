//! The introspect trio (D17, modules/10 §1) — M6 floor 1.
//!
//! Three READ-ONLY, side-effect-free services in the reserved
//! `org.modukit.*` namespace: `LedgerDump.dump()`, `Snapshot.all()/one()`,
//! `EventStream.tail(since_seq)`. They expose (a) state/registry
//! introspection, (b) the merged event feed, (c) the object-table cut —
//! zero UI, zero privileges, zero mutation (D17: if a tool needs more, the
//! tool is wrong, not the kernel). Trust-tier filtering (D2) arrives with
//! the multi-vendor trigger; v0 is single-cockpit.
//!
//! Each trio member is registered as an ordinary service instance (D40) and
//! is `Clone`-out-of-`ServiceRef` cheap: they hold shared handles, not the
//! kernel itself.

use std::sync::Arc;

use crate::bus::TopicStats;
use crate::counters::CountersView;
use crate::edge_ledger::{EdgeKind, EdgeLedger};
use crate::events::{Observatory, Tail};
use crate::identity::PluginId;
use crate::kernel::PluginRecord;
use crate::registry::{RegisteredService, Registry};
use crate::wiring::PluginTable;

/// One load-handshake outcome record: host/ABI stamps as plain numbers
/// (the `LedgerDump` row of 10 §1; the D27 record family).
#[derive(Debug, Clone, Copy)]
pub struct LoadStamp {
    pub expected: u32,
    pub found: u32,
    pub loaded: bool,
}

impl LoadStamp {
    pub(crate) fn ok() -> Self {
        Self {
            expected: crate::abi::ABI_VERSION,
            found: crate::abi::ABI_VERSION,
            loaded: true,
        }
    }

    pub(crate) fn fail(err: &crate::error::KernelError) -> Self {
        let found = match err {
            crate::error::KernelError::AbiMismatch { found, .. } => *found,
            _ => 0,
        };
        Self {
            expected: crate::abi::ABI_VERSION,
            found,
            loaded: false,
        }
    }
}

/// The per-plugin line of the object-table cut (10 §1 Snapshot row:
/// state/placement/version; restart counts ride M7's restart machinery).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginView {
    pub id: String,
    /// `PluginState` display form (`"ACTIVE"`, `"FAILED"`, ...).
    pub state: String,
    pub placement: String,
    pub version: String,
}

/// The live-ledger line of the cut (D18 `Snapshot.edges()`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeView {
    pub seq: u64,
    pub ts_raw: u64,
    pub kind: EdgeKind,
    pub subject: String,
    pub detail: String,
}

/// The exact `Snapshot.all()` answer: one cut of the object table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostView {
    pub plugins: Vec<PluginView>,
    pub services: Vec<RegisteredService>,
    /// Topics with pub/sub/drop counters (10 §1 Snapshot row).
    pub topics: Vec<TopicStats>,
    pub edges: Vec<EdgeView>,
    /// Gate counters (D21 `Snapshot.counters()`).
    pub counters: CountersView,
}

impl HostView {
    /// `Snapshot.one(handle)`: v0 looks plugins up by id string (the
    /// in-process analogue of a table handle; id-string keys are what the
    /// reserved namespace actually has until bridge handles exist).
    #[must_use]
    pub fn one(&self, id: &str) -> Option<&PluginView> {
        self.plugins.iter().find(|p| p.id == id)
    }
}

/// The snapshot service (`org.modukit.snapshot`).
#[derive(Clone)]
pub struct Snapshot {
    registry: Registry,
    bus: Arc<crate::bus::Bus>,
    table: PluginTable,
    obs: Arc<Observatory>,
    edges: Arc<EdgeLedger>,
}

impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Snapshot(introspect)")
    }
}

impl Snapshot {
    pub(crate) fn new(
        registry: &Registry,
        bus: Arc<crate::bus::Bus>,
        table: PluginTable,
        obs: Arc<Observatory>,
        edges: Arc<EdgeLedger>,
    ) -> Self {
        Self {
            registry: registry.shared(),
            bus,
            table,
            obs,
            edges,
        }
    }

    /// The object-table cut: plugins, services, topics (with drop
    /// counters), edges, counters — one consistent read (each source is
    /// snapshotted under its own lock; year-1 single-writer discipline).
    #[must_use]
    pub fn all(&self) -> HostView {
        let plugins: Vec<PluginView> = {
            let table = self
                .table
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            table
                .iter()
                .map(|(id, record)| view_plugin(id, record))
                .collect()
        };
        let counters = {
            let g = self.obs.counters();
            let mut view = CountersView {
                registry_acquires: g.registry_acquires(),
                registry_releases: g.registry_releases(),
                lifecycle_transitions: g.lifecycle_transitions(),
                load_ok: g.load_ok(),
                load_fail: g.load_fail(),
                process_crashes: g.process_crashes(),
                process_restarts: g.process_restarts(),
                ..Default::default()
            };
            let topics = self.bus.topic_stats();
            for st in &topics {
                view.bus_published += st.published;
                view.bus_delivered += st.delivered;
                view.bus_dropped += st.dropped + st.retention_dropped;
            }
            (topics, view)
        };
        HostView {
            plugins,
            services: self.registry.snapshot(),
            topics: counters.0,
            edges: self
                .edges
                .since_seq(0)
                .into_iter()
                .map(|e| EdgeView {
                    seq: e.seq,
                    ts_raw: e.ts.raw(),
                    kind: e.kind,
                    subject: e.subject,
                    detail: e.detail,
                })
                .collect(),
            counters: counters.1,
        }
    }

    /// `Snapshot.one`: a plugin line by id string.
    #[must_use]
    pub fn one(&self, id: &str) -> Option<PluginView> {
        let table = self
            .table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        table
            .iter()
            .find(|(pid, _)| pid.as_str() == id)
            .map(|(pid, record)| view_plugin(pid, record))
    }

    /// Clone out of a registry `ServiceRef` for tool consumers.
    #[must_use]
    pub fn clone_inner(&self) -> Self {
        self.clone()
    }
}

/// The event feed (`org.modukit.event_stream`): tail of the merged ring.
#[derive(Clone)]
pub struct EventStream {
    obs: Arc<Observatory>,
}

impl std::fmt::Debug for EventStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EventStream(introspect)")
    }
}

impl EventStream {
    pub(crate) fn new(obs: Arc<Observatory>) -> Self {
        Self { obs }
    }

    /// `EventStream.tail(since_seq)`: monotonic, resumable, gap-honest.
    #[must_use]
    pub fn tail(&self, since_seq: u64) -> Tail {
        self.obs.tail(since_seq)
    }

    /// Clone out of a registry `ServiceRef` for tool consumers.
    #[must_use]
    pub fn clone_inner(&self) -> Self {
        self.clone()
    }
}

/// One recorded module attempt in the load ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEntry {
    pub plugin: String,
    pub artifact: String,
    /// True once the ABI handshake passed and the image is mapped.
    pub loaded: bool,
    /// Host/ABI version stamps (10 §1 `LedgerDump` row; the fdset half of
    /// the dump is `LedgerDump::contract_ledger()`, fed by the M7
    /// `Kernel::set_contract_ledger` boot seam).
    pub abi_stamps: AbiStamps,
}

/// `{expected, found}` handshake stamps as plain numbers (D27 record).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AbiStamps {
    pub expected: u32,
    pub found: u32,
}

/// The load ledger (`org.modukit.ledger_dump`).
#[derive(Clone)]
pub struct LedgerDump {
    table: PluginTable,
    /// M7: the contract-ledger (fdset) bytes the kernel was booted with.
    contract: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
}

impl std::fmt::Debug for LedgerDump {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LedgerDump(introspect)")
    }
}

impl LedgerDump {
    pub(crate) fn new(
        table: PluginTable,
        contract: std::sync::Arc<std::sync::Mutex<Vec<u8>>>,
    ) -> Self {
        Self { table, contract }
    }

    /// The contract-ledger (protoc fdset, D12) bytes this host was booted
    /// with via `Kernel::set_contract_ledger` — BYTE-EQUAL to what was
    /// handed in (the M7 acceptance of the M6 "fdset equality rides
    /// M7/M8" deferral). Empty until the seam is fed.
    #[must_use]
    pub fn contract_ledger(&self) -> Vec<u8> {
        self.contract
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// The exact set of modules this host has attempted to load, with the
    /// stamps that decided each outcome (10 §1; consumed by CLI `ledger`,
    /// the recorder header, and the bridge self-check).
    #[must_use]
    pub fn dump(&self) -> Vec<ModuleEntry> {
        let table = self
            .table
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        table
            .iter()
            .map(|(id, record)| ModuleEntry {
                plugin: id.to_string(),
                artifact: record.manifest.entry.artifact.clone(),
                loaded: record.last_load.is_some_and(|s| s.loaded),
                abi_stamps: record
                    .last_load
                    .map(|s| AbiStamps {
                        expected: s.expected,
                        found: s.found,
                    })
                    .unwrap_or(AbiStamps {
                        expected: crate::abi::ABI_VERSION,
                        found: 0,
                    }),
            })
            .collect()
    }

    /// Clone out of a registry `ServiceRef` for tool consumers.
    #[must_use]
    pub fn clone_inner(&self) -> Self {
        self.clone()
    }
}

pub(crate) fn view_plugin(id: &PluginId, record: &PluginRecord) -> PluginView {
    PluginView {
        id: id.to_string(),
        state: record.machine.state().to_string(),
        placement: record.manifest.entry.kind.to_string(),
        version: record.manifest.version.clone(),
    }
}
