//! A tiny in-process `PoolStub` (Stage-1 M5): a `Vec`-backed slot allocator that
//! proves the descriptor access-protocol SHAPE before any real SHM exists. A
//! vendor pool backend enters this crate only behind the neutral `vendor-shm`
//! feature, confined to `src/vendor_shm.rs` (D76); this stub stays the
//! default-path pool and the equivalence baseline, gated by
//! `tests/vendor_lock.rs`.
//!
//! The rules it enforces are the 07 §3 ones: reuse bumps the `epoch`, a stale
//! token fails with `EPOCH_EXPIRED` as an ordinary error value (never UB), and
//! a live view pins its generation so the slot is not reassigned under it.

use std::cell::Cell;
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::descriptor::{LayoutSpec, SlotHandle};
use crate::error::TransportError;

/// Epoch a brand-new (never-used) slot is born on. Reuse increments it, so a
/// stale token is always strictly older than the live generation.
const FIRST_EPOCH: u32 = 1;

#[derive(Debug)]
struct Cell0 {
    epoch: u32,
    layout: LayoutSpec,
    bytes: Vec<u8>,
    live: bool,
    views: u32,
}

#[derive(Debug, Default)]
struct Inner {
    slots: Vec<Cell0>,
    /// Slot indices retired and available for reuse (epoch bumps on reuse).
    free: Vec<u64>,
}

/// The in-process stub pool. `&self` verbs over a single mutex — a stand-in for
/// the real mmap'd pool, keeping the API surface honest at hundreds-of-lines
/// scale.
#[derive(Debug, Default)]
pub struct PoolStub {
    inner: Mutex<Inner>,
}

impl PoolStub {
    /// An empty pool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Allocate a slot for `layout`, reusing a retired one (epoch bump) when
    /// available, else appending a fresh cell. Returns the self-sufficient token.
    #[must_use]
    pub fn allocate(&self, layout: LayoutSpec) -> SlotHandle {
        let mut inner = self.lock();
        if let Some(idx) = inner.free.pop() {
            let slot = &mut inner.slots[idx as usize];
            slot.epoch += 1;
            slot.layout = layout;
            slot.bytes.clear();
            slot.live = true;
            slot.views = 0;
            SlotHandle {
                slot: idx,
                epoch: slot.epoch,
                layout,
            }
        } else {
            let idx = inner.slots.len() as u64;
            inner.slots.push(Cell0 {
                epoch: FIRST_EPOCH,
                layout,
                bytes: Vec::new(),
                live: true,
                views: 0,
            });
            SlotHandle {
                slot: idx,
                epoch: FIRST_EPOCH,
                layout,
            }
        }
    }

    /// Write a blob into the slot the token names. A token whose generation has
    /// been recycled (or never existed) is refused — never a silent write into
    /// the live generation (07 §3, "the honest cost of bounded memory").
    ///
    /// # Errors
    ///
    /// `TransportError::StaleEpoch` / `UnknownSlot` on a bad token.
    pub fn write(&self, handle: &SlotHandle, bytes: &[u8]) -> Result<(), TransportError> {
        let mut inner = self.lock();
        let slot = Self::resolve(&mut inner, handle)?;
        slot.layout = handle.layout;
        slot.bytes.clear();
        slot.bytes.extend_from_slice(bytes);
        Ok(())
    }

    /// Stage one of two-stage retrieval (D14): the cheap, always-available
    /// metadata read. No bytes move; a few integers.
    #[must_use]
    pub fn meta(&self, slot: u64) -> Option<LayoutSpec> {
        let inner = self.lock();
        inner
            .slots
            .get(slot as usize)
            .filter(|s| s.live)
            .map(|s| s.layout)
    }

    /// Stage two: borrow the slot's bytes lazily through a `View`. Acquisition
    /// is the `Result`; the data is only copied when `View::data` is called.
    ///
    /// # Errors
    ///
    /// `TransportError::StaleEpoch` / `UnknownSlot` on a bad token.
    pub fn view<'pool>(&'pool self, handle: &SlotHandle) -> Result<View<'pool>, TransportError> {
        let mut inner = self.lock();
        let slot = Self::resolve(&mut inner, handle)?;
        slot.views += 1;
        Ok(View {
            pool: self,
            slot: handle.slot,
            epoch: handle.epoch,
            live: Cell::new(true),
        })
    }

    /// Retire a slot so a later `allocate` may reuse it (bumping its epoch). A
    /// slot still pinned by a live view refuses with `CONTRACT_VIOLATION`
    /// (`TransportError::Busy`) — the release pairing is the caller's duty.
    ///
    /// # Errors
    ///
    /// `TransportError::UnknownSlot` on an absent slot; `TransportError::Busy`
    /// while a view still holds it.
    pub fn free(&self, slot: u64) -> Result<(), TransportError> {
        let mut inner = self.lock();
        let cell = inner
            .slots
            .get_mut(slot as usize)
            .ok_or(TransportError::UnknownSlot { slot })?;
        if cell.views > 0 {
            return Err(TransportError::Busy { slot });
        }
        cell.live = false;
        inner.free.push(slot);
        Ok(())
    }

    /// Shared token validation: in range, live, and epoch-matched. Used by
    /// `write` and `view`; `View::data` re-checks so a slot recycled after the
    /// view was created still fails with `EPOCH_EXPIRED` rather than reading the
    /// new generation.
    fn resolve<'i>(
        inner: &'i mut Inner,
        handle: &SlotHandle,
    ) -> Result<&'i mut Cell0, TransportError> {
        let idx = handle.slot as usize;
        if idx >= inner.slots.len() {
            return Err(TransportError::UnknownSlot { slot: handle.slot });
        }
        // Borrow-split dance: check the immutable fields first, then re-borrow
        // mutably, so no `&mut` alias is formed while reading `epoch`/`live`.
        {
            let cell = &inner.slots[idx];
            if !cell.live || cell.epoch != handle.epoch {
                return Err(TransportError::StaleEpoch {
                    slot: handle.slot,
                    expected: handle.epoch,
                    found: cell.epoch,
                });
            }
        }
        Ok(&mut inner.slots[idx])
    }

    fn read_checked(&self, slot: u64, epoch: u32) -> Result<Vec<u8>, TransportError> {
        let inner = self.lock();
        let cell = inner
            .slots
            .get(slot as usize)
            .ok_or(TransportError::UnknownSlot { slot })?;
        if !cell.live || cell.epoch != epoch {
            return Err(TransportError::StaleEpoch {
                slot,
                expected: epoch,
                found: cell.epoch,
            });
        }
        Ok(cell.bytes.clone())
    }

    fn release_hold(&self, slot: u64, epoch: u32) {
        let mut inner = self.lock();
        if let Some(cell) = inner.slots.get_mut(slot as usize)
            && cell.epoch == epoch
            && cell.views > 0
        {
            cell.views -= 1;
        }
    }
}

/// A borrowed view of a slot (D14 / 07 §3, Rust arm): releasing is the frame's
/// destructor. The C-style `slot_view`/`slot_release` pairing is mirrored by
/// [`View::release`], which consumes the view and does the same bookkeeping once
/// — `Drop` and `release` share one ledger via the `live` flag.
#[derive(Debug)]
pub struct View<'pool> {
    pool: &'pool PoolStub,
    slot: u64,
    epoch: u32,
    live: Cell<bool>,
}

impl View<'_> {
    /// The lazy data half of two-stage retrieval. Re-validates the epoch, so a
    /// slot recycled since this view was created yields `EPOCH_EXPIRED` — an
    /// ordinary error value, never UB (07 §3).
    ///
    /// # Errors
    ///
    /// `TransportError::StaleEpoch` / `UnknownSlot`.
    pub fn data(&self) -> Result<Vec<u8>, TransportError> {
        self.pool.read_checked(self.slot, self.epoch)
    }

    /// The self-sufficient token this view was resolved from (meta).
    #[must_use]
    pub fn descriptor(&self) -> SlotHandle {
        SlotHandle {
            slot: self.slot,
            epoch: self.epoch,
            layout: self
                .pool
                .meta(self.slot)
                .unwrap_or_else(|| LayoutSpec::flat(0)),
        }
    }

    fn release_once(&self) {
        if self.live.replace(false) {
            self.pool.release_hold(self.slot, self.epoch);
        }
    }

    /// Explicit release (parity with the C `slot_release` verb). Consumes the
    /// view; `Drop` becomes a no-op because the `live` flag was cleared.
    ///
    /// # Errors
    ///
    /// Currently infallible; the `Result` mirrors the C verb's status channel.
    pub fn release(self) -> Result<(), TransportError> {
        self.release_once();
        Ok(())
    }
}

impl Drop for View<'_> {
    fn drop(&mut self) {
        self.release_once();
    }
}
