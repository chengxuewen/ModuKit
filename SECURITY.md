# Security policy

## Reporting a vulnerability

Do **not** open a public issue for a suspected vulnerability. Report it privately
to the maintainers through the repository owner contact on the hosting platform
(Gitee `chengxuewen/ModuKit`), including:

- the affected surface: the C ABI (`contracts/modukit_embed.h`), the process wire
  (`modukit_wire.h` / `proc_wire`), plugin loading, manifest/bundle parsing, or an
  environment/script layer;
- a minimal reproduction; and
- the impact you believe it has.

You will get an acknowledgement, then a fix or a written decision on scope.

## Scope and expectations

- The framework parses untrusted input through the wire decoder and bundle
  manifests, and loads plugins through `dlopen`/child processes; `unsafe` blocks
  and the FFI boundary are the highest-value targets.
- Panics must never unwind across the C ABI: the boundary catches or aborts
  (`catch_unwind` / `panic = "abort"` at the seam); the napi-side lesson is PIT-30
  (typed error arm - panics never cross the FFI boundary).
- Every `unsafe` block carries a `// SAFETY:` rationale (see
  `.agents/rules/rust/security.md`).
- The `cargo-deny` leg runs `cargo deny check licenses bans`: license policy and
  banned crates are checked when the tool is available, and an offline skip is
  recorded loudly, never silent. Dependency advisories are outside that command and
  `deny.toml` carries no `[advisories]` section; this box has no outbound network,
  so `cargo deny check advisories` and `cargo audit` cannot run here. Advisory
  screening stays a maintainer-owned activity on a connected machine.
- Secrets must never enter the tree. Generated host layers, local MCP credentials
  (`.agents/mcp.local.json`), and local MCP echoes are gitignored.
