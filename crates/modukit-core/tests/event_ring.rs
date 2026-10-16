//! Shared observatory ring + merged event stream (D17, modules/10 §1) —
//! M6 floor 2a. One monotonic ring carries lifecycle transitions and the
//! registry event pairs; `tail(since_seq)` is resumable; the ring is bounded
//! with visible drop counts (same discipline as the M5 bus: observation,
//! never blocking the producer).

use std::time::Duration;

use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::events::{ObsEvent, Observatory};
use modukit_core::identity::ServiceId;
use modukit_core::kernel::Kernel;
use modukit_core::registry::{Properties, PropertyValue};

fn svc_props(name: &str) -> Properties {
    let mut p = Properties::new();
    p.insert("service.id", PropertyValue::Str(name.to_string()));
    p
}

#[test]
fn registry_events_land_on_the_merged_ring_with_seq_and_stamp() {
    let kernel = Kernel::new();
    let obs = kernel.observatory();
    for i in 0..10u32 {
        kernel
            .registry()
            .register(
                std::sync::Arc::new(ServiceId::new(format!("com.test.echo.{i}"))),
                svc_props(&format!("com.test.echo.{i}")),
            )
            .expect("register");
    }
    let tail = obs.tail(0);
    let registry_events: Vec<&ObsEvent> = tail
        .events
        .iter()
        .map(|r| &r.event)
        .filter(|e| matches!(e, ObsEvent::Registry(_)))
        .collect();
    // 5 boot services (clock + introspect trio + logger) + the 10 above.
    assert!(
        registry_events.len() >= 10,
        "every Registered must be on the ring, got {}",
        registry_events.len()
    );
    let mut prev_seq = 0;
    for rec in &tail.events {
        assert!(
            rec.seq > prev_seq,
            "seq is monotonic: {} after {prev_seq}",
            rec.seq
        );
        prev_seq = rec.seq;
    }
    // Resumable: tail(from last) is empty; tail(one back) yields exactly it.
    let last = tail.events.last().expect("ring non-empty").seq;
    assert!(
        obs.tail(last).events.is_empty(),
        "resume past the end is empty"
    );
    assert_eq!(
        obs.tail(last - 1).events.len(),
        1,
        "resume from the gap works"
    );
}

#[test]
fn lifecycle_transitions_are_emitted_at_the_same_doorways() {
    // A temp-file artifact passes the install guard without any dlopen:
    // the transitions themselves are what this test puts on the ring.
    let dir = std::env::temp_dir().join(format!("m6-ring-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let artifact = dir.join("notloaded.so");
    std::fs::write(&artifact, b"placeholder").expect("artifact file");

    let kernel = Kernel::new();
    let manifest = modukit_core::manifest::Manifest::from_toml_str(&format!(
        "id = \"com.test.ring\"\nversion = \"0.1.0\"\n\
         [entry]\nkind = \"dl\"\nartifact = \"{}\"\n",
        artifact.display()
    ))
    .expect("manifest parses");
    let id = manifest.id.clone();
    kernel.install(manifest).expect("install");
    kernel.uninstall(&id).expect("uninstall");
    std::fs::remove_dir_all(&dir).ok();

    let states: Vec<(String, String, String)> = kernel
        .observatory()
        .tail(0)
        .events
        .iter()
        .filter_map(|r| match &r.event {
            ObsEvent::Lifecycle { plugin, from, to } => {
                Some((plugin.clone(), from.clone(), to.clone()))
            }
            _ => None,
        })
        .collect();
    assert!(
        states.contains(&(
            "com.test.ring".to_string(),
            "UNINSTALLED".into(),
            "INSTALLED".into()
        )),
        "install transition missing: {states:?}"
    );
    assert!(
        states.contains(&(
            "com.test.ring".to_string(),
            "INSTALLED".into(),
            "UNINSTALLED".into()
        )),
        "uninstall transition missing: {states:?}"
    );
}

#[test]
fn bounded_ring_drops_oldest_and_counts_them_loudly() {
    let clock = Clock::new(ClockDomain::Step);
    let obs = Observatory::new(clock);
    // Ring cap is a kernel constant; fill past it and see the shed.
    for i in 0..(modukit_core::events::RING_CAP + 100) {
        obs.emit(ObsEvent::Marker(format!("m{i}")));
    }
    assert_eq!(obs.retained(), modukit_core::events::RING_CAP);
    let tail = obs.tail(0);
    assert_eq!(tail.dropped, 100, "every eviction is visible, never silent");
    assert_eq!(tail.first_seq, 101, "genesis is gone; the gap is reported");
    assert_eq!(tail.events.len(), modukit_core::events::RING_CAP);
    assert_eq!(tail.events.first().expect("first").seq, 101);
}

#[test]
fn event_stamps_follow_the_clock_dial_step_domain_only_moves_when_turned() {
    let kernel = Kernel::with_clock(Clock::new(ClockDomain::Step));
    let obs = kernel.observatory();
    let base = obs.last_seq(); // events so far are boot services; skip them
    kernel.clock().advance(Duration::from_millis(5));
    kernel
        .registry()
        .register(
            std::sync::Arc::new(ServiceId::new("com.test.stamp".to_string())),
            svc_props("com.test.stamp"),
        )
        .expect("register");
    let rec = obs
        .tail(base)
        .events
        .into_iter()
        .find(|r| matches!(r.event, ObsEvent::Registry(_)))
        .expect("the Registered event");
    assert_eq!(
        rec.ts,
        modukit_core::clock::Timestamp::from(5_000_000),
        "the stamp comes from the unified clock, not the wall"
    );
}
