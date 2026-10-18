//! Descriptor access-protocol SHAPE + two-stage retrieval (D14, modules/07 §3-4),
//! exercised against a tiny `PoolStub` so the API shape is testable BEFORE any
//! real SHM (a vendor pool enters only behind the neutral `vendor-shm`
//! feature, confined to one module — D76; the default path never links one).
//!
//! Written before `descriptor.rs`/`pool.rs` existed (C0: the first red is an
//! unresolved-import compile error, the same recipe as the bus and the seam).

use modukit_transport::descriptor::{LayoutSpec, SlotHandle};
use modukit_transport::error::TransportError;
use modukit_transport::pool::{PoolStub, View};

fn layout() -> LayoutSpec {
    LayoutSpec::flat(64)
}

/// Two-stage retrieval (D14/07 §3): the descriptor (meta) is a cheap, always
/// available self-sufficient token; the bytes are a lazy second call.
#[test]
fn meta_is_cheap_and_always_there_then_the_view_lazily_carries_the_bytes() {
    let pool = PoolStub::new();
    let handle = pool.allocate(layout());
    pool.write(&handle, b"frame-bytes")
        .expect("write into the allocated slot");

    // Stage 1: metadata only — cheap, self-sufficient, no data copy.
    let meta = pool
        .meta(handle.slot)
        .expect("meta is always available for a live slot");
    assert_eq!(
        meta,
        layout(),
        "the layout round-trips through the meta call"
    );

    // Stage 2: the lazy data accessor.
    let view = pool.view(&handle).expect("a live handle resolves a view");
    assert_eq!(
        view.data().expect("lazy data"),
        b"frame-bytes".to_vec(),
        "the borrowed view carries exactly what was written"
    );
}

/// A descriptor is self-sufficient: the reader computes a plane address by pure
/// addition from the token alone — no per-frame metadata query, no lockstep
/// with the writer (07 §3).
#[test]
fn a_slot_handle_locates_a_plane_by_pure_addition_with_no_pool_query() {
    // Two 16-byte planes laid out inside one 32-byte cell.
    let layout = LayoutSpec::planes(&[
        modukit_transport::descriptor::Plane {
            offset: 0,
            stride: 16,
            height: 1,
        },
        modukit_transport::descriptor::Plane {
            offset: 16,
            stride: 16,
            height: 1,
        },
    ]);
    let handle = SlotHandle {
        slot: 5,
        epoch: 1,
        layout,
    };
    // cell base for slot 5 = 5 * cell_bytes(32) = 160; plane 1 at offset 16 = 176.
    assert_eq!(
        handle.cell_offset(),
        5 * layout.cell_bytes,
        "slot stride arithmetic needs nothing but the token"
    );
    assert_eq!(
        handle.plane_address(1),
        160 + 16,
        "plane address = slot*cell + plane.offset"
    );
}

/// The core M5 promise: recycling a slot bumps its epoch, and a stale token
/// fails with the typed `EPOCH_EXPIRED` code — an ordinary error value, never
/// UB and never a silent read of the new generation's bytes.
#[test]
fn reusing_a_slot_bumps_the_epoch_and_a_stale_handle_is_typed_epoch_expired() {
    let pool = PoolStub::new();
    let first = pool.allocate(layout());
    pool.write(&first, b"generation-one")
        .expect("write gen one");
    let first_epoch = first.epoch;

    // Retire the slot and re-allocate: the pool reuses the same slot index but
    // bumps the generation.
    pool.free(first.slot).expect("free the retired slot");
    let second = pool.allocate(layout());
    pool.write(&second, b"generation-two")
        .expect("write gen two");

    assert_eq!(second.slot, first.slot, "the freed slot is reused");
    assert!(
        second.epoch > first_epoch,
        "reuse bumps the epoch (got {} -> {})",
        first_epoch,
        second.epoch
    );

    // A view resolved against the NEW handle reads the new bytes.
    let live = pool.view(&second).expect("fresh handle resolves");
    assert_eq!(live.data().expect("live data"), b"generation-two".to_vec());

    // The stale handle is refused with EPOCH_EXPIRED, never the new bytes.
    let err = pool
        .view(&first)
        .expect_err("a stale token must not resolve the recycled slot");
    assert!(
        matches!(err, TransportError::StaleEpoch { .. }),
        "expected a typed StaleEpoch, got {err:?}"
    );
    assert_eq!(err.code(), "EPOCH_EXPIRED", "the D51 frozen code name");

    // And writing through a stale handle fails the same way (07 §3: an ordinary
    // error value, never a silent corruption of the live generation).
    let write_err = pool
        .write(&first, b"clobber")
        .expect_err("a stale write is refused");
    assert_eq!(write_err.code(), "EPOCH_EXPIRED");
    assert_eq!(
        pool.view(&second).expect("live").data().expect("live"),
        b"generation-two".to_vec(),
        "the stale write corrupted nothing"
    );
}

/// A live view pins the slot's generation: while any view borrows it, the pool
/// must not silently reassign that slot out from under it. Release pairs with
/// acquisition exactly once (07 §3 "one ledger, three manners", Rust arm).
#[test]
fn a_view_pinned_slot_is_not_reassigned_until_the_view_is_released() {
    let pool = PoolStub::new();
    let handle = pool.allocate(layout());
    pool.write(&handle, b"pinned").expect("write");

    let view = pool.view(&handle).expect("borrow the pinned slot");
    // Freeing a slot with an outstanding view is refused, not queued silently.
    pool.free(handle.slot)
        .expect_err("cannot retire a slot while it is still viewed");

    // Explicit release (C-style pairing parity) frees it.
    view.release().expect("release the view exactly once");
    pool.free(handle.slot).expect("now the slot can retire");
}

/// Dropping a view without an explicit release also returns the hold (Drop is
/// the Rust manner of the same ledger).
#[test]
fn dropping_a_view_releases_its_hold() {
    let pool = PoolStub::new();
    let handle = pool.allocate(layout());
    pool.write(&handle, b"dropped-view").expect("write");
    {
        let _view: View<'_> = pool.view(&handle).expect("borrow");
        // _view leaves scope here; Drop must release the hold.
    }
    pool.free(handle.slot)
        .expect("the dropped view released the slot");
}
