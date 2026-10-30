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
- Every `unsafe` block carries a `// SAFETY:` rationale (see
  `.agents/rules/rust/security.md`).
- Dependency advisories are enforced by `cargo deny check licenses bans` when the
  tool is available; an offline skip is recorded loudly, never silent.
- Secrets must never enter the tree. Generated host layers, local MCP credentials
  (`.agents/mcp.local.json`), and local MCP echoes are gitignored.
