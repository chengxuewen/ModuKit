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
//! The M6 observability floors ride on top of this: the unified clock
//! (`clock`, D20), the shared event ring (`events`, D17/D19), the edge
//! ledger (`edge_ledger`, D18), gate counters (`counters`, D21), the
//! read-only introspect trio (`introspect`, D17) and the testhost face
//! (`testhost`, D24).
//!
//! Deliberately NOT here (trigger-gated elsewhere): boot DAG parallelism,
//! contract codegen (M4, in the `modukit-codegen` crate), transport seam
//! (M5, in the `modukit-transport` crate); `exe`/`wasm` executors
//! (Stage-2/5). The M7 logical-bundle install protocol (D28/D31) lands
//! in THIS crate: `bundle` (+ `bundle_error`/`bundle_files` helpers) on
//! the std-only `sha256`; the reverse-DNS grammar validator lives in
//! `identity`.

pub mod abi;
pub mod bundle;
mod bundle_error;
mod bundle_files;
pub mod bus;
pub mod clock;
pub mod counters;
pub mod edge_ledger;
pub mod error;
pub mod events;
pub mod executor;
pub mod identity;
pub mod introspect;
pub mod kernel;
pub mod lifecycle;
pub mod logger;
pub mod manifest;
pub mod registry;
mod registry_types;
pub mod sha256;
pub mod testhost;
pub mod topic;
mod wiring;
