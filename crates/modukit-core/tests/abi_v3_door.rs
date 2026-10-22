//! Wave-2 T1 v3-clock-door integration tests (D99, plan
//! docs/wave-2-t1-clock-door-plan.md §1): the declared-major-3 handshake
//! through the REAL install door, the two new verbs (`clock_now` ->
//! `timer_schedule` at stamp+1s) walked on the TestHost step clock, the
//! per-plugin due bell on the kernel bus (`org.modukit.timer.<plugin-id>`,
//! gen u64 LE ++ token u64 LE ++ label UTF-8 (the T2 boot-generation
//! prefix), `LOSSY_RING` (64) capacity unified with the wire vocabulary,
//! late-join replay PINNED), the
//! authority-law token release at stop/uninstall ("token never re-fires"
//! census), and the v1/v2 handoff staying byte-identical (the prefix law
//! the layout test pins; the existing abi_v2_door / dlopen suites prove the
//! handoff unchanged). Fixture build discipline mirrors tests/abi_v2_door.rs
//! (cdylibs skip-green when unbuildable).
//!
//! The exe interim (plan §1, review recast) is PINNED in
//! `executor.rs::v3_door_guard_tests::exe_interim_pin_...`: the child runner
//! loads through the same shared `validate`, so child-local v3 works and its
//! bells stay child-local until lane T2 adds the crossing. No refusal code
//! exists anywhere on the exe arm.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use modukit_core::abi::{HostAbiV1, HostAbiV2, HostAbiV3, NativeService};
use modukit_core::identity::{PluginId, TopicId};
use modukit_core::kernel::Kernel;
use modukit_core::lifecycle::PluginState;
use modukit_core::manifest::Manifest;
use modukit_core::testhost::TestHost;

const TIMERV3_ID: &str = "com.example.modukit.timerv3";
const ECHO_ID: &str = "com.example.modukit.timerv3.echo";
const OWN_TOPIC: &str = "com.example.modukit.timerv3";
/// The composed per-plugin bell topic (D99 R4-A) - grammar pinned below
/// before any wiring relied on it.
const BELL_TOPIC: &str = "org.modukit.timer.com.example.modukit.timerv3";
const JOB_LABEL: &[u8] = b"tick-v3";

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

/// Install/resolve/start a declared-major-3 fixture through the real door
/// and hand back the tracking id. Every fixture-internal assertion lands
/// as a distinct negative start code, surfacing here as PluginStartFailed.
fn start_v3_fixture(kernel: &Kernel, so: &Path) -> PluginId {
    let manifest =
        Manifest::from_toml_str(&manifest_toml(TIMERV3_ID, so)).expect("fixture manifest parses");
    let id = manifest.id.clone();
    kernel
        .install(manifest)
        .expect("v3 install passes the §2 guard");
    kernel
        .resolve(&id)
        .expect("declared major 3 is accepted at the door");
    assert_eq!(kernel.state(&id).expect("resolved"), PluginState::Starting);
    kernel
        .start(&id)
        .expect("the fixture's clock-door walk must pass every cell (start returns its code)");
    assert_eq!(kernel.state(&id).expect("started"), PluginState::Active);
    id
}

/// Decode the pinned bell payload grammar (D99 R4-A + the T2 generation
/// field, D99 §9): gen u64 LE ++ token u64 LE ++ label UTF-8. The gen is
/// this kernel's boot tag - opaque here, never asserted literally; the
/// self-equality law (same-kernel entries share it) lives in
/// `wiring.rs::bell_crossing_tests::same_kernel_entries_share_the_generation`.
fn decode_bell(payload: &[u8]) -> (u64, u64, &[u8]) {
    let (head, label) = payload.split_at(16);
    let (boot, token) = head.split_at(8);
    (
        u64::from_le_bytes(
            boot.try_into()
                .expect("payload carries the u64 LE boot gen"),
        ),
        u64::from_le_bytes(token.try_into().expect("payload carries the u64 LE token")),
        label,
    )
}

fn bell_publish_count(kernel: &Kernel) -> u64 {
    kernel
        .bus()
        .topic_stats()
        .iter()
        .find(|st| st.topic == BELL_TOPIC)
        .map_or(0, |st| st.published)
}

/// Plan §5 pin, asserted BEFORE any wiring: the TopicId reverse-DNS grammar
/// must accept the composed per-plugin timer topic. If this test is RED the
/// hearing's topic scheme does not survive the grammar - that is a finding
/// for the maintainer walk, never a reason to work around the validator.
#[test]
fn composed_timer_topic_passes_the_reverse_dns_grammar() {
    let composed = format!("org.modukit.timer.{TIMERV3_ID}");
    assert_eq!(
        composed, BELL_TOPIC,
        "the fixture topic string is the composition"
    );
    let tid = TopicId::try_new(composed.clone())
        .unwrap_or_else(|e| panic!("grammar refuses the composed timer topic {composed}: {e}"));
    assert_eq!(tid.as_str(), BELL_TOPIC);
    // The same composition for any grammar-valid plugin id stays valid:
    // every label of `org.modukit.timer.` + a reverse-DNS id is lowercase-
    // leading and inside [a-z0-9-_].
    for id in ["com.example.a", "org.modukit.x_1-2"] {
        let composed = format!("org.modukit.timer.{id}");
        assert!(
            TopicId::try_new(composed.clone()).is_ok(),
            "grammar must accept {composed}"
        );
    }
}

/// The layout law the v3 cast rides on: HostAbiV3's first seven fields are
/// the byte-identical HostAbiV2 prefix (which is itself the HostAbiV1
/// prefix), and the two timer slots ride the tail (D99 R2 - a THIRD table,
/// v1/v2 frozen). The v2 handoff is unchanged by construction: the door
/// picks the struct by declared major, so a v2 plugin never sees a v3 table
/// (plan §5 size-guard note); this test pins the prefix that stays readable.
#[test]
fn v3_table_is_the_v2_prefix_plus_two_timer_slots() {
    assert_eq!(
        HostAbiV1::expected_size(),
        std::mem::size_of::<HostAbiV1>() as u32
    );
    assert_eq!(
        HostAbiV2::expected_size(),
        std::mem::size_of::<HostAbiV2>() as u32
    );
    assert_eq!(
        HostAbiV3::expected_size(),
        std::mem::size_of::<HostAbiV3>() as u32
    );
    for (v3, v2) in [
        (
            std::mem::offset_of!(HostAbiV3, size),
            std::mem::offset_of!(HostAbiV2, size),
        ),
        (
            std::mem::offset_of!(HostAbiV3, user),
            std::mem::offset_of!(HostAbiV2, user),
        ),
        (
            std::mem::offset_of!(HostAbiV3, register_service),
            std::mem::offset_of!(HostAbiV2, register_service),
        ),
        (
            std::mem::offset_of!(HostAbiV3, bus_publish),
            std::mem::offset_of!(HostAbiV2, bus_publish),
        ),
        (
            std::mem::offset_of!(HostAbiV3, bus_subscribe),
            std::mem::offset_of!(HostAbiV2, bus_subscribe),
        ),
        (
            std::mem::offset_of!(HostAbiV3, bus_recv),
            std::mem::offset_of!(HostAbiV2, bus_recv),
        ),
        (
            std::mem::offset_of!(HostAbiV3, bus_unsubscribe),
            std::mem::offset_of!(HostAbiV2, bus_unsubscribe),
        ),
    ] {
        assert_eq!(v3, v2, "HostAbiV3 drifted off the byte-identical v2 prefix");
    }
    assert_eq!(
        std::mem::size_of::<HostAbiV3>(),
        std::mem::size_of::<HostAbiV2>() + 2 * std::mem::size_of::<usize>(),
        "two appended fn-pointer slots and nothing else"
    );
}

/// The full v3 walk through the real door on the TestHost step clock:
/// the fixture read `clock_now`, scheduled on the dialed stamp + 1s, and
/// returned; the kernel's dial then fired it and the bell rang on the
/// plugin's OWN topic with the pinned token+label payload. Registrations
/// still land through the v1 prefix slot, and the v2-prefix mailbox answered
/// on the v3 table (the prefix read the layout test pins).
#[test]
fn declared_major_3_walks_the_clock_door_and_the_bell_rings_live() {
    let Some(so) = build_fixture("timerv3-plugin", "timerv3_plugin") else {
        return;
    };
    let host = TestHost::new();
    let kernel = host.kernel();
    let id = start_v3_fixture(kernel, &so);

    // Registrations still land: the echo service came through the v1
    // prefix slot of the v3 table and answers a v1-shaped call.
    let svc = kernel
        .registry()
        .acquire::<NativeService>()
        .expect("echo service from the v1 prefix slot");
    assert_eq!(svc.service_id().as_str(), ECHO_ID);
    let echo = svc
        .invoke("echo", b"v3-door")
        .expect("v1-shaped call over the v3 table");
    assert_eq!(echo, b"v3-door");
    kernel
        .registry()
        .release(&svc)
        .expect("acquire/release pair");

    // The registration leg of the register: the scheduled job is pending
    // on the kernel clock (the fixture scheduled exactly one).
    assert_eq!(
        kernel.clock().pending_timers(),
        1,
        "one plugin timer pending"
    );

    // A live subscriber BEFORE the dial hears the bell as it is posted.
    let bell = TopicId::new(BELL_TOPIC.to_string());
    let mut rx = kernel
        .bus()
        .subscribe(&bell)
        .expect("host subscribe to the bell topic");
    let seqs = host.advance(Duration::from_secs(1));
    assert_eq!(seqs.len(), 1, "the fired job posted exactly one TimerDone");
    let payload = rx
        .try_recv()
        .expect("live bell on the plugin's own topic (the door-suite recv assertion)");
    let (_boot_gen, token, label) = decode_bell(&payload);
    assert_eq!(token, 1, "fresh step-clock token (D74 replay precedent)");
    assert_eq!(
        label, JOB_LABEL,
        "the message label rides the bell verbatim"
    );
    assert_eq!(
        bell_publish_count(kernel),
        1,
        "the bell is the ONE bus post"
    );
    assert_eq!(
        kernel.clock().pending_timers(),
        0,
        "the fired job left the pending register"
    );

    // The v2-prefix publish the fixture made is late-joinable on the SAME
    // kernel bus (no second bus; the handoff stays byte-identical).
    let own = TopicId::new(OWN_TOPIC.to_string());
    let mut rx2 = kernel
        .bus()
        .subscribe(&own)
        .expect("host subscribe to the fixture's own topic");
    assert_eq!(
        rx2.try_recv().as_deref(),
        Some(b"v3-prefix-echo".as_slice()),
        "v2 prefix slot answered on the v3 table"
    );

    kernel.stop(&id).expect("stop");
    kernel.uninstall(&id).expect("uninstall");
    assert_eq!(
        kernel.state(&id).expect("uninstalled"),
        PluginState::Uninstalled
    );
}

/// D99 R4-A PINNED behavior (not accidental): a subscriber that joins AFTER
/// a firing on the plugin's topic receives the Lossy retention ring (the
/// T2 unified `LOSSY_RING` capacity; the replay DEPTH is deliberately not
/// asserted by a literal number) -
/// the same late-join replay every year-1 bus word already does (bus.rs
/// `Lossy` arms); the hearing ACCEPTED and NAMED it for timers.
#[test]
fn late_join_after_a_firing_replays_the_bell_ring() {
    let Some(so) = build_fixture("timerv3-plugin", "timerv3_plugin") else {
        return;
    };
    let host = TestHost::new();
    let kernel = host.kernel();
    let id = start_v3_fixture(kernel, &so);

    let seqs = host.advance(Duration::from_secs(1));
    assert_eq!(seqs.len(), 1, "the firing posted its TimerDone");

    // Subscribe only now: the ring answers the year-old-for-this-subscriber
    // firing exactly as it was published.
    let bell = TopicId::new(BELL_TOPIC.to_string());
    let mut rx = kernel
        .bus()
        .subscribe(&bell)
        .expect("late-join subscribe to the bell topic");
    let payload = rx
        .try_recv()
        .expect("the Lossy ring replays at late join (pinned, D99 R4-A)");
    let (_boot_gen, token, label) = decode_bell(&payload);
    assert_eq!(token, 1);
    assert_eq!(label, JOB_LABEL);
    assert!(
        rx.try_recv().is_none(),
        "the ring held exactly the one firing"
    );

    kernel.stop(&id).expect("stop");
    kernel.uninstall(&id).expect("uninstall");
}

/// The stop census (D99 R4-A authority-law release): stop the plugin, dial
/// past the deadline - no TimerDone post, no bus publish, the pending
/// register drops at the stop door itself.
#[test]
fn stop_releases_registered_tokens_and_the_timer_never_re_fires() {
    let Some(so) = build_fixture("timerv3-plugin", "timerv3_plugin") else {
        return;
    };
    let host = TestHost::new();
    let kernel = host.kernel();
    let id = start_v3_fixture(kernel, &so);
    assert_eq!(kernel.clock().pending_timers(), 1);

    kernel.stop(&id).expect("stop");
    assert_eq!(
        kernel.clock().pending_timers(),
        0,
        "the stop release took the token out of the clock"
    );

    // Dial far past the deadline: nothing is due, nothing posts, no bell.
    let seqs = host.advance(Duration::from_secs(2));
    assert!(seqs.is_empty(), "a released token never re-fires");
    assert_eq!(bell_publish_count(kernel), 0, "no bell was ever published");
    let bell = TopicId::new(BELL_TOPIC.to_string());
    let mut rx = kernel.bus().subscribe(&bell).expect("subscribe");
    assert!(
        rx.try_recv().is_none(),
        "empty ring: the firing never happened"
    );

    kernel.uninstall(&id).expect("uninstall");
}

/// The same census through the uninstall door taken directly from ACTIVE
/// (Any -> UNINSTALLED): the register and the clock are reclaimed before
/// any further dial can reach the token.
#[test]
fn uninstall_releases_registered_tokens_and_the_timer_never_re_fires() {
    let Some(so) = build_fixture("timerv3-plugin", "timerv3_plugin") else {
        return;
    };
    let host = TestHost::new();
    let kernel = host.kernel();
    let id = start_v3_fixture(kernel, &so);
    assert_eq!(kernel.clock().pending_timers(), 1);

    kernel.uninstall(&id).expect("direct uninstall from ACTIVE");
    assert_eq!(
        kernel.clock().pending_timers(),
        0,
        "the uninstall release took the token out of the clock"
    );

    let seqs = host.advance(Duration::from_secs(2));
    assert!(seqs.is_empty(), "a released token never re-fires");
    assert_eq!(bell_publish_count(kernel), 0, "no bell was ever published");
}
