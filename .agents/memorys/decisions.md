# ModuKit — Decisions

> Format per `.agents/rules/common/lesson-memory.md`: `## D{n}: <title>` + decision + rationale + reference (source, date).
> Numbering starts fresh at `D1` for ModuKit. The PolyOrch-era decisions D1–D29 were not ported; their original lives in `PolyOrch/.agents/memorys/decisions.md`.

## D1: Project license = MIT OR Apache-2.0 dual (2026-10-09)

- **Decision**: dual license `MIT OR Apache-2.0`. Full texts at repo root (`LICENSE-MIT`, `LICENSE-APACHE`); the Cargo workspace sets `license = "MIT OR Apache-2.0"` under `[workspace.package]` once it exists (Stage-1 trigger); generated binding artifacts ship under the same dual. Contributors must DCO-sign (`git commit -s`) from the first external contribution.
- **Rationale**: embeddable-SDK form (hosts include closed commercial HMI/cockpit software per README/whitepaper positioning) requires non-viral terms; the Apache leg carries a patent grant + termination clause (real SEP exposure in automotive/industrial); the MIT leg covers GPLv2-only and NOTICE-averse legal contexts. Ecosystem default (serde/tokio/wasmtime/iceoryx2 all same dual) = zero downstream surprise. Reference-series evidence: `docs/reference/ctk.md` (Qt LGPL runtime burden), `zenoh.md` (EPL users switch to the Apache leg), `uniffi-rs.md` (MPL discourages enterprise vendoring).
- **Rejected**: MIT alone (no patent clause); Apache alone (strictly narrower — loses MIT-only scenarios); MPL/EPL (weak copyleft conflicts with embed-anywhere doctrine); GPL/AGPL or open-core commercial now (no monetization mandate in canon; the OSS dual grant keeps a future commercial layer additive, not revocable).
- **Reference**: user approval "luodi/landed" 2026-10-09 (chat); sibling precedents .refinfo/VisiaEngine (same dual), MediaServo (Apache-only).
- **Re-review triggers**: (1) a GPL-family dependency becomes a required vendoring; (2) commercial dual-licensing initiative opens; (3) first external contribution lands (verify DCO tooling is live).

