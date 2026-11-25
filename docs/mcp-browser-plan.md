# MCP local-playwright out-of-the-box plan

> STATUS: AS-BUILT (revision 4), 2026-11-25. History: rev 1 written after a
> five-point adjudication walkthrough; rev 1 passed a four-member expert panel;
> rev 2 folded the panel's no-contest fixes; rev 3 shipped after a three-member
> re-review. **Rev 4 records what the implementation measured** - three of the
> nine rulings rested on mechanisms that do not exist, and the as-built design
> is SMALLER because of it. Execution waves W1-W5 LANDED 2026-11-25; the
> D-entry is D112.

Goal (met): `pixi run mcp setup` yields a WORKING `local-playwright` browser on
this repo's hosts, including hosts the pinned Playwright no longer serves
(Ubuntu 20.04 / glibc 2.31 - the live case).

## 0. What actually happened (measured during W1-W5)

1. **Chrome-for-Testing publishes NO sha256.** The known-good JSON
   (`googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json`,
   5.4 MB, live) carries `url` per platform and nothing else;
   `chrome-linux64.zip.sha256` and `.sha256.txt` both 404. Ruling 8's
   "official hash source" therefore does not exist.
2. **Playwright already ships the fallback build.** With
   `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu22.04-x64`, the pinned installer
   succeeds on 20.04 and prints its own line: `BEWARE: your OS is not
   officially supported by Playwright; downloading fallback build for
   ubuntu22.04-x64`. It downloads the SAME version the pin wants
   (154.0.8037.0 / chromium-1246) into the standard cache
   (`~/.cache/ms-playwright/`). This is a supported knob, not a hack.
3. **The fallback build runs.** `browser_navigate about:blank` through the
   launched server returns a real page snapshot. Verified twice: directly, and
   then through the full launcher path (`bash scripts/mcp-browser-probe.sh`).
4. **Consequence:** the self-built download path (platform URL table, zip
   pairing, GC, hash verify) is unnecessary. Rulings 6/8/9 were made against a
   mechanism that does not exist; their INTENT is preserved by rung 1:
   version-exact (the pin's own build), integrity (the pin's own installer and
   its own CDN, nothing hand-fetched), bounded cache (Playwright manages its
   own `ms-playwright` cache). Recorded here rather than silently dropped.
5. **A pre-existing trap surfaced:** npm bin shims start with
   `#!/usr/bin/env node`, and this box's PATH resolves a v10 system node. The
   launcher now prepends its own node directory to the child PATH.

## 1. Rulings, and what each became

| # | Point | Ruling | As built |
|---|---|---|---|
| 1 | Fallback order | version-exact before system browsers | rung 1 = the pin's own build (via its fallback when the host is dropped); rung 2 = system browser |
| 2 | Browser handoff | env-driven, never spec args | done: spec keeps `browser` only; envs exported by the launcher |
| 3 | Resolution record | `out/mcp-browser.json` | done, atomic (`os.replace`), `record_version: 1` |
| 4 | Headless | auto + `MODUKIT_MCP_FORCE_HEADLESS` | done: `--headless` when no DISPLAY/WAYLAND_DISPLAY; `=0` forces headed |
| 5 | "Prefer latest" | `mcp latest`, report only | done: prints pin vs registry, names the line to edit, never floats |
| 6 | rung-2 platform coverage | full CfT platform segments | **collapsed**: no self-built download; rung 1 covers every host the pin installs for, fallback included |
| 7 | gate-mcp browser leg | read-only SKIP-yellow | done: `gate-mcp` reads the record, never launches |
| 8 | zip integrity | runtime official sha256 | **collapsed**: no official hash exists; rung 1 never hand-fetches |
| 9 | zip lifetime / GC | zip+dir paired under one GC rule | **collapsed**: no zip of our own; Playwright owns its cache |

## 2. The ladder as built

```
mcp setup, local-playwright:
  rung 1  Playwright's own installer:
            node <pkg>/cli.js install-browser chromium
            + PLAYWRIGHT_HOST_PLATFORM_OVERRIDE (default ubuntu22.04-x64 on linux,
              overridable with MODUKIT_MCP_PLATFORM_OVERRIDE) so a dropped host
              still gets the pin's build
          -> record {mode: "playwright", browser, version}
  rung 2  a live system browser on PATH (absolute realpath + `--version` proof)
          -> record {mode: "system", executable}; a drift NOTE is printed because
             system browsers auto-update and Playwright only guarantees its own builds
  rung 3  nothing usable -> WARN (`MODUKIT_MCP_STRICT=1` reds it)
```

Handoff: the record is read by the launcher, which exports
`PLAYWRIGHT_MCP_BROWSER` (playwright mode) or `PLAYWRIGHT_MCP_EXECUTABLE_PATH`
(system mode) and **unsets the other one symmetric** - a stale shell value must
never override the record's intent. No record + no managed build => the
launcher points at `pixi run mcp setup` instead of retrying rung 1 (retrying is
what produced the misleading "run on a supported OS" hint before).

## 3. Waves as landed

- **W1** spec `args` removed (`browser` kept), atomic record writer, `status`
  resolution line, `gate-mcp` browser leg (separate counter, never downloads).
- **W2** the ladder (`_install_browser` rewritten; `_platform_fallback`,
  `_system_browser`, `_playwright_browser_version`, `_resolution_for` added).
- **W3** launcher: `readRecord` / `exportedEnv` / `runtimeArgs`; PATH fix;
  `--output-dir <repo>/out/mcp` (absolute); `--headless` rule; the old
  "browser gate" block deleted (W3 owns resolution now).
- **W4** `mcp latest [server]`; `pixi run mcp latest`.
- **W5** `scripts/mcp-browser-probe.sh` (task `mcp-probe`), `docs/env.md`,
  D112 + architecture §11 + high-water bumps.

## 4. Verification (observed, not planned)

| Check | Result |
|---|---|
| `pixi run mcp setup` on this box (Ubuntu 20.04) | PASS: rung 1 via fallback, record written, exit 0 |
| `bash scripts/mcp-browser-probe.sh` | PASS: `PROBE ✓ local-playwright opened about:blank (snapshot captured)` |
| `pixi run mcp status` | PASS: `browser=chromium playwright v154.0.8037.0` |
| `pixi run gate-mcp` | PASS: `(3 green, 0 skipped, 0 skipped-env, 0 browser unresolved)` |
| `pixi run mcp latest` | PASS: reports pin vs registry for both npm servers |
| 22.04+/mac/win rung 1 | environment-dependent, unchanged path (still UNVERIFIABLE here) |
| `MODUKIT_MCP_STRICT=1` with no browser | path retained from W1 (red on unresolved) |

## 5. Residual risks / open items

- The resolution record is an executable pointer: anyone who can write `out/`
  redirects the agent's browser. Trust base = the single-developer trusted tree.
  Recorded, not silent.
- The platform override is a documented-by-behaviour knob (Playwright prints
  `BEWARE:`), not a contractual API: a future Playwright may remove it. If it
  does, the self-built download path (rulings 6/8/9, rev 3 §2) returns and
  would need a hash source that currently does not exist.
- `MODUKIT_MCP_PLATFORM_OVERRIDE` defaults to `ubuntu22.04-x64` on linux; a host
  that Playwright still serves never needs it (the installer succeeds unaided).
- `out/mcp/` (server scratch) and `out/mcp-probe/` accumulate page snapshots with
  browsed content; gitignored via `/out/`.
- Non-goals unchanged: the pin never floats at setup; `mcp latest` is the only
  registry-facing surface.