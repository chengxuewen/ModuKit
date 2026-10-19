//! The S2-M3b host-loop driver: one step pass collapses poll + restart
//! policy application, and a quiet run drives to idle without spinning.

use super::*;

/// S2-M3b host-loop driver: ONE `step` pass replaces the hand-assembled
/// poll → classify → revive sequence (crash-watch, due-posting, and the
/// D3 restart policy in one doorway). The backoff window stays a gate —
/// the same pass defers instead of force-restarting.
#[test]
fn one_driver_step_pass_collapses_poll_and_restart_policy_application() {
    let Some(dir) = build_standins("driver-step") else {
        return;
    };
    let sup = step_supervisor();
    let restart = "[restart]\npolicy = \"on-failure\"\nmax_retries = 1\nbackoff = \"1s..4s\"\n";
    let id = install_resolve_start(&sup, &dir.join("standin-crasher"), restart);

    // The crasher self-exits right after the START ack; poll each driver
    // pass until it observes the death (real process timing, bounded).
    let deadline = Instant::now() + CRASH_POLL_DEADLINE;
    let first = loop {
        let report = sup.step();
        if !report.crashed.is_empty() {
            break report;
        }
        assert!(
            Instant::now() < deadline,
            "crash never observed within the poll deadline"
        );
        std::thread::sleep(Duration::from_millis(2));
    };
    assert_eq!(first.crashed, vec![ID.to_string()]);
    assert!(first.revived.is_empty());
    assert_eq!(
        first.deferred,
        vec![ID.to_string()],
        "the same pass applies the policy: the window is closed, so DEFER"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("tracked"),
        PluginState::Failed
    );

    // Turn the boot clock past the window; the next single pass restarts.
    sup.kernel().advance_clock(Duration::from_secs(4));
    let second = sup.step();
    assert_eq!(
        second.revived,
        vec![ID.to_string()],
        "the policy opened the window — one pass, respawned"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("restarted"),
        PluginState::Active
    );

    let _ = sup.poll();
    let _ = sup.kernel().stop(&id); // the respawned crasher may be gone; kill path is legal
    let _ = sup.poll();
    assert_no_zombie_children();
    std::fs::remove_dir_all(&dir).expect("cleanup scratch");
    assert!(!dir.exists(), "mk-standin scratch must not leak");
}

/// A quiet supervised run drives to idle after exactly one bounded pass —
/// the host-loop driver never spins (S2-M3b seam contract).
#[test]
fn drive_until_idle_returns_after_one_quiet_pass() {
    let Some(dir) = build_standins("driver-idle") else {
        return;
    };
    let sup = step_supervisor();
    let id = install_resolve_start(&sup, &dir.join("standin-good"), "");

    let driven = sup.drive_until_idle(8);
    assert_eq!(driven.passes, 1, "a quiet host is idle after one pass");
    assert!(driven.report.crashed.is_empty(), "{:?}", driven.report);
    assert!(driven.report.revived.is_empty(), "{:?}", driven.report);
    assert!(driven.report.deferred.is_empty(), "{:?}", driven.report);

    sup.kernel().stop(&id).expect("stop");
    sup.kernel().uninstall(&id).expect("uninstall");
    assert_eq!(sup.poll(), Vec::<String>::new());
    assert_no_zombie_children();
    std::fs::remove_dir_all(&dir).expect("cleanup scratch");
    assert!(!dir.exists(), "mk-standin scratch must not leak");
}
