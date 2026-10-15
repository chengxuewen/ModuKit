//! modukit-core — Stage-1 in-process kernel (M3 vertical slice).
//!
//! Six-state machine + Failed edge (D3, `lifecycle`), manifest v0 TOML
//! (D7, `manifest`), C+ service registry (D4, `registry`), the executor
//! seam with the `dl` side (D13, `executor` + `abi`), and the `Kernel`
//! that ties them together for `dl` placement only (architecture.md §10).
//! Error vocabulary mirrors the D27 record + D51 code names (`error`);
//! canonical string ids (D9) live in `identity`.
//!
//! `dl` placement only (architecture.md §10). The year-1 in-process topic bus
//! (D10, `bus` + `topic`) is reachable via `Kernel::bus()` and stays a
//! separate channel from the registry.
//!
//! Deliberately NOT here (trigger-gated elsewhere): boot DAG parallelism,
//! install/cross-check protocol (M7), contract codegen (M4), transport seam
//! (M5, in the `modukit-transport` crate), observability floors (M6),
//! `exe`/`wasm` executors (Stage-2/5).

pub mod abi;
pub mod bus;
pub mod error;
pub mod executor;
pub mod identity;
pub mod kernel;
pub mod lifecycle;
pub mod manifest;
pub mod registry;
pub mod topic;
