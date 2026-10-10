# 00 — Overview: External Reference Matrix & Stage-2 Fork Decision Sheet

> Snapshot 2026-10-09. Seventeen profiles in two rounds: 8 dated observations
> of local `.refinfo/` clones + 9 remote GitHub profiles (blobless clones measured
> under ~/.cache/modukit-research/) — external reference, never ModuKit content.
> Plus one citation audit (whitepaper-cited-unvendored.md).
> Verdicts: DEPENDENCY-CANDIDATE / BORROW-PATTERNS / REFERENCE-ONLY.
> Licenses verified from clone roots; ModuKit's own license: **MIT OR Apache-2.0 dual**
> (decisions.md D1, 2026-10-09 — profiles predate it; see their inline notes).

## 1. Verdict matrix

| Project | Verdict | License (gate) | Primary relevance | Sharpest single finding |
|---|---|---|---|---|
| CppMicroServices | BORROW-PATTERNS — primary Stage-1 design mirror | Apache-2.0 (green) | Stage 1 | Port lifecycle + registry SEMANTICS (6 states; reference≠instance leases); never let C++ objects cross the .so seam — only `extern "C"` activation symbols |
| zenoh | BORROW-PATTERNS; conditional dependency only for cross-machine | EPL-2.0 OR Apache-2.0 dual (green via Apache leg) | Stages 1, 2, 4, 5 | `zenoh-plugin-trait` = most complete dynamic-plugin ABI found (vtable-in-C-struct, static descriptor data, `#[no_mangle extern "C"]` entry, dylib+static dual build from one crate); its `declare!` macro is a codegen blueprint for generated binding faces |
| iceoryx2 | DEPENDENCY-CANDIDATE (primary, local data plane) + BORROW-PATTERNS | MIT OR Apache-2.0 dual, Eclipse governance (green) | Stage 2 | Zero-copy pub/sub is its whole product; already ships zenoh-as-carrier — the fork in our §5 is narrower than either/or |
| dora | BORROW-PATTERNS — strongest live Stage-2 mirror | Apache-2.0 (green) | Stage 2 | It chose zenoh over iceoryx EARLY and removed iceoryx deliberately — then ate >4KiB zero-copy loss under backpressure (#3428) and a 28-day silent-hang class (timeout/backpressure gaps); real-world cost data for our transport decision |
| VisiaEngine | BORROW-PATTERNS — method source | MIT/Apache-2.0 (green) | All stages (process) | The gate/trace/evidence discipline + 30-candidate adoption adjudication (tribunal) distilled in visiaengine.md §8 |
| MediaServo | BORROW-PATTERNS + DOC-DISCIPLINE SOURCE | Apache-2.0 (green) | Stages 4, 5 | Capability-declared-not-negotiated plugin contract (OBS-modeled, rejected GStreamer-style negotiation); one-C-ABI-four-surfaces binding pipeline actually executed; counter-signal: whitepaper names webrtc-rs, production went libwebrtc (webrtc-sys) and demoted webrtc-rs to skeleton |
| AccessBase | BORROW-PATTERNS (gate/test culture; host-app shape) | (see profile) | post-Stage-1 | e2e gate culture with PIT-numbered commits as living ledger |
| CTK | REFERENCE-ONLY (sliver: persistence + deprecation APIs) | Apache-2.0 BUT Qt LGPL runtime trap | Stage 1 (negative) | Two port-worthy mechanics: SQL-persisted install set across restarts; `CTK_DEPRECATED_SINCE` + compile-time cutoff switches for the future ABI deprecation policy |
| AUDESYS | INTERNAL PRIOR ART (sister project, same maintainer — profile header carries the provenance flag) | Apache-2.0 (green; borrowing is by ownership, not license risk) | Adjudications 11/12/13; D8/D10/D11 cross-check | The maintainer's own Rust sister project already built the middleware seam (their D11: inproc + zenoh-scaffold trait trio), staged hot-swap at cycle boundaries, closed 14-type protocol — and visibly carries the hand-written-serialization debt (1,861-line ipc.rs) that the pending generated-bindings ruling retires |

### Remote round (GitHub projects, measured 2026-10-09; arrow-flight probe added 2026-10-10)

| Project | Verdict | License (gate) | Primary relevance | Sharpest single finding |
|---|---|---|---|---|
| rutis | BORROW-PATTERNS (high priority) | MIT | Stage 1 | only LIVE whitepaper-named runtime; ★91 pushed same-day; its TS/Python faces are the FFI-seam alternative to our C-ABI-only exit — compare before kernel spec |
| Zellij | BORROW-PATTERNS + CAUTIONARY-TALE | MIT | Stages 1/2/5 | production triple-mode plugin host whose external-plugin ABI broke repeatedly on version bumps — the empirical case for C-ABI + capability-version policy |
| CLAP | BORROW-PATTERNS (strong) | check profile (0BSD/MIT-family) | Stages 1/5 ABI canon | pure-C plugin ABI textbook: entry factory symbol + versioned extension query; honest correction — it carries NO size field, so we adopt size+version DOUBLE guard (Visia struct_size ∪ CLAP version gate) |
| arrow-flight (arrow-rs) | CONFIRMS-D12 (framing technique); REFUSE-SUBSTRATE (no gRPC stack, no dependency) | Apache-2.0 (green) | D12 + slot-③ remote design checklist | `FlightData.data_body = tag 1000`: field numbering used as streaming architecture (header-first, tail-untouched); Ticket/PollFlightInfo/DoExchange validate handle/job-token/stream shapes already adjudicated; implementation honesty: one-copy optimized in measured checkout, pointer zero-copy is the C Data Interface lane |
| uniffi-rs | BORROW-PATTERNS | **MPL-2.0** (only non-permissive here — borrow patterns, never vendor into permissive core) | Stage 5 | metadata/IDL is the contract, C ABI its checksum-sealed projection — challenges our header-first phrasing (see §4 item 5) |
| Wasmtime | DEPENDENCY-CANDIDATE | MIT/Zlib-Apache family (verify per profile) | sandbox/Stage 5 | already our whitepaper-named choice; wasip2-plugins is the current official plugin story; killed its own C-API plugin system once (governance lesson) |
| Extism | BORROW-PATTERNS | MIT | sandbox | flat C-ABI-over-wasm host boundary = SAME doctrine as our single-C-exit in a different arena; host-guest ABI stability experiment worth tracking |
| ipc-channel | BORROW-SHAPE (not dependency) | MIT/Apache dual | Stage 2 | typed control-plane shape; alive-but-low-health (22 commits/12mo); its SHM is copy-in single-consumer — NOT a ring; adds the third option to §2 |
| smithay | REFERENCE-ONLY + BORROW-PATTERNS | MIT | Stage 3 | capability-registration globals, backend/trait split, dispatch2 shim-over-moving-ABI; refuse crate dependency (wrong granularity, Linux-only, ~9.5y still 0.x) |
| sysplugin | NOT-FOUND-AS-DESCRIBED | — | audit | expected 'Rust OSGi framework' does not resolve — third instance of the unverified-citation failure mode (see whitepaper audit) |

## 2. Stage-2 fork decision sheet — "iceoryx2 / Zenoh" (whitepaper §5)

Form is adjudicated (D11: narrow transport seam + per-plane slots + bridge plugins — see
`docs/modules/07-transport-architecture.md`); backend selection now adjudicated (D25): iceoryx2 first, exact-pinned tag; self-built ring = trigger-gated fallback; house rule: slots house porters only — worldview vendors (FastDDS/Cyclone/Connext) are bridge-only, never backends.
Evidence from three profiles converges on a
composition, not a choice:

1. **iceoryx2 already embeds zenoh**: its link/tunnel layer defines a pluggable
   `carrier`, and the shipped carrier IS zenoh (integrations/zenoh). Local
   data plane = iceoryx2 shared-memory; cross-host = zenoh tunnel over the same
   abstraction.
2. **zenoh self-limits on the local data path**: zero-copy exists only in the
   shared-memory transport, >4KiB payloads fall out of it under flow control
   (dora #3428), and `zenoh-transport-ramsnmp` (its own SHM attempt) is
   unmaintained/unresolved deps.
3. **dora's precedent is the counterexample**: early switch to zenoh-for-all
   bought topology simplicity, cost large-payload zero-copy, required
   daemon-routed unidirectional backpressure design, and it removed iceoryx
   support entirely (PRs #201/#205) — a choice ModuKit is watching, not copying.

4. **A third option emerged** (ipc-channel.md): plugin SUPERVISION is
   typed-RPC-shaped (spawn/control/stop), not topic-fabric-shaped. The
   composite: typed control plane (ipc-channel SHAPE, borrowed not vendored)
   + iceoryx2 zero-copy data plane + zenoh cross-host carrier. The §5 fork
   names two candidates for one role; the sharper question is role split.
   Stage-4 analogue (webrtc audit): decision axis is sans-I/O engines
   (webrtc-rs `webrtc`/`rtc` + str0m coexisting) vs libwebrtc-FFI
   (webrtc-sys/LiveKit, production counter-signal) vs vendor SDKs — and
   `getstream-rtc` in our canon is a naming+category error (it is `getstream`,
   a preview vendor SDK; REJECT as transport dependency).

**Proposed adjudication (D-entry when Stage 2 gates, per G5 mechanics — NOT
decided today)**: local zero-copy plane = iceoryx2 as dependency candidate;
cross-machine = zenoh (iceoryx2's own carrier choice, keeping one mental model
of publish/subscribe + plugin ABI); the `modukit-transport` abstraction (already
in the crate map) is the seam that makes this swappable, which is why the fork
does not need to be resolved before Stage 1 lands.

## 3. Cross-cutting lessons (all sources)

- **The C-ABI seam keeps converging**: cppms activation symbols, zenoh-plugin's
  vtable-struct + `extern "C" fn _entry`, Visia's 49-function capi, MediaServo's
  per-SDK prefixes + abi-drift gate. ModuKit Stage 1/5 design has four
  worked examples; the shared shape = static metadata declared at rest, thin
  extern-C entry, capability tables embedded, dynamic dispatch behind the seam.
- **Never trust "compiles" as "links"**: MediaServo PIT-71 — `cargo check`
  green, link failed (duplicate symbol across webrtc-sys+mediasoup-sys), and
  a `cfg(target_os)` guard made the passing check an illusion for the real
  platform. Gate at the artifact level.
- **Build-artifact hygiene is a policy, not a .gitignore** (PIT-210): 180 MB
  `.node` committed into tree; fix = staging dir + explicit prohibition.
- **Size is a spec**: MediaServo fat wheel 528 MB → 79.9 MB via strip +
  deliberately dependency-light FFmpeg; Visia's 7.25 MB ≤ 10 MB with dated
  evidence. ModuKit sets its OWN numbers when artifacts exist (tribunal P1).
- **Numbers in prose rot; numbers derived by tools don't** (Visia drift
  incidents ×5; tribunal V2/P4). This series pins every count to a command in
  the profile itself.
- **Registry semantics before code**: reference≠instance, ranking, LDAP filter,
  use-counting, RAII listeners (cppms); start-stop levels as integer ordering
  (CTK); embedded generated manifests vs sidecars (both). Stage-1 kernel spec
  vocabulary is now assembled.

- **ABI stability has a production body count**: Zellij broke its external
  plugins repeatedly on toolchain/version bumps (citable in zellij.md) — the
  empirical why behind C-ABI-only + capability-version gates + append-only
  structs; CLAP's counter-lesson: a version gate WITHOUT a size field still
  works if append-only is frozen by convention — we keep BOTH guards.
- **The metadata-vs-header question** (uniffi): if bindings are generated,
  where does truth live? uniffi answers 'typed metadata, C ABI projected';
  our canon currently reads 'C ABI is the single exit' — §4 item 5 routes
  this to a real design decision before Stage 5, not drift.
- **Citation hygiene paid off immediately**: within this series' own planning
  vocabulary, 3 names failed verification (rustbridge, sysplugin,
  getstream-rtc) and 1 survived glorified (webrtc-rs rumor busted). Every
  future whitepaper edit citing a project must carry the audit line
  (name → repo → stars/pushed/license).

## 4. Open items for the maintainer (not decidable by this series)

1. §5 wording: keep "iceoryx2/Zenoh" until the Stage-2 gate, or adopt the
   composition above into a D-entry now?
2. §8 "I420 frames": confirmed as this project's own design term (UI-composition
   payload) per root README — the tribunal's "smuggled residue" suspicion is
   WITHDRAWN (recorded honestly in r2 pool).
3. ~~12 idle language rule packs in instructions[] — unmount until those
   languages have code (G10)?~~ **EXECUTED 2026-10-09**: instructions 26→16
   (common+rust only; all rule dirs kept on disk per maintainer ruling —
   status.md History).
4. A6 binding wording ("hand-written binding layers prohibited; generated-only
   from the single C ABI") — ratify as ModuKit D-record?
5. **Header-first vs metadata-first contract** (uniffi finding): ModuKit's
   C-ABI canon could mean hand-authored header is truth (Visia shape) OR
   typed IDL is truth with the C ABI as generated projection (uniffi shape).
   UnFreeze before Stage 5 binding work; decision record either way.
6. Whitepaper Stage-4 wording carries a naming+category error
   (`getstream-rtc` → `getstream` vendor SDK); Stage-1 wording carries a
   misfire (`rustbridge`) and unresolvable names (`event-engine`,
   `Parallax`, `Scarlet`). Corrections proposed in
   whitepaper-cited-unvendored.md — maintainer call, whitepaper stays
   verbatim until then.
7. Stage-4 engine axis: keep BOTH sans-I/O candidates (webrtc `rtc` primary,
   str0m LAN/embedded tier) as the profile recommends, or re-evaluate when
   the SFU scope question (webrtc-rust-state OQ list) is answered.
8. Adopt the tribunal PORT_NOW discipline bundle when Stage-1 tooling
   lands: `scripts/gate.sh` seed + doc-liveness check + D-record mechanics
   (source: VisiaEngine adoption tribunal distillation, status.md History
   2026-10-09; elevated into this register by the 2026-10-10 doc-audit, M9).

## 5. Reading order by stage

- Stage 1: cppmicroservices.md → ctk.md (negative/niche) → zenoh.md §2 → visiaengine.md §8 (process)
- Stage 2: iceoryx2.md → dora.md → §2 of THIS file
- Stage 4: mediaservo.md (WebRTC counter-signal section)
- Stage 5: visiaengine.md §2-3 (ABI discipline) → zenoh.md bindings → cppms activation seam
- Stage 1 (+remote): rutis.md → zellij.md (cautionary) → clap-plugin-abi.md (ABI canon) → sysplugin.md/whitepaper audits (citation hygiene)
- Stage 2 (+remote): ipc-channel.md (control-plane option) → re-read §2 of THIS file
- Stage 3: smithay.md
- Stage 4: webrtc-rust-state.md + mediaservo.md counter-signal
- Stage 5: uniffi-rs.md + wasm-plugin-hosts.md (Extism/wasmtime) → visiaengine.md §2-3
