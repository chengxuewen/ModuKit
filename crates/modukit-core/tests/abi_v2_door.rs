//! Stage-5 N1 v2-door integration tests: the declared-major-2 handshake
//! through the REAL install door (install -> resolve -> start), the four
//! mailbox verbs proven by the fixture itself, the v1-prefix echo service
//! still callable, and the ABI-liar guard still parking FAILED at major 99
//! (D96 ruling 2). Fixture build discipline mirrors tests/dlopen_kernel.rs
//! (cdylibs are not built by `cargo test`; skip-green when unbuildable).

use std::path::{Path, PathBuf};
use std::process::Command;

use modukit_core::abi::{HostAbiV1, HostAbiV2, NativeService};
use modukit_core::error::KernelError;
use modukit_core::identity::TopicId;
use modukit_core::kernel::Kernel;
use modukit_core::lifecycle::PluginState;
use modukit_core::manifest::Manifest;

const BUSV2_ID: &str = "com.example.modukit.busv2";
const ECHO_ID: &str = "com.example.modukit.busv2.echo";
const LIAR_ID: &str = "com.example.modukit.liar";
const BUSV2_TOPIC: &str = "com.example.modukit.busv2";

fn build_fixture(crate_name: &str, lib_stem: &str) -> Option<PathBuf> {
    if std::env::var_os("MODUKIT_SKIP_DLOPEN").is_some() {
        eprintln!("SKIPPING: MODUKIT_SKIP_DLOPEN is set");
        return None;
    }
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()? // crates/
        .parent()?; // repo root
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
            eprintln!("SKIPPING: fixture build failed: {crate_name} exited {s}");
            return None;
        }
        Err(e) => {
            eprintln!("SKIPPING: cannot spawn cargo for fixture build: {e}");
            return None;
        }
    }
    let so = target_dir.join("debug").join(format!("lib{lib_stem}.so"));
    if so.exists() {
        Some(so)
    } else {
        eprintln!("SKIPPING: expected artifact {} absent", so.display());
        None
    }
}

fn manifest_toml(id: &str, artifact: &Path) -> String {
    format!(
        "id = \"{id}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"dl\"\nartifact = \"{}\"\n",
        artifact.display()
    )
}

/// The layout law the v2 cast rides on: HostAbiV2's first three fields are
/// the byte-identical HostAbiV1 prefix, and the four slot pointers ride the
/// tail (D96 ruling 2 - a SECOND table, v1 frozen).
#[test]
fn v2_table_is_the_v1_prefix_plus_four_mailbox_slots() {
    assert_eq!(
        HostAbiV1::expected_size(),
        std::mem::size_of::<HostAbiV1>() as u32
    );
    assert_eq!(
        HostAbiV2::expected_size(),
        std::mem::size_of::<HostAbiV2>() as u32
    );
    assert_eq!(
        std::mem::offset_of!(HostAbiV2, size),
        std::mem::offset_of!(HostAbiV1, size)
    );
    assert_eq!(
        std::mem::offset_of!(HostAbiV2, user),
        std::mem::offset_of!(HostAbiV1, user)
    );
    assert_eq!(
        std::mem::offset_of!(HostAbiV2, register_service),
        std::mem::offset_of!(HostAbiV1, register_service)
    );
    assert_eq!(
        std::mem::size_of::<HostAbiV2>(),
        std::mem::size_of::<HostAbiV1>() + 4 * std::mem::size_of::<usize>(),
        "four appended fn-pointer slots and nothing else"
    );
}

/// The full v2 walk through the real door. The fixture's `start` IS the
/// assertion suite (self-echo, need-reporting recv, late-join snapshot,
/// unsubscribe, refusal numbers) - any failed step parks the start at its
/// distinct negative code, surfacing here as PluginStartFailed.
#[test]
fn declared_major_2_walks_the_real_door_and_the_kernel_sees_the_same_bus() {
    let Some(so) = build_fixture("busv2-plugin", "busv2_plugin") else {
        return;
    };
    let kernel = Kernel::new();
    let manifest =
        Manifest::from_toml_str(&manifest_toml(BUSV2_ID, &so)).expect("fixture manifest parses");
    let id = manifest.id.clone();

    kernel
        .install(manifest)
        .expect("v2 install passes the §2 guard");
    kernel
        .resolve(&id)
        .expect("declared major 2 is accepted at the door");
    assert_eq!(kernel.state(&id).expect("resolved"), PluginState::Starting);
    kernel
        .start(&id)
        .expect("the fixture's mailbox walk must pass every ruling (start returns its code)");
    assert_eq!(kernel.state(&id).expect("started"), PluginState::Active);

    // The v1 prefix slot on the v2 table registered a real echo service.
    let svc = kernel
        .registry()
        .acquire::<NativeService>()
        .expect("echo service from the v1 prefix slot");
    assert_eq!(svc.service_id().as_str(), ECHO_ID);
    let echo = svc
        .invoke("echo", b"v2-door")
        .expect("v1-shaped call over the v2 table");
    assert_eq!(echo, b"v2-door");
    kernel
        .registry()
        .release(&svc)
        .expect("acquire/release pair");

    // Kernel-side late-join on the SAME bus (no second bus, ruling 5): the
    // fixture's last retained publish is what a host subscriber snapshots.
    let topic = TopicId::new(BUSV2_TOPIC.to_string());
    let mut rx = kernel.bus().subscribe(&topic).expect("host subscribe");
    let value = rx.try_recv().expect("retained snapshot on the kernel bus");
    assert_eq!(value, b"late-snapshot", "wall-transparent retained-latest");

    // And the fixture's publish counted on the kernel's own bus tallies.
    let stats = kernel.bus().topic_stats();
    let seen = stats
        .iter()
        .find(|st| st.topic == BUSV2_TOPIC)
        .expect("the fixture published on the kernel's real bus");
    assert!(seen.published >= 3, "publish tally: {}", seen.published);

    kernel.stop(&id).expect("stop");
    kernel.uninstall(&id).expect("uninstall");
    assert_eq!(
        kernel.state(&id).expect("uninstalled"),
        PluginState::Uninstalled
    );
}

/// The liar guard stays RED-worthy after the door widened (D96): declaring
/// major 99 is ABI_MISMATCH at resolve and a FAILED parking, with no code
/// run - the same law bad-plugin proves at 999.
#[test]
fn declared_major_99_still_parks_failed_at_the_v2_door() {
    let Some(so) = build_fixture("liar-plugin", "liar_plugin") else {
        return;
    };
    let kernel = Kernel::new();
    let manifest =
        Manifest::from_toml_str(&manifest_toml(LIAR_ID, &so)).expect("liar manifest parses");
    let id = manifest.id.clone();
    kernel
        .install(manifest)
        .expect("install passes the §2 guard");

    let err = kernel
        .resolve(&id)
        .expect_err("ABI v99 must never be accepted silently");
    assert!(
        matches!(err, KernelError::AbiMismatch { found, .. } if found == 99),
        "expected typed AbiMismatch(found=99), got: {err}"
    );
    assert_eq!(
        kernel.state(&id).expect("still tracked"),
        PluginState::Failed,
        "unknown ABI version -> FAILED at RESOLVING (04 §3), never a partial start"
    );
    let plugin_services: Vec<_> = kernel
        .registry()
        .snapshot()
        .into_iter()
        .filter(|s| !s.service_id.as_str().starts_with("org.modukit."))
        .collect();
    assert!(
        plugin_services.is_empty(),
        "a rejected plugin must have run no code"
    );
}
