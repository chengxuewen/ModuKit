//! D76 machine proof: ONE semantic-equivalence corpus, its assertion
//! functions parameterized over the seam trait, driven against the dummy
//! double AND the vendor-backed lane. No copy-pasted second corpus — the
//! same `run_lane_equivalence_corpus` body must hold on both backends, which
//! is what proves the abstraction is not captive. The whole file is gated on
//! the non-default `vendor-shm` feature: with the feature off it compiles to
//! nothing and the default world is untouched.

#![cfg(feature = "vendor-shm")]
// D104 T2.4: test-scaffolding file; the clippy.toml allow-*-in-tests valve
// exempts #[test] fn bodies only, not plain helper fns in a tests/ target
// (clippy 0.1.99), so the exemption lands here. Production stays denied.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::sync::Mutex;

use modukit_transport::seam::{Dummy, Transport};
use modukit_transport::vendor_shm::VendorShmLane;

/// Frame size the big-frame leg round-trips — above any inline threshold,
/// below every backend's ceiling (the vendor's is its named `MAX_FRAME_BYTES`).
const BIG_FRAME_BYTES: usize = 100 * 1024;

/// The locked shared semantics every lane backend must satisfy identically:
/// FIFO message boundaries through the write/read round-trip, the typed error
/// surface where "nothing to receive" and "lane gone" live, and idempotent
/// close (the drop/release accounting a lane can show at this seam — the
/// pool-domain `EPOCH_EXPIRED` leg has no lane-side counterpart and is
/// asserted where applicable: the stub pool's own suite, untouched here).
fn run_lane_equivalence_corpus<T: Transport>(lane: &mut T, big_frame: &[u8]) {
    lane.connect()
        .expect("connect succeeds on a fresh lane (rendezvous)");
    lane.connect()
        .expect("connect is idempotent on an already-connected lane");

    // Write/read round-trip: small frame, then big frame, read back FIFO.
    lane.send(vec![0xA1, 0xB2, 0xC3]).expect("send small frame");
    lane.send(big_frame.to_vec())
        .expect("send big frame at the shared-capability size");
    assert_eq!(
        lane.recv().expect("recv first frame"),
        vec![0xA1, 0xB2, 0xC3],
        "FIFO order within the lane"
    );
    assert_eq!(
        lane.recv().expect("recv big frame"),
        big_frame.to_vec(),
        "message boundary preserved end to end, byte for byte"
    );

    // Typed error surface at the empty-lane point: an error VALUE with a D51
    // code, never a silent zero-length success, never a panic.
    let err = lane
        .recv()
        .expect_err("an empty lane must error on recv, never fabricate a frame");
    assert_eq!(err.code(), "CARRIER_UNREACHABLE", "empty-lane code");

    // Drop/release accounting at the seam: close is idempotent and every verb
    // afterwards refuses with the typed Closed code.
    lane.close().expect("close the lane");
    lane.close().expect("close is idempotent");
    let err = lane
        .send(vec![1])
        .expect_err("a closed lane must error on send, never silently drop");
    assert_eq!(err.code(), "CARRIER_UNREACHABLE", "closed-lane send code");
    let err = lane
        .recv()
        .expect_err("a closed lane must error on recv too");
    assert_eq!(err.code(), "CARRIER_UNREACHABLE", "closed-lane recv code");
}

#[test]
fn the_dummy_double_passes_the_lane_equivalence_corpus() {
    let mut lane = Dummy::new();
    run_lane_equivalence_corpus(&mut lane, &vec![0x5Au8; BIG_FRAME_BYTES]);
}

/// Vendor lanes touch real OS resources (shm segments, event fds); the
/// leak-proof test below counts process-wide fds, so vendor tests sharing
/// this binary must not overlap their lane lifetimes.
static VENDOR_SERIAL: Mutex<()> = Mutex::new(());

#[test]
fn the_vendor_backed_lane_passes_the_same_lane_equivalence_corpus() {
    let _serial = VENDOR_SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut lane = VendorShmLane::loopback().expect("vendor lane opens on this host");
    run_lane_equivalence_corpus(&mut lane, &vec![0x5Au8; BIG_FRAME_BYTES]);
}

/// Count of this process's open file descriptors (leak-proof meter).
fn open_fd_count() -> usize {
    std::fs::read_dir("/proc/self/fd")
        .expect("procfs is readable")
        .count()
}

/// Resident set in kilobytes, read from /proc/self/status VmRSS.
fn resident_kb() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").expect("procfs is readable");
    status
        .lines()
        .find_map(|l| l.strip_prefix("VmRSS:"))
        .and_then(|rest| rest.trim().trim_end_matches(" kB").trim().parse().ok())
        .expect("VmRSS is present in /proc/self/status")
}

/// Generous RSS-growth bound for the leak proof: the loop moves tiny frames
/// through reused ports, so real growth is noise; the bound only has to
/// outrun allocator/CI wobble (documented so it can be argued, not magic).
const RSS_GROWTH_BOUND_KB: u64 = 32 * 1024;

/// Roundtrip legs the leak proof runs — enough to exhaust a leaking
/// loan/release ledger if one existed.
const LEAK_PROOF_ROUNDS: usize = 10_000;

#[test]
fn the_vendor_lane_survives_ten_thousand_roundtrips_without_leaking() {
    let _serial = VENDOR_SERIAL
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut lane = VendorShmLane::loopback().expect("vendor lane opens on this host");
    lane.connect().expect("connect fresh lane");

    // Warm the lazily-mapped legs (first loan/commit paths) BEFORE the
    // measurement window so the baseline counts a settled process.
    for i in 0..20u8 {
        lane.send(vec![i; 64]).expect("warmup send");
        lane.recv().expect("warmup recv");
    }

    let fd_before = open_fd_count();
    let rss_before = resident_kb();

    for i in 0..LEAK_PROOF_ROUNDS {
        let mut frame = vec![0x7Fu8; 64];
        frame[0] = (i % 251) as u8;
        frame[1] = (i / 251 % 251) as u8;
        lane.send(frame.clone())
            .expect("roundtrip send completes without port exhaustion");
        assert_eq!(lane.recv().expect("roundtrip recv"), frame, "round {i}");
    }

    let fd_after = open_fd_count();
    let rss_after = resident_kb();
    assert_eq!(
        fd_after, fd_before,
        "fd count moved across {LEAK_PROOF_ROUNDS} roundtrips — a loan/release ledger leak"
    );
    assert!(
        rss_after.saturating_sub(rss_before) <= RSS_GROWTH_BOUND_KB,
        "RSS grew {}kB over {LEAK_PROOF_ROUNDS} roundtrips (bound {RSS_GROWTH_BOUND_KB}kB)",
        rss_after.saturating_sub(rss_before)
    );

    lane.close().expect("close the lane");
}
