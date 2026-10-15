//! The narrow transport seam (D11, modules/07 §2) and its two Stage-1 local
//! message lanes. The kernel owns the semantics; what this trait hides behind
//! is only "how bytes move" — one small trait, not a middleware worldview.
//!
//! The verbs carry owned values (`Vec<u8>`) and never a reference across the
//! lane (04 §2). No `async`/`await` crosses anything (D6): recv is a blocking
//! read on the underlying primitive, which is fine for a std-thread host.

use std::collections::VecDeque;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use crate::error::TransportError;

/// A local transport lane: the four verbs a rendezvous answers (07 §2).
pub trait Transport {
    /// Establish/validate the lane. Idempotent where the backend is already
    /// connected (a socketpair is born connected).
    ///
    /// # Errors
    ///
    /// `TransportError` if the rendezvous cannot be established or the lane is
    /// already closed.
    fn connect(&mut self) -> Result<(), TransportError>;

    /// Send one owned message.
    ///
    /// # Errors
    ///
    /// `TransportError` on a closed lane or an OS write failure.
    fn send(&mut self, msg: Vec<u8>) -> Result<(), TransportError>;

    /// Receive one message, blocking until a whole frame is available.
    ///
    /// # Errors
    ///
    /// `TransportError` on a closed lane or an OS read failure.
    ///
    /// [`Self::recv`] is the blocking verb; `Dummy` never blocks (it pops its
    /// loopback queue) and returns `TransportError::Backend` when empty.
    fn recv(&mut self) -> Result<Vec<u8>, TransportError>;

    /// Close the lane. A closed lane refuses `send` rather than dropping it.
    ///
    /// # Errors
    ///
    /// `TransportError` on an OS close failure; idempotent on an already
    /// closed lane.
    fn close(&mut self) -> Result<(), TransportError>;
}

/// The in-memory loopback test double (07 §2 — the permanent slot). It keeps
/// the message queue the bus already needs and is the baseline the contract
/// suite compares every real backend against.
#[derive(Debug, Default)]
pub struct Dummy {
    inbox: VecDeque<Vec<u8>>,
    closed: bool,
}

impl Dummy {
    /// A fresh, not-yet-connected loopback.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Transport for Dummy {
    fn connect(&mut self) -> Result<(), TransportError> {
        if self.closed {
            return Err(TransportError::Closed);
        }
        Ok(())
    }

    fn send(&mut self, msg: Vec<u8>) -> Result<(), TransportError> {
        if self.closed {
            return Err(TransportError::Closed);
        }
        self.inbox.push_back(msg);
        Ok(())
    }

    fn recv(&mut self) -> Result<Vec<u8>, TransportError> {
        if self.closed {
            return Err(TransportError::Closed);
        }
        self.inbox
            .pop_front()
            .ok_or_else(|| TransportError::Backend {
                syscall: "recv",
                reason: "loopback queue is empty".to_string(),
            })
    }

    fn close(&mut self) -> Result<(), TransportError> {
        self.closed = true;
        Ok(())
    }
}

/// One end-pair over a real Unix socketpair (07 §2 local-frames slot, std-only:
/// `UnixStream::pair` *is* `socketpair(2)`). It frames each message with a
/// leading `u32` length so the byte stream still honours message boundaries —
/// "local transport may skip serialization but never the message shape"
/// (04 §1). `loopback()` holds both ends so a single value can send and receive
/// against itself, which is what the one contract suite drives.
pub struct SocketpairLane {
    writer: Option<UnixStream>,
    reader: Option<UnixStream>,
}

// The two ends live in this one struct and are only driven from the owning
// thread; no `Send` claim is made across threads here (the seam is single
// threaded per lane at M5 scale).

impl SocketpairLane {
    /// A self-connected socketpair lane (write end + read end).
    ///
    /// `socketpair(2)` on a `AF_UNIX`/`SOCK_STREAM` pair is provided by std,
    /// so no `libc` dependency is needed.
    /// # Errors
    ///
    /// `TransportError::Backend` if the OS `socketpair` call fails.
    pub fn loopback() -> Result<Self, TransportError> {
        // std gives us the pair directly (AF_UNIX / SOCK_STREAM); no `libc`
        // dependency is needed to open a socketpair.
        let (writer, reader) = UnixStream::pair().map_err(|e| TransportError::Backend {
            syscall: "socketpair",
            reason: e.to_string(),
        })?;
        Ok(Self {
            writer: Some(writer),
            reader: Some(reader),
        })
    }
}

impl Transport for SocketpairLane {
    fn connect(&mut self) -> Result<(), TransportError> {
        if self.writer.is_none() || self.reader.is_none() {
            return Err(TransportError::Closed);
        }
        Ok(())
    }

    fn send(&mut self, msg: Vec<u8>) -> Result<(), TransportError> {
        let len = u32::try_from(msg.len()).map_err(|_| TransportError::Backend {
            syscall: "send",
            reason: "message larger than u32 frame length".to_string(),
        })?;
        let writer = self.writer.as_mut().ok_or(TransportError::Closed)?;
        let frame_len = len.to_le_bytes();
        write_all(writer, &frame_len, "send")?;
        write_all(writer, &msg, "send")?;
        writer.flush().map_err(|e| os_err("send", &e.to_string()))
    }

    fn recv(&mut self) -> Result<Vec<u8>, TransportError> {
        let reader = self.reader.as_mut().ok_or(TransportError::Closed)?;
        let mut header = [0u8; 4];
        read_exact(reader, &mut header, "recv")?;
        let len = usize::try_from(u32::from_le_bytes(header))
            .map_err(|e| os_err("recv", &e.to_string()))?;
        let mut buf = vec![0u8; len];
        read_exact(reader, &mut buf, "recv")?;
        Ok(buf)
    }

    fn close(&mut self) -> Result<(), TransportError> {
        // Dropping both ends releases the socketpair; a later send/recv sees
        // `None` and reports `Closed` rather than a half-open lane.
        self.writer = None;
        self.reader = None;
        Ok(())
    }
}

fn os_err(syscall: &'static str, reason: &str) -> TransportError {
    TransportError::Backend {
        syscall,
        reason: reason.to_string(),
    }
}

fn write_all(
    s: &mut UnixStream,
    bytes: &[u8],
    syscall: &'static str,
) -> Result<(), TransportError> {
    s.write_all(bytes)
        .map_err(|e| os_err(syscall, &e.to_string()))
}

fn read_exact(
    s: &mut UnixStream,
    buf: &mut [u8],
    syscall: &'static str,
) -> Result<(), TransportError> {
    match s.read_exact(buf) {
        Ok(()) => Ok(()),
        // A clean EOF on a stream socket means the peer closed: report the
        // lane as closed, never a zero-length "success" that would lie about
        // delivering a frame.
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => Err(TransportError::Closed),
        Err(e) => Err(os_err(syscall, &e.to_string())),
    }
}
