# DeepSeek Harness (DSH) — Reference Profile

> **Research date:** 2026-10-12 · **Checkout:** `dsh-v0.2.1-alpha.2` (d74326738, 2026-10-09; cloned 2026-10-12) · **Upstream:** https://github.com/deepseek-ai/deepseek-harness
>
> **Local path:** `~/.cache/modukit-research/deepseek-harness` (not under `.refinfo/`, never cited as ours)
> **License:** MIT · **Stack:** TypeScript, pnpm monorepo (~60+ packages) · **Kernel:** [Cordis](cordis.md)
> **Scale note:** ★247k, extremely active (pushed 2026-10-09); desktop/SDK/CLI/web all ship from one tree.

---

## 1. What it is

DeepSeek Harness (`dsh`) is DeepSeek AI's open-source **agent harness** built on the
doctrine **"Everything is a Plugin"** (README title) and the Cordis meta-framework
(kernel profile: `cordis.md`). It is the closest thing to a *productized* plugin
framework in the AI-agent world: CLI, web client, Electron desktop, headless runner,
and JSON-RPC SDK all boot from the same plugin tree, differing only in **which plugins
are stacked at boot**.

For ModuKit it is the richest recent case study in: **composition layering (profiles →
bundles → patches), capability seams as named services, durable session events, and
plugin distribution/UX** — all problems we have adjudicated for the industrial/embedded
context, solved here for the agent context.

## 2. Measured shape (this checkout)

| Metric | Value | Notes |
|---|---|---|
| Version | `dsh-v0.2.1-alpha.2` (2026-10-09) | young but already multi-surface |
| Packages | ~60+ under `packages/` (boot/host/core/llm/mcp/fs/jobs/identity/client/...) | strict package-per-service discipline |
| Plugin-management UI | `packages/client/ui-plugin-manager`, `ui-settings-plugins`, `ui-settings-plugin-inventory` | plugin UX is a first-class product surface |
| Plugin inventory | `packages/host/plugin-inventory`, `packages/llm/plugin-package-inventory-deepseek` | server-side plugin registry machinery |
| Docs | `docs/architecture.md`, `agent-lifecycle.md`, `capability-seams.md` (auto-generated service graphs), `config-catalog.md` (generated), `api-gateway.md` | docs generated from code (`scripts/gen-doc-graphs.ts`) |
| Distribution | npm + Creative-Workshop-style plugin ecosystem (`dsh-plugin` topic, awesome lists, community hubs) | the plugin economy layer |

## 3. Key mechanisms (with anchors)

### 3.1 Profiles → bundles → patches: layered composition (the star)
A running `dsh` is "a plugin tree composed at boot from ordered layers"
(`docs/architecture.md`):

- **Profile** = named composition in Harness home (`web`, `headless`, `sdk`, `sdk-minimal`,
  `acp` ship as templates) — lists stacked bundles + user's `cordis.patch.yml`;
- **Bundle** = distribution format for "Cordis config rows and the code they mount",
  declared in `package.json` under a `dsh` field (`dsh.profile` / `dsh.bundle`);
- **Patches** target rows **by id** and replace whole configs or insert rows; layers apply
  in order: profile bundles → profile patch → home patch → `--patch` CLI overlays;
- `dsh --profile web --dump-config` prints the resolved tree; **any printed row is
  patchable**.

**vs ModuKit**: this is D37's overlay>deploy>defaults and D31's TOML discipline,
generalized to *composition of code*, not just values. Their `dsh.bundle` ↔ our D28
bundle; their patch layers ↔ our overlay layer. The striking generalization: **config
rows mount code**, so "patching config" *is* the deployment mechanism — an idea our
deploy-TOML deliberately stops short of (we keep code in bundles, values in files; D13
contract door).

### 3.2 Capability seams = named services with event-attached adapters
`docs/capability-seams.md` (auto-generated graph) enumerates every service as
`ctx.<name>`: `ctx.llm` (adapter registry), `ctx.fs`, `ctx.tools`, `ctx.telemetry`,
`ctx.browserUse`, `ctx.computerUse`, `ctx.tokenMeter`, `ctx.sessions`... Each seam is a
**core spine service, a swappable capability seam, a bundle point, or standalone**.
Adapters attach via **capability events** (`fs/*`, `tools/*`, `telemetry/*`) "without
importing the loop".

**vs ModuKit**: structurally identical to our D4 service model + D26 capability
vocabulary — but their gating is *convention* (event domains), not *enforcement*
(our D26 install-time checks + D2 trust tiers). Their `ctx.*` namespace discipline is
our D36 name-plate doctrine in single-host form.

### 3.3 Session log = durable facts; agent events = live control
The session log "is the source of the context the model sees" — durable facts append to
`session/event`; live control/status ride `agent/*` events; failed attempts are retained
as settled records (`assistant/attempt`) without polluting model history. Fork, resume,
transcripts, telemetry, persistence **all derive from the durable settlements**.

**vs ModuKit**: this is D16 (recording) + D19 (ring) + D18 (ledger) doctrine: durable
facts on the log, live incremental on the stream, everything derives from settlements.
Their "hard process loss before settlement leaves no durable attempt stream" is our
D16 gap-stats honesty in different clothes. Their `deriveMessages()` projection = our
replay-as-virtual-publisher (D16) — both refuse a second truth channel.

### 3.4 Agent lifecycle with waterfall hooks
`docs/agent-lifecycle.md` (generated sequence diagram): turn/step lifecycle with
**waterfall hooks** (`agent/pre-step`, `agent/request`, `system-prompt/assemble`) that can
**authoritatively reject or transform** — failed pre-step rolls back initialization;
cancellation during async phases "commits neither system nor users".

**vs ModuKit**: waterfall hooks ≈ our D6 job pattern + D4 static bindings (declared
ordering); their "failed init rolls back" is our D8 stop-based semantics and D3 Failed
edge. Validation of the same lifecycle instincts from the agent world.

### 3.5 Plugin manager as a product surface
Plugin manager exists thrice: boot (`packages/boot/plugin-manager`), host inventory
(`host/plugin-inventory`), and client UI (`client/ui-plugin-manager` + settings pages).
Plugin **installation** is transactional ("Shell-owned UI runs plugin transactions
through bundled pnpm") — install = apply patch rows + mount code, with lockfiles.

**vs ModuKit**: our D28 install protocol (staging→verify→atomic rename) is the
file-system-grade equivalent; their pnpm-transaction approach is the package-manager
grade. Their plugin **settings UI** (schema-driven from plugin Config types) is exactly
what our D29 ConfigService + D32 live-view tabs would grow into at larger scale —
UI generated from the same schema the kernel validates.

### 3.6 SDK-minimal: the anti-layer profile
`dsh-sdk-minimal` is "the deliberate exception: one bundle owns its complete explicit
SDK tree and does not apply dsh-base" — a profile that **refuses the shared base** for
a minimal, explicit dependency tree.

**vs ModuKit**: a nod to our D5 thin-edge instinct — even a maximal framework ships an
official minimal profile. Their lesson: profiles make "thin" a *choice*, not a fork.

## 4. Strengths (measured)

1. **Composition layering done product-grade**: profiles/bundles/patches with id-targeted
   patches and printed-tree introspection (`--dump-config`) — the most complete
   realization of D37-style layering seen in any reference so far.
2. **Docs generated from code**: capability-seam graphs and config catalogs are
   regenerated (`gen-doc-graphs.ts`) — our D18 analyzer + D32 graph tooling instinct,
   applied to docs.
3. **Durable/live event split** (session log vs agent events) — clean answer to the
   recording-vs-live-streaming tension (D16/D19).
4. **Transactional plugin management with real UX** — the missing layer in every other
   reference (Celix/CppMS ship no plugin manager UI at all).
5. **sdk-minimal anti-layer profile** — thin edge as a supported product shape.
6. **Multi-surface single tree** (CLI/web/desktop/headless/SDK) — proof that one plugin
   tree can serve wildly different entry surfaces.

## 5. Weaknesses / limits (measured + structural)

1. **Alpha maturity** (v0.2.1-alpha.2): APIs moving; the plugin ecosystem is forming
   around it (community hubs already exist — velocity outpaces stability).
2. **Single-process, TS, no isolation**: plugins share the Node event loop; "sandbox and
   approval policy" exist as services but enforcement is convention-level; no trust
   tiers (everything runs with full user authority — the harness trusts its plugins the
   way a shell trusts scripts).
3. **Config-rows-mount-code**: powerful, but means "config" can inject arbitrary code —
   their trust model rests on npm-registry trust, not structural isolation (our D28
   verification ladder + D2 tiers answer this for hostile environments).
4. **pnpm/lockfile coupling**: plugin transactions are package-manager transactions —
   heavyweight for embedded deployment (our D28 staging+rename is the leaner shape).
5. **No determinism story** (inherent to agent domain): long-running turns, model
   latency, retry semantics — inapplicable to our D20 step-clock domain, but instructive
   contrast for D6 sync-boundary choices.

## 6. ModuKit verdict

**Category: BORROW-PATTERNS (composition layering, plugin-management UX, durable/live
event split) + CAUTIONARY contrast (trust-by-convention vs our structural tiers).**

- **Confirms**: layered config (D37), bundle-as-distribution (D28), durable/live event
  split (D16/D19), schema-driven settings UI (D29+D32 future), name-spaced services
  (`ctx.*` ≈ D36), docs-from-code (D18/D32 tooling instinct).
- **Contrasts**: their trust model = convention + registry trust (npm); ours = structural
  (D2 tiers, D26 gates, D28 ladder) — for hostile/embedded deployment we cannot follow.
- **Borrow-worthy**: `--dump-config` (print resolved tree, every row patchable) as a CLI
  affordance for our `modukit graph --view declared` (D32) and future `modukit config
  dump`; profiles-with-minimal-exception (D5 thin edge as a product shape); patch-by-id
  targeting (D37 overlay refinement candidate).

## 7. Cross-references

- `cordis.md` — the kernel underneath (context/fiber/service mechanics).
- `zellij.md`, `deepseek-harness.md` vs — zellij = terminal app with plugin economy;
  DSH = agent harness with plugin economy; both validate the "plugin UX is product
  surface" thesis ModuKit defers (D23 CLI, Stage-2).
- ModuKit decisions exercised: D37 (layering), D28 (bundle), D16/D19/D18 (event
  doctrine), D29+D32 (schema-driven settings), D36 (namespacing), D5/D23 (thin profile,
  CLI tooling), D13 (contract door — the fork: their config-mounts-code vs our
  values-only config).
