# 05 — Manifest and Deployment

Fields and semantics are adjudicated (D2/D3/D4); the file format is **TOML** per D7 (single
format — JSON rejected as policy debt). Where each field physically lives, and how file-vs-
binary lying is policed: §6.

## 1. Identity and entry

| Field | Meaning | Notes |
|---|---|---|
| `id` | stable plugin identity | reverse-DNS style per CLAP precedent (`clap-plugin-abi.md:58`); survives renames |
| `version` | plugin semver | independent of ABI version (04 §3) |
| `entry.kind` | `dl` / `exe` / `wasm` | the placement floor declared by the plugin |
| `entry.artifact` | path to the `.so` (or runner payload) | `dl`/`exe` share the artifact (01 §2) |
| `entry.runner` | optional: interpreter for script plugins | the language axis (01 §1) — e.g. `node`, `quickjs`, `lua` |

Deployment may override `entry.kind` upward (`dl -> exe -> wasm`) — never downward without a
trust review. Overriding is config; the plugin never rebuilds to change placement (D2).

## 2. Placement and supervision fields

```
placement:  floor = dl              # host may raise (01 §5, 07 in architecture)
start_level: 30                      # boot tiers (03 §3)
start_timeout: 5s
host_id: cockpit                     # host instance name (D36, deploy-level, not per-plugin);
                                     #   default = machine hostname; collision on the carrier
                                     #   (cloned machines) = startup warning + forced explicit
                                     #   distinct ids
stop_timeout: 2s                     # shutdown budget per plugin (D34): on expiry,
                                     #   exe = SIGKILL the process, wasm = cancel execution
                                     #   (engine epoch), dl = mark FAILED and move on
                                     #   (in-process code cannot be killed; host never blocks
                                     #   the shutdown sequence past the budget)
restart: { policy: on-failure, max_retries: 5, backoff: 1s..30s, cooldown: 5m }
```

**Capability freeze (D34)**: the capability set is fixed at install time and re-evaluated on
every boot - there is no runtime revocation. Restart IS the revocation mechanism (D8
semantics); anything finer waits for the multi-vendor trigger.

**Duplicate identity (D34)**: installing a second bundle with the same id (any version) is
rejected at staging - identity is unique per host, not per version. Version supersession is
an UPDATE (D8), never a parallel install.

`restart` is honored by the supervisor; under `dl` the host is already dead when the plugin
"crashes" (01 §4), so these fields are inert for `dl` — recorded in the state machine docs,
not hidden.

## 3. Dependency and capability declarations

```
requires: [
  { service = "com.yourco.hmi.CanBus",    required = true,  policy = wait(10s) },
  { service = "com.yourco.hmi.Telemetry", required = false, policy = fallback, on_down = block },
]
provides: [ { service = "com.yourco.hmi.NavEngine", ranking = 100, parallel_ok = false } ]
aliases = { "CANBus" = "com.yourco.hmi.CanBus" }   # bundle-scoped sugar (D9), never crosses wire/ABI
capabilities: [ shm, gpu-share ]     # checked at resolve against host discovery
topics = {
  publish   = [ { topic = "com.yourco.hmi.Gear",  qos = "latest" } ],
  subscribe = [ { topic = "com.yourco.hmi.Speed", slow = "drop" } ],
}
export = []                          # bridged planes see NOTHING unless listed (D11)
```

- `requires` is validated at `RESOLVING` (03 §1) for `required = true` — missing = deterministic
  boot failure before any plugin code runs; the boot graph can say "who waits on whom".
- Service strings are canonical reverse-DNS ids (D9, 02 §1); `aliases` map short names to
  canonical ids within this bundle's parse scope only — resolution happens at manifest read
  time, everything downstream (cross-check, wire, registry) sees canonical strings.
- `requires`/`provides` are the **declared ledger** feeding D18's declared-vs-observed
  analysis (modules/10 §6): dead declared deps and undeclared runtime use both surface here.
- `capabilities` (whitepaper §3 principle 3: declarations beat assumptions), two namespaces
  per D26:
  - `org.modukit.*` — **kernel-owned, hard-checked at resolve** (missing => failure).
    Initial vocabulary, one word per already-adjudicated mechanism: `shm-pool` (D14),
    `gpu-share`, `dma-buf` (D15 trigger lane), `input-inject` (compositor track),
    `precise-clock` (D20 domains), `multi-host` (D11 carriers), `sandbox` (wasm executor),
    `rt-sched` (deployment `resources{}`/`rt_policy` family). The list lives in the ledger
    (D13), never as string literals in code.
  - `com.vendor.*` — **self-registration**: the kernel never interprets these; they are
    notes for policy consumers (bridge export gates D11, quota engine D22, trust routing
    D1). A soft word grants nothing by itself — a note, not a key, read by the policy
    layer that reads them.
- `provides.ranking` is reserved storage only in year 1 (02 §1, invariant 4).
- `provides.parallel_ok` is an author semantics claim (D8): cross-checked between tiers like
  every other descriptor field, inert until Stage-2 multi-instance routing exists. Nobody may
  set it to `true` speculatively and be surprised when two versions run — or when they don't.

## 4. Bundles: logical unit, open physical form, verification ladder (D28)

The kernel defines **what a bundle is, not what it looks like**:

```
logical bundle = manifest (this doc) + artifact(s) + ledger refs + content-hash
install:       staging dir -> cross-check (D7 three tiers + hash integrity)
               -> atomic rename into INSTALLED (03 §1 semantics)
physical forms accepted: directory (dev rigs) | archive (shipping)
               -- both expand into the SAME pipeline; zero resolve-logic fork
```

Verification ladder, each rung owned by a trigger, not by taste:

| Rung | Defends against | Arrives |
|---|---|---|
| content-hash + ledger cross-check | corruption, drift | **year 1** (tens of lines on the resolve path) |
| signature chain | malice | trigger: first untrusted-tier plugin (joins D1/D7 trust axis) |
| repository/OTA semantics | fleet logistics | trigger: production OTA need — split then: platform updater (A/B, swupdi class — general knowledge) vs `modukit-cli` packaging |

Explicitly deferred with owners: archive format choice (decided at CLI packaging day),
rollback policy (D16 neighborhood), delta distribution. `min_host` stays the forward-compat
anchor.

## 5. Resources: one bundle, two doors, pass-by-trust (D30)

Resources are non-code assets a plugin ships: icons, layouts, i18n catalogs, model weights,
calibration curves. They are bundle members per D28 - no second packaging layer, no
embedded-array compilation (CppMS-style resource compilers couple every plugin language to
the build system; Qt rcc external mode works but its container would duplicate the archive
we already have).

- **Tree organization (adjudication 28, D31)**: the directory tree under `resources/` IS the route
  table - runtime path = tree path, no prefix/alias layer (a `.qrc`-style remap duplicates the
  declaration we already verify; renaming is `git mv`, not a second source of truth). The
  only hard rule: `resources/` is the root and the window's single allowed path prefix.
- **Declaration**: the verification checklist lives in the manifest's `[resources]` section
  (path, size, sha256) - same file the cross-check already reads, covered by the same
  integrity envelope, tool-fillable (`modukit resource scan`, CLI-day trigger) without
  touching hand-written annotations (toml_edit preserves comments). A separate index file
  (`resources.idx`) was evaluated and rejected: a second truth source that can drift from the
  manifest (D28 forbids half-new/half-old states); an archive central-directory-as-index was
  rejected because directory-form bundles (the year-1 primary) have none - the two D28 forms
  must fork nothing. The D7 cross-check verifies presence + hash at install; undeclared
  resources are invisible across the boundary (year-1 white-list semantics). Lying fixtures
  join the D7 liar family.
- **Reading**: one semantic answer for all three placements (D2). The kernel mounts a
  ResourceService (ledger-native, introspect-trio shape: list / stat / open):
  - **Fast lane (pass-by-trust)**: in-process `dl` plugins at trusted tier receive a plain
    fd at activation (same process, zero boundary cost) and read natively.
  - **Window lane (everyone else)**: `exe` (isolation semantics must stay intact; fds are
    irrevocable and would bypass the accounting gate), `wasm` (no filesystem by design),
    untrusted tier, and archive-form bundles - open() through the service; bytes or a
    scoped read stream come back. wasm receives data through its linear-memory door, exe
    through the D14 attach-once channel if a stream is needed.
- **Guards at the window**: capability check (D26), quota + accounting (D22), hash
  verification on first open.
- **Cross-bundle references**: in-bundle references use plain relative paths resolved
  against the caller's own bundle (Qt's `:/` scheme is unnecessary inside one bundle).
  Reading another bundle's resources is denied by default; a
  `org.modukit.share-resources` capability word (D26) and a `res://<bundle-id>/...`
  reference convention are both deferred to their triggers - first cross-bundle need
  (capability + policy hearing), first UI-compositor theme mixing (URI convention, with
  the Stage-2 theme day). Qt-style runtime mount/unmount (registerResource) is
  deliberately not provided: resources update with the bundle under D8 semantics; a
  second hot channel would fork the update model. The fast lane skips live accounting by design (accepted for
  trusted in-process plugins; install-time hashes remain).
- **Concurrency**: the window lane may run reads on a pool behind the gate (D6 sync-boundary
  semantics preserved; pool = implementation note, not architecture; build it only when a
  real plugin shows batch-read latency - second trigger after the window itself).
- **Compression**: a packaging-tool concern, not a manifest field. Per-entry strategy by
  type (store already-compressed formats, deflate/zstd text) belongs to the future
  packaging CLI - same trigger day as the archive-format choice (D28).
- **Year 1**: directory-form bundles + `resources[]` declaration + cross-check only.
  The ResourceService itself, the fd fast lane, and the window are trigger-gated on the
  first plugin that needs resources across a boundary (first wasm/exe resource consumer).
- **Evaluated and rejected**: compiling resource packs as wasm modules (code container for
  data: format/latency/memory/update-granularity taxes, zero industry adoption); plugin-side
  direct file reads as the universal path (wasm has no filesystem; archives would need
  unpacking; breaks D2's one-answer rule).

## 6. Configuration: schema in the contract, values in deploy, live keys opt-in (D29)

Configuration is data, so it goes through doors that already exist:

- **Schema** lives in the plugin's own `.proto` (a `Config` message per D13/D12) - typed,
  code-generated cross-language, validated by the same contract gates as any field.
- **Values** live in the deploy-TOML; the D7 cross-check verifies names and value sanity at
  resolve, together with everything else it already checks.
- **Change = restart with new values** (D8 stop-based semantics; zero new machinery). Restart
  is the default and the only year-1 lane.
- **Hot lane is opt-in**: a plugin may declare `config_live = ["key", ...]` in its manifest.
  Declared keys additionally receive change events as ordinary topics; the plugin must
  tolerate mixed old/new values in flight. Undeclared keys never hot-apply - a plugin that
  assumes hot-apply without declaring it is a liar fixture (same family as the D7 liar).

Year 1 implements schema declaration + value read + cross-check only. The hot-lane event
specification is recorded here and implemented when the first `config_live` plugin arrives
(trigger-gated).

## 6.1 Layout, precedence, and effect timing (D37)

File layout (per host tree; directory IS the instance boundary - one host_id, one tree,
configs never shared across hosts in writing):

```
runtime.toml    host itself: host_id, carrier endpoints, log sinks, shutdown budget
deploy.toml     plugin instance values ([plugin-id] sections, D29)
overlay.toml    optional field overrides, deployment-floor hand edits (last word)
bundles/<id>/<ver>/modukit.toml   author-owned manifest + proto defaults (immutable on-site)
```

Precedence: overlay > deploy > manifest defaults (layered overrides, kustomize/drop-in
pattern). Effective value resolution at load; missing keys take the `.proto` default;
surplus stale keys warn (rollback-friendly, upgrade-critical); type mismatch fails resolve
(D37 follow-up from the Cordis/DSH study: overlay sections target plugin ids - patches are
id-addressed, the shape fleet orchestrators will replay; `modukit config dump` prints the
resolved effective tree annotated by source layer - one-screen answer to "where does this
value come from")
(D7 family). Schema migration is the plugin author's job (docs + migration guide); no
framework migration DSL (trigger-gated).

Effect timing: the atomic unit of configuration effect is THE STARTUP OF WHAT IT DESCRIBES -
plugin values ride plugin restart (D29), live keys ride the hot lane (D29 trigger),
overlay edits wait for the next explicit restart (no file watching - predictability, D3),
host config rides host restart (changing host_id at runtime = split brain), bundle
manifest never "takes effect" - changing it means replacing the bundle (D28). There is no
half-applied state.

## 6.2 ConfigService, hot-lane writes, RBAC, and audit (D37 - built at the D29 trigger)

The live hot lane gets a full design now (built when the first `config_live` plugin
arrives):

- **ConfigService** (ledger-native, D4/D9 identity): `set(plugin, key, value)` gates -
  schema check (.proto types, free from D13), constraint check (range/enum extensions in
  proto), live-key verification (undeclared = reject, event recorded per D35 gate/audit),
  capability check (`org.modukit.config-write`, D26 vocabulary +1), then publishes the
  change event (ordinary topic, `org.modukit.*` names) and atomically swaps the plugin's
  Config snapshot (immutable object; old references stay valid - lockless, no half-old).
- **Persistence**: the new value is asynchronously written back to the overlay layer -
  the FILE remains the single source of truth; disk-write failure = event alarm + old
  value returns on next restart (honest mode, restart semantics unchanged).
- **RBAC, deliberately thin**: identity = the bridge plugin's deployment declaration
  (runtime.toml `identity`/`allowed_roles`); NO account system year 1 (solo+AI, D5) -
  a real identity service is platform-level work (D35 orchestrator side). Role-to-key-
  prefix mapping lives in runtime.toml (`ops: ["*"]`, `biz: ["app-biz/*"]`, `viewer: []`);
  prefix = D36 naming discipline as permission boundary, zero new concepts. ABAC rejected,
  trigger-gated on a real attribute-rule requirement. Local CLI = full rights; bridge
  identity authenticity rests on filesystem permissions of the deployment tree (physically
  controlled environment; remote untrusted environments are the bridge's own TLS/token
  problem - D35 three-nos). Audit is forensic-grade NOT claimed (local root can edit
  rings); compliance-grade audit = external compliance system, trigger-gated.
- **Self-build boundary**: the whole permission/audit surface is ~200 lines riding
  existing machinery (ring/capability gates/name prefixes) - no RBAC framework dependency
  (casbin-class frameworks out-weigh the need), no login/token system (bridges co-sign;
  remote scenarios handle their own tokens).

## 6.3 Web admin stance (D37): read = free rider, write = never in the framework

Read-only views (current values/source layer/plugin health) ride the D32 live-view page
tabs (trigger shared with D32) - data from introspect only, zero new machinery. Web WRITE
paths never enter host or framework: the only sanctioned write path is FILES + version
control (GitOps posture; K8s dashboard/kubectl division, rqt-parameter anti-lesson).
A full Web admin (write forms/RBAC UI/audit) is a STANDALONE TOOL hearing if a real
ops-team need materializes - with RBAC + audit + double-confirm + git persistence, still
bound by D37 effect semantics (files first, restart to apply). AccessBase evaluation
(adjudicated): not a submodule (TS stack + door-access domain mismatch), not a hosted
plugin (it assumes itself the authority - conflicts with file-as-truth; permission model
translation loss; lifecycle black box); adopted only as audit-ledger FORMAT reference
(who/what/when/evidence) - a forwardable compatibility note exists; conditional future
path: "external-authority mode" (pluggable persistence + pure-RBAC simplified mode +
stateless web) would make it a candidate at the standalone-tool hearing.

## 6.4 Admin web stack, when that day comes (D37)

Form/admin pages: Svelte 5 + Vite + TS (signals fit form-dense state, compiler-borne
performance, smallest bundle for weak environments). Online IDE cabin (script/blueprint/
DSL debugging, D33 layers): Monaco (editor, language-agnostic) + react-flow (blueprint
canvas, de-facto standard) - that cabin is React-flavored. Monitoring/log STORAGE is never
self-built: Grafana/Loki-class external infrastructure fed by our sinks (D19 posture);
Foxglove stock panels tried first for debug views (D32). Coexistence principle: multi-stack
allowed, prefer SEPARATE-PAGE isolation (per-page builds, plain navigation links) over
in-page microfrontends; single-stack collapse (all-React) explicitly allowed at project
day; re-evaluate React-first if frontend work goes to a larger team. Anything web: a
standalone tool, never in the host (D17 constitution).

## 6.5 Unified web-admin blueprint (D39 - hearing exhibit, zero year-1 work)

The full-form admin web (all seven sections) is designed now, built only at its
triggers - this section is the exhibit for the D37 standalone-tool hearing:

- **Shape**: IDE skeleton + cockpit bays - fixed navigation (seven sections: plugins /
  communication / services / debugging / logs / audit / settings), panel-based interior
  (draggable layout, Foxglove/Grafana style). Shell = Svelte 5 (nav + RBAC facade +
  identity bridge); bays = separately-built static packages on separate pages (D37
  coexistence principle; microfrontends only as fallback).
- **Data law**: every bay speaks exactly three protocols - introspect calls (read),
  EventStream subscriptions (streams), ConfigService (live-key writes). New panels never
  add new protocols (D17 constitution, positive form). The web produces viewports, not
  data; the host remains untouched.
- **Multi-host view**: host switcher fed by carrier discovery (D36); every panel
  annotates data with host_id.
- **Sections** (panels / data source / trigger): plugins (list/detail/install via D28 /
  introspect+dump / D32 bell); communication (topology live-view D32, topic browser with
  D16 replay, D21 rates / EventStream+Snapshot / D32 bell); services (three-view graph
  D32, edge table D18 / Snapshot+EventStream / D32 bell); debugging (online script/
  blueprint/DSL IDE per D33 - Monaco+react-flow, DAP shells D17, replay control D16 /
  bridge debug channel / NEW trigger: first remote-debugging need); logs (live stream +
  historical search via Loki - external, we ship the sink D19 / EventStream+Loki /
  deployment-scale trigger); audit (config-change history = ConfigService event replay,
  governance events per D35 / EventStream / D32 bell); settings (layer-annotated config
  dump, live-key editing under D37 RBAC, role-prefix map view / ConfigService+dump /
  ops-team trigger).
- **Reference designs**: DSH plugin-manager (settings from schema, adopted D38),
  Foxglove (panel layout/replay - tried first at the D32 bell), Grafana (embedded, never
  rebuilt), VS Code Web (IDE cabin), Portainer-class light ops consoles (install/log
  forms).
- **Trigger ladder**: (1) D32 bell -> shell + plugins/communication/services read-only
  (live-view expansion, weeks-scale); (2) first remote-debugging need -> debugging bay;
  (3) ops team appears -> D37 write-path hearing + settings unlock + full RBAC;
  (4) deployment scale -> Loki; (5) compliance export need -> audit export format.

## 7. What a year-1 manifest deliberately does not solve

**Config never mounts code (D37, Cordis/DSH cautionary lesson)**: DSH-style "config rows
mount code" (patching config to load arbitrary modules) is rejected - our config carries
values only; anything executable arrives as a bundle (D28) through the contract door (D13).
Letting config pull code is the first step around the contract door and collapses the
trust model.

Script dependency pinning (runner-level), i18n metadata, UI contribution points (compositor
track, demoted), license/marketplace metadata. Each has a named owner stage in
architecture.md §12 — none of them changes the kernel object model.

## 8. Three-tier residence and cross-check (D7)

| Tier | Carrier | Fields | Owner |
|---|---|---|---|
| **deploy** | bundle `modukit.toml` (TOML by adjudication 28: human annotations + typed + nested tables + Cargo-grade parser; INI cannot nest the resource tree, JSON cannot carry comments, YAML's indentation minefield and parser divergence are deployment-floor hazards; `resources[]`/config ride the same envelope) | `entry.kind` floor (raise-only), artifact/runner, `start_level`, `start_timeout`, `stop_timeout` (D34), `host_id` (D36, host-level config), `restart{}`, requires `policy`/`on_down`, `capabilities[]`, `min_host`, `resources[]` (D30) | ops / packaging |
| **self-report** | `PluginAbiV1` descriptor fields (04 §3) | `id`, `version`, `provides[]` (+ ranking, `parallel_ok`), requires + `topics[]` summaries (D10) | plugin code — or its runner, for script plugins |
| **verification** | pre-dlopen ELF/Mach-O/PE parse | link/dependency sanity | kernel, stage-5 only; a guard, never a truth source |

Ledger note (D13): the cross-check reads the `.proto`+`fdset` originals in the repo —
manifest payload names must resolve in the ledger; descriptor self-reports must match their
ledger entries; anything missing from the ledger cannot be wired at all. Carrier
choice (`shm-first | compressed line | WebRTC | zenoh`) is a deployment-side manifest
field (D14, 07 §4): the type side never encodes how cargo moves. Pool geometry
(`cell_bytes`, depth — the deployment face of the ticket, D15 08 §2) lives here too.

- **Boot DAG builds from the deploy tier alone**: zero plugin code loaded before the ordered
  `RESOLVING` pass (the cockpit boot-budget requirement; same motivation that made VST3
  publish headless scan metadata — industry knowledge).
- **Cross-check rule**: at resolve, deploy ↔ self-report must agree on `id`, `version`, and
  the provides/requires summary; disagreement = resolve failure before any `start()`.
  The contract suite carries a *liar fixture* (manifest claims v1, binary reports v2 ->
  must fail). This is what makes the file tier safe to trust for ordering at all.
- Policy fields (`restart{}`, `on_down`, placement overrides) are deliberately not
  code-verifiable — ops-owned by design, enforced by host policy only.
