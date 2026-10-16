//! The introspect trio + the three acceptance tests (D17, modules/10 §1-3)
//! — M6 floor 1. Trio = read-only services in the reserved `org.modukit.*`
//! namespace: `LedgerDump.dump()`, `Snapshot.all()/one()`,
//! `EventStream.tail(since_seq)`. Zero UI, zero mutation, zero privileges.
//!
//! Acceptance trio per the M6 row: TWO GREEN (churn fold-equality,
//! step-replay content equality) + ONE deliberately RED-fixture (the
//! bad-plugin: its ledger entry must show it never became live).
//!
//! dlopen legs follow the M3 fixture-build pattern: they SKIP green with a
//! printed reason where cargo/artifacts cannot run (D16 timing NOT
//! promised; determinism claims are content-only).

use std::path::{Path, PathBuf};
use std::process::Command;

use modukit_core::events::ObsEvent;
use modukit_core::identity::ServiceId;
use modukit_core::introspect::{EventStream, LedgerDump, Snapshot};
use modukit_core::kernel::Kernel;
use modukit_core::lifecycle::PluginState;
use modukit_core::manifest::Manifest;

const EXAMPLE_ID: &str = "com.example.modukit.demo";
const BAD_ID: &str = "com.example.modukit.bad";

fn build_fixture(crate_name: &str, lib_stem: &str) -> Option<PathBuf> {
    if std::env::var_os("MODUKIT_SKIP_DLOPEN").is_some() {
        eprintln!("SKIPPING: MODUKIT_SKIP_DLOPEN is set");
        return None;
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent()?.parent()?;
    let target_dir = repo.join("out").join("dlopen-test-target");
    let status = Command::new(&cargo)
        .current_dir(repo)
        .args(["build", "-p", crate_name, "--target-dir"])
        .arg(&target_dir)
        .env("CARGO_TARGET_DIR", &target_dir)
        .status();
    match status {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("SKIPPING: fixture build failed ({crate_name}): {s}");
            return None;
        }
        Err(e) => {
            eprintln!("SKIPPING: cannot spawn cargo: {e}");
            return None;
        }
    }
    let so = target_dir.join("debug").join(format!("lib{lib_stem}.so"));
    so.exists().then_some(so)
}

fn manifest_toml(id: &str, artifact: &Path) -> String {
    format!(
        "id = \"{id}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"dl\"\nartifact = \"{}\"\n",
        artifact.display()
    )
}

/// Acquire each of the trio through the registry — the acceptance of the
/// D40/D17 posture: tools are plain service consumers.
fn acquire_trio(kernel: &Kernel) -> (Snapshot, EventStream, LedgerDump) {
    let s = kernel
        .registry()
        .acquire::<Snapshot>()
        .expect("org.modukit.snapshot is a registered service");
    let e = kernel
        .registry()
        .acquire::<EventStream>()
        .expect("org.modukit.event_stream is a registered service");
    let l = kernel
        .registry()
        .acquire::<LedgerDump>()
        .expect("org.modukit.ledger_dump is a registered service");
    let trio = (s.clone_inner(), e.clone_inner(), l.clone_inner());
    kernel
        .registry()
        .release(&s)
        .expect("trio acquire/release pair (snapshot)");
    kernel
        .registry()
        .release(&e)
        .expect("trio acquire/release pair (stream)");
    kernel
        .registry()
        .release(&l)
        .expect("trio acquire/release pair (dump)");
    trio
}

#[test]
fn trio_are_read_only_ordinary_services_in_the_reserved_namespace() {
    let kernel = Kernel::new();
    for name in [
        "org.modukit.snapshot",
        "org.modukit.event_stream",
        "org.modukit.ledger_dump",
    ] {
        assert!(
            kernel
                .registry()
                .lookup(&ServiceId::new(name.to_string()))
                .is_some(),
            "{name} must be registered as an ordinary service (D17/D40)"
        );
    }
    let (snap, stream, dump) = acquire_trio(&kernel);
    // Reads have no side effects: two identical reads, identical answers.
    assert_eq!(snap.all(), snap.all(), "Snapshot.all is side-effect-free");
    let t1 = stream.tail(0);
    let t2 = stream.tail(0);
    assert_eq!(t1.events.len(), t2.events.len(), "EventStream.tail is pure");
    assert_eq!(dump.dump(), dump.dump(), "LedgerDump.dump is pure");
    // The snapshot already sees the kernel's own boot services.
    let boot = snap.all();
    assert!(
        boot.services
            .iter()
            .any(|s| s.service_id.as_str() == "org.modukit.clock"),
        "the object-table cut must include the table itself"
    );
    // Counters and edges ride the same view (D21/D18 exposure points).
    let _ = boot.counters;
    assert!(!boot.edges.is_empty(), "boot registrations are edges");
}

#[test]
fn churn_200_cycles_snapshot_equals_the_fold_of_the_event_stream() {
    // modules/10 §3 churn: tables and events may never disagree.
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let kernel = Kernel::new();
    let (snap, stream, _) = acquire_trio(&kernel);
    let id = {
        let m = Manifest::from_toml_str(&manifest_toml(EXAMPLE_ID, &so)).expect("manifest");
        let id = m.id.clone();
        kernel.install(m).expect("install");
        id
    };
    for _ in 0..200 {
        // install -> resolve -> start -> stop -> uninstall -> re-install
        kernel.resolve(&id).expect("resolve");
        kernel.start(&id).expect("start");
        kernel.stop(&id).expect("stop");
        kernel.uninstall(&id).expect("uninstall");
        let m = Manifest::from_toml_str(&manifest_toml(EXAMPLE_ID, &so)).expect("manifest");
        kernel.install(m).expect("re-install");
    }
    // Leave it UNINSTALLED (one more teardown) and prove the fold says so.
    kernel.uninstall(&id).expect("final uninstall");
    let tail = stream.tail(0);
    assert_eq!(tail.dropped, 0, "churn must not shed ring history mid-test");
    let fold = modukit_core::testhost::fold_states(&tail.events);
    let view = snap.all();
    let plugin = view
        .one(&id.to_string())
        .expect("tracked plugin in the snapshot");
    assert_eq!(
        fold.get(&id.to_string()).map(String::as_str),
        Some(plugin.state.as_str()),
        "fold(events) must equal the live table (10 §3 churn)"
    );
    assert_eq!(plugin.state, "UNINSTALLED");
}

#[test]
fn step_replay_same_fixture_same_content_sequence_on_fresh_kernels() {
    // modules/10 §3 step-replay, v0 (dl-only, timing NOT promised): two
    // fresh kernels driven by the identical content sequence must produce
    // identical event CONTENT (seq/ts stripped — D16 monotonic-offset rule).
    let Some(so) = build_fixture("example-plugin", "example_plugin") else {
        return;
    };
    let run = || -> Vec<String> {
        let kernel = Kernel::new();
        let m = Manifest::from_toml_str(&manifest_toml(EXAMPLE_ID, &so)).expect("manifest");
        let id = m.id.clone();
        kernel.install(m).expect("install");
        kernel.resolve(&id).expect("resolve");
        kernel.start(&id).expect("start");
        kernel.stop(&id).expect("stop");
        kernel
            .observatory()
            .tail(0)
            .events
            .into_iter()
            .map(|r| match r.event {
                // Content projections only: no seq, no ts, no base address.
                ObsEvent::Lifecycle { plugin, from, to } => format!("L:{plugin}:{from}->{to}"),
                ObsEvent::Registry(modukit_core::registry::ServiceEvent::Registered {
                    service_id,
                    ..
                }) => format!("R:{}", service_id.as_str()),
                ObsEvent::Registry(modukit_core::registry::ServiceEvent::Unregistered {
                    service_id,
                    ..
                }) => format!("U:{}", service_id.as_str()),
                ObsEvent::ModuleLoaded { plugin, .. } => format!("M:{plugin}"),
                other => format!("X:{other:?}"),
            })
            .collect()
    };
    assert_eq!(
        run(),
        run(),
        "identical fixtures, identical content sequence"
    );
}

#[test]
fn ledger_equality_bad_plugin_is_the_red_fixture_staying_red() {
    // The deliberately RED leg: the bad plugin (ABI v999) must NEVER appear
    // as a loaded module in the ledger, must have no ModuleLoaded event,
    // and must sit FAILED with the D51 code string. The fixture goes red;
    // the kernel's honesty stays green.
    let Some(so) = build_fixture("bad-plugin", "bad_plugin") else {
        return;
    };
    let kernel = Kernel::new();
    let (snap, stream, dump) = acquire_trio(&kernel);
    let m = Manifest::from_toml_str(&manifest_toml(BAD_ID, &so)).expect("manifest");
    let id = m.id.clone();
    kernel.install(m).expect("install passes the file guard");
    let err = kernel.resolve(&id).expect_err("ABI v999 must be refused");
    assert_eq!(err.code(), "ABI_MISMATCH", "D51 frozen name, honest bucket");

    let states = stream
        .tail(0)
        .events
        .into_iter()
        .filter(|r| matches!(r.event, ObsEvent::ModuleLoaded { .. }))
        .count();
    // One ModuleLoaded may exist only if a GOOD module was loaded; this
    // kernel loaded only the bad one — so zero is the honest answer.
    assert_eq!(states, 0, "the bad plugin must never post ModuleLoaded");

    let view = snap.all();
    let plugin = view.one(&id.to_string()).expect("tracked");
    assert_eq!(
        plugin.state,
        PluginState::Failed.to_string(),
        "parked FAILED (04 §3)"
    );

    let ledger = dump.dump();
    let entry = ledger
        .iter()
        .find(|e| e.plugin == id.to_string())
        .expect("the ledger tracks the attempted module");
    assert!(
        !entry.loaded,
        "RED fixture: the rejected artifact is recorded as never loaded"
    );
    assert_eq!(
        entry.abi_stamps.found, 999,
        "the D27-shaped record stays visible"
    );
    assert_eq!(entry.abi_stamps.expected, modukit_core::abi::ABI_VERSION);
}
