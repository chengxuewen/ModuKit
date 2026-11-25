//! C+ service registry (D4, modules/02 §1).
//!
//! Shapes mirror the doc's API block:
//! `register(Arc<S>, props) -> RegistrationId`, `acquire::<S>() ->
//! Option<ServiceRef<S>>`, `release(&ServiceRef<S>)`, `subscribe()`,
//! `snapshot()`. The six invariants carried here: handles not instances (1);
//! monotonic `i64` ids (2); case-insensitive flat property map (3); reserved
//! `ranking` used as tie-break only (4); `Registered`/`Unregistered`/`Modified`
//! events beside `snapshot()` (5 — `Modified` ships in S2 via `modify_props`,
//! the canonical `service.id` immutable); acquire/release call shape with
//! internal refcount (6). Property-filter query (`find`) ships in S1;
//! filtered subscription (`subscribe_filtered`) in S2; no tracker, no factory,
//! no leases yet (modules/02 §5 non-goals; tracker follows in S2).

use std::any::{Any, TypeId};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::ops::Deref;
use std::sync::mpsc::{Receiver, SyncSender, TrySendError, sync_channel};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::events::{ObsEvent, Observatory};
use crate::filter::{Filter, FilterError};
use crate::identity::ServiceId;
pub use crate::registry_types::{
    Properties, PropertyValue, RegisteredService, RegistrationId, ServiceEvent,
};

/// Registry-side failure (D27 code space lives in `KernelError::code()`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// Canonical key must ride the properties under `service.id` (D9).
    MissingServiceId,
    /// Unknown or already removed registration id.
    UnknownRegistration { id: i64 },
    /// The same handle released twice — call sites owe exactly one release.
    AlreadyReleased { id: i64 },
    /// `modify_props` tried to change the canonical `service.id` (S2: the
    /// identity is immutable; only the surrounding properties may change).
    ServiceIdImmutable { id: i64 },
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
            RegistryError::ServiceIdImmutable { id } => {
                write!(
                    f,
                    "registration {id}: the canonical service.id is immutable"
                )
            }
        }
    }
}

impl std::error::Error for RegistryError {}

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
    /// Subscribers with an optional property filter (S2): `None` receives
    /// every event; `Some(f)` receives only events whose props match `f`.
    senders: Vec<(SyncSender<ServiceEvent>, Option<Filter>)>,
    /// Shared observatory ring (M6): the kernel wires it at boot; a
    /// standalone `Registry::new()` has none and behaves exactly as before.
    observability: Option<Arc<Observatory>>,
}

/// Backpressure floor for the v0 event stream: a bounded window per
/// subscriber; a subscriber that falls behind loses events (kernel keeps
/// moving), which the later observability ring (M6) replaces honestly.
const EVENT_QUEUE: usize = 64;

impl Storage {
    fn emit(&mut self, event: ServiceEvent) {
        // Merged ring first (modules/10 §1: one stream answers both
        // channels); the per-subscriber mpsc window keeps its own semantics.
        if let Some(obs) = &self.observability {
            obs.emit(ObsEvent::Registry(event.clone()));
        }
        self.senders.retain(|(tx, filter)| {
            if let Some(f) = filter {
                let props = match &event {
                    ServiceEvent::Registered { props, .. }
                    | ServiceEvent::Unregistered { props, .. }
                    | ServiceEvent::Modified { props, .. } => props,
                };
                if !f.matches(props) {
                    return true; // keep the subscriber; this event is not its business
                }
            }
            match tx.try_send(event.clone()) {
                Ok(()) | Err(TrySendError::Full(_)) => true,
                Err(TrySendError::Disconnected(_)) => false,
            }
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

    /// Registry wired to the shared observatory ring (kernel boot path).
    #[must_use]
    pub(crate) fn with_observability(obs: Arc<Observatory>) -> Self {
        Self {
            storage: Arc::new(Mutex::new(Storage {
                observability: Some(obs),
                ..Storage::default()
            })),
        }
    }

    /// Another handle to the SAME storage: the introspect trio and the
    /// kernel's platform services are wired against one shared table.
    pub(crate) fn shared(&self) -> Registry {
        self.clone()
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
            RegistrationId::at(store.next_id)
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
        let obs = store.observability.clone(); // D21 gate handle (before the entry borrow).
        let entry = store.entries.get_mut(&best)?;
        let instance = Arc::clone(&entry.instance);
        entry.holds += 1;
        let Ok(instance) = instance.downcast::<S>() else {
            entry.holds -= 1;
            return None;
        };
        if let Some(obs) = &obs {
            obs.counters().note_acquire(); // counted at the acquire doorway.
        }
        Some(ServiceRef {
            storage: Arc::clone(&self.storage),
            id: RegistrationId::at(best),
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
    /// path unregisters a bundle's services when the bundle stops).
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
        lock(&self.storage).senders.push((tx, None));
        rx
    }

    /// Filtered subscription (S2): only events whose properties match the
    /// RFC1960-subset filter are delivered. Late-join discipline is the
    /// caller's: take `find(filter)` first, then subscribe, exactly as
    /// `snapshot()` + `subscribe()` pair for the unfiltered stream.
    ///
    /// # Errors
    ///
    /// `FilterError` when the filter string is malformed.
    pub fn subscribe_filtered(&self, filter: &str) -> Result<Receiver<ServiceEvent>, FilterError> {
        let parsed = Filter::parse(filter)?;
        let (tx, rx) = sync_channel::<ServiceEvent>(EVENT_QUEUE);
        lock(&self.storage).senders.push((tx, Some(parsed)));
        Ok(rx)
    }

    /// Change a registration's properties (S2): emits `Modified`. The
    /// canonical `service.id` is immutable - the new props must carry the
    /// same id; ranking is re-derived from the reserved property.
    ///
    /// # Errors
    ///
    /// `UnknownRegistration` for an absent id; `MissingServiceId` when the
    /// new props lack a string `service.id`; `ServiceIdImmutable` when the
    /// id differs from the registered one.
    pub fn modify_props(&self, id: RegistrationId, props: Properties) -> Result<(), RegistryError> {
        let mut store = lock(&self.storage);
        let entry = store
            .entries
            .get_mut(&id.value())
            .ok_or(RegistryError::UnknownRegistration { id: id.value() })?;
        let new_id = match props.get("service.id") {
            Some(PropertyValue::Str(s)) if !s.is_empty() => ServiceId::new(s.clone()),
            _ => return Err(RegistryError::MissingServiceId),
        };
        if new_id != entry.service_id {
            return Err(RegistryError::ServiceIdImmutable { id: id.value() });
        }
        entry.ranking = match props.get("ranking") {
            Some(PropertyValue::Int(r)) => *r,
            _ => 0,
        };
        entry.props = props.clone();
        let service_id = entry.service_id.clone();
        store.emit(ServiceEvent::Modified {
            id,
            service_id,
            props,
        });
        Ok(())
    }

    #[must_use]
    pub fn snapshot(&self) -> Vec<RegisteredService> {
        let store = lock(&self.storage);
        store.entries.values().map(view).collect()
    }

    /// Property-filter query (bundle-management-plan S1, ruling A2): parse an
    /// RFC1960-subset filter and return the matching registrations best-first
    /// (ranking descending, then registration order - the `acquire` tie-break).
    /// A well-formed filter naming an undeclared property simply yields no
    /// match; it never panics.
    ///
    /// # Errors
    ///
    /// `FilterError` when the filter string is malformed.
    pub fn find(&self, filter: &str) -> Result<Vec<RegisteredService>, FilterError> {
        let parsed = Filter::parse(filter)?;
        let store = lock(&self.storage);
        let mut hits: Vec<&Entry> = store
            .entries
            .values()
            .filter(|e| parsed.matches(&e.props))
            .collect();
        hits.sort_by(|a, b| {
            b.ranking
                .cmp(&a.ranking)
                .then(a.id.value().cmp(&b.id.value()))
        });
        Ok(hits.into_iter().map(view).collect())
    }
}

impl Clone for Registry {
    /// Another handle sharing the one storage ("cheap to clone", see the
    /// type doc). Same table, same events, same counters.
    fn clone(&self) -> Self {
        Self {
            storage: Arc::clone(&self.storage),
        }
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
        if let Some(obs) = &store.observability {
            obs.counters().note_release(); // D21: pairs counted at the release doorway.
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

#[cfg(test)]
mod find_tests {
    use super::*;

    struct Svc;

    fn reg(r: &Registry, id: &str, ranking: i64) {
        let mut p = Properties::new();
        p.insert("service.id", PropertyValue::Str(id.into()));
        p.insert("ranking", PropertyValue::Int(ranking));
        r.register(Arc::new(Svc), p).unwrap();
    }

    #[test]
    fn find_matches_and_orders_best_first() {
        let r = Registry::new();
        reg(&r, "com.acme.Nav", 10);
        reg(&r, "com.acme.Nav", 100);
        reg(&r, "com.acme.Log", 50);
        let hits = r.find("(service.id=com.acme.Nav)").unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].ranking, 100); // best-first
        assert_eq!(hits[1].ranking, 10);
        let ge = r.find("(&(service.id=com.acme.Nav)(ranking>=50))").unwrap();
        assert_eq!(ge.len(), 1);
        assert_eq!(ge[0].ranking, 100);
    }

    #[test]
    fn find_undeclared_property_is_empty_not_panic() {
        let r = Registry::new();
        reg(&r, "com.acme.Nav", 1);
        assert!(r.find("(ghost=x)").unwrap().is_empty());
    }

    #[test]
    fn find_malformed_filter_redds() {
        let r = Registry::new();
        assert!(r.find("(service.id=").is_err());
        assert!(r.find("(&)").is_err());
    }
}

#[cfg(test)]
mod s2_tests {
    use super::*;

    struct Svc;

    fn props(id: &str, ranking: i64) -> Properties {
        let mut p = Properties::new();
        p.insert("service.id", PropertyValue::Str(id.into()));
        p.insert("ranking", PropertyValue::Int(ranking));
        p
    }

    #[test]
    fn modify_props_emits_modified_and_updates() {
        let r = Registry::new();
        let id = r.register(Arc::new(Svc), props("com.acme.Nav", 1)).unwrap();
        let rx = r.subscribe();
        r.modify_props(id, props("com.acme.Nav", 9)).unwrap();
        let ev = rx.try_recv().expect("a Modified event");
        match ev {
            ServiceEvent::Modified { id: mid, props, .. } => {
                assert_eq!(mid, id);
                assert_eq!(props.get("ranking"), Some(&PropertyValue::Int(9)));
            }
            other => panic!("expected Modified, got {other:?}"),
        }
        assert_eq!(
            r.lookup(&ServiceId::new("com.acme.Nav".into()))
                .unwrap()
                .ranking,
            9
        );
    }

    #[test]
    fn modify_props_refuses_unknown_id_and_id_change() {
        let r = Registry::new();
        let id = r.register(Arc::new(Svc), props("com.acme.Nav", 1)).unwrap();
        // unknown id
        assert!(matches!(
            r.modify_props(RegistrationId::at(999), props("com.acme.Nav", 1)),
            Err(RegistryError::UnknownRegistration { .. })
        ));
        // changing the canonical id is refused
        assert!(matches!(
            r.modify_props(id, props("com.acme.Other", 1)),
            Err(RegistryError::ServiceIdImmutable { .. })
        ));
        // missing service.id is refused
        assert!(matches!(
            r.modify_props(id, Properties::new()),
            Err(RegistryError::MissingServiceId)
        ));
    }

    #[test]
    fn filtered_subscription_only_delivers_matches() {
        let r = Registry::new();
        let rx = r.subscribe_filtered("(service.id=com.acme.Nav)").unwrap();
        r.register(Arc::new(Svc), props("com.acme.Log", 1)).unwrap();
        r.register(Arc::new(Svc), props("com.acme.Nav", 1)).unwrap();
        let ev = rx.try_recv().expect("the Nav registration");
        match ev {
            ServiceEvent::Registered { service_id, .. } => {
                assert_eq!(service_id.as_str(), "com.acme.Nav");
            }
            other => panic!("expected Nav Registered, got {other:?}"),
        }
        assert!(rx.try_recv().is_err(), "the Log event must not arrive");
        // malformed filter reddens
        assert!(r.subscribe_filtered("(service.id=").is_err());
    }
}
