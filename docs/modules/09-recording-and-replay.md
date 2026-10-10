# 09 — Recording and Replay

D16. The bus is the instrument (06 §3): a recorder is an ordinary `export`-gated subscriber,
replay re-enters through a virtual publisher. No side channels, no kernel additions.

## 1. What lands in the file

- **Three-part header**: `fdset` copy + deployment manifest snapshot + host/ABI version
  stamps -> a self-describing archive; a reader a decade later resolves schema digests
  without our repo state (the D12 ledger cashing its check).
- **Records store bytes as delivered**: raw waybill + payload tail, never re-encoded
  (sidecar tag-ordering lets readers skip cargo without parsing — `arrow-flight.md` §3).
- **Gap statistics are a separate record stream** (LOSSY drops, backpressure events):
  honest absence, never silently-missing data. D16 explicitly REJECTED a deep-kernel
  pre-QoS probe (would tax the common path for a minority diagnostic); the revisit
  trigger is gap-stat insufficiency in practice.
- **Logs ride the same ring** (D19): `LogEvent`s interleave with frames and lifecycle
  events by construction — a replay narrates itself.
- **Counter ticks too** (D21): periodic `CountersTick` events land in the same file — plot
  curves exist even for replays of replays.
- Descriptor `local` arms are archived as slot snapshots with one copy — the zero-copy
  chain legitimately ends at the disk; the copy is on the budget table, not hidden.

## 2. What gets recorded (policy)

- Default: **nothing** — recording subscribes per an allowlist that mirrors the D11
  bridge export gates.
- **Dual-encoding doctrine** (maintainer's prior art, `mediaservo.md:87`): the recording
  stream is the producer's quality-tier topic, never a degraded delivery copy — a
  bandwidth-ladder leg must not contaminate the archive.
- i420 full-rate budget: ~90 MB/s at 1080p30 -> default = H264 main topic, or
  keyframes-only + decimated raw; full raw is opt-in (bench rigs, accident review).
- Ring buffer + event-triggered flush (Failed edges, job faults) as the cheap standing mode.

## 3. Replay

- **Virtual publisher**: the file re-enters the bus with identical envelopes; consumers
  cannot tell replay from live — that is the acceptance property (dogfooding, 07 §5 style).
- Clock modes: `realtime` (wall pacing) | `burst` (bus-max — feeds the contract suite,
  incl. its second-placement runs, as fixtures) | `step` (one message at a time — the
  debugger lane, adjudication 13, expected to merge here).
- **Honest determinism claim**: content replays, timing does not (transport jitter never
  returns); monotonic-offset rule per `dora.md:222`; `capture_ts` stays in the payload,
  seq fields are rewritten to the replay clock.
- **The dial turns with the film (D20)**: under `step|burst` the replay advances the kernel
  clock (03 §6); the file header records which clock domain was in force (`real|sim|step`)
  beside the manifest snapshot — so sleeps, timeouts and backoffs replay truthfully.
- Replayed topics carry the file-header QoS; LOSSY during `burst` means readers fast-forward
  — an ordinary outcome, not a replay bug.

## 4. Container

- **mcap** first (ROS2/Foxglove lineage; schema-embedded, protobuf+arrow encodings native —
  industry knowledge, verified-at-adoption): one Rust crate, a Stage-2 dependency.
- Fallback: own TL-framed container with the identical record model (header/records/gaps).
  Trigger: mcap health check at adoption, or a format mismatch discovered there.
- The model is container-agnostic by construction — swapping containers swaps a writer.
Adoption acceptance target (D17): a Stage-2 recording must open in Foxglove Studio
(mcap + protobuf encoding + fdset in the schema header) — failure there triggers the
fallback container, not a format negotiation with the kernel.

## 5. Year plan

Year 1: this document; recorder-as-subscriber as a DESIGN record only (zero code, zero
dependency — D5 thin edge intact). Stage 2: recorder plugin + mcap dependency + replay
publisher; CLI packaging arrives with `modukit-cli` (still out of Stage-1).
