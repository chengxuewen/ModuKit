# Status Audit — Rust WebRTC options (ModuKit Stage-4 canon)

> **Canon audit, not a clone profile.** No `.refinfo/` checkout backs this file; every fact is a
> GitHub / crates.io API observation taken 2026-10-09 (unauthenticated), unless marked UNCERTAIN.
> Trigger: `docs/whitepaper.md` names `webrtc-rs / getstream-rtc` as the Stage-4 remote transport
> (§ "Remote transport | webrtc-rs / getstream-rtc"; roadmap item 4 "Integrate webrtc-rs/getstream-rtc:
> reference `vnrit`"; bridge claim "`getstream/rtc` provides `write_i420()` and `next_video_frame()`").
> Companion: `mediaservo.md` §2/§6/§8 holds the production counter-signal (libwebrtc via `webrtc-sys`);
> this doc cross-refs it and does not duplicate it. Recommendations here land as open questions
> (section 8) — the whitepaper is NOT edited by this document.

## 1. Portrait

| Field | Value |
|---|---|
| Audit subject | the two Stage-4 picks (`webrtc-rs`, `getstream-rtc`) + their real-world successors/counter-options |
| Options in scope | `webrtc-rs` (crates `webrtc` + `rtc`), `str0m`, `webrtc-sys` (LiveKit), `getstream` (Stream Video Rust SDK), `vnrit` (whitepaper's reference impl) |
| Method | `api.github.com` repo/commit/release/contributor endpoints + `crates.io` crate/version endpoints + raw README/Cargo.toml fetches, 2026-10-09 |
| Headline result | `webrtc-rs` is **alive and accelerating**; the bytebot-ai/tostream migration story **does not resolve**; `getstream-rtc` **does not exist under that name** — it is `getstream`, a preview-stage vendor SaaS SDK, not a transport library |

## 2. Architecture in focus

Four distinct shapes share the word "WebRTC in Rust":

| Shape | Representative | What it actually is |
|---|---|---|
| Full async W3C stack | `webrtc` crate (webrtc-rs) | PeerConnection-shaped API; since v0.20 built on a Sans-I/O core (see next row) — the async crate is "a clean, ergonomic, runtime-agnostic rewrite on top of a Sans-I/O core" (README, master) |
| Sans-I/O protocol core | `rtc` crate + repo `webrtc-rs/rtc` ("Sans-I/O WebRTC implementation in Rust", README: "95%+ W3C API"); and independently `str0m` | Engine without internal threads/async: caller feeds packets and time, gets events out. str0m README is explicit it is not a drop-in for thread-per-session designs "seen in JavaScript and/or webrtc-rs (or Pion in Go)" |
| libwebrtc FFI | `webrtc-sys` (repo `livekit/rust-sdks`) | cxx bindings over Google libwebrtc; cmake + Corrosion build; ~30 MB `.so`, full codec parity — MediaServo's production default (`.refinfo` evidence, `mediaservo.md` §2, counter-signal §8) |
| Vendor SaaS SDK | `getstream` (repo `GetStream/stream-video-rust`) | Server REST + client session over Stream's hosted SFU; frames as I420 via `write_i420()` / `next_video_frame()` — service SDK, not a self-hostable transport |

ModuKit relevance: only the first three can sit behind `modukit-transport`'s C ABI without a cloud dependency. The I420 bridge the whitepaper describes is real in `getstream`, but the same shape (`write_i420`/`next_video_frame`) exists in any engine that gives raw frame access — `webrtc-rs`/`rtc`/`str0m` all expose RTP-level in/out.

## 3. Key capabilities

- **webrtc-rs**: complete W3C API surface incl. DTLS/SRTP/SCTP/ICE, TURN, data channels; deterministic-time Sans-I/O core ("The Sans-I/O core no longer reads a clock. Time is an input" — README); DTLS GCM record path benchmarked "1.015 µs → 262 ns" during the 2026 work (README).
- **str0m**: small Sans-I/O ICE/DTLS/SRTP+media engine; README self-positions honestly ("A production worthy SFU probably needs an even more sophisticated [setup]"; chat example "not intended for production").
- **webrtc-sys (LiveKit)**: libwebrtc parity (hardware codec paths,NetEQ, transport-wide-cc), production-tuned; cost is the C++ toolchain + vendored checkout (see `mediaservo.md` PIT-76 for the vendor-patch trap).
- **getstream**: publish/subscribe to Stream's SFU, I420 in/out helpers, auth/session management tied to Stream cloud accounts.

## 4. Development & current state

| Option | Crate & latest release | Repo last push | Stars | Maintenance signal | License (verified) |
|---|---|---|---|---|---|
| webrtc-rs `webrtc` | 0.21.0 @ 2026-09-19 | 2026-10-03 (master) | 5,160 | rainliu merged 12/20 last commits; 6 stable releases Jul–Sep 2026 (v0.20.0→v0.21.0); `v1.x` branch opened 2026-09-19; "path to webrtc 1.0" issue #836; only 7 open issues | `MIT/Apache-2.0` per root Cargo.toml + README badge; GitHub API detects Apache-2.0 on repo (dual grant is the crate-level truth) |
| webrtc-rs `rtc` (Sans-I/O core) | published, updated 2026-09-19 (version number UNCERTAIN) | 2026-10-08 | 164 | same org, released in lock-step with `webrtc` 0.21.x | GitHub detects Apache-2.0 (crate metadata UNCERTAIN — not fetched) |
| str0m | 0.24.1 @ 2026-10-03 | 2026-10-04 | 634 | single-core author (Martin Algesten); ~monthly releases; also top-3 contributor to webrtc-rs (136 contributions) | `MIT OR Apache-2.0` (Cargo.toml); GitHub detects MIT-only from LICENSE files |
| webrtc-sys (LiveKit) | 0.3.48 @ 2026-10-07 | 2026-10-09 (rust-sdks) | 495 | top contributor theomonnom 422 commits; weekly release cadence; 3.26 M crate downloads | Apache-2.0 |
| getstream (Stream Video) | 0.1.0-preview.2 @ 2026-08-18 | 2026-10-08 | 0 | two named contributors (7+4 commits); **36 total downloads**; MSRV 1.88+ | `license-file = "LICENSE"` = custom "SOURCE CODE LICENSE AGREEMENT" (crates.io reports `non-standard`; GitHub `NOASSERTION`) |
| vnrit (whitepaper reference) | — | 2026-08-04 | 1 | hobby-scale X11→browser streamer; Cargo.toml: `webrtc = "0.20"` | Apache-2.0 |

Download counts (crates.io, same-day): `webrtc` 7,745,244 · `webrtc-sys` 3,262,186 · `str0m` 2,448,793 · `livekit` 2,278,807 · `getstream` 36.

## 5. Ecosystem

- **LiveKit** is the production gravitational center of Rust WebRTC: `livekit` + `webrtc-sys` + `webrtc-sys-build` from one repo (`rust-sdks`), the same stack MediaServo's docs cite as the napi-rs template (`mediaservo.md` §5).
- **webrtc-rs org** (website webrtc.rs) now spans `webrtc` (async façade) + `rtc` (Sans-I/O core) + the sub-crate family (`inter`, data, dtls, sctp, srtp, ice, …); 517 forks; adoption proxy: 7.7 M downloads. UNCERTAIN: named production users of webrtc-rs — only `vnrit` (1★) is directly cited by ModuKit's own whitepaper.
- **str0m** feeds the Sans-I/O ecosystem (`sans-player`/`sans-server` examples — UNCERTAIN, not verified this pass) and its author's influence is now inside webrtc-rs itself (§7.3).
- **GetStream** (getstream.io) = commercial chat/video API vendor; the Rust SDK exists to route customers to their hosted SFU — an ecosystem position, not shared infrastructure.
- **jitsi**: `jitsi/webrtc-sys` returns 404 on 2026-10-09; jitsi org carries only `jitsi/webrtc` ("WebRTC mirror for building react-native-webrtc", pushed 2026-10-06). The brief's "webrtc-sys by jitsi/livekit" resolves to **LiveKit only** (a `hatomist/webrtc-sys` fork README describes itself as "Fork of livekit/webrtc-sys", corroborating ownership). UNCERTAIN whether jitsi ever had a public webrtc-sys that was removed.

## 6. Highlights & limitations

**webrtc-rs** — Highlights: active dual-licensed full stack, runtime-agnostic Sans-I/O core fits `modukit-transport`'s "host owns I/O" shape; v1.0 freeze documented in the open. Limitations: pre-1.0 (breaking minor bumps through the 0.21 line); revived after a 2-year slump (governance still one-person-merge); codec stack = what its Rust RTP/RTCP modules implement, no libwebrtc hardware parity.
**str0m** — Highlights: smallest honest attack surface (~100 KB class, MediaServo's own sizing in `mediaservo.md` §2), clean deterministic clock. Limitations: not a W3C PeerConnection; you build session logic; effectively single-maintainer.
**webrtc-sys** — Highlights: proven production counter-signal (MediaServo default backend). Limitations: ~30 MB `.so` + cmake/Corrosion build conflicts with ModuKit's cargo-first plugin kernel premise; vendored-patch trap already paid by MediaServo (PIT-76, cross-ref only).
**getstream** — Limitations disqualify it as canon: custom source-code license (fails permissive-dependency gate), preview versioning, 36 downloads, vendor SFU lock-in. Highlight: the I420 frame API the whitepaper wanted is literally there (`write_i420`/`next_video_frame` on README today).

## 7. Historical lessons

1. **The bytebot→tostream migration did not happen (as claimed).** Audit of the rumor in the tasking: `bytebot-ai/webrtc-rs` → 404; org `bytebot-ai` holds 3 repos, its flagship `bytebot` (11,077★ desktop-streaming AI agent) is **archived since 2025-09-12**; a `tostream` GitHub org → 404 and repo search finds only unrelated hobby projects of that name. No fork/transfer/rename/relicensing of webrtc-rs is observable in any API artifact on 2026-10-09. What *is* in git history: commit 2020-12-25 "[WebRTC] move all under github.com/webrtc-rs/" (consolidation into the org) and 2021-06-29 "Update and rename LICENSE to LICENSE-MIT" (the dual-license shape). **Lesson: single-maintainer projects attract death-rumors during quiet windows — ByteBot's archival during webrtc-rs's 2025 lull is the likely origin of the conflation.** Whether a bytebot-internal webrtc-rs fork ever existed (and was deleted) is UNCERTAIN and unverifiable post-deletion.
2. **A project can be quietly dying and then flip in one quarter.** crates.io release histogram for `webrtc`: 2024 → 4, 2025 → 2, then 2026 → 25 (v0.17.0 in January; v0.20.0 July with the Sans-I/O core; six stable minors in August; v0.21.0 2026-09-19 + `v1.x` branch). Liveness checks that sample only annual commit counts misjudge such projects — ModuKit's dependency gates should measure release *cadence deltas*, not snapshots.
3. **Consolidation happened by cross-pollination, not succession.** `str0m` is not webrtc-rs's successor — `algesten` (str0m author) is the #3 webrtc-rs contributor (136 commits) and webrtc-rs 0.20 rebuilt its core *in the Sans-I/O style str0m pioneered*, while str0m's README still contrasts itself against webrtc-rs. Two engines converging on one design axis is a health signal for both; the whitepaper's either/or framing is obsolete.
4. **`getstream-rtc` is a mis-citation, not a fiction.** The crate is `getstream`; the repo `GetStream/stream-video-rust`; the `write_i420()` / `next_video_frame()` APIs quoted by the whitepaper exist verbatim in its README today. The error is *category*: a vendor's cloud-SFU SDK was listed beside `webrtc-rs` as if both were embeddable transports. Canon should never name a SaaS client where an abstraction expects an engine.
5. **Vendor forks of libwebrtc bindings are single-owner.** `webrtc-sys` is LiveKit's (jitsi attribution does not resolve). MediaServo's default backend and its vendored `[patch.crates-io]` fork (`mediaservo.md` PIT-76) show the real cost of the libwebrtc path in 2026: C++ build infra plus upstream-drift patches.

## 8. Value for ModuKit

**Verdict per option:**

| Option | Verdict | Reasoning |
|---|---|---|
| webrtc-rs (`webrtc`, pin ≥ 0.21; watch `v1.x`) | **DEPENDENCY-CANDIDATE (primary, keep)** | Active, permissive dual license (MIT/Apache-2.0 verified at crate level), Sans-I/O core matches `modukit-transport`'s host-owned-I/O contract, W3C surface + 7.7 M downloads |
| str0m | **DEPENDENCY-CANDIDATE (secondary tier: LAN P2P / embedded)** | ~100 KB class, MIT OR Apache-2.0, deterministic time; exactly the tier MediaServo reserved it for — two engines on one design axis, no either/or |
| webrtc-sys (LiveKit) | **REFERENCE-ONLY / conditional** | Production counter-signal only; adopt if and when codec-parity or weak-network requirements force libwebrtc; if forced, the build-infra cost and PIT-76 vendor-patch discipline come with it (`mediaservo.md`) |
| getstream | **REJECT as transport dependency** | Custom license, preview grade, 36 downloads, SFU lock-in; keep at most as a worked example of the I420 bridge pattern |
| vnrit | REFERENCE-ONLY, downgrade citation | 1★ hobby repo; it validates *feasibility* (pure-Rust X11→browser via webrtc 0.20), not infrastructure choice |

**Whitepaper §8 Stage-4 naming IS stale** in two respects (recommendations as open questions for the maintainer — no edits made):
- OQ-1: replace `getstream-rtc` with the correct coordinate (`getstream`, Stream Video SDK) *and* remove it from the "Remote transport" row — it is a vendor SaaS client, not a transport. Proposed row: `webrtc-rs (async) / rtc-core (sans-I/O) / str0m (LAN tier)`, with `webrtc-sys` listed as the libwebrtc escape hatch cross-referencing `mediaservo.md`.
- OQ-2: the `write_i420()` / `next_video_frame()` bridge claim should be re-attributed to ModuKit's own intended `modukit-transport` API shape (the names are fine to keep as design vocabulary — same term-status as I420 per `00-overview.md`), not to the getstream SDK as external fact.
- OQ-3: `reference vnrit` — keep as motivation, or replace with a production citation (LiveKit rust-sdks / MediaServo) once Stage 4 design pins codec requirements.
- OQ-4: pin policy for pre-1.0 webrtc-rs: cargo-pin `=0.21.x` with a mandatory re-audit when the `v1.x` branch tags 1.0 (breaking-public-API window closing, per their own README); record in `decisions.md` at Stage-4 start.
- OQ-5: add a "death-rumor clause" to dependency vetting: verify release cadence deltas + org state via API at decision time (§7.1, §7.2), never from secondhand migration stories.
