# Reference Resolution — "sysplugin" (expected: Rust OSGi-like plugin framework)

> **Resolution profile, not a project profile.** This file documents the *existence check* for the
> whitepaper-adjacent citation "sysplugin", expected to name a Rust OSGi-like plugin framework.
> No such project was found; the expected referent does not exist under this name. The honest artifact
> is therefore the audit record itself, formatted per the series template (`uniffi-rs.md`, `rutis.md`).
> Research date: 2026-10-09 · Method: GitHub search API + crates.io search API + local-clone grep of this
> repository's canon. All facts below are verbatim API output unless marked **UNCERTAIN**.

## 1. Portrait — what the search actually returns

Query: `curl 'https://api.github.com/search/repositories?q=sysplugin&per_page=5'` (unauthenticated),
2026-10-09. **`total_count: 4`** — the complete GitHub repo population for the term:

| # | Repo | Language | Stars | Last push | Archived | Description (verbatim) |
|---|---|---|---|---|---|---|
| 1 | [Blurro/Blurros-Sysplugins](https://github.com/Blurro/Blurros-Sysplugins) | C | 7 | 2026-09-28 | no | "Sysplugins themselves - Uncapped Play Coins and more!" |
| 2 | [Blurro/3NX-Plugin-DevKit](https://github.com/Blurro/3NX-Plugin-DevKit) | Shell | 4 | 2026-09-29 | no | "Devkit to build 3NX Sysplugins to extend boot.firm features" |
| 3 | [Blurro/MENU-Sysplugin-3DS](https://github.com/Blurro/MENU-Sysplugin-3DS) | C | 2 | 2026-09-27 | no | "Mod Menu inspired Sysplugin Menu for Nexus3DS" |
| 4 | [Selenavia/obsidian-sysPlugin-ss](https://github.com/Selenavia/obsidian-sysPlugin-ss) | TypeScript | 0 | 2026-08-16 | no | *(no description)* |

Corroborating query: `https://crates.io/api/v1/crates?q=sysplugin` → **0 crates** (2026-10-09).
The GitHub repos-API re-check path was rate-limited on the audit day (same limit noted in
`rutis.md` §1); the search endpoint above was not.

**Match against the description we expected:** a Rust OSGi-like plugin framework — 0 of 4 hits.
Zero hits are Rust. Zero hits are a plugin *framework* of any kind. Two are single-author Nintendo
3DS homebrew modification repos; one is a 3DS menu front-end; one is an unpublished Obsidian editor
plugin with no description. Star ceiling: 7.

**Verdict: NOT-FOUND-AS-DESCRIBED.** (Series verdict labels — DEPENDENCY-CANDIDATE / BORROW-PATTERNS /
REFERENCE-ONLY — all presuppose a real project; none applies.)

## 2. Why the name resolves where it does

"sysplugin" is an established **proper noun of the Nintendo 3DS custom-firmware ecosystem**: a
system-mode plugin loaded by a CFM payload (the hits' `boot.firm` refers to Luma3DS's firmware
payload). The term was coined there years before any Rust plugin framework could claim it, so GitHub
search returns that community's output, verbatim, as the entire population. A generic-sounding name
that is already a domain term in a large hobby ecosystem will *never* surface a Rust framework query —
the collision is absolute, not a ranking artifact.

Additionally: `grep -rIli sysplugin` over this repository's canon (`docs/whitepaper.md`, all of
`docs/reference/`, `.agents/`) — excluding git-ignored external clones — returns **zero matches**
(2026-10-09). The expected citation is not in the whitepaper, not in the reference README index, and
not in any memory file. Whatever planning context expected "a Rust OSGi-like plugin framework called
sysplugin", that expectation entered session context without a repo-recorded source. There is no
canon line to correct — only a gap to record so nobody re-queries it.

## 3. Key capabilities

Not assessable — there is no project to assess. The capabilities a Rust OSGi-like framework *would*
supply (lifecycle states, service registry, dynamic install) are already covered by profiled, live
references: `cppmicroservices.md` (semantics), `rutis.md` (the live Rust analog), `zenoh.md`
(zenoh-plugin-trait as a shipped dynamic-plugin ABI).

## 4. Development & current state

N/A for a nonexistent referent. The four real hits' activity is 3DS-homebrew maintenance
(pushed Aug–Sep 2026) and one dormant Obsidian repo; none is adjacent to ModuKit's design space.

## 5. Ecosystem

The only ecosystem attached to the string is the 3DS sysplugin scene (Blurro's three repos are
interlinked: devkit + sample plugins + menu). It has no plugin-framework abstractions worth
porting; 3DS sysplugins are fixed-ABI native payloads for one vendor's firmware — closer to our
anti-pattern list than our reference list.

## 6. Highlights & limitations

As a *name*: unusable. It is collision-occupied by another domain's jargon, carries zero Rust
credibility on search, and cannot be disambiguated by context in shared documents — anyone who
greps "sysplugin" gets Game-Boy-adjacent homebrew, not us. Any ModuKit coinage must be searched
**before** it enters canon, not after.

## 7. Historical lessons

1. **Unverified canon citations rot exactly like unverified code.** This is the second case produced
   by the same audit method that surfaced seven whitepaper names (`whitepaper-cited-unvendored.md`):
   rustbridge resolved to a deprecated teaching repo, event-engine/Parallax/Scarlet resolved to
   nothing, and sysplugin resolves to nothing-but-collisions — while the *method* also found rutis,
   the one live Stage-1-named project, now profiled (`rutis.md`). The asymmetry is the lesson: the
   cost of checking is one curl; the cost of not checking is designing against a phantom.
2. **A citation must carry its evidence line.** The audit's reproducibility note (name → repo →
   stars/pushed/license, snapshotted with a date) is the minimum viable citation format; adopt it
   (per audit recommendation 4) for every future reference, including ones added to planning notes
   in chat — sysplugin shows phantoms can enter the plan via session context, the least-audited
   channel.
3. **Search the name before claiming the name.** Domain jargon (3DS "sysplugin") out-ranks generic
   compounds in every search engine; if a planned project/crate name returns another ecosystem's
   proper nouns on page one, it is already taken in meaning regardless of trademark availability.
4. **Record NOT-FOUND as a finding, not as silence.** An empty verdict written down prevents a
   re-query by the next agent or maintainer; this file is the terminal state of the citation, not a
   placeholder awaiting discovery.

## 8. Value for ModuKit

**Verdict: NOT-FOUND-AS-DESCRIBED — zero design value; process value only.**

- **Canon action**: if "sysplugin" appears in any future whitepaper revision or planning record, mark
  it resolved (this file) and delete the expectation. No replacement needed — the Stage-1 slot it was
  expected to fill is covered by CppMicroServices + rutis (see `00-overview.md` and `rutis.md` §8).
- **License gate**: N/A — nothing to license-check. The method note from `whitepaper-cited-unvendored.md`
  applies: snapshot numbers (★ counts, pushed dates in §1) are 2026-10-09 values, not constants.
- **What remains UNCERTAIN**: whether the original expectation intended a *different real project*
  under a misremembered name (no candidate surfaced: the described shape — Rust + OSGi-like + plugin
  framework — resolves to the already-profiled zenoh-plugin-trait and the live rutis). If the source
  of the expectation is ever recovered, re-run §1's two queries against the corrected name.
