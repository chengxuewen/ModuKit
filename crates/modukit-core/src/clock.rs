//! Unified clock + timers-as-jobs (D20, modules/03 §6) — M6 floor 4.
//!
//! One water source for time, kernel-owned, registered as the ordinary
//! service `org.modukit.clock` (D40: the service table is the only extension
//! mechanism). Domains: `Real` (monotonic since host start), `Step` (dial
//! turns only via `advance` — the testhost/replay axis), `Sim` (external
//! time base injected via `advance`; the HIL wiring field stays deferred
//! per D20).
//!
//! Timers are D6 message-shaped jobs: `timer(deadline, job) -> DueJob`
//! hands back a correlation `JobToken`; a timer fires by POSTING that token
//! (returned by `advance` / surfaced by the kernel onto the event ring),
//! never by calling a closure from a hidden thread. `Real` has no dial, so
//! due tokens are collected by explicit polling (`due_jobs`) — the kernel
//! never wakes user code on its own.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

/// Monotonic nanosecond stamp within one clock domain (D16: content keeps
/// one axis with events and logs; comparisons across domains are meaningless).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Timestamp(u64);

impl Timestamp {
    #[must_use]
    pub const fn from(nanos: u64) -> Self {
        Self(nanos)
    }

    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// Elapsed since an earlier stamp in the same domain (saturating).
    #[must_use]
    pub fn delta_since(self, earlier: Timestamp) -> Duration {
        Duration::from_nanos(self.0.saturating_sub(earlier.0))
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ts({}ns)", self.0)
    }
}

/// Which water source feeds `now()` (modules/03 §6: real | sim | step).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClockDomain {
    /// Host monotonic clock (`Instant` since construction).
    Real,
    /// External time base pushed in via `advance` (rigs/HIL; wiring field
    /// deferred per D20 — v0 the dial IS the injection).
    Sim,
    /// Pure dial: time exists only where `advance` moved it (D24 testhost).
    Step,
}

/// Correlation id of a fired job (02 §4: a token, not a future).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobToken(u64);

impl JobToken {
    #[must_use]
    pub const fn value(self) -> u64 {
        self.0
    }

    /// Rebuild a token from its wire value (D74 due-return round trip,
    /// S2-M3b): the dial answer carries `value()` back.
    #[must_use]
    pub const fn at(value: u64) -> Self {
        Self(value)
    }
}

/// One posted timer: the token plus the job label it names (message-shaped,
/// D6). The kernel turns a due `DueJob` into a `TimerDone` event on the ring.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueJob {
    pub token: JobToken,
    pub job: String,
}

impl DueJob {
    #[must_use]
    pub fn token(&self) -> JobToken {
        self.token
    }

    #[must_use]
    pub fn job(&self) -> &str {
        &self.job
    }
}

/// The pending-timer register shared by clones of one clock.
#[derive(Debug, Default)]
struct Timers {
    /// deadline -> next due job (FIFO per identical deadline).
    due: BTreeMap<u64, Vec<DueJob>>,
    next_token: u64,
}

#[derive(Debug)]
struct Inner {
    domain: ClockDomain,
    real_base: Instant,
    /// `Step`/`Sim`: nanoseconds advanced by the dial so far.
    dialed_ns: AtomicU64,
    timers: Mutex<Timers>,
}

/// The kernel's one clock. Cheap to clone (one shared source).
#[derive(Debug, Clone)]
pub struct Clock {
    inner: Arc<Inner>,
}

fn lock_timers(inner: &Inner) -> MutexGuard<'_, Timers> {
    inner.timers.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Clock {
    /// A clock fed by `domain`.
    #[must_use]
    pub fn new(domain: ClockDomain) -> Self {
        Self {
            inner: Arc::new(Inner {
                domain,
                real_base: Instant::now(),
                dialed_ns: AtomicU64::new(0),
                timers: Mutex::new(Timers::default()),
            }),
        }
    }

    #[must_use]
    pub fn domain(&self) -> ClockDomain {
        self.inner.domain
    }

    /// The single stamp source (modules/03 §6: plugin time comes from the
    /// service, never from a private `Instant`).
    #[must_use]
    pub fn now(&self) -> Timestamp {
        match self.inner.domain {
            ClockDomain::Real => Timestamp(
                u64::try_from(self.inner.real_base.elapsed().as_nanos()).unwrap_or(u64::MAX),
            ),
            ClockDomain::Step | ClockDomain::Sim => {
                Timestamp(self.inner.dialed_ns.load(Ordering::Relaxed))
            }
        }
    }

    /// Turn the dial (step|sim). Returns the jobs whose deadline was crossed
    /// by this turn, in deadline order — posting tokens, not calling
    /// closures. On `Real` the dial does not exist: this returns the empty
    /// set and due jobs accumulate for `due_jobs()` polling instead.
    pub fn advance(&self, delta: Duration) -> Vec<DueJob> {
        if matches!(self.inner.domain, ClockDomain::Real) {
            return Vec::new();
        }
        let delta_ns = u64::try_from(delta.as_nanos()).unwrap_or(u64::MAX);
        // fetch_add returns the previous value; prev + delta is the new dial.
        let now = self
            .inner
            .dialed_ns
            .fetch_add(delta_ns, Ordering::Relaxed)
            .saturating_add(delta_ns);
        self.collect_due(now)
    }

    /// Register a timer firing at `deadline` (same-domain stamp). The only
    /// way it "fires" is the token being posted by `advance` / `due_jobs`.
    pub fn timer(&self, deadline: Timestamp, job: impl Into<String>) -> JobToken {
        let mut timers = lock_timers(&self.inner);
        timers.next_token += 1;
        let token = JobToken(timers.next_token);
        timers.due.entry(deadline.raw()).or_default().push(DueJob {
            token,
            job: job.into(),
        });
        token
    }

    /// Poll the `Real` domain for crossed deadlines (explicit, kernel-driven
    /// — no timer thread exists, D6). Step/Sim dial turns already deliver
    /// due jobs from `advance`, so this drains nothing extra there.
    pub fn due_jobs(&self) -> Vec<DueJob> {
        self.collect_due(self.now().raw())
    }

    /// Pending timer count — introspection only, never enforcement.
    #[must_use]
    pub fn pending_timers(&self) -> usize {
        lock_timers(&self.inner).due.values().map(Vec::len).sum()
    }

    /// Wave-2 T1 (D99 R4-A): the ONLY cancel path - drop pending jobs whose
    /// token is in `tokens` (the kernel's stop/uninstall release feeds this
    /// from the per-plugin register; the authority-law grammar lives in
    /// `kernel.rs`, this is the mechanism). Scans the due buckets; a job
    /// already collected by `advance`/`due_jobs` is gone regardless. Returns
    /// the number removed - introspection for the census, never enforcement.
    pub(crate) fn remove_by_tokens(&self, tokens: &[u64]) -> usize {
        if tokens.is_empty() {
            return 0;
        }
        let mut timers = lock_timers(&self.inner);
        let mut removed = 0usize;
        timers.due.retain(|_deadline, bucket| {
            let before = bucket.len();
            bucket.retain(|job| !tokens.contains(&job.token.value()));
            removed += before - bucket.len();
            !bucket.is_empty()
        });
        removed
    }

    fn collect_due(&self, now_ns: u64) -> Vec<DueJob> {
        let mut timers = lock_timers(&self.inner);
        let crossed: Vec<u64> = timers.due.range(..=now_ns).map(|(k, _)| *k).collect();
        let mut fired = Vec::new();
        for k in crossed {
            if let Some(bucket) = timers.due.remove(&k) {
                fired.extend(bucket);
            }
        }
        fired
    }
}
