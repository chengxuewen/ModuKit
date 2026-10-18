//! modukit-transport — the narrow transport seam (D11, modules/07).
//!
//! Stage-1 M5 scope only: the local lane verbs, the descriptor access-protocol
//! SHAPE, the SCM_RIGHTS fd-passing leg (S2-M3a), and test doubles. The kernel
//! owns communication semantics; this crate is the small trait "how bytes and
//! buffers move" hides behind (07 §1). The DEFAULT feature set links no vendor
//! (D73/D76 — proven by `tests/vendor_lock.rs`; the libc symbols are
//! hand-declared extern shims, not a dependency). The one admitted vendor
//! enters only through the module at the bottom of this file, behind the
//! non-default `vendor-shm` feature.

pub mod descriptor;
pub mod error;
pub mod fdpass;
pub mod memfd;
pub mod pool;
pub mod seam;

// D76 confinement: the single admitted vendor pool lives in exactly this one
// module, reachable only through the neutral `vendor-shm` feature (off by
// default — the default world is the std-only seam above).
#[cfg(feature = "vendor-shm")]
pub mod vendor_shm;
