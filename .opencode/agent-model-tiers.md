# PolyOrch Model Tier System

> Five-tier model mapping: premium-max / premium / fast / vision / lite
> Dual-provider architecture: DeepSeek official direct + New API gateway aggregation

## Architecture

PolyOrch integrates LLMs through two independent paths:

| Provider | Integration | Characteristics |
|--------|----------|------|
| **DeepSeek official** | Direct API (`api.deepseek.com`) | Low latency, high stability, officially controlled model versions |
| **New API gateway** | Alias proxy (`192.168.100.47:3000`) | Multi-provider aggregation, automatic fallback, dynamic model switching |

The New API gateway internally aggregates providers such as DeepSeek / Qwen / Kimi / Doubao / GLM / MiniMax,
enabling per-provider multi-model fallback. Changing the alias mapping on the gateway side switches models globally with zero project config changes.

The DeepSeek official direct connection is an independent fallback path when the New API gateway is unavailable, forming a complete high-availability chain.

## Five-Tier Model Mapping

### Dual-Provider Comparison Table

| Tier | Alias | Use | DeepSeek official | New API gateway | New API Fallback 1 | New API Fallback 2 |
|------|------|------|---------------|-------------|-------------------|-------------------|
| premium-max | Deep reasoning | Most complex tasks | `deepseek-reasoner` | `deepseek-v4-pro-max` | `kimi-k2.6` | `minimax-m3` |
| premium | Primary | Orchestration/build/planning | `deepseek-chat` | `deepseek-v4-pro` | `qwen3.7-max` | `glm-5.1` |
| fast | Fast | Execution/search/review | `deepseek-coder` | `deepseek-v4-flash` | `qwen3.6-flash` | `doubao-seed-2.0-lite` |
| vision | Vision | Multimodal analysis | `deepseek-vl2` | `doubao-seed-2.0-pro` | `qwen3.6-plus` | — |
| lite | Light | Minimal/trivial tasks | `deepseek-chat` | `qwen3-32b` | `qwen3-8b` | — |

### New API Alias Mapping Notes

The New API gateway references models through semantic aliases (`premium` / `fast` etc.); OpenCode configs
reference the alias, not the concrete model name. Switching models only requires changing the alias mapping on the gateway side, not the project config files.

Current gateway alias mapping:

```
premium-max       → deepseek-v4-pro-max        # flagship reasoning
premium-max-1     → kimi-k2.6                  # Fallback 1
premium-max-2     → minimax-m3                 # Fallback 2

premium           → deepseek-v4-pro            # primary reasoning
premium-1         → qwen3.7-max                # Fallback 1
premium-2         → glm-5.1                    # Fallback 2

fast              → deepseek-v4-flash          # fast reasoning
fast-1            → qwen3.6-flash              # Fallback 1
fast-2            → doubao-seed-2.0-lite       # Fallback 2

vision            → doubao-seed-2.0-pro        # primary vision
vision-1          → qwen3.6-plus               # Fallback 1
vision-2          → gemini-3.5-flash           # Fallback 2

lite              → qwen3-32b                  # lightweight tasks
lite-1            → qwen3-8b                   # Fallback
```

## Per-Tier Details

### premium-max: Deep Reasoning

- **Use**: Most complex tasks requiring deep reasoning and multi-step thinking
- **Use cases**: Architecture design, system planning, complex refactoring, technical review, security audit
- **DeepSeek official model**: `deepseek-reasoner` — focused on complex reasoning chains, long context
- **New API primary**: `deepseek-v4-pro-max` / `kimi-k2.6` / `minimax-m3` — flagship model chain
- **Temperature**: 0.2 (high determinism)
- **Reasoning Effort**: high
- **Notes**: Highest latency, highest cost; use only for scenarios that genuinely need deep reasoning

### premium: Primary Reasoning

- **Use**: Everyday complex tasks requiring strong understanding and generation
- **Use cases**: Agent orchestration, build script writing, plan generation, code review, architecture consultation
- **DeepSeek official model**: `deepseek-chat` — DeepSeek's latest chat model, strong all-round capability
- **New API primary**: `deepseek-v4-pro` / `qwen3.7-max` / `glm-5.1` — primary reasoning chain
- **Temperature**: 0.2 (high determinism)
- **Notes**: The most-used tier in the project; balances capability and cost

### fast: Lightning Execution

- **Use**: Simple execution tasks requiring low latency and high throughput
- **Use cases**: Code search, file exploration, simple code edits, quick review, library search
- **DeepSeek official model**: `deepseek-coder` — coding-optimized model, fast responses
- **New API primary**: `deepseek-v4-flash` / `qwen3.6-flash` / `doubao-seed-2.0-lite`
- **Temperature**: 0.0 (fully deterministic)
- **Notes**: The most latency-sensitive tier; does not support complex reasoning; for high-frequency small tasks

### vision: Vision Specialist

- **Use**: Multimodal understanding, image/PDF content analysis
- **Use cases**: UI screenshot analysis, document scanning, diagram understanding, visual review
- **DeepSeek official model**: `deepseek-vl2` — vision-language model
- **New API primary**: `doubao-seed-2.0-pro` / `qwen3.6-plus`
- **Temperature**: 0.1 (high determinism)
- **Notes**: Use only when multimodal capability is needed; text-only tasks should use other tiers

### lite: Lightweight

- **Use**: Minimal tasks, maximum cost-effectiveness
- **Use cases**: Simple Q&A, log summaries, content formatting, metadata generation
- **DeepSeek official model**: `deepseek-chat` (lightweight calls)
- **New API primary**: `qwen3-32b` / `qwen3-8b`
- **Temperature**: 0.2
- **Notes**: Limited capability; use only for highly deterministic tasks that need no creativity

## Agent Tier Assignment

### Agents

| Agent | Tier | Description |
|-------|------|------|
| oracle | premium-max | Architecture consultation, technical decisions |
| sisyphus | premium | Main orchestrating agent |
| hephaestus | premium | Build management |
| prometheus | premium-max | Plan generation |
| atlas | premium | Implementation execution |
| librarian | fast | Library/document search |
| explore | fast | Code exploration |
| metis | fast | Metrics and data analysis |
| momus | premium | Review and criticism |
| sisyphus-junior | fast | Simple execution |
| multimodal-looker | vision | Visual analysis |

### Categories (task() dispatch)

| Category | Tier | Description |
|------|------|------|
| visual-engineering | premium | Visual engineering |
| ultrabrain | premium-max | Ultra-deep thinking tasks |
| deep | premium | Deep analysis |
| unspecified-high | premium | High-complexity uncategorized |
| artistry | fast | Creative/document writing |
| quick | fast | Simple tasks |
| unspecified-low | fast | Low-complexity uncategorized |
| writing | fast | Document writing |

## Selection Guide

### When to Use Which Tier

| Task type | Recommended tier | Alternative |
|----------|----------|------|
| Architecture design / system review | premium-max | premium |
| Agent orchestration / planning | premium | fast (simple steps) |
| Code writing / refactoring | premium | fast (small changes) |
| File search / grep | fast | — |
| Bug debugging | premium | premium-max (hard-to-reproduce bugs) |
| UI screenshot analysis | vision | premium (no vision needed) |
| Log summarization / formatting | lite | fast |
| Code review | premium | fast (simple format review) |
| Security audit | premium-max | premium |
| Simple Q&A | lite | fast |

### Quick Decision

1. **Need deep thinking?** → premium-max
2. **Need comprehensive understanding and coding?** → premium
3. **Simple execution or search?** → fast
4. **Need to look at images/PDFs?** → vision
5. **A few sentences can handle it?** → lite

## Provider Selection Guide

### When to Use DeepSeek Official

- When the New API gateway is unreachable (network isolation / VPN down)
- When minimum latency is needed (direct connection saves one forward through the gateway)
- When DeepSeek official releases a new model version and the gateway has not synced it yet
- When DeepSeek official-exclusive features are needed (e.g., `deepseek-reasoner`'s detailed reasoning trace)

### When to Use New API Gateway

- Everyday development (default path)
- When automatic multi-provider fallback is needed (a timed-out model automatically switches to the next)
- When non-DeepSeek model capabilities are needed (e.g., Doubao's vision, Kimi's long context)
- When the gateway already has the optimal model mapping configured; no need to care about the specific provider

### Priority Order

```
New API gateway → automatic fallback → DeepSeek official direct
```

OpenCode's `runtime_fallback` mechanism handles the above logic automatically:
1. Try the New API gateway alias first (e.g., `new-api/premium`)
2. On failure, try the New API fallback models (`new-api/premium-1`, `new-api/premium-2`)
3. After all fail, manually switch the config to a DeepSeek official model ID

## Fallback Chain

### Agent-Level Fallback

Each agent defines 3 fallback levels in `oh-my-openagent.jsonc`:

```jsonc
"oracle": {
  "model": "new-api/premium-max",       // primary
  "fallback_models": [
    "new-api/premium-max-1",            // Fallback 1
    "new-api/premium-max-2"             // Fallback 2
  ]
}
```

### Runtime Fallback

The global `runtime_fallback` config handles network-level errors:

| Config key | Value | Description |
|--------|-----|------|
| retry_on_errors | 402, 429, 500, 502, 503, 504 | HTTP status codes that trigger fallback |
| max_fallback_attempts | 2 | At most 2 fallback attempts |
| cooldown_seconds | 60 | 60s cooldown after a failure |
| timeout_seconds | 60 | Per-request timeout |

### Fallback Data Flow

```
User request
  │
  ▼
Agent primary model (new-api/premium)
  │  ├─ success → return result
  │  └─ failure → retry (up to 2 times)
  │       │
  │       ▼
  │   Fallback 1 (new-api/premium-1)
  │     ├─ success → return result
  │     └─ failure → retry
  │          │
  │          ▼
  │      Fallback 2 (new-api/premium-2)
  │        ├─ success → return result
  │        └─ all failed → gateway unavailable, switch to DeepSeek official
  │             │
  │             ▼
  │         DeepSeek official (deepseek-chat / deepseek-reasoner)
  │           └─ success/failure → return result or error
```

## Adding a New Model

1. Confirm model availability with DeepSeek official and get the API endpoint
2. Register a new model channel in the New API gateway and create an alias mapping
3. Update the mapping tables in this document
4. No OMO config changes needed (aliases are referenced)

## Notes

- DeepSeek official model IDs may change with official updates; follow the `api.deepseek.com` docs
- The New API alias mapping is maintained by the gateway admin; project members don't need to care about the concrete mapping
- When switching to DeepSeek official direct, ensure the API key is configured in environment variables
- Prefer New API for vision tasks (Doubao vision beats DeepSeek official)
- Using the lite tier for low-cost tasks significantly reduces token consumption
