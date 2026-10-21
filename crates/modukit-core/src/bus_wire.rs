//! Stage-5 N2 - the bus crossing (D96 rulings 3+4+6): the machinery behind
//! the two new frame talkers on the TL control connection.
//!
//! Parent side (`ParentFanout`, attached by `Supervisor::with_clock`): a
//! child's FORWARD(pub) hits the PARENT bus through the very same
//! `Bus::publish` an in-process plugin uses - one fan-out code path, no
//! parallel semantics (ruling 6). A child's interest rides as a proxy
//! `Receiver` on that same bus, so the retained-late-join snapshot of
//! ruling 6 needs no second retention machinery: `Bus::subscribe` seeds
//! it and the drain forwards it as DELIVER frames.
//!
//! Child side (`Bridge`, installed once by `modukit-run` when the HELLO
//! declared `HELLO_CAP_BUS_V2`): the `host_v2` mailbox slots consult the
//! process bridge - publish declares locally and forwards, subscribe
//! registers upstream interest, a recv over an empty shadow queue pumps
//! the control socket under a bounded window so the self-echo and the
//! snapshot DELIVERs land in the local ring. The mailbox GRAMMAR (need-
//! report, pending-restore, per-box drop counters, the pinned -11/-14/
//! -19/-26 channels) is N1's and is reused verbatim, never forked
//! (ruling 5); the refusal channel rides a DELIVER with a negative qos.

use std::collections::HashMap;
use std::io;
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};
use std::time::{Duration, Instant};

use crate::bus::{Bus, Receiver};
use crate::embed_codec as codec;
use crate::identity::TopicId;
use crate::proc_wire::{self, Delivery, Forward};
use crate::topic::QoS;

/// The D51 number the bridged mailbox answers when the control pipe died
/// mid-call (CARRIER_UNREACHABLE). Not in `embed_codec` because the embed
/// face never had this doorway - it is the wire's own refusal, and the
/// frozen dictionary already numbers it.
pub(crate) const NEG_CARRIER_UNREACHABLE: i32 = -16;

/// Bounded shadow-pump window (per empty recv): a local unix-socket round
/// trip is microseconds, so one second is three orders of margin. Zero-
/// sleep discipline holds: this is a read timeout, never a sleep.
/// shortcut: sized for same-machine control sockets; a host that ever
/// rides a real remote pipe needs a credit scheme, not a longer timeout.
const PUMP_WINDOW: Duration = Duration::from_secs(1);

/// QoS -> the embed-face int, the exact reverse of `codec::qos_from_i32`.
/// `Lossy` answers the vocabulary's fixed `LOSSY_RING` slot, same
/// stand-in the embed publish door uses.
#[must_use]
pub(crate) const fn qos_to_i32(qos: QoS) -> i32 {
    match qos {
        QoS::LatestValue => 0,
        QoS::KeepAll => 1,
        QoS::Lossy(_) => 2,
        QoS::Bounded => 3,
    }
}

// ---------------------------------------------------------------- parent

/// Per-capable-child topic interest plus the child control sockets, one
/// instance per Supervisor boot (`proc_supervisor`), shared through the
/// executor table so the start-window pump reaches it from the frame loop.
pub(crate) struct ParentFanout {
    bus: Arc<Bus>,
    /// Child plugin id -> a writable control-socket clone. Registration
    /// happens at the child's first FORWARD; the ack retires it (the
    /// child's queues die with its `start` per 01 §3 rule 2).
    children: Mutex<HashMap<String, UnixStream>>,
    /// Topic -> child id -> the proxy `Receiver` held on the PARENT bus
    /// (one box per child x topic; the child fans it into its own per-
    /// queue rings). Interest dedup falls out: a second subscribe for a
    /// live pair is a no-op, so the snapshot never double-delivers - the
    /// child's own local late-join covers same-child re-subscribes.
    subs: Mutex<HashMap<TopicId, HashMap<String, Receiver>>>,
}

impl ParentFanout {
    pub(crate) fn new(bus: Arc<Bus>) -> Self {
        Self {
            bus,
            children: Mutex::new(HashMap::new()),
            subs: Mutex::new(HashMap::new()),
        }
    }

    /// Idempotent: the start pump registers on the child's first FORWARD.
    pub(crate) fn register_child(&self, id: String, ctl: UnixStream) {
        self.lock_children().insert(id, ctl);
    }

    /// The mailbox authority law across the wall (01 §3 rule 2): a START
    /// ack closes the child's queues, so its wall interest and its socket
    /// leave with the call. A revived run re-opens both by its own
    /// FORWARDs - every snapshot it needs comes from `Bus::subscribe`
    /// late-join at that moment.
    pub(crate) fn retire(&self, id: &str) {
        self.lock_children().remove(id);
        let mut subs = self.lock_subs();
        for per_child in subs.values_mut() {
            per_child.remove(id);
        }
        subs.retain(|_, per_child| !per_child.is_empty());
    }

    /// Handle one FORWARD frame from `from`. The `Err` string is prose for
    /// the kill doorway: a malformed or lying frame is a protocol
    /// disagreement (the strict handshake discipline), and the parent's
    /// start pump reaps.
    pub(crate) fn handle_forward(&self, from: &str, fwd: &Forward) -> Result<(), String> {
        let topic = TopicId::try_new(fwd.topic.clone())
            .map_err(|e| format!("forward: ungrammatical topic: {e}"))?;
        match fwd.op {
            proc_wire::FWD_PUB => self.on_publish(&topic, fwd),
            proc_wire::FWD_SUB => self.on_subscribe(from, &topic),
            _ => self.on_unsubscribe(from, &topic),
        }
    }

    fn on_publish(&self, topic: &TopicId, fwd: &Forward) -> Result<(), String> {
        // THE single fan-out doorway, ruling 6: the same `Bus::publish`
        // an in-process plugin calls - parent-local subscribers, the
        // D21 gate tallies, retention, all ride it unchanged.
        let qos = codec::qos_from_i32(fwd.qos).ok_or("forward: qos off the embed vocabulary")?;
        self.bus.publish(topic, qos, fwd.value.clone());
        // Wall fan: every interested child, the publisher itself included
        // (self-echo - the in-process bus already echoes into the proxy
        // Receiver, the drain simply forwards what it holds).
        let targets: Vec<(String, Receiver)> = self
            .lock_subs()
            .get(topic)
            .map(|per_child| {
                per_child
                    .iter()
                    .map(|(child, rx)| (child.clone(), rx.clone()))
                    .collect()
            })
            .unwrap_or_default();
        for (child, mut rx) in targets {
            while let Some(value) = rx.try_recv() {
                self.deliver(&child, topic, fwd.qos, &value);
            }
        }
        Ok(())
    }

    fn on_subscribe(&self, from: &str, topic: &TopicId) -> Result<(), String> {
        if self
            .lock_subs()
            .get(topic)
            .is_some_and(|per_child| per_child.contains_key(from))
        {
            return Ok(()); // live interest: no second box, no snapshot replay
        }
        match self.bus.subscribe(topic) {
            Ok(mut rx) => {
                // Late-join seeds the retained values (ruling 6's snapshot
                // across the wall is the SAME bus mechanism, not a wire
                // invention); forward them to the subscribing child now.
                let qos = self.bus.topic_qos(topic).map(qos_to_i32).unwrap_or(0);
                self.lock_subs()
                    .entry(topic.clone())
                    .or_default()
                    .insert(from.to_string(), rx.clone());
                while let Some(value) = rx.try_recv() {
                    self.deliver(from, topic, qos, &value);
                }
            }
            // A topic deferred at the PARENT side (declared by a peer this
            // child never saw): the refusal rides the DELIVER negative-qos
            // channel so the child's local door answers -26 from now on.
            Err(crate::bus::BusError::Deferred { .. }) => {
                self.deliver(from, topic, codec::NEG_DEFERRAL_DECLINED, &[]);
            }
        }
        Ok(())
    }

    fn on_unsubscribe(&self, from: &str, topic: &TopicId) -> Result<(), String> {
        let mut subs = self.lock_subs();
        if let Some(per_child) = subs.get_mut(topic) {
            per_child.remove(from);
            if per_child.is_empty() {
                subs.remove(topic);
            }
        }
        Ok(())
    }

    /// Write one DELIVER. A dead socket removes that child's registration
    /// and is otherwise swallowed: a gone peer is the crash-watch's to
    /// report at its own doorway, never a reason to kill the publishing
    /// run whose frame triggered this delivery.
    fn deliver(&self, child: &str, topic: &TopicId, qos: i32, value: &[u8]) {
        let payload = proc_wire::deliver_payload(&Delivery {
            topic: topic.to_string(),
            qos,
            value: value.to_vec(),
        });
        let mut children = self.lock_children();
        let Some(ctl) = children.get_mut(child) else {
            return;
        };
        if proc_wire::write_frame(ctl, proc_wire::MSG_DELIVER, &payload).is_err() {
            children.remove(child);
        }
    }

    fn lock_children(&self) -> MutexGuard<'_, HashMap<String, UnixStream>> {
        self.children.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn lock_subs(&self) -> MutexGuard<'_, HashMap<TopicId, HashMap<String, Receiver>>> {
        self.subs.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl std::fmt::Debug for ParentFanout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Counts only: the executor table's `Shared` derives Debug and the
        // socket/interest registries have no printable identity (the
        // shared `Bus` stays Debug-free by design, 06 §2).
        let children = self.children.lock().map(|m| m.len()).unwrap_or(0);
        let topics = self.subs.lock().map(|m| m.len()).unwrap_or(0);
        f.debug_struct("ParentFanout")
            .field("children", &children)
            .field("topics", &topics)
            .finish()
    }
}

// ------------------------------------------------------------------ child

static BRIDGE: OnceLock<Bridge> = OnceLock::new();

/// This process's bus bridge (`Some` only inside a capable runner). The
/// `host_v2` mailbox slots fork on it: local-only semantics when absent
/// (N1's door, byte-for-byte unchanged), crossed-wall semantics when set.
pub(crate) fn bridge() -> Option<&'static Bridge> {
    BRIDGE.get()
}

/// Install the bridge over a control-socket clone - called by `modukit-run`
/// exactly once, right after a HELLO that declared `HELLO_CAP_BUS_V2`.
/// Idempotent (the first set wins, the ambient-law pattern of the embed
/// faces).
pub fn install_bridge(ctl: UnixStream) {
    let _ = BRIDGE.set(Bridge {
        ctl: Mutex::new(ctl),
    });
}

/// The child-side half: FORWARD writes plus the bounded DELIVER pump that
/// feeds this process's local bus (the shadow queue the mailbox reads).
/// All calls run on the runner's single thread inside `start`, so the
/// write path never races the main loop - the main loop is parked in the
/// very `kernel.start` that reaches here (01 §3 rule 2 keeps the window).
/// shortcut: a threaded runner would need the parent's per-child write
/// queue; the one-thread law is what keeps this a mutex of one socket.
pub struct Bridge {
    ctl: Mutex<UnixStream>,
}

impl Bridge {
    /// One FORWARD frame out. A payload over the peer's `FRAME_MAX` read
    /// ceiling is refused here as QUOTA_EXCEEDED territory (the caller
    /// answers the mailbox error) rather than killing the run downstream.
    fn talk(&self, fwd: &Forward) -> io::Result<()> {
        let payload = proc_wire::forward_payload(fwd);
        if payload.len() > proc_wire::FRAME_MAX {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "value over the frame ceiling",
            ));
        }
        proc_wire::write_frame(&mut self.lock_ctl(), proc_wire::MSG_FORWARD, &payload)
    }

    pub(crate) fn publish(&self, topic: &str, qos: i32, value: &[u8]) -> io::Result<()> {
        self.talk(&Forward {
            op: proc_wire::FWD_PUB,
            topic: topic.to_string(),
            qos,
            value: value.to_vec(),
        })
    }

    pub(crate) fn subscribe(&self, topic: &str) -> io::Result<()> {
        self.talk(&Forward {
            op: proc_wire::FWD_SUB,
            topic: topic.to_string(),
            qos: 0,
            value: Vec::new(),
        })
    }

    pub(crate) fn unsubscribe(&self, topic: &str) -> io::Result<()> {
        self.talk(&Forward {
            op: proc_wire::FWD_UNSUB,
            topic: topic.to_string(),
            qos: 0,
            value: Vec::new(),
        })
    }

    /// Pump DELIVER frames into `bus` until `want` carries a value or the
    /// window closes; foreign-delivered values land in their own rings on
    /// the way (a pump never loses a frame meant for a different queue).
    /// `Ok(true)` = fed the wanted topic at least once. A read timeout is
    /// the ordinary `Ok(false)` answer; a foreign frame type is a
    /// protocol violation and errors out.
    /// shortcut: a torn partial frame at timeout means the parent died
    /// mid-write; the main loop's next read classifies that at its own
    /// doorway (the F1 EOF path), so the window never re-syncs by guess.
    pub(crate) fn pump(&self, bus: &Bus, want: &TopicId) -> io::Result<bool> {
        let deadline = Instant::now() + PUMP_WINDOW;
        let mut ctl = self.lock_ctl();
        let result = loop {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                break Ok(false);
            }
            ctl.set_read_timeout(Some(left))?;
            match proc_wire::read_frame(&mut ctl) {
                Ok((proc_wire::WIRE_VERSION, proc_wire::MSG_DELIVER, payload)) => {
                    let Some(delivery) = proc_wire::parse_deliver(&payload) else {
                        break Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "malformed DELIVER payload",
                        ));
                    };
                    let Ok(topic) = TopicId::try_new(delivery.topic.clone()) else {
                        break Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "DELIVER topic off the grammar",
                        ));
                    };
                    if delivery.qos < 0 {
                        // Refusal channel: record the deferral so this
                        // door's later subscribes answer the pinned -26
                        // exactly like the in-process door. No value.
                        bus.declare(&topic, QoS::KeepAll);
                    } else {
                        let qos = codec::qos_from_i32(delivery.qos).unwrap_or(QoS::LatestValue);
                        bus.publish(&topic, qos, delivery.value);
                        if &topic == want {
                            break Ok(true);
                        }
                    }
                }
                Ok((_, proc_wire::MSG_DELIVER, _)) => {
                    break Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "DELIVER frame carries a foreign wire version",
                    ));
                }
                // Anything else cannot legally exist inside the start
                // window; ignore it forward-compatibly (the main loop's
                // own posture) and keep the window running.
                Ok((_, _, _)) => {}
                Err(err)
                    if matches!(
                        err.kind(),
                        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                    ) =>
                {
                    break Ok(false);
                }
                Err(err) => break Err(err),
            }
        };
        let _ = ctl.set_read_timeout(None);
        result
    }

    fn lock_ctl(&self) -> MutexGuard<'_, UnixStream> {
        self.ctl.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proc_wire::{FWD_PUB, FWD_SUB, FWD_UNSUB};
    use std::io::Read;

    fn topic(name: &str) -> TopicId {
        TopicId::try_new(name.to_string()).expect("test topic grammar")
    }

    /// One wired child "c": the fan holds one socket of a real pair, the
    /// test reads the other under a short timeout (silence = TimedOut).
    fn wired_fan() -> (ParentFanout, UnixStream) {
        let fan = ParentFanout::new(Arc::new(Bus::new()));
        let (wired, reader) = UnixStream::pair().expect("socketpair");
        reader
            .set_read_timeout(Some(Duration::from_millis(300)))
            .expect("read timeout");
        fan.register_child("c".into(), wired);
        (fan, reader)
    }

    fn fwd(op: u8, topic: &str, qos: i32, value: &[u8]) -> Forward {
        Forward {
            op,
            topic: topic.into(),
            qos,
            value: value.to_vec(),
        }
    }

    fn next_deliver(reader: &mut UnixStream) -> Delivery {
        let (ver, kind, payload) =
            proc_wire::read_frame(reader).expect("a DELIVER frame must arrive");
        assert_eq!(
            (ver, kind),
            (proc_wire::WIRE_VERSION, proc_wire::MSG_DELIVER)
        );
        proc_wire::parse_deliver(&payload).expect("DELIVER grammar")
    }

    fn assert_silent(reader: &mut UnixStream) {
        let mut byte = [0u8; 1];
        match reader.read(&mut byte) {
            Ok(0) => {} // write half closed (post-retire drop): silence too
            Err(err)
                if matches!(
                    err.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) => {}
            other => panic!("expected wire silence, got {other:?}"),
        }
    }

    #[test]
    fn subscribe_receives_the_parent_retained_snapshot_as_deliver() {
        let (fan, mut reader) = wired_fan();
        let t = topic("com.example.modukit.buswire");
        fan.bus.publish(&t, QoS::LatestValue, b"retained".to_vec());
        fan.handle_forward("c", &fwd(FWD_SUB, t.as_str(), 0, &[]))
            .unwrap();
        let delivery = next_deliver(&mut reader);
        assert_eq!(
            (
                delivery.topic.as_str(),
                delivery.qos,
                delivery.value.as_slice()
            ),
            (t.as_str(), 0, b"retained".as_slice())
        );
        // Ruling 6 dedup: a second subscribe while the pair is live replays
        // nothing (the child's own local late-join covers it).
        fan.handle_forward("c", &fwd(FWD_SUB, t.as_str(), 0, &[]))
            .unwrap();
        assert_silent(&mut reader);
    }

    #[test]
    fn publish_echoes_back_to_a_subscribing_publisher() {
        let (fan, mut reader) = wired_fan();
        let t = topic("com.example.modukit.buswire.echo");
        fan.handle_forward("c", &fwd(FWD_SUB, t.as_str(), 0, &[]))
            .unwrap();
        assert_silent(&mut reader); // nothing retained yet
        fan.handle_forward("c", &fwd(FWD_PUB, t.as_str(), 0, b"self-echo"))
            .unwrap();
        let delivery = next_deliver(&mut reader);
        assert_eq!(
            (delivery.qos, delivery.value.as_slice()),
            (0, b"self-echo".as_slice())
        );
        // The publish ALSO counted on the parent's real bus (one fan-out).
        let stats = fan.bus.topic_stats();
        let seen = stats
            .iter()
            .find(|st| st.topic == t.as_str())
            .expect("tallied");
        assert_eq!(seen.published, 1);
    }

    #[test]
    fn deferred_topic_declared_by_a_peer_answers_the_refusal_marker() {
        let (fan, mut reader) = wired_fan();
        let t = topic("com.example.modukit.buswire.deferred");
        fan.handle_forward("c", &fwd(FWD_PUB, t.as_str(), 1, b"declared"))
            .unwrap(); // KEEP_ALL
        assert_silent(&mut reader); // a deferred publish fans out to nobody
        fan.handle_forward("c", &fwd(FWD_SUB, t.as_str(), 0, &[]))
            .unwrap();
        let delivery = next_deliver(&mut reader);
        assert_eq!(
            (delivery.qos, delivery.value.is_empty()),
            (codec::NEG_DEFERRAL_DECLINED, true),
            "the refusal channel carries the pinned -26"
        );
    }

    #[test]
    fn unsubscribe_and_the_ack_retire_close_the_box() {
        let (fan, mut reader) = wired_fan();
        let t = topic("com.example.modukit.buswire.close");
        fan.handle_forward("c", &fwd(FWD_PUB, t.as_str(), 0, b"v1"))
            .unwrap();
        fan.handle_forward("c", &fwd(FWD_SUB, t.as_str(), 0, &[]))
            .unwrap();
        assert_eq!(next_deliver(&mut reader).value, b"v1");
        fan.handle_forward("c", &fwd(FWD_UNSUB, t.as_str(), 0, &[]))
            .unwrap();
        fan.handle_forward("c", &fwd(FWD_PUB, t.as_str(), 0, b"v2"))
            .unwrap();
        assert_silent(&mut reader); // interest left with the last queue
        // Re-subscribe late-joins the fresh retained value (revive shape),
        // then the START ack retires everything again.
        fan.handle_forward("c", &fwd(FWD_SUB, t.as_str(), 0, &[]))
            .unwrap();
        assert_eq!(next_deliver(&mut reader).value, b"v2");
        fan.retire("c");
        fan.handle_forward("c", &fwd(FWD_SUB, t.as_str(), 0, &[]))
            .unwrap();
        fan.handle_forward("c", &fwd(FWD_PUB, t.as_str(), 0, b"v3"))
            .unwrap();
        assert_silent(&mut reader);
    }

    #[test]
    fn forward_grammar_lies_refuse_the_frame() {
        let (fan, _reader) = wired_fan();
        assert!(
            fan.handle_forward("c", &fwd(FWD_PUB, "Not.Valid-Topic", 0, b"x"))
                .is_err(),
            "an ungrammatical wire topic is a disagreement"
        );
        assert!(
            fan.handle_forward(
                "c",
                &fwd(FWD_PUB, "com.example.modukit.buswire.q", 42, b"x")
            )
            .is_err(),
            "a qos off the embed vocabulary is a disagreement"
        );
    }
}
