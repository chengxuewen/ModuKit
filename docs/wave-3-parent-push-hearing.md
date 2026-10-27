# Wave-3: Parent-to-Child Bus Push - Hearing

> Opened 2026-10-27. Adjudications are walked ONE AT A TIME (house protocol);
> this document records the fact base and each ruling as it lands. Volume name
> wave-3 (plugin door was wave-2/D106; the gap was first NAMED in
> `docs/stage-5-plan.md` section 3 as the "second ordinary-publish consumer").

## 1. Fact base (verified against the tree 2026-10-27)

The wall fan-out today has exactly three drain sites, all in
`crates/modukit-core/src/bus_wire.rs`:

1. **Child-origin publish** - `handle_forward(FWD_PUB)` -> `on_publish`
   (:143-168): publishes onto the parent bus, then drains EVERY subscribed
   child's proxy `Receiver` and writes one DELIVER per value (self-echo
   included). This is "THE single fan-out doorway" per its own comment.
2. **Late-join snapshot** - `on_subscribe` (:170-202): `Bus::subscribe`
   late-join seeds retained values; they are drained into DELIVER frames at
   subscribe time.
3. **Timer bell** - `pump_local_bell` (:217-241): the same drain grammar,
   restricted to children that declared `HELLO_CAP_TIMER_V2`, called from the
   bell site AFTER the bell site publishes locally (D99 R5-A T2).

**The gap**: a publish that does NOT enter through `handle_forward` fills the
proxy `Receiver` queues and nobody drains them. That covers:

- the host calling `kernel.bus().publish(...)` directly (`kernel.rs:157`,
  public API, `Bus::publish` at `bus.rs:194`);
- in-process plugins publishing through the embed face
  (`trampoline_bus_publish`, `host_v2.rs:104`) - so an IN-PROCESS plugin's
  publish never reaches exe children either;
- the shadow runner's `bus_on` path (`modukit-runtime/src/main.rs:254`).

Queues are per-child proxy `Receiver`s on the parent bus
(`ParentFanout.subs`, `bus_wire.rs:77`), created at the child's FWD_SUB and
retired with the start-window mailbox (`retire`, :112-120; 01 section 3 rule 2).

Blocking hazard: `deliver` (:247-260) writes the child's control socket inline
inside a mutex. A child that stops reading fills the AF_UNIX buffer and the
writer blocks - the bus's own commandment 3 says a publisher never blocks on a
subscriber (`bus.rs:34-35`). The bell path accepted this at bell volume; a
general pump raises the stakes.

QoS vocabulary: `qos_from_i32`/`qos_to_i32` (`bus_wire.rs:52`,
`embed_codec.rs:106`); deferred kinds (`KeepAll`/`Bounded`) build no queue and
refuse subscribe, so no proxy `Receiver` exists for them.

Attachment seam: `Kernel::attach_parent_bells` (`kernel.rs:165`) already hands
the supervisor-constructed `Arc<ParentFanout>` to the kernel (one boot-time
setter, leaf lock, standalone-kernel posture unchanged).

One-thread law: `Bridge` (`bus_wire.rs:319-327`) documents it explicitly - a
threaded runner would need a per-child write queue, and the one-thread law is
what keeps this "a mutex of one socket".

## 2. Adjudications (walked one by one; rulings recorded below each)

| # | Question | Ruling |
|---|---|---|
| A1 | Pump hook point (bus-level vs kernel-door vs threaded) | **A - bus-level publish hook** (2026-10-27) |
| A2 | Timer bell path and its capability gate after the general pump | **A - retire the bell-only pump and delivery gate** (2026-10-27) |
| A3 | Slow or dead child discipline (blocking vs non-blocking drop) | **A - non-blocking writes, drop and count** (2026-10-27) |
| A4 | DELIVER QoS code carried down | **A - the publish's own code, mirror of on_publish** (2026-10-27) |
| A5 | Verification shape (legs, planted reds, acceptance rows) | **A - HOST PUSH stanza on both plugin legs + single-point lever** (2026-10-27) |
| A6 | A3's mechanism vs the SHARED child fd (amendment) | **C - bounded write, remainder-retaining outbox, no O_NONBLOCK** (2026-10-27) |

## 3. Rulings (filled as the walkthrough proceeds)

### A1 (2026-10-27) - user-adjudicated: A

Pump hook lives INSIDE `Bus::publish`, after the fan-out and after the topics
lock drops. Consequence set recorded at ruling time:

- `on_publish`'s explicit drain loop is deleted (the hook is the one doorway;
  a child-origin publish now drains through the same path, self-echo included).
- The supervisor installs the hook at the same seam that attaches the parent
  bell fan-out (`Kernel::attach_parent_bells`, `kernel.rs:165`); a kernel with
  no supervised children keeps `None` (standalone posture unchanged).
- The hook is a kernel/supervisor concern living in the bus layer - accepted
  knowingly; the alternative (kernel-door wrappers) leaves `kernel.bus().
  publish` as a permanent silent bypass under the D87 freeze.
- Blocking discipline for the inline socket writes is DEFERRED to A2/A3 (the
  hook makes every publish pay it, in-process plugin publishes included).
- `pump_local_bell` / the TIMER_V2 delivery gate become A2's question: with an
  unconditional pump, the bell site's separate drain would double-deliver.

### A2 (2026-10-27) - user-adjudicated: A

The bell-only pump and its TIMER_V2 delivery gate RETIRE with this volume:

- `pump_local_bell`, `ParentFanout.timer`, `note_timer_child` and the retire
  sweep of the timer set are deleted; the bell site keeps only its
  `Bus::publish` (the general hook drains).
- Bell topics follow ordinary subscription law: whoever subscribed receives.
  The old pin ("a BUS-only child stays silent on a bell topic") is REPLACED by
  a subscriber-law assertion, and D99 R5-A's delivery clause is recorded as
  SUPERSEDED here (the capability bit keeps its other job: DIAL eligibility).
- Wire contract untouched: `HELLO_CAP_TIMER_V2` stays in wire.h and the
  goldens; only the kernel-side targeting gate dies.

### A3 (2026-10-27) - user-adjudicated: A

Child control sockets go non-blocking at registration; `deliver` treats
would-block as a COUNTED DROP for that child (registration kept - a stalled
child may resume reading) and keeps removing registrations on hard errors.
One new counter rides the existing gate-counter family (D21 doorway). The
publisher never stalls, matching commandment 3 (`bus.rs:34-35`); the bell path
inherits the discipline (it now rides the same general pump per A2).

### A4 (2026-10-27) - user-adjudicated: A (no-contest)

The hook carries `qos_to_i32` of the QoS this very publish declared - byte-for-
byte the same rule `on_publish` already runs. Deferred topics need no special
case: subscribe already refuses them with the -26 refusal channel, so no proxy
receiver exists and the pump is a no-op.

### A5 (2026-10-27) - user-adjudicated: A

Legs twelve (python) and thirteen (node) each gain a HOST PUSH stanza: the
host-side driver publishes on the seed topic through the embed floor (the
in-process trampoline - today's exact gap) and the child must receive it back
via its `seen` method. One named single-point lever,
`MODUKIT_HOST_PUSH_OFF=1`, is read at the hook-install site (same discipline
as the skins' SABOTAGE_ENV): set, the stanza reds while the child-origin
stanzas stay green (precise attribution). Three unit tests ride beside: the
hook fires outside the topics lock (no deadlock), a filled socket yields a
prompt return plus one counted shed, and a deferred topic pumps nothing.

### A6 (2026-10-27) - user-adjudicated: C (amends A3's mechanism, keeps its intent)

The review pass found A3's mechanism collides with a fact the hearing did not
see: the child's control socket is SHARED - `executor_proc` reads and writes
it (STOP/ACK/CALL/DIAL at :449/:574/:723/:795, reads at :359/:468/:585) and
`write_frame` is two `write_all` calls (`proc_wire.rs:123-134`).
`O_NONBLOCK` is per file description, so setting it would break every blocking
read/write on that channel.

Ruling C: the fd stays blocking. `deliver` writes through a per-child outbox
that retains a partial-write remainder and finishes it on the next flush; a
write timeout (`SO_SNDTIMEO`, set once at registration, writes only - reads
untouched) bounds the stall; a timeout keeps the remainder and counts a
STALL; a hard error deregisters and counts; an over-cap outbox drops WHOLE
frames and counts. No `set_nonblocking` anywhere. The bus commandment reads
"a publish stalls at most WALL_WRITE_TIMEOUT per stuck child" on the wall
side - named in the plan, not swallowed.

## 4. Hearing closed 2026-10-27

Six adjudications (five A + one C amendment), walked one by one on the user's
rulings:
bus-level hook; bell gate retired; non-blocking shed-and-count; the publish's
own qos code; HOST PUSH on both twin legs with a single-point lever.
Minted as D108 (decisions.md); plan drafting is the next step.
