# bindings/py/plugin_runtime.py - the hand-written plugin-wire v1 skin for
# python script children of the modukit kernel (D106 batch 2, plan T2.1:
# docs/wave-2-plugin-door-plan.md section 4). The kernel spawns a script
# bundle as `<runner> <artifact> --fd=N` with the control socketpair injected
# as fd 0 (executor_proc.rs CHILD_CONTROL_FD) and stdout/stderr nulled -
# there is NO console: diagnostics ride the RESULT channel or die with the
# exit code (plan section 0; no log() on purpose).
#
# PYTHON 3.8 FLOOR (pinned by the plan: no walrus, no match, no 3.9+ APIs).
# stdlib only: socket, struct, os, sys.
#
# The wire truth is the CODE, mirrored here function-by-function:
#   crates/modukit-core/src/proc_wire.rs  (framing + per-kind grammars)
#   crates/modukit-runtime/src/main.rs    (the Rust runner's choreography)
#   crates/modukit-core/src/executor_proc.rs (the kernel's exchange pumps)
# Frame envelope (write_frame proc_wire.rs:123-131, read_frame its mirror):
#   [u32 LE payload-length][u8 WIRE_VERSION=1][u8 kind][payload...]
# The length covers the payload only; FRAME_MAX is a TWO-WAY ceiling: this
# skin pre-checks the declared length BEFORE allocating the read buffer
# (proc_wire.rs:140-149 host posture) and before every write.
#
# Choreography confirmed against the code (NOT the plan prose, which inverts
# the REGISTER/ACK order - executor_proc.rs:460-549 breaks the start pump on
# ACK and proc_proxy.rs:103-122 KILLS the child on any stray frame after a
# CALL, so a REGISTER pushed after the ack would be protocol death; the
# Rust runner pushes it BEFORE the ack, modukit-runtime/src/main.rs:221 and
# :349-351, and this skin does the same):
#   HELLO (self-report + caps tail when capable) -> wait START ->
#   push REGISTER per service + FORWARD(SUB) per topic + optional
#   FORWARD(PUB) seed -> bounded DELIVER drain -> ACK(0) ->
#   steady loop { CALL -> on_call -> RESULT(seq echo) ; DELIVER -> on_bus ;
#   DIAL -> DUE(-2) clean refusal ; STOP -> ACK(0) -> cooperative exit ;
#   unknown kinds ignored + counted }
# DELIVER carries NO ack reply (bus_wire.rs Bridge.pump feeds the local bus
# and answers nothing; the reply a bus callback may carry is its own
# FORWARD, never an ACK).
#
# Why the seed/echo dance lives in the START window: the wall is transparent
# ONLY inside it (01 section 3 rule 2; stage-5-plan section 3 names the
# absent post-start pump - ParentFanout.retire sweeps the child's interest
# at the START ack). A FORWARD written by publish() in steady state would
# poison the kernel's next proxy read (proc_proxy.rs "call reply: unexpected
# frame type" -> kill). Callers therefore use publish()/unsubscribe() from
# on_bus inside the window only.
#   shortcut: window-law confinement; upgrade when the steady-state crossing
#   lands (docs/stage-5-plan.md section 3, named agenda).

import os
import socket
import struct
import sys

# --- ledger constants (contracts/modukit_wire.h; parity-gated to proc_wire.rs)
WIRE_VERSION = 1
FRAME_MAX = 65536
MSG_HELLO = 1
MSG_START = 2
MSG_STOP = 3
MSG_ACK = 4
MSG_REGISTER = 5
MSG_CALL = 6
MSG_RESULT = 7
MSG_DIAL = 8
MSG_DUE = 9
MSG_FORWARD = 10
MSG_DELIVER = 11
FWD_PUB = 0
FWD_SUB = 1
FWD_UNSUB = 2
HELLO_CAP_BUS_V2 = 1
# HELLO_CAP_TIMER_V2 = 2 exists on the ledger but is NEVER claimed here:
# a script skin answers DIAL with the clean refusal, so it holds no clock
# side of the deal this volume (plan section 7).
DIAL_STATUS_REFUSED_REAL = -2
# The handshake ABI major the kernel cross-checks at spawn (abi.rs
# ABI_VERSION = 1; executor_proc.rs:292-298 AbiMismatch on any other).
ABI_VERSION = 1
PSTR_MAX = 255
# Bounded pre-ack drain window per read. The parent writes the window's
# DELIVERs before it ever observes the child's ACK (handle_forward drains
# synchronously inside the same read), so a quiet read means "no more
# in-window deliveries" - microseconds on a unix socket, three orders of
# margin in this bound (same discipline as bus_wire.rs PUMP_WINDOW, 1s).
DRAIN_WINDOW = 1.0
# THE single named sabotage point of leg twelve (plan T2.4 stage [4]):
# DEMO_PLUGIN_PY_SABOTAGE=1 skips the REGISTER push - the kernel then has
# no proxy for the plugin's services and the driver's first service_call
# answers NOT_FOUND, failing the leg's lifecycle stage red with no PASS
# line. Read at exactly one place below (the REGISTER loop).
SABOTAGE_ENV = "DEMO_PLUGIN_PY_SABOTAGE"
# Driver-side publish channel for the roundtrip seed: the embed face has no
# bus verb on the sup rung (the two rungs own separate kernels -
# embed_sup.rs header), so the leg's driver hands the seed bytes to the
# child through the spawn environment, the same lever class the runner's
# DEMO_WAVE2_CLOCK_SABOTAGE rides (modukit-runtime/src/main.rs:268).
SEED_ENV = "MODUKIT_PY_SEED"


class SkinError(Exception):
    """Base for every wire disagreement this skin raises."""


class FrameTooLarge(SkinError):
    """Declared payload length over FRAME_MAX - refused BEFORE allocating."""


class FrameReject(SkinError):
    """Strict-grammar disagreement (trailing bytes, length lies, bad op)."""


class WireVersionMismatch(SkinError):
    """A foreign envelope version: protocol disagreement, never a reinterpret."""


class PeerGone(SkinError):
    """EOF mid-frame: the parent is gone; the supervised run is over."""


# ------------------------------------------------------------- framing codec

def encode_frame(kind, payload):
    """One complete frame - the mirror of proc_wire::write_frame. The
    FRAME_MAX pre-check rides the WRITE side too (two-way ceiling)."""
    n = len(payload)
    if n > FRAME_MAX:
        raise FrameTooLarge("outbound payload %d over the ceiling" % n)
    return struct.pack("<IBB", n, WIRE_VERSION, kind) + payload


def decode_header(six_bytes):
    """Split an envelope header into (payload_len, wire_ver, kind)."""
    n, ver, kind = struct.unpack("<IBB", six_bytes)
    return n, ver, kind


def _recv_exactly(sock, n):
    """Blocking read of exactly n bytes; short read = EOF = PeerGone. EINTR
    retries (a signal is not wire noise)."""
    chunks = []
    got = 0
    while got < n:
        try:
            part = sock.recv(n - got)
        except InterruptedError:
            continue
        if not part:
            raise PeerGone("control EOF after %d of %d bytes" % (got, n))
        chunks.append(part)
        got += len(part)
    return b"".join(chunks)


def read_frame(sock):
    """One inbound frame -> (kind, payload). The length is checked against
    FRAME_MAX BEFORE any payload allocation (the host's own posture,
    proc_wire.rs:140-149); a foreign version is a typed disagreement."""
    head = _recv_exactly(sock, 6)
    n, ver, kind = decode_header(head)
    if n > FRAME_MAX:
        raise FrameTooLarge("declared payload %d over the ceiling" % n)
    payload = _recv_exactly(sock, n) if n else b""
    if ver != WIRE_VERSION:
        raise WireVersionMismatch("frame wire version %d, skin frames v%d" % (ver, WIRE_VERSION))
    return kind, payload


def _put_pstr(out, s):
    """pstr = u8 length prefix + UTF-8 bytes (cap 255; a longer string is a
    programming error, refused here, not truncated silently)."""
    b = s.encode("utf-8") if not isinstance(s, bytes) else s
    if len(b) > PSTR_MAX:
        raise FrameReject("pstr over the wire cap (%d bytes)" % len(b))
    out.append(struct.pack("<B", len(b)))
    out.append(b)


def _take_pstr(buf, off):
    """Read one pstr at off -> (text, next_offset). Truncation and non-UTF-8
    refuse the frame (the strict parse_hello discipline: silence and lies are
    both disagreement)."""
    if off >= len(buf):
        raise FrameReject("pstr header past the payload end")
    n = buf[off]
    off += 1
    if len(buf) - off < n:
        raise FrameReject("pstr body truncated")
    b = buf[off:off + n]
    try:
        text = b.decode("utf-8")
    except UnicodeDecodeError:
        raise FrameReject("pstr is not UTF-8")
    return text, off + n


def hello_payload(abi, pid, version, name, caps=0):
    """Mirror of hello_payload/hello_payload_caps: the u32 caps tail is
    TAIL-ADDED and emitted ONLY when non-zero (proc_wire.rs:239-243 plus
    the runner's own choice at main.rs:289-293 - a tail-less legacy
    handshake stays byte-for-byte)."""
    p = [struct.pack("<I", abi)]
    _put_pstr(p, pid)
    _put_pstr(p, version)
    _put_pstr(p, name)
    if caps:
        p.append(struct.pack("<I", caps))
    return b"".join(p)


def parse_hello(payload):
    """u32 LE abi | pstr x3 | [u32 LE caps] - the tail-tolerant
    parse_hello_caps: a 1-3 byte tail or anything past the 4 caps bytes
    still refuses (proc_wire.rs:221-225)."""
    if len(payload) < 4:
        raise FrameReject("HELLO shorter than the ABI word")
    abi = struct.unpack_from("<I", payload, 0)[0]
    off = 4
    pid, off = _take_pstr(payload, off)
    version, off = _take_pstr(payload, off)
    name, off = _take_pstr(payload, off)
    rest = len(payload) - off
    if rest not in (0, 4):
        raise FrameReject("HELLO tail of %d bytes (grammar: 0 or 4)" % rest)
    caps = struct.unpack_from("<I", payload, off)[0] if rest == 4 else 0
    return abi, pid, version, name, caps


def ack_payload(status):
    """Mirror of proc_wire::ack_payload (batch-0 T0.2 twin): exactly one
    i32 LE - the D27 author-code channel."""
    return struct.pack("<i", status)


def parse_ack(payload):
    if len(payload) != 4:
        raise FrameReject("ACK must be exactly 4 bytes")
    return struct.unpack("<i", payload)[0]


def register_payload(service_id, methods):
    """pstr service_id | u8 count | pstr x count (parse_register's mirror)."""
    if len(methods) > PSTR_MAX:
        raise FrameReject("method list over the u8 cap")
    p = []
    _put_pstr(p, service_id)
    p.append(struct.pack("<B", len(methods)))
    for m in methods:
        _put_pstr(p, m)
    return b"".join(p)


def parse_register(payload):
    """Strict: trailing bytes refuse (proc_wire.rs:287-302)."""
    off = 0
    service_id, off = _take_pstr(payload, off)
    if off >= len(payload):
        raise FrameReject("REGISTER missing the method count")
    count = payload[off]
    off += 1
    methods = []
    for _ in range(count):
        m, off = _take_pstr(payload, off)
        methods.append(m)
    if off != len(payload):
        raise FrameReject("REGISTER trailing bytes")
    return service_id, methods


def call_payload(seq, method, input_bytes):
    """u32 LE seq | pstr method | u32 LE len | bytes."""
    p = [struct.pack("<I", seq)]
    _put_pstr(p, method)
    p.append(struct.pack("<I", len(input_bytes)))
    p.append(input_bytes)
    return b"".join(p)


def parse_call(payload):
    """The input length must equal the remaining bytes exactly
    (proc_wire.rs:331-351) - a length lie or trailing junk is a
    disagreement, not a reinterpret."""
    if len(payload) < 4:
        raise FrameReject("CALL shorter than the seq word")
    seq = struct.unpack_from("<I", payload, 0)[0]
    method, off = _take_pstr(payload, 4)
    if len(payload) - off < 4:
        raise FrameReject("CALL missing the input length")
    n = struct.unpack_from("<I", payload, off)[0]
    off += 4
    if len(payload) - off != n:
        raise FrameReject("CALL input length %d, %d bytes present" % (n, len(payload) - off))
    return seq, method, payload[off:]


def result_payload(seq, status, output):
    """u32 LE seq | i32 LE status | u32 LE len | bytes. status 0 carries
    bytes; a negative status is the author's own D27 code, verbatim."""
    p = [struct.pack("<I", seq), struct.pack("<i", status)]
    p.append(struct.pack("<I", len(output)))
    p.append(output)
    return b"".join(p)


def parse_result(payload):
    if len(payload) < 8:
        raise FrameReject("RESULT shorter than seq+status")
    seq, status = struct.unpack_from("<Ii", payload, 0)
    rest = payload[8:]
    if len(rest) < 4:
        raise FrameReject("RESULT missing the output length")
    n = struct.unpack_from("<I", rest, 0)[0]
    if len(rest) - 4 != n:
        raise FrameReject("RESULT output length %d, %d bytes present" % (n, len(rest) - 4))
    return seq, status, rest[4:]


def dial_payload(delta_micros):
    return struct.pack("<Q", delta_micros)


def parse_dial(payload):
    """Exactly one u64 LE microsecond count (proc_wire.rs:417-423)."""
    if len(payload) != 8:
        raise FrameReject("DIAL must be exactly 8 bytes")
    return struct.unpack("<Q", payload)[0]


def due_payload(status, fired_at_ns, jobs):
    """i32 LE status | u64 LE fired_at_ns | u16 LE count |
    (u64 LE token | pstr job) x count."""
    if len(jobs) > 0xFFFF:
        raise FrameReject("DUE batch over the u16 cap")
    p = [struct.pack("<iQ", status, fired_at_ns), struct.pack("<H", len(jobs))]
    for token, job in jobs:
        p.append(struct.pack("<Q", token))
        _put_pstr(p, job)
    return b"".join(p)


def parse_due(payload):
    if len(payload) < 14:
        raise FrameReject("DUE shorter than the fixed header")
    status, fired = struct.unpack_from("<iQ", payload, 0)
    count = struct.unpack_from("<H", payload, 12)[0]
    off = 14
    jobs = []
    for _ in range(count):
        if len(payload) - off < 8:
            raise FrameReject("DUE job token truncated")
        token = struct.unpack_from("<Q", payload, off)[0]
        off += 8
        job, off = _take_pstr(payload, off)
        jobs.append((token, job))
    if off != len(payload):
        raise FrameReject("DUE trailing bytes")
    return status, fired, jobs


def forward_payload(op, topic, qos, value):
    """u8 op | pstr topic | i32 LE qos | u32 LE len | bytes. FWD_SUB and
    FWD_UNSUB leave qos 0 and value empty (the ledger convention)."""
    if op not in (FWD_PUB, FWD_SUB, FWD_UNSUB):
        raise FrameReject("FORWARD op %d off the ledger" % op)
    p = [struct.pack("<B", op)]
    _put_pstr(p, topic)
    p.append(struct.pack("<iI", qos, len(value)))
    p.append(value)
    return b"".join(p)


def parse_forward(payload):
    if not payload:
        raise FrameReject("FORWARD payload empty")
    op = payload[0]
    if op not in (FWD_PUB, FWD_SUB, FWD_UNSUB):
        raise FrameReject("FORWARD op %d unknown" % op)
    topic, off = _take_pstr(payload, 1)
    if len(payload) - off < 8:
        raise FrameReject("FORWARD missing qos+len")
    qos, n = struct.unpack_from("<iI", payload, off)
    off += 8
    if len(payload) - off != n:
        raise FrameReject("FORWARD value length %d, %d bytes present" % (n, len(payload) - off))
    return op, topic, qos, payload[off:]


def deliver_payload(topic, qos, value):
    """pstr topic | i32 LE qos | u32 LE len | bytes. A negative qos is the
    refusal channel (DEFERRAL_DECLINED -26) and carries no value."""
    p = []
    _put_pstr(p, topic)
    p.append(struct.pack("<iI", qos, len(value)))
    p.append(value)
    return b"".join(p)


def parse_deliver(payload):
    topic, off = _take_pstr(payload, 0)
    if len(payload) - off < 8:
        raise FrameReject("DELIVER missing qos+len")
    qos, n = struct.unpack_from("<iI", payload, off)
    off += 8
    if len(payload) - off != n:
        raise FrameReject("DELIVER value length %d, %d bytes present" % (n, len(payload) - off))
    return topic, qos, payload[off:]


# --------------------------------------------------------------- the server

class _Conn(object):
    """One control socket. fromfd DUPLICATES the descriptor, so the original
    fd 0 the kernel injected stays intact (the task's rule: never steal the
    inherited descriptor - its close is the kernel's, and EOF on it is the
    parent-is-gone signal we must observe, not one we can manufacture)."""

    def __init__(self, fd):
        self.sock = socket.fromfd(fd, socket.AF_UNIX, socket.SOCK_STREAM)
        self.unknown_kinds = 0

    def send(self, kind, payload):
        self.sock.sendall(encode_frame(kind, payload))

    def recv(self):
        return read_frame(self.sock)

    def close(self):
        try:
            self.sock.close()
        except OSError:
            pass


_ACTIVE = None


def publish(topic, value, qos=0):
    """FORWARD(FWD_PUB) on the active control socket - the runtime helper the
    plan's bus marshalling demands. Window law (module header): legal from
    an on_bus callback running inside the START-window drain; a steady-state
    call would stray into the proxy's RESULT read and cost the run its
    life - kernel-side, not skin-side, enforcement (proc_proxy.rs:120-122)."""
    if _ACTIVE is None:
        raise SkinError("publish() before serve(): no control socket")
    _ACTIVE.send(MSG_FORWARD, forward_payload(FWD_PUB, topic, qos, value))


def unsubscribe(topic):
    """FORWARD(FWD_UNSUB) - the child speaks it only when its last interest
    in the topic goes away (proc_wire.rs:87-89); same window law as
    publish(). Kept for ledger completeness; no shipped fixture calls it."""
    if _ACTIVE is None:
        raise SkinError("unsubscribe() before serve(): no control socket")
    _ACTIVE.send(MSG_FORWARD, forward_payload(FWD_UNSUB, topic, 0, b""))


def _die(code, why):
    """One-line note then the exit code: stderr is nulled by the kernel so
    "the code is the only channel a host reads (the runner's own die(),"
    "main.rs:75-78); the note still helps a human running standalone."""
    try:
        sys.stderr.write("plugin_runtime: %s\n" % why)
    except OSError:
        pass
    sys.exit(code)


def _parse_fd(argv):
    """The kernel's command shape: `<runner> <artifact> --fd=N`
    (executor_proc.rs:248-251). N is 0 today (stdin swap) but the
    convention, not the number, is what the skin honors."""
    for a in argv:
        if a.startswith("--fd="):
            try:
                fd = int(a[5:])
            except ValueError:
                _die(2, "bad --fd value: %s" % a)
            if fd < 0:
                _die(2, "bad --fd value: %s" % a)
            return fd
    _die(2, "usage: the kernel spawns <script> --fd=N; no --fd found")


def _normalize_topics(topics):
    """topics entries: str (interest only) or (str, seed-bytes) pairs -
    a seeded topic carries its initial FORWARD(PUB) through the START
    window (the wall's only transparent moment; see module header)."""
    out = []
    for t in topics or []:
        if isinstance(t, str):
            out.append((t, None))
        else:
            topic, seed = t
            out.append((topic, seed))
    return out


def _drain_window(conn, on_bus):
    """Bounded pre-ack pump: read DELIVERs until the socket goes quiet
    (inbound FORWARD/unknown kinds are counted, never acted on). The parent
    writes every in-window DELIVER BEFORE it observes our ACK, so a read
    that times out means the window has drained. Cooperative: EOF here is
    the parent dying - exit clean via the caller's PeerGone path."""
    conn.sock.settimeout(DRAIN_WINDOW)
    try:
        while True:
            try:
                kind, payload = conn.recv()
            except socket.timeout:
                return
            if kind == MSG_DELIVER:
                topic, _qos, value = parse_deliver(payload)
                if on_bus is not None:
                    on_bus(topic, value)
            else:
                conn.unknown_kinds += 1
    finally:
        conn.sock.settimeout(None)


def serve(id, version, name, services=None, on_call=None, on_bus=None, topics=None):
    """Walk the wire as a kernel-spawned script child. The API is fixed by
    plan T2.1: services={service_id: [methods]} -> one REGISTER per service;
    topics=[str | (str, seed)] -> one FORWARD(SUB) per topic (and the seed's
    PUB in-window); a bus-only plugin pushes SUBs and NO REGISTER (F8);
    on_call(method, bytes) -> (status, bytes) with status!=0 passthrough;
    on_bus(topic, bytes); DIAL always answers the clean refusal
    DUE(DIAL_STATUS_REFUSED_REAL) - no timer claims this volume.
    Returns {"unknown_kinds": n} when the loop ends (STOP or EOF)."""
    global _ACTIVE
    fd = _parse_fd(sys.argv[1:])
    conn = _Conn(fd)
    _ACTIVE = conn
    services = services or {}
    topics = _normalize_topics(topics)
    # Self-report FIRST (the kernel never speaks before HELLO; read_hello is
    # the spawn's only doorway, executor_proc.rs:355-394). id/version MUST
    # equal the manifest's - the strict cross-check at executor_proc.rs:301-314
    # kills a lie typed (IdentityMismatch); `name` is metadata only.
    caps = HELLO_CAP_BUS_V2 if topics else 0
    try:
        conn.send(MSG_HELLO, hello_payload(ABI_VERSION, id, version, name, caps))
    except OSError:
        _die(6, "control closed before the hello was read")

    stats = {"unknown_kinds": 0}
    try:
        _pump(conn, services, topics, on_call, on_bus)
    except WireVersionMismatch as why:
        _die(12, "foreign frame version: %s" % why)
    except FrameTooLarge as why:
        _die(11, "hostile length refused pre-alloc: %s" % why)
    except FrameReject as why:
        _die(13, "strict-grammar disagreement: %s" % why)
    except PeerGone:
        # Parent hung up: the supervised run is over, leave clean (the
        # runner's own EOF posture, main.rs:334-335 - no atexit tricks,
        # the interpreter unwinds normally from here).
        pass
    finally:
        conn.close()
        _ACTIVE = None
    stats["unknown_kinds"] = conn.unknown_kinds
    return stats


def _pump(conn, services, topics, on_call, on_bus):
    while True:
        kind, payload = conn.recv()
        if kind == MSG_START:
            # payload MUST be empty (the parent writes it with &[];
            # executor_proc.rs:449-455) - anything else is a disagreement.
            if payload:
                raise FrameReject("START carries %d bytes" % len(payload))
            # Announce BEFORE the ack - the start pump breaks on the ACK and
            # the post-ack proxy lane kills on a stray frame (module header).
            if os.environ.get(SABOTAGE_ENV) is None:
                for sid, methods in services.items():
                    conn.send(MSG_REGISTER, register_payload(sid, methods))
            else:
                # THE single named sabotage point (plan T2.4 stage [4]):
                # skip the REGISTER push; nothing else moves.
                pass
            for topic, _seed in topics:
                conn.send(MSG_FORWARD, forward_payload(FWD_SUB, topic, 0, b""))
            for topic, seed in topics:
                if seed is not None:
                    conn.send(MSG_FORWARD, forward_payload(FWD_PUB, topic, 0, seed))
            if topics:
                _drain_window(conn, on_bus)
            conn.send(MSG_ACK, ack_payload(0))
        elif kind == MSG_CALL:
            seq, method, input_bytes = parse_call(payload)
            # The plugin's own code is the unknown-method authority (the
            # ledger keeps REGISTER lists as advertisement only).
            if on_call is None:
                status, output = -1, b""
            else:
                status, output = on_call(method, input_bytes)
            conn.send(MSG_RESULT, result_payload(seq, status, output))
        elif kind == MSG_DELIVER:
            topic, _qos, value = parse_deliver(payload)
            if on_bus is not None:
                on_bus(topic, value)
            # NO ack on DELIVER: the in-code child (bus_wire Bridge.pump)
            # answers nothing; the only reply a callback sends is its own
            # FORWARD. (Ledger "ack-on-deliver rule" - there is none.)
        elif kind == MSG_DIAL:
            parse_dial(payload)  # strict length check (8 bytes)
            conn.send(MSG_DUE, due_payload(DIAL_STATUS_REFUSED_REAL, 0, []))
        elif kind == MSG_STOP:
            if payload:
                raise FrameReject("STOP carries %d bytes" % len(payload))
            # Best-effort reply (main.rs:359-361: a gone parent means the run
            # ended anyway) - then exit cooperatively, kernel reaps with
            # grace (executor_proc.rs:585-619 reads exactly this ACK).
            try:
                conn.send(MSG_ACK, ack_payload(0))
            except OSError:
                pass
            return
        else:
            # Forward-compatible silence, the runner's own steady posture
            # (main.rs:430-432): HELLO/ACK/REGISTER/FORWARD are child->host
            # talkers; any unknown kind is ignored and counted, never
            # reinterpreted (plan EXTENSIBILITY: amendments ride NEW kinds).
            conn.unknown_kinds += 1
