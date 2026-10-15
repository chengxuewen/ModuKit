//! The seam's typed failure set. Every variant maps to an EXISTING D51 kernel
//! code (the frozen 14 in `org.modukit.runtime.proto`) — the seam never
//! invents a code string; that would be a D51 hearing, not a milestone call.

use std::fmt;

/// One cross-seam failure. `code()` names the D51 dictionary entry so a runner
/// on the other side of the ABI sees the same vocabulary as `KernelError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// The lane is closed (either end) or its rendezvous is gone — the bytes
    /// have nowhere to travel. Maps to the runtime `CARRIER_UNREACHABLE`.
    Closed,
    /// An OS-level failure in a named syscall; the message is the OS string.
    /// Maps to `CARRIER_UNREACHABLE` (the carrier leg did not move the bytes).
    Backend {
        syscall: &'static str,
        reason: String,
    },
    /// A descriptor referenced a slot whose epoch has advanced — the view was
    /// resolved against a generation that has since been recycled. Maps to the
    /// runtime `EPOCH_EXPIRED` (an ordinary error value, never UB — 07 §3).
    StaleEpoch {
        slot: u64,
        expected: u32,
        found: u32,
    },
    /// A descriptor referenced a slot the pool never handed out. Maps to the
    /// runtime `NOT_FOUND`.
    UnknownSlot { slot: u64 },
    /// Retiring a slot a live view still pins — a release-pairing break
    /// (07 §3 one-ledger). Maps to the contract-domain `CONTRACT_VIOLATION`.
    Busy { slot: u64 },
}

impl TransportError {
    /// D51 kernel code name for this failure. `Display` is the prose channel.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            TransportError::Closed | TransportError::Backend { .. } => "CARRIER_UNREACHABLE",
            TransportError::StaleEpoch { .. } => "EPOCH_EXPIRED",
            TransportError::UnknownSlot { .. } => "NOT_FOUND",
            TransportError::Busy { .. } => "CONTRACT_VIOLATION",
        }
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportError::Closed => write!(f, "the transport lane is closed"),
            TransportError::Backend { syscall, reason } => {
                write!(f, "transport backend: {syscall} failed: {reason}")
            }
            TransportError::StaleEpoch {
                slot,
                expected,
                found,
            } => write!(
                f,
                "slot {slot} resolved at epoch {expected} but the pool is now at epoch {found} \
                 (EPOCH_EXPIRED: the honest cost of bounded memory, 07 §3)"
            ),
            TransportError::UnknownSlot { slot } => {
                write!(f, "descriptor references unknown slot {slot}")
            }
            TransportError::Busy { slot } => write!(
                f,
                "slot {slot} is still viewed; release it before retiring (07 §3)"
            ),
        }
    }
}

impl std::error::Error for TransportError {}
