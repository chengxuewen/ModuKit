# CppMicroServices — Reference Profile

> **Research date:** 2026-10-09 · **Checkout:** `v3.8.10-35-g571917d7` (HEAD 571917d7, 2026-10-05) · **Upstream:** https://github.com/CppMicroServices/CppMicroServices
>
> **EXTERNAL REFERENCE — read-only clone under `.refinfo/`. Not ModuKit code, not a dependency, do not edit or cite as ours.** Profiled as the canonical C++ implementation of the OSGi bundle-lifecycle + service-registry pattern named in Whitepaper §8 Stage 1.

## 1. Portrait

CppMicroServices ("C++ Micro Services") is a 14-year-old, production-grade port of the OSGi core framework model to native C++: bundles as shared libraries with embedded metadata, a six-state lifecycle machine, and a dynamic LDAP-queryable service registry. It is the closest thing the C++ world has to a reference OSGi kernel, and the README positions it exactly where ModuKit Stage 1 wants to be: "collection of components for building modular and dynamic service-oriented applications... based on OSGi, but tailored to support native cross-platform solutions."

### Architecture in focus: lifecycle + registry + manifest + ABI boundary

Code lives in two trees: `framework/` (core kernel, ~31k LoC in `framework/src` + `framework/include`) and `compendium/` (OSGi-spec add-ons, ~56k LoC).

**Bundle lifecycle state machine** — `framework/include/cppmicroservices/Bundle.h`:

```
            install                       resolve (lazy dlopen)            start
  (archive) ────────▶ INSTALLED ────────────────▶ RESOLVED ──────────▶ STARTING ──┐
                     ▲     │  (framework may            │                        ▼
                     │     │   resolve eagerly)          │                     ACTIVE
   BUNDLE_INSTALLED  │     │ uninstall                   │ stop          ▲        │
   BUNDLE_RESOLVED   │     ▼                             ▼               │        │
   BUNDLE_STARTED    │  UNINSTALLED ◀──────────── STOPPING ◀─────────────┼────────┘
   BUNDLE_STOPPED    │   (terminal: "can not be       (BUNDLE_STOPPED)   │
   BUNDLE_UNINSTALLED│    set to another state")              BUNDLE_LAZY_ACTIVATION fires
   BUNDLE_LAZY_ACT.  │                                        when a service on a lazy bundle is got
```

States are bitflag `uint32_t` (`STATE_UNINSTALLED = 0x1` … `STATE_ACTIVE`); every transition fires a `BundleEvent` (`framework/include/cppmicroservices/BundleEvent.h`: `BUNDLE_INSTALLED/STARTED/STOPPED/UNINSTALLED/RESOLVED/LAZY_ACTIVATION`). Transition logic: `framework/src/bundle/BundlePrivate.cpp`; install path: `framework/src/util/FrameworkPrivate.cpp`. `Bundle::Start(options)` supports `START_TRANSIENT` (do not persist autostart) and `START_ACTIVATION_POLICY` (respect the `ACTIVATION_LAZY` manifest policy, `Constants.h:200`) — i.e. lazy activation is first-class: a lazy bundle resolves its library only when someone touches its services.

**Service registry** — `framework/src/service/ServiceRegistry.{h,cpp}` (~400 LoC). Per-class ordered vector of registrations plus `MapBundleServices = unordered_map<BundlePrivate*, vector<ServiceRegistrationBase>>` (bundle ↔ registration ownership), a 128-entry LDAP filter cache (`FILTER_CACHE_MAX_SIZE`, `ServiceRegistry.h:200`), and ranking-based ordering (`UpdateServiceRegistrationOrder`). API surface (`framework/include/cppmicroservices/BundleContext.h`, 1160 lines):

- `RegisterService<I1,...>(shared_ptr<Impl>, props)` — variadic interface list; impls become `InterfaceMap` (string class-name → `shared_ptr<void>`).
- `RegisterService(shared_ptr<ServiceFactory>, ...)` — `ServiceFactory::GetService/UngetService` gives per-bundling-bundle instance control (OSGi factory pattern, `ServiceFactory.h`).
- `GetServiceReference(s)` with `LDAPFilter`/`LDAPProp` (`LDAPFilter.h`, `LDAPProp.h`) over the property bag (`AnyMap`); well-known keys `service.id`, `service.ranking`, `service.vendor` (`Constants.h:356-404`).
- `ServiceReferenceBase` (`ServiceReferenceBase.h`) is a **handle, not the instance**: value-semantic, `explicit operator bool()` validity check, `GetProperty(key) -> Any`; the actual object arrives only via `BundleContext::GetService(ref)`; `ServiceObjects.cpp` + `ServiceUseData` track per-bundle use counts, and `UngetService` balances each `GetService` — the registry leases instances, it never hands out raw ones.
- Listener management is RAII-token based (`ListenerToken.cpp`, `ServiceListenerHook`/`EventListenerHook`/`BundleEventHook`/`BundleFindHook` in `framework/src/service/` and `framework/src/bundle/`) — the OSGi Framework Hooks spec, used to intercept/visibly-filter events and service finds per bundle.

**Manifest** — `resources/manifest.json`, compiled *into* the shared library by the CMake resource pipeline (`cmake/usMacroCreateBundle.cmake`, `cmake/usFunctionEmbedResources.cmake`, resource compiler under `tools/rc/`; JSON Schema in `schemas/manifest_schema.json`). Keys: `bundle.symbolic_name` (identifier-only pattern, no dots), `bundle.activator: true`, `bundle.version`, `bundle.description`, `bundle.vendor`, `bundle.activation_policy`, plus `scr.components[]` (`immediate`, `enabled`, `implementation-class`, `inject-references`) for Declarative Services. At runtime the manifest is read from the binary as an embedded resource (`framework/src/bundle/BundleArchive.cpp`, `BundleManifest.cpp`, `BundleResourceContainer.cpp`) — bundles are self-describing single files, no sidecar metadata.

**C++ ABI boundary** — each bundle is a real `.so`/`.dll`/`.dylib` loaded by `dlopen`/`LoadLibrary` (`framework/src/util/SharedLibrary.cpp`, `framework/src/bundle/BundleUtils.cpp:52` dlsym wrapper). The activation contract is deliberately thin: the bundle's `.cpp` writes `CPPMICROSERVICES_INITIALIZE_BUNDLE` (`BundleInitialization.h:70`), which emits `extern "C"` symbols `<name>_GetContext` / `<name>_SetContext` (plus a static activator-creation entry) against the `US_BUNDLE_NAME` define the CMake macro injects. The framework dlsyms those C symbols, injects the `BundleContext` via a shared_ptr handshake, then constructs the C++ `BundleActivator` subclass across the boundary and calls `Start`/`Stop`. Everything else (interface maps, `Any`, vtables) crosses the boundary implicitly — which is exactly the fragility ModuKit's C-ABI-only exit is designed to avoid.

### 2. Key capabilities

- **Core (framework/):** bundle install/resolve/start/stop/uninstall with event stream; lazy activation; per-bundle `BundleContext` isolation; service registry with LDAP filters, ranking, factories, per-bundle use counting; framework hooks; embedded resources (archives can also read files *inside* the bundle library); `Any`/`AnyMap` property system.
- **Compendium:** `DeclarativeServices` (OSGi DS R7-style component runtime: XML/JSON component descriptions, bind/unbind lifecycle driven by registry events), `ServiceComponent` (SCR DTOs/superset helpers), `ConfigurationAdmin` (per-PID config store + async `CM`), `LogService`/`LogServiceImpl`, `AsyncWorkService`, `EM` (event admin).
- **Tooling:** CMake macros (`usMacroCreateBundle`, `US_ADD_SUBDIRECTORIES`), resource compiler (`tools/rc`), JSON-schema validator tool, Conan packaging (a `v3.8.11-conan-rc1` tag exists), sanitizers (tsan/ubsan/valgrind suppression files at root), Coverity, code coverage via Codecov.

### 3. Development & current state

- **Velocity (12 mo):** 61 commits, 10 unique authors (2025-10-01→2026-10-09); ~107 commits across calendar 2025. Steady maintenance cadence, small core team; releases v3.8.10 (2026-02-27) → v3.8.13 (2026-08-25), roughly quarterly patch releases. CHANGELOG is Keep-a-Changelog + semver, PR-referenced per entry.
- **Dominant contributor base:** MathWorks (bweed-mathworks, tcormackMW, Jeff DiClemente, James Knee...) — copyright assigns 2015–2023 The MathWorks, Inc. alongside founder Sascha Zelzer.
- **Language/toolchain:** C++17 required (`CMakeLists.txt:49`, `CMAKE_CXX_STANDARD_REQUIRED ON`), CMake ≥ 3.17, GCC/Clang/MSVC/MinGW/Xcode CI matrix, Windows CI actively patched (HEAD commit "update windows runner").
- **Tags:** v1.0.0 → v3.8.13 (35 tags; v2 in 2015, v3.0 in 2017).
- **CI health:** dual-OS GH Actions, Coverity, codecov, RTD docs (Sphinx + Breathe).

### 4. Ecosystem

- Used in medical/scientific imaging (founder lineage: DKFZ Heidelberg; successor of the CTK-style `us` micro-services layer; `tools/change_namespace` marks the `us::` → `cppmicroservices::` rename).
- Docs are unusually complete for a C++ infra project: Sphinx manual with OSGi mapping, doxygen API, tutorials (`framework/doc`, `compendium/*/doc`).
- Packaging: Conan recipe in progress (tag), vcpkg community port exists.
- No plugin marketplace beyond the framework itself; ecosystem value is the OSGi spec lineage (DS/CM/Log semantics) rather than third-party bundles.

### 5. Highlights & limitations

**Highlights**

1. The lifecycle+registry semantics are the most battle-tested C++ expression of OSGi core: lazy activation, lease-based service access (`GetService`/`UngetService` pairing, per-use counting), ranking + LDAP filtering, hooks for visibility control — all spec-faithful and production-hardened (recent commits are race fixes: PR 1230, 1238, 1248, 1264).
2. Self-contained binary bundles (manifest + resources embedded in the `.so`) — one artifact, atomic install, no directory-layout assumptions.
3. `ServiceReferenceBase` as value-semantic, explicitly-invalidatable handle — a clean model for "lookup without instantiation".
4. Listener tokens instead of register/unregister pairs (3.1 deprecation of `RemoveBundleListener`/`RemoveFrameworkListener`) — RAII auto-unsubscribe kills an entire bug class.
5. Apache-2.0, quarterly releases, sanitizer+CI discipline.

**Limitations**

1. The ABI boundary is only `extern "C"` at activation; after that, C++ objects (`shared_ptr<void>`, `Any`, vtable activators) cross the `.so` seam, so all bundles must share compiler/stdlib/`Any` version — no stable-ABI story.
2. `bundle.symbolic_name` rejects dotted names (schema pattern `^[a-zA-Z_][a-zA-Z0-9_]*$`) — a deviation from OSGi for C-identifier convenience.
3. Heavy build-time coupling: bundles are expected to build inside the project's CMake macro system; non-CMake embedders reimplement the resource-compiler invocation.
4. No remote/distributed story (in-process only), no capability/security model — OSGi permission concepts were never ported.
5. Declarative Services is where the complexity and bugs live (changelog is ~50% DS race fixes) — the declarative layer is much harder to get right than the kernel.

### 6. Historical lessons

- **OSGi translation decisions:** class names mirror Java 1:1 (`Bundle`, `BundleContext`, `ServiceRegistration`, `ServiceFactory`); the deliberate divergences are instructive — listeners → RAII tokens, Java's checked casts → `Any`+class-name strings, dotted symbolic names → C identifiers, sidecar JAR directory → single self-describing library, no classloaders so "resolved == dlopenable".
- **Component-model shift:** the DS/SCR layer arrived late (2019, v3.3: `compendium/DeclarativeServices` + `ServiceComponent` added 2019-10-24; `ConfigurationAdmin` 2020) and immediately became the dominant bug surface — a decade of clean kernel + years of race-fix churn in the declarative tier. Lesson: the kernel semantics are the stable part; the dependency-injection tier is not.
- **Deprecation hygiene:** CHANGELOG carries per-release `Added/Changed/Deprecated/Removed` sections; 3.1 deprecated manual listener-removal APIs, 3.0 deprecated out-param `GetPropertyKeys` — replaced by tokens and return-by-value respectively.
- **Stewardship lineage:** DKFZ research project (2012, Sascha Zelzer) → MathWorks takeover (2015) → org-ified repo under its own GitHub org. Corporate stewardship kept 14 years of quarterly releases but concentrated the bus-factor.

### 7. Value for ModuKit

**Verdict: primary design mirror for Stage 1 — port the semantics, not the code.**

| ModuKit concern | What to take | Source |
|---|---|---|
| Lifecycle | Exactly the 6-state machine + event set (`Bundle.h` state docs enumerate per-transition event ordering — write the Rust enum from that spec); lazy activation as manifest policy + explicit `START_TRANSIENT`/policy start options | `framework/include/cppmicroservices/Bundle.h`, `BundleEvent.h` |
| Registry | Reference≠instance handle model; lease semantics (`GetService`/`UngetService` + per-consumer use counting); `service.id`/`service.ranking` + string property bag + filter queries; ordered-by-(ranking,id) lookup | `framework/src/service/ServiceRegistry.h`, `ServiceObjects.cpp`, `Constants.h` |
| Events | Token-based auto-unsubscribing listeners from day one (skip their 3.0→3.1 mistake cycle) | `ListenerToken.cpp`, CHANGELOG v3.1 Deprecated |
| Metadata | Embedded manifest in the artifact (one file = one bundle); JSON schema for it checked into repo | `schemas/manifest_schema.json`, `cmake/usFunctionEmbedResources.cmake` |
| Hooks | Registry/event **hooks as an interception layer** — Rust kernel can reuse this seam later for capability checks the OSGi port lacks | `framework/src/service/ServiceHooks.h`, `framework/src/bundle/BundleHooks.cpp` |
| ABI | Only the activation seam pattern (per-plugin `extern "C"` init/get-context symbols resolved via dlsym) — matches ModuKit's C-ABI-only exit; replace everything past it | `BundleInitialization.h:70`, `BundleUtils.cpp` |

**Avoid:** C++ objects crossing the bundle boundary (their `InterfaceMap`/`Any`/`shared_ptr<void>` casts are the exact anti-pattern ModuKit's whitepaper rejects); build-system-locked bundling (CMake macros as the only path); their DS tier for Stage 1 (defer any declarative DI until the kernel is proven — the changelog shows why).

**License gate:** Apache-2.0 — permissive, compatible with ModuKit's TBD license for study and semantics-porting; attribution preserved via this profile. No GPL contamination risk. **Green.**
