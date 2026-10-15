//! The narrow transport seam (D11, modules/07 §2). ONE contract suite run
//! against every message-lane backend proves the backends are interchangeable
//! porters behind the trait — the 04 §4 "one suite, shapes locked" philosophy
//! applied small (C0: written before `seam.rs` existed, so the first red is an
//! unresolved-import compile error).

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
