//! The descriptor access-protocol SHAPE (D14, modules/07 §3): a token that is
//! self-sufficient — every descriptor-bearing envelope carries `slot`, `epoch`
//! and the full `layout`, so the reader computes addresses by pure addition and
//! never queries the writer per frame (07 §3 "no per-frame metadata query, no
//! lockstep with the writer ever").
//!
//! Hand-written here as the shape the M4.5+ codegen will mirror from a `.proto`.
//! It is NOT a `.proto`-in-boundary type yet (D13 pipeline boundary 1): kernel
//! shape first, contract door later); pixels never enter `.proto` (04 §2, D15).

/// One plane's geometry: a byte offset into the cell, a row stride, and a row
/// count. Mirrors the `local{slot,epoch,layout}` arm of the cargo oneof (D15).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plane {
    /// Byte offset of the plane from the start of its cell.
    pub offset: u64,
    /// Bytes per row.
    pub stride: u32,
    /// Number of rows.
    pub height: u32,
}

/// The largest plane count a `LayoutSpec` carries inline. Beyond this a real
/// pool backend (Stage-2) would side-table the geometry; year-1 keeps a fixed
/// array so `LayoutSpec` stays `Copy` and self-sufficient.
pub const MAX_PLANES: usize = 4;

/// A self-sufficient buffer layout: the cell size and the plane geometry inside
/// it. `Copy` so it rides a descriptor token without a heap hop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutSpec {
    /// Bytes one slot occupies; a slot's base is `slot * cell_bytes`.
    pub cell_bytes: u64,
    /// Planes laid out inside a cell (unused tail entries are all-zero).
    pub planes: [Plane; MAX_PLANES],
    /// How many `planes` entries are meaningful (≤ `MAX_PLANES`).
    pub plane_count: u32,
}

impl LayoutSpec {
    /// A single-plane flat buffer of `total` bytes.
    #[must_use]
    pub const fn flat(total: u64) -> Self {
        let mut planes = [Plane {
            offset: 0,
            stride: 0,
            height: 0,
        }; MAX_PLANES];
        planes[0] = Plane {
            offset: 0,
            // A flat buffer is one `total`-byte row.
            stride: total as u32,
            height: 1,
        };
        Self {
            cell_bytes: total,
            planes,
            plane_count: 1,
        }
    }

    /// A multi-plane layout; the cell size is the highest byte the planes touch
    /// (`max(offset + stride * height)`), the geometry a reader uses to compute
    /// plane addresses. Truncates to `MAX_PLANES`.
    #[must_use]
    pub fn planes(planes: &[Plane]) -> Self {
        let mut arr = [Plane {
            offset: 0,
            stride: 0,
            height: 0,
        }; MAX_PLANES];
        let count = planes.len().min(MAX_PLANES);
        arr[..count].copy_from_slice(&planes[..count]);
        let cell_bytes = planes.iter().fold(0u64, |acc, p| {
            acc.max(p.offset + u64::from(p.stride) * u64::from(p.height))
        });
        Self {
            cell_bytes,
            planes: arr,
            plane_count: count as u32,
        }
    }

    /// Byte offset of `plane` inside its cell (0 when out of range).
    #[must_use]
    pub const fn plane_offset(&self, plane: u32) -> u64 {
        let idx = plane as usize;
        if idx < self.plane_count as usize {
            self.planes[idx].offset
        } else {
            0
        }
    }
}

/// The self-sufficient descriptor token (D14/07 §3): enough to locate bytes with
/// no pool round-trip. `epoch` makes a recycled slot detectable (→ `View`
/// returns `EPOCH_EXPIRED`, never reads the new generation — 07 §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotHandle {
    /// Slot index in the pool.
    pub slot: u64,
    /// Generation the token was resolved against.
    pub epoch: u32,
    /// The full geometry, carried so the reader needs nothing else.
    pub layout: LayoutSpec,
}

impl SlotHandle {
    /// Cell base = `slot * cell_bytes` — pure arithmetic on the token (07 §3).
    #[must_use]
    pub const fn cell_offset(self) -> u64 {
        self.slot * self.layout.cell_bytes
    }

    /// Address of `plane` from the pool base: `slot*cell + plane.offset`.
    #[must_use]
    pub const fn plane_address(self, plane: u32) -> u64 {
        self.cell_offset() + self.layout.plane_offset(plane)
    }
}
