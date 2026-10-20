//! modukit-codegen - contract codegen v0 (Stage 1, milestone M4).
//!
//! D13 doctrine: the hand-written `.proto` under `contracts/` is the single
//! source; protoc's `fdset` output is the machine ledger; this crate reads
//! that ledger with a HAND-WRITTEN protobuf wire decoder (no prost / no
//! protobuf crate - the decoder itself dogfoods D12 rule 2, decode-and-skip)
//! and emits, per contract:
//!
//! - a Rust skeleton (SERVICE_ID const, dispatch enum, author trait, and a
//!   `modukit::Error`-shaped record mirrored from the runtime ledger);
//! - a C ABI vtable SHADE + `.h` (Stage 1: shape only, "frozen later" per
//!   modules/04 §1 - function tables with i32 status returns, no Rust types
//!   in the header);
//! - a C++ plugin-face skeleton `.hpp` (Stage-4 M1, ruling C++-3 face A -
//!   messages ride the stock protoc C++ output, C++-1).
//!
//! Module map: `wire` (raw varint/LD reader), `model` (decoded subset
//! types), `descriptor` (descriptor.proto subset decode), `emit` (Rust
//! skeleton), `cabi` (vtable shade + header), `cpp` (C++ plugin face),
//! `embed` (C++ embed-face wrapper generator from
//! `contracts/modukit_embed.h`, C++-3 face B), `error` (typed failures).

use std::path::{Path, PathBuf};

pub mod cabi;
pub mod cpp;
pub mod descriptor;
pub mod embed;
pub mod emit;
pub mod error;
pub mod model;
pub mod wire;

pub use descriptor::decode_file_descriptor_set;
pub use embed::emit_embed_hpp;
pub use error::CodegenError;
use model::{File, RUNTIME_PACKAGE};

/// Decode every fdset in `fdsets` and write the generated corpus into
/// `out_dir`. File set per contract package `a.b.c`:
///
/// - `a_b_c.rs` - Rust skeleton (every contract file, the runtime ledger
///   included: its vocabulary is generated output too, same door);
/// - `a_b_c.abi.rs` + `a_b_c.h` + `a_b_c.hpp` - the C ABI shade and the
///   C++ plugin face, only for files that declare services.
///
/// # Errors
///
/// [`CodegenError`] on unreadable inputs, wire failures, contract-rule
/// violations (missing option stamp, unnumbered field, missing sentinel),
/// or io while writing.
pub fn generate(fdsets: &[PathBuf], out_dir: &Path) -> Result<Vec<PathBuf>, CodegenError> {
    let mut files: Vec<File> = Vec::new();
    for path in fdsets {
        let bytes = std::fs::read(path)?;
        files.extend(decode_file_descriptor_set(&bytes)?);
    }
    let ledger = files
        .iter()
        .find(|f| f.package == RUNTIME_PACKAGE)
        .ok_or(CodegenError::RuntimeLedgerMissing)?;

    std::fs::create_dir_all(out_dir)?;
    let mut written = Vec::new();
    for file in &files {
        let is_ledger = file.package == RUNTIME_PACKAGE;
        written.push(write(
            out_dir,
            &format!("{}.rs", file.module_name()),
            &emit::emit_rust(file, (!is_ledger).then_some(ledger))?,
        )?);
        if !file.services.is_empty() {
            written.push(write(
                out_dir,
                &format!("{}.abi.rs", file.module_name()),
                &cabi::emit_cabi_rust(file)?,
            )?);
            written.push(write(
                out_dir,
                &format!("{}.h", file.module_name()),
                &cabi::emit_cabi_header(file)?,
            )?);
            written.push(write(
                out_dir,
                &format!("{}.hpp", file.module_name()),
                &cpp::emit_cpp_header(file)?,
            )?);
        }
    }
    Ok(written)
}

fn write(out_dir: &Path, name: &str, content: &str) -> Result<PathBuf, CodegenError> {
    let path = out_dir.join(name);
    std::fs::write(&path, content)?;
    Ok(path)
}
