# smithay — External Reference Profile

> Research date: 2026-10-09 · Checkout: blobless clone at `~/.cache/modukit-research/smithay`
> (not under `.refinfo/`) · HEAD `cf43a02a` (master, 2026-10-08) · 4,244 commits · latest tag `v0.7.0`
> Upstream: <https://github.com/Smithay/smithay> · Cargo version `0.7.0` · License: MIT (`LICENSE.txt`)
> **EXTERNAL REFERENCE — read-only third-party clone; not ModuKit content, not a dependency, never cite as
> ours.** Claims measured on this checkout; anything not locally verifiable is marked **UNCERTAIN**.

## 1. Project portrait

| Field | Value |
|---|---|
| Project | Smithay — "a library for writing Wayland compositors" / "A smithy for rusty Wayland compositors" (Cargo.toml `description`, README) |
| Language | Rust, `edition = "2024"`, MSRV `rust-version = "1.87"` (Cargo.toml) |
| What it is | **Compositor toolkit / building blocks, NOT a finished compositor.** README Goals: "provide building blocks to create Wayland compositors … While not being a full-blown compositor". Self-described "**not a framework and does not impose constraints. You are never required to use components you don't need.**" |
| License | **MIT** — `LICENSE.txt`: "Copyright (c) 2017 Victor Berger and Victoria Brekenfeld", verbatim MIT body (Cargo.toml `license = "MIT"`). Cleanest license in this series; see §8.2 license gate. |
| Governance | Smithay GitHub org; Matrix `#smithay:matrix.org` + Libera `#Smithay`; homepage `smithay.github.io`. No `CODEOWNERS`/`policies` dir found → governance is maintainer-meritocratic, **UNCERTAIN** as formal policy. |
| Size | One published crate `smithay` + 5 workspace members: `anvil` (sample compositor), `smallvil` (minimal example), `wlcs_anvil` (compliance-suite runner), `test_clients`, `smithay-drm-extras`. Library tree: `src/backend/` (11 submodules), `src/wayland/` (**36 protocol modules**), `src/desktop/`, `src/input/`, `src/utils/`, `src/xwayland/`. |
| Velocity | **403 commits / 12 months**; **74 distinct contributors / 12 months** (git `%an`). All-time 4,244 commits / 171 author-names (over-counted — see §7.4). First commit 2017-01-20, HEAD 2026-10-08 → **~9.5 years, still 0.x.** |
| Maturity | 13 tags, all pre-1.0 (`v0.1.0`…`v0.7.0`; latest `v0.7.0` 2025-06-24). Long-plateau-then-burst release arc — §7.1. |
| Real adoption | **System76 COSMIC** (`pop-os/cosmic-epoch`), **Niri**, plus ~11 other named compositors — README "Other compositors that use Smithay" (in-repo evidence, §5). Current lead maintainer Drakulix / Victoria Brekenfeld commits under `victoria@system76.com` (git email). |
| MSRV policy | `rust-version = "1.87"` — tracks recent stable; no MSRV-downgrade policy doc found in-clone (**UNCERTAIN** whether rolling or fixed floor). |

## 2. Architecture in focus

Two orthogonal axes, cleanly separated in `src/lib.rs` (`backend`, `wayland`, `desktop`, `input`, `output`, `utils`, `xwayland`, `reexports`):
- **`backend/` = "down"** — talks to the OS/hardware (session, input, graphics). `src/backend/mod.rs`: "structured around three main aspects of interaction with the OS: session management, input handling, and graphics." Each is feature-gated (`backend_drm`, `backend_gbm`, `backend_libinput`, `backend_udev`, `backend_session_libseat`, `backend_x11`, `backend_winit`, `backend_vulkan`, `backend_libei`, `renderer_gl`/`_pixman`/`_multi`).
- **`wayland/` = "up"** — the server side of the Wayland protocol toward clients. 36 modules, one per protocol/interface, all built on **one repeated pattern** (§2.1). `wayland_frontend` cargo feature gates the whole client-facing half (pulls `wayland-server`, `wayland-protocols`, `-wlr`, `-misc`).

### 2.1 The extension / module pattern (how you add a Wayland protocol)

Every protocol module in `src/wayland/*` works identically — verbatim from `src/wayland/mod.rs`:
1. A module `*State` struct's **constructor takes the `DisplayHandle` and inserts one or more globals** (e.g. `ShmState::new(&display.handle(), vec![Format::Yuyv, Format::C8])`, `shm/mod.rs`).
2. The `*State` is **stored inside your global compositor state** — the same type you parameterized `wayland_server::Display<State>` over.
3. You **implement a module-specific `*Handler` trait** on your state (`ShmHandler`, `BufferHandler`, `DmabufHandler`, …) — called when protocol events need custom handling.
4. You call the matching **`delegate_*!` macro** (`delegate_shm!`) to wire the required `wayland_server` `Dispatch`/`GlobalDispatch` impls.
5. **Drop the `*State` to remove the global** — capability add/remove is literally constructor/destructor scope.

`src/wayland/dispatch2.rs` is the emerging refinement: `Dispatch2`/`GlobalDispatch2` traits, doc-commented "**A future version of `wayland-server` will replace `Dispatch` with this**" — a simplified request/bind handler surface smithay carries in-tree ahead of upstream.

This is a **capability-registry pattern**: opt-in modules register interface globals, declare which versions/formats they support, and deregister by dropping. (Maps directly onto ModuKit's "capability discovery … declared and checked explicitly", §8.1.)

### 2.2 Frame intake — SHM vs dmabuf (the ModuKit Stage-3 path)

ModuKit Stage-3 is "main HMI as compositor; child processes submit frames via Wayland/X11 or SHM (I420)". Smithay's two client→compositor content paths are exactly the frame-intake machinery. Both feed the **surface commit** path, both converge on one `Buffer` type.

- **`wayland/compositor`** (`src/wayland/compositor/mod.rs`) registers the `wl_compositor`/`wl_subcompositor` globals and "stores in a coherent way the state of surface trees … and handles the application of **double-buffered state**." This is where an attached `wl_buffer` becomes visible on `surface.commit()`.
- **`wayland/shm`** (mod.rs + `pool.rs`) — "SHM is the most basic way wayland clients can send content … by sending a **file descriptor** to some (likely RAM-backed) storage … accessing their contents as **simple `&[u8]` slices**." `ShmState` advertises formats (ARGB8888/XRGB8888 mandatory; extras like Yuyv/C8 declared by the compositor). Zero hardware acceleration required. → **This is the closest analogue to ModuKit's SHM/I420 child-frame path.**
- **`wayland/dmabuf`** (mod.rs + `dispatch.rs`) — linux-dmabuf: "clients submit their contents as **dmabuf file descriptors**". Setup needs a `DmabufFeedback` (main device + supported (code, modifier) format pairs, typically read from a renderer via `ImportDma::dmabuf_formats`) and a `DmabufHandler` to test renderer importability. Tightly linked to `backend::allocator` (`src/backend/allocator/dmabuf.rs` = the `Dmabuf` type; also `gbm.rs`, `dumb.rs`, `udmabuf.rs`, `format.rs`, `swapchain.rs`).
- **Convergence type**: `src/utils/geometry.rs:23` `pub struct Buffer` carries `BufferContents` — either a dmabuf set of planes or an SHM `MmapGuard` — so downstream code (renderer, or a ModuKit frame consumer) treats both intake paths uniformly. `wayland/buffer/mod.rs` defines the `BufferHandler` trait (`buffer_destroyed`) shared by shm and dmabuf.
- **Composition** (optional, feature `desktop`): `src/desktop/mod.rs` provides opinionated `Window`, `Space`/`SpaceElement`, `LayerSurface`/`LayerMap`, popup helpers and `render_output` — the "place surfaces in 2-D, stack, scan out" layer a full HMI uses. A frame-intake-only host can ignore this module entirely.

```
        child process (Wayland client)                      ModuKit host  (= a smithay-style server)
  ┌───────────────────────────────────┐   register globals   ┌───────────────────────────────────────────┐
  │ wl_shm::create_pool(fd)           │  ◀── ShmState::new ──┤   wayland::shm   (pool.rs: fd → mmap &[u8]) │
  │   → wl_buffer   (non-accelerated) │                       │        │                                   │
  │                                   │                       │        │  attach + surface.commit()        │
  │ linux-dmabuf::params → create_immed│ ◀─ DmabufState + ───┤   wayland::dmabuf (DmabufFeedback/Handler)  │
  │   → wl_buffer   (accelerated)     │   Feedback           │        │                                   │
  │                                   │                       │        ▼                                   │
  │ wl_compositor::create_surface ────┼──────────────────────▶│   wayland::compositor                     │
  │   .attach(buf); .commit()         │                       │   (surface tree + double-buffered state)  │
  └───────────────────────────────────┘                       │        │ BufferHandler::buffer_created     │
                                                             │        ▼                                   │
                                                             │   backend::allocator::Buffer              │
                                                             │   { Dmabuf | Mmap(&[u8]) }               │
                                                             │        │                                   │
                                                             │   renderer → drm/output  ◀── FULL compositor│
                                                             │   (ModuKit Stage-3 stops at Buffer, hands   │
                                                             │    the frame to its own SHM/I420 consumer)  │
                                                             └───────────────────────────────────────────┘
```

## 3. Key capabilities

- **Wayland server building blocks**: core protocols + official `wayland-protocols` extensions + *some* wlr-protocols and KDE extensions (README Goals). 36 protocol modules under `src/wayland/` (compositor, shm, dmabuf, seat, output, shell/xdg, presentation, viewporter, `image_copy_capture`, `image_capture_source`, fractional_scale, single_pixel_buffer, security_context, drm_lease, …).
- **Uniform extension pattern** — `*State` constructor → insert global; `*Handler` trait → callbacks; `delegate_*!` → wiring; drop → remove (§2.1).
- **Dual frame intake** — SHM (`&[u8]`, no accel) and linux-dmabuf (fd + modifier/feedback, accelerated), both unified into one `Buffer` type (§2.2).
- **Backend abstraction** (feature-gated): session via logind/seatd (`libseat`), input via libinput (`input` crate) / udev device discovery, graphics via allocator (GBM) + renderer (GLES2 / pixman / glow / vulkan / multi). Plus **X11 and `winit` backends** that run a smithay compositor *as a client of another display* (dev/test + nested).
- **Input routing to clients**: `wayland/seat` + `wayland/selection` forward pointer/keyboard to focus and handle clipboard/DnD — the "input events routed back" leg of Stage-3.
- **XWayland**: `src/xwayland/` (xwm, recent focus-tracking fixes visible in HEAD log) — optional.
- **Test/reference surface in-tree**: `anvil` (full sample compositor), `smallvil` (minimal), `wlcs_anvil` (Wayland Landrun Compliance Suite runner), `test_clients`, `renderer_test` feature, `buffer_test`/`geometry`/`benchmark` test harnesses.
- `reexports.rs` re-exports `wayland_server`, `wayland_protocols*`, `calloop`, `gbm`, `drm`, `input`, `winit`, `x11rb`, `rustix` so downstream pins one version — smithay and wayland-rs are **co-maintained** (same org).

## 4. Development & current state (measured)

- HEAD `cf43a02a` (master, 2026-10-08); a CHANGELOG commit is literally the tip ("CHANGELOG: Mention input_intercept() change"). Actively maintained.
- 403 commits / 74 contributors in the last 12 months — healthy, not decaying.
- Published crate is **`0.7.0`** (2025-06-24). Master carries post-0.7.0 work; 0.8.0 not yet tagged at this checkout.
- Recent churn is exactly the client-facing protocol/handler layer (the ModuKit-relevant half): `input: rework virtual_keyboard to work like a regular device`, `input: implement zwlr_virtual_pointer_v1`, `image-copy: Raise the duplicate_frame protocol error`, `Guard against negative sizes passed in wl_region add/subtract requests` (hardening client input), `desktop: don't send frame callbacks to unmapped surfaces`. API signatures still move (`input_intercept()` gained a source arg) → **breaking-ish churn continues in `main` between minors.**
- `edition = "2024"`, MSRV 1.87 — rides recent stable, so it is not a low-MSRV conservative library.

## 5. Ecosystem

Downstream users **named in-repo** (README "Other compositors that use Smithay") — this is local evidence, not web lore:
- **COSMIC** (`pop-os/cosmic-epoch`) — System76's next-gen desktop; lead maintainer Drakulix/Victoria Brekenfeld commits under `victoria@system76.com`, so smithay is COSMIC's actual engine.
- **Niri** (scrollable-tiling), **Catacomb** (mobile), **MagmaWM**, **Strata**, **Pinnacle**, **DriftWM**, **Otto**, **Denial**, **emskin** (nested, embeds apps in Emacs), **wprs** ("like xpra, but for Wayland" — remote/transpositor, adjacent to ModuKit Stage-4), **Sudbury** (ChromeOS), **Local Desktop** (Android PRoot+Wayland).
- **`anvil`** is smithay's own sample compositor; `wlcs_anvil` runs the official compliance suite against anvil — proof the library passes protocol conformance, not just compiles.

Sister crates / same Smithay org (`reexports.rs`, Cargo deps): **wayland-rs** (`wayland-server` 0.31.13, `wayland-protocols` 0.32.13, `-wlr`/`-misc` 0.3.12, `wayland-backend` 0.3.15) and **calloop** 0.14 (event loop) — smithay is the compositor-side consumer of the wayland-rs ecosystem it co-homes. **Health/activity of the named downstreams is UNCERTAIN** (not verifiable from this clone).

## 6. Highlights & limitations

**Highlights**
1. **The** battle-tested Rust Wayland *server* toolkit — the direct reference implementation for ModuKit Stage-3's "child processes submit frames via Wayland … input events routed back" path, with the SHM `&[u8]` intake as a near-exact match for the I420-over-SHM design.
2. **Capability-registration pattern is liftable as-is**: global-per-protocol `*State` + `*Handler` + `delegate_*!` + drop-to-remove is a clean model of "declare a capability, wire its callbacks, deregister" — precisely ModuKit's capability-discovery contract.
3. **Backend abstraction over one trait surface** (DRM / X11 / winit / libei behind interchangeable input + allocator + renderer traits) mirrors ModuKit's "one IPC/transport interface over local/remote."
4. **Buffer-type unification** — SHM and dmabuf converge on a single `Buffer { Dmabuf | Mmap(&[u8]) }` so consumers are path-agnostic; exactly the abstraction ModuKit's frame consumer needs.
5. **MIT license** — zero legal friction; borrow patterns or even code freely (§8.2).
6. **Conformance-proven** (wlcs_anvil) and **self-documented** (module-level doc-comments with runnable `ShmState::new` / `DmabufState` examples ModuKit can crib).

**Limitations**
1. **It is a full compositor toolkit, not a frame-intake library.** The SHM/Buffer path you want is welded to `wayland-server`, surface/double-buffer machinery, and (if you take `desktop`) renderer/output. You do **not** want the drm/gbm/session/xwm half.
2. **Linux-only by construction** (dmabuf, GBM, `libseat`, udev, DRM, libinput). ModuKit Stage-3 also targets **Win32 `SetParent` and macOS Cocoa** (README platform table) — smithay contributes **nothing** there; the Windows/macOS frame paths are entirely ModuKit's problem.
3. **0.x churn, high MSRV**: 9.5 years with no 1.0, breaking signature moves between minors, `edition 2024` + MSRV 1.87 — a moving, modern-Rust target, not a stable frozen ABI.
4. **Heavy transitive/system dep surface**: `libwayland`, `libxkbcommon`, `libudev`, `libinput`, `libgbm`, `libseat`, `xwayland`, `libEGL`, `libpixman`, `libdisplay-info` (README System Dependencies). Wrong shape for a "lightweight plugin host."
5. **No first-class "embed just the protocol server" story** — extraction means depending on the whole crate (feature-subset) or forking `wayland/shm`+`compositor`, not calling a narrow intake API.
6. **Bus-factor concentration**: one maintainer (Drakulix/Victoria Brekenfeld) accounts for roughly half of all-time commits (§7.4) — the single-critical-mass risk this series flags on zenoh/uniffi, sharper here.

## 7. Historical lessons

1. **The 0.x churn arc — long plateau, then a burst.** Tags: `v0.1.0` 2017-10 → `v0.2.0` 2019-01 → `v0.2.1` 2020-02 → `v0.3.0` 2021-07, then a **~3.5-year gap with no release** while master kept moving, then `v0.4.0` 2025-01 → `v0.5.0` 2025-02 → `v0.6.0` 2025-04 → `v0.7.0` 2025-06 (four minors in five months, incl. same-day micro-bumps `v0.4.1–4`). Lesson: a pre-1.0 toolkit can be *development-active but release-dormant* for years, then dump a huge accumulated breaking change; **pin to a tag and budget for the next burst**, never track `main`.
2. **Backend abstraction evolution.** Smithay grew from a single DRM/GBM on-TTY path into a swappable multi-backend: session (`libseat`/logind), input (libinput/winit), and *nested* backends (X11 via `x11rb`, `winit`) that let a compositor run as a client for testing and embedding. The abstraction (traits per capability, `#[cfg(feature=...)]` per backend) is the mature artifact — the concrete backends are secondary. Lesson: **separate "what a capability needs" (trait) from "how this OS provides it" (feature-gated impl)** — the pattern ModuKit's `modukit-platform-{linux,win,macos}` crates must reach.
3. **A shim for an ABI that is still moving.** `dispatch2.rs` (`Dispatch2`/`GlobalDispatch2`) exists because smithay anticipates a `wayland-server` dispatch-API replacement it cannot take yet. Lesson: when your host sits on a fast-moving lower layer, **hold your own adapter trait** rather than re-exporting theirs, so the churn is absorbed in one file — directly relevant to ModuKit's C ABI staying stable while deps move.
4. **Identity-fragmented history (methodology caveat).** `git shortlog`/`%an` has **no `.mailmap`**, so one person appears as multiple names: *Drakulix* = *Victoria Brekenfeld* = (likely) *Victor Brekenfeld* ≈ 2,071 combined commits (~49% of history); *Ian Douglas Scott* = *i509VCB* ≈ 390. My "171 contributors" is therefore an **upper bound**; true all-time contributor count is lower. Lesson (for ModuKit's own gate metrics): normalize author identity before drawing maintenance/velocity conclusions.

## 8. Value for ModuKit

**Verdict: REFERENCE-ONLY + BORROW-PATTERNS (Stage-3 Wayland frame-intake design mirror) — NOT a dependency for the SHM/I420 path.**

### 8.1 Adopt / Adapt / Avoid vs Stage 3

| Aspect | Position | Detail |
|---|---|---|
| Capability-discovery shape (§2.1) | **Adapt (pattern)** | `*State` ctor inserts a global + declares versions/formats, `*Handler` for callbacks, `delegate_*!` wires it, **drop-to-remove** = register/deregister a capability. Mirror this for `modukit-compositor` protocol registration and, more generally, for ModuKit's "capability discovery, declared and checked explicitly." |
| Backend-vs-trait split (§7.2) | **Adapt (pattern)** | "one trait per capability, feature-gated impls per OS/device" is exactly the `modukit-platform-*` + `modukit-transport` design; smithay proves it at scale on Linux. |
| SHM → `&[u8]` buffer intake (§2.2) | **Study (mechanism)** | `wayland/shm/pool.rs` is a reference for the mechanics ModuKit needs on Linux: fd receipt over the socket, mmap, format/size validation, guarding against malformed client input (cf. HEAD's "guard against negative sizes" hardening). Even if ModuKit defines its own protocol, crib the intake/validation shape. |
| Single `Buffer { Dmabuf \| Mmap }` unification (§2.2) | **Adopt (concept)** | Path-agnostic frame type is what ModuKit's Stage-3 consumer wants; adopt the *idea* of one buffer type over two intake paths. |
| **Whole `smithay` crate** | **AVOID as dependency** | Wrong granularity + wrong platform span + moving 0.x target (§6.1–6.4). Do not pull it for the frame-intake subset. |
| Real Wayland compositor, Linux-only branch | **Conditional future dependency (gate as a D-record)** | *If* ModuKit decides Stage-3 on Linux should host genuine Wayland clients (rather than a bespoke SHM protocol), smithay is the strongest candidate — but that is a whole-compositor, Linux-only commitment; decide it explicitly, do not drift into it. |

### 8.2 License gate

**MIT** (`LICENSE.txt`, "Copyright (c) 2017 Victor Berger and Victoria Brekenfeld"). This is the most permissive license in the reference series (vs zenoh/iceoryx2 Apache-2.0, uniffi MPL-2.0, CTK LGPL). Consequences:
- No copyleft, no NOTICE contamination, no per-file obligation. ModuKit may **borrow patterns freely and even copy code** with attribution retained. Compatible with **any** ModuKit license TBD.
- The binding constraint here is **scope and platform coverage, not license** — §8.3. MIT means the license never blocks adoption; the decision to not depend on smithay is purely architectural.
- Gate action: none required. If smithay code is ever lifted, retain the MIT copyright header in the copied file and note it in `decisions.md`.

### 8.3 The honest "subset, not a compositor" note

ModuKit Stage-3 needs a **frame-intake subset**: receive buffers that child processes commit (SHM in particular, per the I420-over-SHM design), plus the input-routing-back leg. Smithay is a **complete Wayland compositor library** — the intake path ModuKit wants is real, clean, and MIT-licensed, but it arrives welded to surface/compositor machinery and, if you take the optional `desktop`/`backend` halves, a renderer + DRM + GBM + session stack ModuKit does not need. On Linux the *patterns* (§8.1) transfer directly; on Windows/macOS smithay offers **nothing** (Wayland is Linux-only), so those Stage-3 platform rows remain entirely ModuKit-authored. Net: **study it hard for the capability-registration + SHM-buffer + backend-trait patterns; treat the crate itself as reference, not a build dependency.**

### 8.4 Not verified here (UNCERTAIN)
- **Downstream activity/health** of COSMIC, Niri, etc. (named in README; not checkable from this clone) — verify before citing them as live adoption proof in the whitepaper.
- **MSRV policy** (rolling vs fixed floor), formal **governance/`CODEOWNERS`**, and any **1.0 roadmap intent** — not found in-clone.
