/* modukit_wire.h - the plugin wire v1 LEDGER (D106 batch 1, plan T1.1).
 *
 * The `exe`-placement control-channel contract between the kernel (parent)
 * and a process plugin (child runner), as implemented by
 * crates/modukit-core/src/proc_wire.rs. THIS FILE DECLARES NOTHING THAT
 * COMPILES - no types, no functions, no include guard. It is the readable
 * cross-language spec the batch-2/3 skins (Python, Node) code against,
 * hand-authored, pinned to the Rust implementation by
 * scripts/check-wire-parity.sh (every "NAME = value" line in the constant
 * table below must match its `pub const` twin, both directions).
 *
 * The control channel rides the child's fd 0 (stdin swap, `--fd=0` argv
 * convention); stdout and stderr are nulled by the kernel - scripts get no
 * console, diagnostics ride the RESULT channel or the reader face.
 *
 * Frame envelope - EXACTLY six header bytes, verbatim from write_frame
 * (proc_wire.rs:123-131; read_frame proc_wire.rs:140-155 is its mirror):
 *
 *   [u32 LE payload-length][u8 WIRE_VERSION=1][u8 kind][payload...]
 *
 * The length covers the PAYLOAD ONLY; the 6-byte header is not counted.
 * The version byte is stamped by the framer - speakers never choose their
 * own version (proc_wire.rs:118-119 doc). A foreign version is a protocol
 * disagreement, never a reinterpret: the kernel doors surface
 * KernelError::WireVersionMismatch (read_frame reports the raw byte,
 * proc_wire.rs:133-136 doc) and the runner dies typed, exit 12
 * (crates/modukit-runtime/src/main.rs:341-346).
 *
 * FRAME_MAX is a TWO-WAY ceiling. The read side enforces it BEFORE
 * allocating - a length above it refuses the frame as InvalidData
 * (proc_wire.rs:144-149). The write side has no built-in cap: writers MUST
 * pre-check their payload length against FRAME_MAX before write or alloc
 * (declared here; the check is hosted by the skin conformance stages,
 * docs/wave-2-plugin-door-plan.md section 6).
 *
 * Field grammar used by the per-kind lines below:
 *   pstr     = u8 length prefix + that many UTF-8 bytes, cap 255
 *              (read_pstr proc_wire.rs:264-273, push_pstr :485-489;
 *              HELLO_STR_MAX = u8::MAX, modukit-runtime/src/main.rs:54).
 *   bytes    = u32 LE opaque length + that many raw bytes. The u32-LE
 *              length belongs to the OPAQUE payloads (CALL input, RESULT
 *              output, FORWARD/DELIVER value) - never to a pstr string;
 *              the two are distinct field shapes (plan section 0 fixed
 *              the v1 conflation).
 *   uN/iN LE = fixed-width little-endian integer, N bits.
 */

/* -------------------------------------------------------------------
 * Constant ledger. The "NAME = value" lines below are machine-pinned
 * to the `pub const` items of proc_wire.rs by check-wire-parity.sh;
 * keep name, value, and decimal form in sync with the Rust side.
 * -------------------------------------------------------------------
 *
 * WIRE_VERSION = 1
 *   u8 envelope byte. Every frame carries it; a foreign ver is refused.
 * FRAME_MAX = 65536
 *   usize payload ceiling (Rust writes it 64 * 1024). Two-way: read-side
 *   enforced, writers must pre-check (see envelope block above).
 * MSG_HELLO = 1
 *   u8 kind, child -> host: handshake self-report (D7 tier).
 * MSG_START = 2
 *   u8 kind, host -> child: run the plugin start (03 section 2 STARTING).
 * MSG_STOP = 3
 *   u8 kind, host -> child: cooperative stop; timeout escalates to kill.
 * MSG_ACK = 4
 *   u8 kind, child -> host: completion of the last verb, i32 status.
 * MSG_REGISTER = 5
 *   u8 kind, child -> host: service self-report, PUSHED after START. The
 *   kernel never sends REGISTER; the child announces (plan section 0 -
 *   v1's dispatcher description was inverted).
 * MSG_CALL = 6
 *   u8 kind, host -> child: invoke one method of a REGISTERed service.
 * MSG_RESULT = 7
 *   u8 kind, child -> host: reply to one CALL (seq-correlated).
 * MSG_DIAL = 8
 *   u8 kind, host -> child: turn the embedded Step/Sim dial (D74).
 * MSG_DUE = 9
 *   u8 kind, child -> host: answer one dial turn (due jobs as data).
 * MSG_FORWARD = 10
 *   u8 kind, child -> host: the bus talker (Stage-5 N2, D96 ruling 3);
 *   the subscription op rides inside the payload (see FWD_ rows).
 * MSG_DELIVER = 11
 *   u8 kind, host -> child: the parent fan-out across the wall (D96
 *   ruling 6: self-echo included).
 * DIAL_STATUS_REFUSED_REAL = -2
 *   i32 DUE status: no dial exists on the Real domain (D74). The parent
 *   maps it to KernelError::ClockDialRefused, never a silent zero-turn
 *   success (proc_wire.rs:64-67 doc). The clean-refusal code the script
 *   skins answer DIAL with this volume (plan section 7).
 * FWD_PUB = 0
 *   FORWARD op: publish topic with the embed qos int and value bytes.
 * FWD_SUB = 1
 *   FORWARD op: register interest in topic; retained-latest snapshots
 *   return as DELIVER frames (D96 ruling 6, late-join across the wall).
 * FWD_UNSUB = 2
 *   FORWARD op: drop interest in topic (spoken when the child's LAST
 *   queue for the topic goes away).
 * HELLO_CAP_BUS_V2 = 1
 *   u32 HELLO caps bit (D96 ruling 4): declaring child is spoken to on
 *   the bus talkers; a silent (tail-less, old) child never is.
 * HELLO_CAP_TIMER_V2 = 2
 *   u32 HELLO caps bit (D99 R5-A): the timer crossing - own bells
 *   FORWARD up, kernel bells DELIVER back.
 */

/* -------------------------------------------------------------------
 * Per-kind payload grammar. Field order is transcribed from each
 * parse_X/payload_X pair in proc_wire.rs (both cited).
 *
 *   MSG_HELLO   abi u32 LE, id pstr, version pstr, name pstr
 *               [ caps u32 LE ]  - optional 4-byte tail, see below.
 *               parse_hello proc_wire.rs:161-180 (strict: trailing
 *               bytes reject :170-172; silence and lies are both
 *               disagreement). parse_hello_caps :212-233 tolerates the
 *               caps tail ONLY: a 1-3 byte tail or anything past 4
 *               bytes still refuses (:221-225). Encoders: hello_payload
 *               :188-202, hello_payload_caps :239-243. The kernel
 *               cross-checks abi/id/version against the manifest at
 *               spawn (executor_proc.rs:300-314, typed
 *               IdentityMismatch); wrong kind or wrong first frame
 *               kills the hello exchange (executor_proc.rs:378).
 *   MSG_START   EMPTY payload (parent writes it with &[],
 *               executor_proc.rs:452-456; runner arm
 *               modukit-runtime/src/main.rs:348-353).
 *   MSG_STOP    EMPTY payload (executor_proc.rs:576-580; runner arm
 *               main.rs:354-362 - acks, then exits cooperatively).
 *   MSG_ACK     status i32 LE - exactly 4 payload bytes; any other
 *               length refuses (parse_ack proc_wire.rs:248-253).
 *               Encoder ack_payload :260-262 - the D106 batch-0 T0.2
 *               twin the runner and goldens frame through (no more
 *               hand-rolled to_le_bytes).
 *   MSG_REGISTER  service_id pstr, count u8, method pstr x count.
 *               Trailing bytes refuse (parse_register
 *               proc_wire.rs:287-302; register_payload :307-319).
 *   MSG_CALL    seq u32 LE, method pstr, len u32 LE, input bytes.
 *               Input length must equal the remaining bytes exactly
 *               (parse_call proc_wire.rs:331-351; call_payload
 *               :355-361).
 *   MSG_RESULT  seq u32 LE, status i32 LE, len u32 LE, output bytes.
 *               Status 0 carries bytes; a negative status is the D27
 *               author code verbatim - typed passthrough, the
 *               kernel-side proxy surfaces ServiceCallFailed { code }
 *               (doc proc_wire.rs:48-53; parse_result :374-394,
 *               result_payload :398-404). Strict on trailing bytes.
 *   MSG_DIAL    delta u64 LE microseconds - exactly 8 payload bytes
 *               (parse_dial proc_wire.rs:417-423; dial_payload
 *               :410-413 saturates at u64::MAX - a dial that long is
 *               a broken parent).
 *   MSG_DUE     status i32 LE, fired_at_ns u64 LE, count u16 LE,
 *               then (token u64 LE, job pstr) x count. Minimum
 *               payload 14 bytes; bad count, short buffer, non-UTF-8
 *               label or trailing bytes refuse the whole frame
 *               (parse_due proc_wire.rs:454-483; due_payload
 *               :436-449). Status -2 = the clean Real-domain refusal
 *               above; 0 carries the due batch (timers-as-jobs,
 *               the return is data, never callbacks).
 *   MSG_FORWARD  op u8, topic pstr, qos i32 LE, len u32 LE, value
 *               bytes. op MUST be one of FWD_PUB / FWD_SUB /
 *               FWD_UNSUB - an unknown op refuses the frame
 *               (parse_forward proc_wire.rs:506-527, op gate
 *               :508-510; forward_payload :532-539). Convention:
 *               FWD_SUB and FWD_UNSUB leave qos 0 and value empty
 *               (doc :491-493). Topic strings are TopicId-grammar
 *               bound (max 255, inside the pstr cap).
 *   MSG_DELIVER  topic pstr, qos i32 LE, len u32 LE, value bytes.
 *               A negative qos is the refusal channel
 *               (DEFERRAL_DECLINED, -26; doc proc_wire.rs:541-543).
 *               parse_deliver :553-570, deliver_payload :574-581.
 *
 * ACK is answered to START and STOP (one per verb, the exchange pump
 * breaks on it); DUE answers DIAL; RESULT answers CALL; REGISTER is
 * pushed unasked between START and its ACK; FORWARD/DELIVER flow on
 * the bus legs once HELLO declared the caps bit.
 */

/* -------------------------------------------------------------------
 * EXTENSIBILITY posture (plan T1.1) - two verified statements.
 *
 * 1) Tail tolerance is per-parse, not global: HELLO tolerates the
 *    u32 caps tail ONLY (parse_hello_caps proc_wire.rs:221-225 - a
 *    0-length tail keeps the legacy byte-exact handshake, a 4-length
 *    tail declares caps, anything else refuses). EVERY OTHER parse
 *    is strict trailing-bytes-reject: REGISTER, CALL, RESULT, DUE,
 *    FORWARD, DELIVER, ACK (length != 4), DIAL (length != 8).
 *
 * 2) Unknown-kind posture is DIRECTIONAL, not one policy:
 *    - host -> child: the runner's STEADY-STATE loop IGNORES unknown
 *      kinds - "forward-compatible silence: unknown frame types are
 *      ignored" (crates/modukit-runtime/src/main.rs:430-432, the
 *      `_ => {}` arm). New verbs may arrive before a child learns
 *      them; a listening child stays alive.
 *    - child -> host: the START-exchange pump KILLS - any kind
 *      other than ACK / REGISTER / FORWARD inside the start
 *      exchange is `start reply: wrong frame type` -> kill_and_reap
 *      (crates/modukit-core/src/executor_proc.rs:529-533). The stop
 *      and dial exchange pumps carry the same wrong-kind kill arm
 *      (executor_proc.rs:600-603 "stop reply", :735-738 and
 *      :829-832 "dial reply"). A version mismatch in any pump is
 *      likewise kill_and_reap + WireVersionMismatch
 *      (executor_proc.rs:534-538).
 *    Amendments therefore ride NEW kinds or NEW HELLO cap bits -
 *    never a redefinition of an existing kind's payload, and never
 *    a tail field on a strict parse.
 *
 * Amendment policy (D83): new kinds tail-append at the constant
 * ledger tail with the next MSG_ number; existing kind numbers and
 * payload grammars never move; a BREAKING change to a live row
 * requires a hearing. The WIRE_VERSION byte gates every cross-
 * version read either way - vocabulary growth has never bumped it
 * (the caps tail landed with WIRE_VERSION staying 1,
 * proc_wire.rs:91-95 doc).
 *
 * Install-door note (D106 batch 0, amendment A1): a manifest entry
 * with a runner AND a non-".so" artifact skips the install-time
 * dlopen probe (crates/modukit-core/src/bundle.rs:350 - the
 * cross_check branch); its identity defers to the spawn-time HELLO
 * cross-check above. The [[files]] hash envelope and the manifest
 * grammar are untouched by the exemption.
 */
