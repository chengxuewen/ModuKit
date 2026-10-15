//! Tests: six-state machine + Failed edge + policy types (D3, modules/03 §1-2).
//! Written before the implementation (C0: red observed first).

use modukit_core::error::KernelError;
use modukit_core::lifecycle::{
    Backoff, PluginState, RestartPolicy, StateMachine, is_valid_transition,
};

#[test]
fn state_machine_starts_uninstalled() {
    assert_eq!(StateMachine::new().state(), PluginState::Uninstalled);
}

#[test]
fn happy_walk_follows_the_transition_table() {
    let mut m = StateMachine::new();
    for to in [
        PluginState::Installed,
        PluginState::Resolving,
        PluginState::Starting,
        PluginState::Active,
        PluginState::Stopping,
        PluginState::Resolved,
        PluginState::Uninstalled,
    ] {
        m.transition(to)
            .unwrap_or_else(|e| panic!("walk to {to} rejected: {e}"));
    }
    assert_eq!(m.state(), PluginState::Uninstalled);
}

#[test]
fn failed_edge_reachable_from_resolving_starting_stopping() {
    let mut m = StateMachine::new();
    m.transition(PluginState::Installed).ok();
    m.transition(PluginState::Resolving).ok();
    m.fail().expect("RESOLVING -> FAILED rejected");

    let mut m = StateMachine::new();
    for to in [
        PluginState::Installed,
        PluginState::Resolving,
        PluginState::Starting,
    ] {
        m.transition(to).ok();
    }
    m.fail().expect("STARTING -> FAILED rejected");

    let mut m = StateMachine::new();
    for to in [
        PluginState::Installed,
        PluginState::Resolving,
        PluginState::Starting,
        PluginState::Active,
        PluginState::Stopping,
    ] {
        m.transition(to).ok();
    }
    m.fail().expect("STOPPING -> FAILED rejected");
}

#[test]
fn dl_placement_has_no_active_to_failed_edge() {
    // modules/03 §1-2: FAILED from ACTIVE is process-placement only; the
    // §2 table has no ACTIVE -> FAILED row (dl host corruption is total).
    let mut m = StateMachine::new();
    for to in [
        PluginState::Installed,
        PluginState::Resolving,
        PluginState::Starting,
        PluginState::Active,
    ] {
        m.transition(to).ok();
    }
    let err = m
        .transition(PluginState::Failed)
        .expect_err("ACTIVE -> FAILED must be rejected");
    assert!(matches!(err, KernelError::InvalidTransition { .. }));
    assert_eq!(m.state(), PluginState::Active, "rejected move keeps state");
}

#[test]
fn invalid_transition_reports_typed_error_and_keeps_state() {
    let mut m = StateMachine::new();
    let err = m
        .transition(PluginState::Active)
        .expect_err("UNINSTALLED -> ACTIVE is not in the table");
    assert!(matches!(err, KernelError::InvalidTransition { .. }));
    assert_eq!(m.state(), PluginState::Uninstalled);
}

#[test]
fn restart_from_failed_requires_policy_budget() {
    // The FAILED -> RESOLVING edge is guarded by the restart policy (03 §2).
    assert!(!RestartPolicy::None.allows_restart(0));
    let on_failure = RestartPolicy::OnFailure {
        max_retries: 2,
        backoff: Backoff {
            min: std::time::Duration::from_secs(1),
            max: std::time::Duration::from_secs(30),
        },
        cooldown: std::time::Duration::from_secs(5 * 60),
    };
    assert!(on_failure.allows_restart(1));
    // Budget exhaustion parks in FAILED (03 §2 "stopped trying").
    assert!(!on_failure.allows_restart(2));
}

#[test]
fn uninstall_reachable_from_every_state() {
    // "any | UNINSTALLED" row of the §2 table.
    for from in [
        PluginState::Uninstalled,
        PluginState::Installed,
        PluginState::Resolving,
        PluginState::Starting,
        PluginState::Active,
        PluginState::Stopping,
        PluginState::Resolved,
        PluginState::Failed,
    ] {
        assert!(
            is_valid_transition(from, PluginState::Uninstalled),
            "uninstall must be reachable from {from}"
        );
    }
}
