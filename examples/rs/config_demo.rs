//! config_demo - the ConfigAdmin topic example (plugin-management-plan
//! example-volume sweep, ruling B1; the hot-lane + persistence walk).
//!
//! The D29 contract: only keys a plugin DECLARED in its `config_live` are
//! hot-applyable; accepted keys persist across restarts and publish on
//! `config/<plugin-id>` (LatestValue - a late joiner reads the current
//! value). This demo walks the whole grammar against a temp territory:
//!   declare [interval] -> set interval OK + a subscriber received it
//!   set an UNDECLARED key -> typed NotLive refusal (the liar enforcement)
//!   a NEW admin over the same persistence dir reads the value back
//! Gated by `scripts/demo-rs-examples.sh` on the final `CONFIG-DEMO PASS`.

use modukit_core::bus::Bus;
use modukit_core::config::ConfigAdmin;
use modukit_core::identity::{BundleId, TopicId};
use std::sync::Arc;

fn plugin(name: &str) -> BundleId {
    BundleId::new(format!("com.acme.{name}"))
}

fn main() {
    println!("=== ModuKit ConfigAdmin demo ===");
    let dir = std::env::temp_dir().join("modukit-config-demo");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the demo territory creates");

    let bus = Arc::new(Bus::new());
    let sensor = plugin("sensor");
    let admin = ConfigAdmin::new()
        .with_bus(Arc::clone(&bus))
        .with_persistence(dir.clone());

    // 1. declare: the manifest's config_live = ["interval"] equivalent.
    admin.declare(sensor.clone(), vec!["interval".into()]);

    // 2. a late subscriber to the hot topic (LatestValue: it will receive
    //    the current value the moment it subscribes - after the set below).
    // the hot-lane topic rides the unpublish-validated constructor
    // (config.rs parity: the bus publish path is deliberately unvalidated)
    let topic = TopicId::new(format!("config/{}", sensor.as_str()));
    let mut rx = bus.subscribe(&topic).expect("the topic subscribes");

    // 3. set a declared key: stored, persisted, published.
    admin
        .set(&sensor, "interval", "250ms")
        .expect("a declared key hot-applies");
    assert_eq!(admin.get(&sensor, "interval").as_deref(), Some("250ms"));

    // 4. an undeclared key is refused OUTRIGHT - stored nowhere, published
    //    nowhere. This is the liar-family enforcement point.
    match admin.set(&sensor, "threshold", "high") {
        Err(modukit_core::config::ConfigError::NotLive { key, .. }) => {
            println!("  NotLive refusal for {key:?} (the undeclared key)");
        }
        other => panic!("an undeclared key must be refused, got {other:?}"),
    }
    assert!(
        admin.get(&sensor, "threshold").is_none(),
        "the refusal stored nothing"
    );

    // 5. the hot topic carried the accepted value (LatestValue discipline).
    //    The payload is the line codec: key TAB value NEWLINE
    //    (config.rs publishes format!("{key}\t{value}\n")).
    let v = rx.try_recv().expect("the hot lane delivered");
    assert_eq!(v, b"interval\t250ms\n", "the line-codec payload matches");

    // 6. persistence: a NEW admin over the same dir reads the value back -
    //    the deploy source of truth outlives one process.
    let reloaded = ConfigAdmin::new().with_persistence(dir.clone());
    assert_eq!(
        reloaded.get(&sensor, "interval").as_deref(),
        Some("250ms"),
        "the value survives a restart via build/config-live"
    );

    let _ = std::fs::remove_dir_all(&dir);
    println!("CONFIG-DEMO PASS");
}
