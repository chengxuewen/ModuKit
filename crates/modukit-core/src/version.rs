//! Version ranges for `requires` (plugin-management-plan S2, ruling S2-3 A).
//!
//! A small, dependency-free comparator-range parser: comma-separated
//! comparators like `>=1.2,<2` (operators `>=`, `<=`, `>`, `<`, `=`). Versions
//! are dot-separated numeric segments compared numerically with zero-padding
//! (`1.2` == `1.2.0`). A range matches when EVERY comparator holds. The
//! provider's version rides the `service.version` property (module 05 §3
//! convention); cascade-start of a matching provider is deliberately NOT here
//! (trigger-gated, plan §8 - the CTK Require-Plugin behavior is a later ruling).

/// A parsed version range (a conjunction of comparators).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRange {
    comparators: Vec<Comparator>,
}

/// Parse failure (deterministic, never a panic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionError(pub String);

impl std::fmt::Display for VersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid version range: {}", self.0)
    }
}

impl std::error::Error for VersionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Gt,
    Ge,
    Lt,
    Le,
    Eq,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Comparator {
    op: Op,
    version: Vec<u64>,
}

fn parse_version(text: &str) -> Result<Vec<u64>, VersionError> {
    if text.is_empty() {
        return Err(VersionError("empty version".into()));
    }
    text.split('.')
        .map(|seg| {
            seg.parse::<u64>()
                .map_err(|_| VersionError(format!("non-numeric segment '{seg}'")))
        })
        .collect()
}

fn cmp_versions(a: &[u64], b: &[u64]) -> std::cmp::Ordering {
    let n = a.len().max(b.len());
    for i in 0..n {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        match x.cmp(&y) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

impl VersionRange {
    /// Parse a comma-separated comparator range (e.g. `>=1.2,<2`).
    ///
    /// # Errors
    ///
    /// `VersionError` on empty input, an unknown operator, or a non-numeric
    /// version segment.
    pub fn parse(text: &str) -> Result<Self, VersionError> {
        let mut comparators = Vec::new();
        for part in text.split(',') {
            let part = part.trim();
            if part.is_empty() {
                return Err(VersionError("empty comparator".into()));
            }
            let (op, rest) = if let Some(r) = part.strip_prefix(">=") {
                (Op::Ge, r)
            } else if let Some(r) = part.strip_prefix("<=") {
                (Op::Le, r)
            } else if let Some(r) = part.strip_prefix('>') {
                (Op::Gt, r)
            } else if let Some(r) = part.strip_prefix('<') {
                (Op::Lt, r)
            } else if let Some(r) = part.strip_prefix('=') {
                (Op::Eq, r)
            } else {
                (Op::Eq, part)
            };
            comparators.push(Comparator {
                op,
                version: parse_version(rest.trim())?,
            });
        }
        if comparators.is_empty() {
            return Err(VersionError("no comparators".into()));
        }
        Ok(Self { comparators })
    }

    /// Does the concrete version string satisfy every comparator? A malformed
    /// concrete version simply does not match (it never panics).
    #[must_use]
    pub fn matches(&self, version: &str) -> bool {
        let Ok(v) = parse_version(version) else {
            return false;
        };
        self.comparators.iter().all(|c| {
            let ord = cmp_versions(&v, &c.version);
            match c.op {
                Op::Gt => ord == std::cmp::Ordering::Greater,
                Op::Ge => ord != std::cmp::Ordering::Less,
                Op::Lt => ord == std::cmp::Ordering::Less,
                Op::Le => ord != std::cmp::Ordering::Greater,
                Op::Eq => ord == std::cmp::Ordering::Equal,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_conjunction_and_padding() {
        let r = VersionRange::parse(">=1.2,<2").unwrap();
        assert!(r.matches("1.2"));
        assert!(r.matches("1.2.0")); // zero-padding equality
        assert!(r.matches("1.9.9"));
        assert!(!r.matches("2.0"));
        assert!(!r.matches("1.1"));
        assert!(!r.matches("0.9"));
    }

    #[test]
    fn single_operators() {
        assert!(VersionRange::parse(">1.0").unwrap().matches("1.0.1"));
        assert!(!VersionRange::parse(">1.0").unwrap().matches("1.0"));
        assert!(VersionRange::parse("<=1.0").unwrap().matches("1.0"));
        assert!(VersionRange::parse("=1.2.3").unwrap().matches("1.2.3"));
        assert!(VersionRange::parse("1.2.3").unwrap().matches("1.2.3")); // bare = exact
    }

    #[test]
    fn malformed_ranges_redden() {
        for bad in ["", ">=1.2,", ">=abc", ">", ">=1.2,,<2", "~=1.0"] {
            assert!(VersionRange::parse(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn malformed_concrete_version_never_matches() {
        let r = VersionRange::parse(">=1.0").unwrap();
        assert!(!r.matches("not-a-version"));
        assert!(!r.matches(""));
    }
}
