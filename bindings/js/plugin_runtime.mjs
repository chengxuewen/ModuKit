// bindings/js/plugin_runtime.mjs - the hand-written plugin-wire v1 skin for
// node script children of the modukit kernel (D106 batch 3, plan T3.1:
// docs/wave-2-plugin-door-plan.md section 5). The kernel spawns a script
// bundle as `<runner> <artifact> --fd=N` with the control socketpair injected
// as fd 0 (executor_proc.rs CHILD_CONTROL_FD) and stdout/stderr nulled -
// there is NO console: diagnostics ride the RESULT channel or die with the
// exit code (plan section 0; no console.log on purpose).
//
// Node 22 floor, ESM only (no CommonJS dual path until asked). Stdlib only:
// node:net / node:process - offline box, no npm.
//
// The wire truth is the CODE, mirrored here function-by-function:
//   crates/modukit-core/src/proc_wire.rs        (framing + per-kind grammars)
//   crates/modukit-runtime/src/main.rs          (the Rust runner's choreography)
//   crates/modukit-core/src/executor_proc.rs    (the kernel's exchange pumps)
// Frame envelope (write_frame proc_wire.rs:123-131, read_frame its mirror):
//   [u32 LE payload-length][u8 WIRE_VERSION=1][u8 kind][payload...]
// The length covers the payload only; FRAME_MAX is a TWO-WAY ceiling: this
// skin pre-checks the declared length BEFORE allocating the read buffer
// (proc_wire.rs:140-149 host posture) and before every write.
//
// Choreography confirmed against the code (NOT the plan prose, which inverts
// the REGISTER/ACK order - executor_proc.rs:460-549 breaks the start pump on
// ACK and proc_proxy.rs:103-122 KILLS the child on a stray frame after a
// CALL, so a REGISTER pushed after the ack would be protocol death; the
// Rust runner pushes it BEFORE the ack, modukit-runtime/src/main.rs:221 and
// :349-351, and this skin does the same):
//   HELLO (self-report + caps tail when capable) -> wait START ->
//   push REGISTER per service + FORWARD(SUB) per topic + optional
//   FORWARD(PUB) seed -> bounded DELIVER drain -> ACK(0) ->
//   steady loop { CALL -> on_call -> RESULT(seq echo) ; DELIVER -> on_bus ;
//   DIAL -> DUE(-2) clean refusal ; STOP -> ACK(0) -> cooperative exit ;
//   unknown kinds ignored + counted }
// DELIVER carries NO ack reply (bus_wire.rs Bridge.pump feeds the local bus
// and answers nothing; the reply a bus callback may carry is its own
// FORWARD, never an ACK).
//
// Why the seed/echo dance lives in the START window: the wall is transparent
// ONLY inside it (modules doc 01 section 3 rule 2; stage-5-plan section 3
// names the absent post-start pump - ParentFanout.retire sweeps the child's
// interest at the START ack). A FORWARD written by publish() in steady state
// would poison the kernel's next proxy read (proc_proxy.rs "call reply:
// unexpected frame type" -> kill). Callers therefore use publish()/
// unsubscribe() from on_bus inside the window only.
//   shortcut: window-law confinement; upgrade when the steady-state crossing
//   lands (docs/stage-5-plan.md section 3, named agenda).
//
// Channel split note (batch 3, verified against the spawn shape): the kernel
// binds fd 0 to ONE full-duplex unix socketpair end (Stdio::from(OwnedFd) as
// the child's stdin; executor_proc.rs) - reads AND writes ride fd 0, there is
// no second channel (fd 1 is nulled). The py skin dups fd 0 into a fromfd
// socket; node 22's `new net.Socket({fd: 0})` adopts the existing descriptor
// the same way, and responses ride that same Socket (plan T3.1: the v1
// "read-only use" wording was wrong and is fixed here).

import net from 'node:net';
import process from 'node:process';

// --- ledger constants (contracts/modukit_wire.h; parity-gated to proc_wire.rs)
export const WIRE_VERSION = 1;
export const FRAME_MAX = 65536;
export const MSG_HELLO = 1;
export const MSG_START = 2;
export const MSG_STOP = 3;
export const MSG_ACK = 4;
export const MSG_REGISTER = 5;
export const MSG_CALL = 6;
export const MSG_RESULT = 7;
export const MSG_DIAL = 8;
export const MSG_DUE = 9;
export const MSG_FORWARD = 10;
export const MSG_DELIVER = 11;
export const FWD_PUB = 0;
export const FWD_SUB = 1;
export const FWD_UNSUB = 2;
export const HELLO_CAP_BUS_V2 = 1;
// HELLO_CAP_TIMER_V2 = 2 exists on the ledger but is NEVER claimed here:
// a script skin answers DIAL with the clean refusal, so it holds no clock
// side of the deal this volume (plan section 7).
export const DIAL_STATUS_REFUSED_REAL = -2;
// The handshake ABI major the kernel cross-checks at spawn (abi.rs
// ABI_VERSION = 1; executor_proc.rs AbiMismatch on any other).
export const ABI_VERSION = 1;
export const PSTR_MAX = 255;
// Bounded pre-ack drain window per read (ms). The parent writes the window's
// DELIVERs before it ever observes the child's ACK (handle_forward drains
// synchronously inside the same read), so a quiet window means "no more
// in-window deliveries" - microseconds on a unix socket, three orders of
// margin in this bound (same discipline as bus_wire.rs PUMP_WINDOW, 1s).
export const DRAIN_WINDOW_MS = 1000;
// THE single named sabotage point of leg thirteen (plan T3.3): env
// DEMO_PLUGIN_JS_SABOTAGE=1 skips the REGISTER push - the kernel then has
// no proxy for the plugin's services and the driver's first service_call
// answers NOT_FOUND, failing the leg's lifecycle stage red with no PASS
// line. Read at exactly one place below (the REGISTER loop).
export const SABOTAGE_ENV = 'DEMO_PLUGIN_JS_SABOTAGE';
// Driver-side publish channel for the roundtrip seed: the embed face has no
// bus verb on the sup rung (the two rungs own separate kernels -
// embed_sup.rs header), so the leg's driver hands the seed bytes to the
// child through the spawn environment, the same lever class the runner's
// clock lever rides (modukit-runtime/src/main.rs:268).
export const SEED_ENV = 'MODUKIT_JS_SEED';
export const SKIN_ENV = 'MODUKIT_JS_SKIN';

// --- typed errors (the py SkinError family, one for one)

// Base for every wire disagreement this skin raises.
export class SkinError extends Error {}
// Declared payload length over FRAME_MAX - refused BEFORE allocating.
export class FrameTooLarge extends SkinError {}
// Strict-grammar disagreement (trailing bytes, length lies, bad op).
export class FrameReject extends SkinError {}
// A foreign envelope version: protocol disagreement, never a reinterpret.
export class WireVersionMismatch extends SkinError {}
// EOF mid-frame: the parent is gone; the supervised run is over.
export class PeerGone extends SkinError {}

// ------------------------------------------------------------- framing codec

// One complete frame - the mirror of proc_wire::write_frame. The FRAME_MAX
// pre-check rides the WRITE side too (two-way ceiling).
export function encodeFrame(kind, payload) {
  const p = Buffer.isBuffer(payload) ? payload : Buffer.from(payload);
  if (p.length > FRAME_MAX) {
    throw new FrameTooLarge(`outbound payload ${p.length} over the ceiling`);
  }
  const head = Buffer.alloc(6);
  head.writeUInt32LE(p.length, 0);
  head.writeUInt8(WIRE_VERSION, 4);
  head.writeUInt8(kind, 5);
  return Buffer.concat([head, p]);
}

// Split an envelope header into {payloadLen, wireVer, kind}.
export function decodeHeader(sixBytes) {
  return {
    payloadLen: sixBytes.readUInt32LE(0),
    wireVer: sixBytes.readUInt8(4),
    kind: sixBytes.readUInt8(5),
  };
}

// One inbound frame -> {kind, payload}. The length is checked against
// FRAME_MAX BEFORE any payload allocation (the host's own posture,
// proc_wire.rs:140-149); a foreign version is a typed disagreement.
// `read(n)` must resolve exactly n bytes or reject with PeerGone (an empty
// slice is EOF - the _recv_exactly rule).
export async function readFrame(read) {
  const head = await readExact(read, 6);
  const { payloadLen, wireVer, kind } = decodeHeader(head);
  if (payloadLen > FRAME_MAX) {
    throw new FrameTooLarge(`declared payload ${payloadLen} over the ceiling`);
  }
  const payload = payloadLen ? await readExact(read, payloadLen) : Buffer.alloc(0);
  if (wireVer !== WIRE_VERSION) {
    throw new WireVersionMismatch(`frame wire version ${wireVer}, skin frames v${WIRE_VERSION}`);
  }
  return { kind, payload };
}

async function readExact(read, n) {
  const chunks = [];
  let got = 0;
  while (got < n) {
    const part = await read(n - got);
    if (!part || part.length === 0) {
      throw new PeerGone(`control EOF after ${got} of ${n} bytes`);
    }
    chunks.push(part);
    got += part.length;
  }
  return Buffer.concat(chunks);
}

// pstr = u8 length prefix + UTF-8 bytes (cap 255; a longer string is a
// programming error, refused here, not truncated silently).
export function putPstr(out, s) {
  const b = Buffer.isBuffer(s) ? s : Buffer.from(s, 'utf-8');
  if (b.length > PSTR_MAX) {
    throw new FrameReject(`pstr over the wire cap (${b.length} bytes)`);
  }
  const len = Buffer.alloc(1);
  len.writeUInt8(b.length, 0);
  out.push(len, b);
}

// Read one pstr at off -> [text, nextOffset]. Truncation and non-UTF-8
// refuse the frame (the strict parse_hello discipline: silence and lies are
// both disagreement). The round-trip compare is the UTF-8 validator node
// lacks: Buffer.toString never throws, it silently substitutes U+FFFD.
export function takePstr(buf, off) {
  if (off >= buf.length) throw new FrameReject('pstr header past the payload end');
  const n = buf.readUInt8(off);
  off += 1;
  if (buf.length - off < n) throw new FrameReject('pstr body truncated');
  const b = buf.subarray(off, off + n);
  const text = b.toString('utf-8');
  if (!Buffer.from(text, 'utf-8').equals(b)) {
    throw new FrameReject('pstr is not UTF-8');
  }
  return [text, off + n];
}

// Mirror of hello_payload/hello_payload_caps: the u32 caps tail is
// TAIL-ADDED and emitted ONLY when non-zero (proc_wire.rs:239-243 plus the
// runner's own choice at main.rs:289-293 - a tail-less legacy handshake
// stays byte-for-byte).
export function helloPayload(abi, pid, version, name, caps = 0) {
  const p = [];
  const w = Buffer.alloc(4);
  w.writeUInt32LE(abi >>> 0, 0);
  p.push(w);
  putPstr(p, pid);
  putPstr(p, version);
  putPstr(p, name);
  if (caps) {
    const c = Buffer.alloc(4);
    c.writeUInt32LE(caps, 0);
    p.push(c);
  }
  return Buffer.concat(p);
}

// u32 LE abi | pstr x3 | [u32 LE caps] - the tail-tolerant parse_hello_caps:
// a 1-3 byte tail or anything past the 4 caps bytes still refuses
// (proc_wire.rs:221-225).
export function parseHello(payload) {
  if (payload.length < 4) throw new FrameReject('HELLO shorter than the ABI word');
  const abi = payload.readUInt32LE(0);
  let off = 4;
  let pid, version, name;
  [pid, off] = takePstr(payload, off);
  [version, off] = takePstr(payload, off);
  [name, off] = takePstr(payload, off);
  const rest = payload.length - off;
  if (rest !== 0 && rest !== 4) {
    throw new FrameReject(`HELLO tail of ${rest} bytes (grammar: 0 or 4)`);
  }
  const caps = rest === 4 ? payload.readUInt32LE(off) : 0;
  return { abi, pid, version, name, caps };
}

// Mirror of proc_wire::ack_payload (batch-0 T0.2 twin): exactly one
// i32 LE - the D27 author-code channel.
export function ackPayload(status) {
  const b = Buffer.alloc(4);
  b.writeInt32LE(status, 0);
  return b;
}

export function parseAck(payload) {
  if (payload.length !== 4) throw new FrameReject('ACK must be exactly 4 bytes');
  return payload.readInt32LE(0);
}

// pstr service_id | u8 count | pstr x count (parse_register's mirror).
export function registerPayload(serviceId, methods) {
  if (methods.length > PSTR_MAX) {
    throw new FrameReject('method list over the u8 cap');
  }
  const p = [];
  putPstr(p, serviceId);
  const count = Buffer.alloc(1);
  count.writeUInt8(methods.length, 0);
  p.push(count);
  for (const m of methods) putPstr(p, m);
  return Buffer.concat(p);
}

// Strict: trailing bytes refuse (proc_wire.rs:287-302).
export function parseRegister(payload) {
  let off = 0;
  let serviceId;
  [serviceId, off] = takePstr(payload, off);
  if (off >= payload.length) throw new FrameReject('REGISTER missing the method count');
  const count = payload.readUInt8(off);
  off += 1;
  const methods = [];
  for (let i = 0; i < count; i++) {
    let m;
    [m, off] = takePstr(payload, off);
    methods.push(m);
  }
  if (off !== payload.length) throw new FrameReject('REGISTER trailing bytes');
  return { serviceId, methods };
}

// u32 LE seq | pstr method | u32 LE len | bytes.
export function callPayload(seq, method, inputBytes) {
  const p = [];
  const s = Buffer.alloc(4);
  s.writeUInt32LE(seq >>> 0, 0);
  p.push(s);
  putPstr(p, method);
  const len = Buffer.alloc(4);
  len.writeUInt32LE(inputBytes.length, 0);
  p.push(len, inputBytes);
  return Buffer.concat(p);
}

// The input length must equal the remaining bytes exactly
// (proc_wire.rs:331-351) - a length lie or trailing junk is a
// disagreement, not a reinterpret.
export function parseCall(payload) {
  if (payload.length < 4) throw new FrameReject('CALL shorter than the seq word');
  const seq = payload.readUInt32LE(0);
  let off;
  let method;
  [method, off] = takePstr(payload, 4);
  if (payload.length - off < 4) throw new FrameReject('CALL missing the input length');
  const n = payload.readUInt32LE(off);
  off += 4;
  if (payload.length - off !== n) {
    throw new FrameReject(`CALL input length ${n}, ${payload.length - off} bytes present`);
  }
  return { seq, method, input: payload.subarray(off) };
}

// u32 LE seq | i32 LE status | u32 LE len | bytes. status 0 carries
// bytes; a negative status is the author's own D27 code, verbatim.
export function resultPayload(seq, status, output) {
  const p = [];
  const s = Buffer.alloc(4);
  s.writeUInt32LE(seq >>> 0, 0);
  const st = Buffer.alloc(4);
  st.writeInt32LE(status, 0);
  p.push(s, st);
  const len = Buffer.alloc(4);
  len.writeUInt32LE(output.length, 0);
  p.push(len, output);
  return Buffer.concat(p);
}

export function parseResult(payload) {
  if (payload.length < 8) throw new FrameReject('RESULT shorter than seq+status');
  const seq = payload.readUInt32LE(0);
  const status = payload.readInt32LE(4);
  const rest = payload.subarray(8);
  if (rest.length < 4) throw new FrameReject('RESULT missing the output length');
  const n = rest.readUInt32LE(0);
  if (rest.length - 4 !== n) {
    throw new FrameReject(`RESULT output length ${n}, ${rest.length - 4} bytes present`);
  }
  return { seq, status, output: rest.subarray(4) };
}

export function dialPayload(deltaMicros) {
  const b = Buffer.alloc(8);
  b.writeBigUInt64LE(BigInt(deltaMicros));
  return b;
}

// Exactly one u64 LE microsecond count (proc_wire.rs:417-423).
export function parseDial(payload) {
  if (payload.length !== 8) throw new FrameReject('DIAL must be exactly 8 bytes');
  return payload.readBigUInt64LE(0);
}

// i32 LE status | u64 LE fired_at_ns | u16 LE count |
// (u64 LE token | pstr job) x count.
export function duePayload(status, firedAtNs, jobs) {
  if (jobs.length > 0xffff) throw new FrameReject('DUE batch over the u16 cap');
  const p = [];
  const fixed = Buffer.alloc(14);
  fixed.writeInt32LE(status, 0);
  fixed.writeBigUInt64LE(BigInt(firedAtNs), 4);
  fixed.writeUInt16LE(jobs.length, 12);
  p.push(fixed);
  for (const [token, job] of jobs) {
    const t = Buffer.alloc(8);
    t.writeBigUInt64LE(BigInt(token), 0);
    p.push(t);
    putPstr(p, job);
  }
  return Buffer.concat(p);
}

export function parseDue(payload) {
  if (payload.length < 14) throw new FrameReject('DUE shorter than the fixed header');
  const status = payload.readInt32LE(0);
  const fired = payload.readBigUInt64LE(4);
  const count = payload.readUInt16LE(12);
  let off = 14;
  const jobs = [];
  for (let i = 0; i < count; i++) {
    if (payload.length - off < 8) throw new FrameReject('DUE job token truncated');
    const token = payload.readBigUInt64LE(off);
    off += 8;
    let job;
    [job, off] = takePstr(payload, off);
    jobs.push([token, job]);
  }
  if (off !== payload.length) throw new FrameReject('DUE trailing bytes');
  return { status, fired, jobs };
}

// u8 op | pstr topic | i32 LE qos | u32 LE len | bytes. FWD_SUB and
// FWD_UNSUB leave qos 0 and value empty (the ledger convention).
export function forwardPayload(op, topic, qos, value) {
  if (op !== FWD_PUB && op !== FWD_SUB && op !== FWD_UNSUB) {
    throw new FrameReject(`FORWARD op ${op} off the ledger`);
  }
  const p = [];
  const o = Buffer.alloc(1);
  o.writeUInt8(op, 0);
  p.push(o);
  putPstr(p, topic);
  const tail = Buffer.alloc(8);
  tail.writeInt32LE(qos, 0);
  tail.writeUInt32LE(value.length, 4);
  p.push(tail, value);
  return Buffer.concat(p);
}

export function parseForward(payload) {
  if (payload.length === 0) throw new FrameReject('FORWARD payload empty');
  const op = payload.readUInt8(0);
  if (op !== FWD_PUB && op !== FWD_SUB && op !== FWD_UNSUB) {
    throw new FrameReject(`FORWARD op ${op} unknown`);
  }
  let off;
  let topic;
  [topic, off] = takePstr(payload, 1);
  if (payload.length - off < 8) throw new FrameReject('FORWARD missing qos+len');
  const qos = payload.readInt32LE(off);
  const n = payload.readUInt32LE(off + 4);
  off += 8;
  if (payload.length - off !== n) {
    throw new FrameReject(`FORWARD value length ${n}, ${payload.length - off} bytes present`);
  }
  return { op, topic, qos, value: payload.subarray(off) };
}

// pstr topic | i32 LE qos | u32 LE len | bytes. A negative qos is the
// refusal channel (DEFERRAL_DECLINED -26) and carries no value.
export function deliverPayload(topic, qos, value) {
  const p = [];
  putPstr(p, topic);
  const tail = Buffer.alloc(8);
  tail.writeInt32LE(qos, 0);
  tail.writeUInt32LE(value.length, 4);
  p.push(tail, value);
  return Buffer.concat(p);
}

export function parseDeliver(payload) {
  let off;
  let topic;
  [topic, off] = takePstr(payload, 0);
  if (payload.length - off < 8) throw new FrameReject('DELIVER missing qos+len');
  const qos = payload.readInt32LE(off);
  const n = payload.readUInt32LE(off + 4);
  off += 8;
  if (payload.length - off !== n) {
    throw new FrameReject(`DELIVER value length ${n}, ${payload.length - off} bytes present`);
  }
  return { topic, qos, value: payload.subarray(off) };
}

// --------------------------------------------------------------- the server

class Conn {
  // One control socket. net.Socket({fd}) ADOPTS the existing descriptor
  // without dup2: the kernel-injected end's lifecycle stays the kernel's,
  // and EOF on it is the parent-is-gone signal we must observe, not one we
  // can manufacture (the py skin's fromfd rule, node-shaped).
  constructor(fd) {
    this.sock = new net.Socket({ fd, readable: true, writable: true });
    this.unknownKinds = 0;
    this.pending = Buffer.alloc(0);
    this.eof = false;
    this.err = null;
    this.waiter = null;
    // Data arrival is owned by THESE listeners alone: bytes always land in
    // pending, and a parked read is served from there - so a timed-out
    // (abandoned) read can never lose or double-consume a byte; consumption
    // stays atomic inside serve().
    this.sock.on('data', (chunk) => {
      this.pending = Buffer.concat([this.pending, chunk]);
      this.serve();
    });
    this.sock.on('end', () => {
      this.eof = true;
      this.serve();
    });
    this.sock.on('error', (e) => {
      this.err = e;
      this.serve();
    });
  }

  // Serve the parked read out of pending; EOF/error settle it typed.
  serve() {
    const w = this.waiter;
    if (!w) return;
    if (this.pending.length >= w.need) {
      this.waiter = null;
      const out = this.pending.subarray(0, w.need);
      this.pending = this.pending.subarray(w.need);
      w.resolve(out);
      return;
    }
    if (this.err) {
      this.waiter = null;
      w.reject(new PeerGone(`control error: ${this.err.message}`));
      return;
    }
    if (this.eof) {
      this.waiter = null;
      w.reject(new PeerGone(`control EOF with ${this.pending.length} of ${w.need} bytes in hand`));
    }
  }

  send(kind, payload) {
    this.sock.write(encodeFrame(kind, payload));
  }

  // Exactly n bytes or a typed PeerGone (the _recv_exactly rule). The
  // single waiter slot keeps reads serialized by construction.
  read(n) {
    return new Promise((resolve, reject) => {
      this.waiter = { need: n, resolve, reject };
      this.serve();
    });
  }

  async recv() {
    return readFrame((n) => this.read(n));
  }

  // One frame or null when nothing arrives within ms. An abandoned parked
  // read is simply replaced by the next recv() - its promise never settles,
  // and mid-frame bytes stay in pending (the node shape of the py skin's
  // settimeout window; no byte is lost across the abandonment).
  recvQuiet(ms) {
    let timer;
    const timeout = new Promise((res) => {
      timer = setTimeout(() => res(null), ms);
    });
    return Promise.race([this.recv(), timeout]).finally(() => clearTimeout(timer));
  }

  close() {
    try {
      this.sock.destroy();
    } catch {
      // best effort - the py close() posture
    }
  }
}

let ACTIVE = null;

// FORWARD(FWD_PUB) on the active control socket - the runtime helper the
// plan's bus marshalling demands. Window law (module header): legal from
// an on_bus callback running inside the START-window drain; a steady-state
// call would stray into the proxy's RESULT read and cost the run its
// life - kernel-side, not skin-side, enforcement (proc_proxy.rs:120-122).
export function publish(topic, value, qos = 0) {
  if (ACTIVE === null) {
    throw new SkinError('publish() before serve(): no control socket');
  }
  ACTIVE.send(MSG_FORWARD, forwardPayload(FWD_PUB, topic, qos, value));
}

// FORWARD(FWD_UNSUB) - the child speaks it only when its last interest
// in the topic goes away (proc_wire.rs:87-89); same window law as
// publish(). Kept for ledger completeness; no shipped fixture calls it.
export function unsubscribe(topic) {
  if (ACTIVE === null) {
    throw new SkinError('unsubscribe() before serve(): no control socket');
  }
  ACTIVE.send(MSG_FORWARD, forwardPayload(FWD_UNSUB, topic, 0, Buffer.alloc(0)));
}

function die(code, why) {
  // One-line note then the exit code: stderr is nulled by the kernel so
  // "the code is the only channel a host reads" (the runner's own die(),
  // main.rs:75-78); the note still helps a human running standalone.
  try {
    process.stderr.write(`plugin_runtime: ${why}\n`);
  } catch {
    // console-less posture: the note is best effort
  }
  process.exit(code);
}

// The kernel's command shape: `<runner> <artifact> --fd=N`
// (executor_proc.rs:248-251). N is 0 today (stdin swap) but the
// convention, not the number, is what the skin honors.
function parseFd(argv) {
  for (const a of argv) {
    if (a.startsWith('--fd=')) {
      const fd = Number(a.slice(5));
      if (!Number.isInteger(fd) || fd < 0) {
        die(2, `bad --fd value: ${a}`);
      }
      return fd;
    }
  }
  die(2, 'usage: the kernel spawns <script> --fd=N; no --fd found');
  return -1; // unreachable (die exits)
}

// topics entries: string (interest only) or [string, seed-bytes] pairs -
// a seeded topic carries its initial FORWARD(PUB) through the START
// window (the wall's only transparent moment; see module header).
function normalizeTopics(topics) {
  const out = [];
  for (const t of topics ?? []) {
    if (typeof t === 'string') {
      out.push([t, null]);
    } else {
      out.push([t[0], t[1]]);
    }
  }
  return out;
}

// Bounded pre-ack pump: read DELIVERs until the socket goes quiet
// (inbound FORWARD/unknown kinds are counted, never acted on). The parent
// writes every in-window DELIVER BEFORE it observes our ACK, so a read
// that times out means the window has drained. Cooperative: EOF here is
// the parent dying - exit clean via the caller's PeerGone path. The one
// event-loop caveat: a timeout only fires when the loop is otherwise idle,
// so a synchronous-forever onBus would hang the window - node's callback
// shape is async by contract (plan T3.1), and the fixtures are.
async function drainWindow(conn, onBus) {
  for (;;) {
    const frame = await conn.recvQuiet(DRAIN_WINDOW_MS);
    if (frame === null) return;
    if (frame.kind === MSG_DELIVER) {
      const { topic, value } = parseDeliver(frame.payload);
      if (onBus) await onBus(topic, value);
    } else {
      conn.unknownKinds += 1;
    }
  }
}

// Walk the wire as a kernel-spawned script child. The API is fixed by
// plan T3.1 (mirror of the py serve()): services={serviceId: [methods]} ->
// one REGISTER per service; topics=[string | [string, seed]] -> one
// FORWARD(SUB) per topic (and the seed's PUB in-window); a bus-only plugin
// pushes SUBs and NO REGISTER (F8); onCall(method, bytes) -> [status,
// bytes] with status!=0 passthrough (sync or async); onBus(topic, bytes);
// DIAL always answers the clean refusal DUE(DIAL_STATUS_REFUSED_REAL) -
// no timer claims this volume. Returns {unknownKinds} when the loop ends
// (STOP or EOF).
export async function serve(opts) {
  const fd = parseFd(process.argv.slice(2));
  const conn = new Conn(fd);
  ACTIVE = conn;
  const services = opts.services ?? {};
  const topics = normalizeTopics(opts.topics);
  // Self-report FIRST (the kernel never speaks before HELLO; read_hello is
  // the spawn's only doorway, executor_proc.rs:355-394). id/version MUST
  // equal the manifest's - the strict cross-check at executor_proc.rs:301-314
  // kills a lie typed (IdentityMismatch); `name` is metadata only.
  const caps = topics.length ? HELLO_CAP_BUS_V2 : 0;
  try {
    conn.send(MSG_HELLO, helloPayload(ABI_VERSION, opts.id, opts.version, opts.name, caps));
  } catch (e) {
    die(6, `control closed before the hello was read: ${e.message}`);
  }

  try {
    await pump(conn, services, topics, opts.onCall, opts.onBus);
  } catch (e) {
    if (e instanceof WireVersionMismatch) {
      die(12, `foreign frame version: ${e.message}`);
    } else if (e instanceof FrameTooLarge) {
      die(11, `hostile length refused pre-alloc: ${e.message}`);
    } else if (e instanceof FrameReject) {
      die(13, `strict-grammar disagreement: ${e.message}`);
    } else if (e instanceof PeerGone) {
      // Parent hung up: the supervised run is over, leave clean (the
      // runner's own EOF posture, main.rs:334-335 - no process.exit(0)
      // call: the event loop simply drains and node exits 0 naturally,
      // mirroring the py "the interpreter unwinds normally" rule).
    } else {
      throw e;
    }
  } finally {
    conn.close();
    ACTIVE = null;
  }
  return { unknownKinds: conn.unknownKinds };
}

async function pump(conn, services, topics, onCall, onBus) {
  for (;;) {
    const { kind, payload } = await conn.recv();
    if (kind === MSG_START) {
      // payload MUST be empty (the parent writes it with &[];
      // executor_proc.rs:449-455) - anything else is a disagreement.
      if (payload.length) {
        throw new FrameReject(`START carries ${payload.length} bytes`);
      }
      // Announce BEFORE the ack - the start pump breaks on the ACK and
      // the post-ack proxy lane kills on a stray frame (module header).
      if (process.env[SABOTAGE_ENV] === undefined) {
        for (const sid of Object.keys(services)) {
          conn.send(MSG_REGISTER, registerPayload(sid, services[sid]));
        }
      } else {
        // THE single named sabotage point (plan T3.3): skip the REGISTER
        // push; nothing else moves.
      }
      for (const [topic] of topics) {
        conn.send(MSG_FORWARD, forwardPayload(FWD_SUB, topic, 0, Buffer.alloc(0)));
      }
      for (const [topic, seed] of topics) {
        if (seed !== null) {
          conn.send(MSG_FORWARD, forwardPayload(FWD_PUB, topic, 0, seed));
        }
      }
      if (topics.length) {
        await drainWindow(conn, onBus);
      }
      conn.send(MSG_ACK, ackPayload(0));
    } else if (kind === MSG_CALL) {
      const { seq, method, input } = parseCall(payload);
      // The plugin's own code is the unknown-method authority (the
      // ledger keeps REGISTER lists as advertisement only).
      let status;
      let output;
      if (onCall === undefined || onCall === null) {
        [status, output] = [-1, Buffer.alloc(0)];
      } else {
        [status, output] = await onCall(method, input);
      }
      conn.send(MSG_RESULT, resultPayload(seq, status, output));
    } else if (kind === MSG_DELIVER) {
      const { topic, value } = parseDeliver(payload);
      if (onBus) await onBus(topic, value);
      // NO ack on DELIVER: the in-code child (bus_wire Bridge.pump)
      // answers nothing; the only reply a callback sends is its own
      // FORWARD. (Ledger "ack-on-deliver rule" - there is none.)
    } else if (kind === MSG_DIAL) {
      parseDial(payload); // strict length check (8 bytes)
      conn.send(MSG_DUE, duePayload(DIAL_STATUS_REFUSED_REAL, 0, []));
    } else if (kind === MSG_STOP) {
      if (payload.length) {
        throw new FrameReject(`STOP carries ${payload.length} bytes`);
      }
      // Best-effort reply (main.rs:359-361: a gone parent means the run
      // ended anyway) - then exit cooperatively, kernel reaps with
      // grace (executor_proc.rs:585-619 reads exactly this ACK).
      try {
        conn.send(MSG_ACK, ackPayload(0));
      } catch {
        // best effort - the gone-parent case
      }
      return;
    } else {
      // Forward-compatible silence, the runner's own steady posture
      // (main.rs:430-432): HELLO/ACK/REGISTER/FORWARD are child->host
      // talkers; any unknown kind is ignored and counted, never
      // reinterpreted (plan EXTENSIBILITY: amendments ride NEW kinds).
      conn.unknownKinds += 1;
    }
  }
}
