---
name: lsp-code-analysis
description: "Pre-change impact analysis for ModuKit: who defines, implements, references, or calls a symbol before it is edited, across the Rust kernel, the C ABI seam, the process wire, and the generated bindings. Uses serena (Rust) first, codegraph for call paths, ripgrep plus the parity gates as the truth check. Use when changing a public trait, an ABI table, a wire constant, an exported C symbol, or a verb name shared by crates, contracts, bindings, and docs."
type: process
notes: "symbol impact before the edit"
---

# LSP Code Analysis — Impact Before the Change

> Ask what breaks before you touch the symbol. The file tree is not the call
> graph, and in this repo one name can live in four places at once: the Rust
> trait, the C header, the generated skin, and the capability ledger.

## Trigger Conditions

- Changing a public trait, enum, or function used across crates.
- Touching the C ABI seam (`contracts/*.h`, `crates/modukit-core/src/abi.rs`,
  `embed_abi.rs`, `host_v2.rs`, `host_v3.rs`).
- Changing a process-wire frame kind or constant (`proc_wire.rs`,
  `contracts/modukit_wire.h`).
- Renaming or removing a verb, a manifest key, or a service name.
- The user asks "who implements this", "who calls this", "what breaks if I ...".

## Do Not Use

| Need | Use instead | Why |
|---|---|---|
| The layer is still unknown | `repo-triage` | Map kernel / transport / codegen / bindings / docs first |
| Designing a contract, not changing one | `api-interface-design` | Contract shape is a ruling, not a search |
| Docs-only edit | plain `grep` | No symbol graph involved |
| Bulk structural rewrite | `ast-grep-rewrites` | This skill stops at the impact list |
| Already green and reviewing | `code-review-and-quality` | Analysis is upstream of review |

## Tool Ladder

### 1. serena — LSP-backed, Rust only here

`.serena/project.yml` declares `language_servers: [rust]`. Use it for definitions,
implementations, references, and file overviews inside `crates/`:

| Question | Tool |
|---|---|
| Where is the symbol defined? | `find_symbol` / `find_declaration` |
| Who implements this trait? | `find_implementations` |
| Who references it? | `find_referencing_symbols` |
| What is in this file? | `get_symbols_overview` |
| Cross-project pattern | `search_for_pattern` |

Read the returned file and range, not only the symbol name. Ambiguous names need
the full symbol path.

### 2. codegraph — call paths and cross-file shape

`local-codegraph` answers "how does the call reach that" and groups results by
file. Its output counts as read: do not re-open the same file with `Read` only
because the answer came from the index. If the path is not indexed, the server
says so — follow that guidance instead of retrying.

### 3. ripgrep — the surfaces the index cannot see

C headers, `.proto`, golden frames, fixtures, docs, and the binding skins are
text, not Rust symbols. For any ABI or wire name, this step is mandatory:

```bash
grep -rn '<name>' contracts/ crates/ bindings/ test-plugins/ docs/ \
  scripts/check-*.sh
```

Never claim "no references" from one tool alone: an index can miss generated or
macro-produced items, and a text search can return a superset.

## Impact Map (this repo)

| Surface you are changing | Also check | Gate that proves it |
|---|---|---|
| Embed verb (`contracts/modukit_embed.h`) | `embed_abi.rs` `EMBED_VERBS`, `docs/capability-ledger.md`, cpp/py/js skins | `embed-parity`, `capability-ledger` |
| Host ABI table (`abi.rs`) | `host_v2.rs`, `host_v3.rs`, `contracts/modukit_plugin.h`, ABI version gates | `gate-abi`, `cargo test` |
| Wire constant (`proc_wire.rs`) | `contracts/modukit_wire.h`, `contracts/golden/wire-v1/` | `wire-parity`, `scripts/regen-golden.sh` diff |
| Manifest key | `bundle.rs`, `docs/modules/05-manifest-and-deployment.md`, deploy trees | `spec-trace`, core tests |
| Bus/clock behaviour | proxy face, demo legs | the matching `demo-*` leg |
| Service registry name | `docs/modules/02-service-model.md`, plugin fixtures | `cargo test`, `ctest` |

## Process

1. Name the symbol and its surface class (kernel / ABI / wire / manifest / bus).
2. Run ladder step 1-3; write the impact list as `file:line` entries.
3. Mark frozen surfaces: v1 is STABLE (D87) — an impact list that touches a v1
   shape means a hearing, not an edit (D83 tail-add policy).
4. Change, then run the owning gate from the impact-map table.
5. If the gate is absent for that surface, say so in the verdict — that is a
   coverage hole, not a pass.

## Rationalizations

| Excuse | Rebuttal |
|---|---|
| "The compiler will catch it" | The C header, the Python skin, and the ledger are not typed against the Rust symbol. |
| "grep found nothing else" | The index can miss generated code; run the parity gates before concluding. |
| "It is an internal rename" | `pub(crate)` names still ride the wire tables and the capability ledger. |
| "Impact list is obvious" | Obvious means unverified; cite `file:line` or say it is unknown. |

## Red Flags

- Editing `abi.rs` without reading `host_v2.rs` / `host_v3.rs` tails.
- Removing a verb row from `docs/capability-ledger.md` silently.
- Treating a frozen v1 shape as editable because nothing failed yet.
- An impact answer with no `file:line` and no gate name.

## Verification

- The impact list: every entry `file:line`, grouped by surface class.
- The named owning gate for each touched surface, run after the change.
- For ABI/wire work: the parity legs green, plus the golden diff when frame
  shapes are involved.

## References

- `.agents/rules/common/tool-routing.md` — search-tool division of labour.
- `.agents/skills/api-interface-design/SKILL.md` — contract decisions.
- `.agents/skills/repo-triage/SKILL.md` — layer map first.
- `docs/capability-ledger.md`, `docs/architecture.md` §11 (D83/D87).
