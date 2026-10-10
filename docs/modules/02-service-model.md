# 02 — Service Model: C+ (static binding, dynamic events)

D4. Type-safe discovery and dependency injection (whitepaper §4.1), operationalized as:
**dependencies declared in the manifest, validated at resolve; a dynamic event stream for
presence; a frozen object model so OSGi-grade vocabulary can arrive later without touching
call sites.**

## 1. Registration and lookup

```
Registry::register::<S>(Arc<S>, props) -> RegistrationId
Registry::acquire::<S>() -> Option<ServiceRef<S>>
Registry::release(&ServiceRef<S>)
Registry::subscribe(Filter) -> Receiver<ServiceEvent>
Registry::snapshot(Filter) -> Vec<RegisteredService>
```

- **Handles, never raw instances** (invariant 1). `ServiceRef` derefs to the instance locally;
  across a process boundary the same handle routes a proxy. Consumers write against `acquire`.
- `RegistrationId`: monotonic `i64` (invariant 2). Identity for events and properties.
- Properties: flat `BTreeMap<String, Value>` with **case-insensitive keys** (invariant 3) —
  the string-keyed future host of LDAP vocabulary.
- `ranking: i64` reserved, tie-break only in year 1 (invariant 4).
- Events (invariant 5): `Registered{id, props}` / `Unregistered{id, props-at-last-known}` /
  `Modified{id, props}`. A new subscriber first receives `snapshot()`, then events — late-join
  is defined, not accidental.
- Acquire/release pairs are mandatory at call sites even though the year-1 local transport
  keeps refcounts internally (invariant 6) — per-bundle leases later require no consumer edits.

**Canonical service identity (D9)**: the true key is a reverse-DNS string (`com.<vendor>.
<area>.<Service>` — the CLAP id grammar, `docs/reference/clap-plugin-abi.md:58`), carried in
the registration `properties` map under `service.id`; versions never embed in the name
(invariant 3 carries them). Rust code never hand-writes the string: `#[modukit_service("...")]`
(v0: a plain `const SERVICE_ID`) derives typed lookups from it; manifest short names resolve
through **bundle-scoped aliases** (05 §3) that never cross the wire or the ABI. The
cross-check (05 §6) compares canonical strings only — the alias layer is sugar by construction.

Every call through a generated client stub also records one observation into the live
edge ledger `(caller, service, method) -> {count, last-use, latency sums, max,
slow-count}` (D18/D21, modules/10 §6-6.1).

## 2. Dependency model

Manifest-declared (05 §3), resolved at `RESOLVING`:

| Field | Values | Meaning |
|---|---|---|
| `service` | type name | what is required |
| `required` | `true` / `false` | missing required -> resolve fails (deterministic startup); optional may appear/disappear anytime |
| `policy` | `wait(timeout)` / `failfast` / `fallback` | consumer behavior while absent |
| `on_down` | `restart-self` / `block` / `fail-self` | behavior after a *previously present* optional/required service vanishes |

`fallback` semantics are an open item (cached-stale vs default-value vs error-carrying stub):
architecture.md §11 queue. The event stream plus snapshot covers both implementations, so the
enum can stay unimplemented without freezing the design.

**No polling path exists in the API.** A consumer either holds a `ServiceRef` or subscribes;
repeated `acquire()` loops are a review smell.

## 3. Why C+ and not the two edges

- **Not static-only**: the placement model (01) plus restart policy (03) make service churn a
  normal event — an all-static model would force process-level reboots for one service blip.
- **Not full OSGi dynamic**: property filters, `ServiceTracker`, factories and per-bundle
  leases have **no production precedent across a process wall** — CppMicroServices, the
  canonical C++ OSGi, declares "no remote/distributed story" outright
  (`docs/reference/cppmicroservices.md:80`), and its own consumer surface weighs
  ~1160 header lines (`cppmicroservices.md:38`) that every plugin author would pay.
  CTK carries "15 years of edge-case hardening" including tracker races
  (`docs/reference/ctk.md:79`). The invariants above exist so adopting those features later is
  purely additive, once a real cross-process need — not taste — demands it.

## 4. Long tasks: the job pattern (D6)

Boundary calls are synchronous by contract (04 §2). Long-running work is modeled explicitly:

```
trait RenderService { fn submit(&self, req: Frame) -> Result<JobToken>; }
// completion: a JobToken result arrives on the registry event stream as
// { token, progress | done(result) | failed(error) }
```

- `JobToken` is a correlation id, not a future — it survives serialization, placement
  switching, and later per-language runners unchanged.
- Waiting on a job uses the same consumer-policy vocabulary as §2 (`wait(timeout) |
  failfast | fallback`); cancellation, where honored, is a message (`cancel(token)`) —
  never a dropped `await`.
- Lineage: LSP request ids / Wayland's deliberate absence of sync calls (its
  `wl_display_sync` is the token ancestor); WinRT `IAsyncInfo` proves the same shape works
  across an in-process COM ABI. X11's sync roundtrip is the named trap this design avoids.
- **Timers are this pattern's purest citizen (D20)**: `timer(deadline) -> JobToken`, due as
  a `done` event — no new mechanism; waiting joins the job vocabulary (03 §6).

## 5. Deliberately not in year 1

LDAP filter vocabulary; `ServiceTracker` as a public type; `ServiceFactory` (per-consumer
instances); per-bundle lease accounting beyond internal refcount; service registry queries
across namespaces/tenants; multi-instance version routing for `parallel_ok` services
(D8 — the declaration field ships inert; the routing mechanism arrives with exe placement).
