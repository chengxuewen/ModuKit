//! The memfd leg (D11 / modules/07 §2): write and read a blob through a
//! `memfd_create` descriptor, using the M3-style `extern "C"` shim — no `libc`
//! crate. If the kernel here does not support memfd the test skips green (the
//! M3 dlopen skip discipline).

use modukit_transport::memfd::MemfdBlob;
use modukit_transport::seam::Transport;

#[test]
fn the_memfd_leg_round_trips_a_blob_by_name() {
    let mut blob = match MemfdBlob::create("modukit-m5-probe") {
        Ok(b) => b,
        Err(e) => {
            eprintln!("SKIPPING: memfd_create unavailable here: {e}");
            return;
        }
    };
    blob.connect().expect("a created memfd is connected");
    blob.send(b"cargo-bytes".to_vec()).expect("write the blob");
    assert_eq!(
        blob.recv().expect("read the blob back"),
        b"cargo-bytes".to_vec(),
        "the memfd carries the whole blob it was written"
    );
    assert_eq!(blob.name(), "modukit-m5-probe", "the label round-trips");

    // A single-slot lane: the next write replaces the blob, not appends.
    blob.send(b"second".to_vec()).expect("rewrite");
    assert_eq!(blob.recv().expect("read"), b"second".to_vec());

    blob.close().expect("close the memfd");
    blob.send(b"x".to_vec())
        .expect_err("a closed memfd refuses a write");
}

#[test]
fn the_memfd_leg_is_locked_to_the_transport_trait() {
    fn assert_transport<T: Transport>() {}
    assert_transport::<MemfdBlob>();
}
