//! Topic vocabulary (D10, modules/06 §2): the kernel-owned QoS list.
//!
//! The bus carries opaque payloads (schema-shaped bytes per 04 §2) and never
//! inspects an envelope (D42 / modules/06 §3 — deadbands and the tag envelope
//! are the publisher's discipline, not the kernel's). The QoS is declared at
//! publish; the enum below is the ONLY vocabulary the kernel understands and
//! bridges translate to (D11 §6) — it borrows nothing from any vendor.

use std::fmt;

/// Per-topic delivery quality, declared at publish (modules/06 §2).
///
/// Year-1 placement (06 §5): `LatestValue` and `Lossy(n)` are implemented;
/// `KeepAll` and `Bounded` are accepted in the type so a contract can name
/// them, but the bus refuses them at subscribe-time with a typed deferral
/// rather than silently degrading to latest-value (the D7 liar class).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QoS {
    /// The cockpit signal model: keep only the current value; a late joiner
    /// receives it immediately (the default).
    #[default]
    LatestValue,
    /// Deliver every value ever published. Accepted in the type; refused at
    /// subscribe in year 1 (no persistent/durable queue — 06 §6).
    KeepAll,
    /// Keep the freshest `n` values, dropping the oldest when the ring is
    /// full; the evictions are counted and visible (06 §6, observation only).
    Lossy(u32),
    /// Apply backpressure to the publisher. Accepted in the type; refused at
    /// subscribe in year 1 (enforcement is trigger-gated on a real storm).
    Bounded,
}

impl QoS {
    /// True for the QoS kinds the year-1 bus accepts at publish but refuses
    /// at subscribe (06 §5/§6): `KeepAll` and `Bounded`.
    #[must_use]
    pub const fn is_deferred(self) -> bool {
        matches!(self, QoS::KeepAll | QoS::Bounded)
    }
}

impl fmt::Display for QoS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QoS::LatestValue => f.write_str("latest-value"),
            QoS::KeepAll => f.write_str("keep-all"),
            QoS::Lossy(n) => write!(f, "lossy({n})"),
            QoS::Bounded => f.write_str("bounded"),
        }
    }
}
