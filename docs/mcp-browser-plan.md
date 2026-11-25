# MCP local-playwright out-of-the-box plan

> STATUS: PLAN OF RECORD (revision 2), 2026-11-25. History: rev 1 written
> 2026-11-25 after a five-point adjudication walkthrough (user-ruled one by
> one); rev 1 then passed a four-member expert panel (correctness /
> evidence-devil's-advocate / security / operations, independent contexts,
> frozen findings); rev 2 folds every no-contest panel fix (F1-F10 set)
> PLUS four follow-up adjudications the panel disagreement surfaced
> (user-ruled one by one, 2026-11-25). No execution before this plan is
> closed; the D-entry lands with execution. Nothing below marked TODO is
> silent - every open edge is named.

Goal: `pixi run mcp setup` yields a WORKING `local-playwright` browser on
every host this repo touches - including hosts the pinned Playwright no
longer serves (Ubuntu 20.04 / glibc 2.31 is the live case).

## 0. Problem statement (measured, not assumed)

`local-playwright` (`@playwright/mcp@0.0.82`, pinned in
`scripts/agents/mcp-servers.json`) launches a browser on first tool call.
Facts measured on this box (evidence base stated per finding):

1. The pinned Playwright (1.64.0-alpha) **refuses to install its managed
   chromium** on Ubuntu 20.04: `install-browser chromium` fails with
   `Playwright does not support chromium on ubuntu20.04-x64`. Firefox:
   same. Escape via older pins is **sampled-closed**: versions 0.0.10 /
   0.0.14 / 0.0.20 / 0.0.25 / 0.0.30 / 0.0.40 / 0.0.60 / 0.0.82 were
   queried (npm registry, live) and every sampled one maps to pw >= 1.52;
   pw 1.52 itself measured refusing 20.04; pw 1.49.1 unprobed (probe
   failed on environment interference, not on the question). The
   "auto-degrade the pin" option is closed on this sample, not on a full
   enumeration - rev 1's "every public version" wording is hereby
   corrected to "every sampled version".
2. The host has NO system browser (no chrome/chromium/edge binary on THIS
   box; typical 20.04 desktops often have snap chromium - rung 3 will find
   those when present).
3. conda-forge has no chromium package **per the current anaconda.org API
   query** (404, measured once) - the pixi lane cannot supply a browser
   through it today.
4. The Chrome-for-Testing build the pin wants is downloadable DIRECTLY
   from the CfT distribution (`https://storage.googleapis.com/chrome-for-testing-public/<version>/<plat>/chrome-<plat>.zip`),
   where `<version>` = the `browserVersion` field in the installed
   `playwright-core/browsers.json` (154.0.8037.0 for pin 0.0.82) and
   `<plat>` = the host segment. Measured on this box (linux64): download
   -> `--version` OK -> headless `--dump-dom` OK -> a full MCP
   `browser_navigate` tool call succeeds end-to-end through the CLI
   (`--executable-path` + `--headless`). Version-matched compatibility
   proven; "same build as Playwright's package" is INFERRED from the
   `browsers.json` title ("Chrome for Testing") + equal version string +
   the working probe, not hash-compared.
5. The pinned server's browser defaults: Linux headless-without-DISPLAY
   is already the server's own default for X11-less checks (source:
   `playwright-core/lib/coreBundle.js` envToBoolean/headless default
   logic); the launcher rule makes it explicit and covers Wayland-only
   and mac/win display-less shapes.

The gap: setup installs the server binary but nothing guarantees a usable
browser at the far end. The fix: a browser-resolution ladder in
`mcp setup` + env-driven handoff at launch.

## 1. Adjudicated rulings

### 1.1 Original five (rev 1 walkthrough, 2026-11-25)

| # | Point | Ruling | Key note |
|---|---|---|---|
| 1 | Fallback order when the official source refuses the host | **CfT direct download first, system browser as the offline last rung** | version-exact beats download-cost; rung 1 (official managed build) already covers supported hosts |
| 2 | How the chosen browser reaches the MCP server | **env-driven** - `PLAYWRIGHT_MCP_BROWSER` / `PLAYWRIGHT_MCP_EXECUTABLE_PATH` / `PLAYWRIGHT_MCP_HEADLESS`, all official envs of `@playwright/mcp` | spec `args` removed - CLI flags outrank envs and would block rungs 2/3; spec keeps only the declarative `"browser": "chromium"` field |
| 3 | Where the resolution record lives | **`out/mcp-browser.json`** (repo-local, gitignored via `/out/`) | short-lived data belongs where rebuild regenerates it; `node_modules/` is npm's territory; user-cache keys add indirection |
| 4 | Headless on display-less hosts | **auto-detect + override** - no `DISPLAY` and no `WAYLAND_DISPLAY` => `PLAYWRIGHT_MCP_HEADLESS=1`; `MODUKIT_MCP_FORCE_HEADLESS=0` forces headed | auto is required for out-of-the-box; the escape hatch catches Xvfb/VNC |
| 5 | Where "prefer the latest version" lands | **a new `mcp latest` subcommand** - networked, explicit, prints registry latest vs pin + the exact `mcp-servers.json` line to edit | `mcp status` keeps its written contract (read-only, always exit 0); setup NEVER floats the pin (fail-fast drift discipline) |

### 1.2 Panel follow-up four (rev 2 walkthrough, 2026-11-25)

| # | Point | Ruling | Key note |
|---|---|---|---|
| 6 | rung 2 platform coverage | **full platform segments** (`linux64` / `mac-arm64` / `mac64` / `win64`) | coverage wins over minimalism; HONESTY BOUNDARY: mac/win segments are URL-pattern-derived, only linux64 is live-measured - §4 marks them unverifiable on this box |
| 7 | gate-mcp grows a browser leg? | **YES, read-only yellow** - resolution record missing or all-rungs-failed => SKIP-yellow + `pixi run mcp setup` hint, same convention as missing npm packages | the plan's core claim gets a daily-observed invariant; the gate never downloads/launches browsers |
| 8 | CfT zip integrity | **runtime hash verify from Google's official JSON endpoint** (no in-repo hash ledger) | same-channel official hash closes the corrupted-package / bad-mirror class; an in-repo hand-copied hash would be a new drift source (family lesson: "any version string that is not generated is a drift source") |
| 9 | the downloaded zip's lifetime | **keep, paired with its unpacked dir under one GC rule** - `cft-<version>.zip` + `cft-<version>/` die together when the version stops matching the pin | reinstall (pin rollback, cache rebuild) stays zero-download; GC keeps disk bounded; deletion tolerates "in-use, skip" |

## 2. The resolution ladder (as ruled)

```
mcp setup, local-playwright, browser resolution:
  rung 1  playwright managed:  node <pkg>/cli.js install-browser chromium
          -> ok (Ubuntu 22.04+/mac/win):  mode=playwright, env PLAYWRIGHT_MCP_BROWSER=chromium
  rung 2  CfT direct (rung 1 refused the host):
          version = playwright-core/browsers.json chromium.browserVersion (rides the pin)
          plat    = linux64 | mac-arm64 | mac64 | win64 (per host; ONLY linux64 live-measured)
          GET chrome-for-testing-public/<version>/<plat>/chrome-<plat>.zip (~200MB)
          + fetch the official CfT known-good JSON; verify sha256 of the zip before unpack
          unpack into ~/.cache/modukit-mcp/cft-<version>/  (user-level; NEVER inside the repo tree)
          zip kept at ~/.cache/modukit-mcp/cft-<version>.zip (paired for zero-download reinstall)
          -> mode=cft, env PLAYWRIGHT_MCP_EXECUTABLE_PATH=<...>/chrome[.exe]
  rung 3  system browser (rung 2 failed: offline):
          probe google-chrome / chromium / chromium-browser / msedge / snap chromium;
          each candidate must PASS a live `--version` exec; resolved to an ABSOLUTE path only
          -> mode=system, env PLAYWRIGHT_MCP_EXECUTABLE_PATH=<abs path>, status carries the drift warning
  all fail -> WARN (MODUKIT_MCP_STRICT=1 turns it red) + manual guidance
```

Handoff contract: `mcp setup` writes the outcome ATOMICALLY
(`os.replace` tmp+rename) to `out/mcp-browser.json` (mode, executable
path, browser family, resolved version, record version). `mcp-serve.mjs`
reads it and exports the envs above before spawning the server bin. A
missing record does NOT re-run rung 1 semantics on a host that needs
rung 2/3 - the launcher's record-missing branch points straight at
`pixi run mcp setup` (never a silent wrong-browser launch, and never the
wrong recovery hint). On clean clones / fresh-22.04 hosts a missing
record equals default (rung-1-capable) behavior with the setup hint.

Invariants the implementation must keep (rev 2 additions in **bold**):

- The pin never floats. `mcp latest` is the ONLY surface that talks to the
  registry, and only to report.
- The repo tree stays artifact-free: CfT lives under
  `~/.cache/modukit-mcp/`; the server's scratch output is redirected to
  `out/mcp/` by the launcher with an ABSOLUTE path (spawn sets no cwd).
- `mcp status` stays local-only, exit 0 (file-header contract).
- The launcher browser gate runs AFTER the npm package step (clean clone:
  cli.js must exist before `install-browser` can run).
- **The resolution record is an authenticated-by-convention executable
  pointer: anyone who can write the repo's `out/` (or the user cache)
  can redirect the agent's browser to an arbitrary binary. Trust base =
  the single-developer trusted tree; recorded here as an accepted
  residual risk, not a silent one.**
- **The browser-execution path never passes `--no-sandbox`; sandbox
  flags are a user decision made outside this plan.**
- **GC: after a successful rung-2 record, `cft-*` dirs and zips whose
  version != the pin's `browserVersion` are removed (in-use => skip
  quietly, retry next setup).**
- **Rung 2 failure classification: `does not support` (host-refused,
  eligible for rung 2) vs download/verify failure (offline or bad
  package, eligible for rung 3) is read from the installer's stderr
  detail, not from a uniform "warn" return value.**

## 3. Work items (execution waves)

### W1 - spec + record + gate leg (mechanical)

- `scripts/agents/mcp-servers.json`: REMOVE `"args": ["--browser","chromium"]`;
  KEEP `"browser": "chromium"` (declarative; drives rung 1 vocabulary).
- `mcp-cli.py`: `setup` writes `out/mcp-browser.json` ATOMICALLY
  (schema only in W1; content-producing ladder logic is W2 - the wave
  boundary is stated so W1 alone does not claim a filled record);
  `status` prints the resolution line (local read, contract intact).
- `gate-mcp.py`: new read-only browser-resolution leg - record missing or
  all-rungs-failed => SKIP-yellow + hint (same convention as missing npm
  packages, `gate-mcp.py` header). The gate never downloads or launches
  a browser. Rung-1 hosts with a valid record: green.
- NOT deferred: W1 wires the gate leg so the plan's core claim is
  observed from the first wave on.

### W2 - the ladder (core, ~200-250 lines in mcp-cli.py)

- rung 1 wrapper reuses the 2026-11-25 `_install_browser` scaffold
  (landed scaffolding, to be reclassed per the failure-classification
  invariant above).
- rung 2: read `browsers.json` -> resolve `<plat>` per host -> download
  with size-guard AND timeout VALUES NAMED (TODO W2: pick, e.g. 600s /
  350MB) -> fetch CfT known-good JSON, sha256-verify the zip -> unpack
  (`ZipFile.extract`, which filters traversal components) -> verify the
  binary executes `--version` -> atomic record write -> GC pass.
  Standard library only (`urllib.request`, `zipfile`, `hashlib`) - no new
  deps. TODO W2 (live probe): confirm the exact known-good JSON endpoint
  URL and its per-file hash field shape.
- rung 3: PATH probe list + live `--version` exec per candidate,
  absolute-path resolution only, drift warning recorded.
- Failure modes: WARN + guidance; `MODUKIT_MCP_STRICT=1` red.
- Cache reuse fast path: an existing `cft-<version>/` (or its zip) for
  the CURRENT pin version short-circuits the download (offline-but-
  previously-configured hosts resolve here).

### W3 - launcher env handoff (~40 lines in mcp-serve.mjs)

- Read `out/mcp-browser.json` (torn-read tolerant: unreadable record =>
  the setup hint, never a crash on malformed JSON).
- Export per mode: `PLAYWRIGHT_MCP_BROWSER` (rung-1 record) or
  `PLAYWRIGHT_MCP_EXECUTABLE_PATH` (rung 2/3); in cft/system modes
  UNSET `PLAYWRIGHT_MCP_BROWSER` so a stale user env cannot stack a
  channel on top of the executable path.
- Headless rule: auto (`no DISPLAY && no WAYLAND_DISPLAY` -> `=1`), and
  `MODUKIT_MCP_FORCE_HEADLESS=0` forces headed by EXPLICITLY exporting
  `PLAYWRIGHT_MCP_HEADLESS=0` (the env parser treats an unset env and a
  false-valued one differently - unset lets the CLI default win).
- Append `--output-dir <repo>/out/mcp` (absolute path) to the server
  args; rung-2 record-missing branch points at `pixi run mcp setup`.
- Keep the browser gate AFTER the npm package install (ordering
  invariant).

### W4 - `mcp latest` subcommand (~30 lines)

- `npm view @playwright/mcp version` (short timeout; offline -> clear
  message, exit 0 with a note, NOT a red).
- Print: registry latest, current pin, lag, the exact JSON line to edit,
  the `pixi run mcp setup` refresh step. Note in output: pin-vs-registry
  drift reds already have a gate owner (`gate-mcp` start-mode probes);
  `mcp latest` is interactive-only by design.

### W5 - docs + write-back + acceptance

- `docs/env.md` MCP lifecycle section: ladder summary, envs, strict
  flag, resolution record path, `mcp latest`, the gate leg, the trust
  note, the GC note, the 200MB-per-version user-cache cost.
- D-entry in `.agents/memorys/decisions.md` (D112 - NINE adjudications
  total across the two walkthroughs, the ladder, the env handoff, the
  resolution-record home, hash policy, GC policy; dated
  "user-adjudicated one by one, 2026-11-25") + architecture.md §11 row
  + AGENTS.md/status.md high-water bump (D111 -> D112; numbering gate
  `scripts/check-numbering.sh` stays green).
- `pixi run agents sync` + `gate-agents`, `gate-mcp` (new leg green on
  this box after the rung-2 setup), and a live `pixi run mcp setup` on
  THIS box (rung 2 path) as the acceptance leg.
- Scripted tool-call probe (C0-closable): a small script drives the
  launched server (initialize -> initialized -> `browser_navigate
  about:blank`) and asserts a snapshot reference in the response; the
  probe command + assertion are named in §4.

## 4. Verification (what "done" means, per C0)

| Check | Expected |
|---|---|
| `pixi run mcp setup` on THIS box (Ubuntu 20.04) | rung 2 fires: hash-verified CfT downloaded to `~/.cache/modukit-mcp/`, zip+dir paired, `out/mcp-browser.json` atomic-written, exit 0 |
| Scripted MCP tool call: `bash scripts/mcp-browser-probe.sh` (W5) - JSON-RPC navigate `about:blank` through the launched server | response contains a page snapshot reference; exit 0 |
| `pixi run mcp setup` on a 22.04+ host (rung 1) | rung 1 path unchanged, zero new download beyond the managed build; **host availability: none in-repo (CI is a standby mirror) - this row is environment-dependent, marked UNVERIFIABLE HERE** |
| rung 2 platform segments mac-arm64 / mac64 / win64 | **URL-pattern-derived only; live-measured on linux64 ONLY - UNVERIFIABLE on this box; first real mac/win refusal is the live-fire trigger** |
| `pixi run mcp status` | resolution line present; local-only, exit 0 |
| `mcp latest` offline | offline note, exit 0 |
| Repo tree after server use | no `.playwright-mcp/` at root (output redirected to `out/mcp/` - absolute path) |
| `MODUKIT_MCP_STRICT=1` with all rungs failed | red, exit 1 |
| `gate-mcp` browser leg | yellow on missing/failed record with setup hint; green with a valid record; never downloads/launches |
| `gate-agents` / `check-numbering.sh` | green |

## 5. Risks / explicit non-goals / open items

- CfT download URL: Google's stable CfT distribution API (historically
  stable; a break degrades to WARN + guidance, not a hard setup failure).
  The known-good JSON endpoint URL is a W2 live-probe TODO (named above).
- Per-version user-cache cost: ~200MB zip + ~500MB unpacked while the
  version matches the pin; the GC rule bounds accumulation.
- System-browser rung carries the drift warning (ruling 1); it is the
  OFFLINE fallback, not the default on unsupported hosts.
- Residual risks RECORDED (not silent): the resolution-record trust
  boundary (single-dev trusted tree); the `out/mcp/` scratch dir
  accumulates page snapshots with browsed content (gitignored; user
  may clean); a pre-planted symlink in the user cache can redirect the
  unpack target (single-user box, recorded); the rung-2 URL segments for
  mac/win are pattern-derived.
- Non-goals: floating the pin at setup time (forbidden - drift
  discipline); using conda-forge to supply a browser (no package per
  current query); auto-degrading the MCP pin itself (sampled-closed,
  not enumerated - wording corrected from rev 1).
- Execution waves W1-W5 land after this plan is closed; the in-flight
  bundle-unification waves (D111) are a separate track and unaffected.
- OPEN ITEMS (named, not swallowed): W2 must pick concrete size/timeout
  guard values; W2 must live-probe the CfT known-good JSON endpoint
  shape; the W5 probe script needs its name registered in §4 upon
  landing; the two UNVERIFIABLE rows above stay in the table until a
  mac/win/22.04 host materializes.
