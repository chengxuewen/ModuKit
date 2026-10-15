//! Canonical string identities (D9): plugin ids and service ids.
//!
//! Grammar target is reverse-DNS (CLAP id precedent, modules/05 §1); v0 keeps
//! the constructors infallible — the cross-check that would police the
//! grammar lives at M7 (install protocol), so inventing validation here is
//! out of scope.

/// Stable plugin identity from the manifest `id` field (05 §1).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginId(String);

/// Canonical service identity — the reverse-DNS string carried in
/// registration properties under `service.id` (D9, modules/02 §1).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServiceId(String);

impl PluginId {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ServiceId {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for ServiceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
