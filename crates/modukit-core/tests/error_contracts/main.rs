//! Contract tests for the error-vocabulary surfaces (D84 coverage debt).
//!
//! Every row asserts the adjudicated behavior verbatim: the D51/D85 `code()`
//! mapping, the D86 stable human `hint()` lines, and the `Display` prose arm
//! (exact rendered line, including the carried field values). No
//! touching-lines-without-asserting: one row per variant, pinning text.

mod bundle_error;
mod error_traits;
mod kernel_error;
mod manifest_registry_error;
mod proc_wire_frames;
