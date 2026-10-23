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

/// The one guard `create` really carries before the syscall: an interior NUL
/// is refused as a typed Backend error (the CString boundary). There is no
/// size guard or seal-name collision check in this source — memfd labels are
/// not unique (they only show up in /proc/self/fd), so the sibling test below
/// pins that a second create under the same label simply succeeds.
#[test]
fn an_interior_nul_in_the_label_is_refused_before_the_syscall() {
    let err = match MemfdBlob::create("bad\0name") {
        Ok(_) => panic!("the CString boundary must refuse"),
        Err(e) => e,
    };
    assert_eq!(err.code(), "CARRIER_UNREACHABLE", "{err}");
    assert!(
        format!("{err}").contains("interior NUL"),
        "the prose names the refusal, got: {err}"
    );
}

/// Closed-state arms: close is idempotent, the descriptor verb reports Closed
/// afterwards, and Transport::connect on a closed blob recreates the memfd
/// under the same label (the re-creation arm of the trait impl).
#[test]
fn a_closed_blob_reports_closed_and_reconnect_recreates_it() {
    let mut blob = match MemfdBlob::create("modukit-m5-reconnect") {
        Ok(b) => b,
        Err(e) => {
            eprintln!("SKIPPING: memfd_create unavailable here: {e}");
            return;
        }
    };
    blob.send(b"first".to_vec()).expect("write");
    blob.close().expect("close");
    blob.close().expect("close is idempotent");
    let err = blob.raw_fd().expect_err("a closed blob has no descriptor");
    assert_eq!(err.code(), "CARRIER_UNREACHABLE", "{err}");
    assert!(blob.recv().is_err(), "a closed blob refuses a read");

    blob.connect()
        .expect("connect recreates the memfd under the same label");
    assert!(blob.raw_fd().expect("open again") >= 0, "a fresh fd");
    // A second blob sharing the label is legal: the name is not a key.
    let _twin = MemfdBlob::create("modukit-m5-reconnect")
        .expect("memfd labels are not unique; a same-name create succeeds");
    blob.send(b"second".to_vec()).expect("write after recreate");
    assert_eq!(blob.recv().expect("read"), b"second".to_vec());
}

/// Zero-length edge: the single-slot lane accepts an empty blob and reads
/// back exactly nothing (truncate-to-zero is the guard's whole behaviour).
#[test]
fn a_zero_length_blob_round_trips_as_empty() {
    let mut blob = match MemfdBlob::create("modukit-m5-empty") {
        Ok(b) => b,
        Err(e) => {
            eprintln!("SKIPPING: memfd_create unavailable here: {e}");
            return;
        }
    };
    blob.send(vec![]).expect("write an empty blob");
    assert_eq!(blob.recv().expect("read"), Vec::<u8>::new());
}
