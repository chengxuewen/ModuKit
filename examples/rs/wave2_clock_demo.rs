//! Wave-2 T2 clock-crossing demo (D99 R5-A, the crossing lane): the REAL
//! shadow-runner hosts the timerv3 fixture across the `exe` wall. The child
//! schedules a plugin timer on its runner clock inside the v3 walk; the
//! supervisor's WIRE DIAL turns it; the bell rides the existing FWD_PUB frame
//! mid-dial into the PARENT's bus, where a host subscriber decodes the
//! pinned payload (gen u64 LE ++ token u64 LE ++ label UTF-8, the token
//! equal to the DUE answer of the same turn - mirror law, no literals). The
//! wall-IN direction is asserted on the same DELIVER writer the parent-side
//! bell pump clones: the fixture's self-echo lands across the wall inside
//! the start window (scene [S3]). `scripts/demo-wave2-clock.sh` gates on
//! the final 'WAVE2-CLOCK-DEMO PASS' line.
//!
//! FINDING (docs/wave-2-t2-clock-crossing-plan.md execution notes): the
//! parent->child LIVE bell pump (a supervisor-originated bell reaching a
//! child) is not end-to-end observable today - the Supervisor kernel boots
//! the `exe` executor and hosts no dl plugin (no bell producer with a
//! fanout), the child's wall interest lives only inside the start window
//! (retire at the ack sweeps it), and `Kernel` is not `Send`/`Sync`, so no
//! second thread may dial while the start call is in flight. The pump
//! method itself lands with frame-level proof in `bus_wire.rs::tests` and
//! `wiring.rs::bell_crossing_tests`; this demo covers the directions the
//! real tree can speak, with the lever reddening scene [S4].
//!
//! Falsifiability lever (C0): `DEMO_WAVE2_CLOCK_SABOTAGE=1` drops the
//! runner's TIMER capability bit at exactly ONE named point (scene [S2],
//! before `resolve` - the runner inherits the env at spawn). The child then
//! walks and schedules happily child-local (the T1 exe interim), the dial
//! still answers DUE, but NO bell crosses: scene [S4] goes RED and no PASS
//! line prints:
//!   DEMO_WAVE2_CLOCK_SABOTAGE=1 bash scripts/demo-wave2-clock.sh  # expect RED

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::executor::Supervisor;
use modukit_core::identity::{PluginId, TopicId};
use modukit_core::manifest::Manifest;

const ID: &str = "com.example.modukit.timerv3";
const BELL_TOPIC: &str = "org.modukit.timer.com.example.modukit.timerv3";
const JOB_LABEL: &[u8] = b"tick-v3";

fn fail(msg: &str) -> ! {
    eprintln!("wave2_clock_demo: RED - {msg}");
    std::process::exit(1)
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("demo gate: examples package lives at <repo>/examples")
        .to_path_buf()
}

/// The stage1/n2 staging grammar: build the fixture cdylib and the runner
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

/// Decode the pinned bell payload grammar (D99 R4-A + the T2 generation
/// field): gen u64 LE ++ token u64 LE ++ label UTF-8. The generation is the
/// runner kernel's boot tag - read as bytes, compared only to itself.
fn decode_bell(payload: &[u8]) -> (u64, u64, &[u8]) {
    let Some((head, label)) = payload.split_at_checked(16) else {
        fail("bell payload shorter than the pinned gen(8) + token(8) head");
    };
    let (boot, token) = head.split_at(8);
    (
        u64::from_le_bytes(boot.try_into().expect("gen slot")),
        u64::from_le_bytes(token.try_into().expect("token slot")),
        label,
    )
}

fn main() {
    let so = build_through_cargo(
        &["build", "-p", "timerv3-plugin"],
        Path::new("libtimerv3_plugin.so"),
    );
    println!("[1] v3 fixture cdylib built: {}", so.display());
    let runner = build_through_cargo(
        &["build", "-p", "modukit-runtime"],
        Path::new("modukit-run"),
    );
    println!("[2] shadow-runner built: {}", runner.display());

    let sabotage = std::env::var("DEMO_WAVE2_CLOCK_SABOTAGE").is_ok();
    if sabotage {
        // [S2] THE ONE named sabotage point: the runner's timer capability
        // lever is set before the child is exec'd (the runner spawns at
        // `resolve` and inherits the env), so its HELLO drops ONLY the
        // HELLO_CAP_TIMER_V2 bit. The bus bridge stays live, the child
        // schedules child-local exactly as the T1 exe interim pinned - what
        // must break is the BELL CROSSING, scene [S4].
        // SAFETY: single-threaded demo process; the runner inherits at spawn.
        unsafe { std::env::set_var("DEMO_WAVE2_CLOCK_SABOTAGE", "1") };
        println!("[S2] DEMO_WAVE2_CLOCK_SABOTAGE=1: runner timer capability off");
    }

    // Step boot domain: the host turns every dial by hand (zero-sleep,
    // D24/D74 posture); the supervisor owns the child's dial over the wire.
    let sup = Supervisor::with_clock(Clock::new(ClockDomain::Step));
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
    println!(
        "[3] child spawned, HELLO cross-checked (caps tail carries the timer bit unless [S2])"
    );

    sup.kernel().start(&id).unwrap_or_else(|err| {
        // Every v3-walk disagreement across the wall parks a distinct
        // negative code in the fixture's own `start` - name it here. The
        // walk's self-echo step IS the wall-in leg: the parent-side fan-out
        // wrote a DELIVER frame across the wall and the child pumped it into
        // its local bus inside the start window (scene [S3]).
        fail(&format!("v3 walk disagreed across the wall: {err}"))
    });
    if !sabotage {
        unsafe { std::env::remove_var("DEMO_WAVE2_CLOCK_SABOTAGE") };
    }
    println!(
        "[S3] the v3 walk held ACROSS the wall: mailbox self-echo landed from the parent-side DELIVER writer (wall-in path), schedule child-local"
    );

    // [S4] The crossing claim: dial the child over the wire; its plugin
    // bell must ride the existing FWD_PUB mid-dial into the PARENT's bus
    // and decode there as gen ++ token ++ label with the token equal to the
    // DUE answer of the same turn. Under the [S2] lever no FORWARD is ever
    // spoken and this scene is the designed RED point.
    let bell = TopicId::new(BELL_TOPIC.to_string());
    let mut rx = sup
        .kernel()
        .bus()
        .subscribe(&bell)
        .expect("demo gate: host subscribe to the child's bell topic");
    let turn = sup
        .dial(&id, Duration::from_millis(1500))
        .unwrap_or_else(|err| fail(&format!("the crossing dial refused: {err}")));
    let fired = match turn.jobs.iter().find(|job| job.job() == "tick-v3") {
        Some(job) => job,
        None => {
            fail("the plugin job did not fire on the runner clock (child side broke, not the wall)")
        }
    };
    let payload = match rx.try_recv() {
        Some(bytes) => bytes,
        None => {
            fail("crossing absent: no bell reached the parent bus (this is the scene [S2] RED)")
        }
    };
    let (boot, token, label) = decode_bell(&payload);
    if token != fired.token().value() {
        fail(&format!(
            "bell token {token} disagrees with the DUE answer {}",
            fired.token().value()
        ));
    }
    if label != JOB_LABEL {
        fail(&format!("bell label disagreed: {label:?}"));
    }
    let _ = boot; // the runner's boot generation: opaque, never a literal
    // Late-join replay of the SAME entry: the unified LOSSY_RING ring answers
    // byte-identical content (the mirror law of ruling 6, replayed).
    let mut rx2 = sup
        .kernel()
        .bus()
        .subscribe(&bell)
        .expect("demo gate: late-join subscribe");
    match rx2.try_recv() {
        Some(replay) if replay == payload => {}
        _ => fail("the Lossy ring replayed a different generation/entry than the live bell"),
    }
    println!(
        "[S4] bell crossed the wall OUT: token={} label={:?} (DUE-agreeing), late-join replay byte-equal",
        fired.token().value(),
        std::str::from_utf8(label).unwrap_or("?")
    );

    if sup.kernel().counters_view().process_crashes != 0 {
        fail("the crossing must not cost a crash");
    }
    sup.kernel().stop(&id).expect("demo gate: cooperative stop");
    sup.kernel().uninstall(&id).expect("demo gate: uninstall");
    if sabotage {
        // The sabotage run must NEVER reach a PASS line: the RED above is
        // the designed outcome; belt the scene with an explicit stop.
        fail("sabotage: crossing scene passed - the gate would go green on a dead wire");
    }
    println!("[5] stopped + uninstalled");
    println!("WAVE2-CLOCK-DEMO PASS");
}
