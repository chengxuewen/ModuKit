//! RFC1960 service-filter subset (plugin-management-plan S1, ruling A2).
//!
//! A hand-rolled, dependency-free parser + matcher for the OSGi-style filter
//! grammar the three reference frameworks all speak (CppMicroServices
//! `LDAPExpr`, celix `filter.c`, CTK `ctkLDAPExpr`). The subset covers the
//! shapes real queries use:
//!
//! ```text
//! filter   = '(' item ')'
//! item     = '&' filter+ | '|' filter+ | '!' filter | simple
//! simple   = attr '=' value        (equality; '*' wildcards = substring)
//!          | attr '=' '*'          (presence test)
//!          | attr '>=' value       (greater-or-equal)
//!          | attr '<=' value       (less-or-equal)
//!          | attr '~=' value       (approximate = case-insensitive equality)
//! ```
//!
//! Semantics follow the registry's property vocabulary (`registry_types`):
//! keys are case-insensitive (the map folds them); an `Int` property compares
//! numerically, `Str`/`Bool` compare as strings. A filter that names a property
//! the service never declared simply does not match - it never panics (the
//! liar-fixture family). Full RFC1960 escaping and substring initial/final
//! segments are deferred until a real consumer needs them (ruling A2).

use crate::registry_types::{Properties, PropertyValue};

/// A parsed filter tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Filter {
    /// Conjunction: every child must match (non-empty).
    And(Vec<Filter>),
    /// Disjunction: at least one child must match (non-empty).
    Or(Vec<Filter>),
    /// Negation.
    Not(Box<Filter>),
    /// A leaf comparison or presence test.
    Item(Item),
}

/// A leaf test against one property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    /// `(attr=*)` - the property exists.
    Present(String),
    /// `(attr=value)` - equality; `value` may carry `*` wildcards.
    Eq(String, String),
    /// `(attr>=value)`.
    Ge(String, String),
    /// `(attr<=value)`.
    Le(String, String),
    /// `(attr~=value)` - case-insensitive equality.
    Approx(String, String),
}

/// Parse failure (deterministic, never a panic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterError(pub String);

impl std::fmt::Display for FilterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid service filter: {}", self.0)
    }
}

impl std::error::Error for FilterError {}

impl Filter {
    /// Parse a filter string. The whole string must be one `( ... )` form.
    ///
    /// # Errors
    ///
    /// `FilterError` on any malformed input (empty, unbalanced, unknown
    /// operator, empty attribute, trailing garbage).
    pub fn parse(input: &str) -> Result<Filter, FilterError> {
        let mut p = Parser {
            bytes: input.as_bytes(),
            pos: 0,
        };
        let f = p.filter()?;
        p.skip_ws();
        if p.pos != p.bytes.len() {
            return Err(FilterError("trailing characters after filter".into()));
        }
        Ok(f)
    }

    /// Does this filter match the given property map?
    #[must_use]
    pub fn matches(&self, props: &Properties) -> bool {
        match self {
            Filter::And(children) => children.iter().all(|c| c.matches(props)),
            Filter::Or(children) => children.iter().any(|c| c.matches(props)),
            Filter::Not(inner) => !inner.matches(props),
            Filter::Item(item) => item.matches(props),
        }
    }
}

impl Item {
    fn matches(&self, props: &Properties) -> bool {
        let (attr, op, value) = match self {
            Item::Present(a) => return props.get(a).is_some(),
            Item::Eq(a, v) => (a, Op::Eq, v),
            Item::Ge(a, v) => (a, Op::Ge, v),
            Item::Le(a, v) => (a, Op::Le, v),
            Item::Approx(a, v) => (a, Op::Approx, v),
        };
        let Some(pv) = props.get(attr) else {
            return false; // undeclared property never matches (liar family)
        };
        match pv {
            PropertyValue::Int(n) => {
                let Ok(want) = value.trim().parse::<i64>() else {
                    return false;
                };
                match op {
                    Op::Eq => *n == want,
                    Op::Ge => *n >= want,
                    Op::Le => *n <= want,
                    Op::Approx => *n == want,
                }
            }
            PropertyValue::Str(s) => match op {
                Op::Eq => wildcard_match(value, s),
                Op::Ge => s.as_str() >= value.as_str(),
                Op::Le => s.as_str() <= value.as_str(),
                Op::Approx => s.eq_ignore_ascii_case(value),
            },
            PropertyValue::Bool(b) => {
                let s = if *b { "true" } else { "false" };
                match op {
                    Op::Eq => wildcard_match(value, s),
                    Op::Approx => s.eq_ignore_ascii_case(value),
                    _ => false,
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Op {
    Eq,
    Ge,
    Le,
    Approx,
}

/// Substring wildcard match: `*` matches any run (including empty). A pattern
/// with no `*` is plain equality. The first/last segments anchor to the
/// string's start/end unless the pattern begins/ends with `*`.
fn wildcard_match(pattern: &str, text: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == text;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut rest = text;
    for (i, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }
        if i == 0 {
            // anchored at the start
            let Some(stripped) = rest.strip_prefix(part) else {
                return false;
            };
            rest = stripped;
        } else if i == parts.len() - 1 {
            // anchored at the end
            let Some(stripped) = rest.strip_suffix(part) else {
                return false;
            };
            rest = stripped;
        } else {
            let Some(idx) = rest.find(part) else {
                return false;
            };
            rest = &rest[idx + part.len()..];
        }
    }
    true
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, c: u8) -> Result<(), FilterError> {
        self.skip_ws();
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(FilterError(format!(
                "expected '{}' at byte {}",
                c as char, self.pos
            )))
        }
    }

    fn filter(&mut self) -> Result<Filter, FilterError> {
        self.expect(b'(')?;
        self.skip_ws();
        let f = match self.peek() {
            Some(b'&') => {
                self.pos += 1;
                Filter::And(self.list()?)
            }
            Some(b'|') => {
                self.pos += 1;
                Filter::Or(self.list()?)
            }
            Some(b'!') => {
                self.pos += 1;
                Filter::Not(Box::new(self.filter()?))
            }
            _ => Filter::Item(self.simple()?),
        };
        self.expect(b')')?;
        Ok(f)
    }

    fn list(&mut self) -> Result<Vec<Filter>, FilterError> {
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            if self.peek() != Some(b'(') {
                break;
            }
            out.push(self.filter()?);
        }
        if out.is_empty() {
            return Err(FilterError("empty '&'/'|' filter list".into()));
        }
        Ok(out)
    }

    fn simple(&mut self) -> Result<Item, FilterError> {
        self.skip_ws();
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == b'=' || c == b'>' || c == b'<' || c == b'~' || c == b')' {
                break;
            }
            self.pos += 1;
        }
        let attr = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| FilterError("non-utf8 attribute".into()))?
            .trim()
            .to_string();
        if attr.is_empty() {
            return Err(FilterError("empty attribute name".into()));
        }
        // operator
        let op = match self.peek() {
            Some(b'=') => {
                self.pos += 1;
                Op::Eq
            }
            Some(b'>') => {
                self.pos += 1;
                self.expect(b'=')?;
                Op::Ge
            }
            Some(b'<') => {
                self.pos += 1;
                self.expect(b'=')?;
                Op::Le
            }
            Some(b'~') => {
                self.pos += 1;
                self.expect(b'=')?;
                Op::Approx
            }
            _ => return Err(FilterError("missing comparison operator".into())),
        };
        // value: up to the closing ')'
        let vstart = self.pos;
        while let Some(c) = self.peek() {
            if c == b')' {
                break;
            }
            self.pos += 1;
        }
        let value = std::str::from_utf8(&self.bytes[vstart..self.pos])
            .map_err(|_| FilterError("non-utf8 value".into()))?
            .to_string();
        if value.is_empty() {
            return Err(FilterError("empty comparison value".into()));
        }
        Ok(match op {
            Op::Eq if value == "*" => Item::Present(attr),
            Op::Eq => Item::Eq(attr, value),
            Op::Ge => Item::Ge(attr, value),
            Op::Le => Item::Le(attr, value),
            Op::Approx => Item::Approx(attr, value),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(pairs: &[(&str, PropertyValue)]) -> Properties {
        let mut p = Properties::new();
        for (k, v) in pairs {
            p.insert(k, v.clone());
        }
        p
    }

    fn sample() -> Properties {
        props(&[
            ("service.id", PropertyValue::Str("com.acme.Nav".into())),
            ("ranking", PropertyValue::Int(100)),
            ("live", PropertyValue::Bool(true)),
        ])
    }

    #[test]
    fn equality_and_case_insensitive_key() {
        let f = Filter::parse("(SERVICE.ID=com.acme.Nav)").unwrap();
        assert!(f.matches(&sample()));
        let g = Filter::parse("(service.id=com.other.Nav)").unwrap();
        assert!(!g.matches(&sample()));
    }

    #[test]
    fn numeric_ge_le() {
        assert!(Filter::parse("(ranking>=100)").unwrap().matches(&sample()));
        assert!(Filter::parse("(ranking<=100)").unwrap().matches(&sample()));
        assert!(!Filter::parse("(ranking>=101)").unwrap().matches(&sample()));
        assert!(!Filter::parse("(ranking<=99)").unwrap().matches(&sample()));
    }

    #[test]
    fn presence_and_substring() {
        assert!(Filter::parse("(ranking=*)").unwrap().matches(&sample()));
        assert!(!Filter::parse("(absent=*)").unwrap().matches(&sample()));
        assert!(
            Filter::parse("(service.id=com.acme.*)")
                .unwrap()
                .matches(&sample())
        );
        assert!(
            Filter::parse("(service.id=*Nav)")
                .unwrap()
                .matches(&sample())
        );
        assert!(
            Filter::parse("(service.id=com*Nav)")
                .unwrap()
                .matches(&sample())
        );
        assert!(
            !Filter::parse("(service.id=com*Boat)")
                .unwrap()
                .matches(&sample())
        );
    }

    #[test]
    fn boolean_and_approx() {
        assert!(Filter::parse("(live=true)").unwrap().matches(&sample()));
        assert!(
            Filter::parse("(service.id~=COM.ACME.NAV)")
                .unwrap()
                .matches(&sample())
        );
    }

    #[test]
    fn and_or_not() {
        let f = Filter::parse("(&(service.id=com.acme.Nav)(ranking>=50))").unwrap();
        assert!(f.matches(&sample()));
        let g = Filter::parse("(&(service.id=com.acme.Nav)(ranking>=200))").unwrap();
        assert!(!g.matches(&sample()));
        let h = Filter::parse("(|(ranking>=200)(live=true))").unwrap();
        assert!(h.matches(&sample()));
        let n = Filter::parse("(!(ranking>=200))").unwrap();
        assert!(n.matches(&sample()));
    }

    #[test]
    fn undeclared_property_never_matches() {
        // liar family: a filter claiming a property the service never declared
        assert!(
            !Filter::parse("(ghost=anything)")
                .unwrap()
                .matches(&sample())
        );
        assert!(!Filter::parse("(ghost>=1)").unwrap().matches(&sample()));
    }

    #[test]
    fn malformed_filters_redden() {
        for bad in [
            "",
            "(",
            "(service.id=)",
            "(service.id=com.acme.Nav",
            "(&)",
            "(|)",
            "(=value)",
            "(service.id~value)",
            "(service.id=com.acme.Nav))",
            "(& (service.id=x) ) trailing",
            "(service.id>1)",
        ] {
            assert!(Filter::parse(bad).is_err(), "should reject: {bad:?}");
        }
    }

    #[test]
    fn whitespace_tolerated() {
        let f = Filter::parse("( & (service.id=com.acme.Nav) (ranking>=50) )").unwrap();
        assert!(f.matches(&sample()));
    }
}
