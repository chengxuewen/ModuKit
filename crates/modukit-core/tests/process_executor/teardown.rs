//! Teardown paths: graceful STOP message, the hung-peer kill at the named
//! constant, and the dial timeout guard killing a silent child.

use super::*;

#[test]
fn stop_message_drives_graceful_teardown_no_kill_needed() {
    // Arrange
    let Some(dir) = build_standins("graceful") else {
        return;
    };
    let sup = step_supervisor();
    let id = install_resolve_start(&sup, &dir.join("standin-good"), "");

    // Act: kernel.stop succeeds only if the child READ the STOP frame and
    // acked it - the timeout path would return Err(ProcessCrashed).
    sup.kernel().stop(&id).expect("graceful STOP message acked");

    // Assert: cooperative stop lands RESOLVED (03 §2), process reaped.
    assert_eq!(
        sup.kernel().state(&id).expect("stopped"),
        PluginState::Resolved
    );
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "nothing crashed on the way out"
    );
    assert_no_zombie_children();
}

#[test]
fn hung_peer_is_killed_after_named_timeout_constant() {
    // Arrange
    let Some(dir) = build_standins("hung") else {
        return;
    };
    let sup = step_supervisor();
    let id = install_resolve_start(&sup, &dir.join("standin-hung"), "");

    // Act: STOP is ignored; teardown must bound itself by the named constant.
    let at = Instant::now();
    let err = sup
        .kernel()
        .stop(&id)
        .expect_err("a peer that never acks STOP cannot stop cleanly");
    let waited = at.elapsed();

    // Assert: typed crash failure, FAILED edge, kill happened, bounded wait.
    assert!(matches!(err, KernelError::ProcessCrashed { .. }), "{err}");
    assert_eq!(
        sup.kernel().state(&id).expect("host alive"),
        PluginState::Failed
    );
    assert!(
        waited <= STOP_KILL_GRACE * 3,
        "teardown waited {waited:?}; the kill timeout constant is {STOP_KILL_GRACE:?}"
    );
    assert_no_zombie_children();
}

/// D74 timeout guard (S2-M3b): a child that never answers the dial is
/// killed at the named `DIAL_TIMEOUT` bound and reported through the
/// ordinary crash family — the dial can never be held open by a wedged
/// runner (the plan's timeout guard against a blocked runner).
#[test]
fn dial_of_a_silent_child_is_killed_at_the_named_dial_timeout() {
    let Some(dir) = build_standins("dial-timeout") else {
        return;
    };
    let sup = step_supervisor();
    let id = install_resolve_start(&sup, &dir.join("standin-hung"), "");

    let at = Instant::now();
    let err = sup
        .dial(&id, Duration::from_secs(1))
        .expect_err("a silent peer cannot hold the dial open");
    let waited = at.elapsed();

    assert!(matches!(err, KernelError::ProcessCrashed { .. }), "{err}");
    assert!(
        waited <= DIAL_TIMEOUT * 3,
        "dial waited {waited:?}; the named guard is {DIAL_TIMEOUT:?}"
    );
    assert_eq!(
        sup.poll(),
        Vec::<String>::new(),
        "killed+reaped at the dial doorway"
    );
    assert_no_zombie_children();
    std::fs::remove_dir_all(&dir).expect("cleanup scratch");
    assert!(!dir.exists(), "mk-standin scratch must not leak");
}
