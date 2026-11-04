# Tool Routing — exploration division of labour

This file defines a division of labour, not a priority order. Host-native search
instructions are not overridden.

| Query type | Tool |
|---|---|
| Exact text, known symbol, exact reference | native grep or LSP (`findReferences`, `goToDefinition`) |
| Semantic exploration, call chain, impact surface, unfamiliar-code survey | `codegraph_explore` from `local-codegraph` |
| Ambiguous case that either tool can answer | choose one path and finish it; do not run both |

Rules:

- `codegraph_explore` source output counts as read. Do not re-read the same file
  only because the result came from an index.
- If the index is absent or the path is not indexed, follow the server's guidance
  and use the native path. This is graceful degradation, not a retry loop.
- An index can miss macro-generated or generated items; a text search can return
  a superset. Do not claim "no references" from one tool alone.

This file is locked to the `local-codegraph` MCP server by `gate-mcp`.
