//! ServiceTracker (bundle-management-plan S2, ruling S2-2 A).
//!
//! A callback-driven tracker riding the registry's filtered subscription
//! (`subscribe_filtered`, S2). It keeps the current set of matching
//! registrations and dispatches a `ServiceCustomizer`'s adding / modified /
//! removed callbacks as events arrive. Deliberately bounded (module 02 §5
//! warns against the "15 years of edge-case hardening" a full OSGi tracker
//! carries): single-threaded drain of one `Receiver`, no leases, no
//! multi-thread race machinery. Cross-language thin sugar rides above this
//! core type later (D63).
//!
//! Late-join follows the documented discipline (module 02 §3 invariant 5):
//! `find(filter)` first, then `subscribe_filtered` - the same snapshot+events
//! pair the unfiltered stream uses.

use std::collections::BTreeMap;
use std::sync::mpsc::Receiver;

use crate::filter::FilterError;
use crate::registry::{Registry, ServiceEvent};
use crate::registry_types::{Properties, PropertyValue, RegisteredService};

/// The callbacks a tracker consumer implements (the OSGi "customizer" shape,
/// bounded to the three presence transitions).
pub trait ServiceCustomizer {
    /// A matching service appeared.
    fn adding(&mut self, service: &RegisteredService);
    /// A matching service's properties changed.
    fn modified(&mut self, service: &RegisteredService);
    /// A matching service went away.
    fn removed(&mut self, service: &RegisteredService);
}

/// Tracks the registrations matching one filter. Cheap to hold; `drain`
/// pumps pending events into the customizer.
pub struct ServiceTracker {
    rx: Receiver<ServiceEvent>,
    tracked: BTreeMap<i64, RegisteredService>,
}

fn ranking_of(props: &Properties) -> i64 {
    match props.get("ranking") {
        Some(PropertyValue::Int(r)) => *r,
        _ => 0,
    }
}

impl ServiceTracker {
    /// Open a tracker on `filter`: seed with the current matches, then
    /// subscribe (the documented late-join order).
    ///
    /// # Errors
    ///
    /// `FilterError` when the filter string is malformed.
    pub fn open(registry: &Registry, filter: &str) -> Result<Self, FilterError> {
        let seed = registry.find(filter)?;
        let rx = registry.subscribe_filtered(filter)?;
        let tracked = seed.into_iter().map(|s| (s.id.value(), s)).collect();
        Ok(Self { rx, tracked })
    }

    /// Pump every pending event into the customizer, updating the tracked
    /// set. Non-blocking: returns when the queue is empty.
    pub fn drain(&mut self, customizer: &mut dyn ServiceCustomizer) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                ServiceEvent::Registered {
                    id,
                    service_id,
                    props,
                } => {
                    let service = RegisteredService {
                        id,
                        service_id,
                        ranking: ranking_of(&props),
                        props,
                    };
                    self.tracked.insert(id.value(), service.clone());
                    customizer.adding(&service);
                }
                ServiceEvent::Modified {
                    id,
                    service_id,
                    props,
                } => {
                    let service = RegisteredService {
                        id,
                        service_id,
                        ranking: ranking_of(&props),
                        props,
                    };
                    self.tracked.insert(id.value(), service.clone());
                    customizer.modified(&service);
                }
                ServiceEvent::Unregistered { id, .. } => {
                    if let Some(service) = self.tracked.remove(&id.value()) {
                        customizer.removed(&service);
                    }
                }
            }
        }
    }

    /// The current tracked set (best-effort snapshot of the local map).
    #[must_use]
    pub fn tracked(&self) -> Vec<RegisteredService> {
        self.tracked.values().cloned().collect()
    }

    /// How many services are currently tracked.
    #[must_use]
    pub fn size(&self) -> usize {
        self.tracked.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ServiceId;
    use std::sync::Arc;

    struct Svc;

    #[derive(Default)]
    struct Recorder {
        added: Vec<String>,
        modified: Vec<String>,
        removed: Vec<String>,
    }

    impl ServiceCustomizer for Recorder {
        fn adding(&mut self, s: &RegisteredService) {
            self.added.push(s.service_id.as_str().to_string());
        }
        fn modified(&mut self, s: &RegisteredService) {
            self.modified.push(s.service_id.as_str().to_string());
        }
        fn removed(&mut self, s: &RegisteredService) {
            self.removed.push(s.service_id.as_str().to_string());
        }
    }

    fn props(id: &str) -> Properties {
        let mut p = Properties::new();
        p.insert("service.id", PropertyValue::Str(id.into()));
        p
    }

    #[test]
    fn tracker_seeds_then_tracks_add_modify_remove() {
        let r = Registry::new();
        let seed_id = r.register(Arc::new(Svc), props("com.acme.Nav")).unwrap();
        let mut t = ServiceTracker::open(&r, "(service.id=com.acme.Nav)").unwrap();
        assert_eq!(t.size(), 1, "late-join seed rides the tracker");
        let mut rec = Recorder::default();
        // a new matching registration -> adding
        let nav2 = r.register(Arc::new(Svc), props("com.acme.Nav")).unwrap();
        // a non-matching registration -> ignored
        r.register(Arc::new(Svc), props("com.acme.Log")).unwrap();
        t.drain(&mut rec);
        assert_eq!(rec.added, vec!["com.acme.Nav".to_string()]);
        assert_eq!(t.size(), 2);
        // modify -> modified
        r.modify_props(nav2, props("com.acme.Nav")).unwrap();
        t.drain(&mut rec);
        assert_eq!(rec.modified, vec!["com.acme.Nav".to_string()]);
        // unregister the seed -> removed
        r.unregister(seed_id).unwrap();
        t.drain(&mut rec);
        assert_eq!(rec.removed, vec!["com.acme.Nav".to_string()]);
        assert_eq!(t.size(), 1);
    }

    #[test]
    fn tracker_filter_excludes_and_malformed_redds() {
        let r = Registry::new();
        let mut t = ServiceTracker::open(&r, "(service.id=com.acme.Nav)").unwrap();
        let mut rec = Recorder::default();
        r.register(Arc::new(Svc), props("com.acme.Log")).unwrap();
        t.drain(&mut rec);
        assert!(
            rec.added.is_empty(),
            "non-matching never reaches the customizer"
        );
        assert_eq!(t.size(), 0);
        assert!(ServiceTracker::open(&r, "(service.id=").is_err());
    }

    #[test]
    fn tracker_tracked_snapshot_reflects_set() {
        let r = Registry::new();
        r.register(Arc::new(Svc), props("com.acme.Nav")).unwrap();
        let mut t = ServiceTracker::open(&r, "(service.id=*)").unwrap();
        let mut rec = Recorder::default();
        t.drain(&mut rec);
        let ids: Vec<ServiceId> = t.tracked().into_iter().map(|s| s.service_id).collect();
        assert!(ids.iter().any(|id| id.as_str() == "com.acme.Nav"));
    }
}
