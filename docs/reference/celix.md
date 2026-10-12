# Apache Celix — Reference Profile

> **Research date:** 2026-10-12 · **Checkout:** `master` 0fff19097 (2026-09-29) · **Upstream:** https://github.com/apache/celix
>
> **Local path:** `.refinfo/celix/` (git-ignored reference checkout; never edited, never cited as ours)
> **C1 note:** this profile is an English artifact per project convention.

---

## 1. What it is

Apache Celix is the Apache Software Foundation's **OSGi-style framework for C and C++** —
the canonical "OSGi on the C ABI" implementation. It is the closest open-source structural
cousin to ModuKit's problem domain: an in-process plugin framework with a service registry,
bundle lifecycle, declarative dependencies, and a remote-services layer — built entirely on
`dlopen`/`dlsym` and plain C structs.

**Provenance / trust**: Apache top-level project (ASF governance, Apache-2.0), active
upstream (checkout is from 2026-09-29), large real-world user base in automotive/embedded.

## 2. Measured shape (this checkout)

| Metric | Value | Notes |
|---|---|---|
| Total C sources | 221 `.c` files (15 test files) | `find`-measured |
| Framework library | ~15.2k LoC (`libs/framework/src`) | 45+ translation units |
| Utils library | ~7.7k LoC (`libs/utils`) | threads, maps, lists, properties, filters |
| dfi (dynamic function interface) | ~3.4k LoC (`libs/dfi`) | runtime type/JSON-RPC machinery |
| framework.c alone | 2,723 lines | the lifecycle core |
| service_registry.c | 1,191 lines | + service_reference/registration/tracker |
| dm_component_impl.c | 1,228 lines | Declarative Services core |
| C++ surface | `include/celix/` (BundleActivator.h, Trackers.h, UseServiceBuilder.h, dm/) | RAII builders over the C API |
| Public C API | `libs/framework/include/*.h`, `CELIX_FRAMEWORK_EXPORT` | 30+ public headers |
| Deprecated API | `include_deprecated/` (8+ headers) | old `bundle_pt`-style API still shipped |
| Bundles shipped | remote_services (RSA-dfi, topology_manager, discovery_etcd/configured/zeroconf), event_admin (+MQTT remote provider), logging (log_admin/log_service/sinks/writers), http_admin, shell, cxx_remote_services | the "Compendium" |
| Version | `CELIX_FRAMEWORK_MANIFEST_VERSION "2.0.0"` | manifest schema version constant |

## 3. Architecture map (measured)

```
libs/framework   lifecycle + registry + DM + events (15.2k LoC, 45+ TUs)
libs/utils       threads / maps / lists / properties / filters (7.7k)
libs/dfi         runtime type descriptors -> JSON-RPC (3.4k)   [RSA's backbone]
libs/rcm         resource change monitoring (440)
libs/promises / pushstreams   C++ header-only async primitives (headers, no src LoC above)
libs/etcdlib     standalone etcd client (722)
bundles/
  remote_services/  RSA (dfi + shm_v2) + topology manager + 3 discoveries
  event_admin/      EventAdmin + MQTT remote provider
  logging/          log_admin + log_service + sinks + writers
  shell/            C-shell + remote shell
  http_admin, components_ready_check, cxx_remote_services
```

**Framework internals (src/ inventory):** framework.c (2.7k), service_registry.c (1.2k),
service_reference/registration/tracker trio, bundle/bundle_context/bundle_cache,
celix_bundle_archive (zip-format bundle archive), celix_bundle_manifest,
celix_libloader (dlopen wrapper), dm_component_impl.c + dm_service_dependency (the DS
engine), celix_scheduled_event + celix_dm_event (framework event queue), celix_condition.

## 4. Key mechanisms (with anchors)

### 4.1 Bundle = zip archive + properties manifest
A Celix bundle is a **zip** with `META-INF/MANIFEST.MD`-style properties inside
(`celix_bundle_manifest.c` loads it via `celix_properties_load` — properties are a
flat key/value type with typed values and version support; symbolic-name charset is
`-_:.` per `CELIX_BUNDLE_SYMBOLIC_NAME_ALLOWED_SPECIAL_CHARS`). The archive is unpacked
into a **bundle cache** directory at install (`celix_bundle_cache.c`), then modules load
native libs from there. Manifest version constant `2.0.0` shows a versioned manifest
schema discipline.

**vs ModuKit**: our D28 accepted **directory AND archive** forms on one pipeline with
**no unpacking** (staging + atomic rename; the archive is read in place through the
ResourceService window). Celix unpacks to a cache — a third location that can drift
from the installed artifact; our cross-check (D7) binds install-time hashes instead.
Celix's flat properties manifest vs our TOML (D31): theirs predates modern options and
is weaker at nesting (our `[resources]`/config sections need nesting).

### 4.2 Service model: registry + use/callback discipline
The C API is **option-struct based**: `celix_bundleContext_registerServiceWithOptions`
(+Async variant, `celix_bundle_context.h:220/231`), and **use-based access**:
`celix_bundleContext_useServiceWithOptions` (`:568`) — services are used inside a
callback while the framework holds them; the old `useService` trio is **formally
deprecated** in favor of builder APIs. C++ side wraps this in RAII builders
(`UseServiceBuilder.h:170` calls the C options function directly).

**vs ModuKit**: the callback/use discipline is the same contract family as our D6
synchronous-boundary + event completion, and their options-struct API proves the C ABI
can stay narrow. But Celix's registry is **in-process only** (their own docs are
explicit that remote is an add-on), matching our D4 assessment; their tracker/factory
surface is what we rejected at `docs/modules/02-service-model.md:64-68` — and the
1.2k-line service_registry.c shows the price of the full tracker machinery we skip.

### 4.3 Declarative Services (dm_*) — the 1.2k-line DS engine
`dm_component_impl.c` (1,228 lines) + `dm_service_dependency` implement OSGi-DS-style
components: declared service dependencies with required/optional, cardinality,
callbacks, and a component state machine managed by the dependency manager. Public API
~700 lines across `celix_dm_*.h`. This is where Celix keeps its complexity: their own
docs position DM as the recommended component model over raw bundle activators.

**vs ModuKit**: our D4 chose static bindings + dynamic events with six invariants and
**no DS component layer** (CppMS changelog analysis: ~50% of DS churn is race fixes).
Celix's 1.2k-line DS core + separate dependency-manager event queue
(`celix_dm_event.h`) is a live measurement of what we chose not to build — and their
event-queue decoupling (dm events processed on the framework event loop) is evidence
for our D3 start-level ordering + wait|failfast policies being the right *simpler*
shape.

### 4.4 Remote services = three pluggable pillars
RSA splits into **topology_manager** (policy), **remote_service_admin_dfi** (transport +
dfi-generated marshalling), **discovery_{etcd,configured,zeroconf}** (endpoint discovery).
dfi reads **runtime type descriptors** (dyn_type/dyn_interface/dyn_function +
json_rpc.c) to serialize C structs over JSON-RPC — dynamic introspection machinery so
that services can be remoted **without IDL** at the cost of a 3.4k-line runtime type
system.

**vs ModuKit**: this is the exact contrast our D13 protoc-plugin decision anticipated.
Celix pays for "no IDL" with a runtime descriptor interpreter; we pay at build time
(codegen from `.proto`, D13/D12) and get a frozen C ABI for free. Their topology
manager / RSA / discovery trichotomy validates our D11 narrow-seam + per-plane slots
shape (policy / transport / discovery separable).

### 4.5 Event admin & logging as bundles, not kernel
EventAdmin (+MQTT remote provider) and logging (log_admin/log_service/sinks/writers)
are **bundles on top of the framework**, not kernel services — the OSGi Compendium
pattern. ModuKit agrees on logging (D19 sinks-as-subscribers) but went further: our
topic bus is a kernel-level handle-table object (D10) rather than a bundle.

**vs ModuKit**: Celix event_admin is essentially a topic bus implemented as an optional
bundle. Our D10 made it a first-class kernel object with latest-value semantics and
instrumentation (D21 counters on the same doorways); Celix's choice shows the
framework-only version is buildable, but their event admin has no QoS/instrumentation
story — ours does by construction (D10/D21).

### 4.6 Framework event loop + scheduled events
`celix_scheduled_event.c` + the framework event queue (celix_dm_event) run the
framework's async work on an internal event loop with thread primitives from
`libs/utils` (celix_thread). Services are used synchronously inside callbacks; async
variants exist for registration/use (`...WithOptionsAsync`).

**vs ModuKit**: matches our D6 (sync boundary + job tokens + event completion) and
D20 (timers-as-jobs) — Celix independently validates the "framework loop owns async,
user code stays sync" model. Their scheduled events ≈ our timer→JobToken.

## 5. Strengths (measured)

1. **C-ABI-native OSGi, production-grade**: the dlopen/dlsym seam + option-struct APIs
   prove a narrow C exit is buildable and stable — direct evidence for our D2/D13
   approach.
2. **Versioned manifest schema discipline** (`2.0.0` constant) — same instinct as our
   D7 cross-check + versioned ABI.
3. **C++ RAII layer over the C API** (builders/trackers) — validates our SDK layering
   (thick author face over thin C exit).
4. **Compendium as bundles** — logging/event_admin/shell ship as bundles, keeping the
   kernel thin.
5. **Error injection test suites** (`libs/framework/error_injector`,
   `ManifestErrorInjectionTestSuite.cc`) — fail-path testing infrastructure we should
   note for our liar-fixture family (D7).
6. **Apache governance + docs** (`documents/framework.md`, `components.md`,
   `patterns.md`, `properties_and_filter.md`).

## 6. Weaknesses (measured)

1. **Unpacking bundle cache**: install copies/unpacks to a cache dir — a third copy of
   the artifact; drift risk that our D28 staging+rename avoids.
2. **Flat properties manifest**: key/value with typed values but no real nesting — our
   resources/config sections (D31/D29) need nesting TOML gives us.
3. **1.2k-line DS engine + DM event queue**: the component layer is where the
   complexity concentrates — exactly the layer we rejected (D4).
4. **3.4k-line dfi runtime type interpreter**: the "no IDL" price — we pay codegen
   instead (D13) and keep a frozen ABI.
5. **In-process framework, remote as add-on bundles**: three discovery backends and a
   topology manager = integration surface we deliberately replaced with the D11
   transport seam + zenoh bridge.
6. **Legacy API layering**: `include_deprecated/` still ships the old API — an
   archaeological tax on the codebase; ours starts clean.

## 7. ModuKit verdict

**Category: sibling prior art (C/C++ OSGi) — the closest open-source analogue to
ModuKit's kernel; a validation-rich, cautionary-tale-rich reference.**

- **Confirms**: narrow C ABI seam (option structs, D2/D13); sync-boundary + framework
  event loop (D6/D20); manifest versioning (D7); compendium-as-bundles thinness (D19);
  error-injection testing (D7 liar family); remote = separable seams (D11).
- **Rejects for us**: unpacking bundle cache (D28 staging+rename instead); flat
  manifest (D31 TOML); DS component engine (D4 static+events); dfi runtime-type
  interpreter (D13 codegen); three-discovery RSA zoo (D11 seam + zenoh).
- **Borrow-worthy**: error-injector test pattern for our fixture families; the
  `celix_condition.h` mechanism (a lease-like readiness primitive) as prior art when
  our D3 `wait` policy meets real components; C++ builder layering proof for our SDK.

## 8. Cross-references

- `docs/reference/cppmicroservices.md` — the other C++ OSGi (in-process only, embedded
  resources); Celix vs CppMS = "with remote machinery" vs "leaner, no remote".
- `docs/reference/00-overview.md` — decision matrix; add Celix row: C-ABI-native OSGi
  with RSA/discovery/dfi machinery.
- ModuKit decisions exercised: D2 (placement), D4 (service model), D6/D20 (sync +
  timers), D7 (manifest/cross-check), D10 (bus), D11 (transport seam), D13 (codegen vs
  dfi), D19 (logging), D28 (bundle install), D31 (TOML).
