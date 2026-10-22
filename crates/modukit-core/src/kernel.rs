//! The Kernel: manifest + state machine + C+ registry + executor seam.
//!
//! Default executor is the `dl` placement (Stage-1 scope, architecture.md
//! §10); the seam installs any executor of the closed set — S2-M1 adds the
//! `exe` branch (`ProcessExecutor`, see `executor`).
//! Boot orchestration here is the single-plugin spine: `install -> resolve
//! -> start -> stop -> uninstall` follows the modules/03 §2 guards; the M7
//! install/cross-check protocol lives in `bundle.rs`. Boot DAG: later.

use std::collections::HashMap;
use std::ffi::{c_char, c_void};
use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::abi::{HostAbiV1, NativeService, ServiceCallFn, cstr_to_string};
use crate::bus::Bus;
use crate::clock::{Clock, ClockDomain};
use crate::edge_ledger::EdgeLedger;
use crate::error::KernelError;
use crate::events::{ObsEvent, Observatory};
use crate::executor::{Executor, LoadedPlugin};
use crate::identity::{PluginId, ServiceId};
use crate::introspect::LoadStamp;
use crate::lifecycle::{PluginState, StateMachine};
use crate::logger::Logger;
use crate::manifest::Manifest;
use crate::registry::{Properties, PropertyValue, RegistrationId, Registry, RegistryError};

/// One tracked plugin: its manifest, state machine, loaded handle (while
/// resolved/started) and the service registrations it produced.
pub(crate) struct PluginRecord {
    pub(crate) manifest: Manifest,
    pub(crate) machine: StateMachine,
    pub(crate) handle: Option<Box<dyn LoadedPlugin + Send>>,
    pub(crate) registrations: Vec<RegistrationId>,
    /// Last load-handshake outcome for the `LedgerDump` service (D17).
    pub(crate) last_load: Option<LoadStamp>,
}

/// Per-`start` bookkeeping shared with the C ABI trampoline through the
/// opaque host `user` pointer. Lives on the kernel's stack only for the
/// duration of the call (01 §3 rule 2: the plugin keeps no ambient
/// authority).
struct ActivationSink<'a> {
    registry: &'a Registry,
    registered: Vec<RegistrationId>,
}

/// The C+ kernel for `dl` plugins.
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
    pub(crate) plugins: Arc<Mutex<HashMap<PluginId, PluginRecord>>>,
    /// Contract-ledger (fdset, D12) bytes this host booted with — the M7
    /// seam closing the M6 deferral; read back by `LedgerDump` byte-equal.
    pub(crate) contract_ledger: Arc<Mutex<Vec<u8>>>,
    /// Wave-2 T1 (D99 R4-A): the per-plugin timer register - token -> the
    /// plugin that scheduled it. Filled by the v3 `timer_schedule` door;
    /// read by `post_jobs` to scope the bus bell to registered plugin
    /// tokens; drained at stop/uninstall by `release_plugin_timers` (the
    /// "token never re-fires" census). Host-owned timers are NOT in it.
    pub(crate) plugin_timers: Mutex<HashMap<u64, PluginId>>,
}

fn lock(
    plugins: &Mutex<HashMap<PluginId, PluginRecord>>,
) -> MutexGuard<'_, HashMap<PluginId, PluginRecord>> {
    plugins.lock().unwrap_or_else(PoisonError::into_inner)
}

fn unknown(id: &PluginId) -> KernelError {
    KernelError::UnknownPlugin {
        id: id.as_str().to_string(),
    }
}

/// The `register_service` arm of `HostAbiV1`: builds a `NativeService` from
/// the C call table and registers it under the canonical id. Returns the
/// monotonic registration id, or -1 (the plugin's `start` then fails with a
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

    /// The service registry plugins acquire through.
    #[must_use]
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// The year-1 in-process topic bus (D10, modules/06 §5) plugins and the
    /// host publish/subscribe through. Separate channel from `registry()`.
    #[must_use]
    pub fn bus(&self) -> &Bus {
        &self.bus
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
        record: &mut PluginRecord,
        id: &PluginId,
        to: PluginState,
    ) -> Result<(), KernelError> {
        let from = record.machine.state();
        record.machine.transition(to)?;
        self.obs.counters().note_transition(); // D21 at the same doorway.
        self.obs.emit(ObsEvent::Lifecycle {
            plugin: id.to_string(),
            from: from.to_string(),
            to: to.to_string(),
        });
        Ok(())
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
    pub fn install(&self, manifest: Manifest) -> Result<PluginId, KernelError> {
        if !Path::new(&manifest.entry.artifact).is_file() {
            return Err(KernelError::ArtifactMissing {
                path: manifest.entry.artifact.clone(),
            });
        }
        let id = manifest.id.clone();
        let mut table = lock(&self.plugins);
        if let Some(record) = table.get_mut(&id) {
            if !matches!(
                record.machine.state(),
                PluginState::Uninstalled | PluginState::Resolved | PluginState::Failed
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
                        plugin: id.as_str().to_string(),
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
        self.mv(record, &id, PluginState::Installed)?;
        Ok(id)
    }

    /// INSTALLED -> RESOLVING -> STARTING. Runs the modules/03 §1 resolve
    /// pass: placement check, required-dependency validation, then the
    /// handshake with no plugin service logic executed. Every guard failure
    /// lands on the FAILED edge (04 §3: unknown ABI -> FAILED at
    /// RESOLVING, never a partial start).
    ///
    /// # Errors
    ///
    /// Any resolve-path `KernelError`; the plugin is parked in FAILED on
    /// every guard failure.
    pub fn resolve(&self, id: &PluginId) -> Result<(), KernelError> {
        let mut table = lock(&self.plugins);
        let record = table.get_mut(id).ok_or_else(|| unknown(id))?;
        self.mv(record, id, PluginState::Resolving)?;
        let outcome = resolve_inner(record, self.executor.as_ref(), &self.registry);
        // D21: the load handshake doorway counts its own outcome — the
        // bad-plugin class stays visible without adding any new gate.
        self.obs.counters().note_load(outcome.is_ok());
        match outcome {
            Ok(()) => {
                record.last_load = Some(LoadStamp::ok());
                if let Some(h) = record.handle.as_ref() {
                    // 10 §2: the cheap trigger for pending breakpoints.
                    self.obs.emit(ObsEvent::ModuleLoaded {
                        plugin: id.to_string(),
                        artifact: record.manifest.entry.artifact.clone(),
                        load_base: h.load_base(),
                        symbol_hint: "modukit_plugin_init".to_string(),
                    });
                }
                self.mv(record, id, PluginState::Starting)
            }
            Err(err) => {
                // FAILED edge (always legal from RESOLVING; the transition
                // helper's error cannot mask the original failure here.
                record.last_load = Some(LoadStamp::fail(&err));
                self.mv(record, id, PluginState::Failed)?;
                Err(err)
            }
        }
    }

    /// STARTING -> ACTIVE: the plugin's `start()` runs with the host table;
    /// services it registers are tracked under this plugin. `start_timeout`
    /// is recorded in the manifest but not enforced under `dl` (03 §5
    /// honest asymmetry: native code cannot be preempted).
    ///
    /// # Errors
    ///
    /// `KernelError::PluginStartFailed` on a non-zero status (FAILED edge
    /// taken); `InvalidTransition` from any other state.
    pub fn start(&self, id: &PluginId) -> Result<(), KernelError> {
        let mut table = lock(&self.plugins);
        let record = table.get_mut(id).ok_or_else(|| unknown(id))?;
        let handle = record
            .handle
            .as_mut()
            .ok_or_else(|| KernelError::InvalidTransition {
                from: record.machine.state().to_string(),
                to: "STARTING (no loaded artifact)".to_string(),
            })?;

        // Stage-5 N1 v2 door (D96 ruling 2): a plugin that declared major 2
        // is handed the HostAbiV2 table (wiring in host_v2.rs). Purely
        // additive - the v1 table build below stays byte-identical (D87).
        if handle.abi_major() == crate::abi::ABI_VERSION_V2 {
            let (started, registered) =
                crate::host_v2::start_v2(&mut **handle, &self.registry, &self.bus);
            record.registrations.extend(registered);
            if let Err(err) = started {
                self.mv(record, id, PluginState::Failed)?;
                return Err(err);
            }
            return self.mv(record, id, PluginState::Active);
        }

        // Wave-2 T1 v3 clock door (D99 R2): a plugin that declared major 3
        // is handed the HostAbiV3 table (wiring in host_v3.rs; its
        // timer_schedule slot fills the plugin-timer register below).
        // Purely additive - the v2 branch above stays byte-identical.
        if handle.abi_major() == crate::abi::ABI_VERSION_V3 {
            let (started, registered) = crate::host_v3::start_v3(
                &mut **handle,
                &self.registry,
                &self.bus,
                self.clock.clone(),
                id.clone(),
                &self.plugin_timers,
            );
            record.registrations.extend(registered);
            if let Err(err) = started {
                self.mv(record, id, PluginState::Failed)?;
                return Err(err);
            }
            return self.mv(record, id, PluginState::Active);
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
        // orphan services the plugin managed to publish.
        record.registrations.append(&mut sink.registered);
        if let Err(err) = started {
            self.mv(record, id, PluginState::Failed)?;
            return Err(err);
        }
        self.mv(record, id, PluginState::Active)
    }

    /// ACTIVE -> STOPPING -> RESOLVED: cooperative stop, then the kernel
    /// unregisters the plugin's services (one `Unregistered` per service,
    /// invariant 5). `stop_timeout` is recorded but not enforced for `dl`
    /// (05 §2: in-process code cannot be killed; the host never blocks the
    /// shutdown sequence past the budget — with no blocking mechanism at v0,
    /// the budget is documentation, per 03 §5).
    ///
    /// # Errors
    ///
    /// `KernelError` from a non-zero plugin stop status (FAILED edge) or
    /// registry bookkeeping failures other than already-removed records.
    pub fn stop(&self, id: &PluginId) -> Result<(), KernelError> {
        let mut table = lock(&self.plugins);
        let record = table.get_mut(id).ok_or_else(|| unknown(id))?;
        self.mv(record, id, PluginState::Stopping)?;

        let stopped = match record.handle.as_mut() {
            Some(handle) => handle.stop(),
            None => Ok(()),
        };
        // Drain this plugin's registrations whether or not the plugin
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
        // Wave-2 T1 (D99 R4-A): the authority-law release - this plugin's
        // registered tokens leave the clock and the register at stop, so a
        // stopped plugin's timer can never re-fire (the door-suite census).
        self.release_plugin_timers(id);
        match (stopped, last_err) {
            (Ok(()), None) => self.mv(record, id, PluginState::Resolved),
            (Err(err), _) => {
                self.mv(record, id, PluginState::Failed)?;
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
    pub fn uninstall(&self, id: &PluginId) -> Result<(), KernelError> {
        let mut table = lock(&self.plugins);
        let record = table.get_mut(id).ok_or_else(|| unknown(id))?;
        for rid in record.registrations.drain(..) {
            if let Err(e) = self.registry.unregister(rid)
                && !matches!(e, RegistryError::UnknownRegistration { .. })
            {
                return Err(e.into());
            }
        }
        // Wave-2 T1 (D99 R4-A): same release at the uninstall door (a stop
        // already drained it for a stopped plugin; the pairs are idempotent).
        self.release_plugin_timers(id);
        // Drop the handle before the state move so the artifact is released
        // exactly once.
        record.handle = None;
        self.mv(record, id, PluginState::Uninstalled)
    }

    /// Current state of a tracked plugin.
    ///
    /// # Errors
    ///
    /// `KernelError::UnknownPlugin` for an untracked id.
    pub fn state(&self, id: &PluginId) -> Result<PluginState, KernelError> {
        let table = lock(&self.plugins);
        let record = table.get(id).ok_or_else(|| unknown(id))?;
        Ok(record.machine.state())
    }

    /// Wave-2 T1 (D99 R4-A): the authority-law release at stop/uninstall -
    /// drop the plugin's tokens from the register AND from the clock, so a
    /// stopped or uninstalled plugin's timer can never re-fire. Kernel-side
    /// grammar, not an ABI verb: there is no cancel slot (D99 §3 non-
    /// decision). Returns nothing by design - the census is observable on
    /// `clock.pending_timers()` and the bus, not through this door.
    fn release_plugin_timers(&self, id: &PluginId) {
        let mut owned: Vec<u64> = Vec::new();
        {
            let mut register = self
                .plugin_timers
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

fn new_record(manifest: Manifest) -> PluginRecord {
    PluginRecord {
        manifest,
        machine: StateMachine::new(),
        handle: None,
        registrations: Vec::new(),
        last_load: None,
    }
}

/// The modules/03 §1 resolve pass, executor-flavored: required deps are
/// validated before anything runs, then the executor's `load` performs the
/// placement-floor check and the handshake (04 §3 "without executing plugin
/// code" in the sense of no lifecycle/service calls).
fn resolve_inner(
    record: &mut PluginRecord,
    executor: &dyn Executor,
    registry: &Registry,
) -> Result<(), KernelError> {
    // 01 §1 closed set, S2-M1: each executor answers exactly one placement
    // kind and refuses the others inside `load`, before touching the
    // artifact — the kernel stays placement-agnostic. Both gates remain
    // code-free and run before any plugin code executes (03 §1).
    for req in record.manifest.requires.iter().filter(|r| r.required) {
        if registry.lookup(&req.service).is_none() {
            return Err(KernelError::DependencyMissing {
                service: req.service.as_str().to_string(),
            });
        }
    }
    let handle = executor.load(&record.manifest)?;
    record.handle = Some(handle);
    Ok(())
}
