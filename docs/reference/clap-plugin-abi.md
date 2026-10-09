# Reference Profile — CLAP (free-audio)

> **External reference.** This is a read-only third-party checkout under `~/.cache/modukit-research/clap`;
> it is not ModuKit content and none of it is vendored or copied into this repository.
> Research date: 2026-10-09 · Checkout: `a47f6ba` (main, 2026-07-28; `origin/next` = `fb15bc4`, 2026-09-14, 4 commits ahead) · Upstream: <https://github.com/free-audio/clap> · Header ABI version: 1.2.10 · License: MIT
> All facts below are derived from this local clone unless marked UNCERTAIN.
> Note: the subject repo bundles the extension headers itself (`include/clap/ext/**`); a separate `clap-extensions` repo was not found at the expected URL and was not cloned. Extensions are analyzed from the in-repo headers.

## 1. Portrait

| Field | Value |
|---|---|
| Project | CLAP — "CLever Audio Plugin" — a pure-C ABI for audio plugins (synths, FX) ↔ DAW hosts (README) |
| Language | C headers only (`add_library(clap INTERFACE)`, CMakeLists:21); 68 headers, 5,323 lines; **no compiled library** — plugins/hosts just `#include` |
| Governance | Multi-vendor "free-audio" consortium (Bitwig / u-he / REAPER authors appear in core header history); **no** GOVERNANCE/CONTRIBUTING file in the clone → process UNCERTAIN |
| License | MIT, `Copyright (c) 2021 Alexandre BIQUE` (LICENSE) — pure permissive, no copyleft leg |
| Core author | Alexandre Bique (~1,509 commits across two name spellings) dominates; Paul Walker (34), joshnatis (22), Dalton Messmer (21), trinitou (20), Robbert van der Helm (18), Adrien Prokopowicz (REAPER) contribute |
| Size | 1,725 commits; first 2014-09-29; 39 tags |
| Velocity | 31 commits / ~11 contributors in last 12 months — low churn, maintenance cadence on a frozen ABI |
| Maturity | 1.0.0 tagged 2022-06-03; latest 1.2.10 tagged 2026-07-13; ABI held at `major = 1` for 4+ years |
| Roles in repo | `include/clap/` (core structs), `include/clap/ext/` (27 stable + 18 draft extension interfaces), `include/clap/factory/` (2 stable + 2 draft factories), `src/` (plugin template + sample main), `conventions/` (the ABI-stability rulebook) |

## 2. Architecture in focus

### 2.1 The entrypoint — an exported **data symbol**, not a function
`include/clap/entry.h` closes with `CLAP_EXPORT extern const clap_plugin_entry_t clap_entry;`. The host `dlsym`s the symbol **`clap_entry`** (a global struct, not an exported call), and that struct carries the version plus three function pointers:

```c
typedef struct clap_plugin_entry {
   clap_version_t clap_version;               // initialized to CLAP_VERSION {1,2,10}
   bool(CLAP_ABI *init)(const char *plugin_path);
   void(CLAP_ABI *deinit)(void);
   const void *(CLAP_ABI *get_factory)(const char *factory_id);   // e.g. "clap.plugin-factory"
} clap_plugin_entry_t;
```

Version negotiation is one integer compare: `clap_version_is_compatible(v)` returns `v.major >= 1` (`version.h`). `init`/`deinit` are matched-pair DSO load/unload; since **1.2.0** the spec dropped the "called exactly once" requirement — a plugin that wraps another CLAP can be inited twice in one process, so writers **must** guard with a mutex + refcount (`src/plugin-template.c` `entry_init_guard` shows the canonical counter/`mtx_lock` recipe). This is the whole load path: symbol → version check → `get_factory`.

```text
  host dlopen(.clap) -> dlsym("clap_entry")
        |
        v
  clap_plugin_entry  { clap_version, init, deinit, get_factory(id) }
        |  get_factory("clap.plugin-factory")
        v
  clap_plugin_factory { get_plugin_count, get_plugin_descriptor, create_plugin(host,id) }
        |  create_plugin  ->  a live instance
        v
  clap_plugin { desc, init, activate, process, get_extension(id), ... }
        |  host passes a  *clap_host { clap_version, get_extension(id), request_* }  INTO the plugin
        v
  extensions: BOTH sides call get_extension(name) -> const void* (NULL = absent)
     plugin->get_extension("clap.params")  -> clap_plugin_params   (host queries plugin)
     host  ->get_extension("clap.log")     -> clap_host_log         (plugin queries host)
```

### 2.2 Plugin / host / descriptor structs (exact shape)
- **`clap_plugin_descriptor_t`** (`plugin.h:12`): first field `clap_version_t clap_version`, then opaque identity strings — `id` ("com.u-he.diva", reverse-URI), `name`, `vendor`, `url`, `version`, `description`, and a null-terminated `const char *const *features` keyword list (`plugin-features.h`).
- **`clap_plugin_t`** (`plugin.h:41`): `desc` pointer + `void *plugin_data` (opaque self handle) + lifecycle vtable `init / destroy / activate / deactivate / start_processing / stop_processing / reset / process / get_extension / on_main_thread`.
- **`clap_host_t`** (`host.h`): `clap_version` + `void *host_data` + `name/vendor/url/version` + `get_extension` + three async requests `request_restart / request_process / request_callback`.

**Honest deviation from the task hint:** CLAP core structs carry **NO `size` field** — verified across `entry.h`, `plugin.h`, `host.h`, `factory/plugin-factory.h` (grep for `size`/`abi_size` → none). Cross-boundary stability rests only on (a) the leading `clap_version_t` on the entry, descriptor and host, and (b) the never-reordered, append-only field layout of a frozen `major = 1` struct. This is *lighter* than a size-versioned COM/`cbSize` style and is a deliberate, load-bearing design choice — see §7.

### 2.3 Extension query — a string-keyed C service registry (`DIRECT` ModuKit input)
Both sides expose the identical lookup — `get_extension(self, const char *id) -> const void *` returning **null if unsupported**. Host and plugin are *peers* offering interfaces the other may or may not provide:

```c
// host extension (host offers, plugin queries)
const clap_host_log *log = host->get_extension(host, CLAP_EXT_LOG);        // id "clap.log"
// plugin extension (plugin offers, host queries)
const clap_plugin_params *params = plugin->get_extension(plugin, CLAP_EXT_PARAMS);  // id "clap.params"
```

Conventions (`README` "Extensions"): every extension = one header + a `#define CLAP_EXT_XXX "clap/XXX"` string id + `struct clap_host_xxx` / `struct clap_plugin_xxx` (function-pointer vtables) + a per-method thread tag (`[main-thread]`, `[audio-thread]`, `[thread-safe]`). Third-party extensions must namespace the id as `"$REVERSE_URI.$NAME/$REV"` (`conventions/extension-id.md`) — the `/REV` integer is where an id embeds its own version so a breaking revision gets a **new string**, not a mutated struct. Fixed-capacity string buffers (`string-sizes.h`: `CLAP_NAME_SIZE = 256`, `CLAP_PATH_SIZE = 1024`) keep small strings inside the ABI with no cross-boundary realloc. `event-registry.h` is even an in-ABI typed event bus. This maps 1:1 onto ModuKit's service-registry + capability-discovery layer.

### 2.4 Factories
`get_factory` returns `const void*` per string id, same registry idiom one level up:
- `CLAP_PLUGIN_FACTORY_ID = "clap.plugin-factory"` → `clap_plugin_factory_t{ get_plugin_count, get_plugin_descriptor, create_plugin(host, id) }` — enumerate descriptors, then instantiate by id.
- `CLAP_PRESET_DISCOVERY_FACTORY_ID` → index presets without loading the plugin.
- Draft factories (`factory/draft/`): plugin-invalidation (FS-change detection), plugin-state-converter.

## 3. Key capabilities
- A **pure-C, header-only** plugin ABI with a single exported data symbol and version-gated load — nothing to link, trivially consumable from any language that can `dlsym` + read a struct.
- **String-keyed capability negotiation** in both directions (`get_extension` → `void*`), namespaced ids with embedded revision, null = unsupported.
- **Draft/stable quarantine**: 18 draft extensions in `ext/draft/` are excluded from `clap.h` and only pulled via `all.h`, so unfrozen interfaces cannot leak into the stable contract (1.2.0 reorganization).
- **Explicit threading contract** per method (`[main-thread]/[audio-thread]/[thread-safe]`) + a `thread-check` extension the host provides for runtime assertion; `clap-validator` (third-party) auto-tests conformance.
- Cross-ABI identity: `clap_universal_plugin_id_t{ const char *abi; const char *id; }` normalizes CLAP/VST2/VST3/AU ids ("abi"="clap"/"vst3"…, id reverse-URI/UUID/"type:subs:manu"/int).
- Reverse-URI plugin id, keyword feature taxonomy, UTF-8-everywhere.
- Real ecosystem adoption (Bitwig, REAPER, u-he, and a wrapper/VST3↔CLAP adapter layer — see §5).

## 4. Development & current state
- **History**: 1,725 commits since 2014-09-29 (originated as a u-he internal "clap" — the modern ABI restart is the 0.17 line). Rapid pre-1.0 tail 0.17.0 (2021-12-23) → 0.26.0 (2022-05-30), then **1.0.0 (2022-06-03)** — no literal `rc`/`beta` tags in the clone; the 0.17→0.26 run *is* the de-facto release-candidate phase (UNCERTAIN whether RCs were ever separately named).
- **Post-1.0**: 1.1.0 (2022-07), 1.2.0 (2024-01-22), then patch releases to **1.2.10 (2026-07-13)**. `major` never advanced past 1 — the whole 4-year stability story lives in minor/patch + draft→stable promotions.
- **Velocity vs maturity**: 31 commits/12mo vs 1,725 total — a *frozen* surface in steady maintenance, not a stalled one (`origin/next` is 4 commits ahead of `main` at research time, all doc/spec clarifications like "Deactivation can happen regardless of the processing state").
- **Concentration risk**: one maintainer (Bique) authored ~88% of history and the core `entry/plugin/host.h` headers; cross-vendor contributors (u-he, REAPER) shape extensions. Duo/committee sign-off process is **UNCERTAIN** — no governance doc in-repo.
- **Changelog hygiene**: `ChangeLog.md` (354 lines) is per-version and *is* the ABI-break ledger — every stabilization/break is written down (unlike zenoh's empty changelog).
- **Conventions dir is the governance artifact**: `conventions/extension-id.md` codifies naming + the draft→stable ID-stability rule + records the one historical break. This is ModuKit-relevant meta-material, not just code.

## 5. Ecosystem
- **Sibling repos** (free-audio org, referenced in README — not cloned): `clap-wrapper` (CLAP↔VST3/bridge), `clap-host` (reference minimal host), `clap-plugins` (sample plugins), `clap-juce-extension`, `clap-helpers` (C++ RAII façades). Third-party: `clap-validator` (auto conformance suite), `clapdb.tech` (compatibility DB), MIP2, Avendish.
- **Language bindings are generated/thin over the C headers** — exactly ModuKit's doctrine: the `clap` Rust crate (the "nice-plug"/RustAudio ecosystem), iPlug2 (C++), signalsmith-clap-cpp, plus WebAssembly (WCLAP). One C ABI, many façades.
- **Host adoption**: Bitwig Studio (co-author), REAPER (Adrien Prokopowicz's `cockos.reaper_extension` is a documented third-party ext), u-he plugins; a `clap-abi` design discussion culture via the free-audio GitHub org.
- Published as headers consumed by submodule/vendoring; MIT so any DAW/vendor can embed without copyleft obligations.

## 6. Highlights & limitations
**Highlights**
1. **The `get_extension(id) -> void*` pattern is ModuKit's service registry + capability discovery already built, battle-tested, and pure-C** (§2.3). Namespaced versioned string ids + "null means unsupported" is the cheapest honest capability protocol. DIRECT input to Stage 1 and Stage 5.
2. A **stable ABI held at major=1 for 4+ years** with a written rulebook for *how* to evolve without breaking (`conventions/`, changelog-as-ledger). The stability governance is the artifact to study, more than the audio structs.
3. **Draft/stable physical quarantine** (`ext/draft/`, `all.h` vs `clap.h`) — a concrete mechanism for "ship experimental interfaces without poisoning the frozen contract."
4. **Header-only, zero-link, exported-data-symbol** entry — the lowest-friction polyglot surface; any language that can read a struct and `dlsym` can host or be a plugin.
5. Per-method **thread tags** as part of the ABI, plus host-provided runtime `thread-check` — a validated way to encode concurrency contracts in C.

**Limitations**
1. **Audio-domain shapes** (`clap_process`, audio/note ports, latency, params) are irrelevant to ModuKit's generic host — only the *meta* (entry/registry/versioning) transfers; the payload structs do not.
2. **No `size`/`cbSize` field** — stability leans on frozen append-only layout + leading version. That is elegant for a consortium that can *coordinate a freeze* but riskier for a solo-authored evolving kernel; ModuKit may prefer an explicit size to survive careless field reorder (§7, §8).
3. **Person-centric governance**: one dominant maintainer, no in-repo decision record — the "duo/consortium sign-off" is inferred, not documented (UNCERTAIN).
4. Very low commit velocity now — fine for a stable spec, but this is a *frozen* design, not an active R&D source.
5. No `clap-extensions` standalone repo at the given URL; extension review happens across the free-audio org (GitHub PRs/discussions), outside the clone → the governance trail is **not locally verifiable**.

## 7. Historical lessons — how the audio world handled ABI stability

The audio-plugin ABI is the industry's longest, messiest natural experiment in exactly the problem ModuKit is betting its C-ABI exit on. Three generations, readable straight off CLAP's own design choices:

1. **VST2 = "no versioning" chaos.** Steinberg's VST2 shipped a single exported `main`/`VSTPluginMain` function returning a fixed struct with **no version field, no extension registry, and a leaked-but-unlicensed header set**. Every host hard-copied the same `aeffectx.h`; when fields were needed they were appended and everyone *assumed* the same offsets, and there was no negotiation to detect drift. Result: a decade of "works on host A, crashes on host B" and no sanctioned way to extend. **Lesson: an ABI with no version handshake and no capability query is undeployable at scale.** CLAP's `clap_version` + `get_extension` is the direct answer. (VST2 specifics are general industry history, not from this clone — UNCERTAIN on exact mechanics.)

2. **VST3 = COM-like heaviness.** VST3 imported Steinberg's `FUnknown`/`queryInterface`/class-UUID machinery — capability discovery via interface IDs and reference counting, correct but with a large hand-written C++ runtime and a licensing gate. **Lesson: string/UUID-keyed querying is right, but the ceremony (refcounting, factory registries, base-class trees) is the cost when you bolt it on instead of designing it in.** CLAP keeps the query *semantics* and drops the object system: `void*` per string id, refcount-free, ownership spelled out in comments ("owned by the host, valid until `plugin->destroy()`").

3. **CLAP = "one struct + versioned extensions".** The frozen core is *deliberately* small and append-only: entry / host / plugin / factory vtables, a leading `clap_version`, and *everything* else pushed into string-keyed extensions. Stability then has exactly two levers, and both are documented in `conventions/extension-id.md` + the changelog:
   - **Draft folder** — new/unsettled interfaces live in `ext/draft/` and are excluded from `clap.h`; you opt into them via `all.h`. Freezing is a *promotion* (`draft/`→`ext/`), not a version bump.
   - **ID stability** — the rule that landed *late*: "When the extension migrates from draft to stable, its ID **must not change**." Before 1.2.0 the stable ids contained the literal string `draft`, so promoting them was an unavoidable **ABI break** — recorded honestly in the changelog ("We changed the extension ID … which leads to a **break**") and mitigated with dual `_COMPAT` ids so old and new resolve. The written convention exists precisely "so this kind of break won't happen again."

4. **Break history via tags proves the discipline is real.** 1.0.0 (2022) → 1.2.10 (2026): `major` stays 1 across 4 years; the one recorded break (1.2.0 extension-id churn) was contained with compatibility aliases rather than a 2.0. Governance is effectively **freeze the core, version the periphery by name, and when you must break, ship both names and codify why.** (Whether a formal duo/committee ratified each stabilization is UNCERTAIN — no governance file in the clone; strong-maintainer + cross-vendor-review is the local evidence.)

5. **The 1.2.0 init/deinit correction** is itself an ABI-stability lesson in humility: the spec *had* documented "init/deinit called exactly once" as a requirement, hosts couldn't honor it once plugin-wrapping-plugins became real, so CLAP *relaxed the contract and moved the defensive burden to plugin authors* (mutex+counter). Lesson: when a guarantee the ABI makes cannot be kept in practice, the ABI must change and say so — do not let the un-honorable invariant silently rot.

## 8. Value for ModuKit

**Verdict: BORROW-PATTERNS (strong) — a reference textbook for ModuKit's Stage-1/Stage-5 C-ABI governance; never a dependency (it is an audio-specific ABI, zero overlap with our domain).** CLAP is the single most relevant *stability-discipline* profile in this series: it solves the exact "pure-C ABI that must not break across versions while still extending" problem our whitepaper commits to, and it left a written record of how it failed and fixed that once.

### 8.1 Adopt / Adapt / Avoid — C-ABI canon for Stages 1 & 5
| CLAP mechanism | Verdict | How it lands in ModuKit canon |
|---|---|---|
| `get_extension(id) -> void*`, null = unsupported, both peers | **ADOPT** | This *is* the Stage-1 service registry + capability-discovery primitive. One C call, string-keyed, version-embeddable. Add our `size`/version guard (§8.2) on top. |
| Namespaced extension ids `clap.$NAME/$REV` + reverse-URI for third parties | **ADOPT** | Capability/service id grammar: `<scope>.<name>/<rev>`; a breaking revision is a *new string*, never a mutated struct. Mirrors zenoh's versioned `struct_features`. |
| **Exported data symbol** (`clap_entry`) carrying a leading `clap_version`, `dlsym`-then-check | **ADAPT** | ModuKit's plugin entry: export one symbol whose first field is an ABI version; host validates before dereferencing. Keep it a struct (cheaper than a factory function), but see §8.2 on adding size. |
| **Draft/stable quarantine** (`ext/draft/`, `all.h` vs `clap.h`) | **ADOPT** | Directly reusable: keep unfrozen ModuKit interfaces in a `draft/` tree excluded from the stable umbrella header, so we can ship experiments without risking the frozen contract. |
| **Frozen core + append-only layout**, version *only* (no `size`) | **ADAPT** | Copy the "freeze core, version periphery" doctrine, but **do not copy the no-size choice** for a solo-authored kernel — add a struct `size` field so a careless reorder is caught at load, not at crash. (CLAP can skip size because it can *coordinate a freeze*; we may not be able to early on.) |
| Per-method thread tags + host-provided `thread-check` | **ADAPT** | Encode the calling-thread contract in the ABI doc-comment and provide a runtime check service — cheap, validated, fits our capability-discovery model. |
| `clap_universal_plugin_id{abi,id}` cross-format identity | **ADAPT** | For Stage-5 multi-ABI interop (our C ABI vs foreign plugin systems), a `{abi, id}` pair is the minimal normalization. |
| Audio payload structs (process/ports/params/latency) | **AVOID** | Domain-specific; irrelevant to the generic host. Import none of them. |
| Changelog-as-ABI-break-ledger + `conventions/` rulebook | **ADOPT** | Cultural canon: every ABI-affecting change gets a written D-record + a break log with dual-ids when unavoidable. Matches our C0 "executable constraints" ethos. |

### 8.2 Stage matrix
| Stage | Position | Detail |
|---|---|---|
| 1 — plugin kernel | **Adopt (pattern)** | Service registry = CLAP's `get_extension` void*-per-string-id + "null = absent"; lifecycle state machine = CLAP's `init → activate → start_processing → … → deactivate → destroy` with DSO-level `entry.init/deinit` refcount guard (§2.1). Add a struct `size` + version to the entry symbol (CLAP omits size; we want it as a load-time safety net). |
| 2 — transport/SHM | **Reference-only** | Nothing here; SHM lives in iceoryx2/zenoh profiles. CLAP's `event-registry` typed in-ABI event bus is a shape reference only. |
| 3 — registry/services | **Adopt (pattern)** | Namespaced versioned ids (`$REVERSE_URI.$NAME/$REV`) and the draft→stable promotion path are the registry governance model; `universal_plugin_id` informs service identity normalization. |
| 4 — compositor/UI | **Reference-only** | `gui.h`/`webview`/`context-menu` are host-UI negotiation idioms; the "host provides a parent, plugin attaches" asymmetry is a loose analogue to I420/SHM frame intake but not directly portable. |
| 5 — polyglot bindings | **Adopt (doctrine + discipline)** | CLAP proves ModuKit's exact thesis: **header-only pure-C ABI, one exported symbol, generated/thin façades per language** (`clap` Rust crate, iPlug2/RAII helpers, C++/WebAssembly) — never hand-write the low layer per language. Copy the version-gated `dlsym` contract, the string-id capability query, fixed-capacity C strings (`CLAP_NAME_SIZE`/`PATH_SIZE`, no cross-boundary realloc), and the append-only frozen-struct rule. |

### 8.3 License gate
MIT (`Copyright (c) 2021 Alexandre BIQUE`) — **fully permissive, no copyleft leg, no dual-license choice to record** (unlike zenoh's EPL/Apache and iceoryx2's MIT/Apache). Pattern- and idea-derived work is copyright-safe regardless; even literal re-use of the *shape* of the registry/versioning idioms carries no obligation beyond preserving attribution if any header text were copied (which we will not do — patterns only). Gate action: pin `MIT` in the future SBOM/NOTICE with a "CLAP (free-audio), reference-profile patterns, not vendored" note; no relicensing or fork-audit risk observed. Because the license is permissive, the *only* thing worth importing is the stability **governance** — the audio ABI itself stays AVOID.

### 8.4 Not verified here (UNCERTAIN)
- **VST2 / VST3 history** (§7.1-7.2) is general industry background, not derivable from this clone; the precise failure mechanics (offset assumptions, `FUnknown` ceremony) are stated as context, **UNCERTAIN** against primary sources.
- **Multi-vendor "duo/consortium" governance** is inferred from the `free-audio` org plus cross-vendor commit authors (u-he Bique, REAPER contributor); no `GOVERNANCE`/`CONTRIBUTING` file exists in the clone, so the sign-off process is **UNCERTAIN**. Treat "two independent maintainers" as a ModuKit *requirement*, not a CLAP-proven pattern.
- **Sibling repos and bindings** (`clap-wrapper`, `clap-host`, `clap-plugins`, `clap-helpers`, `clap-validator`, `clap-sys`, Delphi/Zig/Ada, iPlug2/JUCE/nice-plug) and **real host adoption** (Bitwig, REAPER, `clapdb.tech`) are README citations only — **not present in this blobless clone**; verify each upstream before citing in the whitepaper.
- **No standalone `clap-extensions` repo was found at the expected URL**; extension incubation is in-repo (`ext/draft/`) and review happens via free-audio GitHub PRs/discussions, which are outside the clone and therefore **not locally verifiable**.
- **`origin/next` (`fb15bc4`, 2026-09-14)** sits ahead of `main`; the un-released content was not deep-read here, so the next minor's stable-interface additions are **UNCERTAIN**.
