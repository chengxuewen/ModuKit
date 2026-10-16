//! Unified clock + timers-as-jobs (D20, modules/03 §6) — M6 floor 4.
//!
//! One monotonic water source the kernel owns; timers are explicit jobs:
//! `timer(deadline) -> JobToken`, due by posting a token (message-shaped,
//! D6) — never a closure waking on some hidden thread. The step domain
//! advances only when the dial is turned, so tests carry zero wall-clock
//! sleeps.

use std::time::Duration;

use modukit_core::clock::{Clock, ClockDomain, Timestamp};
use modukit_core::identity::ServiceId;

#[test]
fn step_clock_is_monotonic_and_only_moves_when_advanced() {
    let clock = Clock::new(ClockDomain::Step);
    let t0 = clock.now();
    clock.advance(Duration::from_millis(100));
    let t1 = clock.now();
    clock.advance(Duration::from_millis(50));
    let t2 = clock.now();
    assert!(t0 < t1 && t1 < t2, "advance must move the dial forward");
    assert_eq!(
        t2.delta_since(t0),
        Duration::from_millis(150),
        "step domain accumulates exactly what advance() was given"
    );
}

#[test]
fn real_clock_is_monotonic() {
    let clock = Clock::new(ClockDomain::Real);
    let a = clock.now();
    std::hint::black_box(&a);
    let b = clock.now();
    assert!(a <= b, "monotonic: never goes backwards");
}

#[test]
fn timer_fires_by_posting_a_token_not_by_calling_a_closure() {
    let clock = Clock::new(ClockDomain::Step);
    let base = clock.now();
    let token = clock.timer(Timestamp::from(base.raw() + 2_000_000_000), "demo-fire");
    // Nothing is due before the dial reaches the deadline.
    assert!(clock.advance(Duration::from_millis(500)).is_empty());
    let due = clock.advance(Duration::from_secs(2));
    assert_eq!(due.len(), 1, "the deadline is reached exactly once");
    assert_eq!(due[0].token(), token, "same correlation token");
    assert_eq!(due[0].job(), "demo-fire", "timer is message-shaped (D6)");
    // Firing is terminal: a second turn does not re-post the token.
    assert!(clock.advance(Duration::from_secs(1)).is_empty());
}

#[test]
fn multiple_timers_fire_in_deadline_order() {
    let clock = Clock::new(ClockDomain::Step);
    let base = clock.now().raw();
    let late = clock.timer(Timestamp::from(base + 3_000_000_000), "late");
    let early = clock.timer(Timestamp::from(base + 1_000_000_000), "early");
    let due = clock.advance(Duration::from_secs(5));
    assert_eq!(due.len(), 2);
    assert_eq!(due[0].token(), early, "earliest deadline first");
    assert_eq!(due[1].token(), late);
}

#[test]
fn kernel_owns_the_clock_as_org_modukit_clock_service() {
    let kernel = modukit_core::kernel::Kernel::new();
    // The clock is a registered ordinary service (D40: one table, no
    // ambient authority): plugins acquire it like anything else.
    let view = kernel
        .registry()
        .lookup(&ServiceId::new("org.modukit.clock".to_string()))
        .expect("kernel registers org.modukit.clock");
    assert_eq!(view.service_id.as_str(), "org.modukit.clock");
    let clock = kernel.clock();
    let a = clock.now();
    let b = clock.now();
    assert!(a <= b, "the kernel clock is monotonic");
}
