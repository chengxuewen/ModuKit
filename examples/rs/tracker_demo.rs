//! tracker_demo - the ServiceTracker topic example (bundle-management-plan
//! example-volume sweep, ruling B1; the celix track_tracker analogue).
//!
//! The tracker watches one RFC1960 filter and pumps presence events into a
//! customizer: adding / modified / removed. This demo walks the full
//! transition set against a registry it drives itself:
//!   add two matching + one non-matching -> drain -> adding x2 only
//!   modify one (same service id, new ranking) -> drain -> modified
//!   remove one -> drain -> removed; tracked() shrinks
//!   clean close: nothing double-fires (transition-only dispatch)
//! Gated by `scripts/demo-rs-examples.sh` on the final `TRACKER-DEMO PASS`.

use modukit_core::registry::{Properties, PropertyValue, Registry};
use modukit_core::tracker::{ServiceCustomizer, ServiceTracker};
use std::sync::Arc;

struct PrintCustomizer {
    adding: usize,
    modified: usize,
    removed: usize,
}

impl ServiceCustomizer for PrintCustomizer {
    fn adding(&mut self, service: &modukit_core::registry::RegisteredService) {
        self.adding += 1;
        println!(
            "  adding   {} (ranking {})",
            service.service_id.as_str(),
            service.ranking
        );
    }
    fn modified(&mut self, service: &modukit_core::registry::RegisteredService) {
        self.modified += 1;
        println!(
            "  modified {} (ranking {})",
            service.service_id.as_str(),
            service.ranking
        );
    }
    fn removed(&mut self, service: &modukit_core::registry::RegisteredService) {
        self.removed += 1;
        println!("  removed  {}", service.service_id.as_str());
    }
}

fn echo_props(tag: &str, ranking: i64) -> Properties {
    let mut p = Properties::new();
    p.insert(
        "service.id",
        PropertyValue::Str(format!("com.acme.echo.{tag}")),
    );
    p.insert("ranking", PropertyValue::Int(ranking));
    p
}

struct Svc;

fn main() {
    println!("=== ModuKit ServiceTracker demo ===");
    let registry = Registry::new();
    let mut tracker =
        ServiceTracker::open(&registry, "(service.id=com.acme.echo.*)").expect("the filter parses");
    let mut seen = PrintCustomizer {
        adding: 0,
        modified: 0,
        removed: 0,
    };

    // 1. two matching + one non-matching: only the matching pair fires.
    let a = registry
        .register(Arc::new(Svc), echo_props("one", 10))
        .unwrap();
    let b = registry
        .register(Arc::new(Svc), echo_props("two", 20))
        .unwrap();
    let mut other = Properties::new();
    other.insert("service.id", PropertyValue::Str("com.other.svc".into()));
    let _c = registry.register(Arc::new(Svc), other).unwrap();
    tracker.drain(&mut seen);
    assert_eq!(
        (seen.adding, seen.modified, seen.removed),
        (2, 0, 0),
        "adding x2"
    );
    assert_eq!(tracker.size(), 2, "the non-matching service is not tracked");

    // 2. modify one in place: modify_props emits Modified, the tracker
    //    relays it (the S2 event reaching a consumer, end to end).
    let mut updated = echo_props("one", 99);
    updated.insert("service.id", PropertyValue::Str("com.acme.echo.one".into()));
    registry.modify_props(a, updated).unwrap();
    tracker.drain(&mut seen);
    assert_eq!(seen.modified, 1, "modified x1");
    assert_eq!(
        tracker.tracked()[0].ranking,
        99,
        "the tracked view refreshed"
    );

    // 3. remove one: removed fires, the tracked view shrinks.
    registry.unregister(b).unwrap();
    tracker.drain(&mut seen);
    assert_eq!(
        (seen.adding, seen.modified, seen.removed),
        (2, 1, 1),
        "removed x1"
    );
    assert_eq!(tracker.size(), 1, "the tracked view shrank");

    // 4. re-drain with no changes: transition-only dispatch, nothing fires.
    tracker.drain(&mut seen);
    assert_eq!(
        (seen.adding, seen.modified, seen.removed),
        (2, 1, 1),
        "no double-fire"
    );

    println!("TRACKER-DEMO PASS");
}
