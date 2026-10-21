# v1-STABLE factual note for the MediaServo maintainer (D89-K7)

> STATUS: DRAFT - awaiting maintainer send. This is a factual notice, NOT a proposal:
> per K7=C the wiring-hearing proposal stays armed but unopened until maintainer word
> or a first MediaServo signal. Owner of the send action: ModuKit maintainer.

## What changed on our side (facts)

- ModuKit declared its **v1 surface STABLE** on 2026-10-18 (decision record D87), closing
  the Stage-2 freeze hearing. Concretely, what is now frozen:
  - four shipped crates: `modukit-core`, `modukit-runtime`, `modukit-transport`,
    `modukit-codegen` (239 tests across 51 suites as of 2026-10-21; the repo's check script runs five standing demo acceptance gates);
  - the C ABI as the only cross-language plugin exit; kernel v1 verbs
    (lifecycle / registry / manifest / bundle / bus v1 / clock v1 / logger);
  - the process wire: TL frames `{len, ver, type, payload}` at `WIRE_VERSION = 1`,
    SCM_RIGHTS fd-passing, the checked-in protobuf contract ledger
    (`contracts/org.modukit.runtime.proto` + generated fdsets);
  - the error dictionary: 19 stable names (supervision domain added at tags 23-27).
- Amendment policy (D83): **tail-additive changes land by decision record; anything
  breaking requires a prior amendment hearing.** In practice: code or contracts written
  against v1 keep working; new fields may only ever appear at the tail. Demonstrated twice
  since: the C++ embed table and the v2 bus door each arrived as a separate table or
  tail-add - zero v1 lines touched.
- What HAS grown since the freeze (all additive, v1 untouched): the C/C++ bindings
  line (the HostEmbedV1 embed table, 24 verbs over two rungs, C++ plugin fixtures; the
  volume was accepted 2026-10-21) and the v2 plugin-facing bus door - four mailbox
  verbs (publish/subscribe/recv/unsubscribe) as a SECOND table (HostAbiV2), negotiated
  at handshake, working in-process AND across the process wall; WIRE_VERSION stayed 1
  throughout.
- What is explicitly NOT promised: the schedule-timer verb is PLANNED, not drawn
  (deferred by D96 ruling 1 to its own hearing); the UI-composition tier stays demoted
  until a real beachhead demand (K2); process placement currently covers in-host
  supervisor + runner pairs (Stage-2), remote transport is a later stage
  (§12-5, trigger-gated).

## What this note is not

- Not an integration proposal and not a schedule. MediaServo/ModuKit wiring would open
  its own hearing when (and only when) a consumer demand exists (D73 trigger, armed).
- Nothing in this note asks for a response.
