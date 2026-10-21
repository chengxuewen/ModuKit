//! Stage-5 N2 wall-transparency suite (D96 ruling 6): the SAME mailbox walk
//! the in-process N1 fixture runs (`tests/abi_v2_door.rs` + the busv2-plugin
//! fixture) driven through the REAL install door, Supervisor, shadow-runner
//! and hosted child - wall-out; the two columns must reach identical
//! outcomes: self-echo, need-report, non-consuming probe, late-join
//! snapshot, unsubscribe -> NOT_FOUND, deferred-QoS refusal, grammar
//! refusals. The negative column is the capability law (ruling 4): a runner
//! that does not declare the HELLO capability never speaks the bus talkers,
//! so the parent's bus stays at zero - asserted by counter, not by prose.
//!
//! Harness discipline is the dual_placement/embed_sup pattern verbatim
//! (fixtures are not built by `cargo test`; a runner build failure is FATAL,
//! a fixture build failure skips cleanly).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, PoisonError};

use modukit_core::executor::Supervisor;
use modukit_core::identity::TopicId;
use modukit_core::lifecycle::PluginState;
use modukit_core::manifest::Manifest;
use modukit_core::proc_wire::{
    self, Delivery, Forward, HELLO_CAP_BUS_V2, Hello, MSG_DELIVER, MSG_FORWARD,
};

/// The busv2 fixture's own self-report (its descriptor id/version - the exe
/// arm cross-checks HELLO against the manifest strictly).
const ID: &str = "com.example.modukit.busv2.echo";
const VERSION: &str = "0.1.0";
/// The fixture's walk topic (same constant the in-process column asserts).
const BUSV2_TOPIC: &str = "com.example.modukit.busv2";

/// The supervisor ambient is per-instance here, but the capability flag is
/// process env (the runner reads it at spawn) - serialize the two tests that
/// spawn children so the flag cannot leak across them.
static SERIAL: Mutex<()> = Mutex::new(());

fn serial() -> MutexGuard<'static, ()> {
    SERIAL.lock().unwrap_or_else(PoisonError::into_inner)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace layout: crates/modukit-core")
        .to_path_buf()
}

/// Dedicated target dir (the embed_sup pattern): the cross suite never races
/// the other test binaries' shared dirs for the same artifacts.
fn target_dir() -> PathBuf {
    repo_root().join("out").join("abi-v2-cross-target")
}

fn cargo_build(args: &[&str]) -> Option<bool> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let dir = target_dir();
    let status = Command::new(&cargo)
        .current_dir(repo_root())
        .args(args)
        .arg("--target-dir")
        .arg(&dir)
        .env("CARGO_TARGET_DIR", &dir)
        .status();
    match status {
        Ok(s) => Some(s.success()),
        Err(e) => {
            eprintln!("SKIPPING: cannot spawn cargo: {e}");
            None
        }
    }
}

fn build_busv2_so() -> Option<PathBuf> {
    if std::env::var_os("MODUKIT_SKIP_DLOPEN").is_some() {
        eprintln!("SKIPPING: MODUKIT_SKIP_DLOPEN is set");
        return None;
    }
    match cargo_build(&["build", "-p", "busv2-plugin"]) {
        Some(true) => {}
        Some(false) => {
            eprintln!("SKIPPING: fixture build failed");
            return None;
        }
        None => return None,
    }
    let so = target_dir().join("debug").join("libbusv2_plugin.so");
    if so.is_file() {
        Some(so)
    } else {
        eprintln!("SKIPPING: expected artifact {} absent", so.display());
        None
    }
}

fn build_runner() -> PathBuf {
    let bin = target_dir().join("debug").join("modukit-run");
    match cargo_build(&["build", "-p", "modukit-runtime"]) {
        Some(true) => {}
        Some(false) => panic!("modukit-runtime must build (workspace member)"),
        None => panic!("cargo must be spawnable to build the runner"),
    }
    assert!(bin.is_file(), "runner missing at {}", bin.display());
    bin
}

fn manifest(so: &Path, runner: &Path) -> Manifest {
    Manifest::from_toml_str(&format!(
        "id = \"{ID}\"\nversion = \"{VERSION}\"\n[entry]\nkind = \"exe\"\n\
         artifact = \"{}\"\nrunner = \"{}\"\n",
        so.display(),
        runner.display()
    ))
    .expect("cross-suite manifest parses")
}

/// The wall-out column: the SAME fixture walk, now hosted inside the real
/// runner. `start` returning Ok IS the assertion suite passing across the
/// wall (any mailbox-law disagreement parks a distinct negative code there).
#[test]
fn wall_out_runs_the_same_walk_and_the_parent_bus_sees_the_same_topic() {
    let _serial = serial();
    let Some(so) = build_busv2_so() else { return };
    let runner = build_runner();
    let sup = Supervisor::new();
    let m = manifest(&so, &runner);
    let id = m.id.clone();
    // Pin the capability flag ON explicitly (not by default): the two spawn
    // tests share process env under the serial guard, so order-independence
    // is the guarantee. SAFETY: same single-writer env discipline as the
    // flag-off column.
    unsafe { std::env::set_var("MODUKIT_BUS", "on") };
    sup.kernel().install(m).expect("exe install door");
    let resolved = sup.kernel().resolve(&id);
    unsafe { std::env::remove_var("MODUKIT_BUS") };
    resolved.expect("spawn + HELLO");
    sup.kernel()
        .start(&id)
        .expect("the fixture's mailbox walk must hold across the wall (code names the step)");
    assert_eq!(
        sup.kernel().state(&id).expect("tracked"),
        PluginState::Active
    );

    // Ruling 6 host-side: the parent's OWN bus carries the child's publishes
    // (same fan-out doorway), and a host late-join sees the retained value -
    // the exact assertions the in-process door suite makes.
    let topic = TopicId::new(BUSV2_TOPIC.to_string());
    let mut rx = sup
        .kernel()
        .bus()
        .subscribe(&topic)
        .expect("host subscribe on the parent bus");
    let value = rx.try_recv().expect("retained snapshot across the wall");
    assert_eq!(value, b"late-snapshot", "wall-transparent retained-latest");

    let stats = sup.kernel().bus().topic_stats();
    let seen = stats
        .iter()
        .find(|st| st.topic == BUSV2_TOPIC)
        .expect("the child's forwards reached the parent's real bus");
    assert!(seen.published >= 3, "publish tally: {}", seen.published);

    assert_eq!(
        sup.kernel().counters_view().process_crashes,
        0,
        "the crossing must not cost a crash"
    );
    sup.kernel().stop(&id).expect("cooperative stop");
    assert_eq!(
        sup.kernel().state(&id).expect("tracked"),
        PluginState::Resolved
    );
    sup.kernel().uninstall(&id).expect("uninstall");
}

/// The capability negative (ruling 4): the runner with the flag off declares
/// no capability, hosts the same fixture locally (the N1 walk still passes in
/// the child), and the parent bus sees NOTHING - silence by counter.
#[test]
fn flag_off_runner_is_never_spoken_to_and_leaves_the_parent_bus_at_zero() {
    let _serial = serial();
    let Some(so) = build_busv2_so() else { return };
    let runner = build_runner();
    let sup = Supervisor::new();
    let m = manifest(&so, &runner);
    let id = m.id.clone();

    // The child is exec'd during `resolve` (ProcessExecutor::load), so the
    // capability flag must already be in the environment when the runner
    // image starts - setting it only around `start` would be one doorway
    // too late. SAFETY: the serial guard holds this binary's env gate; no
    // other thread in this test process reads the runner's flag.
    unsafe { std::env::set_var("MODUKIT_BUS", "off") };
    sup.kernel().install(m).expect("exe install door");
    sup.kernel().resolve(&id).expect("spawn + HELLO");
    let started = sup.kernel().start(&id);
    // SAFETY: paired with the `set_var` above under the same serial guard.
    unsafe { std::env::remove_var("MODUKIT_BUS") };
    started.expect("the fixture still walks its own local bus (N1 grammar)");
    assert_eq!(
        sup.kernel().state(&id).expect("tracked"),
        PluginState::Active
    );

    // No FORWARD ever crossed: the parent's bus never saw a publish, and the
    // walk topic does not exist on it at all (the in-process column's clone
    // lived only inside the child).
    let stats = sup.kernel().bus().topic_stats();
    assert!(
        stats.iter().all(|st| st.topic != BUSV2_TOPIC),
        "an uncapable runner must never be heard on the parent bus: {stats:?}"
    );
    assert_eq!(
        sup.kernel()
            .bus()
            .topic_stats()
            .iter()
            .map(|st| st.published)
            .sum::<u64>(),
        0,
        "parent bus publish counter stays at zero"
    );
    assert_eq!(
        sup.kernel().counters_view().bus_published,
        0,
        "host counter view agrees (pipe from the same bus)"
    );

    sup.kernel().stop(&id).expect("cooperative stop");
    sup.kernel().uninstall(&id).expect("uninstall");
}

/// Grammar pinning for the two new talkers + the HELLO capability tail
/// (D96 rulings 3+4): byte-exact roundtrips, the legacy tail-less HELLO
/// reading as caps 0, and the strict refusals (unknown ops, length lies,
/// partial capability tails).
#[test]
fn talker_and_capability_grammars_are_pinned() {
    assert_eq!(MSG_FORWARD, 10, "first new frame type code");
    assert_eq!(MSG_DELIVER, 11, "second new frame type code");
    assert_eq!(HELLO_CAP_BUS_V2, 1);
    assert_eq!(
        proc_wire::WIRE_VERSION,
        1,
        "vocabulary growth never bumps it"
    );

    // HELLO: legacy payload (no tail) reads caps 0 - the whole v1 handshake.
    let legacy = Hello {
        abi: 1,
        id: "com.example.modukit.legacy".into(),
        version: "0.9.0".into(),
        name: "legacy".into(),
        caps: 0,
    };
    let parsed = proc_wire::parse_hello_caps(&proc_wire::hello_payload(&legacy))
        .expect("legacy grammar still parses");
    assert_eq!(parsed, legacy);
    assert_eq!(parsed.caps, 0);

    // HELLO: extended payload carries the capability tail.
    let capable = Hello {
        caps: HELLO_CAP_BUS_V2,
        ..legacy.clone()
    };
    let parsed =
        proc_wire::parse_hello_caps(&proc_wire::hello_payload_caps(&capable)).expect("caps tail");
    assert_eq!(parsed.caps, HELLO_CAP_BUS_V2);
    let mut torn = proc_wire::hello_payload_caps(&capable);
    torn.pop();
    assert_eq!(
        proc_wire::parse_hello_caps(&torn),
        None,
        "a 3-byte cap tail is a lie, refused"
    );

    // FORWARD/DELIVER: exact roundtrips for every op and the refusal shape.
    for op in [proc_wire::FWD_PUB, proc_wire::FWD_SUB, proc_wire::FWD_UNSUB] {
        let fwd = Forward {
            op,
            topic: "com.example.modukit.busv2".into(),
            qos: i32::from(op),
            value: if op == proc_wire::FWD_PUB {
                b"payload".to_vec()
            } else {
                Vec::new()
            },
        };
        assert_eq!(
            proc_wire::parse_forward(&proc_wire::forward_payload(&fwd)),
            Some(fwd)
        );
    }
    let delivery = Delivery {
        topic: "com.example.modukit.busv2".into(),
        qos: -26,
        value: Vec::new(),
    };
    assert_eq!(
        proc_wire::parse_deliver(&proc_wire::deliver_payload(&delivery)),
        Some(delivery)
    );

    // Strict refusals: unknown op, short payload, length lie, trailing bytes.
    let mut bad_op = proc_wire::forward_payload(&Forward {
        op: proc_wire::FWD_SUB,
        topic: "com.example.modukit.busv2".into(),
        qos: 0,
        value: Vec::new(),
    });
    bad_op[0] = 9;
    assert_eq!(proc_wire::parse_forward(&bad_op), None);
    assert_eq!(proc_wire::parse_forward(&[]), None);
    let lie = proc_wire::forward_payload(&Forward {
        op: proc_wire::FWD_PUB,
        topic: "com.example.modukit.busv2".into(),
        qos: 0,
        value: b"x".to_vec(),
    });
    let mut lying = lie.clone();
    let tail = lying.len() - 1;
    lying[tail] = 0xFF;
    lying.push(0);
    assert_eq!(
        proc_wire::parse_forward(&lying),
        None,
        "a declared length that disagrees with the frame is refused"
    );
}
