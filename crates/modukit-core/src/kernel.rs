//! The Kernel: manifest + state machine + C+ registry + executor seam.
//!
//! Default executor is the `dl` placement (Stage-1 scope, architecture.md
//! §10); the seam installs any executor of the closed set — S2-M1 adds the
//! `exe` branch (`ProcessExecutor`, see `executor`).
//! Boot orchestration here is the single-bundle spine: `install -> resolve
//! -> start -> stop -> uninstall` follows the modules/03 §2 guards; the M7
//! install/cross-check protocol lives in `bundle.rs`. Boot DAG: later.

use std::collections::HashMap;
use std::ffi::{c_char, c_void};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::abi::{HostAbiV1, NativeService, ServiceCallFn, cstr_to_string};
use crate::bus::Bus;
use crate::clock::{Clock, ClockDomain};
use crate::dm::{DmHandler, RequiredDep};
use crate::edge_ledger::EdgeLedger;
use crate::error::KernelError;
use crate::events::{ObsEvent, Observatory};
use crate::executor::{Executor, LoadedPlugin};
use crate::identity::{BundleId, ServiceId};
use crate::introspect::LoadStamp;
use crate::lifecycle::{BundleState, StateMachine};
use crate::logger::Logger;
use crate::manifest::{Manifest, OnDown};
use crate::registry::{Properties, PropertyValue, RegistrationId, Registry, RegistryError};
use std::sync::atomic::Ordering;

/// One tracked bundle: its manifest, state machine, loaded handle (while
/// resolved/started) and the service registrations it produced.
pub(crate) struct BundleRecord {
    pub(crate) manifest: Manifest,
    pub(crate) machine: StateMachine,
    pub(crate) handle: Option<Box<dyn LoadedPlugin + Send>>,
    pub(crate) registrations: Vec<RegistrationId>,
    /// Last load-handshake outcome for the `LedgerDump` service (D17).
    pub(crate) last_load: Option<LoadStamp>,
}

/// Per-`start` bookkeeping shared with the C ABI trampoline through the
/// opaque host `user` pointer. Lives on the kernel's stack only for the
/// duration of the call (01 §3 rule 2: the bundle keeps no ambient
/// authority).
struct ActivationSink<'a> {
    registry: &'a Registry,
    registered: Vec<RegistrationId>,
}

/// The C+ kernel for `dl` bundles.
pub struct Kernel {
    pub(crate) registry: Registry,
    pub(crate) bus: Arc<Bus>,
    pub(crate) clock: Clock,
    /// Shared observatory ring (D17): the merged event stream behind the
    /// introspect trio and the logger floor.
    pub(crate) obs: Arc<Observatory>,
    pub(crate) logger: Arc<Logger>,
    pub(crate) edges: Arc<EdgeLedger>,
    pub(crate) executor: Box<dyn Executor>,
    pub(crate) bundles: Arc<Mutex<HashMap<BundleId, BundleRecord>>>,
    /// Contract-ledger (fdset, D12) bytes this host booted with — the M7
    /// seam closing the M6 deferral; read back by `LedgerDump` byte-equal.
    pub(crate) contract_ledger: Arc<Mutex<Vec<u8>>>,
    /// Wave-2 T1 (D99 R4-A): the per-bundle timer register - token -> the
    /// bundle that scheduled it. Filled by the v3 `timer_schedule` door;
    /// read by `post_jobs` to scope the bus bell to registered bundle
    /// tokens; drained at stop/uninstall by `release_plugin_timers` (the
    /// "token never re-fires" census). Host-owned timers are NOT in it.
    pub(crate) bundle_timers: Mutex<HashMap<u64, BundleId>>,
    /// S3-1b DM-lite wiring (bundle-management-plan): the dependency
    /// monitor plus the blocked / deferred-restart / dm-failed markers.
    /// Drained at lifecycle-verb tails outside the bundles lock; the depth
    /// guard keeps nested drains (dep_lost, then stop, then drain) from
    /// double-dispatching.
    pub(crate) dm: Mutex<crate::dm::DependencyMonitor>,
    pub(crate) dm_state: Mutex<DmState>,
    pub(crate) dm_depth: std::sync::atomic::AtomicU32,
    /// S3-2 ConfigAdmin: the D29 hot-lane door (declare at install, set/get
    /// for hosts; undeclared keys refuse outright). Persistence rides
    /// build/config-live; the deploy-TOML seeds it once that type lands.
    pub(crate) config: crate::config::ConfigAdmin,
}

/// The DM-lite marker sets (S3-1b): one lock, three per-bundle flags.
#[derive(Debug, Default)]
pub(crate) struct DmState {
    /// `on_down = block`: start is refused until the dep is restored.
    pub(crate) blocked: std::collections::BTreeSet<BundleId>,
    /// `on_down = restart-self`: auto-started on dep restore.
    pub(crate) restart_pending: std::collections::BTreeSet<BundleId>,
    /// `on_down = fail-self`: parked on the Resolved tier with a failed
    /// marker (the Failed state is a start-time edge, 03 §2).
    pub(crate) failed: std::collections::BTreeSet<BundleId>,
}

/// One `dm_report` row (S4 ops face): a required dependency, its loss
/// policy, and whether the service is present right now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DmRow {
    pub bundle: String,
    pub service: String,
    pub on_down: String,
    pub present: bool,
}

fn lock(
    bundles: &Mutex<HashMap<BundleId, BundleRecord>>,
) -> MutexGuard<'_, HashMap<BundleId, BundleRecord>> {
    bundles.lock().unwrap_or_else(PoisonError::into_inner)
}

fn unknown(id: &BundleId) -> KernelError {
    KernelError::UnknownPlugin {
        id: id.as_str().to_string(),
    }
}

/// The `register_service` arm of `HostAbiV1`: builds a `NativeService` from
/// the C call table and registers it under the canonical id. Returns the
/// monotonic registration id, or -1 (the bundle's `start` then fails with a
/// typed error via its non-zero status).
///
/// # Safety
/// `user` must be a live `&mut ActivationSink` pointer (kernel-owned,
/// passed through `HostAbiV1` and valid for the whole `start` call);
/// `service_id` a readable NUL-terminated C string.
unsafe extern "C" fn trampoline_register_service(
    user: *mut c_void,
    service_id: *const c_char,
    call: ServiceCallFn,
) -> i64 {
    // SAFETY: the kernel only ever passes a `&mut ActivationSink` as `user`
    // and keeps it alive across the `start` call it originated from.
    let sink = unsafe { &mut *(user.cast::<ActivationSink<'_>>()) };
    let Some(id_str) = cstr_to_string(service_id) else {
        return -1;
    };
    let service_id = ServiceId::new(id_str.clone());
    let mut props = Properties::new();
    props.insert("service.id", PropertyValue::Str(id_str));
    let instance = Arc::new(NativeService::new(service_id, call));
    match sink.registry.register(instance, props) {
        Ok(rid) => {
            sink.registered.push(rid);
            rid.value()
        }
        Err(_) => -1,
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

impl Kernel {
    /// Kernel with the `dl` executor (the Stage-1 default placement).
    #[must_use]
    pub fn new() -> Self {
        Self::with_clock(Clock::new(ClockDomain::Real))
    }

    /// Kernel with an explicit executor of the closed set (S2-M1: the
    /// supervisor installs `ProcessExecutor` here; placement kinds are
    /// policed by each executor's own `load`, not by the kernel).
    #[must_use]
    pub fn with_executor(executor: Box<dyn Executor>) -> Self {
        Self::with_clock_and_executor(Clock::new(ClockDomain::Real), executor)
    }

    // Boot wiring (with_clock, floor accessors, platform service
    // registration) lives in `wiring.rs` so this file stays the lifecycle
    // spine. `new()` above delegates to `Kernel::with_clock`.

    /// The service registry bundles acquire through.
    #[must_use]
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// The year-1 in-process topic bus (D10, modules/06 §5) bundles and the
    /// host publish/subscribe through. Separate channel from `registry()`.
    #[must_use]
    pub fn bus(&self) -> &Bus {
        &self.bus
    }

    /// D108: install the general parent->child publish hook at the single
    /// boot seam. The hook observes every publish after fan-out and outside
    /// the topics lock; `MODUKIT_HOST_PUSH_OFF` skips the install at this
    /// one site. A kernel without supervised children installs nothing.
    pub(crate) fn attach_parent_fanout(&self, fan: Arc<crate::bus_wire::ParentFanout>) {
        let host_push_off = std::env::var_os("MODUKIT_HOST_PUSH_OFF").is_some();
        self.bus
            .set_publish_hook(Arc::new(move |topic, qos, value| {
                if host_push_off && crate::bus_wire::is_host_origin() {
                    return;
                }
                fan.pump_topic(topic, crate::bus_wire::qos_to_i32(qos), value);
            }));
    }

    /// Boot seam (M7, closing the M6 "fdset equality rides M7/M8"
    /// deferral): record the contract-ledger (protoc fdset, D12) bytes
    /// this host boots with; `LedgerDump::contract_ledger()` reads them
    /// back byte-equal. The M6 module-ledger half of the dump is
    /// untouched.
    pub fn set_contract_ledger(&self, bytes: Vec<u8>) {
        *self
            .contract_ledger
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = bytes;
    }

    /// The kernel-side transition doorway: move the machine and post the
    /// lifecycle edge onto the shared ring (same doorway, richer, D21).
    fn mv(
        &self,
        record: &mut BundleRecord,
        id: &BundleId,
        to: BundleState,
    ) -> Result<(), KernelError> {
        let from = record.machine.state();
        record.machine.transition(to)?;
        self.obs.counters().note_transition(); // D21 at the same doorway.
        self.obs.emit(ObsEvent::Lifecycle {
            bundle: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
        });
        Ok(())
    }

    /// S3-1b: pump the dependency monitor into the kernel actions. Drained
    /// at lifecycle-verb tails; a nested call (dep_lost -> stop -> drain)
    /// returns immediately and the outer pump converges - the depth guard
    /// keeps this recursion bounded and deadlock-free.
    // Hosts and the embed face poll this after registry mutations (S3-1b):
    // the monitor dispatches dep_lost/dep_restored to the kernel actions.
    pub fn dm_drain(&self) {
        if self.dm_depth.fetch_add(1, Ordering::SeqCst) != 0 {
            self.dm_depth.fetch_sub(1, Ordering::SeqCst);
            return; // already draining deeper in this call stack
        }
        let mut dm = self.dm.lock().unwrap_or_else(PoisonError::into_inner);
        let mut handler = KernelDm { kernel: self };
        dm.drain(&mut handler);
        self.dm_depth.fetch_sub(1, Ordering::SeqCst);
    }

    /// Register a bundle's required dependencies for DM-lite watching
    /// (install tail). Only `required` deps carrying an `on_down` policy
    /// are watched - optional deps have no lifecycle consequence.
    pub(crate) fn dm_register_dependents(&self, id: &BundleId, deps: Vec<RequiredDep>) {
        let mut dm = self.dm.lock().unwrap_or_else(PoisonError::into_inner);
        dm.register_dependent(id.clone(), deps);
    }

    pub(crate) fn dm_forget(&self, id: &BundleId) {
        let mut dm = self.dm.lock().unwrap_or_else(PoisonError::into_inner);
        dm.forget(id);
    }

    /// UNINSTALLED -> INSTALLED. §2 guard: manifest valid (already parsed)
    /// and artifact present.
    ///
    /// # Errors
    ///
    /// `KernelError::ArtifactMissing` when the declared artifact is not a
    /// file; `InvalidTransition` when the id is already tracked in a live
    /// state (duplicate-identity policing lands with the M7 install
    /// protocol, D34).
    pub fn install(&self, manifest: Manifest) -> Result<BundleId, KernelError> {
        if !Path::new(&manifest.entry.artifact).is_file() {
            return Err(KernelError::ArtifactMissing {
                path: manifest.entry.artifact.clone(),
            });
        }
        let id = manifest.id.clone();
        // S3-1b: declare the watched dependencies before the manifest moves
        // into the record (only required deps with a policy drive DM-lite).
        let dm_deps: Vec<RequiredDep> = manifest
            .requires
            .iter()
            .filter(|r| r.required)
            .filter_map(|r| {
                r.on_down.map(|on_down| RequiredDep {
                    service: r.service.clone(),
                    on_down,
                })
            })
            .collect();
        // S3-2: the D29 hot-lane declaration rides the same install tail.
        let config_live = manifest.config_live.clone();
        let mut table = lock(&self.bundles);
        if let Some(record) = table.get_mut(&id) {
            if !matches!(
                record.machine.state(),
                BundleState::Uninstalled | BundleState::Resolved | BundleState::Failed
            ) {
                // D80 (S2-M3b): a co-host request at the swap door gets a
                // typed deferral, not a silent ignore and not the bare M7
                // refusal — either side's declaration is the voice's cue
                // (the old version's claim is as real a co-host ask as the
                // incoming one's). Stop-based swap stays the legal lane.
                let claim = manifest
                    .parallel_ok_claim()
                    .or_else(|| record.manifest.parallel_ok_claim())
                    .map(str::to_string);
                if let Some(service) = claim {
                    return Err(KernelError::RoutingDeferred {
                        bundle: id.as_str().to_string(),
                        service,
                    });
                }
                return Err(KernelError::InvalidTransition {
                    from: record.machine.state().to_string(),
                    to: "INSTALLED (re-install refused: M7 duplicate-identity policy)".to_string(),
                });
            }
            // Terminal-state re-install replaces the record (fresh machine).
            *record = new_record(manifest);
        } else {
            table.insert(id.clone(), new_record(manifest));
        }
        let record = table.get_mut(&id).ok_or_else(|| unknown(&id))?;
        self.mv(record, &id, BundleState::Installed)?;
        self.dm_register_dependents(&id, dm_deps);
        self.config.declare(id.clone(), config_live);
        Ok(id)
    }

    /// INSTALLED -> RESOLVING -> STARTING. Runs the modules/03 §1 resolve
    /// pass: placement check, required-dependency validation, then the
    /// handshake with no bundle service logic executed. Every guard failure
    /// lands on the FAILED edge (04 §3: unknown ABI -> FAILED at
    /// RESOLVING, never a partial start).
    ///
    /// # Errors
    ///
    /// Any resolve-path `KernelError`; the bundle is parked in FAILED on
    /// every guard failure.
    pub fn resolve(&self, id: &BundleId) -> Result<(), KernelError> {
        let mut table = lock(&self.bundles);
        let record = table.get_mut(id).ok_or_else(|| unknown(id))?;
        self.mv(record, id, BundleState::Resolving)?;
        let outcome = resolve_inner(record, self.executor.as_ref(), &self.registry);
        // D21: the load handshake doorway counts its own outcome — the
        // bad-bundle class stays visible without adding any new gate.
        self.obs.counters().note_load(outcome.is_ok());
        match outcome {
            Ok(()) => {
                record.last_load = Some(LoadStamp::ok());
                if let Some(h) = record.handle.as_ref() {
                    // 10 §2: the cheap trigger for pending breakpoints.
                    self.obs.emit(ObsEvent::ModuleLoaded {
                        bundle: id.to_string(),
                        artifact: record.manifest.entry.artifact.clone(),
                        load_base: h.load_base(),
                        symbol_hint: "modukit_bundle_init".to_string(),
                    });
                }
                self.mv(record, id, BundleState::Starting)
            }
            Err(err) => {
                // FAILED edge (always legal from RESOLVING; the transition
                // helper's error cannot mask the original failure here.
                record.last_load = Some(LoadStamp::fail(&err));
                self.mv(record, id, BundleState::Failed)?;
                Err(err)
            }
        }
    }

    /// STARTING -> ACTIVE: the bundle's `start()` runs with the host table;
    /// services it registers are tracked under this bundle. `start_timeout`
    /// is recorded in the manifest but not enforced under `dl` (03 §5
    /// honest asymmetry: native code cannot be preempted).
    ///
    /// # Errors
    ///
    /// `KernelError::BundleStartFailed` on a non-zero status (FAILED edge
    /// taken); `InvalidTransition` from any other state.
    pub fn start(&self, id: &BundleId) -> Result<(), KernelError> {
        // S3-1b: a DM-blocked bundle refuses start until its dependency is
        // restored and the block is cleared (the requires check below would
        // refuse anyway, but the typed error names the DM state).
        {
            let state = self.dm_state.lock().unwrap_or_else(PoisonError::into_inner);
            if state.blocked.contains(id) {
                return Err(KernelError::DependencyBlocked {
                    bundle: id.as_str().to_string(),
                });
            }
        }
        let mut table = lock(&self.bundles);
        let record = table.get_mut(id).ok_or_else(|| unknown(id))?;
        let handle = record
            .handle
            .as_mut()
            .ok_or_else(|| KernelError::InvalidTransition {
                from: record.machine.state().to_string(),
                to: "STARTING (no loaded artifact)".to_string(),
            })?;

        // Stage-5 N1 v2 door (D96 ruling 2): a bundle that declared major 2
        // is handed the HostAbiV2 table (wiring in host_v2.rs). Purely
        // additive - the v1 table build below stays byte-identical (D87).
        if handle.abi_major() == crate::abi::ABI_VERSION_V2 {
            let (started, registered) =
                crate::host_v2::start_v2(&mut **handle, &self.registry, &self.bus);
            record.registrations.extend(registered);
            if let Err(err) = started {
                self.mv(record, id, BundleState::Failed)?;
                return Err(err);
            }
            return self.mv(record, id, BundleState::Active);
        }

        // Wave-2 T1 v3 clock door (D99 R2): a bundle that declared major 3
        // is handed the HostAbiV3 table (wiring in host_v3.rs; its
        // timer_schedule slot fills the bundle-timer register below).
        // Purely additive - the v2 branch above stays byte-identical.
        if handle.abi_major() == crate::abi::ABI_VERSION_V3 {
            let (started, registered) = crate::host_v3::start_v3(
                &mut **handle,
                &self.registry,
                &self.bus,
                self.clock.clone(),
                id.clone(),
                &self.bundle_timers,
            );
            record.registrations.extend(registered);
            if let Err(err) = started {
                self.mv(record, id, BundleState::Failed)?;
                return Err(err);
            }
            return self.mv(record, id, BundleState::Active);
        }

        let mut sink = ActivationSink {
            registry: &self.registry,
            registered: Vec::new(),
        };
        let host = HostAbiV1 {
            size: HostAbiV1::expected_size(),
            user: &raw mut sink as *mut c_void,
            register_service: trampoline_register_service,
        };
        let started = handle.start(std::ptr::from_ref(&host));
        // Collect what registered even on failure: the error path must not
        // orphan services the bundle managed to publish.
        record.registrations.append(&mut sink.registered);
        if let Err(err) = started {
            self.mv(record, id, BundleState::Failed)?;
            return Err(err);
        }
        self.mv(record, id, BundleState::Active)
    }

    /// ACTIVE -> STOPPING -> RESOLVED: cooperative stop, then the kernel
    /// unregisters the bundle's services (one `Unregistered` per service,
    /// invariant 5). `stop_timeout` is recorded but not enforced for `dl`
    /// (05 §2: in-process code cannot be killed; the host never blocks the
    /// shutdown sequence past the budget — with no blocking mechanism at v0,
    /// the budget is documentation, per 03 §5).
    ///
    /// # Errors
    ///
    /// `KernelError` from a non-zero bundle stop status (FAILED edge) or
    /// registry bookkeeping failures other than already-removed records.
    pub fn stop(&self, id: &BundleId) -> Result<(), KernelError> {
        let mut table = lock(&self.bundles);
        let record = table.get_mut(id).ok_or_else(|| unknown(id))?;
        self.mv(record, id, BundleState::Stopping)?;

        let stopped = match record.handle.as_mut() {
            Some(handle) => handle.stop(),
            None => Ok(()),
        };
        // Drain this bundle's registrations whether or not the bundle
        // cooperated; an already-removed record is not an error (pairs are
        // idempotent at the kernel boundary).
        let mut last_err = None;
        for rid in record.registrations.drain(..) {
            if let Err(e) = self.registry.unregister(rid)
                && !matches!(e, RegistryError::UnknownRegistration { .. })
            {
                last_err = Some(e);
            }
        }
        // Wave-2 T1 (D99 R4-A): the authority-law release - this bundle's
        // registered tokens leave the clock and the register at stop, so a
        // stopped bundle's timer can never re-fire (the door-suite census).
        self.release_plugin_timers(id);
        match (stopped, last_err) {
            (Ok(()), None) => self.mv(record, id, BundleState::Resolved),
            (Err(err), _) => {
                self.mv(record, id, BundleState::Failed)?;
                Err(err)
            }
            (Ok(()), Some(err)) => Err(err.into()),
        }
    }

    /// Any -> UNINSTALLED: release services, drop the artifact (the `dl`
    /// handle's drop closes the library; dlclose is not a reload strategy
    /// per D8 — nothing here pretends otherwise).
    ///
    /// # Errors
    ///
    /// `KernelError::UnknownPlugin` for an untracked id.
    pub fn uninstall(&self, id: &BundleId) -> Result<(), KernelError> {
        let mut table = lock(&self.bundles);
        let record = table.get_mut(id).ok_or_else(|| unknown(id))?;
        for rid in record.registrations.drain(..) {
            if let Err(e) = self.registry.unregister(rid)
                && !matches!(e, RegistryError::UnknownRegistration { .. })
            {
                return Err(e.into());
            }
        }
        // Wave-2 T1 (D99 R4-A): same release at the uninstall door (a stop
        // already drained it for a stopped bundle; the pairs are idempotent).
        self.release_plugin_timers(id);
        // Drop the handle before the state move so the artifact is released
        // exactly once.
        record.handle = None;
        let res = self.mv(record, id, BundleState::Uninstalled);
        self.dm_forget(id);
        self.config.forget(id);
        res
    }

    /// Hot-apply one config key (S3-2, D29 hot lane): only keys the bundle
    /// declared in `config_live` are accepted; accepted keys are stored,
    /// persisted to build/config-live, and published on `config/<bundle>`.
    ///
    /// # Errors
    ///
    /// `ConfigError::NotLive` for an undeclared key (the liar-family
    /// refusal); `ConfigError::Persist` when the disk write fails.
    pub fn set_config(
        &self,
        bundle: &BundleId,
        key: &str,
        value: &str,
    ) -> Result<(), crate::config::ConfigError> {
        self.config.set(bundle, key, value)
    }

    /// Read one config value (the year-1 value read path, 05 §6).
    #[must_use]
    pub fn get_config(&self, bundle: &BundleId, key: &str) -> Option<String> {
        self.config.get(bundle, key)
    }

    /// S4 ops face: one row per REQUIRED dependency, with its loss policy
    /// and current presence — the `dm` diagnose command's data source
    /// (bundle-management-plan S4; the celix `dm wtf` shape).
    #[must_use]
    pub fn dm_report(&self) -> Vec<DmRow> {
        let table = lock(&self.bundles);
        let mut rows = Vec::new();
        for record in table.values() {
            let bundle = record.manifest.id.as_str().to_string();
            for req in &record.manifest.requires {
                if !req.required {
                    continue;
                }
                let on_down = req
                    .on_down
                    .as_ref()
                    .map_or("stop".to_string(), |o| match o {
                        OnDown::FailSelf => "fail-self".to_string(),
                        OnDown::Block => "block".to_string(),
                        OnDown::RestartSelf => "restart-self".to_string(),
                    });
                rows.push(DmRow {
                    bundle: bundle.clone(),
                    service: req.service.as_str().to_string(),
                    on_down,
                    present: self.registry.lookup(&req.service).is_some(),
                });
            }
        }
        rows.sort_by(|a, b| a.bundle.cmp(&b.bundle).then(a.service.cmp(&b.service)));
        rows
    }

    /// Current state of a tracked bundle.
    ///
    /// # Errors
    ///
    /// `KernelError::UnknownPlugin` for an untracked id.
    pub fn state(&self, id: &BundleId) -> Result<BundleState, KernelError> {
        let table = lock(&self.bundles);
        let record = table.get(id).ok_or_else(|| unknown(id))?;
        Ok(record.machine.state())
    }

    /// Wave-2 T1 (D99 R4-A): the authority-law release at stop/uninstall -
    /// drop the bundle's tokens from the register AND from the clock, so a
    /// stopped or uninstalled bundle's timer can never re-fire. Kernel-side
    /// grammar, not an ABI verb: there is no cancel slot (D99 §3 non-
    /// decision). Returns nothing by design - the census is observable on
    /// `clock.pending_timers()` and the bus, not through this door.
    fn release_plugin_timers(&self, id: &BundleId) {
        let mut owned: Vec<u64> = Vec::new();
        {
            let mut register = self
                .bundle_timers
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            register.retain(|token, pid| {
                if pid == id {
                    owned.push(*token);
                    false
                } else {
                    true
                }
            });
        }
        self.clock.remove_by_tokens(&owned);
    }
}

fn new_record(manifest: Manifest) -> BundleRecord {
    BundleRecord {
        manifest,
        machine: StateMachine::new(),
        handle: None,
        registrations: Vec::new(),
        last_load: None,
    }
}

/// The modules/03 §1 resolve pass, executor-flavored: required deps are
/// validated before anything runs, then the executor's `load` performs the
/// placement-floor check and the handshake (04 §3 "without executing bundle
/// code" in the sense of no lifecycle/service calls).
fn resolve_inner(
    record: &mut BundleRecord,
    executor: &dyn Executor,
    registry: &Registry,
) -> Result<(), KernelError> {
    // 01 §1 closed set, S2-M1: each executor answers exactly one placement
    // kind and refuses the others inside `load`, before touching the
    // artifact — the kernel stays placement-agnostic. Both gates remain
    // code-free and run before any bundle code executes (03 §1).
    for req in record.manifest.requires.iter().filter(|r| r.required) {
        let Some(provider) = registry.lookup(&req.service) else {
            return Err(KernelError::DependencyMissing {
                service: req.service.as_str().to_string(),
            });
        };
        // S2 (ruling S2-3 A): a declared version range must be satisfied by
        // the provider's `service.version` property; an absent or non-matching
        // version is a deterministic resolve failure before any bundle code.
        if let Some(range) = &req.version {
            let satisfied = match provider.props.get("service.version") {
                Some(PropertyValue::Str(v)) => range.matches(v),
                _ => false,
            };
            if !satisfied {
                return Err(KernelError::DependencyMissing {
                    service: req.service.as_str().to_string(),
                });
            }
        }
    }
    let handle = executor.load(&record.manifest)?;
    record.handle = Some(handle);
    Ok(())
}

/// The kernel-side `DmHandler` (S3-1b): loss stops the dependent and parks it
/// per policy; a restore clears the markers and auto-restarts the
/// `restart-self` dependents. Every action re-enters the public verbs, which
/// drain the monitor at their tails - the depth guard keeps this recursion
/// bounded and the pump converges (transition-only notifications).
struct KernelDm<'a> {
    kernel: &'a Kernel,
}

impl DmHandler for KernelDm<'_> {
    fn dep_lost(&mut self, bundle: &BundleId, dep: &RequiredDep) {
        // Stop first: the bundle's own services leave the table, which is
        // exactly what feeds the other dependents' edges. stop lands in
        // Resolved; the Failed state is a start-time edge (03 §2), so
        // fail-self parks Resolved with a dm-failed marker rather than
        // bending the machine.
        let _ = self.kernel.stop(bundle);
        {
            let mut state = self
                .kernel
                .dm_state
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            match dep.on_down {
                OnDown::FailSelf => {
                    state.failed.insert(bundle.clone());
                }
                OnDown::Block => {
                    state.blocked.insert(bundle.clone());
                }
                OnDown::RestartSelf => {
                    state.restart_pending.insert(bundle.clone());
                }
            }
        }
        self.kernel.dm_drain();
    }

    fn dep_restored(&mut self, bundle: &BundleId, _dep: &RequiredDep) {
        let auto_restart;
        {
            let mut state = self
                .kernel
                .dm_state
                .lock()
                .unwrap_or_else(PoisonError::into_inner);
            state.blocked.remove(bundle);
            state.failed.remove(bundle);
            auto_restart = state.restart_pending.remove(bundle);
        }
        if auto_restart {
            let _ = self.kernel.start(bundle);
        }
        self.kernel.dm_drain();
    }
}
