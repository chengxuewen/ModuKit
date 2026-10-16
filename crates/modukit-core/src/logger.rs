//! The plugin mouth — logger service (D19, modules/10 §5) — M6 floor 3.
//!
//! `acquire::<Logger>()` like any service (01 §3: no ambient authority; the
//! ban on stray `printf` implies a formal mouth). A record is
//! `{level, component, msg, kv[], seq, ts}` — seq and ts ride the shared
//! observatory ring's wrapper, so logs, lifecycle edges and future frames
//! stay ONE axis (D16 replay narration, `graph --diff` audit trail).
//!
//! The ring is the M5 discipline again: bounded, drop-OLDEST, counted —
//! a flooding producer is never blocked and never dropped silently. Per
//! component the kernel keeps visible line counts; year 1 is observation
//! only, rate LIMITS stay trigger-gated (06 §6, D34: no enforcement policy
//! in year 1 — the v0 reading of the 10 §5 "rate limit with explicit
//! dropped counter" is exactly this visible-count posture).

use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::events::{ObsEvent, Observatory};

/// Severity of a log record. Plain data — sinks format downstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            LogLevel::Trace => "TRACE",
            LogLevel::Debug => "DEBUG",
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
        };
        f.write_str(name)
    }
}

/// The log record itself (10 §5 shape minus seq/ts, which the ring's
/// `ObsRecord` supplies so logs share the one monotonic axis).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEvent {
    pub level: LogLevel,
    /// Component = plugin id (or `org.modukit.*` for kernel parts).
    pub component: String,
    pub msg: String,
    pub kv: Vec<(String, String)>,
}

/// The logger service instance the kernel registers under
/// `org.modukit.logger`. Cheap to share; every clone writes one ring.
#[derive(Debug)]
pub struct Logger {
    obs: Arc<Observatory>,
    counts: Mutex<HashMap<String, u64>>,
}

impl Logger {
    /// A logger writing onto `obs`'s ring.
    #[must_use]
    pub fn new(obs: Arc<Observatory>) -> Self {
        Self {
            obs,
            counts: Mutex::new(HashMap::new()),
        }
    }

    /// Emit one record. Never fails, never blocks on a consumer: the ring
    /// sheds its oldest entry under lock and the eviction is counted
    /// (`Observatory::drops`).
    pub fn log(&self, level: LogLevel, component: &str, msg: &str, kv: Vec<(String, String)>) {
        self.obs.emit(ObsEvent::Log(LogEvent {
            level,
            component: component.to_string(),
            msg: msg.to_string(),
            kv,
        }));
        let mut counts = self
            .counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *counts.entry(component.to_string()).or_default() += 1;
    }

    /// Lines ever logged by `component` — visibility, not enforcement.
    #[must_use]
    pub fn component_count(&self, component: &str) -> u64 {
        let counts = self
            .counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        counts.get(component).copied().unwrap_or(0)
    }

    /// Distinct components seen — small cardinality by construction
    /// (plugin ids), so a plain count is enough.
    #[must_use]
    pub fn components(&self) -> usize {
        let counts = self
            .counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        counts.len()
    }
}
