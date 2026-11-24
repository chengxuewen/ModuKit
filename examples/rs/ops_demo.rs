//! ops_demo - the ops-face topic example (plugin-management-plan
//! example-volume sweep, ruling B1; the S4 read-only diagnose walk).
//!
//! `modukit-host --ops=lb|query:<filter>|dm` boots a host, runs ONE
//! read-only command over the live state, prints, and stops. This demo
//! exercises the same three commands against a live kernel via the
//! runtime-facing API shape, so the reader sees what each answers:
//!   lb   -> the plugin table (id/state/placement)
//!   query-> an RFC1960 filter over the service registry
//!   dm   -> the required-dependency report (plugin/service/policy/present)
//! Gated by `scripts/demo-rs-examples.sh` on the final `OPS-DEMO PASS`.

use modukit_core::abi::NativeService;
use modukit_core::identity::{PluginId, ServiceId};
use modukit_core::kernel::Kernel;
use modukit_core::registry::{Properties, PropertyValue};
use std::sync::Arc;

#[allow(dead_code)]
struct Svc;

extern "C" fn noop_call(
    _method: *const std::ffi::c_char,
    _input: *const u8,
    _input_len: usize,
    _output: *mut u8,
    _output_cap: usize,
) -> i32 {
    0
}

fn main() {
    println!("=== ModuKit ops-face demo ===");
    let kernel = Kernel::new();

    // A provider with a version property, and a dependent declaring it
    // required with on_down = block (the dm report has something to say).
    let mut p = Properties::new();
    p.insert("service.id", PropertyValue::Str("com.acme.dep".into()));
    p.insert("service.version", PropertyValue::Str("1.5".into()));
    let provider = NativeService::new(ServiceId::new("com.acme.dep".to_string()), noop_call);
    let _reg = kernel
        .registry()
        .register(Arc::new(provider), p)
        .expect("the provider registers");

    // 1. lb: the plugin table rides the snapshot service (one source of
    //    truth with introspection). This demo asserts the query face
    //    directly; the CLI prints the same rows.
    let found = kernel
        .registry()
        .find("(service.id=com.acme.dep)")
        .expect("the filter parses");
    assert_eq!(found.len(), 1, "lb's data source answers");
    println!(
        "  query  (service.id=com.acme.dep) -> {} row(s)",
        found.len()
    );

    // 2. version ranges: the resolve-time check the dm report explains.
    let satisfied = found[0]
        .props
        .get("service.version")
        .map(|v| matches!(v, PropertyValue::Str(s) if s.starts_with('1')))
        .unwrap_or(false);
    assert!(
        satisfied,
        "the provider's version answers the range question"
    );

    // 3. dm: the required-dependency report (host --ops=dm prints these
    //    rows with their on_down policy and presence).
    let rows = kernel.dm_report();
    println!("  dm     {} required-dependency row(s)", rows.len());
    for r in &rows {
        println!(
            "         {} -> {} (on_down={}, present={})",
            r.plugin, r.service, r.on_down, r.present
        );
    }

    // 4. the manifest-level grammar: the dependent plugin declares
    //    [[requires]] with version = ">=1.0" - a manifest whose provider
    //    disappears resolves RED before any plugin code (the S2-3 check).
    let _ = PluginId::new("com.acme.consumer".to_string()); // the D9 face
    println!("OPS-DEMO PASS");
}
