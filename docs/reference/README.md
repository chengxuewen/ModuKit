# docs/reference — External Project Reference Profiles

Series of profiles for open-source / sibling projects under `.refinfo/`
(git-ignored external clones), analyzed for what ModuKit can **depend on**,
**borrow patterns from**, or treat as **reference only**.

> Format adapted from MediaServo's `docs/reference/janus-gateway.md` analysis
> template (8 sections: portrait / architecture / capabilities / development
> & current state / ecosystem / highlights & limitations / historical
> lessons / value-for-us verdict). Per repo convention C1, every file here
> is English-only; per repo anti-pattern rules, nothing in `.refinfo/` is
> ever ModuKit content — these are labeled external observations.

## Index

| Project | Profile | Verdict | Why it matters to ModuKit |
|---|---|---|---|
| VisiaEngine | [visiaengine.md](visiaengine.md) | BORROW-PATTERNS | Completed sibling at our exact design point (Rust core + C ABI + staged delivery); source of the gate/trace/evidence discipline |
| iceoryx2 | [iceoryx2.md](iceoryx2.md) | DEPENDENCY-CANDIDATE (primary) + BORROW-PATTERNS | Stage-2 zero-copy local path; pre-1.0 but SHM-is-the-product; +100% ABI test ratio (v1 lesson); bindings/contract-layer lessons |
| zenoh | [zenoh.md](zenoh.md) | BORROW-PATTERNS (conditional dependency only if cross-machine Stage-2) | zenoh-plugin-trait = most complete dynamic-plugin ABI found (Stage-1 shape); network-first, stable 1.x |
| CppMicroServices | [cppmicroservices.md](cppmicroservices.md) | BORROW-PATTERNS (primary Stage-1 design mirror) | 6-state lifecycle + registry leases (reference!=instance) to port; C++-across-.so anti-pattern to avoid; Apache-2.0 green |
| dora | [dora.md](dora.md) | BORROW-PATTERNS (strongest Stage-2 mirror) | live multi-process graph w/ zero-copy flag API; chose zenoh over iceoryx deliberately; daemon-routed backpressure trap; 28-day silent-hang lesson |
| CTK | [ctk.md](ctk.md) | REFERENCE-ONLY (sliver: SQL install-set persistence + DEPRECATED_SINCE cutoff policy) | 6313-commit Qt/medical OSGi twin of CppMS; kernel frozen legacy (active maintenance is Qt6 hygiene); Qt LGPL runtime is the real gate; alive-but-frozen health read |
| AccessBase | [accessbase.md](accessbase.md) | BORROW-PATTERNS (gate/test culture, host-app shape) | TS/Node family sibling; PIT-numbered e2e gate commits as living ledger; ModuKit's .agents ancestor |
| MediaServo | [mediaservo.md](mediaservo.md) | BORROW-PATTERNS + DOC-DISCIPLINE SOURCE | Stage-4 mirror with a counter-signal (webrtc-rs in our canon vs libwebrtc in production); OBS-modeled declared-capability plugin contract; one-C-ABI-four-surfaces executed; PIT-71/210 link-illusion & artifact-hygiene lessons |

### Remote round (2026-10-09; GitHub projects, blobless clones measured in ~/.cache/modukit-research/)

| Project | Profile | Verdict | Why it matters |
|---|---|---|---|
| rutis | [rutis.md](rutis.md) | BORROW-PATTERNS (high priority) | Only live whitepaper-named plugin runtime (★91, MIT, pushed same day); Stage-1 semantics + its TS/Python faces as FFI-seam alternative |
| sysplugin | [sysplugin.md](sysplugin.md) | NOT-FOUND-AS-DESCRIBED | Existence audit: expected 'Rust OSGi framework' resolves to nothing — third evidence for 'never inherit unverified references' |
| Zellij | [zellij.md](zellij.md) | BORROW-PATTERNS + CAUTIONARY-TALE | Production triple-mode plugin host; external-plugin ABI broken repeatedly on version bumps — the policy motivation for our C-ABI stability rules |
| ipc-channel | [ipc-channel.md](ipc-channel.md) | BORROW-SHAPE (not dependency) | Stage-2 THIRD option: typed control-plane shape guiding an iceoryx2 data-plane (its SHM is copy-in single-consumer — not a ring, iceoryx2 status unchanged) |
| CLAP | [clap-plugin-abi.md](clap-plugin-abi.md) | BORROW-PATTERNS (strong) | Pure-C plugin ABI stability textbook (entry factory symbol, versioned extension query); honest correction: it has NO size field — we adopt size+version double guard |
| smithay | [smithay.md](smithay.md) | REFERENCE-ONLY + BORROW-PATTERNS | Stage-3 frame-intake patterns (capability registration, backend split, dispatch2 shim); refuse the crate as dependency (wrong granularity, Linux-only, 0.x churn) |
| uniffi-rs | [uniffi-rs.md](uniffi-rs.md) | BORROW-PATTERNS | Validates generated-only at Firefox scale AND challenges header-first phrasing: metadata/IDL is the contract, C ABI its checksum-sealed projection; MPL-2.0 flagged (first non-permissive in series) |
| WASM hosts survey | [wasm-plugin-hosts.md](wasm-plugin-hosts.md) | Wasmtime DEPENDENCY-CANDIDATE; Extism BORROW | Sandbox line: wasmtime already named in our canon; Extism's flat C-over-wasm host ABI = same doctrine, different arena; component-model migration friction documented |
| WebRTC-Rust audit | [webrtc-rust-state.md](webrtc-rust-state.md) | audit | webrtc-rs alive & accelerating; 'getstream-rtc' name/category ERROR (vendor SDK, 36 dl, REJECT); str0m coexists not replaces; Stage-4 decision axis is sans-I/O vs libwebrtc-FFI vs vendor |
| Whitepaper citations audit | [whitepaper-cited-unvendored.md](whitepaper-cited-unvendored.md) | audit | 7 named-but-unvendored refs: 1 strong (rutis), 1 toy (vnrit), 1 misfire (rustbridge), 4 unresolvable — canon-fix recommendations queued |

## Series status

- Snapshot date: 2026-10-09 (each profile header carries its own checkout pin).
- [00-overview.md](00-overview.md) is live: verdict matrix, Stage-2 fork decision
  sheet (iceoryx2/Zenoh), cross-cutting lessons, open maintainer items.
- Local round: eight .refinfo profiles via one 7-member team + substitutes.
- Remote round: 8 profiles + 2 audits via one 8-member team + 5 recovery
  substitutes (model failures recovered per PIT-2); all 20 files C1-verified.
- 00-overview.md updated to cover both rounds (full matrix + fork sheet + OQ queue).
- Method notes: claims measured on local clones where possible; anything not
  locally verifiable is marked UNCERTAIN in the profile itself.
