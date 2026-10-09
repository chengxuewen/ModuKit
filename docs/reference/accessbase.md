# AccessBase — External Reference Profile

> Research date: 2026-10-09 | Checkout: 26cdcb9 (586 commits, 2026-08-21..2026-10-08;
> no tags) | Upstream: gitee.com/chengxuewen/AccessBase.git (private, same owner
> namespace as VisiaEngine — a sibling project, not third-party OSS)
> External reference profile. AccessBase lives in `.refinfo/` (git-ignored
> clone); it is NOT part of ModuKit. Claims below were measured on this checkout;
> anything not locally verifiable is marked UNCERTAIN. Its own `.agents/memorys/`
> tree is a near-direct ancestor of ModuKit's governance tree (ported via PolyOrch
> 2026-10-09), so several "borrow" rows here are really "keep enforcing what we
> already inherited."

## 1. Project portrait

| Attribute | Value |
|---|---|
| Name | AccessBase — enterprise access-control foundation (IAM) |
| Developer | Sibling of this repo family (`chengxuewen`); not a ModuKit module |
| Maturity | Production-shaped feature surface, no release tags; `private: true` |
| License | **none** — no `LICENSE*` at root (UNCERTAIN: closed/internal by design) |
| Language | TypeScript / Node (137 `.test.ts` files; pnpm workspace `packages/*` + `apps/*`) |
| Stack | Fastify + Drizzle ORM + PostgreSQL 16 + Redis; React + Ant Design + Vite admin |
| Positioning | A web application (server + SPA), NOT a native plugin kernel — the tech-stack mismatch with ModuKit is total |
| Why it is here | It is the *method* source (gate/test/PIT culture) and the literal ancestor of ModuKit's `.agents/` tree — not an architecture blueprint |

## 2. Architecture in focus

Monorepo split (measured): `apps/server` (Fastify: `routes/`, `utils/`, `oidc/`,
`middleware/`, `__tests__/`) + `apps/admin-ui` (Vite React: 20 `pages/`,
`components/`, `hooks/`); eight L0 packages `@accessbase/{types,logging,i18n,
migration,health,identity,audit,admin}`. `identity` carries the provider set
(`LdapProvider`, `OAuthProvider`, `SamlProvider`, `WebAuthnProvider`) plus manager
funnels. Persistence is Drizzle: schema glob `./packages/*/src/db/schema.ts`,
committed SQL chain `packages/migration/drizzle/0000..0013`, and a **single runtime
migration writer** `scripts/migrate.sh` (decision D117) that serializes via
`pg_advisory_lock` and is live-fired in three states (fresh / legacy / idempotent).

What a future ModuKit *host application* would and would not reuse here:
- **Reuse (discipline only)**: the "one authoritative writer per dangerous operation"
  rule (D117) maps onto ModuKit's plugin-install / registry-mutation surface; the
  mock-vs-real backend split for tests is a generic host-app lesson.
- **Do NOT reuse (shape)**: a Node HTTP server + React SPA is a *web* app topology.
  ModuKit's host is a native process embedding a Rust kernel (`modukit-runtime`), a
  CLI (`modukit-cli`), and in-process/multi-process plugins behind a C ABI — the
  Fastify route-table + Drizzle-DB pattern is the wrong substrate. Treat this section
  as a cautionary "what our host is NOT."

## 3. Key capabilities

AuthN/AuthZ breadth (all locally present): RBAC1 + role inheritance + tenant
partition, MFA (TOTP/WebAuthn), OAuth2/OIDC provider + generic RP (`end_session`),
SAML SP (SSO + SLO), LDAP, magic-link, SMS OTP, API keys, SCIM, data-scope row
filtering (D-scope middle path), audit + metrics, events outbox + webhooks +
bilingual email templates, key-rotation + `re-encrypt` tool. Multi-node coherence
(Redis pub/sub options-cache) and cross-instance smoke are documented in its status.
None of these features are ModuKit-relevant; the *breadth + per-feature gate* is the
signal.

## 4. Development & current state

- Velocity: 586 commits in ~7 weeks (single-owner lineage; bus factor ≈ 1).
- Memory corpus (measured): **89 PIT records**, **124 D decision records**
  (D1..D126) — the D/PIT numbering ModuKit reset to fresh templates.
- Machine-enforced gates live in `.agents/memorys/conventions.md` + CI + package
  scripts, **not** a monolithic `scripts/gate.sh` (no such file exists here). Note
  the asymmetry with ModuKit's ported `rules/common`, which still references a
  `scripts/gate.sh` that does not exist yet (recorded open item).
- Test baselines (flipped in the same commit as any count change, per convention):
  vitest `1530 passed (137 files, PG-up)`, e2e `186 passed + 3 skipped`.
- E2E discipline: 34 Playwright specs; default config serves `admin-ui` only under
  CI (mock API needs no backend), a separate serial `setup-real` project runs against
  a live backend; `e2e/helpers/backend-control.ts` drives `accessbase.sh reset` +
  detached `setsid nohup` relaunch. Real-hardware verification days are recorded
  explicitly and destroy their scratch instances afterwards.

## 5. Ecosystem & adoption

No public GitHub presence (private gitee origin; decision D122 notes a Gitee→GitHub
one-way mirror for Actions). Stars/adoption UNCERTAIN by design. Stack analogues:
Fastify, Drizzle, Playwright, Vitest, Ant Design, pnpm + (historically) pixi. It runs
its own `docs/reference/` series (19 profiles) — the same reference-analysis practice
this document belongs to; the family reciprocates.

## 6. Highlights & limitations

**Highlights.** A mature, *self-policing* gate culture with negative-proof discipline
(every check must be seen red at least once); PIT-numbered commits that round-trip
into `.agents/memorys/pitfalls.md`; baselines flipped in the same commit as the count
change; an implementation-status header gate on every module doc (D126); and a hard
"mock green ≠ real green" ethic — defects are chased down on a live IdP↔RP loop, not
asserted past by mocks. The CJK-free artifact gate (its conventions lines 197-199) is
the literal origin point of ModuKit's C1.

**Limitations.** Tech stack is a Node/TS web app — zero code reuse for a Rust plugin
kernel. 124+89 records with prose-heavy bilingual history (legacy Chinese entries
predating the 2026-09-21 English-only policy D121) is high loaded-instruction rent;
ModuKit deliberately reset memory to terse English templates. Rule-pack sprawl (14
language dirs incl. `zh/`) exceeds ModuKit's 6 mounted sets. Single-owner velocity
means doc↔reality drift is policed by the author's own gates (works, but
self-referential). Several gates are Bash + psql + pixi-shaped and carry GNU-isms —
unportable into ModuKit as-is.

## 7. Historical lessons (from its own recorded history)

1. **PIT-086 (HEAD commit) — mocks structurally hide one-time-code bugs.** React 18
   StrictMode double-fires the mount effect; the second `POST /auth/oauth/exchange`
   401s and the axios 401→refresh→logout interceptor wipes the session the *first*
   exchange had just minted. Mock-API e2e cannot see it (mocked handlers are
   idempotent). Fix = per-code one-shot `useRef` guards; a `=== 1` POST-counter e2e
   gate shipped same day and immediately caught a regression from its own insertion.
   **For ModuKit:** any single-consume flow (token exchange, install-once plugin
   activation, lease acquisition) is invisible to mock tests — gate it on a live loop.
2. **PIT-072 — "the upgrade fixes everything" is a false fact.** A hand-written
   Drizzle snapshot (partial index with `where`) poisoned all future `generate`
   calls; the assumption that bumping drizzle-kit would heal it was killed by a
   T0 scratch probe *before any repo change* (the upgrade silently skipped the
   hand-written files). **For ModuKit:** probe a tool's real behaviour before
   writing the fix into a plan; schedule "upgrade-day re-baseline" as a named node.
3. **PIT-064 — a gate that never runs is worse than no gate.** A Vitest coverage
   `exclude` prefix that did not match the `.pnpm` path layout produced a fake
   denominator and a never-executed 80% threshold ("zombie config"). This is the
   canonical negative-proof case that ModuKit's C0 ("constraints must be executable
   and observed red") generalizes.
4. **PIT-081 / PIT-083 — test infra self-sabotage.** A promise-memoized async
   constructor degraded to per-request under the concurrent first wave; a shared
   Redis rate bucket exhausted across back-to-back runs and surfaced as cross-file
   429 false-reds. Both became isolation doctrines. **For ModuKit:** concurrency and
   shared-state fixtures need explicit isolation, not luck.
5. **D121 + conventions gate — the birth of ModuKit's C1.** Its 2026-09-21
   English-only-persistence policy (with the CJK grep scans) is the ancestor ModuKit
   ported and hardened into a repo-wide executable gate from day one. Related
   retirements worth mirroring: D123 (drop the `openspec` skill family once a better
   spec pipeline lands) and D126 (gap-audit remediation ladder + status-header gate).

## 8. Value for ModuKit

**Verdict: BORROW-PATTERNS (method/gate culture) + REFERENCE-ONLY (tech stack).**
Honest split: as a *Node/TS web application* AccessBase is reference-only — no code,
no crate, no dependency, and its host-app shape is the opposite substrate of ModuKit's
Rust-core/C-ABI plugin kernel. As a *method source* it is high-value, and partly
already inherited: this tree is a near-ancestor of ModuKit's own `.agents/` governance,
so the borrow list is largely "keep enforcing what we ported, once code exists." No
license gate issue (patterns-only; the absent LICENSE is moot for method adoption).

- **Keep enforcing now (docs-only, already ported):** the C1 CJK-scan gate
  (its 197-199 → our executable C1); negative-proof as a law (its PIT-064 → our C0);
  D/PIT record numbering + a commit→memory round-trip habit.
- **Adopt when code lands:** PIT-numbered commit ↔ `pitfalls.md` linkage; "flip the
  test baseline in the same commit as the count change"; the implementation-status
  header gate on every `docs/modules` page (D126); the live-fire / verification-day
  ethic and its rule that mock-green ≠ real-green (PIT-086) — apply to single-consume
  plugin/lease/token flows; per-dangerous-operation single authoritative writer
  (its D117 `migrate.sh` → our registry-mutation surface).
- **Reference-only / avoid:** Fastify + Drizzle + Postgres/Redis mechanism and the
  React-SPA host topology (stack mismatch; not our host shape); the 14-language rule
  sprawl and bilingual legacy memory (ModuKit keeps 6 sets + terse English); Bash/psql
  + pixi GNU-ism gate scripts (rewrite to ModuKit `gate.sh` primitives, do not copy).
- **Divergence record (what changed on port):** memory reset from 89 PIT / 124 D
  bilingual corpus → fresh terse English templates; `rules/zh` and 8 language dirs
  dropped (6 mounted); the mid-life D121 English policy → enforced C1 from commit 0;
  the `scripts/gate.sh` reference ModuKit inherited does not correspond to any file in
  this ancestor (gates were conventions/CI-shaped) — trim or inline when Stage-1
  tooling lands.
