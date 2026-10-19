//! The bundle install protocol's failure type, kept in its own module so
//! `bundle.rs` stays under the file-size cap. Re-exported as
//! `modukit_core::bundle::BundleError`.

use crate::error::{KernelError, ManifestError};
use crate::identity::IdGrammar;

/// Bundle-side failure. Standalone enum (BusError precedent); `code()`
/// reuses ONLY the frozen D51 names — the gaps (content integrity,
/// duplicate identity, io) map to the closest frozen bucket and are
/// reported as AMBIGUITIES for the vocabulary pass, never invented codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BundleError {
    /// A deploy-config `[install]` field failed its cross-check.
    Config(String),
    /// The `modukit.toml` did not parse as a manifest.
    Manifest(ManifestError),
    /// A bundle-TOML field outside the ManifestError grammar (checklist
    /// shape, member paths, version-as-path-component).
    ManifestField(String),
    /// Reverse-DNS grammar violation on the id or a declared service id
    /// (the shared M7 validator).
    Grammar { value: String, reason: IdGrammar },
    /// Filesystem refusal (AMBIGUITY: no io-shaped name in the frozen D51
    /// list; rides NOT_FOUND until the vocabulary pass).
    Io { path: String, reason: String },
    /// A listed member file is absent from the staged bundle.
    MissingFile { path: String },
    /// Content hash disagreement (AMBIGUITY: D51 has no INTEGRITY_FAILED;
    /// rides CONTRACT_VIOLATION — the frozen list offers no better fit).
    HashMismatch {
        path: String,
        expected: String,
        found: String,
    },
    /// Declared size disagrees with the file.
    SizeMismatch {
        path: String,
        expected: u64,
        found: u64,
    },
    /// The artifact has no hash-manifest entry: the integrity envelope
    /// would leave the loadable bytes unclaimed.
    ArtifactNotListed { path: String },
    /// A bundle with this manifest id is already installed (D34).
    DuplicateId { id: String },
    /// D7 deploy <-> self-report disagreement (or a silent descriptor).
    CrossCheck { what: String },
    /// The executor/handshake door refused the staged artifact
    /// (ABI_MISMATCH and friends flow through with their own code).
    Kernel(KernelError),
    /// Uninstall asked for an id that is not installed.
    NotInstalled { id: String },
}

impl BundleError {
    /// D51 code name (machine channel; `Display` is prose, never a
    /// decision input — D27).
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            BundleError::Config(_) => "SCHEMA_INVALID",
            BundleError::Manifest(_)
            | BundleError::ManifestField(_)
            | BundleError::Grammar { .. } => "MANIFEST_INVALID",
            BundleError::Io { .. } | BundleError::MissingFile { .. } => "NOT_FOUND",
            BundleError::HashMismatch { .. }
            | BundleError::SizeMismatch { .. }
            | BundleError::ArtifactNotListed { .. } => "INTEGRITY_FAILED",
            BundleError::DuplicateId { .. } | BundleError::CrossCheck { .. } => {
                "CONTRACT_VIOLATION"
            }
            BundleError::Kernel(err) => err.code(),
            BundleError::NotInstalled { .. } => "NOT_FOUND",
        }
    }
}

impl BundleError {
    /// The hearing-owed one-line human hint (M8 deferral, S2-M5b ruling 5-A):
    /// what the installer user should do next, per variant. Stable prose, not
    /// an API surface - codes stay the machine contract.
    #[must_use]
    pub fn hint(&self) -> &'static str {
        match self {
            BundleError::Config(_) => "check the installer arguments and the bundle layout root",
            BundleError::Manifest(_)
            | BundleError::ManifestField(_)
            | BundleError::Grammar { .. } => {
                "fix modukit.toml: id/version/entry fields must match the contract grammar"
            }
            BundleError::Io { .. } | BundleError::MissingFile { .. } => {
                "verify the bundle contents on disk; a listed member is unreadable"
            }
            BundleError::HashMismatch { .. } | BundleError::SizeMismatch { .. } => {
                "the bundle was tampered or corrupted in transit - do not install; rebuild and re-sign the bundle"
            }
            BundleError::ArtifactNotListed { .. } => {
                "list the entry artifact in the [[files]] checklist, then rebuild the bundle"
            }
            BundleError::DuplicateId { .. } => {
                "a bundle with this id is already installed - uninstall it first or bump the version"
            }
            BundleError::CrossCheck { .. } => {
                "the artifact self-report disagrees with modukit.toml - rebuild the bundle from matching sources"
            }
            BundleError::Kernel(_) => {
                "the artifact failed the door probe - see the kernel error for the specific stage"
            }
            BundleError::NotInstalled { .. } => {
                "nothing is installed under this id - install the bundle first"
            }
        }
    }
}

impl std::fmt::Display for BundleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BundleError::Config(m) => write!(f, "install config: {m}"),
            BundleError::Manifest(e) => write!(f, "bundle manifest: {e}"),
            BundleError::ManifestField(m) => write!(f, "bundle TOML: {m}"),
            BundleError::Grammar { value, reason } => {
                write!(f, "id {value:?} violates the reverse-DNS grammar: {reason}")
            }
            BundleError::Io { path, reason } => write!(f, "{path}: {reason}"),
            BundleError::MissingFile { path } => write!(f, "listed member file missing: {path}"),
            BundleError::HashMismatch {
                path,
                expected,
                found,
            } => write!(
                f,
                "hash mismatch for {path}: manifest says {expected}, file says {found}"
            ),
            BundleError::SizeMismatch {
                path,
                expected,
                found,
            } => write!(
                f,
                "size mismatch for {path}: manifest says {expected}, file says {found}"
            ),
            BundleError::ArtifactNotListed { path } => {
                write!(f, "artifact {path} carries no hash-manifest entry")
            }
            BundleError::DuplicateId { id } => {
                write!(
                    f,
                    "plugin id {id} is already installed (D34: one identity per host)"
                )
            }
            BundleError::CrossCheck { what } => write!(f, "D7 cross-check failed: {what}"),
            BundleError::Kernel(err) => write!(f, "artifact door: {err}"),
            BundleError::NotInstalled { id } => write!(f, "no installed bundle for id {id}"),
        }
    }
}

impl std::error::Error for BundleError {}

impl From<ManifestError> for BundleError {
    fn from(e: ManifestError) -> Self {
        BundleError::Manifest(e)
    }
}
