//! The narrow transport seam (D11, modules/07 §2). ONE contract suite run
//! against every message-lane backend proves the backends are interchangeable
//! porters behind the trait — the 04 §4 "one suite, shapes locked" philosophy
//! applied small (C0: written before `seam.rs` existed, so the first red is an
//! unresolved-import compile error).

// D104 T2.4: this whole file is test scaffolding. The clippy.toml
// allow-*-in-tests valve exempts #[test] fn bodies only, not plain helper
// fns in a tests/ target (measured on clippy 0.1.99), so the exemption
// lands here; production zones stay denied.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use modukit_transport::seam::{Dummy, SocketpairLane, Transport};

/// The locked contract every local lane must satisfy identically. Message
/// boundaries survive, order is FIFO within a lane, and a closed lane refuses
/// rather than silently swallowing a send.
fn run_lane_contract<T: Transport>(mut lane: T) {
    lane.connect()
        .expect("connect must succeed on a fresh lane (rendezvous)");

    // Owned-value payloads (04 §2): a small frame and a frame large enough to
    // cross any inline threshold — both must round-trip as whole messages.
    lane.send(vec![0xA1, 0xB2, 0xC3]).expect("send small frame");
    lane.send(vec![7u8; 300]).expect("send large frame");
    assert_eq!(
        lane.recv().expect("recv first frame"),
        vec![0xA1, 0xB2, 0xC3],
        "FIFO order within the lane"
    );
    assert_eq!(
        lane.recv().expect("recv second frame"),
        vec![7u8; 300],
        "message boundary preserved (a stream backend must frame, never merge)"
    );

    lane.close().expect("close the lane");
    let err = lane
        .send(vec![1])
        .expect_err("a closed lane must error on send, never silently drop");
    assert_eq!(
        err.code(),
        "CARRIER_UNREACHABLE",
        "closed-lane failure reuses a D51 runtime code, never an invented one"
    );
}

#[test]
fn the_dummy_double_satisfies_the_seam_contract() {
    run_lane_contract(Dummy::new());
}

#[test]
fn the_socketpair_lane_satisfies_the_same_seam_contract() {
    run_lane_contract(SocketpairLane::loopback().expect("socketpair is available here"));
}

/// Compile-time lock: the shapes implement the seam trait. If a backend drifts
/// off the trait this fails to build, not to pass a vacuous test.
#[test]
fn every_local_lane_backend_is_locked_to_the_trait() {
    fn assert_transport<T: Transport>() {}
    assert_transport::<Dummy>();
    assert_transport::<SocketpairLane>();
}

/// The closed-state arms the happy-path contract never reaches: a closed
/// Dummy refuses connect/recv as well as send, and an empty loopback queue
/// answers with the documented Backend prose (never a blocking wait — the
/// double must not hang a contract run).
#[test]
fn a_closed_dummy_refuses_every_verb_and_an_empty_queue_is_a_typed_backend() {
    let mut closed = Dummy::new();
    closed.close().expect("close");
    closed
        .connect()
        .expect_err("a closed double refuses the rendezvous");
    closed.recv().expect_err("a closed double refuses a read");

    let mut fresh = Dummy::new();
    fresh.connect().expect("connect");
    let err = fresh
        .recv()
        .expect_err("an empty loopback must answer, never block");
    assert_eq!(err.code(), "CARRIER_UNREACHABLE", "{err}");
    assert!(
        format!("{err}").contains("loopback queue is empty"),
        "the prose names the empty queue, got: {err}"
    );
}

/// Close drops both ends of the pair, so every later verb reports Closed
/// through the Option arms — the half-open lane the header warns about
/// never materialises. `new()` is pinned alongside as the fresh state.
#[test]
fn a_closed_socketpair_lane_reports_closed_on_send_recv_and_connect() {
    let mut lane = SocketpairLane::loopback().expect("socketpair is available here");
    lane.send(vec![1, 2, 3]).expect("live lane accepts a frame");
    assert_eq!(lane.recv().expect("frame back"), vec![1, 2, 3]);
    lane.close().expect("close both ends");
    assert!(lane.send(vec![9]).is_err(), "closed send");
    let err = lane.recv().expect_err("closed recv");
    assert_eq!(err, modukit_transport::error::TransportError::Closed);
    lane.connect()
        .expect_err("a dropped pair cannot re-validate the rendezvous");
}
