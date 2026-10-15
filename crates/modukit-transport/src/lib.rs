//! modukit-transport — the narrow transport seam (D11, modules/07).
//!
//! Stage-1 M5 scope only: the local lane verbs, the descriptor access-protocol
//! SHAPE, and test doubles. The kernel owns communication semantics; this crate
//! is the small trait "how bytes and buffers move" hides behind (07 §1). Nothing
//! here links a vendor (D73 — proven by `tests/vendor_lock.rs`).

pub mod descriptor;
pub mod error;
pub mod memfd;
pub mod pool;
pub mod seam;
