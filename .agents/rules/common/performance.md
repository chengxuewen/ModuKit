# Performance Optimization

## Model Selection Strategy

> Model names and prices below were ported Claude-Code-era specifics - environment-
> dependent (dangling-refs sweep, 2026-10-19). The tiering principle is what holds:
> match model cost to task depth across whatever models the host config offers.

- **Fast/cheap tier**: lightweight agents with frequent invocation; pair programming;
  worker agents in multi-agent systems
- **Mid/coding tier**: main development work; orchestrating multi-agent workflows;
  complex coding tasks
- **Deepest-reasoning tier**: complex architectural decisions; maximum reasoning
  requirements; research and analysis tasks

## Context Window Management

Avoid last 20% of context window for:
- Large-scale refactoring
- Feature implementation spanning multiple files
- Debugging complex interactions

Lower context sensitivity tasks:
- Single-file edits
- Independent utility creation
- Documentation updates
- Simple bug fixes

## Extended Thinking + Plan Mode

(Ported as Claude-Code keybindings and settings flags - dead on this host, stripped
2026-10-19. What holds is the habit, not the mechanism: deep reasoning has an
adjustable budget on most hosts; raise it for architecture-grade problems.)

For complex tasks requiring deep reasoning:
1. Ensure extended thinking is enabled (on by default)
2. Enable **Plan Mode** for structured approach
3. Use multiple critique rounds for thorough analysis
4. Use split role sub-agents for diverse perspectives

## Build Troubleshooting

If build fails:
1. Use **build-error-resolver** agent
2. Analyze error messages
3. Fix incrementally
4. Verify after each fix

## Runnable Check

```bash
# Verify: no unwrap() misuse
grep -rn "unwrap()" --include="*.rs" crates/ | grep -v "test" | grep -v "// OK" || true
```
