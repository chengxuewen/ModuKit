//! The vendor-shm lane (D76 confinement module): iceoryx2 is the admitted
//! first pool backend (D25, pinned `=0.10.0` on the single dependency line in
//! this crate's Cargo.toml), and EVERYTHING that names or touches it lives in
//! this file. Outside here the world sees only the neutral `Transport` seam —
//! the factory's vocabulary (service, publisher, subscriber, loan) never
//! crosses into public types, errors, or other modules (D25c quarantine).
//!
//! Frames cross as BYTES ONLY (D69): the lane's verbs are the seam's owned
//! `Vec<u8>`; no vendor handle, pointer, or sample type appears in any
//! signature. A frame larger than the lane's named ceiling is refused with
//! the typed `TransportError` — never a fake success, never a silent stub.

use std::process;
use std::sync::atomic::{AtomicU64, Ordering};

use iceoryx2::node::{Node, NodeBuilder};
use iceoryx2::port::publisher::Publisher;
use iceoryx2::port::subscriber::Subscriber;
use iceoryx2::service::ipc;
use iceoryx2::service::port_factory::publish_subscribe::PortFactory;
use iceoryx2::service::service_name::ServiceName;

use crate::error::TransportError;
use crate::seam::Transport;

/// The lane's frame ceiling in bytes. Loaning above it is a typed refusal,
/// mirroring how `SocketpairLane` refuses frames past the `u32` length field:
/// every real porter has SOME ceiling; the seam's job is to name it and fail
/// honestly, not to pretend it away.
pub const MAX_FRAME_BYTES: usize = 1 << 20;

/// Sequence giving every lane a distinct vendor service name inside a
/// process; paired with the pid so lanes from concurrent test runs (or a
/// crashed earlier run's residue) never silently share a port set.
static LANE_SEQ: AtomicU64 = AtomicU64::new(0);

fn os_err(syscall: &'static str, reason: &str) -> TransportError {
    TransportError::Backend {
        syscall,
        reason: reason.to_string(),
    }
}

/// The zero-copy lane behind the neutral seam: a vendor publish-subscribe
/// service carrying `[u8]` payloads, with one publisher and one subscriber
/// port per lane. The name says "vendor", never "factory" (D76); a future
/// swap re-implements this module and touches nothing outside it.
///
/// Like `SocketpairLane`, both ends live in one value driven from the owning
/// thread — no `Send` claim is made across threads here.
pub struct VendorShmLane {
    // Rust drops fields in declaration order, so the node is listed FIRST and
    // dies LAST: it must outlive its ports for a clean unregister.
    node: Option<Node<ipc::Service>>,
    service: Option<PortFactory<ipc::Service, [u8], ()>>,
    publisher: Option<Publisher<ipc::Service, [u8], ()>>,
    subscriber: Option<Subscriber<ipc::Service, [u8], ()>>,
}

impl VendorShmLane {
    /// Open a self-connected loopback lane: fresh node, fresh service (unique
    /// name), one publisher port and one subscriber port joined on it.
    ///
    /// # Errors
    ///
    /// `TransportError::Backend` (code `CARRIER_UNREACHABLE`) if the vendor
    /// cannot create the node, service, or either port on this host.
    pub fn loopback() -> Result<Self, TransportError> {
        let seq = LANE_SEQ.fetch_add(1, Ordering::Relaxed);
        let name = format!("ModuKit/S2M4/VendorLane-{}-{seq}", process::id());
        let service_name =
            ServiceName::new(&name).map_err(|e| os_err("service_name", &e.to_string()))?;

        let node = NodeBuilder::new()
            .create::<ipc::Service>()
            .map_err(|e| os_err("node_create", &e.to_string()))?;
        let service = node
            .service_builder(&service_name)
            .publish_subscribe::<[u8]>()
            .history_size(0)
            .max_publishers(1)
            .max_subscribers(1)
            .open_or_create()
            .map_err(|e| os_err("service_open_or_create", &e.to_string()))?;
        let publisher = service
            .publisher_builder()
            .initial_max_slice_len(MAX_FRAME_BYTES)
            .create()
            .map_err(|e| os_err("publisher_create", &e.to_string()))?;
        let subscriber = service
            .subscriber_builder()
            .create()
            .map_err(|e| os_err("subscriber_create", &e.to_string()))?;

        Ok(Self {
            node: Some(node),
            service: Some(service),
            publisher: Some(publisher),
            subscriber: Some(subscriber),
        })
    }
}

impl Transport for VendorShmLane {
    /// The lane is born connected (the ports rendezvous at `loopback`), so
    /// this validates liveness only — the same contract `SocketpairLane`
    /// documents for an already-connected lane.
    fn connect(&mut self) -> Result<(), TransportError> {
        if self.publisher.is_none() || self.subscriber.is_none() {
            return Err(TransportError::Closed);
        }
        Ok(())
    }

    /// Loan a zero-copy slice of exactly `msg.len()` bytes, fill it, commit.
    /// Loan/send failures surface as typed `Backend` errors with the vendor's
    /// message as the prose channel — the code stays a D51 one.
    fn send(&mut self, msg: Vec<u8>) -> Result<(), TransportError> {
        let publisher = self.publisher.as_ref().ok_or(TransportError::Closed)?;
        if msg.len() > MAX_FRAME_BYTES {
            return Err(os_err(
                "send",
                &format!(
                    "message of {} bytes exceeds the lane frame ceiling of {MAX_FRAME_BYTES}",
                    msg.len()
                ),
            ));
        }
        let sample = publisher
            .loan_slice_uninit(msg.len())
            .map_err(|e| os_err("loan", &e.to_string()))?;
        let payload = msg;
        let sample = sample.write_from_fn(|i| payload[i]);
        sample
            .send()
            .map_err(|e| os_err("commit_send", &e.to_string()))?;
        Ok(())
    }

    /// Non-blocking poll of the vendor queue, mapped onto the seam's error
    /// contract: an empty queue is the same typed error `Dummy` raises
    /// (`Backend`, code `CARRIER_UNREACHABLE`), never a fabricated frame; a
    /// received sample's bytes are copied out and the loan releases on drop.
    fn recv(&mut self) -> Result<Vec<u8>, TransportError> {
        let subscriber = self.subscriber.as_ref().ok_or(TransportError::Closed)?;
        match subscriber
            .receive()
            .map_err(|e| os_err("receive", &e.to_string()))?
        {
            Some(sample) => Ok(sample.to_vec()),
            None => Err(os_err("recv", "lane queue is empty")),
        }
    }

    /// Release the ports (and with them the loan ledger) before the node
    /// they belong to. Afterwards every verb reports `Closed`; a second
    /// `close` is a no-op success, matching `Dummy` and `SocketpairLane`.
    fn close(&mut self) -> Result<(), TransportError> {
        self.subscriber = None;
        self.publisher = None;
        self.service = None;
        self.node = None;
        Ok(())
    }
}
