//! The live edge ledger (D18, modules/10 §6) — M6 floor 2b: the kernel's
//! one append-only record of lifecycle/registry edges — who registered,
//! unregistered and transitioned, when, with the floor-4 clock stamp.
//! Ring-capped in memory; long history belongs to recordings, never RAM
//! (D16). Queryable snapshot feeds `Snapshot.edges()`.

use std::sync::Arc;

use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::edge_ledger::{EdgeKind, EdgeLedger};
use modukit_core::events::Observatory;
use modukit_core::identity::ServiceId;
use modukit_core::kernel::Kernel;
use modukit_core::registry::{Properties, PropertyValue};

fn svc_props(name: &str) -> Properties {
    let mut p = Properties::new();
    p.insert("service.id", PropertyValue::Str(name.to_string()));
    p
}

#[test]
fn registry_edges_record_who_came_and_went_with_a_clock_stamp() {
    let kernel = Kernel::with_clock(Clock::new(ClockDomain::Step));
    kernel.clock().advance(std::time::Duration::from_millis(3));
    let rid = kernel
        .registry()
        .register(
            Arc::new(ServiceId::new("com.test.edge.svc".to_string())),
            svc_props("com.test.edge.svc"),
        )
        .expect("register");
    kernel.clock().advance(std::time::Duration::from_millis(4));
    kernel.registry().unregister(rid).expect("unregister");

    let edges = kernel.edge_ledger().since_seq(0);
    let kinds: Vec<(EdgeKind, String, u64)> = edges
        .iter()
        .map(|e| (e.kind, e.subject.clone(), e.ts.raw()))
        .collect();
    let registered = kinds
        .iter()
        .filter(|(k, s, _)| *k == EdgeKind::Registered && s == "com.test.edge.svc")
        .count();
    let unregistered = kinds
        .iter()
        .any(|(k, s, _)| *k == EdgeKind::Unregistered && s == "com.test.edge.svc");
    assert_eq!(registered, 1, "exactly one Registered edge: {kinds:?}");
    assert!(unregistered, "the provider's retirement is an edge too");
    // Stamps ride the dial: the register happened at ts=3ms, unregister at 7ms.
    let reg = kinds
        .iter()
        .find(|(k, s, _)| *k == EdgeKind::Registered && s == "com.test.edge.svc")
        .expect("registered edge");
    assert_eq!(reg.2, 3_000_000);
}

#[test]
fn lifecycle_transitions_are_edges_at_the_same_doorway() {
    let dir = std::env::temp_dir().join(format!("m6-edge-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("temp dir");
    let artifact = dir.join("placeholder.so");
    std::fs::write(&artifact, b"placeholder").expect("artifact");
    let kernel = Kernel::new();
    let manifest = modukit_core::manifest::Manifest::from_toml_str(&format!(
        "id = \"com.test.edge\"\nversion = \"0.1.0\"\n\
         [entry]\nkind = \"dl\"\nartifact = \"{}\"\n",
        artifact.display()
    ))
    .expect("manifest parses");
    let id = manifest.id.clone();
    kernel.install(manifest).expect("install");
    kernel.uninstall(&id).expect("uninstall");
    std::fs::remove_dir_all(&dir).ok();

    let trans: Vec<String> = kernel
        .edge_ledger()
        .since_seq(0)
        .iter()
        .filter(|e| e.kind == EdgeKind::Transition)
        .map(|e| format!("{}:{}", e.subject, e.detail))
        .collect();
    assert!(
        trans.contains(&"com.test.edge:UNINSTALLED->INSTALLED".to_string()),
        "install edge missing: {trans:?}"
    );
    assert!(
        trans.contains(&"com.test.edge:INSTALLED->UNINSTALLED".to_string()),
        "uninstall edge missing: {trans:?}"
    );
}

#[test]
fn ledger_is_ring_capped_and_counts_its_evictions() {
    let clock = Clock::new(ClockDomain::Step);
    let obs = Arc::new(Observatory::new(clock));
    let ledger = Arc::new(EdgeLedger::new());
    obs.attach_edges(Arc::clone(&ledger));
    for i in 0..(modukit_core::edge_ledger::EDGE_CAP + 300) {
        obs.emit(modukit_core::events::ObsEvent::Registry(
            modukit_core::registry::ServiceEvent::Registered {
                id: modukit_core::registry::RegistrationId::at(i as i64),
                service_id: ServiceId::new(format!("com.test.flood.{i}")),
                props: Properties::new(),
            },
        ));
    }
    assert_eq!(ledger.retained(), modukit_core::edge_ledger::EDGE_CAP);
    assert!(ledger.drops() >= 300, "retention shedding stays visible");
    let oldest = ledger.since_seq(0).first().expect("retained").clone();
    assert_eq!(oldest.seq, 301, "drop-oldest: the first 300 edges are gone");
}
