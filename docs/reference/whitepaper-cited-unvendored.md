# Whitepaper-Cited, Unvendored References — Existence Audit

> Research date: 2026-10-09 | Method: GitHub search API queries per name,
> top-3 hits recorded verbatim; "intended referent" judged from whitepaper
> context (`docs/whitepaper.md` §8). This is a CANON AUDIT: it reports what
> the cited names actually resolve to. Recommendations only — the whitepaper
> is verbatim maintainer-provided canon and was not edited.

## Why this audit exists

The adoption tribunal flagged a recurring failure mode ("smuggled/aspirational
references"): citations inherited without provenance. Before Stage-1 design
work cites its reference set, each named project must at least EXIST as
described. Seven whitepaper-named references were not vendored in `.refinfo/`;
all seven were queried today.

## Findings

| Whitepaper reference (stage) | Resolves to | Evidence (2026-10-09) | Audit verdict |
|---|---|---|---|
| `rutis` (Stage 1 — in-process kernel) | **arcships/rutis** | ★91, pushed **same day**, MIT, "plugin runtime for programs that keep running — Rust core, TypeScript and Python" | **REAL & highly relevant** — the closest active Rust analog to Stage 1; full profile commissioned (see rutis.md) |
| `rustbridge` (Stage 1) | rust-community/rustbridge | "[DEPRECATED] Workshop material to teach Rust in the style of Railsbridge" — a teaching-workshop repo, last push 2018 | **MISFIRED REFERENCE** — no plugin-framework project answers to this name (a crate-name confusion candidate: possibly meant another FFI bridge project) |
| `event-engine` (Stage 2 — multi-process/zero-copy) | unresolvable | query returns twisted/nautilus/siddhi — no project of that exact name fits "multi-process plugin IPC" | **UNRESOLVABLE** as a proper noun; treat as a category descriptor ("event-driven engines"), pick concrete references from the profiled set instead |
| `RS VST Host` (Stage 2) | Jadujoel/rs-vst-host ★3 / RustAudio/vst-rs ★1061 | vst-rs (the real Rust VST lineage) last pushed **2023-06**, archived-adjacent stagnation; tiny host repos unrelated | **PARTIAL** — the ecosystem exists but is stale; audio-plugin hosting patterns remain borrowable (host loads .so/.dll via C entry points — same shape as our C-ABI seam), but cite `vst-rs` lineage with a staleness flag |
| `Parallax` (Stage 3 — UI composition) | unresolvable | hits are bevy-parallax (scrolling backgrounds) and agent-defense tooling | **UNRESOLVABLE** — no Rust Wayland-composition project answers to this name; use `smithay.md` (profiled) as the Stage-3 concrete reference |
| `Scarlet` (Stage 3) | unresolvable | hits: an OS-kernel hobby project, a color library, a toy language | **UNRESOLVABLE** — same remedy as Parallax |
| `vnrit` (Stage 4 — remote WebRTC) | nlsidf/vnrit | ★1, Apache-2.0, "Lightweight X11 WebRTC streaming server", pushed 2026-08 | **REAL but marginal** — single-digit adoption hobby project; usable as an implementation sketch, NOT as an authority; `webrtc-rust-state.md` (profiled) is the Stage-4 evidentiary doc |

## Recommended canon actions (maintainer decision items)

1. **Add rutis** to the Stage-1 reference set alongside CppMicroServices
   (it is the only live Rust plugin runtime found; CppMicroServices supplies
   the semantics, rutis may supply a Rust-shape comparison — see its profile).
2. **Drop or correct** `rustbridge` (deprecated teaching repo), `event-engine`,
   `Parallax`, `Scarlet` (unresolvable names) at the next whitepaper revision —
   replacing them with the profiled set: ipc-channel, dora, iceoryx2, zenoh,
   smithay, zellij, CLAP.
3. Keep `vnrit` only as a "toy precedent" citation; Stage-4 authority should
   come from the webrtc-rust-state audit (str0m/tostream vs libwebrtc-sys).
4. Every future citation into the whitepaper should carry the audit line used
   here (name → repo → stars/pushed/license) — institutionalizing the P4
   lesson: never inherit an unverified reference.

## Method note (reproducibility)

Queries: `curl https://api.github.com/search/repositories?q=<name>&per_page=3`
executed 2026-10-09; fields printed: full_name, stars, pushed_at, description
(first 80 chars), license spdx. Unauthenticated rate limits apply; numbers are
snapshot values, not constants.
