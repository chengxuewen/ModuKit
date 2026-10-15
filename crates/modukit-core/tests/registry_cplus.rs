//! Tests: C+ service registry (D4, modules/02 §1) — the six invariants.
//! Written before the implementation (C0: red observed first).
//!
//! Shapes mirrored from modules/02 §1:
//! register::<S>(Arc<S>, props) -> RegistrationId / acquire::<S>() ->
//! Option<ServiceRef<S>> / release(&ServiceRef<S>) / snapshot() / events
//! Registered{id, props} + Unregistered{id, props-at-last-known}.

use std::sync::Arc;

use modukit_core::identity::ServiceId;
use modukit_core::registry::{Properties, PropertyValue, Registry, RegistryError, ServiceEvent};

fn props_for(id: &str) -> Properties {
    let mut p = Properties::new();
    p.insert("service.id", PropertyValue::Str(id.to_string()));
    p
}

struct Counter(u32);
struct Greeter(&'static str);

impl Greeter {
    fn hello(&self) -> String {
        self.0.to_string()
    }
}

#[test]
fn register_returns_monotonic_ids_and_snapshot_shows_the_entry() {
    let reg = Registry::new();
    let first = reg
        .register(Arc::new(Counter(1)), props_for("com.test.Counter"))
        .unwrap_or_else(|e| panic!("register failed: {e}"));
    let second = reg
        .register(Arc::new(Counter(2)), props_for("com.test.Counter2"))
        .unwrap_or_else(|e| panic!("register failed: {e}"));
    // Invariant 2: monotonic i64.
    assert!(second.value() > first.value(), "ids must increase");

    let snap = reg.snapshot();
    assert_eq!(snap.len(), 2);
    let ids: Vec<_> = snap.iter().map(|r| r.id.value()).collect();
    assert!(ids.contains(&first.value()) && ids.contains(&second.value()));
}

#[test]
fn register_requires_service_id_in_properties() {
    // D9: the canonical key rides the properties map under `service.id`.
    let reg = Registry::new();
    let err = reg
        .register(Arc::new(Counter(1)), Properties::new())
        .expect_err("registration without service.id must fail");
    assert!(matches!(err, RegistryError::MissingServiceId));
}

#[test]
fn acquire_returns_handle_that_derefs_to_instance() {
    let reg = Registry::new();
    reg.register(Arc::new(Greeter("hi")), props_for("com.test.Greeter"))
        .ok();
    let r = reg
        .acquire::<Greeter>()
        .expect("acquire must find the registered instance");
    // Invariant 1: handle derefs to the instance locally.
    assert_eq!(r.hello(), "hi");
    assert_eq!(r.service_id().as_str(), "com.test.Greeter");
}

#[test]
fn acquire_without_registration_returns_none() {
    let reg = Registry::new();
    assert!(reg.acquire::<Counter>().is_none());
}

#[test]
fn release_closes_the_pair_and_rejects_unknown_refs() {
    // Invariant 6: acquire/release pairs are mandatory at call sites even
    // though the local transport keeps refcounts internally.
    let reg = Registry::new();
    let r = reg
        .register(Arc::new(Counter(7)), props_for("com.test.Counter"))
        .ok()
        .and_then(|_| reg.acquire::<Counter>())
        .expect("acquire after register");
    assert_eq!(r.0, 7, "the handle derefs to the registered instance");
    reg.release(&r)
        .unwrap_or_else(|e| panic!("release failed: {e}"));
    let err = reg
        .release(&r)
        .expect_err("second release of the same handle must not silently pass");
    assert!(matches!(err, RegistryError::AlreadyReleased { .. }));
}

#[test]
fn lookup_by_canonical_service_id_returns_metadata() {
    let reg = Registry::new();
    reg.register(Arc::new(Counter(1)), props_for("com.test.Counter"))
        .ok();
    let found = reg.lookup(&ServiceId::new("com.test.Counter".to_string()));
    assert!(found.is_some(), "lookup by canonical id must hit");
    assert!(
        reg.lookup(&ServiceId::new("com.test.Nope".to_string()))
            .is_none()
    );
}

#[test]
fn property_keys_are_case_insensitive() {
    // Invariant 3.
    let reg = Registry::new();
    let mut p = Properties::new();
    p.insert("Vendor", PropertyValue::Str("acme".to_string()));
    p.insert(
        "service.ID",
        PropertyValue::Str("com.test.Case".to_string()),
    );
    reg.register(Arc::new(Counter(1)), p).ok();
    let got = reg.lookup(&ServiceId::new("com.test.Case".to_string()));
    assert!(got.is_some(), "mixed-case service.ID must canonicalize");
    let snap = reg.snapshot();
    let props = &snap.first().unwrap().props;
    assert!(
        matches!(props.get("vendor"), Some(PropertyValue::Str(v)) if v == "acme"),
        "case-insensitive get must hit"
    );
}

#[test]
fn acquire_tie_breaks_on_ranking_reserved_field() {
    // Invariant 4: ranking is reserved storage; year-1 use is tie-break only.
    let reg = Registry::new();
    let mut low = props_for("com.test.Greeter.low");
    low.insert("ranking", PropertyValue::Int(1));
    let mut high = props_for("com.test.Greeter.high");
    high.insert("ranking", PropertyValue::Int(9));
    reg.register(Arc::new(Greeter("low")), low).ok();
    reg.register(Arc::new(Greeter("high")), high).ok();
    let r = reg.acquire::<Greeter>().expect("acquire");
    assert_eq!(r.hello(), "high", "same-type acquire picks the higher rank");
}

#[test]
fn event_stream_pairs_registered_unregistered_with_props() {
    // Invariant 5.
    let reg = Registry::new();
    let rx = reg.subscribe();
    let id = reg
        .register(Arc::new(Counter(1)), props_for("com.test.Counter"))
        .unwrap();
    match rx.recv() {
        Ok(ServiceEvent::Registered {
            id: ev_id, props, ..
        }) => {
            assert_eq!(ev_id.value(), id.value());
            assert!(matches!(
                props.get("service.id"),
                Some(PropertyValue::Str(s)) if s == "com.test.Counter"
            ));
        }
        other => panic!("expected Registered event, got {other:?}"),
    }
    reg.unregister(id).expect("unregister");
    match rx.recv() {
        Ok(ServiceEvent::Unregistered {
            id: ev_id, props, ..
        }) => {
            assert_eq!(ev_id.value(), id.value());
            // props-at-last-known ride the event (invariant 5).
            assert!(matches!(
                props.get("service.id"),
                Some(PropertyValue::Str(s)) if s == "com.test.Counter"
            ));
        }
        other => panic!("expected Unregistered event, got {other:?}"),
    }
    // Unregister of a gone registration: typed error, no panic.
    let err = reg.unregister(id).expect_err("double unregister must fail");
    assert!(matches!(err, RegistryError::UnknownRegistration { .. }));
}

#[test]
fn unregister_drops_acquire_results() {
    let reg = Registry::new();
    let id = reg
        .register(Arc::new(Counter(1)), props_for("com.test.Counter"))
        .unwrap();
    assert!(reg.acquire::<Counter>().is_some());
    reg.unregister(id).ok();
    assert!(reg.acquire::<Counter>().is_none());
    assert!(reg.snapshot().is_empty());
}
