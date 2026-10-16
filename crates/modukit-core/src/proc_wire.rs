//! The `exe` control-socket envelope (S2-M1; modules/01 §1 `exe` row: "wire
//! over socketpair/stdio"). One frame = `u32 LE payload length | u8 wire
//! version | u8 type byte | payload`. The version byte is the S2-M3 frame
//! versioning: every frame carries `WIRE_VERSION`, and a peer that sees a
//! foreign ver rejects it as a typed protocol error (the caller decides —
//! `read_frame` reports the raw byte, it never guesses a policy). S2-M3a
//! added the service frames (REGISTER/CALL/RESULT); S2-M3b added the D74
//! dial pair (DIAL/DUE) — the wire is the product: every frame small,
//! explicit, version-tagged.
//!
//! Parse, don't validate: `parse_hello`/`parse_ack` return `Option` and the
//! caller decides the typed error; a malformed frame never reaches the
//! plugin's code paths (04 §3 discipline, `exe` flavor).

use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use crate::clock::DueJob;

/// Accepted payload ceiling: a broken or lying peer cannot make the host
/// allocate from its length prefix unchecked.
pub const FRAME_MAX: usize = 64 * 1024;

/// The envelope version this build frames and expects (S2-M3: TL-versioned
/// frames). A foreign ver is a protocol disagreement, never a reinterpret.
pub const WIRE_VERSION: u8 = 1;

/// child -> host: ABI v1 handshake self-report (D7 tier).
pub const MSG_HELLO: u8 = 1;
/// host -> child: run the plugin's start (03 §2 STARTING).
pub const MSG_START: u8 = 2;
/// host -> child: cooperative stop (01 §3 rule 6; teardown of `exe`
/// escalates to kill on timeout, 01 §3 rule 5).
pub const MSG_STOP: u8 = 3;
/// child -> host: completion of the last verb, payload = `i32 LE` status.
pub const MSG_ACK: u8 = 4;
/// child -> host (S2-M3a, before the START ack): the plugin published a
/// service — payload = service id + method list (04 §2: every call names
/// its service and method; the list is advertisement, the child's own
/// code stays the unknown-method authority). See `parse_register`.
pub const MSG_REGISTER: u8 = 5;
/// host -> child (S2-M3a): invoke one method of a REGISTERed service —
/// payload = `u32 LE seq | pstr method | u32 LE input len | input`. The
/// seq correlates the RESULT (04 §2: id-correlated request/response, the
/// LSP/Wayland shape). See `parse_call`.
pub const MSG_CALL: u8 = 6;
/// child -> host (S2-M3a): reply to one CALL — payload = `u32 LE seq |
/// i32 LE status | u32 LE output len | output`. status 0 carries bytes;
/// a negative status is the D27 author code verbatim (typed passthrough —
/// the kernel-side proxy surfaces `ServiceCallFailed { code }`, same as
/// the `dl` arm). See `parse_result`.
pub const MSG_RESULT: u8 = 7;
/// host -> child (D74, S2-M3b): turn the embedded Step/Sim dial — payload
/// = `u64 LE delta_micros`. The supervisor is the SOLE authority dialing on
/// the test/replay track; a Real-domain child REFUSES with a typed status.
pub const MSG_DIAL: u8 = 8;
/// child -> host (S2-M3b): answer one dial turn — payload = `i32 LE status
/// | u64 LE fired_at_ns | u16 LE count | (u64 LE token | pstr job) x count`.
/// status 0 carries the due jobs the turn crossed (timers-as-jobs, D20/D6 —
/// the return is data, never callbacks); `DIAL_STATUS_REFUSED_REAL` answers
/// a dial against a Real-domain clock. See `parse_due`.
pub const MSG_DUE: u8 = 9;
/// The documented refusal code: no dial exists on the Real domain (D74).
/// The parent maps it to `KernelError::ClockDialRefused`, never a silent
/// zero-turn success.
pub const DIAL_STATUS_REFUSED_REAL: i32 = -2;

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

/// Write one frame (header + payload, flushed as one write). The envelope
/// stamps `WIRE_VERSION` — speakers do not get to choose their own ver.
///
/// Errors: the peer's socket died or the stream timed out — the caller
/// turns either into a crash observation; nothing here decides policy.
pub fn write_frame(sock: &mut UnixStream, kind: u8, payload: &[u8]) -> io::Result<()> {
    let mut header = Vec::with_capacity(6);
    header.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    header.push(WIRE_VERSION);
    header.push(kind);
    sock.write_all(&header)?;
    sock.write_all(payload)?;
    sock.flush()
}

/// Read one frame, returning the raw `(wire version, type, payload)`.
/// A foreign version is NOT refused here — the caller turns it into the
/// typed handshake/protocol error it owes its doorway (parse, don't
/// validate; `KernelError::WireVersionMismatch` at the kernel doors).
/// EOF mid-header is `UnexpectedEof` (the peer is gone); a
/// `> FRAME_MAX` length is `InvalidData`; the blocking read honors the
/// stream's configured timeout (`WouldBlock`/`TimedOut`).
pub fn read_frame(sock: &mut UnixStream) -> io::Result<(u8, u8, Vec<u8>)> {
    let mut len = [0u8; 4];
    sock.read_exact(&mut len)?;
    let len = usize::try_from(u32::from_le_bytes(len)).unwrap_or(usize::MAX);
    if len > FRAME_MAX {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "frame length over the wire ceiling",
        ));
    }
    let mut head = [0u8; 2];
    sock.read_exact(&mut head)?;
    let mut payload = vec![0u8; len];
    sock.read_exact(&mut payload)?;
    Ok((head[0], head[1], payload))
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

/// A REGISTER payload's decoded shape: what the child's plugin published.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceRegistration {
    pub service_id: String,
    /// Advertised method names (informational in v0: the child's own code
    /// rejects unknown methods, mirroring the `dl` arm byte-exactly).
    pub methods: Vec<String>,
}

/// Decode a `REGISTER` payload: `pstr service_id | u8 count | pstr x count`.
/// Trailing bytes refuse the frame (strict, the `parse_hello` discipline).
#[must_use]
pub fn parse_register(payload: &[u8]) -> Option<ServiceRegistration> {
    let mut rest = payload;
    let service_id = read_pstr(&mut rest)?;
    let (&count, mut rest) = rest.split_first()?;
    let mut methods = Vec::new();
    for _ in 0..count {
        methods.push(read_pstr(&mut rest)?);
    }
    if !rest.is_empty() {
        return None;
    }
    Some(ServiceRegistration {
        service_id,
        methods,
    })
}

/// Encode a `REGISTER` payload — the mirror of `parse_register`; strings
/// must fit the one-byte cap (the runner validates, debug asserts).
#[must_use]
pub fn register_payload(reg: &ServiceRegistration) -> Vec<u8> {
    let mut p = Vec::new();
    push_pstr(&mut p, &reg.service_id);
    debug_assert!(
        reg.methods.len() <= u8::MAX as usize,
        "method list over the cap"
    );
    p.push(reg.methods.len() as u8);
    for m in &reg.methods {
        push_pstr(&mut p, m);
    }
    p
}

/// A CALL payload's decoded shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Call {
    pub seq: u32,
    pub method: String,
    pub input: Vec<u8>,
}

/// Decode a `CALL` payload: `u32 LE seq | pstr method | u32 LE len | bytes`.
#[must_use]
pub fn parse_call(payload: &[u8]) -> Option<Call> {
    if payload.len() < 4 {
        return None;
    }
    let seq = u32::from_le_bytes(payload[..4].try_into().ok()?);
    let mut rest = &payload[4..];
    let method = read_pstr(&mut rest)?;
    if rest.len() < 4 {
        return None;
    }
    let len = usize::try_from(u32::from_le_bytes(rest[..4].try_into().ok()?)).ok()?;
    let bytes = &rest[4..];
    if bytes.len() != len {
        return None;
    }
    Some(Call {
        seq,
        method,
        input: bytes.to_vec(),
    })
}

/// Encode a `CALL` payload — the mirror of `parse_call`.
#[must_use]
pub fn call_payload(seq: u32, method: &str, input: &[u8]) -> Vec<u8> {
    let mut p = seq.to_le_bytes().to_vec();
    push_pstr(&mut p, method);
    p.extend_from_slice(&(input.len() as u32).to_le_bytes());
    p.extend_from_slice(input);
    p
}

/// A RESULT payload's decoded shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceResult {
    pub seq: u32,
    pub status: i32,
    pub output: Vec<u8>,
}

/// Decode a `RESULT` payload: `u32 LE seq | i32 LE status | u32 LE len |
/// bytes`. Trailing bytes refuse the frame.
#[must_use]
pub fn parse_result(payload: &[u8]) -> Option<ServiceResult> {
    if payload.len() < 8 {
        return None;
    }
    let seq = u32::from_le_bytes(payload[..4].try_into().ok()?);
    let status = i32::from_le_bytes(payload[4..8].try_into().ok()?);
    let rest = &payload[8..];
    if rest.len() < 4 {
        return None;
    }
    let len = usize::try_from(u32::from_le_bytes(rest[..4].try_into().ok()?)).ok()?;
    let bytes = &rest[4..];
    if bytes.len() != len {
        return None;
    }
    Some(ServiceResult {
        seq,
        status,
        output: bytes.to_vec(),
    })
}

/// Encode a `RESULT` payload — the mirror of `parse_result`.
#[must_use]
pub fn result_payload(seq: u32, status: i32, output: &[u8]) -> Vec<u8> {
    let mut p = seq.to_le_bytes().to_vec();
    p.extend_from_slice(&status.to_le_bytes());
    p.extend_from_slice(&(output.len() as u32).to_le_bytes());
    p.extend_from_slice(output);
    p
}

/// Encode a `DIAL` payload: the turn as `u64 LE microseconds` (named
/// saturation at `u64::MAX` — a dial that long is a broken parent, and the
/// child's own bounded semantics stay honest either way).
#[must_use]
pub fn dial_payload(delta: Duration) -> Vec<u8> {
    let micros = u64::try_from(delta.as_micros()).unwrap_or(u64::MAX);
    micros.to_le_bytes().to_vec()
}

/// Decode a `DIAL` payload: exactly one `u64 LE` microsecond count.
#[must_use]
pub fn parse_dial(payload: &[u8]) -> Option<Duration> {
    if payload.len() != 8 {
        return None;
    }
    let micros = u64::from_le_bytes(payload[..8].try_into().ok()?);
    Some(Duration::from_micros(micros))
}

/// One dial turn's answer: the refusal/exec status, the clock stamp the
/// turn completed at, and the jobs it crossed (in deadline order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueTurn {
    pub status: i32,
    pub fired_at_ns: u64,
    pub jobs: Vec<DueJob>,
}

/// Encode a `DUE` payload — see `MSG_DUE` for the byte shape.
#[must_use]
pub fn due_payload(turn: &DueTurn) -> Vec<u8> {
    let mut p = turn.status.to_le_bytes().to_vec();
    p.extend_from_slice(&turn.fired_at_ns.to_le_bytes());
    debug_assert!(
        turn.jobs.len() <= u16::MAX as usize,
        "due batch over the wire cap"
    );
    p.extend_from_slice(&(turn.jobs.len() as u16).to_le_bytes());
    for job in &turn.jobs {
        p.extend_from_slice(&job.token.value().to_le_bytes());
        push_pstr(&mut p, &job.job);
    }
    p
}

/// Decode a `DUE` payload. A bad count, a short buffer, or non-UTF-8 job
/// labels refuse the whole frame (the strict `parse_hello` discipline).
#[must_use]
pub fn parse_due(payload: &[u8]) -> Option<DueTurn> {
    if payload.len() < 14 {
        return None;
    }
    let status = i32::from_le_bytes(payload[..4].try_into().ok()?);
    let fired_at_ns = u64::from_le_bytes(payload[4..12].try_into().ok()?);
    let count = usize::from(u16::from_le_bytes(payload[12..14].try_into().ok()?));
    let mut rest = &payload[14..];
    let mut jobs = Vec::new();
    for _ in 0..count {
        if rest.len() < 8 {
            return None;
        }
        let token = u64::from_le_bytes(rest[..8].try_into().ok()?);
        rest = &rest[8..];
        let job = read_pstr(&mut rest)?;
        jobs.push(DueJob {
            token: crate::clock::JobToken::at(token),
            job,
        });
    }
    if !rest.is_empty() {
        return None;
    }
    Some(DueTurn {
        status,
        fired_at_ns,
        jobs,
    })
}

fn push_pstr(out: &mut Vec<u8>, s: &str) {
    debug_assert!(s.len() <= u8::MAX as usize, "wire string over the cap");
    out.push(s.len() as u8);
    out.extend_from_slice(s.as_bytes());
}
