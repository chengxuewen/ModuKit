# Wave-2 CLOCK hearing agenda (schedule-timer - the D82 trio's third seat)

> STATUS: HEARING HELD 2026-10-21 - the six rulings were walked one by one and
> drawn B,A,A,A,A,A (results: §8; ledger row: D99). R1 landed as B, deviating
> from the doc's A-recommendation for a reason surfaced during the walk.
> Lanes per R6: T1 (door) LANDED 2026-10-21 - see §9 pointer; T2 (crossing)
> awaits its own go order.
> D96 ruling 1 named this agenda ("the third keeps its §6 slot with a named agenda");
> I3-A drew the line ("the two bus verbs are one mechanism, the timer is another").

## 1. Background - what the house already owns on the clock side

- **One clock PER KERNEL** (`crates/modukit-core/src/clock.rs`, D20 timers-as-jobs;
  `kernel.rs:53` is per-Kernel - D74 landed SEPARATE parent-dialed clocks for
  child runners, there is no single shared clock across the wall):
  `now()`, `advance(delta) -> Vec<DueJob>` (Step/Sim domains only - the Real
  domain refuses the dial), `timer(deadline, job) -> JobToken` (the ONLY
  registration entry, one-shot, message-shaped per D6), `due_jobs()`,
  `pending_timers()`. There is NO cancel and NO recurring in the host grammar,
  no per-plugin ownership anywhere: the `Timers` register is global to the clock
  and unowned (`clock.rs:103-106`), tokens count from 1 in EVERY kernel
  (`clock.rs:106`) - plugin tokens across the wall are ambiguous by themselves.
- A due timer POSTS `TimerDone { token, job }` as an event on the ring
  (`events.rs:47-49`) - read today by introspect (the debug-reader face shows
  the tail) and by the supervisor pass (`embed.h:319-321` "post due timers").
- Clock is a platform SERVICE: `org.modukit.clock` registered at boot
  (`wiring.rs:86`, D40 service-table-is-the-only-extension-point).
- **Embed face already speaks clock**: `modukit_clock_now` +
  `modukit_clock_advance` (`modukit_embed.h:211-217`, HostEmbedV1 slots 272-274)
  and the M2/M3 sup rung carries a dial-the-child's-step-clock verb
  (`embed.h:325-326`). The MISSING half is the plugin-facing door (D82's seat 3).
- **v2 door shape landed** (D96 N1): `HostAbiV2` = v1 prefix byte-identical +
  4 mailbox slots (`abi.rs:210-230`), size-guard read, plugin.h mirror, trio
  parity leg, HOST2 ledger rows. N2: two talkers FORWARD/DELIVER, HELLO
  capability u32 tail, WIRE_VERSION stays 1.
- **v1-frozen determinism proof exists**: `step_replay` - "one fixed pair of
  timer jobs both placements schedule identically, then step - the due-job
  transcripts must come back byte-equal" (`testhost.rs:57-59`, verbatim),
  in-process (`TestHost::advance`) AND over the control socket.
- **The authority law grammar** (`host_v2.rs:13-14`): the per-call queue table
  and its ids live "only for the duration of the plugin's start call" (verbatim
  shape); the embed comment reclaims dropped queue ids when the publisher side
  drops (`embed.h:190-192`). This is the GRAMMAR a per-plugin timer register
  would follow - note the register itself is NEW state (see the clock bullet).

## 2. What this hearing draws (six signatures)

R1 round scope / R2 door version / R3 verb grammar / R4 due-delivery +
cleanup / R5 wall semantics / R6 landing split.

## 3. What this hearing deliberately does NOT draw

- `now()` plugin-facing exposure (no named plugin consumer today; the host
  embeds and reads; named here, not a silent drop).
- cancel/recurring timer semantics (the host face has neither - a plugin door
  may not invent what the mirror lacks; recurring = fresh host-grammar hearing).
- wall clock-domain choices for child runners (dial ownership unchanged: the
  parent/supervisor turns, per the M2/M3 landed shape).
- demo-leg form / fixture naming / parity ledger row counts (landing detail,
  not contract - the N1/N2 lane grammar already fixed the shape).
- Python/JS skins of the timer verb (wave-2 §6 armed rows unchanged).
- A NON-retained bus QoS word: every year-1 QoS replays at late-join
  (`LatestValue` the current, `Lossy` the ring - bus.rs:205-213, 284-288);
  timer semantics ACCEPT and NAME the replay instead of growing the QoS
  grammar in this hearing.

## 4. Proposed rulings

### R1 - Round scope

A. Draw the D82 seat 3 exactly: ONE plugin-facing `timer_schedule`; due delivery
   rides whatever R4 picks. Nothing else.
B. Timer + plugin-facing `now()` (two verbs) - plugins compute relative delays
   themselves.
C. Full timer surface: schedule + cancel + now (three verbs) - needs a new
   cancel mechanism in the host grammar first (clock.rs has none).

Rec: **A**. Smallest door, and R4-A (due-on-bus) makes now() unnecessary for
the scheduled case; B/C expand the mirror without a named consumer (C also
breaks §3's no-invention line).

### R2 - Door version

A. **`HostAbiV3`**: a new table = the byte-identical V2 table prefix + the ONE
   tail-added timer slot (R3); plugin declares major 3 at the handshake. Executes D83 verbatim
   ("appended verbs enter the next version candidate") and recurses the N1 shape
   (V2 was born as v1 prefix + tail; the size-guard/parity/ledger machinery
   exists and is additive).
B. Tail-append the timer slots INTO `HostAbiV2` (no new table). Old v2 plugins
   survive it mechanically (they read a byte-identical prefix behind a size
   check), but it breaks the N1 landed meaning of `expected_size()`: a table
   published on the field stops being immutable, and "what does v2 provide"
   starts depending on the calendar.
C. No table change - reach the timer through service magic (acquire
   `org.modukit.clock` across FFI). The plugin-side v1 door has no consumer
   acquire shape (`abi.rs` shows only `register_service`); C would open a
   bigger, undocumented second channel - against D40's one-table law in spirit.

Rec: **A**. The whole house (D83/D96 ruling 2) has version-per-shape discipline;
B saves one struct at the price of the sentence that made v1 trustworthy.

### R3 - Verb grammar (mirror law, D96 ruling 5)

A. One slot mirroring `Clock::timer` exactly:
   `timer_schedule(u64 deadline_ns, const char *job_label) -> status` with the
   `JobToken` as `uint64_t *out_token`. Absolute deadline like the host face;
   the job label is the D6 message-shaped string; token mirrors `JobToken(u64)`.
   Out-param + status-return mirrors the `bus_subscribe` landed shape.
B. Relative-time verb `timer_after(delta_ns, ...)` - dies under R1-A: a plugin
   with no `now()` cannot anchor an absolute deadline... and a relative fire on
   a Step clock is exactly the non-determinism the step face exists to forbid.
C. Recurring / interval variant - not in the host grammar (§3), refuses itself.

Rec: **A** (no-contest once R1=A: B/C have no ground truth to mirror).

### R4 - Due delivery + cleanup

A. **Due rides the bus, drawn to the grammar.** ONE reserved topic PER
   PLUGIN: `org.modukit.timer.<plugin-id>` - bounded (one per plugin, not
   per token), which removes cross-plugin noise AND the token-collision
   case in one stroke: a plugin instance lives in exactly ONE kernel, so on
   its own topic its token space is unambiguous FOR A KERNEL LIFETIME - an
   exe revive recounts from 1 (`clock.rs:106`), so a pre-crash firing still
   on the parent's Lossy ring can share a token with a post-revive one: the
   landing pins the dedup rule (candidate: token + a schedule-time generation
   field) - named here, not hidden at the walk. Payload = `token` u64 LE +
   `job label` UTF-8 bytes (an `origin` field was drafted for a shared
   topic; the per-plugin topic retires it - stated here, not silent).
   Publish-time QoS = `Lossy(16)` with the replay truth NAMED: Lossy keeps
   a bounded retention ring and REPLAYS it at late-join (`bus.rs:205-213,
   284-288`) - EVERY year-1 bus word replays at subscribe (`LatestValue`
   the current, `Lossy` the ring; KeepAll/Bounded deferred, bus.rs:38-40).
   There is no non-retained word to pick and growing the QoS grammar is
   §3's business-as-not: the hearing ACCEPTS late-join replay for timers
   as pinned behavior - a (re)subscribing plugin may see up to 16 of its
   OWN recent firings and rides them idempotently by token. Plugins read
   the topic with the EXISTING mailbox `subscribe`/`recv` verbs - zero new
   recv verbs on the delivery side. Cleanup named honestly as NEW state:
   the per-plugin timer register does not exist today (global unowned
   `Timers` map, `clock.rs:103-106`, no cancel/release anywhere) - the
   landing builds it in the door ctx table, released at stop by the v1
   authority-law GRAMMAR (host_v2's queue table is the shape precedent,
   not the mechanism). NO cancel verb either way. (Host-owned timers keep
   posting TimerDone on the ring exactly as today; the bus posting is
   additive and scoped to plugin-registered timers.)
B. A dedicated recv pair (`timer_recv`/poll slot mirroring `bus_recv`) -
   duplicates mailbox machinery for a second queue type; more verbs, second
   authority table, more to audit.
C. Doorbell callback (host calls into the plugin) - REFUSED on the books:
   D96 ruling 5 "zero reverse-direction function pointers", D63 keeps
   doorbell ergonomics as plugin-side sugar only.

Rec: **A**. One verb net added; delivery, ordering, wall-crossing and cleanup
all inherit landed, gated machinery instead of new promises.

### R5 - Wall semantics (where a child plugin's timer actually lives)

Ground truth, all verified against the tree: `kernel.rs:53` is one clock PER
KERNEL - D74 landed separate parent-dialed child clocks (`sup_dial`,
embed.h:325-326); what is wall-transparent (ruling 6) is the TOPIC space.
AND the crossing machinery has a LANDED ASYMMETRY this hearing discovers and
names: `ParentFanout.deliver` is called ONLY from `on_publish` (an INBOUND
child FWD_PUB fanning to parent-local + other children, bus_wire.rs:142) and
`on_subscribe` (the retained snapshot, bus_wire.rs:167). A PARENT-originated
publish never live-pushes to children today - ordinary topics shipped the
child->shared->wall leg; the reverse leg does not exist. Plugin timers are
the first consumer that needs it (a dl plugin's timer fires on the PARENT
kernel).

A. **Local schedule, due routed through the existing FRAMES both ways.** The
   v3 timer slot fires against whichever kernel hosts the plugin's door (dl
   -> parent clock; exe -> its own runner's D74 clock). At the due-posting
   site the posting explicitly rides the LANDED frame grammar, both
   directions: the child-side post calls the bridge publish (existing
   FWD_PUB frame - the same call the door verb makes); the parent-side post
   calls the ParentFanout deliver loop (existing DELIVER frame grammar, a
   NEW call site - the pump whose absence was just named). Honest cost
   ledger: zero new wire grammar, zero new talkers, zero new frame types;
   TWO new call sites at one posting family. Ruling 6 then holds for timers
   in fact, not by assumption: the assertion suite runs wall-in AND wall-out
   in BOTH directions, and the absent parent->child push stops being a
   hidden column because this hearing put it on paper. The general
   parent->child live-push for ORDINARY publishes is a named follow-up row
   (stage-5 plan §3 list) - the timer due site is NOT where a global bus
   pump gets invented.
B. **Proxied schedule** (child's `timer_schedule` round-trips to the
   PARENT's clock through a new FWD op with response semantics) - unifies
   token space and epoch across the wall, but the mailbox wire grammar has
   NO request-response today: this grows new interaction mechanics, not
   just a bit - against the D96 ruling 3 "two talkers, no second pipe" line.
C. **In-process only** (a v3-declaring plugin against a runner is FAILED at
   the door; timers become a dl privilege) - the single-column semantic
   discount ruling 6 bans ("a semantic discount cannot hide in only one
   column", stage-5-plan.md:32, verbatim).

Rec: **A**. R4 draws the topic per-plugin, so due IS a publish; A pays two
call sites where B pays a whole new interaction shape and C pays in law.

### R6 - Landing split

A. **Two lanes mirroring N1/N2 exactly**: T1 = the door (HostAbiV3 + one slot +
   kernel bus-posting + authority-law release at stop + plugin.h mirror + trio
   parity + HOST3 ledger + codegen emitter + in-process dual assertion suite);
   T2 = the crossing (capability bit + HELLO tail + wall-in/wall-out run of the
   same suite + demo leg + sabotage hook). Each lane has its own go order;
   T1 alone is already useful (in-process plugins can schedule).
B. One lane, everything same day - the bus precedent split for a reason
   (wire-side risk in its own eyes); same family here.

Rec: **A**.

## 5. Verification grammar (what "drawn" will mean at landing)

- Door suite: layout test (expected_size), the version matrix in three honest
  cells - v1-plugin on a v3 host unchanged; v2-plugin on a v3 host reads its
  byte-identical prefix and NO timer slot (the compat negative); v3-plugin
  against a v2 host FAILED at the door (the liar family extends); size-guard
  reads; parity trio (plugin.h / abi.rs / capability-ledger rows) moves with
  the table. No new proto message: the timer publish rides the existing
  byte-payload bus verb - R4 DREW the payload grammar (token + label on a
  per-plugin-id topic), so the wire stays untouched and regen-diff has
  nothing new to eat.
- Capability: HELLO_CAP_TIMER_V2 = 2 (the next bit after the landed BUS_V2 =
  1, proc_wire.rs:96); WIRE_VERSION stays 1 - vocabulary growth never bumps
  a transport promise (ruling 4 verbatim).
- Behavior: wall-in AND wall-out assertion suite (dual_placement + step_replay
  precedent) in BOTH crossing directions: child-plugin firings reach
  parent-local and sibling subscribers (bridge/FWD_PUB leg) AND
  parent-plugin firings reach child subscribers (the deliver-loop call site
  this hearing names as missing) - each leg's new call site has its own
  planted red; determinism under Step domain only where the suite steps;
  plugin-stop census over the NEW per-plugin timer register (pending timers
  released, token never re-fires - the authority-law grammar proven in its
  own eyes, first landing = first census); late-join is a PINNED behavior,
  not a negative: a (re)subscribing plugin sees up to 16 of its OWN recent
  firings off the Lossy ring and rides them idempotently.
- Gates: each lane carries a documented sabotage lever, planted RED observed
  once (C0); demo leg count grows by one at T2 (ledger sweep included, the
  way D98 did it).

## 6. Ledger touchpoints (post-ruling)

- D99 mint (hearing record, six rulings, walked one by one); numbering gate
  widened the same day, probe red observed (the D93-D98 ritual).
- `docs/stage-5-plan.md` gains the CLOCK half rows (the bus file §1 precedent)
  + its §3 "deliberately does NOT decide" entries move here only if ruled.
- capability-ledger + architecture §11 index row; §12 untouched (this volume
  is inside the wave-2 frame, D95 amendment stands).

## 7. Review trail

Momus plan critic, three passes, 2026-10-21: REJECT (3 blockers: wall-crossing
and cleanup claimed free; topic grammar unpicked; QoS/retention undecided) ->
REJECT (2: kernel-internal publishes do NOT cross the wall - only door-verb
publishes bridge, and no parent->child pump exists; Lossy is retained, not
the "non-retained word" claimed) -> OKAY. The passes turned the agenda's
strongest content: the LANDED parent->child live-push asymmetry (R5 ground
truth) and the per-plugin-id topic draw. One non-blocking nuance captured in
R4-A: token uniqueness is per kernel lifetime (revive recounts from 1). Every rejected-option death was verified against real rulings - no strawmen.

## 8. Adjudication results (user-adjudicated one by one, 2026-10-21)

| # | Question | Verdict | Consequence drawn |
|---|---|---|---|
| R1 | Round scope | **B** (doc rec A - flipped on the walk) | TWO verbs this round: schedule + clock-read; the absolute-deadline timer is unspeakable by a plugin that cannot read the clock |
| R2 | Door version | A | HostAbiV3 = V2 prefix byte-identical + TWO tail slots (per R1=B, replacing the "one slot" wording of the draft) |
| R3 | Verb grammar | A | `timer_schedule(deadline_ns u64, label) -> status + token out`; `clock_now(out ns) -> status` - both mirror landed host-side shapes |
| R4 | Due delivery | A | per-plugin topic `org.modukit.timer.<plugin-id>`; payload token+label; Lossy(16) with late-join replay pinned as behavior; NEW per-plugin register, authority-law grammar; no cancel verb |
| R5 | Wall semantics | A | local schedule on the kernel hosting the door; due routed through EXISTING frames both ways (two new call sites, zero new wire grammar); the landed parent->child absence named, general pump deferred to stage-5-plan §3 |
| R6 | Landing split | A | T1 door -> T2 crossing, each its own go order (N1/N2 precedent) |

No ruling was skipped or deferred. The §3 non-decisions stand as written; the
two items that grew legs of their own during the walk are named where they
live: the cross-revive token-generation dedup rule (R4-A) and the general
parent->child pump (now a stage-5-plan §3 row).

## 9. Lane pointers

- **T1 - the clock door**: LANDED 2026-10-21. Evidence row lives in
  docs/stage-5-plan.md (CLOCK lane) per the D99 §6 ruling - rows go to the
  plan-of-record file, this file stays the drawn contract. Plan of record
  for the lane: docs/wave-2-t1-clock-door-plan.md.
- **T2 - the crossing**: armed, not open. Scope by D99: HELLO_CAP_TIMER_V2=2,
  bridge/FWD_PUB due routing child->parent, the ParentFanout deliver call
  site parent->child (the named landed absence), wall-out suite legs, demo
  leg + its sabotage lever, cross-revive generation-field dedup candidate.
