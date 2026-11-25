//! DM-lite dependency monitor (bundle-management-plan S3, ruling S3-1 A).
//!
//! Activates the previously-inert `on_down` declaration: a bundle's required
//! dependencies drive its lifecycle. The monitor watches the registry's
//! presence stream and, when a required service transitions present->absent,
//! dispatches the dependent's `on_down` policy to a `DmHandler` (the kernel
//! wires the actual stop / restart / block action); absent->present dispatches
//! a restore so start eligibility can be re-evaluated.
//!
//! Deliberately bundle-level and bounded (celix DM is the reference, lighter
//! than CppMS SCR; module 02 §5 warns against full component machinery): one
//! bundle = one component, single-threaded drain, transition-only notifications
//! (no duplicate fires while a service stays absent). No intra-bundle
//! multi-component SCR, no cardinality/locking strategies in year 1.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::mpsc::Receiver;

use crate::identity::{BundleId, ServiceId};
use crate::manifest::OnDown;
use crate::registry::{Registry, ServiceEvent};

/// One required dependency of a bundle, with its loss policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequiredDep {
    pub service: ServiceId,
    pub on_down: OnDown,
}

/// The lifecycle actions the kernel implements; the monitor only decides WHEN
/// and WHY, never how to stop/restart a bundle (that stays in the kernel).
pub trait DmHandler {
    /// A required service went away; apply the dependent's `on_down` policy.
    fn dep_lost(&mut self, bundle: &BundleId, dep: &RequiredDep);
    /// A required service (re)appeared; re-evaluate start eligibility.
    fn dep_restored(&mut self, bundle: &BundleId, dep: &RequiredDep);
}

/// Watches presence and dispatches dependency transitions to a handler.
pub struct DependencyMonitor {
    rx: Receiver<ServiceEvent>,
    dependents: BTreeMap<BundleId, Vec<RequiredDep>>,
    available: BTreeSet<ServiceId>,
}

impl DependencyMonitor {
    /// Open a monitor on the registry's presence stream, seeded with the
    /// currently-registered services (late-join: snapshot then subscribe).
    #[must_use]
    pub fn new(registry: &Registry) -> Self {
        let available = registry
            .snapshot()
            .into_iter()
            .map(|s| s.service_id)
            .collect();
        let rx = registry.subscribe();
        Self {
            rx,
            dependents: BTreeMap::new(),
            available,
        }
    }

    /// Declare a bundle's required dependencies (idempotent replace).
    pub fn register_dependent(&mut self, bundle: BundleId, deps: Vec<RequiredDep>) {
        self.dependents.insert(bundle, deps);
    }

    /// Remove a bundle's declarations (it uninstalled).
    pub fn forget(&mut self, bundle: &BundleId) {
        self.dependents.remove(bundle);
    }

    /// Pump pending presence events, dispatching only real transitions.
    pub fn drain(&mut self, handler: &mut dyn DmHandler) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                ServiceEvent::Registered { service_id, .. } => {
                    if self.available.insert(service_id.clone()) {
                        self.notify(handler, &service_id, true);
                    }
                }
                ServiceEvent::Unregistered { service_id, .. } => {
                    if self.available.remove(&service_id) {
                        self.notify(handler, &service_id, false);
                    }
                }
                ServiceEvent::Modified { .. } => {} // presence unchanged
            }
        }
    }

    fn notify(&self, handler: &mut dyn DmHandler, service: &ServiceId, present: bool) {
        for (bundle, deps) in &self.dependents {
            for dep in deps {
                if &dep.service == service {
                    if present {
                        handler.dep_restored(bundle, dep);
                    } else {
                        handler.dep_lost(bundle, dep);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry_types::{Properties, PropertyValue};
    use std::sync::Arc;

    struct Svc;

    #[derive(Default)]
    struct Recorder {
        lost: Vec<(String, String)>,
        restored: Vec<(String, String)>,
    }

    impl DmHandler for Recorder {
        fn dep_lost(&mut self, bundle: &BundleId, dep: &RequiredDep) {
            self.lost.push((
                bundle.as_str().to_string(),
                dep.service.as_str().to_string(),
            ));
        }
        fn dep_restored(&mut self, bundle: &BundleId, dep: &RequiredDep) {
            self.restored.push((
                bundle.as_str().to_string(),
                dep.service.as_str().to_string(),
            ));
        }
    }

    fn props(id: &str) -> Properties {
        let mut p = Properties::new();
        p.insert("service.id", PropertyValue::Str(id.into()));
        p
    }

    fn dep(service: &str) -> RequiredDep {
        RequiredDep {
            service: ServiceId::new(service.into()),
            on_down: OnDown::Block,
        }
    }

    #[test]
    fn dep_loss_and_restore_dispatch_once_per_transition() {
        let r = Registry::new();
        let id = r.register(Arc::new(Svc), props("com.acme.Nav")).unwrap();
        let mut mon = DependencyMonitor::new(&r);
        mon.register_dependent(
            BundleId::new("com.acme.consumer".into()),
            vec![dep("com.acme.Nav")],
        );
        let mut rec = Recorder::default();
        // loss -> one dep_lost
        r.unregister(id).unwrap();
        mon.drain(&mut rec);
        assert_eq!(rec.lost.len(), 1);
        assert_eq!(rec.lost[0].0, "com.acme.consumer");
        // a second unregister of an already-absent service does not re-fire
        mon.drain(&mut rec);
        assert_eq!(rec.lost.len(), 1);
        // restore -> one dep_restored
        r.register(Arc::new(Svc), props("com.acme.Nav")).unwrap();
        mon.drain(&mut rec);
        assert_eq!(rec.restored.len(), 1);
    }

    #[test]
    fn unrelated_service_and_modified_do_not_fire() {
        let r = Registry::new();
        let mut mon = DependencyMonitor::new(&r);
        mon.register_dependent(
            BundleId::new("com.acme.consumer".into()),
            vec![dep("com.acme.Nav")],
        );
        let mut rec = Recorder::default();
        let other = r.register(Arc::new(Svc), props("com.acme.Log")).unwrap();
        r.unregister(other).unwrap();
        mon.drain(&mut rec);
        assert!(rec.lost.is_empty(), "unrelated service must not fire");
        // modify a tracked service: presence unchanged, no fire
        let nav = r.register(Arc::new(Svc), props("com.acme.Nav")).unwrap();
        mon.drain(&mut rec); // restored fires once
        r.modify_props(nav, props("com.acme.Nav")).unwrap();
        mon.drain(&mut rec);
        assert_eq!(rec.restored.len(), 1, "Modified must not re-fire restore");
        assert!(rec.lost.is_empty());
    }

    #[test]
    fn forget_stops_notifications() {
        let r = Registry::new();
        let id = r.register(Arc::new(Svc), props("com.acme.Nav")).unwrap();
        let mut mon = DependencyMonitor::new(&r);
        let bundle = BundleId::new("com.acme.consumer".into());
        mon.register_dependent(bundle.clone(), vec![dep("com.acme.Nav")]);
        mon.forget(&bundle);
        let mut rec = Recorder::default();
        r.unregister(id).unwrap();
        mon.drain(&mut rec);
        assert!(
            rec.lost.is_empty(),
            "forgotten bundle gets no notifications"
        );
    }
}
