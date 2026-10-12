# Cordis — Reference Profile

> **Research date:** 2026-10-12 · **Checkout:** `master` (cloned 2026-10-12, ~80 TS packages, core src 1,891 lines) · **Upstream:** https://github.com/cordiverse/cordis
>
> **Local path:** `~/.cache/modukit-research/cordis` (blobful clone; not under `.refinfo/`, never cited as ours)
> **Companion:** DeepSeek Harness (`deepseek-harness.md`) — the flagship application built ON Cordis.

---

## 1. What it is

Cordis describes itself as a **"Meta-Framework of Spatiotemporal Composability"** (and,
more plainly in `packages/core/package.json`, "Meta-Framework for Modern Applications").
It is the plugin/dependency-injection kernel underneath DeepSeek Harness ("Everything is
a Plugin") and historically under the Koishi bot framework family. The design is written
up in an arXiv paper: [_A Programming Paradigm for Spatiotemporal Composability_]
(https://arxiv.org/abs/2608.25512).

For ModuKit's purposes, Cordis is the closest **pure plugin-kernel** in the TypeScript
world: a Context/Fiber/Service model where *everything* — config loaders, HTTP servers,
LLM adapters, UI panels — is a plugin, services are context-scoped injections, and
plugins compose as a tree of scoped contexts.

## 2. Measured shape (this checkout)

| Metric | Value | Notes |
|---|---|---|
| Monorepo | ~80 TS packages (`packages/`) | yakumo build orchestration |
| Core (`@cordis/core`, pkg name `cordis`) | **1,891 lines** across 8 files: `context.ts`, `fiber.ts`, `service.ts`, `registry.ts`, `events.ts`, `logger.ts`, `reflect.ts`, `utils.ts` | the entire kernel |
| Core tests | 20+ spec files (plugin/invoke/service/events/fiber/shadow/isolate/dispose/decorator...) | thorough kernel-level tests |
| Config validation | `StandardSchemaV1` (the community standard interface) via `runtime.Config['~standard'].validate` | sync-only validation, async explicitly rejected (`resolveConfig` in `fiber.ts`) |
| Dependency | `cosmokit` (shared utils), `@standard-schema/spec` | tiny surface |

## 3. The kernel model (mechanisms, with anchors)

### 3.1 Context = scoped service container, plugin tree = context tree
`Context` is both the DI container and the lifecycle scope. `root.plugin(fn)` forks a
**child Context** whose lifetime is bound to the plugin's return value (a dispose
function); disposing the parent tears down the subtree. Plugin identity is the function
name (`Context <foo>` in inspect output). The tree is the composition spine: **D36-style
scoping falls out naturally** (each subtree can host its own service instances).

### 3.2 Service = named, context-filtered, lazily-resolved injection
`Service` base class registers itself under a name via `ctx.reflect.provide(name, self)`
(`service.ts`); consumers declare `inject: ['service']` (or `@Inject` decorators).
Resolution is **context-filtered**: a service instance is only visible to contexts in its
isolation scope (`[symbols.filter]` compares `ctx[symbols.isolate][name]`). Config
resolution walks a prototype chain of intercept layers (`[symbols.resolveConfig]`) — a
**layered config override** remarkably similar in spirit to our D37 overlay>deploy>defaults.

### 3.3 Fiber = the lifecycle/disposal spine
`Fiber` owns plugin application, effect registration, and teardown ordering
(`fiber.ts`). Config validation failures throw `ValidationError` with structured issues
(StandardSchema); **inactive context** guards (`'inactive context'` throws on
`ctx.plugin/effect/on` after dispose) make use-after-dispose a loud error, not a
silent leak. Effects compose into DisposableLists.

### 3.4 Events as THE extension surface
`ctx.on(event, listener)` with typed event maps; DeepSeek Harness layers three event
domains on top (session events = durable facts, agent events = live control,
capability events = seam attachment). Cordis itself just provides the bus + typed maps.

### 3.5 Everything-is-a-plugin, taken literally
There is no privileged "core app" — even the config system, logger, and HTTP servers of
host applications are plugins. DSH's own architecture doc calls its running state "a
plugin tree composed at boot from ordered layers".

## 4. Strengths (measured)

1. **Tiny kernel, total doctrine**: 1,891 lines deliver full DI + lifecycle + events +
   scoped config; the "everything is a plugin" doctrine is enforced by there being
   literally nothing else in the kernel.
2. **Standard-schema config validation** (StandardSchemaV1): config contracts are
   validated through a *community standard interface*, not a bespoke validator —
   plugins declare their config type, the kernel validates at apply-time, structured
   errors (`ValidationError` with issue paths).
3. **Layered config intercepts**: per-service config resolved through a prototype chain
   of override layers (base → intercepts) — the same layering insight as our D37,
   implemented at the kernel level.
4. **Scope isolation as a first-class filter**: services resolve per isolation scope
   (`[symbols.filter]`) — multi-tenant/multi-instance composition is native, not bolted on.
5. **Use-after-dispose as loud error**: lifecycle discipline enforced at runtime.
6. **Thorough kernel tests** including failure paths (invalid plugins, inactive contexts,
   dispose ordering).

## 5. Weaknesses / limits (measured + structural)

1. **In-process, single-language (TS)**: no IPC story, no cross-language boundary, no
   process isolation — "plugin" means in-memory module. For ModuKit (dl/exe/wasm across
   a C ABI), Cordis is a *composition-model* reference only, not an ABI reference.
2. **No trust tiers / no capability gating**: any plugin can inject/observe anything;
   security is out of scope by design (same as OSGi's unported permissions).
3. **Sync-only config validation** (async explicitly thrown) — fine for TS, a limitation
   for registry-backed config.
4. **Runtime reflection-heavy DI** (`InjectKey` derives from Context interface members
   with `[symbols.config]`) — TS-type-magic heavy; not portable to a C ABI world.
5. **Monorepo sprawl** (80 packages) vs a 1.9k core — the doctrine scales, the repo
   around it is standard TS sprawl.

## 6. ModuKit verdict

**Category: composition-kernel reference (TS world) — validates our registry/scope/
config-layering choices from a completely independent stack.**

- **Confirms**: registry-as-the-universe (D4/D10: everything through the handle table);
  scoped config layering (D37 overlay>deploy>defaults); config-as-contract validated at
  apply time (D7 cross-check analog); plugin tree with child scopes (D36 host_id
  scoping analog); "kernel owns nothing, plugins own everything" (D17/D19 posture).
- **Rejects for us**: runtime-reflection DI (C ABI world needs generated, frozen
  bindings — D13); no-trust-tier openness (D2/D26); in-process-only plugins (D2
  placements).
- **Borrow-worthy**: StandardSchemaV1 *pattern* — contracts validated through a standard
  interface at apply time with structured errors; the kernel-doctrine purity (if it's
  not for everyone, it's a plugin) as a design smell-detector for our own kernel growth.

## 7. Cross-references

- `deepseek-harness.md` — the flagship consumer of Cordis (profile/bundle/patch
  composition layering is where Cordis's kernel gets production exercise at scale).
- ModuKit decisions exercised: D4 (service model), D7/D37 (manifest/config layering),
  D13 (contracts vs runtime reflection — the fork), D36 (scopes), D17/D19
  (everything-through-the-kernel posture).
