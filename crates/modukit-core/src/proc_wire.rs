//! The `exe` control-socket envelope (S2-M1; modules/01 §1 `exe` row: "wire
//! over socketpair/stdio"). One frame = `u32 LE payload length | u8 type
//! byte | payload`. That is the whole format: the clock-advance variant
//! (D74, supervisor-driven dial) lands in S2-M3, so NOTHING here carries
//! speculative fields for it (plan §1 Verdicts A1 = A).
//!
//! Parse, don't validate: `parse_hello`/`parse_ack` return `Option` and the
//! caller decides the typed error; a malformed frame never reaches the
//! plugin's code paths (04 §3 discipline, `exe` flavor).

use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;

/// Accepted payload ceiling: a broken or lying peer cannot make the host
/// allocate from its length prefix unchecked.
pub const FRAME_MAX: usize = 64 * 1024;

/// child -> host: ABI v1 handshake self-report (D7 tier).
pub const MSG_HELLO: u8 = 1;
/// host -> child: run the plugin's start (03 §2 STARTING).
pub const MSG_START: u8 = 2;
/// host -> child: cooperative stop (01 §3 rule 6; teardown of `exe`
/// escalates to kill on timeout, 01 §3 rule 5).
pub const MSG_STOP: u8 = 3;
/// child -> host: completion of the last verb, payload = `i32 LE` status.
pub const MSG_ACK: u8 = 4;

/// The handshake self-report (D7 tier for the `exe` placement): the ABI
/// major plus the artifact's own `(id, version)` strings and human name,
/// each length-prefixed with a one-byte count.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hello {
    pub abi: u32,
    pub id: String,
    pub version: String,
    pub name: String,
}

/// Write one frame (header + payload, flushed as one write).
///
/// Errors: the peer's socket died or the stream timed out — the caller
/// turns either into a crash observation; nothing here decides policy.
pub fn write_frame(sock: &mut UnixStream, kind: u8, payload: &[u8]) -> io::Result<()> {
    let mut header = Vec::with_capacity(5);
    header.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    header.push(kind);
    sock.write_all(&header)?;
    sock.write_all(payload)?;
    sock.flush()
}

/// Read one frame. EOF mid-header is `UnexpectedEof` (the peer is gone);
/// a `> FRAME_MAX` length is `InvalidData`; the blocking read honors the
/// stream's configured timeout (`WouldBlock`/`TimedOut`).
pub fn read_frame(sock: &mut UnixStream) -> io::Result<(u8, Vec<u8>)> {
    let mut len = [0u8; 4];
    sock.read_exact(&mut len)?;
    let len = usize::try_from(u32::from_le_bytes(len)).unwrap_or(usize::MAX);
    if len > FRAME_MAX {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame length over the wire ceiling",
        ));
    }
    let mut kind = [0u8; 1];
    sock.read_exact(&mut kind)?;
    let mut payload = vec![0u8; len];
    sock.read_exact(&mut payload)?;
    Ok((kind[0], payload))
}

/// Decode a `HELLO` payload: `u32 LE abi | (u8 len, bytes) x (id, version,
/// name)`. Trailing bytes or non-UTF-8 strings refuse the whole frame —
/// silence and lies are both disagreement (the strict `bundle.rs` side).
#[must_use]
pub fn parse_hello(payload: &[u8]) -> Option<Hello> {
    if payload.len() < 4 {
        return None;
    }
    let abi = u32::from_le_bytes(payload[..4].try_into().ok()?);
    let mut rest = &payload[4..];
    let id = read_pstr(&mut rest)?;
    let version = read_pstr(&mut rest)?;
    let name = read_pstr(&mut rest)?;
    if !rest.is_empty() {
        return None;
    }
    Some(Hello {
        abi,
        id,
        version,
        name,
    })
}

/// Encode a `HELLO` payload — the exact mirror of `parse_hello` (S2-M2:
/// the runner's one source of truth for the handshake bytes; the frame
/// format itself is unchanged by this addition). Every string must already
/// fit the wire's one-byte length cap — the caller validates (the runner
/// checks `HELLO_STR_MAX` before speaking); debug builds assert it.
#[must_use]
pub fn hello_payload(hello: &Hello) -> Vec<u8> {
    fn pstr(out: &mut Vec<u8>, s: &str) {
        debug_assert!(
            s.len() <= u8::MAX as usize,
            "hello string over the wire cap"
        );
        out.push(s.len() as u8);
        out.extend_from_slice(s.as_bytes());
    }
    let mut p = hello.abi.to_le_bytes().to_vec();
    pstr(&mut p, &hello.id);
    pstr(&mut p, &hello.version);
    pstr(&mut p, &hello.name);
    p
}

/// Decode an `ACK` payload: exactly one `i32 LE` status (the D27 author
/// code space rides the same int channel the `dl` stop path uses).
#[must_use]
pub fn parse_ack(payload: &[u8]) -> Option<i32> {
    if payload.len() != 4 {
        return None;
    }
    Some(i32::from_le_bytes(payload.try_into().ok()?))
}

fn read_pstr(rest: &mut &[u8]) -> Option<String> {
    let (&n, tail) = rest.split_first()?;
    if tail.len() < usize::from(n) {
        return None;
    }
    let (bytes, after) = tail.split_at(usize::from(n));
    let s = std::str::from_utf8(bytes).ok()?.to_string();
    *rest = after;
    Some(s)
}
