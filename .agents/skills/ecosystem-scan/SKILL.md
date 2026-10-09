---
name: ecosystem-scan
description: "Audit the .agents/ system and scan the community ecosystem for adoptable skills/rules/MCP. Two-tier (Quick/Full) + quality scoring + security gate. Use when the user asks to 'optimize agents', 'scan ecosystem', 'find new skills', 'audit .agents/', or '/ecosystem-scan'."
---

# ecosystem-scan — Project Agent System Audit + Ecosystem Scan

> Two-tier scanning (Quick daily / Full deep), quality scoring, security audit gate.
> Reference precedents: [autoskills](https://github.com/B143KC47/autoskills), [agent-skill-discovery](https://github.com/ericgandrade/claude-superskills), [skill-update-team](https://github.com/franktsai2008-eng/skill-update-team), [agent-self-audit](https://github.com/Xxt-XN/agent-self-audit).

## Trigger Conditions

- User says "optimize agents" / "audit .agents" / "scan ecosystem" / "find new skills"
- `/ecosystem-scan` — opens mode selection menu
- `/ecosystem-scan quick` — quick scan (default)
- `/ecosystem-scan full` — deep scan (auto-upgrades on first run or after >=5 Quick scans)
- `/ecosystem-scan report` — view last scan report

## Entry Menu

Opens when no arguments are provided:

```
[1] Quick Scan — 8-item quick audit (< 30s)
[2] Full Scan — 15-item deep audit + community sync + security gate (3-5min)
[3] View Report — view last scan results
[4] Quick Fix — auto-fix LOW/MEDIUM issues (non-interactive)
```

## Two-Tier Modes

| Feature | Quick | Full |
|------|:-----:|:----:|
| Local audit | 8 quick checks | 15 deep checks |
| External scan | 3-5 known high-star repos | Comprehensive websearch + GitHub search |
| Scoring | 3-dimension quick | 5-dimension full scoring |
| Security audit | None | 6-item security check |
| Duration | < 30s | 3-5min |
| Trigger | `/ecosystem-scan` | `/ecosystem-scan full` or >=5 Quick scans |

---

## Phase 1: Quick Scan (default)

### 1A: Local Quick Audit (8 items)

```
1. opencode.json instructions files all exist
2. Each SKILL.md has name + description frontmatter
3. conventions/decisions/pitfalls numbering is sequential
4. memorys cross-reference completeness
5. Duplicate content detection (grep key passages)
6. Orphan file detection (exists but unreferenced)
7. Skill directory table sync: AGENTS.md SKILL DIRECTORY covers all skills
8. Task routing table sync: context-engineering routing table matches skill inventory
```

### 1B: External Quick Scan

Searches 3-5 known high-star repos:
- VoltAgent/awesome-agent-skills (index)
- addyosmani/agent-skills (engineering skills)
- ECC/affaan-m everything-claude-code (full-stack config)

Uses `webfetch` to fetch READMEs, 3-dimension quick scoring: tech stack match / project already has / quality.

### 1C: Quick Output

```
🟢 No issues — 3 items passed, no deep scan needed
🟡 Found N items worth attention — recommend /ecosystem-scan full
```

---

## Phase 2: Full Scan

### 2A: Local Deep Audit (15 items)

| # | Check Item | Quick | Full |
|---|--------|:---:|:---:|
| 1 | Config file health | Count | 5-dimension scoring + compress/split suggestions |
| 2 | Skill inventory | Count | Duplicate detection + community comparison |
| 3 | Security | Plaintext secrets | Permission audit |
| 4 | Memory system | Count | Staleness + structure |
| 5 | Rule quality | — | Runnable command check |
| 6 | Update availability | — | Changelog + priority |
| 7 | Skill utilization | — | Usage rate vs install count |
| 8 | Orphan recovery | — | Recovery candidates |
| 9 | Agent audit quality | — | Compliance spot-check |
| 10 | Environment | 4 atomic checks | Toolchain + packages + network |
| 11 | Cross-references | — | Dead link detection |
| 12 | Duplicate rules | — | Semantic dedup |
| 13 | Community trends | — | Market scan with 24h cache |
| 14 | Skill directory table | Count | AGENTS.md SKILL DIRECTORY vs actual skills consistency |
| 15 | Task routing table | — | context-engineering routing table vs actual skills consistency |

### 2B: External Deep Scan

#### Search Strategy (4-way parallel)

1. **Known repos**: anthropics/skills, addyosmani/agent-skills, VoltAgent/awesome-agent-skills, ECC/affaan-m, superpowers
2. **High-star discovery**: `websearch: "github opencode skills popular stars"`
3. **Tech-stack specialization**: `websearch: "best AI agent skills for <tech-stack> github"`
4. **MCP search**: GitHub MCP server repos (Rust, cargo, git, docker)

#### Scoring System (5 dimensions, max 10)

| Dimension | Weight | Scoring Criteria |
|------|:---:|------|
| **Fit** | 0.30 | Tech stack match (Rust/TS/DevOps/Web) |
| **Trust** | 0.20 | Repo star count + owner reputation + LICENSE |
| **Track-record** | 0.20 | Actual usage verification (not auto-generated) |
| **Freshness** | 0.15 | Last update time (>180 days without update penalized) |
| **Specificity** | 0.15 | Content specificity vs generic |

**Sanity Gate**: Any Trust < 2 or unreadable content → discard immediately.

#### Security Audit Gate (Full mode, before installation)

| Check Item | Severity | Content |
|--------|:---:|------|
| repo-trust | **block** | Stars, owner reputation, LICENSE, not archived |
| code-review | **block** | No `curl | sh`, no `eval()`, no unauthorized file access |
| permissions-scope | **block** | No global filesystem access, no sudo |
| dependency-audit | warn | Dependency audit, CVE check |
| data-exfil | **block** | No unauthorized data transfer |
| freshness | warn | Recent commit < 180 days |

- Any **block** → reject
- Any **warn** → warning + requires confirmation
- All pass → SAFE

---

## Phase 3: Comprehensive Recommendations

### 3A: Cross-comparison

- Multiple sources recommend the same item → +2 points
- Tech stack mismatch but pattern is portable → mark as "rewrite to adapt"

### 3B: Output Format

```markdown
## Ecosystem Scan Report — {date}

### 🟢 Phase 1: Quick (N items passed)
### 🔴 Phase 2: Full Local Audit (M items found)
### 🟡 Phase 2: External Scan (K recommendations)
### ❌ Rejected (L items)

#### P1: Strongly Recommended
| # | Content | Source | Score | Security | Effort |
|---|------|------|:---:|:---:|:---:|
| 1 | ... | repo | 9/10 | SAFE | Low |

#### P2: Worth Considering
| # | Content | Source | Score | Security | Effort |
|---|------|------|:---:|:---:|:---:|

#### Rejected
| Content | Reason | Security |
|------|------|:---:|
```

### 3C: Persistence

Quick scan results recorded to `.agents/memorys/`:
- Findings, scores, and decisions from each scan
- Next scan prioritizes checking whether previous issues were fixed
- >=5 Quick scans → auto-suggest Full

---

## Team Mode Configuration

```
Member 1: structure-analyst (deep) — Phase 2A structure audit (2-6 items parallel)
Member 2: content-auditor (deep) — Phase 2A content audit (7-13 items parallel)
Member 3: ecosystem-scanner (deep) — Phase 2B external 4-way parallel
Member 4: security-auditor (deep) — Phase 2B security audit (Full mode)
Member 5: synthesizer (deep) — Phase 3 synthesis (waits for 1-4)
```

---

## Community References

This skill's overall design draws from the following community precedents:

| Precedent | Patterns Adopted |
|------|-----------|
| [autoskills](https://github.com/B143KC47/autoskills) | 5-dimension scoring system, Sanity Gate, persistent memory |
| [agent-skill-discovery](https://github.com/ericgandrade/claude-superskills) | Two-tier scope (installed / repo), platform detection |
| [skill-update-team](https://github.com/franktsai2008-eng/skill-update-team) | Security audit gate (6-item check), scoring weight allocation |
| [agent-self-audit](https://github.com/Xxt-XN/agent-self-audit) | Quick/Full two-tier design, 13-item check, auto-upgrade |
| [claude-ecosystem](https://github.com/melodic-software/claude-code-plugins) | Meta-skill architecture, 16 audit agents, component health |
| [skill-optimizer](https://github.com/hqhq1025/skill-optimizer) | Skill lifecycle (miner → personalizer → generalizer) |
| [large-codebase-audit](https://github.com/MJWNA/large-codebase-audit-skill) | 9-surface AI layer audit, aligned with Anthropic best practices |

---

## Adapt to Any Project

Replace variables:
```
- Tech stack: Rust + TypeScript + Docker + React → {your tech stack}
- Agent platform: OpenCode → {your platform}
- Rules path: .agents/ → {your rules path}
- Package manager: pixi → {your package manager}
- Memory path: .agents/memorys/ → {your memory path}
```
