//! Wave-2 T2 clock-crossing suite (D99 R5-A, contract drawn at
//! docs/wave-2-clock-hearing.md §8, plan docs/wave-2-t2-clock-crossing-plan.md):
//! the plugin bell rides the EXISTING frames across the wall. Harness is the
//! abi_v2_cross / dual_placement real-runner precedent verbatim (fixtures are
//! not built by `cargo test`; a runner build failure is FATAL, a fixture
//! build failure skips cleanly; the runner env levers ride the serial guard).
//!
//! leg OUT: an exe child's v3-scheduled timer fires on its runner clock when
//! the SUPERVISOR dials it; `post_timer_bell`'s bridge outlet writes the bell
//! as an ordinary FWD_PUB mid-dial, the crossing dial absorbs it into the
//! parent's fan-out (the existing `handle_forward`, zero new frame types),
//! and a parent-bus subscriber decodes `gen ++ token ++ label` off the
//! delivered bytes. The token equality is the DUE answer of the same dial
//! turn (mirror law, never a literal); the boot generation is opaque.
//!
//! FINDING (reported, NOT worked around - the plan's stop clause): the
//! parent->child LIVE pump leg (leg DOWN end to end) is not observable with
//! the named precedents. Derivation: (1) the supervisor's kernel is booted
//! with `ProcessExecutor`, which polices its placement kind, so it hosts NO
//! dl plugin and its `plugin_timers` register can never fill - there is no
//! parent-originated bell producer on any tree that also owns a fanout;
//! (2) even with a producer, `pump_local_bell` can reach a child only while
//! the child's interest is live, and that window is the START exchange
//! (`retire` at the ack sweeps it - 01 §3 rule 2, the T2 plan keeps it);
//! (3) the parent's dial of its own kernel cannot run concurrently with the
//! child's start call: `Kernel` holds a `Box<dyn Executor>` trait object, so
//! it is not `Send`/`Sync` and no second thread may touch it; a test-driven
//! dial happens strictly before or after the window. The runner's main loop
//! after `start` also has no DELIVER consumer. The mechanism therefore lands
//! with its proof at the levels that ARE observable: `bus_wire.rs::tests`
//! (pump grammar, timer-target filter, retire purge - frame-level) and
//! `wiring.rs::bell_crossing_tests` (the REAL bell site pumping a wired
//! fan-out, gen self-equality). The legacy-pin receive half (a BUS-only
//! child gets no pump DELIVER) is asserted there; the send half is the
//! lever column below.
//!
//! The child-local-subscriber assertion the plan drafted ("mirror law") is
//! the same class: the runner's local bus has no post-start subscriber and
//! dies with the process, so child-side receipt is proven via the DUE answer
//! (the runner fired the token against its own clock) plus the parent-side
//! crossing of the identical payload.

// D104 T2.4: test-scaffolding file; the clippy.toml allow-*-in-tests valve
// exempts #[test] fn bodies only, not plain helper fns in a tests/ target
// (clippy 0.1.99), so the exemption lands here. Production stays denied.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use modukit_core::executor::Supervisor;
use modukit_core::identity::TopicId;
use modukit_core::lifecycle::PluginState;
use modukit_core::manifest::Manifest;
use modukit_core::proc_wire::{self, HELLO_CAP_BUS_V2, HELLO_CAP_TIMER_V2, Hello, WIRE_VERSION};

/// The timerv3 fixture's self-report (its descriptor id/version - the exe
/// arm cross-checks HELLO against the manifest strictly).
const ID: &str = "com.example.modukit.timerv3";
const VERSION: &str = "0.1.0";
/// The composed per-plugin bell topic (D99 R4-A) for the exe child.
const BELL_TOPIC: &str = "org.modukit.timer.com.example.modukit.timerv3";
const JOB_LABEL: &[u8] = b"tick-v3";

/// Runner env levers are process env (inherited at spawn, read during
/// `resolve`) - serialize the two columns that spawn children.
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

fn target_dir() -> PathBuf {
    repo_root().join("out").join("clock-cross-target")
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

fn build_timerv3_so() -> Option<PathBuf> {
    if std::env::var_os("MODUKIT_SKIP_DLOPEN").is_some() {
        eprintln!("SKIPPING: MODUKIT_SKIP_DLOPEN is set");
        return None;
    }
    match cargo_build(&["build", "-p", "timerv3-plugin"]) {
        Some(true) => {}
        Some(false) => {
            eprintln!("SKIPPING: fixture build failed");
            return None;
        }
        None => return None,
    }
    let so = target_dir().join("debug").join("libtimerv3_plugin.so");
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
    .expect("crossing-suite manifest parses")
}

/// Decode the pinned bell payload grammar (D99 R4-A + the T2 generation
/// field): gen u64 LE ++ token u64 LE ++ label UTF-8. The generation is
/// opaque here - the child runner's own kernel tag, visible only as bytes.
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

/// Grammar pinning for the T2 capability bit (D99 R5-A): the next bit after
/// the landed BUS_V2, the caps tail carries both, and the wire version
/// stands (vocabulary growth never bumps a transport promise, ruling 4).
#[test]
fn timer_capability_grammar_is_pinned() {
    assert_eq!(HELLO_CAP_TIMER_V2, 2, "the next bit after BUS_V2 = 1");
    assert_eq!(HELLO_CAP_BUS_V2, 1);
    assert_eq!(WIRE_VERSION, 1, "T2 grew no wire version");
    let hello = Hello {
        abi: 1,
        id: ID.to_string(),
        version: VERSION.to_string(),
        name: "timerv3-plugin".to_string(),
        caps: HELLO_CAP_BUS_V2 | HELLO_CAP_TIMER_V2,
    };
    let parsed = proc_wire::parse_hello_caps(&proc_wire::hello_payload_caps(&hello))
        .expect("the two-bit caps tail parses");
    assert_eq!(parsed.caps, HELLO_CAP_BUS_V2 | HELLO_CAP_TIMER_V2);
    // A BUS-only runner (the lever shape) round-trips unchanged, and the
    // legacy tail-less grammar still reads caps 0.
    let bus_only = proc_wire::Hello {
        caps: HELLO_CAP_BUS_V2,
        ..hello.clone()
    };
    let parsed = proc_wire::parse_hello_caps(&proc_wire::hello_payload_caps(&bus_only)).unwrap();
    assert_eq!(parsed.caps & HELLO_CAP_TIMER_V2, 0);
    let parsed =
        proc_wire::parse_hello_caps(&proc_wire::hello_payload(&hello)).expect("legacy grammar");
    assert_eq!(parsed.caps, 0);
    assert_eq!(proc_wire::MSG_DIAL, 8, "the dial talker the crossing rides");
}

/// leg OUT: the exe child schedules on its runner clock during the walk,
/// the supervisor's wire dial turns it, and the bell crosses the wall as an
/// ordinary FWD_PUB into the PARENT's bus - decoded gen+token+label by a
/// host subscriber, token equal to the DUE answer of the same turn.
#[test]
fn leg_out_child_bell_crosses_the_wall_at_the_dial_site() {
    let _serial = serial();
    let Some(so) = build_timerv3_so() else { return };
    let runner = build_runner();
    let sup = Supervisor::new();
    let m = manifest(&so, &runner);
    let id = m.id.clone();
    // Capability ON explicitly (the serial-guard pattern of abi_v2_cross):
    // no demo lever - the runner declares BUS|TIMER.
    // SAFETY: single-writer env discipline under the serial guard.
    unsafe { std::env::set_var("MODUKIT_BUS", "on") };
    sup.kernel().install(m).expect("exe install door");
    sup.kernel().resolve(&id).expect("spawn + HELLO");
    sup.kernel()
        .start(&id)
        .expect("the v3 walk (prefix mailbox + schedule) holds across the wall");
    unsafe { std::env::remove_var("MODUKIT_BUS") };
    assert_eq!(
        sup.kernel().state(&id).expect("tracked"),
        PluginState::Active
    );

    // A live parent subscriber BEFORE the dial hears the bell mid-turn.
    let bell = TopicId::new(BELL_TOPIC.to_string());
    let mut rx = sup
        .kernel()
        .bus()
        .subscribe(&bell)
        .expect("host subscribe to the child's bell topic");

    // The dial turn: the runner fires marker alpha (1 s) and the plugin job
    // (1 s); the plugin bell's FORWARD arrives BEFORE the DUE answer and the
    // crossing dial absorbs it into the fan-out. A hang here is RED by
    // construction (plan §4: this is the first test that exercises it).
    let turn = sup
        .dial(&id, Duration::from_millis(1500))
        .expect("the wire dial answers DUE, absorbing the child's bell FORWARD");
    let fired = turn
        .jobs
        .iter()
        .find(|job| job.job() == "tick-v3")
        .expect("the plugin job fired on the runner clock (mirror law)");

    let payload = rx
        .try_recv()
        .expect("the child's bell reached the PARENT bus (the crossing claim)");
    let (boot, token, label) = decode_bell(&payload);
    assert_eq!(
        token,
        fired.token().value(),
        "the bell names the fired token"
    );
    assert_eq!(label, JOB_LABEL, "the message label rides verbatim");
    let _ = boot; // the runner's boot generation: opaque here, never literal
    assert_eq!(
        sup.kernel()
            .bus()
            .topic_stats()
            .iter()
            .find(|st| st.topic == BELL_TOPIC)
            .expect("tallied")
            .published,
        1,
        "exactly one bell crossed"
    );
    // Late-join replay of the same entry: byte-identical ring content (the
    // unified LOSSY_RING capacity, the mirror law of ruling 6).
    let mut rx2 = sup.kernel().bus().subscribe(&bell).expect("late-join");
    assert_eq!(
        rx2.try_recv().as_deref(),
        Some(payload.as_slice()),
        "the replayed entry carries the same generation bytes"
    );

    assert_eq!(
        sup.kernel().counters_view().process_crashes,
        0,
        "the crossing must not cost a crash"
    );
    sup.kernel().stop(&id).expect("cooperative stop");
    sup.kernel().uninstall(&id).expect("uninstall");
}

/// The legacy pin (send half): a runner that drops ONLY the timer bit
/// (`DEMO_WAVE2_CLOCK_SABOTAGE=1` - BUS stays on, the bridge stays live)
/// bells child-local and crosses NOTHING. The walk still passes (the T1
/// exe-interim law), the dial still answers, and the parent's bus never
/// hears the bell topic - asserted by counter, never by prose.
#[test]
fn lever_drops_the_timer_bit_and_the_bell_stays_home() {
    let _serial = serial();
    let Some(so) = build_timerv3_so() else { return };
    let runner = build_runner();
    let sup = Supervisor::new();
    let m = manifest(&so, &runner);
    let id = m.id.clone();
    // SAFETY: the serial guard owns this env window; the runner inherits at
    // the spawn that `resolve` performs.
    unsafe {
        std::env::set_var("MODUKIT_BUS", "on");
        std::env::set_var("DEMO_WAVE2_CLOCK_SABOTAGE", "1");
    };
    sup.kernel().install(m).expect("exe install door");
    sup.kernel().resolve(&id).expect("spawn + HELLO (BUS only)");
    sup.kernel()
        .start(&id)
        .expect("the walk holds on the local bus with the timer bit off");
    unsafe {
        std::env::remove_var("DEMO_WAVE2_CLOCK_SABOTAGE");
        std::env::remove_var("MODUKIT_BUS");
    };

    let bell = TopicId::new(BELL_TOPIC.to_string());
    let mut rx = sup
        .kernel()
        .bus()
        .subscribe(&bell)
        .expect("host subscribe to the child's bell topic");
    let turn = sup
        .dial(&id, Duration::from_millis(1500))
        .expect("the dial still answers DUE (no FORWARD was owed to it)");
    assert!(
        turn.jobs.iter().any(|job| job.job() == "tick-v3"),
        "the plugin still fired - child-local, exactly as T1 pinned"
    );
    assert!(
        rx.try_recv().is_none(),
        "a BUS-only child's bell must never cross the wall"
    );
    assert!(
        !sup.kernel()
            .bus()
            .topic_stats()
            .iter()
            .any(|st| st.topic == BELL_TOPIC && st.published > 0),
        "no publish ever reached the parent's bell topic (own subscribe aside)"
    );
    assert_eq!(
        sup.kernel().counters_view().process_crashes,
        0,
        "the lever silences the outlet, it does not kill the run"
    );
    sup.kernel().stop(&id).expect("cooperative stop");
    sup.kernel().uninstall(&id).expect("uninstall");
}
