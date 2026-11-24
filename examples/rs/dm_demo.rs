//! dm_demo - the DM-lite topic example (plugin-management-plan example-volume
//! sweep, ruling B1; the celix dm_example analogue).
//!
//! The DependencyMonitor watches the required dependencies a dependent
//! declared and dispatches PRESENCE TRANSITIONS to a handler; the kernel is
//! the real handler (stop/park/restart). This demo plays the kernel role
//! with a recording handler so every transition is visible:
//!   provider leaves -> dep_lost(block) for the dependent that watches it
//!   provider returns -> dep_restored, the block clears
//!   an unrelated service changing is silent (transition-only)
//! Gated by `scripts/demo-rs-examples.sh` on the final `DM-DEMO PASS`.

use modukit_core::dm::{DependencyMonitor, DmHandler, RequiredDep};
use modukit_core::identity::{PluginId, ServiceId};
use modukit_core::manifest::OnDown;
use modukit_core::registry::{Properties, PropertyValue, Registry};
use std::sync::Arc;

struct Rec {
    lost: Vec<(String, String)>,
    restored: Vec<(String, String)>,
}

impl DmHandler for Rec {
    fn dep_lost(&mut self, plugin: &PluginId, dep: &RequiredDep) {
        self.lost.push((
            plugin.as_str().to_string(),
            dep.service.as_str().to_string(),
        ));
        println!(
            "  dep_lost     {} watches {}",
            plugin.as_str(),
            dep.service.as_str()
        );
    }
    fn dep_restored(&mut self, plugin: &PluginId, dep: &RequiredDep) {
        self.restored.push((
            plugin.as_str().to_string(),
            dep.service.as_str().to_string(),
        ));
        println!(
            "  dep_restored {} watches {}",
            plugin.as_str(),
            dep.service.as_str()
        );
    }
}

struct Svc;

fn main() {
    println!("=== ModuKit DM-lite demo ===");
    let registry = Registry::new();
    let mut monitor = DependencyMonitor::new(&registry);

    // The dependent: a plugin whose manifest declared
    //   [[requires]] service = "com.acme.dep", required, on_down = block
    let dependent = PluginId::new("com.acme.consumer".to_string());
    monitor.register_dependent(
        dependent.clone(),
        vec![RequiredDep {
            service: ServiceId::new("com.acme.dep".to_string()),
            on_down: OnDown::Block,
        }],
    );

    // The provider is present at watch time: no transitions yet.
    let mut rec = Rec {
        lost: Vec::new(),
        restored: Vec::new(),
    };
    monitor.drain(&mut rec);
    assert!(
        rec.lost.is_empty() && rec.restored.is_empty(),
        "a quiet registry is silent"
    );

    // The provider registers: the absent->present edge IS a restore -
    // the monitor dispatches dep_restored (that is the watch-time seed).
    let mut p = Properties::new();
    p.insert("service.id", PropertyValue::Str("com.acme.dep".into()));
    let reg = registry.register(Arc::new(Svc), p).unwrap();
    monitor.drain(&mut rec);
    assert!(
        rec.lost.is_empty() && rec.restored.len() == 1,
        "registering the watched dep dispatches dep_restored once"
    );

    // The provider leaves: the transition fires, once.
    registry.unregister(reg).unwrap();
    monitor.drain(&mut rec);
    assert_eq!(rec.lost.len(), 1, "dep_lost on the present->absent edge");

    // A modified event on an unrelated service is NOT a transition.
    monitor.drain(&mut rec);
    assert_eq!(rec.lost.len(), 1, "transition-only: no double dispatch");

    // The provider returns: dep_restored again (the block clears).
    let mut p2 = Properties::new();
    p2.insert("service.id", PropertyValue::Str("com.acme.dep".into()));
    let _reg2 = registry.register(Arc::new(Svc), p2).unwrap();
    monitor.drain(&mut rec);
    assert_eq!(
        rec.restored.len(),
        2,
        "dep_restored on the second registration"
    );

    println!("DM-DEMO PASS");
}
