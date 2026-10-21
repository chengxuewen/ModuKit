//! Stage-5 N2 cross-wall acceptance demo (D96 ruling 6, the wall-transparency
//! law): the REAL shadow-runner hosts the busv2 fixture over `exe` placement
//! and the fixture's whole mailbox walk - self-echo, need-report, non-
//! consuming probe, late-join snapshot, unsub -> NOT_FOUND, deferred refusal,
//! grammar refusals - runs ACROSS the wall. The parent's own bus must carry
//! the child's publishes (one fan-out code path) and hand a host late-joiner
//! the retained value, exactly like the in-process column
//! (tests/abi_v2_door.rs). `scripts/demo-stage5-n2.sh` gates on the final
//! `STAGE5-N2-DEMO PASS` line.
//!
//! Falsifiability lever (C0): `DEMO_STAGE5_N2_SABOTAGE=1` turns the runner's
//! capability flag off at exactly ONE named point (scene [S2], before
//! `resolve`). The child then walks its local bus - start still passes - but
//! the parent bus must stay silent, so the crossing assertion at scene [S4]
//! goes RED and no PASS line is printed:
//!   DEMO_STAGE5_N2_SABOTAGE=1 bash scripts/demo-stage5-n2.sh  # expect RED

use std::path::{Path, PathBuf};
use std::process::Command;

use modukit_core::executor::Supervisor;
use modukit_core::identity::{PluginId, TopicId};
use modukit_core::manifest::Manifest;

const ID: &str = "com.example.modukit.busv2.echo";
const TOPIC: &str = "com.example.modukit.busv2";

fn fail(msg: &str) -> ! {
    eprintln!("stage5_n2_demo: RED - {msg}");
    std::process::exit(1)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("demo gate: examples package lives at <repo>/examples")
        .to_path_buf()
}

/// The stage1_demo staging grammar: build the fixture cdylib and the runner
/// through cargo (the CARGO env the demo falls back for - the routed tool
/// chain, never a bare shim). An unbuilt fixture is fatal: this binary IS
/// the gate.
fn build_through_cargo(args: &[&str], expect_file: &Path) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let target = repo_root().join("out").join("dlopen-test-target");
    let status = Command::new(&cargo)
        .current_dir(repo_root())
        .args(args)
        .arg("--target-dir")
        .arg(&target)
        .env("CARGO_TARGET_DIR", &target)
        .status()
        .expect("demo gate: cargo must be spawnable");
    if !status.success() {
        panic!("demo gate: build failed ({args:?}): {status}");
    }
    let path = target.join("debug").join(expect_file);
    assert!(
        path.is_file(),
        "demo gate: artifact {} missing",
        path.display()
    );
    path
}

fn main() {
    let so = build_through_cargo(
        &["build", "-p", "busv2-plugin"],
        Path::new("libbusv2_plugin.so"),
    );
    println!("[1] fixture cdylib built: {}", so.display());
    let runner = build_through_cargo(
        &["build", "-p", "modukit-runtime"],
        Path::new("modukit-run"),
    );
    println!("[2] shadow-runner built: {}", runner.display());

    let sabotage = std::env::var("DEMO_STAGE5_N2_SABOTAGE").is_ok();
    if sabotage {
        // [S2] THE ONE named sabotage point: the capability flag is off
        // before the child is exec'd (the runner spawns at `resolve`), so
        // it never declares HELLO_CAP_BUS_V2 and never speaks the bus
        // talkers. The child-side walk still passes on its local bus -
        // what must break is the CROSSING, scene [S4].
        // SAFETY: single-threaded demo process; the runner inherits at spawn.
        unsafe { std::env::set_var("MODUKIT_BUS", "off") };
        println!("[S2] DEMO_STAGE5_N2_SABOTAGE=1: runner capability flag off");
    }

    let sup = Supervisor::new();
    let manifest = Manifest::from_toml_str(&format!(
        "id = \"{ID}\"\nversion = \"0.1.0\"\n[entry]\nkind = \"exe\"\nartifact = \"{}\"\nrunner = \"{}\"\n",
        so.display(),
        runner.display()
    ))
    .expect("demo gate: manifest parses");
    let id: PluginId = manifest.id.clone();
    sup.kernel()
        .install(manifest)
        .expect("demo gate: exe install door");
    sup.kernel()
        .resolve(&id)
        .expect("demo gate: spawn + HELLO cross-check");
    println!("[3] child spawned, HELLO cross-checked");

    sup.kernel().start(&id).unwrap_or_else(|err| {
        // Every mailbox-law disagreement across the wall parks a distinct
        // negative code in the fixture's own `start` - name it here.
        fail(&format!("mailbox walk disagreed across the wall: {err}"))
    });
    if !sabotage {
        unsafe { std::env::remove_var("MODUKIT_BUS") };
    }
    println!("[4] the full mailbox walk held ACROSS the wall (fixture codes 0)");

    // [S4] The crossing claim: the PARENT's bus carried the child's
    // publishes (same-name=same-topic across the wall) and a host
    // late-join snapshots the retained value. Under the sabotage flag the
    // parent bus stayed silent and this scene is the RED point.
    let topic = TopicId::new(TOPIC.to_string());
    let stats = sup.kernel().bus().topic_stats();
    let seen = stats
        .iter()
        .find(|st| st.topic == TOPIC)
        .unwrap_or_else(|| fail("crossing absent: parent bus never saw the child's topic"));
    if seen.published < 3 {
        fail(&format!(
            "parent bus publish tally {} < 3 (the forwards did not cross)",
            seen.published
        ));
    }
    let mut rx = sup
        .kernel()
        .bus()
        .subscribe(&topic)
        .expect("demo gate: host subscribe on the parent bus");
    let snapshot = rx.try_recv().unwrap_or_else(|| {
        fail("no retained snapshot on the parent bus (late-join across the wall broke)")
    });
    if snapshot != b"late-snapshot" {
        fail(&format!("retained value disagreed: {snapshot:?}"));
    }
    println!(
        "[S4] parent bus: {} published={} (retained late-join = {:?})",
        seen.topic, seen.published, snapshot
    );

    sup.kernel().stop(&id).expect("demo gate: cooperative stop");
    sup.kernel().uninstall(&id).expect("demo gate: uninstall");
    let census: Vec<String> = sup
        .kernel()
        .state(&id)
        .map(|s| s.to_string())
        .into_iter()
        .collect();
    if sabotage {
        // The sabotage run must NEVER reach a PASS line: the RED above is
        // the designed outcome; belt the scene with an explicit stop.
        fail("sabotage: crossing scene passed - the gate would go green on a dead wire");
    }
    println!("[5] stopped + uninstalled, state {}", census[0]);
    println!("STAGE5-N2-DEMO PASS");
}
