# 11 - Media-Server Base (Doctrine Card)

> Deliberately thin: pointers only, never restates normative text (placement ruling 7
> of the 2026-10-15 walkthrough). Rulings were user-adjudicated one by one on 2026-10-15 (votes A / A'' /
> C / C / C / C). Full records: `.agents/memorys/decisions.md` D68-D73.

## Stance (D68)

Control-plane base (P1) when serving a media server: the kernel owns
lifecycle/registry/manifest/supervisor/ABI governance; the media pipeline core stays
compiled; GPU inner loops are core, never plugins. Industry comparison and sibling
facts: `docs/reference/mediaservo.md`, `docs/reference/00-overview.md`.

## Frame clause (D69) - law lives at home

The pool + descriptor channel is the ONLY path frame bytes take to a plugin, any
placement. Normative wording: `docs/modules/07-transport-architecture.md` §3 (promoted
clause) and `docs/modules/01-plugin-placement-model.md` §3 (rule 3/7 note). Contract
signatures pass handles/descriptors/registration ids - never raw frame pointers, never
foreign malloc'd buffers. A dl plugin mapping the pool is a trusted-tier privilege; SDK
consumers take frames over API borrow/pull/out-buffer (Visia-validated shapes).

## Factories and work units (D70)

The kind+formats+priority query rides the plugin family's own contract, NOT the
registry; observable work units (drainable/countable) register ephemeral service
records and deregister on completion. Law + default rule: `docs/modules/02-service-model.md`.

## Coexistence doorbell (D71)

`parallel_ok` routing stays inert; the media-base recheck did NOT trigger it - swaps
ride the two shipped lanes (exe bundle swap, dl next-boot). Doorbell wording:
`docs/modules/03-lifecycle-state-machine.md` §update path.

## Vocabulary (D72)

Audit passed: existing hard words cover the base; device entrances (capture nodes,
encoder render nodes) ride deployment `resources{}` + `com.vendor.*` notes. Iron rule
unchanged: hard words follow adjudicated mechanisms. Evidence:
`docs/modules/05-manifest-and-deployment.md` §capabilities.

## Integration status (D73)

Parallel, not wired - declared intent, no coupling until the kernel API stabilizes;
direction library-first when the day comes. MediaServo facts stay at
`docs/reference/mediaservo.md`; the C-ABI export-freeze trigger keeps its anonymous
wording (`docs/architecture.md` §11 tail note).

## What this card never does

Never copies normative text. When a home law changes, change the home first; the card
only re-points. (Frozen-whitepaper/dual-code drift lesson: `docs/reference/polyorch.md` §7.)
