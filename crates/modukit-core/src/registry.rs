//! C+ service registry (D4, modules/02 §1).
//!
//! Shapes mirror the doc's API block:
//! `register(Arc<S>, props) -> RegistrationId`, `acquire::<S>() ->
//! Option<ServiceRef<S>>`, `release(&ServiceRef<S>)`, `subscribe()`,
//! `snapshot()`. The six invariants carried here: handles not instances (1);
//! monotonic `i64` ids (2); case-insensitive flat property map (3); reserved
//! `ranking` used as tie-break only (4); `Registered`/`Unregistered` event
//! pairs beside `snapshot()` (5 — `Modified` is a documented additive
//! extension, deliberately not shipped in year 1); acquire/release call shape
//! with internal refcount (6). No property filters, no tracker, no factory,
//! no leases (modules/02 §5 non-goals).

use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::identity::ServiceId;

/// Monotonic registration identity (invariant 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegistrationId(i64);

impl RegistrationId {
    #[must_use]
    pub fn value(self) -> i64 {
        self.0
    }
}

/// Flat property value vocabulary (invariant 3: the string-keyed future
/// host of LDAP vocabulary; year 1 keeps three scalar shapes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

/// Flat property map with case-insensitive keys (invariant 3). Keys are
/// stored folded to lowercase; `get` folds the query the same way.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Properties(BTreeMap<String, PropertyValue>);

impl Properties {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: &str, value: PropertyValue) -> Option<PropertyValue> {
        self.0.insert(key.to_lowercase(), value)
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&PropertyValue> {
        self.0.get(&key.to_lowercase())
    }
}

/// Registry-side failure (D27 code space lives in `KernelError::code()`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// Canonical key must ride the properties under `service.id` (D9).
    MissingServiceId,
    /// Unknown or already removed registration id.
    UnknownRegistration { id: i64 },
    /// The same handle released twice — call sites owe exactly one release.
    AlreadyReleased { id: i64 },
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistryError::MissingServiceId => {
                write!(f, "registration properties must carry `service.id` (D9)")
            }
            RegistryError::UnknownRegistration { id } => {
                write!(f, "no registration with id {id}")
            }
            RegistryError::AlreadyReleased { id } => {
                write!(f, "handle for registration {id} was already released")
            }
        }
    }
}

impl std::error::Error for RegistryError {}

/// Presence event (invariant 5). `Modified` is reserved-additive, not shipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceEvent {
    Registered {
        id: RegistrationId,
        service_id: ServiceId,
        props: Properties,
    },
    Unregistered {
        id: RegistrationId,
        service_id: ServiceId,
        props: Properties,
    },
}

/// Snapshot view of one registration (invariant 5's late-join half).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredService {
    pub id: RegistrationId,
    pub service_id: ServiceId,
    /// Reserved storage, year-1 tie-break only (invariant 4).
    pub ranking: i64,
    pub props: Properties,
}

#[derive(Debug)]
struct Entry {
    id: RegistrationId,
    service_id: ServiceId,
    type_id: TypeId,
    instance: Arc<dyn Any + Send + Sync>,
    ranking: i64,
    props: Properties,
    holds: i64,
}

#[derive(Debug, Default)]
struct Storage {
    entries: BTreeMap<i64, Entry>,
    next_id: i64,
    senders: Vec<SyncSender<ServiceEvent>>,
}

/// Backpressure floor for the v0 event stream: a bounded window per
/// subscriber; a subscriber that falls behind loses events (kernel keeps
/// moving), which the later observability ring (M6) replaces honestly.
const EVENT_QUEUE: usize = 64;

impl Storage {
    fn emit(&mut self, event: ServiceEvent) {
        self.senders.retain(|tx| match tx.try_send(event.clone()) {
            Ok(()) | Err(TrySendError::Full(_)) => true,
            Err(TrySendError::Disconnected(_)) => false,
        });
    }
}

fn lock(storage: &Mutex<Storage>) -> MutexGuard<'_, Storage> {
    storage.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The C+ registry handle (cheap to clone: one shared storage).
#[derive(Debug, Default)]
pub struct Registry {
    storage: Arc<Mutex<Storage>>,
}

impl Registry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an instance. The canonical `service.id` (D9) is read from
    /// `props`; ranking from the reserved `ranking` int property (invariant
    /// 4). Returns the monotonic id used by events and release bookkeeping.
    ///
    /// # Errors
    ///
    /// `RegistryError::MissingServiceId` when `props` has no string
    /// `service.id`.
    pub fn register<S: Any + Send + Sync>(
        &self,
        instance: Arc<S>,
        props: Properties,
    ) -> Result<RegistrationId, RegistryError> {
        let service_id = match props.get("service.id") {
            Some(PropertyValue::Str(s)) if !s.is_empty() => ServiceId::new(s.clone()),
            _ => return Err(RegistryError::MissingServiceId),
        };
        let ranking = match props.get("ranking") {
            Some(PropertyValue::Int(r)) => *r,
            _ => 0,
        };
        let mut store = lock(&self.storage);
        // ids are positive and monotonic from 1 (invariant 2).
        let id = {
            store.next_id += 1;
            RegistrationId(store.next_id)
        };
        let event_props = props.clone();
        store.emit(ServiceEvent::Registered {
            id,
            service_id: service_id.clone(),
            props: event_props,
        });
        store.entries.insert(
            id.value(),
            Entry {
                id,
                service_id,
                type_id: TypeId::of::<S>(),
                instance,
                ranking,
                props,
                holds: 0,
            },
        );
        Ok(id)
    }

    /// Typed handle acquisition (invariant 1: a handle, never the raw
    /// instance; derefs locally). Same-type candidates tie-break on the
    /// reserved ranking, then registration order (invariant 4).
    #[must_use]
    pub fn acquire<S: Any + Send + Sync>(&self) -> Option<ServiceRef<S>> {
        let want = TypeId::of::<S>();
        let mut store = lock(&self.storage);
        let best = store
            .entries
            .values()
            .filter(|e| e.type_id == want)
            .max_by_key(|e| (e.ranking, -e.id.value()))
            .map(|e| e.id.value())?;
        let entry = store.entries.get_mut(&best)?;
        let instance = Arc::clone(&entry.instance);
        entry.holds += 1;
        let Ok(instance) = instance.downcast::<S>() else {
            entry.holds -= 1;
            return None;
        };
        Some(ServiceRef {
            storage: Arc::clone(&self.storage),
            id: RegistrationId(best),
            service_id: entry.service_id.clone(),
            instance,
            live: Cell::new(true),
        })
    }

    /// Close one acquire handle (invariant 6: pairs are mandatory at call
    /// sites even though the local transport keeps refcounts internally).
    ///
    /// # Errors
    ///
    /// `RegistryError::AlreadyReleased` when the same handle is released
    /// twice.
    pub fn release<S: Any + Send + Sync>(&self, r: &ServiceRef<S>) -> Result<(), RegistryError> {
        r.release_inner()
    }

    /// Remove a registration (kernel-side housekeeping: the `dl` executor
    /// path unregisters a plugin's services when the plugin stops).
    ///
    /// # Errors
    ///
    /// `RegistryError::UnknownRegistration` for an absent id.
    pub fn unregister(&self, id: RegistrationId) -> Result<(), RegistryError> {
        let mut store = lock(&self.storage);
        let entry = store
            .entries
            .remove(&id.value())
            .ok_or(RegistryError::UnknownRegistration { id: id.value() })?;
        store.emit(ServiceEvent::Unregistered {
            id,
            service_id: entry.service_id.clone(),
            props: entry.props.clone(), // props-at-last-known (invariant 5)
        });
        Ok(())
    }

    /// Metadata lookup by canonical service id (D9); handles come from
    /// `acquire::<S>`.
    #[must_use]
    pub fn lookup(&self, service_id: &ServiceId) -> Option<RegisteredService> {
        let store = lock(&self.storage);
        store
            .entries
            .values()
            .find(|e| e.service_id == *service_id)
            .map(view)
    }

    /// Presence stream (invariant 5). Late joiners are defined, not
    /// accidental: take `snapshot()` first, then events.
    #[must_use]
    pub fn subscribe(&self) -> Receiver<ServiceEvent> {
        let (tx, rx) = sync_channel::<ServiceEvent>(EVENT_QUEUE);
        lock(&self.storage).senders.push(tx);
        rx
    }

    #[must_use]
    pub fn snapshot(&self) -> Vec<RegisteredService> {
        let store = lock(&self.storage);
        store.entries.values().map(view).collect()
    }
}

fn view(e: &Entry) -> RegisteredService {
    RegisteredService {
        id: e.id,
        service_id: e.service_id.clone(),
        ranking: e.ranking,
        props: e.props.clone(),
    }
}

/// A handle to a registered instance (invariant 1). Derefs to the instance;
/// the matching `Registry::release` marks it closed. Dropping an unreleased
/// handle also decrements the internal refcount (the count is transport
/// bookkeeping, not a lease).
pub struct ServiceRef<S> {
    storage: Arc<Mutex<Storage>>,
    id: RegistrationId,
    service_id: ServiceId,
    instance: Arc<S>,
    live: Cell<bool>,
}

impl<S> ServiceRef<S> {
    #[must_use]
    pub fn id(&self) -> RegistrationId {
        self.id
    }

    #[must_use]
    pub fn service_id(&self) -> &ServiceId {
        &self.service_id
    }

    pub(crate) fn release_inner(&self) -> Result<(), RegistryError> {
        if !self.live.replace(false) {
            return Err(RegistryError::AlreadyReleased {
                id: self.id.value(),
            });
        }
        let mut store = lock(&self.storage);
        if let Some(entry) = store.entries.get_mut(&self.id.value()) {
            entry.holds -= 1;
        }
        Ok(())
    }
}

impl<S> Deref for ServiceRef<S> {
    type Target = S;

    fn deref(&self) -> &S {
        &self.instance
    }
}

impl<S> Drop for ServiceRef<S> {
    fn drop(&mut self) {
        if self.live.replace(false) {
            let mut store = lock(&self.storage);
            if let Some(entry) = store.entries.get_mut(&self.id.value()) {
                entry.holds -= 1;
            }
        }
    }
}
