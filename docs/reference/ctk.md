# CTK (Common Toolkit) - External Reference Profile

> **Research date:** 2026-10-09 · **Checkout:** `2026.09.02-4-gc7bc5859` (master, 6313 commits, HEAD c7bc5859 2026-09-30; tags `2026.09.02` / `2026.08.06` / `2023.07.13` / `2018-10-29`) · **Upstream:** https://github.com/commontk/CTK
>
> **EXTERNAL REFERENCE** — `.refinfo/CTK` is a git-ignored read-only clone; NOT ModuKit content, not a dependency, never cite as ours. Locally unverifiable claims (GitHub stars, downstream usage) are marked **UNCERTAIN**.

## 1. Portrait

| Attribute | Value | Evidence |
|---|---|---|
| Name / origin | Common Toolkit (CTK); Kitware + medical-imaging community (DKFZ lineage) | `git shortlog -sn HEAD`, `README.md` |
| Scope | DICOM, DICOM Application Hosting, Qt Widgets, Plugin Framework, Command Line Modules | `Libs/`, `Plugins/` |
| Core language | C++ **tightly bound to Qt** 5/6; CMake SuperBuild (VTK/ITK/DCMTK externals) | `CMakeLists.txt`, `CMakeExternals/` |
| Size | 1,872 C++ sources repo-wide; `Libs/PluginFramework` = 250 files / ~23.0k LoC | `find`, `wc -l` |
| First release | 2010-03-09 (first commit); framework designed April-May 2010 | `git log --reverse`, `dc1b3523` |
| Current version | CalVer tags; CMake still declares `0.1.0` — project semver is de-facto frozen | `git describe`, `CMakeLists.txt:168-172` |
| License | Apache-2.0 (`LICENSE`) + `NOTICE` | repo root |
| Target users | Medical-imaging hosts; README carries official 3D Slicer build instructions | `README.md:32` |
| Positioning | The Qt-world OSGi kernel + the domain stacks (DICOM/CLI) built on it | `Libs/PluginFramework/Documentation/CTKPluginFramework.dox` ("an OSGi like modularization framework") |

## 2. Architecture in focus: plugin framework core

Path correction: the brief's `Libs/ctkPluginFramework*` does not exist — the kernel is **`Libs/PluginFramework/`** (flat tree); `ctkPluginFramework` is the class/target prefix, not a directory.

**Lifecycle** — `ctkPlugin.h:87-170`: six bitflag states `UNINSTALLED=0x1, INSTALLED=0x2, RESOLVED=0x4, STARTING=0x8, STOPPING=0x10, ACTIVE=0x20`, a direct OSGi clone. `ctkPluginActivator.h` documents the contract verbatim: `start()`/`stop()` run on one instance, `stop()` only if `start()` succeeded, the framework "must not concurrently call" an activator. `REQUIRE_PLUGIN` manifest header (`ctkPluginConstants.h:196`) gates INSTALLED->RESOLVED; activation policy `eager`/lazy via `Plugin-ActivationPolicy` (`ctkPluginConstants.h:240`).

**Manifest** — generated, not authored, and **embedded in the binary**: `CMake/ctkFunctionGeneratePluginManifest.cmake` emits `Plugin-SymbolicName: ...` etc. from a fixed arg list (`SYMBOLIC_NAME` mandatory with `FATAL_ERROR`, lines 10-28), wraps it in `CMake/plugin_manifest.qrc.in`, and the Qt resource compiles into the shared library. At runtime the archive is read from a resource prefix derived from the library filename (`ctkPluginStorageSQL.cpp:421-426`, `ctkPluginArchiveSQL.cpp:54` reading `META-INF/MANIFEST.MF`). Header names are deliberately de-OSGi'd: `Plugin-*`, not `Bundle-*`. No `plugin.xml` exists anywhere in `Plugins/`.

**Service registry** — imperative, QObject-typed, raw-pointer: `ctkPluginContext::registerService(QStringList clazzes, QObject* service, ctkDictionary props)` (`ctkPluginContext.h:215`). Real usage (`Plugins/org.commontk.log/ctkLogPlugin.cpp:38-47`): `new ctkLogQDebug()` in `start()`, manual `delete` in `stop()`. Lookup is an LDAP engine (`ctkLDAPExpr.cpp`, used at `ctkServices.cpp:201-248`); consumers track via `ctkServiceTracker` templates. `ctkServiceFactory` gives per-bundle instance control. No declarative registration: `grep -rn REGISTER_SERVICE` returns zero.

**Persistence** — unique among C++ OSGi ports: a SQLite plugin database keeps the installed set across runs (`ctkPluginStorageSQL_p.h:43,274`, `ctkPluginArchiveSQL`).

```
ctkPluginFrameworkLauncher  (init/startup; scans plugin dirs; relaunch props ctkPluginFrameworkLauncher.h:81-84)
     |
     v  library file --> resource prefix --> META-INF/MANIFEST.MF (compiled-in .qrc)
ctkPluginManifest --> ctkPluginArchiveSQL --> SQLite plugin DB (survives restart)
     |
     v  Require-Plugin gates resolution
UNINSTALLED <-> INSTALLED -> RESOLVED -> STARTING -> ACTIVE -> STOPPING -+
              activator start()/stop() drive transitions; events: ctkPlugin/Service/FrameworkEvent
                                    |
                       registerService(QStringList, QObject*)   [raw ptr, manual delete]
                                    v
                       ctkServices + ctkLDAPExpr --> ctkServiceTracker (consumers)
```

## 3. Key capabilities

| Area | Capability |
|---|---|
| Plugin model | OSGi 6-state lifecycle, start levels, start/stop options, eager/lazy activation, `Require-Plugin` deps |
| Service registry | Multi-class name registration, LDAP filters, `ctkServiceFactory`, trackers + customizers, service events |
| Persistence | SQL-backed plugin archive/storage (`ctkPluginArchiveSQL`/`ctkPluginStorageSQL`) — installed set survives restart |
| App model | Eclipse-style launcher: `ctkApplicationRunnable`, `ctkDefaultApplicationLauncher`, relaunch properties |
| Compendium plugins | 14 deployable `Plugins/org.commontk.*`: configadmin, eventadmin, eventbus, metatype, log/log4qt, dah.* (DICOM Application Hosting), plugingenerator core+ui |
| Domain stacks | `Libs/DICOM` (SCU/SCP, SQLite DICOM DB, visual browser), `Libs/CommandLineModules` (Slicer execution-model XML, async run), XNAT client, PythonQt scripting |
| Apps | 18 under `Applications/` (`ctkDICOMQueryRetrieve`, `ctkCommandLineModuleExplorer`, `ctkPluginBrowser`, `ctkPluginGenerator`, ...) |

## 4. Development & current state

- 12-month velocity: **241 commits** repo-wide (`git log --since=12.months.1 --oneline | wc -l`); only ~19 touch `Libs/PluginFramework`, none in the recent months — kernel work is finished, not ongoing.
- Contributors: 137 distinct authors all-time (`git shortlog -sn HEAD | wc -l`); last 12 months: J.-C. Fillion-Robin 67, Hans (J.) Johnson 90 combined, Andras Lasso 19, Stefan Dinkelacker 17, Davide Punzo 16 — practical bus factor ~2, medical-imaging motivated.
- Release tags: `2018-10-29` -> `2023.07.13` -> `2026.08.06` -> `2026.09.02`; five years near-silent, then a 2026 reactivation (CI/modernization, dependabot entries).
- Recent maintenance is hygiene, not kernel: clazy slot renames (`943d52ba`), Qt6 compat (`3948e8df`, `07f8616b`), DICOM bugfixes (HEAD `c7bc5859`).
- **Honest health verdict: alive-but-peripheral.** CTK is a maintained legacy platform in active DICOM use, not an evolving plugin kernel. An aging-kernel finding is legitimate here: the framework internals are stable because they are frozen by downstream (Slicer-lineage) compatibility, not because they are polished.

## 5. Ecosystem & adoption

- GitHub stars/forks: **UNCERTAIN** (offline clone, no API access).
- Downstream: 3D Slicer builds on CTK per its own README instructions (`README.md:32`) — the strongest adoption signal; Parsol/other consumers **UNCERTAIN**.
- In-repo proof of use: 14 plugins + 18 applications all built on the same framework (`Plugins/`, `Applications/`).
- Adjacent ecosystem captured in `CMakeExternals/`: DCMTK, VTK, ITK, PythonQt, qRestAPI, qxmlrpc — a medical-imaging stack, not a general plugin ecosystem.

## 6. Highlights & limitations

Highlights:
- The only C++ OSGi port found with **built-in persisted plugin state** (SQLite DB) — real install-across-restart semantics CppMicroServices does not offer at this weight.
- 15 years of edge-case hardening: lazy activation, relaunch, tracker races (`ctkPluginAbstractTracked`), thread-safety contracts documented in header doc-comments.
- Standalone reusable LDAP filter engine (`ctkLDAPExpr.cpp`) and CMake-time manifest generation pipeline (`CMake/ctkFunctionGeneratePluginManifest.cmake`).

Limitations:
- **Qt is load-bearing end-to-end**: `QObject*` services, `QVariant` dictionaries, `QSqlDatabase`, Qt resource paths, moc-based plugins. It cannot host non-Qt processes and cannot be the core of a polyglot system — every language binding would have to link Qt.
- Raw-pointer service lifetime (manual `delete` in `stop()`, `ctkLogPlugin.cpp:47`) — ownership is a convention, not enforced.
- Frozen semver (`0.1.0`) + CalVer tags: no dependency contract to build on.
- Superbuild pulls VTK/ITK/DCMTK — huge transitive surface if consumed wholesale.

## 7. Historical lessons (15+ years)

- Kernel built in one 2010 burst: `80908c72` (framework classes) -> `dc1b3523` ("First functional design", 2010-04-29) -> `01d60beb` (persistence + PluginBrowser, 2010-05-03). **SQL persistence was a day-one decision**, not retrofitted — persistence should be in the Stage-1 schema from the start, not an upgrade.
- The Java vocabulary was deliberately shed: `Bundle-*` headers renamed to `Plugin-*` (`ctkPluginConstants.h:132-240`). ModuKit should make the same naming-ownership decision once, early.
- Path/memory rot: `Libs/ctkPluginFramework` survives only as the `ctk`-prefixed target convention; even the project brief mis-cited it. Verify paths before citing them.
- Formal deprecation machinery: `CTK_DEPRECATED_SINCE` (`f9deffb5`) + `CTK_DISABLE_DEPRECATED_BEFORE` compile-time cutoff (`CMakeLists.txt:174-181`) — deprecated APIs are *hidden by version*, then removed, auditable and reversible. A cheap, proven sunset policy.
- Platform-vendor churn is the dominant maintenance tax: Qt5->Qt6 compat sweeps still landing in 2024-2026 (`3948e8df` QVariant::type; `07f8616b` deprecation rotted into an actual `BUG:`), clazy-driven slot renames (`943d52ba`). Building ON a fast-moving GUI framework's meta-object system transfers that churn forever.
- Five-year near-silence (2018->2023->2026 tags) while downstream (Slicer) kept the code alive: a kernel's liveness is set by its host apps, not its repo cadence.

## 8. Value for ModuKit

**Verdict: REFERENCE-ONLY** (a BORROW-PATTERNS sliver for two mechanics; dependency is off the table — see gate).

Stage-1 comparison: CTK and CppMicroServices (`docs/reference/cppmicroservices.md`) are sibling OSGi ports with the same 6-state machine and LDAP registries, and both embed the manifest into the binary via generated Qt resources / compiled resources. CppMicroServices stays the primary design mirror: no Qt dependency, live semver releases, `ServiceReference`-as-handle lease semantics, and C++ sitting right at ModuKit's FFI boundary. CTK's value is the *delta*: it shows what a Qt-bound variant of the same design buys (SQLite persistence, Eclipse-style application relaunch) and costs (QObject services that cannot cross a C ABI). Read together, the pair sharply defines what ModuKit's kernel must NOT inherit from either: a host-framework object model in the service path.

- **Adopt** (pattern) -> **Stage 1**: persisted-install-set semantics from `ctkPluginStorageSQL` — record which plugins were installed/started so the host restores state across restarts. Specify in the kernel design doc; neither CppMicroServices nor typical Rust plugin crates cover this.
- **Adapt** -> **Stage 5**: the `CTK_DEPRECATED_SINCE` + disable-before-version cutoff as the C ABI deprecation policy for generated polyglot bindings — deprecate-by-version, hide-by-build-flag, delete-later.
- **Avoid**: QObject-typed services and moc-based plugin contracts (would force Qt into every language binding); raw-pointer service lifetime; CalVer-over-frozen-semver versioning.
- **Not applicable**: Stage 2 (zero-copy IPC), Stage 3 (Wayland/X11 composition), Stage 4 (WebRTC) — nothing in CTK touches these; `Libs/Visualization` is VTK widget glue.

**License gate:** Apache-2.0 with patent grant — permissive, clears any ModuKit license outcome. The real gate is not the license but the **Qt runtime LGPL obligations CTK drags in**: consuming CTK as code would pull Qt + its superbuild externals into every bound language, which alone keeps the verdict REFERENCE-ONLY.
