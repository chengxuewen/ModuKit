//! The testhost face (D24, modules/10 §3.1) — M6 floor 6.
//!
//! Programmatic host: kernel construction on the step-domain clock, fixture
//! services and plugin manifests installed/driven through the PRODUCTION
//! path, clock turning via `advance()` (no wall-clock sleeps anywhere),
//! snapshot/event reads, and the fold(events)->snapshot checker — the §3
//! machinery exported for plugin authors' own rigs.

use std::sync::Arc;
use std::time::Duration;

use modukit_core::clock::ClockDomain;
use modukit_core::events::ObsEvent;
use modukit_core::identity::{ServiceId, TopicId};
use modukit_core::registry::{Properties, PropertyValue};
use modukit_core::testhost::TestHost;
use modukit_core::topic::QoS;

fn svc_props(name: &str) -> Properties {
    let mut p = Properties::new();
    p.insert("service.id", PropertyValue::Str(name.to_string()));
    p
}

#[test]
fn host_boots_on_the_step_clock_with_zero_wall_time_used() {
    let host = TestHost::new();
    assert_eq!(host.kernel().clock().domain(), ClockDomain::Step);
    // The dial is the only time source: before turning it, now() is zero
    // (a wall-clock-backed host could not promise that).
    assert_eq!(
        host.kernel().clock().now().raw(),
        0,
        "step clock starts at zero"
    );
    let _ = host.advance(Duration::from_millis(40));
    assert_eq!(host.kernel().clock().now().raw(), 40_000_000);
}

#[test]
fn timers_fire_as_posted_tokens_when_the_dial_crosses_the_deadline() {
    let host = TestHost::new();
    host.kernel()
        .clock()
        .timer(modukit_core::clock::Timestamp::from(50_000_000), "job-a");
    // Nothing due yet — the ring must show no TimerDone.
    let before = host.events_since(host.head());
    assert!(
        !before
            .events
            .iter()
            .any(|r| matches!(r.event, ObsEvent::TimerDone { .. })),
        "no hidden wakeups: due only after the dial turns"
    );
    let posted_seqs = host.advance(Duration::from_millis(50));
    assert_eq!(posted_seqs.len(), 1, "advance POSTS the due job");
    let after = host.events_since(before.events.last().map_or(0, |r| r.seq));
    let posted: Vec<&ObsEvent> = after
        .events
        .iter()
        .map(|r| &r.event)
        .filter(|e| matches!(e, ObsEvent::TimerDone { .. }))
        .collect();
    assert_eq!(
        posted.len(),
        1,
        "the token was POSTED (D6), no closure called"
    );
}

#[test]
fn drives_registry_bus_and_clock_then_the_fold_checker_agrees_with_snapshot() {
    let host = TestHost::new();
    let kernel = host.kernel();
    // Ordinary host-side fixture services through the production path.
    for i in 0..5u32 {
        kernel
            .registry()
            .register(
                Arc::new(ServiceId::new(format!("com.host.fixture.{i}"))),
                svc_props(&format!("com.host.fixture.{i}")),
            )
            .expect("register fixture");
    }
    let topic = TopicId::new("com.host.tick".to_string());
    let mut rx = kernel.bus().subscribe(&topic).expect("subscribe");
    for _ in 0..3 {
        kernel
            .bus()
            .publish(&topic, QoS::LatestValue, b"tick".to_vec());
    }
    assert_eq!(
        rx.try_recv().as_deref(),
        Some(&b"tick"[..]),
        "no sleep needed: latest-value is here on arrival"
    );
    let _ = host.advance(Duration::from_secs(1));

    // The §3 machinery: fold of the whole stream must equal the live tables.
    host.assert_consistent();

    let snap = host.snapshot();
    let fixture_count = snap
        .services
        .iter()
        .filter(|s| s.service_id.as_str().starts_with("com.host.fixture."))
        .count();
    assert_eq!(fixture_count, 5, "snapshot sees every fixture");
    assert!(
        snap.topics
            .iter()
            .any(|t| t.topic == "com.host.tick" && t.published == 3),
        "bus state rides the same cut"
    );
    // Unregister two, then re-check consistency (delta + fold + snapshot).
    let ids: Vec<i64> = snap
        .services
        .iter()
        .filter(|s| s.service_id.as_str().starts_with("com.host.fixture."))
        .map(|s| s.id.value())
        .collect();
    for id in ids.iter().take(2) {
        kernel
            .registry()
            .unregister(modukit_core::registry::RegistrationId::at(*id))
            .expect("unregister fixture");
    }
    host.assert_consistent();
    assert_eq!(
        host.snapshot()
            .services
            .iter()
            .filter(|s| s.service_id.as_str().starts_with("com.host.fixture."))
            .count(),
        3,
        "tables moved with the events"
    );
}

#[test]
fn snapshot_reads_are_the_trio_service_not_a_second_door() {
    // The host's `snapshot()` and the introspect service answer identically
    // (one source of truth; D17's constitutional rule in the test rig).
    let host = TestHost::new();
    host.kernel()
        .registry()
        .register(
            Arc::new(ServiceId::new("com.host.readcheck".to_string())),
            svc_props("com.host.readcheck"),
        )
        .expect("register");
    // Acquire first: the doorway itself bumps counters, so both reads must
    // happen with the handle held or they legitimately differ (D21).
    let svc = host
        .kernel()
        .registry()
        .acquire::<modukit_core::introspect::Snapshot>()
        .expect("trio service");
    let via_service = svc.all();
    let via_host = host.snapshot();
    assert_eq!(
        via_host, via_service,
        "the face reads the trio, nothing else"
    );
    host.kernel()
        .registry()
        .release(&svc)
        .expect("pair the trio acquire");
}
