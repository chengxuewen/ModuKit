//! Registry vocabulary types (D4 invariants 2/3/5): the monotonic
//! registration id, the flat property map, and the presence event pair.
//! Split from `registry.rs` to keep both files under the project line cap;
//! re-exported through `crate::registry` so the public paths never moved.

use std::collections::BTreeMap;

use crate::identity::ServiceId;

/// Monotonic registration identity (invariant 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegistrationId(i64);

impl RegistrationId {
    #[must_use]
    pub fn value(self) -> i64 {
        self.0
    }

    /// Construct at a given value. The kernel's own ids come from
    /// `Registry::register` only; this exists so ring/ledger fixtures can
    /// build presence events without registering (M6 tests).
    #[must_use]
    pub const fn at(value: i64) -> Self {
        Self(value)
    }
}

/// Flat property value vocabulary (invariant 3: the string-keyed future
/// host of LDAP vocabulary; year 1 keeps three scalar shapes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

/// Flat property map with case-insensitive keys (invariant 3). Keys are
/// stored folded to lowercase; `get` folds the query the same way.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Properties(BTreeMap<String, PropertyValue>);

impl Properties {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: &str, value: PropertyValue) -> Option<PropertyValue> {
        self.0.insert(key.to_lowercase(), value)
    }

    #[must_use]
    pub fn get(&self, key: &str) -> Option<&PropertyValue> {
        self.0.get(&key.to_lowercase())
    }
}

/// Presence event (invariant 5). `Modified` is reserved-additive, not shipped.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceEvent {
    Registered {
        id: RegistrationId,
        service_id: ServiceId,
        props: Properties,
    },
    Unregistered {
        id: RegistrationId,
        service_id: ServiceId,
        props: Properties,
    },
}

/// Snapshot view of one registration (invariant 5's late-join half).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisteredService {
    pub id: RegistrationId,
    pub service_id: ServiceId,
    /// Reserved storage, year-1 tie-break only (invariant 4).
    pub ranking: i64,
    pub props: Properties,
}
