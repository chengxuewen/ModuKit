//! Typed failures for the contract codegen (no unwrap in lib paths).

use std::fmt;

/// Codegen error vocabulary.
#[derive(Debug)]
pub enum CodegenError {
    /// Filesystem failure while reading fdsets or writing generated files.
    Io(std::io::Error),
    /// A protobuf wire element is truncated or structurally invalid.
    Wire(String),
    /// A group wire type (3/4) appeared - deprecated, unsupported.
    GroupUnsupported(u32),
    /// `(modukit.service_id)` missing on a service, or present but not a
    /// dotted reverse-DNS-shaped id (D9).
    ServiceIdOption(String),
    /// A field carried no explicit number (D12 rule 1) or a required label
    /// (this vocabulary is optional/repeated only).
    FieldRule(String),
    /// An enum lacks its 0-valued sentinel variant (D12 rule 3).
    MissingSentinel(String),
    /// A type reference points outside this contract file (v0 is
    /// self-contained except for the runtime ledger's error record).
    UnresolvedType(String),
    /// The runtime ledger (`org.modukit.runtime`) was not among the inputs;
    /// service skeletons mirror its D27 error record and cannot be emitted
    /// without it.
    RuntimeLedgerMissing,
    /// Usage error for the `protoc-gen-modukit` binary.
    Usage(String),
}

impl CodegenError {
    pub(crate) fn wire(msg: impl Into<String>) -> Self {
        CodegenError::Wire(msg.into())
    }
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenError::Io(e) => write!(f, "io: {e}"),
            CodegenError::Wire(m) => write!(f, "wire decode: {m}"),
            CodegenError::GroupUnsupported(n) => {
                write!(f, "group wire type at field {n}: deprecated, unsupported")
            }
            CodegenError::ServiceIdOption(m) => {
                write!(f, "service option (modukit.service_id): {m} (D9/D13)")
            }
            CodegenError::FieldRule(m) => {
                write!(f, "field rule: {m} (D12 rule 1: numbers are hand-written)")
            }
            CodegenError::MissingSentinel(m) => {
                write!(
                    f,
                    "enum sentinel: {m} (D12 rule 3: a 0 variant is mandatory)"
                )
            }
            CodegenError::UnresolvedType(m) => {
                write!(
                    f,
                    "unresolved type reference: {m} (v0 files stay self-contained)"
                )
            }
            CodegenError::RuntimeLedgerMissing => write!(
                f,
                "org.modukit.runtime ledger missing from the inputs - service \
                 skeletons mirror its D27 error record"
            ),
            CodegenError::Usage(m) => write!(f, "usage: {m}"),
        }
    }
}

impl std::error::Error for CodegenError {}

impl From<std::io::Error> for CodegenError {
    fn from(e: std::io::Error) -> Self {
        CodegenError::Io(e)
    }
}
