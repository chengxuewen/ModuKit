//! The in-process topic bus (D10, modules/06 §2/§5, Stage-1 M5).
//!
//! Year-1 placement (06 §5): broadcast channels + latest-value, "hundreds of
//! lines" scale. Topics are named by a canonical reverse-DNS id (D9) and carry
//! opaque schema-shaped bytes (04 §2) — the bus NEVER inspects a payload
//! (D42 / 06 §3). Registry presence events and the bus are separate channels:
//! nothing here reaches into `registry.rs`.
//!
//! The three commandments (06 §2), asserted by `tests/bus_topics.rs`:
//!   1. No request/response — `publish` returns `()` and a `Receiver` has no
//!      send-side verb; topics have no return channel by construction.
//!   2. No cross-publisher ordering — a subscription is topic-scoped; the bus
//!      never sequences across publishers. (Per-topic order is FIFO, but no
//!      global ordering is promised.)
//!   3. A slow subscriber never pushes through the publisher — publish takes a
//!      short lock, drops the oldest value of an undrained per-subscription
//!      ring when it is full, counts that eviction, and returns; it NEVER
//!      blocks waiting for a consumer. Observation only: there is no rate-limit
//!      or enforcement policy (06 §6).

use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError, Weak};
use std::time::Duration;

use crate::identity::TopicId;
use crate::topic::QoS;

/// The frozen D51 code a year-1-unsupported QoS reports under. Reused (not
/// invented): the deferral is a contract the subscriber asked for and the
/// kernel declined to honour implicitly (D7 liar class).
const DEFERRAL_CODE: &str = "CONTRACT_VIOLATION";

/// Bus-side failure. Today the only refusal is a deferred QoS at subscribe
/// time; `publish` itself never fails (commandment 3 — it never blocks).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusError {
    /// The topic declared a QoS the year-1 bus does not implement. Carries
    /// the offending QoS so the caller can tell `KeepAll` from `Bounded`.
    Deferred { qos: QoS },
}

impl BusError {
    /// D51 code name for the failure (machine channel; `Display` is prose).
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            // A year-1 deferral is not transient: retrying the same subscribe
            // cannot help until the QoS is actually built (a later milestone).
            BusError::Deferred { .. } => DEFERRAL_CODE,
        }
    }
}

impl std::fmt::Display for BusError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BusError::Deferred { qos } => write!(
                f,
                "QoS {qos} is declared but not implemented in year 1; refusing to \
                 silently behave like latest-value (06 §2/§6, D7 liar class)"
            ),
        }
    }
}

impl std::error::Error for BusError {}

/// The per-subscription ring. One queue + condvar + a monotonic drop counter
/// the consumer can read. Shared with the bus through a `Weak` so a receiver
/// that is dropped (or never created) prunes cleanly without a shutdown hook.
#[derive(Debug)]
struct Shared {
    queue: Mutex<VecDeque<Vec<u8>>>,
    not_empty: Condvar,
    drops: AtomicU64,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            not_empty: Condvar::new(),
            drops: AtomicU64::new(0),
        }
    }
}

fn lock_queue(shared: &Shared) -> MutexGuard<'_, VecDeque<Vec<u8>>> {
    shared.queue.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The handle `Bus::subscribe` hands out (a `Receiver` in the message-passing
/// sense: recv-side verbs only — commandment 1). Cloning shares one queue and
/// one drop counter with every clone.
#[derive(Clone, Debug)]
pub struct Receiver {
    shared: Arc<Shared>,
}

impl Receiver {
    /// Take the oldest buffered value without waiting. `None` when empty or
    /// after the publisher side has dropped this subscription's queue.
    pub fn try_recv(&mut self) -> Option<Vec<u8>> {
        lock_queue(&self.shared).pop_front()
    }

    /// Block until a value arrives or the wait times out. Pure std
    /// Mutex+Condvar (D6: no async, no crossbeam).
    pub fn recv_timeout(&mut self, timeout: Duration) -> Option<Vec<u8>> {
        let guard = lock_queue(&self.shared);
        let (mut guard, wait) = self
            .shared
            .not_empty
            .wait_timeout_while(guard, timeout, |q: &mut VecDeque<Vec<u8>>| q.is_empty())
            .unwrap_or_else(PoisonError::into_inner);
        if wait.timed_out() && guard.is_empty() {
            None
        } else {
            guard.pop_front()
        }
    }

    /// Values this subscription has dropped because its ring was full when a
    /// publish arrived. The per-subscription commandment-3 policy is visible
    /// here and nowhere else — there is no enforcement on top (06 §6).
    #[must_use]
    pub fn drops(&self) -> u64 {
        self.shared.drops.load(Ordering::Relaxed)
    }
}

/// Per-topic state: the declared QoS, the latest-value register, the lossy
/// retention ring (with its own eviction count), and the live subscribers.
#[derive(Default)]
struct TopicState {
    qos: QoS,
    current: Option<Vec<u8>>,
    retention: VecDeque<Vec<u8>>,
    topic_drops: u64,
    subscribers: Vec<Weak<Shared>>,
}

/// The in-process bus. Cheap to clone (one shared table); reachable from the
/// kernel via `Kernel::bus()`.
#[derive(Default)]
pub struct Bus {
    topics: Mutex<HashMap<TopicId, TopicState>>,
    // A `Weak` upgrade only happens under the table lock; no queue is ever
    // locked across a topic lookup, so publish holds the table briefly.
}

/// The per-publish ring capacity implied by a QoS (06 §2). Deferred kinds map
/// to `0` — they have no subscribers, so fan-out is a no-op for them.
fn cap_of(qos: QoS) -> usize {
    match qos {
        QoS::LatestValue => 1,
        QoS::Lossy(n) => usize::try_from(n).unwrap_or(usize::MAX),
        QoS::KeepAll | QoS::Bounded => 0,
    }
}

fn lock_topics(bus: &Bus) -> MutexGuard<'_, HashMap<TopicId, TopicState>> {
    bus.topics.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Bus {
    /// An empty bus.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Publish a value on `topic` declaring its `qos` (QoS is a publish-time
    /// property, 06 §2). Returns `()` — commandment 1 (no response channel).
    ///
    /// The publisher never blocks on a subscriber (commandment 3): each live
    /// ring that is already full sheds its oldest value and counts the drop.
    pub fn publish(&self, topic: &TopicId, qos: QoS, value: Vec<u8>) {
        let mut topics = lock_topics(self);
        let state = topics.entry(topic.clone()).or_default();
        state.qos = qos;

        // Topic-side retention for late-join.
        match qos {
            QoS::LatestValue => {
                state.current = Some(value.clone());
                state.retention.clear();
            }
            QoS::Lossy(n) => {
                state.current = None;
                let cap = usize::try_from(n).unwrap_or(usize::MAX);
                state.retention.push_back(value.clone());
                while state.retention.len() > cap.max(1) {
                    state.retention.pop_front();
                    state.topic_drops += 1;
                }
            }
            // Deferred QoS: the declaration is recorded so a subscribe can
            // refuse it; no queue is built (06 §6 — no durable queue year 1).
            QoS::KeepAll | QoS::Bounded => {
                state.current = None;
                state.retention.clear();
            }
        }

        // Fan-out. The table lock is held only for this short pass; a slow or
        // dead subscriber never stalls it.
        let cap = cap_of(qos);
        state.subscribers.retain(|weak| {
            let Some(shared) = weak.upgrade() else {
                return false; // consumer dropped: prune.
            };
            if cap == 0 {
                // Lossy(0) or a deferred topic with a leftover subscription:
                // there is no slot, so the value is shed and counted.
                shared.drops.fetch_add(1, Ordering::Relaxed);
                return true;
            }
            {
                let mut queue = lock_queue(&shared);
                while queue.len() >= cap {
                    queue.pop_front();
                    shared.drops.fetch_add(1, Ordering::Relaxed);
                }
                queue.push_back(value.clone());
            }
            shared.not_empty.notify_one();
            true
        });
    }

    /// Subscribe to `topic`, obtaining a `Receiver`.
    ///
    /// # Errors
    ///
    /// `BusError::Deferred` when the topic's declared QoS is `KeepAll` or
    /// `Bounded` — implemented in a later milestone, refused here rather than
    /// quietly degrading to latest-value (06 §2/§6, D7 liar class).
    pub fn subscribe(&self, topic: &TopicId) -> Result<Receiver, BusError> {
        let mut topics = lock_topics(self);
        let state = topics.entry(topic.clone()).or_default();
        if state.qos.is_deferred() {
            return Err(BusError::Deferred { qos: state.qos });
        }

        let shared = Arc::new(Shared::default());
        {
            // Late-join: seed from the register the declared QoS maintains.
            let mut queue = lock_queue(&shared);
            match state.qos {
                QoS::LatestValue => {
                    if let Some(v) = &state.current {
                        queue.push_back(v.clone());
                    }
                }
                QoS::Lossy(_) => {
                    for v in state.retention.iter() {
                        queue.push_back(v.clone());
                    }
                }
                QoS::KeepAll | QoS::Bounded => {}
            }
        }
        state.subscribers.push(Arc::downgrade(&shared));
        Ok(Receiver { shared })
    }

    /// Values evicted from the topic's own retention ring (the `Lossy(n)`
    /// "visible drop count" of 06 §2; latest-value topics always report 0).
    #[must_use]
    pub fn topic_drop_count(&self, topic: &TopicId) -> u64 {
        lock_topics(self)
            .get(topic)
            .map_or(0, |state| state.topic_drops)
    }
}
