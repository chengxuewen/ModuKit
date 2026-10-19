//! Crash-watch and the D3 restart policy: bounded restarts, flapping parks.

use super::*;

#[test]
fn crash_injection_keeps_host_alive_lands_failed_edge_and_restarts_bounded() {
    // Arrange: on-failure policy with a budget of exactly one restart.
    let Some(dir) = build_standins("crash") else {
        return;
    };
    let sup = step_supervisor();
    let restart = "[restart]\npolicy = \"on-failure\"\nmax_retries = 1\nbackoff = \"1s..4s\"\n";
    let id = install_resolve_start(&sup, &dir.join("standin-crasher"), restart);

    // Act: the child self-kills right after the START ack; the watcher sees it.
    assert_eq!(wait_crash(&sup), vec![ID.to_string()]);

    // Assert: FAILED edge through the existing machine, crash counter moved,
    // and the restart budget gates on the D3 policy BEFORE the dial allows.
    assert_eq!(
        sup.kernel().state(&id).expect("host alive"),
        PluginState::Failed
    );
    assert_eq!(sup.kernel().counters().process_crashes(), 1);
    assert_eq!(
        lifecycle_walk(&sup),
        vec![
            "UNINSTALLED->INSTALLED",
            "INSTALLED->RESOLVING",
            "RESOLVING->STARTING",
            "STARTING->ACTIVE",
            "ACTIVE->STOPPING",
            "STOPPING->FAILED",
        ]
    );
    assert!(
        matches!(sup.revive(&id), Ok(Revive::Deferred)),
        "backoff is a gate, not a suggestion: no restart before the window"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("still FAILED"),
        PluginState::Failed
    );
    sup.kernel().advance_clock(Duration::from_secs(1));
    assert!(
        matches!(sup.revive(&id), Ok(Revive::Revived)),
        "budget allows one restart"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("restarted"),
        PluginState::Active
    );
    assert_eq!(sup.kernel().counters().process_restarts(), 1);

    // Act: the flapped child crashes again; the budget is now spent.
    wait_crash(&sup);
    sup.kernel().advance_clock(Duration::from_secs(1));

    // Assert: parked in FAILED with the budget exhausted - bounded, visible.
    assert!(
        matches!(sup.revive(&id), Ok(Revive::Parked)),
        "max_retries=1 must bound"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("parked"),
        PluginState::Failed
    );
    assert_eq!(sup.kernel().counters().process_crashes(), 2);
    assert_eq!(sup.kernel().counters().process_restarts(), 1);
    assert_no_zombie_children();
}

#[test]
fn flapping_peer_hits_max_restarts_and_parks_in_failed() {
    // Arrange: budget of two restarts, crasher stand-in.
    let Some(dir) = build_standins("flap") else {
        return;
    };
    let sup = step_supervisor();
    let restart = "[restart]\npolicy = \"on-failure\"\nmax_retries = 2\nbackoff = \"1s..4s\"\n";
    let id = install_resolve_start(&sup, &dir.join("standin-crasher"), restart);

    for cycle in 1..=3usize {
        wait_crash(&sup);
        sup.kernel().advance_clock(Duration::from_secs(10));
        let outcome = sup.revive(&id).expect("revive answers, never errors");
        if cycle <= 2 {
            assert!(
                matches!(outcome, Revive::Revived),
                "cycle {cycle} must restart"
            );
        } else {
            assert!(matches!(outcome, Revive::Parked), "cycle {cycle} must park");
        }
    }

    // Assert: three crashes observed, exactly two restarts granted, the
    // reload doorway ran exactly three handshakes - zero phantom reloads.
    assert_eq!(sup.kernel().counters().process_crashes(), 3);
    assert_eq!(sup.kernel().counters().process_restarts(), 2);
    assert_eq!(
        sup.kernel().counters().load_ok(),
        3,
        "one load per supervised run"
    );
    assert_eq!(
        sup.kernel().state(&id).expect("parked"),
        PluginState::Failed
    );
    let walk = lifecycle_walk(&sup);
    assert_eq!(
        walk.iter()
            .filter(|e| e.as_str() == "FAILED->RESOLVING")
            .count(),
        2,
        "restarts ride only the real FAILED->RESOLVING edge: {walk:?}"
    );
    assert_no_zombie_children();
}
