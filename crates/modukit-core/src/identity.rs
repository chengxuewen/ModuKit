//! Canonical string identities (D9): plugin ids, service ids, topic ids,
//! plus the reverse-DNS grammar validator (M7 — the M3 deferral lands
//! here: ONE shared validator, reused by `identity` constructors,
//! `bundle` install and the bus's topic declaration paths; the publish
//! hot path keeps zero added cost).

/// Reverse-DNS id grammar (D9, CLAP precedent, modules/05 §1) — the rule,
/// written down for the record:
///
/// ```text
/// id    = label ( "." label )+
/// label = 'a'..'z' ( 'a'..'z' | '0'..'9' | '-' | '_' )*
/// ```
///
/// At least two dot-separated labels; every label must START with an
/// ASCII lowercase letter (rejects leading digits and uppercase-leading
/// segments per the classic reverse-DNS convention) and continue with
/// lowercase letters, digits, '-' or '_'; no empty labels (so no leading,
/// doubled or trailing dot); total length <= 255.
///
/// v0 minimal-deterministic pick, documented: stricter than the literal
/// "reject uppercase-leading" rule — uppercase is illegal ANYWHERE in a
/// label (kernel-authored and fixture ids are lowercase by construction).
///
/// # Errors
///
/// `IdGrammar` naming the first violated clause.
pub fn validate_reverse_dns(value: &str) -> Result<(), IdGrammar> {
    if value.is_empty() {
        return Err(IdGrammar::Empty);
    }
    if value.len() > 255 {
        return Err(IdGrammar::TooLong);
    }
    let mut labels = 0usize;
    for label in value.split('.') {
        labels += 1;
        let mut chars = label.chars();
        let Some(first) = chars.next() else {
            return Err(IdGrammar::EmptyLabel);
        };
        if first.is_ascii_uppercase() {
            return Err(IdGrammar::LeadingUppercase);
        }
        if first.is_ascii_digit() {
            return Err(IdGrammar::LeadingDigit);
        }
        if !first.is_ascii_lowercase() {
            return Err(IdGrammar::IllegalChar);
        }
        for c in chars {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_') {
                return Err(IdGrammar::IllegalChar);
            }
        }
    }
    if labels < 2 {
        return Err(IdGrammar::SingleLabel);
    }
    Ok(())
}

/// Which reverse-DNS grammar clause an id violated (M7 cross-check).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdGrammar {
    /// The empty string.
    Empty,
    /// Over 255 characters total.
    TooLong,
    /// Reverse-DNS needs at least two dot-separated labels.
    SingleLabel,
    /// Leading, doubled or trailing dot: an empty label.
    EmptyLabel,
    /// A label starting with a digit.
    LeadingDigit,
    /// A label starting with an uppercase letter.
    LeadingUppercase,
    /// A character outside `[a-z0-9-_]`, or uppercase anywhere.
    IllegalChar,
}

impl std::fmt::Display for IdGrammar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdGrammar::Empty => write!(f, "id is empty"),
            IdGrammar::TooLong => write!(f, "id exceeds 255 characters"),
            IdGrammar::SingleLabel => write!(f, "id needs at least two dot-separated labels"),
            IdGrammar::EmptyLabel => {
                write!(f, "id has an empty label (leading/doubled/trailing dot)")
            }
            IdGrammar::LeadingDigit => write!(f, "a label starts with a digit"),
            IdGrammar::LeadingUppercase => write!(f, "a label starts with an uppercase letter"),
            IdGrammar::IllegalChar => {
                write!(f, "a label holds a character outside [a-z0-9-_]")
            }
        }
    }
}

impl std::error::Error for IdGrammar {}

/// Stable plugin identity from the manifest `id` field (05 §1).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PluginId(String);

/// Canonical service identity — the reverse-DNS string carried in
/// registration properties under `service.id` (D9, modules/02 §1).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ServiceId(String);

/// Canonical topic identity — the reverse-DNS string naming a bus topic (D9,
/// modules/06 §2). `new` stays infallible for kernel-authored ids and the
/// publish hot path; call sites parsing untrusted topic strings go through
/// `TopicId::try_new` (the shared M7 validator).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TopicId(String);

impl PluginId {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Grammar-checked constructor (M7): call sites that parse untrusted
    /// id strings use this; `new` stays for kernel-authored ids.
    ///
    /// # Errors
    ///
    /// `IdGrammar` when `value` violates the reverse-DNS rule above.
    pub fn try_new(value: String) -> Result<Self, IdGrammar> {
        validate_reverse_dns(&value)?;
        Ok(Self(value))
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

    /// Grammar-checked constructor (M7); see `PluginId::try_new`.
    ///
    /// # Errors
    ///
    /// `IdGrammar` when `value` violates the reverse-DNS rule.
    pub fn try_new(value: String) -> Result<Self, IdGrammar> {
        validate_reverse_dns(&value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TopicId {
    #[must_use]
    pub fn new(value: String) -> Self {
        Self(value)
    }

    /// Grammar-checked constructor (M7); see `PluginId::try_new`. The bus
    /// publish path is deliberately NOT retrofitted with validation
    /// (hot-path cost) — declaration sites use this door.
    ///
    /// # Errors
    ///
    /// `IdGrammar` when `value` violates the reverse-DNS rule.
    pub fn try_new(value: String) -> Result<Self, IdGrammar> {
        validate_reverse_dns(&value)?;
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TopicId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
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
