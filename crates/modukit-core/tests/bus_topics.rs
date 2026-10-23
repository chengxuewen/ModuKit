//! In-process topic bus (D10, modules/06 §2/§5, M5). Year-1 behavior tests.
//!
//! Written before the `bus`/`topic` modules exist (C0: first-red observed as an
//! unresolved-import compile error, the M3/M4 recipe). The three commandments
//! (no request/response, no cross-publisher ordering, slow subscriber never
//! pushes through the publisher) are asserted as behavior here and annotated
//! at each `bus.rs` site.

// D104 T2.4: test-scaffolding file; the clippy.toml allow-*-in-tests valve
// exempts #[test] fn bodies only, not plain helper fns in a tests/ target
// (clippy 0.1.99), so the exemption lands here. Production stays denied.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use modukit_core::bus::{Bus, BusError};
use modukit_core::identity::TopicId;
use modukit_core::kernel::Kernel;
use modukit_core::topic::QoS;

fn topic(name: &str) -> TopicId {
    TopicId::new(name.to_string())
}

// ---- Commandment 1: no request/response ---------------------------------
// Topics are a one-way broadcast: `publish` returns nothing that could carry
// an answer, and a `Receiver` exposes no send-side verb. This test pins the
// shape (there is no correlation id, no reply channel to assert against).

#[test]
fn publish_is_fire_and_forget_and_the_receiver_is_receive_only() {
    let bus = Bus::new();
    let t = topic("com.example.signal.gear");
    // `publish` yields `()` — the type itself forbids a response value.
    let unit: () = bus.publish(&t, QoS::LatestValue, b"P".to_vec());
    assert_eq!(unit, ());

    let mut rx = bus.subscribe(&t).expect("latest-value subscribes");
    // The receiver only has recv-side methods (try_recv/recv/drops); there is
    // no `send`/`request`/`call` on it — commandment 1 by construction.
    bus.publish(&t, QoS::LatestValue, b"D".to_vec());
    assert_eq!(rx.try_recv().as_deref(), Some(&b"D"[..]));
}

// ---- Late-join on latest-value (modules/06 §2) --------------------------
// A subscriber that joins after the fact receives the current value
// immediately, without the publisher republishing.

#[test]
fn late_join_on_latest_value_receives_current_value_immediately() {
    let bus = Bus::new();
    let t = topic("com.example.cockpit.speed");

    // Two publishes BEFORE anyone is listening: the register keeps the latest.
    bus.publish(&t, QoS::LatestValue, b"12".to_vec());
    bus.publish(&t, QoS::LatestValue, b"34".to_vec());

    let mut rx = bus.subscribe(&t).expect("subscribe late");
    assert_eq!(
        rx.try_recv().as_deref(),
        Some(&b"34"[..]),
        "late-joiner must see the current value at once (06 §2)"
    );

    // ...and then the live stream.
    bus.publish(&t, QoS::LatestValue, b"56".to_vec());
    assert_eq!(rx.try_recv().as_deref(), Some(&b"56"[..]));
}

// ---- Lossy(n): drop-oldest ring with a visible drop count ---------------

#[test]
fn lossy_n_holds_the_freshest_n_and_reports_a_drop_count() {
    let bus = Bus::new();
    let t = topic("com.example.telemetry.vibration");

    // Publish a,b,c,d with no reader; Lossy(2) keeps the freshest two,
    // evicting the oldest two and counting those evictions.
    for v in [b"a".to_vec(), b"b".to_vec(), b"c".to_vec(), b"d".to_vec()] {
        bus.publish(&t, QoS::Lossy(2), v);
    }
    assert_eq!(
        bus.topic_drop_count(&t),
        2,
        "two oldest values evicted from the capacity-2 ring (visible count)"
    );

    let mut rx = bus.subscribe(&t).expect("lossy subscribes");
    assert_eq!(rx.try_recv().as_deref(), Some(&b"c"[..]));
    assert_eq!(rx.try_recv().as_deref(), Some(&b"d"[..]));
    assert!(rx.try_recv().is_none(), "no more than the ring held");
}

// ---- KeepAll / Bounded: accepted at publish, refused at subscribe -------
// Silent wrongness is the D7 liar class: an unimplemented QoS must NOT
// quietly behave like latest-value. It refuses with a typed Deferral error.

#[test]
fn keep_all_is_accepted_at_publish_but_refuses_subscribe_with_a_typed_deferral() {
    let bus = Bus::new();
    let t = topic("com.example.audit.keepall");

    // Publish accepts the declaration (the vocabulary is kernel-owned).
    bus.publish(&t, QoS::KeepAll, b"row".to_vec());

    let err = bus
        .subscribe(&t)
        .expect_err("KeepAll is not built in year 1 — it must refuse");
    assert!(
        matches!(err, BusError::Deferred { qos: QoS::KeepAll }),
        "expected a typed Deferral naming KeepAll, got {err:?}"
    );
    assert_eq!(err.code(), "CONTRACT_VIOLATION", "reuses a D51 frozen code");
    // Crucially: the error carries no implicit latest-value value.
}

#[test]
fn bounded_is_accepted_at_publish_but_refuses_subscribe_with_a_typed_deferral() {
    let bus = Bus::new();
    let t = topic("com.example.batch.bounded");
    bus.publish(&t, QoS::Bounded, b"payload".to_vec());
    let err = bus
        .subscribe(&t)
        .expect_err("Bounded backpressure is trigger-gated, not year 1");
    assert!(
        matches!(err, BusError::Deferred { qos: QoS::Bounded }),
        "expected a typed Deferral naming Bounded, got {err:?}"
    );
}

// ---- Commandment 2: no cross-publisher ordering -------------------------
// The bus does not merge or sequence traffic across topics/publishers: a
// subscriber to one topic sees only that topic's values. (Per-topic streams
// are FIFO by construction, but there is no global ordering claim.)

#[test]
fn subscribers_are_topic_scoped_so_publishers_do_not_bleed_ordering() {
    let bus = Bus::new();
    let a = topic("com.example.plant.a");
    let b = topic("com.example.plant.b");
    let mut ra = bus.subscribe(&a).expect("subscribe a");
    let mut rb = bus.subscribe(&b).expect("subscribe b");

    // Two "publishers" interleaved across two topics.
    bus.publish(&a, QoS::Lossy(8), b"a1".to_vec());
    bus.publish(&b, QoS::Lossy(8), b"b1".to_vec());
    bus.publish(&a, QoS::Lossy(8), b"a2".to_vec());

    let a_vals: Vec<_> = std::iter::from_fn(|| ra.try_recv()).collect();
    let b_vals: Vec<_> = std::iter::from_fn(|| rb.try_recv()).collect();
    assert_eq!(a_vals, vec![b"a1".to_vec(), b"a2".to_vec()], "a only");
    assert_eq!(b_vals, vec![b"b1".to_vec()], "b only");
}

// ---- Commandment 3: a slow subscriber never pushes through the publisher -
// Publishing to a subscription nobody drains must not block the publisher and
// must surface a per-subscription drop count (observation only — no
// enforcement policy, per 06 §6).

#[test]
fn a_slow_subscriber_drops_its_own_backlog_and_never_blocks_the_publisher() {
    let bus = Arc::new(Bus::new());
    let t = topic("com.example.flood.sensor");
    let mut rx = bus.subscribe(&t).expect("latest-value subscribes (cap 1)");

    // Publish from a watchdog thread; the main thread measures that publish()
    // returns promptly for every call despite the subscriber never reading.
    let publisher = Arc::clone(&bus);
    let pt = topic("com.example.flood.sensor");
    let handle = thread::spawn(move || {
        for i in 0..50u32 {
            publisher.publish(&pt, QoS::LatestValue, i.to_le_bytes().to_vec());
        }
    });

    // If publish() ever blocked on the slow subscriber this join would hang.
    handle
        .join()
        .expect("publisher never blocks on a slow subscriber");

    // The single-slot subscription kept only the newest of the flood.
    let got = rx.try_recv().expect("the freshest value survived");
    assert_eq!(got, 49u32.to_le_bytes().to_vec(), "latest value wins");
    assert!(
        rx.drops() > 0,
        "the superseded intermediates are counted (06 §6 observation), got {}",
        rx.drops()
    );
}

// ---- Bus is reachable from the kernel; registry and bus stay separate ----

#[test]
fn kernel_exposes_the_bus_handle_and_it_is_a_separate_channel_from_the_registry() {
    let kernel = Kernel::new();
    let t = topic("org.modukit.reserved.example");

    // The kernel hands out the bus; publishing through it reaches a receiver
    // obtained from the same handle.
    {
        let mut rx = kernel.bus().subscribe(&t).expect("kernel bus subscribes");
        kernel.bus().publish(&t, QoS::LatestValue, b"hi".to_vec());
        assert_eq!(rx.try_recv().as_deref(), Some(&b"hi"[..]));
    }

    // Registry presence events and the bus are separate channels: publishing
    // on the bus must not create a registry entry, and the registry's event
    // stream is a distinct typed channel that shares no queue with the bus.
    // Platform services (clock, logger — M6/D40) are the boot baseline;
    // the channel-separation claim is that BUS traffic adds nothing.
    let boot_services = kernel.registry().snapshot().len();
    let registry_events = kernel.registry().subscribe();
    assert_eq!(
        kernel.registry().snapshot().len(),
        boot_services,
        "bus traffic must not register a service - the two channels are separate",
    );
    assert!(
        registry_events.try_recv().is_err(),
        "no registry activity happened, so the presence stream stays empty",
    );
}

// The bus must survive crossing a thread boundary (std threads only — D6).
#[test]
fn bus_is_send_and_shared_state_flows_across_threads() {
    fn assert_send<T: Send + Sync>() {}
    assert_send::<Bus>();
    assert_send::<BusError>();

    let bus = Arc::new(Bus::new());
    let t = topic("com.example.threaded.topic");
    let (done_tx, done_rx) = std::sync::mpsc::sync_channel::<()>(1);
    let mut rx = bus.subscribe(&t).expect("subscribe before the thread");

    let b = Arc::clone(&bus);
    let tt = t.clone();
    thread::spawn(move || {
        b.publish(&tt, QoS::LatestValue, b"across".to_vec());
        done_tx.try_send(()).ok();
    });

    done_rx
        .recv_timeout(Duration::from_secs(2))
        .expect("publisher thread finished");
    assert_eq!(rx.try_recv().as_deref(), Some(&b"across"[..]));
}
