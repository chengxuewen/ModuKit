//! Typed error vocabulary (D27: two code spaces, one record; D51: the kernel
//! dictionary is the frozen 14-name code list).
//!
//! Every kernel-side failure is a `KernelError`; `code()` returns the D51
//! SCREAMING_SNAKE name and `hint()` the retry hint that feeds the policy
//! engines (D3 `on_down`, D6 backoff). The prose `Display` is never a
//! decision input (D27).

use std::fmt;

use crate::registry::RegistryError;

/// D27 hint bits — machine-decidable retry guidance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryHint {
    /// Retrying the same operation now can succeed.
    RetryNow,
    /// Retry only after an external condition changes (backoff/cooldown).
    RetryAfter,
    /// Retrying cannot help; the input or the artifact is wrong.
    DoNotRetry,
}

/// Manifest parse/grammar failures (D51 family MANIFEST_INVALID; Display
/// and the per-variant messages live in `manifest.rs` to keep the vocabulary
/// module short).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// TOML shape error (missing field, bad type, ...).
    Toml(String),
    /// `"<count>s|m|h>"` violated.
    BadDuration(String),
    /// `entry.kind` outside {dl, exe, wasm} (05 §1).
    UnknownPlacement(String),
    /// `requires.policy` outside wait(<t>) / failfast / fallback (02 §2).
    BadDepPolicy(String),
    /// `on_down` outside restart-self / block / fail-self (02 §2).
    BadOnDown(String),
    /// `restart.policy` outside none / on-failure (05 §2).
    BadRestartPolicy(String),
    /// `backoff` outside `"<min>..<max>"` (05 §2).
    BadBackoff(String),
}

impl ManifestError {
    /// D51 code for manifest failures (MANIFEST_INVALID family).
    #[must_use]
    pub fn code(&self) -> &'static str {
        "MANIFEST_INVALID"
    }
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ManifestError::Toml(m) => write!(f, "TOML shape error: {m}"),
            ManifestError::BadDuration(s) => write!(
                f,
                "duration must be `<count>s|m|h` (or `<min>..<max>` for backoff), got {s:?}"
            ),
            ManifestError::UnknownPlacement(s) => {
                write!(f, "entry.kind must be dl / exe / wasm, got {s:?} (05 §1)")
            }
            ManifestError::BadDepPolicy(s) => {
                write!(
                    f,
                    "requires policy must be wait(<t>) / failfast / fallback, got {s:?} (02 §2)"
                )
            }
            ManifestError::BadOnDown(s) => {
                write!(
                    f,
                    "on_down must be restart-self / block / fail-self, got {s:?} (02 §2)"
                )
            }
            ManifestError::BadRestartPolicy(s) => {
                write!(
                    f,
                    "restart policy must be none / on-failure, got {s:?} (05 §2)"
                )
            }
            ManifestError::BadBackoff(s) => {
                write!(
                    f,
                    "backoff must be `<min>..<max>` durations, got {s:?} (05 §2)"
                )
            }
        }
    }
}

impl std::error::Error for ManifestError {}

/// The kernel error enum. Variants map onto D51 codes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelError {
    /// Lifecycle guard rejected a move (modules/03 §2 table).
    InvalidTransition { from: String, to: String },
    /// The manifest could not be parsed or its grammar is wrong (D51).
    Manifest(ManifestError),
    /// Registry misuse (D51 vocabulary: NOT_FOUND / CONTRACT_VIOLATION).
    Registry(RegistryError),
    /// `entry.artifact` does not exist at install time (§2 guard).
    ArtifactMissing { path: String },
    /// `dlopen` failed (message carried verbatim from the OS loader).
    LoadFailed { path: String, reason: String },
    /// The entry symbol `modukit_plugin_init` is absent (04 §3).
    SymbolMissing { path: String },
    /// `modukit_plugin_init` returned a null descriptor (04 §3).
    HandshakeNull,
    /// Descriptor `abi_version` is an unknown major (04 §3 -> FAILED).
    AbiMismatch { expected: u32, found: u32 },
    /// Descriptor `size` is smaller than the ABI v1 layout (04 §3).
    DescriptorTooSmall { expected: u32, found: u32 },
    /// Required descriptor function pointers are null (04 §3).
    HandshakeIncomplete { what: &'static str },
    /// A `required = true` dependency was absent at resolve (03 §1, D51).
    DependencyMissing { service: String },
    /// The plugin's `start()` returned a non-zero status (03 §2).
    PluginStartFailed { status: i32 },
    /// A message-shaped call through an ABI service returned an error code
    /// (02 §4 sync boundary; the plugin's own code space is the int).
    ServiceCallFailed { code: i32 },
    /// The placement has no executor installed in this kernel (01 §1 closed
    /// set: each executor answers exactly one kind; wasm stays Stage-5).
    UnsupportedPlacement { kind: String },
    /// No plugin record under this id.
    UnknownPlugin { id: String },
    /// A supervised `exe` child died outside the cooperative stop path
    /// (crash-watch reap or kill-after-timeout teardown; 01 §4: the host
    /// survives and the plugin lands on the FAILED edge). S2-M1.
    ProcessCrashed { id: String, exit: String },
    /// The control envelope carries a foreign wire version (S2-M3 frame
    /// versioning): peer and host disagree on the frame format itself. The
    /// closest existing family was HandshakeIncomplete ("descriptor field ...
    /// is null" — a SHAPE complaint, not a version one), so this one variant
    /// is added; it joins the ABI_MISMATCH code family with the rest of the
    /// disagreement arms.
    WireVersionMismatch { expected: u8, found: u8 },
    /// The child's handshake self-report disagrees with the deploy tier
    /// (D7 strict cross-check, `exe` flavor: silence or a lie is refusal —
    /// the `bundle.rs` liar family, never a panic). S2-M1.
    IdentityMismatch {
        what: &'static str,
        expected: String,
        found: String,
    },
}

impl KernelError {
    /// D51 kernel code name for this failure (machine channel; `Display` is
    /// prose). v0 mappings not covered by the frozen 14 names are marked in
    /// the M3 report for the orchestrator.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            KernelError::UnknownPlugin { .. } | KernelError::LoadFailed { .. } => "NOT_FOUND",
            KernelError::InvalidTransition { .. } => "CONTRACT_VIOLATION",
            KernelError::Manifest(_) | KernelError::ArtifactMissing { .. } => "MANIFEST_INVALID",
            KernelError::Registry(RegistryError::MissingServiceId) => "CONTRACT_VIOLATION",
            KernelError::Registry(_) => "NOT_FOUND",
            KernelError::SymbolMissing { .. }
            | KernelError::HandshakeNull
            | KernelError::AbiMismatch { .. }
            | KernelError::DescriptorTooSmall { .. }
            | KernelError::IdentityMismatch { .. }
            | KernelError::WireVersionMismatch { .. }
            | KernelError::HandshakeIncomplete { .. } => "ABI_MISMATCH",
            KernelError::DependencyMissing { .. } => "DEPENDENCY_MISSING",
            // A supervised crash is transient in the D27 sense: the policy
            // engine may restart it inside the D3 budget; the identity lie
            // is input-shaped. Both live in the contract/handshake families.
            KernelError::ProcessCrashed { .. } => "CONTRACT_VIOLATION",
            KernelError::PluginStartFailed { .. }
            | KernelError::ServiceCallFailed { .. }
            | KernelError::UnsupportedPlacement { .. } => "CONTRACT_VIOLATION",
        }
    }

    /// D27 hint feeding the adjudicated policy engines.
    #[must_use]
    pub fn hint(&self) -> RetryHint {
        match self {
            KernelError::DependencyMissing { .. }
            | KernelError::LoadFailed { .. }
            | KernelError::ProcessCrashed { .. } => RetryHint::RetryAfter,
            // RetryNow stays unused in this slice: every other failure here is
            // input-shaped (fix the artifact/manifest/id), not transient.
            _ => RetryHint::DoNotRetry,
        }
    }
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KernelError::InvalidTransition { from, to } => {
                write!(
                    f,
                    "lifecycle transition {from} -> {to} is not in the rules table"
                )
            }
            KernelError::Manifest(e) => write!(f, "manifest: {e}"),
            KernelError::Registry(e) => write!(f, "registry: {e}"),
            KernelError::ArtifactMissing { path } => write!(f, "artifact not present: {path}"),
            KernelError::LoadFailed { path, reason } => {
                write!(f, "dlopen of {path} failed: {reason}")
            }
            KernelError::SymbolMissing { path } => {
                write!(f, "entry symbol modukit_plugin_init missing in {path}")
            }
            KernelError::HandshakeNull => {
                write!(f, "modukit_plugin_init returned a null descriptor")
            }
            KernelError::AbiMismatch { expected, found } => write!(
                f,
                "ABI mismatch: host speaks v{expected}, plugin reports v{found}"
            ),
            KernelError::DescriptorTooSmall { expected, found } => write!(
                f,
                "PluginAbiV1 descriptor too small: expected {expected} bytes, got {found}"
            ),
            KernelError::HandshakeIncomplete { what } => {
                write!(f, "descriptor field {what} is null")
            }
            KernelError::DependencyMissing { service } => {
                write!(f, "required service missing at resolve: {service}")
            }
            KernelError::PluginStartFailed { status } => {
                write!(f, "plugin start() returned error status {status}")
            }
            KernelError::ServiceCallFailed { code } => {
                write!(f, "service call failed with plugin-side code {code}")
            }
            KernelError::UnsupportedPlacement { kind } => write!(
                f,
                "placement {kind} has no executor in this kernel (dl only until S2-M1 wiring)"
            ),
            KernelError::UnknownPlugin { id } => write!(f, "no plugin record for {id}"),
            KernelError::ProcessCrashed { id, exit } => {
                write!(f, "supervised plugin {id} died outside stop ({exit})")
            }
            KernelError::WireVersionMismatch { expected, found } => write!(
                f,
                "frame wire version mismatch: host speaks v{expected}, peer framed v{found}"
            ),
            KernelError::IdentityMismatch {
                what,
                expected,
                found,
            } => write!(
                f,
                "handshake self-report {what} mismatch: manifest says {expected:?}, artifact says {found:?}"
            ),
        }
    }
}

impl std::error::Error for KernelError {}

impl From<ManifestError> for KernelError {
    fn from(e: ManifestError) -> Self {
        KernelError::Manifest(e)
    }
}

impl From<RegistryError> for KernelError {
    fn from(e: RegistryError) -> Self {
        KernelError::Registry(e)
    }
}
