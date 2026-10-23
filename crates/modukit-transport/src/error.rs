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

#[cfg(test)]
mod tests {
    use super::TransportError;

    /// Every variant names its D51 dictionary entry — the vocabulary claim
    /// of the module header, asserted verbatim.
    #[test]
    fn each_variant_names_its_d51_code() {
        assert_eq!(TransportError::Closed.code(), "CARRIER_UNREACHABLE");
        assert_eq!(
            TransportError::Backend {
                syscall: "recvmsg",
                reason: "x".into()
            }
            .code(),
            "CARRIER_UNREACHABLE",
            "a backend refusal never invents a code"
        );
        assert_eq!(
            TransportError::StaleEpoch {
                slot: 1,
                expected: 2,
                found: 3
            }
            .code(),
            "EPOCH_EXPIRED"
        );
        assert_eq!(TransportError::UnknownSlot { slot: 4 }.code(), "NOT_FOUND");
        assert_eq!(
            TransportError::Busy { slot: 5 }.code(),
            "CONTRACT_VIOLATION"
        );
    }

    /// Display is the prose channel: each arm renders its documented sentence
    /// with the fields interpolated (the lanes' tests only ever read `code()`).
    #[test]
    fn display_renders_each_variant_prose() {
        assert_eq!(
            TransportError::Closed.to_string(),
            "the transport lane is closed"
        );
        assert_eq!(
            TransportError::Backend {
                syscall: "sendmsg",
                reason: "Broken pipe".into()
            }
            .to_string(),
            "transport backend: sendmsg failed: Broken pipe"
        );
        let prose = TransportError::StaleEpoch {
            slot: 7,
            expected: 1,
            found: 2,
        }
        .to_string();
        assert!(
            prose.contains("slot 7 resolved at epoch 1 but the pool is now at epoch 2"),
            "{prose}"
        );
        assert!(prose.contains("EPOCH_EXPIRED"), "{prose}");
        assert_eq!(
            TransportError::UnknownSlot { slot: 9 }.to_string(),
            "descriptor references unknown slot 9"
        );
        let busy = TransportError::Busy { slot: 10 }.to_string();
        assert!(
            busy.contains("slot 10 is still viewed; release it before retiring"),
            "{busy}"
        );
    }

    /// The std `Error` impl rides `Display` — a boxed seam error still speaks.
    #[test]
    fn the_std_error_impl_carries_the_display_string() {
        let err: &dyn std::error::Error = &TransportError::Closed;
        assert_eq!(err.to_string(), "the transport lane is closed");
    }
}
