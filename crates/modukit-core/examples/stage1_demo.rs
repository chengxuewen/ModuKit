//! M8 Stage-1 acceptance demo (plan §2 M8 row + §8 Definition of Done): the
//! full-chain vertical slice over production code paths (D24 — no mocks).
//! Builds the example cdylib, assembles a real bundle (D7 manifest + D28
//! `[[files]]` sha256 checklist), installs it (verify -> dedupe -> atomic
//! rename), boots the testhost step-clock kernel, walks all six states on
//! the event ring, round-trips bytes through the ABI-v1 `NativeService`
//! seam (D69: message shapes only — no raw pointer in any signature here),
//! exercises latest-value late-join on the bus, reads the D21 counters
//! view, uninstalls (data-root removal included), then lands the negative
//! scenes: tampered bundle RED before the install root is touched; the bad
//! plugin RED with ABI_MISMATCH and parked FAILED with a D27-shaped record
//! — zero segfaults, zero plugin code executed. Public API surface only.
//! `scripts/demo-stage1.sh` gates on the final `STAGE1-DEMO PASS` line.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use modukit_core::abi::{ABI_VERSION, NativeService};
use modukit_core::bundle::{BundleError, BundleInstaller};
use modukit_core::clock::ClockDomain;
use modukit_core::error::KernelError;
use modukit_core::events::{ObsEvent, Tail};
use modukit_core::identity::{PluginId, TopicId};
use modukit_core::introspect::{EventStream, LedgerDump, Snapshot};
use modukit_core::lifecycle::PluginState;
use modukit_core::manifest::Manifest;
use modukit_core::registry::{Registry, ServiceEvent};
use modukit_core::sha256;
use modukit_core::testhost::TestHost;
use modukit_core::topic::QoS;

const GOOD_ID: &str = "com.example.modukit.demo";
const BAD_ID: &str = "com.example.modukit.bad";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("demo gate: example lives under <repo>/crates/")
        .to_path_buf()
}

/// The dlopen fixture-build pattern from `tests/dlopen_kernel.rs`, minus
/// the skip escape: this binary IS the gate, an unbuilt fixture is fatal.
fn build_so(crate_name: &str, lib_stem: &str) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let target = repo_root().join("out").join("dlopen-test-target");
    let status = Command::new(&cargo)
        .current_dir(repo_root())
        .args(["build", "-p", crate_name, "--target-dir"])
        .arg(&target)
        .env("CARGO_TARGET_DIR", &target)
        .status()
        .expect("demo gate: cargo must be spawnable to build fixtures");
    if !status.success() {
        panic!("demo gate: fixture build failed ({crate_name}): {status}");
    }
    let so = target.join("debug").join(format!("lib{lib_stem}.so"));
    assert!(so.is_file(), "demo gate: artifact {} missing", so.display());
    println!("[1] fixture cdylib built: {}", so.display());
    so
}

/// Stage one bundle directory: `modukit.toml` (manifest + `[[files]]` hash
/// checklist computed by the SAME sha256 the installer verifies with) plus
/// the artifact at the bundle-relative path `plugin.so`.
fn stage_bundle(staging: &Path, name: &str, artifact_src: &Path, tamper: bool) -> PathBuf {
    let dir = staging.join(name);
    fs::create_dir_all(&dir).expect("demo gate: staging bundle dir");
    let member = dir.join("plugin.so");
    fs::copy(artifact_src, &member).expect("demo gate: copy artifact into bundle");
    let hash = sha256::to_hex(&sha256::hash_file(&member).expect("demo gate: hash member"));
    let size = fs::metadata(&member).expect("demo gate: stat member").len();
    let toml = format!(
        "id = \"{name}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"dl\"\nartifact = \"plugin.so\"\n\n[[files]]\npath = \"plugin.so\"\nsha256 = \"{hash}\"\nsize = {size}\n"
    );
    fs::write(dir.join("modukit.toml"), toml).expect("demo gate: write modukit.toml");
    if tamper {
        let mut bytes = fs::read(&member).expect("demo gate: read member to tamper");
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;
        fs::write(&member, bytes).expect("demo gate: flip one byte after the checklist");
    }
    dir
}

// rustfmt::skip on the scene helpers + main: one-line-per-step transcript
// density survives the default chain-width unwrapping that would otherwise
// blow the file past the 400-line cap.
#[rustfmt::skip]
/// Lifecycle edges of one plugin as a single `A->B->C` line (ring walk).
fn walk(tail: &Tail, plugin: &str) -> String {
    tail.events.iter().filter_map(|r| match &r.event {
        ObsEvent::Lifecycle { plugin: p, to, .. } if p == plugin => Some(to.clone()),
        _ => None,
    }).collect::<Vec<_>>().join("->")
}

#[rustfmt::skip]
fn module_loads(tail: &Tail, plugin: &str) -> usize {
    tail.events.iter().filter(|r| matches!(&r.event,
        ObsEvent::ModuleLoaded { plugin: p, .. } if p == plugin)).count()
}

#[rustfmt::skip]
/// The D17 trio acquired as ordinary services, cloned, pairs closed.
fn grab_trio(registry: &Registry) -> (Snapshot, EventStream, LedgerDump) {
    let s = registry.acquire::<Snapshot>().expect("org.modukit.snapshot");
    let e = registry.acquire::<EventStream>().expect("org.modukit.event_stream");
    let l = registry.acquire::<LedgerDump>().expect("org.modukit.ledger_dump");
    let trio = (s.clone_inner(), e.clone_inner(), l.clone_inner());
    registry.release(&s).expect("snapshot pair");
    registry.release(&e).expect("stream pair");
    registry.release(&l).expect("dump pair");
    trio
}

#[rustfmt::skip]
fn main() {
    println!("=== ModuKit Stage-1 M8 end-to-end acceptance demo ===");
    let good_so = build_so("example-plugin", "example_plugin");
    let bad_so = build_so("bad-plugin", "bad_plugin");

    // [2] D28 install protocol over a fresh host tree: staging -> verify
    // all listed hashes -> inode-dedupe >1MiB -> atomic rename (D31/D55).
    let root = repo_root().join("out").join("m8-demo");
    let _ = fs::remove_dir_all(&root);
    let installer = BundleInstaller::open(&root, None).expect("demo gate: installer open");
    let id = PluginId::new(GOOD_ID.to_string());
    stage_bundle(installer.staging_dir(), GOOD_ID, &good_so, false);
    let installed = installer.install(GOOD_ID).expect("demo gate: clean install");
    assert_eq!(installed.id, id);
    let member = installed.dir.join("plugin.so");
    let meta = fs::metadata(&member).expect("demo gate: stat installed so");
    if meta.len() > 1 << 20 {
        use std::os::unix::fs::MetadataExt;
        let hex = sha256::hash_file_hex(&member).expect("demo gate: hash installed so");
        let object = installer.objects_dir().join("sha256").join(&hex);
        let links = fs::metadata(&object).expect("demo gate: stat object").nlink();
        assert!(object.is_file() && links >= 2, "dedupe: member hardlinks the object store");
        println!("[3] install green + dedupe: nlink={links}, object {}", object.display());
    } else {
        println!("[3] install green (member <= 1MiB, plain bundle file)");
    }
    installed.verify_for_load().expect("demo gate: D31 load re-verify, clean tree");

    // [4] Kernel boot: the testhost face (D24) — production kernel on the
    // step clock; the M7 fdset-equality seam fed with the checked-in ledger.
    let host = TestHost::new();
    let kernel = host.kernel();
    assert_eq!(kernel.clock().domain(), ClockDomain::Step, "testhost step clock");
    let ledger_bytes = fs::read(repo_root().join("contracts/org.modukit.runtime.proto.fdset"))
        .expect("demo gate: checked-in fdset must be readable");
    kernel.set_contract_ledger(ledger_bytes.clone());
    let registry = kernel.registry();
    let (snap, stream, dump) = grab_trio(registry);
    println!("[4] kernel booted (step clock), introspect trio acquired");

    // [5] Walk the forward states with the INSTALLED artifact (the bundle
    // manifest already carries the installed absolute artifact path).
    let mid = kernel.install(installed.manifest.clone()).expect("demo gate: kernel install");
    let st = |p: &PluginId| kernel.state(p).expect("state");
    assert_eq!(st(&mid), PluginState::Installed);
    kernel.resolve(&mid).expect("demo gate: resolve, ABI v1 handshake");
    assert_eq!(st(&mid), PluginState::Starting);
    kernel.start(&mid).expect("demo gate: start");
    assert_eq!(st(&mid), PluginState::Active);
    let tail = stream.tail(0);
    assert_eq!(walk(&tail, GOOD_ID), "INSTALLED->RESOLVING->STARTING->ACTIVE");
    assert_eq!(module_loads(&tail, GOOD_ID), 1, "live module posts ModuleLoaded (10 §2)");
    println!("[5] INSTALLED->RESOLVING->STARTING->ACTIVE observed on the ring");

    // [6] One SERVICE CALL through the ABI-v1 NativeService seam: method
    // name + bytes in, owned bytes out (D69 shapes; D6 sync boundary).
    let payload = b"stage1 m8 vertical slice";
    let svc = registry.acquire::<NativeService>().expect("demo gate: plugin service");
    assert_eq!(svc.service_id().as_str(), GOOD_ID);
    let echoed = svc.invoke("echo", payload).expect("demo gate: call over the C ABI");
    assert_eq!(echoed, payload.to_vec(), "echo returns the same bytes");
    let bad = svc.invoke("no-such-method", b"x").expect_err("unknown method fails typed");
    assert!(matches!(bad, KernelError::ServiceCallFailed { .. }), "{bad}");
    registry.release(&svc).expect("acquire/release pair (invariant 6)");
    println!("[6] echo round-tripped {} bytes; unknown method -> {}", echoed.len(), bad.code());

    // [7] Topic bus (D10): latest-value late-join, then a live delivery.
    let topic = TopicId::try_new(format!("{GOOD_ID}.status")).expect("grammar-clean topic id");
    let bus = kernel.bus();
    bus.publish(&topic, QoS::LatestValue, b"alpha".to_vec());
    let mut rx = bus.subscribe(&topic).expect("latest-value subscribes");
    assert_eq!(rx.try_recv().expect("late join sees current value"), b"alpha".to_vec());
    bus.publish(&topic, QoS::LatestValue, b"beta".to_vec());
    assert_eq!(rx.try_recv().expect("live subscriber gets beta"), b"beta".to_vec());
    println!("[7] bus: late-join latest value + one live delivery");

    // [8] Back half of the walk + UNINSTALL: kernel teardown (the dl handle
    // releases), then the installer removes the install dir + data root.
    kernel.stop(&mid).expect("demo gate: stop");
    assert_eq!(st(&mid), PluginState::Resolved);
    kernel.uninstall(&mid).expect("demo gate: kernel uninstall");
    assert_eq!(st(&mid), PluginState::Uninstalled);
    let data_slot = installer.data_dir().join(GOOD_ID);
    fs::create_dir_all(&data_slot).expect("demo gate: fake plugin data root");
    fs::write(data_slot.join("state.db"), b"cache").expect("demo gate: write data");
    installer.uninstall(&id).expect("demo gate: bundle uninstall");
    assert!(!installer.bundles_dir().join(GOOD_ID).exists(), "install dir removed");
    assert!(!data_slot.exists(), "data root removed (D26)");
    let tail = stream.tail(0);
    assert_eq!(walk(&tail, GOOD_ID),
        "INSTALLED->RESOLVING->STARTING->ACTIVE->STOPPING->RESOLVED->UNINSTALLED");
    assert!(tail.events.iter().any(|r| matches!(&r.event,
        ObsEvent::Registry(ServiceEvent::Unregistered { service_id, .. })
            if service_id.as_str() == GOOD_ID)), "service retired with it");
    println!("[8] ACTIVE->STOPPING->RESOLVED->UNINSTALLED on the ring; dirs removed");

    // [9] Counters view (D21): the instrumentation must have seen the
    // demo — every touched choke point reads non-zero.
    let c = kernel.counters_view();
    assert!(c.lifecycle_transitions > 0 && c.load_ok > 0, "mv + load doors counted");
    assert!(c.registry_acquires > 0 && c.registry_releases > 0, "acquire/release doors counted");
    assert!(c.bus_published > 0 && c.bus_delivered > 0, "publish fan-out doors counted");
    println!("[9] counters: {} transitions, {} ok loads", c.lifecycle_transitions, c.load_ok);
    println!("[9] counters: {}/{} acq/rel, bus pub/del {}/{}", c.registry_acquires,
        c.registry_releases, c.bus_published, c.bus_delivered);

    // [10] Negative A: tampered bundle. The checklist goes RED with a typed
    // BundleError BEFORE the install root is touched (D28 step order).
    stage_bundle(installer.staging_dir(), GOOD_ID, &good_so, true);
    let err = installer.install(GOOD_ID).expect_err("demo gate: tampered bundle RED");
    assert!(matches!(err, BundleError::HashMismatch { .. }), "{err}");
    assert_eq!(err.code(), "CONTRACT_VIOLATION", "integrity rides the frozen D51 bucket");
    assert!(!installer.bundles_dir().join(GOOD_ID).exists(), "install root untouched");
    assert!(installer.staging_dir().join(GOOD_ID).is_dir(), "staged bundle kept for the operator");
    println!("[10] negative A RED as expected: {err} (code {} domain=contract source=bundle-installer)", err.code());

    // [11] Negative B through the bundle door: the D7 cross-check loads the
    // staged descriptor; the version guard refuses with ABI_MISMATCH.
    stage_bundle(installer.staging_dir(), BAD_ID, &bad_so, false);
    let err = installer.install(BAD_ID).expect_err("demo gate: bad bundle RED");
    assert_eq!(err.code(), "ABI_MISMATCH", "the handshake code flows through the bundle door");
    assert!(!installer.bundles_dir().join(BAD_ID).exists(), "install root untouched");
    println!("[11] negative B (bundle door) RED: {err} (code {} domain=contract source=d7-cross-check)", err.code());

    // [12] Negative B through the kernel door: resolve lands FAILED with a
    // D27-shaped record; nothing registers, no ModuleLoaded, no plugin code
    // runs — and the process never segfaults.
    let bad_staged = installer.staging_dir().join(BAD_ID).join("plugin.so");
    let bad_manifest = Manifest::from_toml_str(&format!(
        "id = \"{BAD_ID}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"dl\"\nartifact = \"{}\"\n",
        bad_staged.display()
    )).expect("demo gate: bad manifest parses");
    let bad_id = kernel.install(bad_manifest).expect("demo gate: §2 guard sees the file");
    let err = kernel.resolve(&bad_id).expect_err("demo gate: ABI v999 refused");
    assert!(matches!(err, KernelError::AbiMismatch { found, .. } if found == 999), "{err}");
    assert_eq!(err.code(), "ABI_MISMATCH");
    assert_eq!(st(&bad_id), PluginState::Failed, "setup error parks FAILED (04 §3)");
    let tail = stream.tail(0);
    assert_eq!(walk(&tail, BAD_ID), "INSTALLED->RESOLVING->FAILED");
    assert_eq!(module_loads(&tail, BAD_ID), 0, "a rejected module never posts ModuleLoaded");
    assert!(snap.all().services.iter().all(|s| s.service_id.as_str() != BAD_ID),
        "a rejected plugin runs no code and registers nothing");
    let entry = dump.dump().into_iter().find(|e| e.plugin == BAD_ID)
        .expect("demo gate: ledger tracks the attempt");
    assert!(!entry.loaded, "the RED fixture stays red in the ledger");
    assert_eq!((entry.abi_stamps.expected, entry.abi_stamps.found), (ABI_VERSION, 999));
    assert!(kernel.counters_view().load_fail > 0, "the refused handshake is counted");
    println!("[12] FAILED + D27 record: code={} domain=contract source=dl-handshake hint={:?}", err.code(), err.hint());
    println!("[12] message: {err} | ledger loaded=false, stamps={}/{}", entry.abi_stamps.expected, entry.abi_stamps.found);

    // [13] Honesty closes: fold(events) == live tables (D24 checker) and
    // the contract ledger reads back byte-equal (M7/M8 fdset seam).
    host.assert_consistent();
    assert_eq!(dump.contract_ledger(), ledger_bytes, "fdset reads back byte-equal");
    println!("[13] fold-checker green; fdset equality green");

    println!("STAGE1-DEMO PASS");
    println!("(scratch host tree kept for inspection: {})", root.display());
}
