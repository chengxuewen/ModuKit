//! Manifest v0 (TOML per D7; field grammar modules/05 §1-3). The
//! `ManifestError` type and its Display live in `error` (vocabulary module).
//!
//! Scope discipline: this is the deploy-tier *file* type only. Tier
//! residence policing and the deploy<->self-report cross-check are M7
//! (install protocol), and nothing here touches the install flow. Unknown
//! TOML fields are tolerated (future deploy-tier sections like `topics`,
//! `[resources]`, `[net]` ride the same envelope per 05 §5-6).

use std::str::FromStr;
use std::time::Duration;

use serde::Deserialize;

use crate::error::ManifestError;
use crate::identity::{PluginId, ServiceId};
use crate::lifecycle::{Backoff, RestartPolicy};

/// Placement floor declared by `entry.kind` (05 §1). Stage-1 executes `dl`
/// only; the other arms parse so deploy files for later stages can exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementKind {
    Dl,
    Exe,
    Wasm,
}

impl PlacementKind {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            PlacementKind::Dl => "dl",
            PlacementKind::Exe => "exe",
            PlacementKind::Wasm => "wasm",
        }
    }
}

impl std::fmt::Display for PlacementKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Entry descriptor: what artifact exists and how it may run (05 §1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub kind: PlacementKind,
    pub artifact: String,
    /// Optional interpreter for script plugins (05 §1); inert in Stage-1.
    pub runner: Option<String>,
}

/// Consumer policy while the dependency is absent (02 §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepPolicy {
    Wait(Duration),
    FailFast,
    Fallback,
}

/// Behavior after a previously present dependency vanishes (02 §2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnDown {
    RestartSelf,
    Block,
    FailSelf,
}

/// One `[[requires]]` declaration, service string already canonicalized
/// through the bundle-scoped alias layer (05 §3, D9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requires {
    pub service: ServiceId,
    pub required: bool,
    pub policy: DepPolicy,
    pub on_down: Option<OnDown>,
}

/// One `[[provides]]` declaration. `ranking` is reserved storage (invariant
/// 4); `parallel_ok` ships inert (D8, routing arrives Stage-2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Provides {
    pub service: ServiceId,
    pub ranking: i64,
    pub parallel_ok: bool,
}

/// The parsed deploy-tier manifest (modules/05 §1-3 fields settled for v0).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub id: PluginId,
    pub version: String,
    /// D40 audit/accounting label; no permission semantics.
    pub team: String,
    /// Boot tier (03 §3): level by level, parallel within a level.
    pub start_level: i32,
    pub start_timeout: Option<Duration>,
    pub stop_timeout: Option<Duration>,
    /// Forward-compat anchor (05 §4).
    pub min_host: Option<String>,
    /// D26 word list; hard-check lives with the resolve gates of later M's.
    pub capabilities: Vec<String>,
    pub entry: Entry,
    pub restart: RestartPolicy,
    pub requires: Vec<Requires>,
    pub provides: Vec<Provides>,
}

impl Manifest {
    /// Parse a manifest from TOML text (D7: single format, Cargo-grade
    /// parser per D31).
    ///
    /// # Errors
    ///
    /// `ManifestError` for TOML shape errors and for every grammar rule of
    /// 05 §1-3 violated (unknown placement, bad duration, bad policy word).
    pub fn from_toml_str(input: &str) -> Result<Self, ManifestError> {
        let raw: RawManifest =
            toml::from_str(input).map_err(|e| ManifestError::Toml(e.to_string()))?;
        raw.into_manifest()
    }
}

/// `"<count><unit>"` with unit in s/m/h (05 §2 examples `5s`, `2s`, `5m`).
fn parse_duration(text: &str) -> Result<Duration, ManifestError> {
    let text = text.trim();
    let (digits, secs_per, tail_ok) = match text.as_bytes().last() {
        Some(b's') => (&text[..text.len() - 1], 1u64, true),
        Some(b'm') => (&text[..text.len() - 1], 60, true),
        Some(b'h') => (&text[..text.len() - 1], 3600, true),
        _ => (text, 1, false),
    };
    if !tail_ok || digits.is_empty() {
        return Err(ManifestError::BadDuration(text.to_string()));
    }
    let count = u64::from_str(digits).map_err(|_| ManifestError::BadDuration(text.to_string()))?;
    count
        .checked_mul(secs_per)
        .map(Duration::from_secs)
        .ok_or_else(|| ManifestError::BadDuration(text.to_string()))
}

/// `"1s..30s"` window (05 §2 restart backoff).
fn parse_backoff(text: &str) -> Result<Backoff, ManifestError> {
    let Some((lo, hi)) = text.split_once("..") else {
        return Err(ManifestError::BadBackoff(text.to_string()));
    };
    let min = parse_duration(lo)?;
    let max = parse_duration(hi)?;
    if min > max {
        return Err(ManifestError::BadBackoff(text.to_string()));
    }
    Ok(Backoff { min, max })
}

/// `"wait(10s)" | "failfast" | "fallback"` (02 §2).
fn parse_dep_policy(text: &str) -> Result<DepPolicy, ManifestError> {
    let text = text.trim();
    if text == "failfast" {
        return Ok(DepPolicy::FailFast);
    }
    if text == "fallback" {
        return Ok(DepPolicy::Fallback);
    }
    if let Some(inner) = text.strip_prefix("wait(").and_then(|t| t.strip_suffix(')')) {
        return Ok(DepPolicy::Wait(parse_duration(inner)?));
    }
    Err(ManifestError::BadDepPolicy(text.to_string()))
}

fn parse_on_down(text: &str) -> Result<OnDown, ManifestError> {
    match text.trim() {
        "restart-self" => Ok(OnDown::RestartSelf),
        "block" => Ok(OnDown::Block),
        "fail-self" => Ok(OnDown::FailSelf),
        other => Err(ManifestError::BadOnDown(other.to_string())),
    }
}

fn parse_placement(text: &str) -> Result<PlacementKind, ManifestError> {
    match text.trim() {
        "dl" => Ok(PlacementKind::Dl),
        "exe" => Ok(PlacementKind::Exe),
        "wasm" => Ok(PlacementKind::Wasm),
        other => Err(ManifestError::UnknownPlacement(other.to_string())),
    }
}

#[derive(Deserialize)]
struct RawManifest {
    id: String,
    version: String,
    #[serde(default)]
    team: Option<String>,
    #[serde(default)]
    start_level: Option<i32>,
    #[serde(default)]
    start_timeout: Option<String>,
    #[serde(default)]
    stop_timeout: Option<String>,
    #[serde(default)]
    min_host: Option<String>,
    #[serde(default)]
    capabilities: Vec<String>,
    entry: RawEntry,
    #[serde(default)]
    restart: Option<RawRestart>,
    #[serde(default)]
    aliases: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    requires: Vec<RawRequires>,
    #[serde(default)]
    provides: Vec<RawProvides>,
}

#[derive(Deserialize)]
struct RawEntry {
    kind: String,
    artifact: String,
    #[serde(default)]
    runner: Option<String>,
}

#[derive(Deserialize)]
struct RawRestart {
    #[serde(default = "default_restart_policy")]
    policy: String,
    #[serde(default)]
    max_retries: Option<u32>,
    #[serde(default)]
    backoff: Option<String>,
    #[serde(default)]
    cooldown: Option<String>,
}

fn default_restart_policy() -> String {
    "none".to_string()
}

#[derive(Deserialize)]
struct RawRequires {
    service: String,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    policy: Option<String>,
    #[serde(default)]
    on_down: Option<String>,
}

#[derive(Deserialize)]
struct RawProvides {
    service: String,
    #[serde(default)]
    ranking: Option<i64>,
    #[serde(default)]
    parallel_ok: Option<bool>,
}

impl RawManifest {
    fn into_manifest(self) -> Result<Manifest, ManifestError> {
        // Alias layer resolves at manifest-read time; everything downstream
        // sees canonical strings only (05 §3, D9). Unknown names pass
        // through — they are already canonical.
        let resolve = |name: &str| -> String {
            self.aliases
                .get(name)
                .cloned()
                .unwrap_or_else(|| name.to_string())
        };

        let kind = parse_placement(&self.entry.kind)?;
        let requires = self
            .requires
            .iter()
            .map(|r| {
                Ok(Requires {
                    service: ServiceId::new(resolve(&r.service)),
                    required: r.required,
                    // No documented default in 02 §2; v0 picks the most
                    // predictable behavior (fail fast at use time).
                    policy: match &r.policy {
                        Some(p) => parse_dep_policy(p)?,
                        None => DepPolicy::FailFast,
                    },
                    on_down: match &r.on_down {
                        Some(o) => Some(parse_on_down(o)?),
                        None => None,
                    },
                })
            })
            .collect::<Result<Vec<_>, ManifestError>>()?;
        let provides = self
            .provides
            .iter()
            .map(|p| Provides {
                service: ServiceId::new(resolve(&p.service)),
                ranking: p.ranking.unwrap_or(0),
                parallel_ok: p.parallel_ok.unwrap_or(false),
            })
            .collect();

        let restart = match &self.restart {
            None => RestartPolicy::None,
            Some(rs) => match rs.policy.trim() {
                "none" => RestartPolicy::None,
                "on-failure" => RestartPolicy::OnFailure {
                    max_retries: rs.max_retries.ok_or_else(|| {
                        ManifestError::BadRestartPolicy(
                            "on-failure requires max_retries".to_string(),
                        )
                    })?,
                    backoff: match &rs.backoff {
                        Some(b) => parse_backoff(b)?,
                        None => Backoff::default(),
                    },
                    cooldown: match &rs.cooldown {
                        Some(c) => parse_duration(c)?,
                        None => Duration::from_secs(0),
                    },
                },
                other => return Err(ManifestError::BadRestartPolicy(other.to_string())),
            },
        };

        Ok(Manifest {
            id: PluginId::new(self.id),
            version: self.version,
            team: self.team.unwrap_or_else(|| "default".to_string()),
            start_level: self.start_level.unwrap_or(0),
            start_timeout: match &self.start_timeout {
                Some(t) => Some(parse_duration(t)?),
                None => None,
            },
            stop_timeout: match &self.stop_timeout {
                // D34 records the 2s default; the kernel keeps it as the
                // documented fallback when the field is absent.
                Some(t) => Some(parse_duration(t)?),
                None => Some(Duration::from_secs(2)),
            },
            min_host: self.min_host,
            capabilities: self.capabilities,
            entry: Entry {
                kind,
                artifact: self.entry.artifact,
                runner: self.entry.runner,
            },
            restart,
            requires,
            provides,
        })
    }
}
