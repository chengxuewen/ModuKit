//! Logger service with `LogEvent` in the shared ring (D19, modules/10 §5) —
//! M6 floor 3. The plugin mouth is an ordinary kernel service: acquire it
//! like anything else (no ambient authority), records ride the same bounded
//! drop-counted ring as frames and lifecycle events, and a slow consumer
//! never blocks the producer (observation, not enforcement).

use std::sync::Arc;

use modukit_core::clock::{Clock, ClockDomain};
use modukit_core::events::ObsEvent;
use modukit_core::identity::ServiceId;
use modukit_core::kernel::Kernel;
use modukit_core::logger::{LogLevel, Logger};

#[test]
fn logger_is_an_ordinary_kernel_service_under_org_modukit() {
    let kernel = Kernel::new();
    let view = kernel
        .registry()
        .lookup(&ServiceId::new("org.modukit.logger".to_string()))
        .expect("kernel registers org.modukit.logger");
    assert_eq!(view.service_id.as_str(), "org.modukit.logger");
    let handle = kernel
        .registry()
        .acquire::<Logger>()
        .expect("acquire the logger like any service (01 §3)");
    handle.log(
        LogLevel::Info,
        "com.test.writer",
        "hello ring",
        vec![("lane".to_string(), "bus".to_string())],
    );
    let rec = kernel
        .observatory()
        .tail(0)
        .events
        .into_iter()
        .find(|r| matches!(r.event, ObsEvent::Log(_)))
        .expect("the record landed on the shared ring");
    match rec.event {
        ObsEvent::Log(log) => {
            assert_eq!(log.level, LogLevel::Info);
            assert_eq!(log.component, "com.test.writer");
            assert_eq!(log.msg, "hello ring");
            assert_eq!(log.kv, vec![("lane".to_string(), "bus".to_string())]);
        }
        other => panic!("expected Log, got {other:?}"),
    }
    kernel
        .registry()
        .release(&handle)
        .expect("acquire/release pair");
}

#[test]
fn log_records_carry_seq_and_the_unified_clock_stamp() {
    let kernel = Kernel::with_clock(Clock::new(ClockDomain::Step));
    let logger = kernel.logger();
    kernel.clock().advance(std::time::Duration::from_millis(7));
    logger.log(LogLevel::Warn, "com.test.t", "stamped", Vec::new());
    let base = kernel.observatory().last_seq() - 1;
    let rec = &kernel.observatory().tail(base).events[0];
    assert_eq!(
        rec.ts.raw(),
        7_000_000,
        "the log axis IS the clock dial (D19/D20 one ruler)"
    );
    assert!(rec.seq > base, "monotonic seq like every other ring event");
}

#[test]
fn a_flooding_producer_never_blocks_and_drops_are_visible() {
    // Same bounded-ring discipline as the M5 bus: overflow sheds the oldest
    // and counts it — the producer keeps moving (06 §2 commandments).
    let clock = Clock::new(ClockDomain::Step);
    let obs = Arc::new(modukit_core::events::Observatory::new(clock));
    let logger = Logger::new(Arc::clone(&obs));
    let n = modukit_core::events::RING_CAP + 250;
    for i in 0..n {
        logger.log(
            LogLevel::Info,
            "com.test.flood",
            &format!("line {i}"),
            Vec::new(),
        );
    }
    assert_eq!(obs.retained(), modukit_core::events::RING_CAP);
    assert!(
        obs.drops() >= 250,
        "every shed line is counted, never silent"
    );
    let tail = obs.tail(0);
    assert_eq!(
        tail.first_seq,
        n as u64 - modukit_core::events::RING_CAP as u64 + 1
    );
    // The newest record survived (drop-OLDEST).
    let last = tail.events.last().expect("retained");
    match &last.event {
        ObsEvent::Log(log) => assert_eq!(log.msg, format!("line {}", n - 1)),
        other => panic!("expected the newest Log, got {other:?}"),
    }
}

#[test]
fn per_component_line_counts_are_read_back_not_enforced() {
    let kernel = Kernel::new();
    let logger = kernel.logger();
    for _ in 0..5 {
        logger.log(LogLevel::Debug, "com.test.counter", "tick", Vec::new());
    }
    logger.log(LogLevel::Debug, "com.test.other", "tick", Vec::new());
    assert_eq!(
        logger.component_count("com.test.counter"),
        5,
        "visibility per component (overflow is visible, never silent)"
    );
    assert_eq!(logger.component_count("com.test.absent"), 0);
}
