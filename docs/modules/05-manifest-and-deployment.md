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
restart: { policy: on-failure, max_retries: 5, backoff: 1s..30s, cooldown: 5m }
```

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
- `capabilities` (whitepaper §3 principle 3: declarations beat assumptions): platform features
  the plugin needs; the host's capability discovery either satisfies them or the plugin fails to
  resolve. Vocabulary design is a queued item (architecture.md §11) — the field exists now.
- `provides.ranking` is reserved storage only in year 1 (02 §1, invariant 4).
- `provides.parallel_ok` is an author semantics claim (D8): cross-checked between tiers like
  every other descriptor field, inert until Stage-2 multi-instance routing exists. Nobody may
  set it to `true` speculatively and be surprised when two versions run — or when they don't.

## 4. Bundling and installation (stage 2+)

A plugin *bundle* = artifact + manifest + resources in one directory/archive; install writes
`INSTALLED` state entries (03 §1). **Out of year-1 scope**: OTA distribution, signature chains,
rollback. The manifest carries an optional `min_host` version field today so forward-compat
questions have an anchor later.

## 5. What a year-1 manifest deliberately does not solve

Script dependency pinning (runner-level), i18n metadata, UI contribution points (compositor
track, demoted), license/marketplace metadata. Each has a named owner stage in
architecture.md §12 — none of them changes the kernel object model.

## 6. Three-tier residence and cross-check (D7)

| Tier | Carrier | Fields | Owner |
|---|---|---|---|
| **deploy** | bundle `modukit.toml` | `entry.kind` floor (raise-only), artifact/runner, `start_level`, `start_timeout`, `restart{}`, requires `policy`/`on_down`, `capabilities[]`, `min_host` | ops / packaging |
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
