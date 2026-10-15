//! modukit-core — Stage-1 in-process kernel (M3 vertical slice).
//!
//! Six-state machine + Failed edge (D3, `lifecycle`), manifest v0 TOML
//! (D7, `manifest`), C+ service registry (D4, `registry`), the executor
//! seam with the `dl` side (D13, `executor` + `abi`), and the `Kernel`
//! that ties them together for `dl` placement only (architecture.md §10).
//! Error vocabulary mirrors the D27 record + D51 code names (`error`);
//! canonical string ids (D9) live in `identity`.
//!
//! Deliberately NOT here (trigger-gated elsewhere): boot DAG parallelism,
//! install/cross-check protocol (M7), contract codegen (M4), bus/transport
//! (M5), observability floors (M6), `exe`/`wasm` executors (Stage-2/5).

pub mod abi;
pub mod error;
pub mod executor;
pub mod identity;
pub mod kernel;
pub mod lifecycle;
pub mod manifest;
pub mod registry;
