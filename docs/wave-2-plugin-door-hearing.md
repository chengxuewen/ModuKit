# Wave-2 Plugin-Door Hearing - language plugins via the wire (D106)

> Date: 2026-10-23. Question: how do Python and Node become plugin AUTHORING
> languages (the whitepaper's "polyglot extension toolkit" claim currently ships
> three languages on the embed side and only Rust + C++ on the plugin side).
> Four adjudications walked one by one with the maintainer (plain-language
> protocol). This hearing adjudicates boundaries only - no code (D97 precedent).

## Roster of adjudications

1. Which door the script languages enter through (this volume decides the shape).
2. Whether the process wire is promoted to a published contract original.
3. Skin form: home, hand-written vs generated, artifact shape.
4. Acceptance surface: which gates guard the door.

## Rulings

| # | Decision | Ruling |
|---|----------|--------|
| 1 | Door | **Option "C-proposed" - wire-native now, embed-road armed.** A plugin is a LANGUAGE PROCESS speaking the existing process wire (TL frames + HELLO self-report) through the inherited control fd - zero new dependencies, zero kernel change (the exe arm has always accepted arbitrary executables: the stand-in-executable precedent, modules/01 §2; manifest's runner field is already plumbed). The embedded-interpreter road (.so hosting CPython/QuickJS) is ARMED with named offline blockers (no Python.h on this box; no JS engine crate in the cache) - it returns as a fast-lane volume when a same-process-need or egress lands. |
| 2 | Wire contract | **Option "A" - full promotion.** Frame ledger + golden frame vectors enter contracts/ (same checked-in grammar as the fdsets); a new Rust-side parity leg (fast section, never interpreter-skippable) re-exports goldens from the encoder and byte-compares, with a built-in --self-test hook (house C0); both language skins' conformance tests read the SAME vectors - three parties shake on one stone. Amendment policy rides D83 verbatim: WIRE_VERSION byte already heads every frame; new frame types tail-append, breaking changes require a hearing. |
| 3 | Skin form | **Option "A" - bindings shelf, hand-written, fixtures band up.** `bindings/py/` + `bindings/js/` each gain a plugin-side runtime (framing loop, self-report, register/call/bus/clock marshalling) - hand-written on purpose: the event loop is the skin's substance and the frame ledger is dozens of bytes (a generator for a newborn contract is a mold before the casting). An emitter extension is ARMED for when the ledger proves it changes. Artifact shape: the plugin itself is an executable script (shebang python3 / node), manifest kind=exe, runner field optional. test-plugins/ completes its language bands: rs/ cpp/ py/ js/. |
| 4 | Acceptance | **Option "A" - two-layer guard + ledger table.** Fast section: the env-free frame-parity leg (ruling 2). Demo legs: two new ones (wave2-plugin-py, wave2-plugin-js), each opening with its skin-vs-golden conformance stage, then a real end-to-end lifecycle (host spawns the script plugin; register/call/bus roundtrip; the SAME fidelity constant echoes across the new door), then an embedded orphan census stage (kill the host, zero strays - supervision is intrinsic to a process door, no separate sup leg), each with a sabotage lever. Capability ledger gains its plugin-direction table (frame types + plugin ABI face) inside the existing self-tested ledger leg. Leg-count prose (AGENTS/status/README) syncs at landing - the three-site rule this volume already learned. |

## Open ledger (explicitly not silently swallowed)

- Embed road (in-process interpreters, dl-position scripts): armed; unblock = egress
  (dev headers / engine crates) OR a first plugin that truly needs the same-process
  fast lane.
- Skin emitter/codegen for the plugin direction: deferred until the frame ledger is
  hammered by a real plugin (ruling 3).
- res:// cross-bundle resources + share-resources word: untouched (D30 triggers).
- CLI day (modukit resource scan, modukit-cli crate): untouched (D97-B grammar).

## Verified sources (orchestrator re-read the same day)

- exe arm eats arbitrary executables: modules/01 §2 (stand-in executables in early
  tests; modukit-run is the REFERENCE runner, --fd=N convention honored);
  manifest.rs:49 runner field in place.
- HELLO carries the plugin's OWN self-report: modukit-runtime/src/main.rs header
  (dlopen is the runner's choice, not the kernel's requirement).
- Offline blockers of the embed road: /usr/include/python3.8/Python.h absent (ls,
  this session); registry cache 135 crates with pyo3 family present but NO js engine;
  headers decisive regardless.
- Golden-vector precedent: contracts/*.proto.fdset checked-in + regen-diff leg
  (check.sh fast section); frame header carries WIRE_VERSION=1 (proc_wire.rs).
- Whitepaper naming: modukit-script-* are language-axis RUNNER PACKAGES, not core
  mechanisms (D2/§9) - ruling 1's door matches the adjudicated taxonomy.

## Review trail

- [ ] plan document (docs/wave-2-plugin-door-plan.md) - to be drafted, Momus review pending
- [ ] Maintainer approval word
