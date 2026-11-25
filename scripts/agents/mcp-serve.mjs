#!/usr/bin/env node
// mcp-serve.mjs — pinned npm-MCP launcher for the agent single-source (pixi tasks call this).
//
// Why: review #16 forbids `npx -y <pkg>` (floating version, unreproducible). This launcher
// installs <pkg>@<version> ONCE into ./node_modules (gitignored — same dir the legacy
// .opencode/init-mcp-*.mjs wrappers used) and then spawns the bin with the given args.
// Verified package sources (2026-10-28): codegraph = npm `@colbymchenry/codegraph`
// (NOT pypi, contrary to the first plan sketch — corrected per review #17).
//
// Usage:
//   node scripts/agents/mcp-serve.mjs --pkg <npm-pkg> --version <ver> --bin <bin> -- <server-args...>
//   node scripts/agents/mcp-serve.mjs --from scripts/agents/mcp-servers.json --name <server-name>
// The --from/--name form is the SINGLE SOURCE for launcher specs (shared by the pixi
// tasks AND scripts/gate-mcp.py / mcp-bootstrap.py), so the pinned version lives in one file.
//   --pkg     npm package name (e.g. @colbymchenry/codegraph)
//   --version pinned semver (e.g. 1.6.0) — MUST be confirmed via `npm view <pkg> version`
//   --bin     executable name inside node_modules/.bin (e.g. codegraph)
//   --from    path to scripts/agents/mcp-servers.json
//   --name    server key inside that json
//   --        separator; everything after is passed verbatim to the server bin
import { spawn, spawnSync } from 'node:child_process'
import { existsSync, readFileSync, readdirSync } from 'node:fs'
import { homedir } from 'node:os'
import { join, dirname } from 'node:path'
import { fileURLToPath } from 'node:url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const root = join(__dirname, '..', '..')  // repo root (this script lives in scripts/agents/)

const argv = process.argv.slice(2)
const sep = argv.indexOf('--')
const pre = sep === -1 ? argv : argv.slice(0, sep)
const serverArgs = sep === -1 ? [] : argv.slice(sep + 1)

function need(name, val) {
  if (!val) { console.error(`[mcp-serve] missing required --${name}`); process.exit(2) }
  return val
}

let pkg, version, bin, from, name, spec = null
for (let i = 0; i < pre.length; i++) {
  if (pre[i] === '--pkg') pkg = need('pkg', pre[++i])
  else if (pre[i] === '--version') version = need('version', pre[++i])
  else if (pre[i] === '--bin') bin = need('bin', pre[++i])
  else if (pre[i] === '--from') from = pre[++i]
  else if (pre[i] === '--name') name = pre[++i]
  else { console.error(`[mcp-serve] unknown arg: ${pre[i]}`); process.exit(2) }
}

// single-source mode: load launcher spec from scripts/agents/mcp-servers.json
if (from) {
  spec = JSON.parse(readFileSync(join(root, from), 'utf8'))[need('name', name)]
  if (!spec) { console.error(`[mcp-serve] no server "${name}" in ${from}`); process.exit(2) }
  pkg = spec.pkg
  version = spec.version
  bin = spec.bin
  if (Array.isArray(spec.args)) serverArgs.push(...spec.args)
}
need('pkg', pkg); need('version', version); need('bin', bin)

// Browser bootstrap: a spec may declare `browser` (chromium/firefox/webkit) so the
// server runs Playwright's own downloaded build instead of a system Chrome. The
// cache path mirrors playwright-core; mcp-cli.py implements the same rules so a
// fresh clone works from either entry point (setup or first host spawn).
function playwrightBrowsersDir() {
  const override = process.env.PLAYWRIGHT_BROWSERS_PATH
  if (override && override !== '0') return override
  const home = homedir()
  if (process.platform === 'win32') return join(process.env.LOCALAPPDATA || join(home, 'AppData', 'Local'), 'ms-playwright')
  if (process.platform === 'darwin') return join(home, 'Library', 'Caches', 'ms-playwright')
  return join(home, '.cache', 'ms-playwright')
}

function browserInstalled(family) {
  const prefix = family === 'chromium-headless-shell' ? 'chromium_headless_shell-' : `${family}-`
  try { return readdirSync(playwrightBrowsersDir()).some((entry) => entry.startsWith(prefix)) } catch { return false }
}

const binName = process.platform === 'win32' ? `${bin}.cmd` : bin
const binPath = join(root, 'node_modules', '.bin', binName)

// read the version of <pkg> currently installed in ./node_modules (scoped-aware path)
function installedVersion() {
  const rel = pkg.startsWith('@')
    ? join('node_modules', pkg.split('/')[0], pkg.split('/')[1])
    : join('node_modules', pkg)
  const pp = join(root, rel, 'package.json')
  if (!existsSync(pp)) return null
  try { return JSON.parse(readFileSync(pp, 'utf8')).version } catch { return null }
}

// review #4 (version pinning): never reuse a pre-existing binary at a different version.
// Missing binary -> install (first run); installed-but-different -> fail (pin drift).
const have = installedVersion()
if (!existsSync(binPath) || have !== version) {
  // Pin drift is a real state (someone ran `npm install`, or the pin was bumped
  // without setup): fail loudly instead of reinstalling silently on every start.
  if (have && have !== version) {
    console.error(`[mcp-serve] pin drift: ${pkg} installed @${have}, pin wants @${version}`)
    console.error('[mcp-serve] run: pixi run mcp setup')
    process.exit(1)
  }
  // Missing binary is the first-run path (GUI host first click, clean clone, CI
  // probe) and must keep working. npm writes its summary to stdout, which IS the
  // MCP JSON-RPC channel here, so capture it and re-emit on stderr.
  console.error(`[mcp-serve] installing ${pkg}@${version} (into ./node_modules, gitignored) ...`)
  // Arguments are passed as an array (no shell), so a package name or version
  // from the manifest can never turn into shell syntax. Windows resolves npm
  // through npm.cmd, hence the explicit command name.
  const npm = process.platform === 'win32' ? 'npm.cmd' : 'npm'
  const res = spawnSync(npm, ['install', '-D', '--save-exact', `${pkg}@${version}`], {
    cwd: root,
    stdio: ['ignore', 'pipe', 'inherit'],
  })
  if (res.stdout && res.stdout.length) process.stderr.write(res.stdout)
  if (res.error) {
    console.error('[mcp-serve] install failed:', res.error.message)
    process.exit(1)
  }
  if (res.status !== 0) process.exit(res.status ?? 1)
}

// W3 (plan: docs/mcp-browser-plan.md): resolve the browser through the record that
// `mcp setup` wrote, and hand it to the server via the official envs. Everything here
// is env-driven on purpose -- CLI flags outrank envs, so a flag in the spec would
// silently pin one browser mode and block the ladder.
const RECORD_PATH = join(root, 'out', 'mcp-browser.json')

function readRecord() {
  if (!existsSync(RECORD_PATH)) return null
  try {
    const parsed = JSON.parse(readFileSync(RECORD_PATH, 'utf8'))
    const entry = parsed?.resolutions?.[name]
    return entry && entry.mode ? entry : null
  } catch { return null }  // torn/malformed read must never crash the launcher
}

function exportedEnv() {
  const env = { ...process.env }
  // npm's bin shims start with `#!/usr/bin/env node`; on a box whose PATH holds an
  // ancient system node (v10 here) that shim resolves the WRONG interpreter. Put the
  // node that runs this launcher first so every shim sees the pinned one.
  const nodeDir = dirname(process.execPath)
  env.PATH = nodeDir + (process.platform === 'win32' ? ';' : ':') + (env.PATH || '')
  if (!spec?.browser) return env   // server has no browser face: inherit untouched
  const record = readRecord()
  if (!record) {
    // No resolution: point at the installer instead of re-running rung 1 semantics.
    // On a host the pin dropped, that retry would fail with a misleading OS hint.
    console.error('[mcp-serve] no browser resolution for this server -- run: pixi run mcp setup')
    if (!managedBrowserPresent(spec.browser)) {
      console.error(`[mcp-serve] ${spec.browser} has no managed build here and no recorded fallback`)
      process.exit(1)
    }
    env.PLAYWRIGHT_MCP_BROWSER = spec.browser
    delete env.PLAYWRIGHT_MCP_EXECUTABLE_PATH
    return env
  }
  if (record.mode === 'playwright') {
    env.PLAYWRIGHT_MCP_BROWSER = record.browser || spec.browser
    delete env.PLAYWRIGHT_MCP_EXECUTABLE_PATH   // symmetric unset: a stale shell value must not win
  } else {
    env.PLAYWRIGHT_MCP_EXECUTABLE_PATH = record.executable
    delete env.PLAYWRIGHT_MCP_BROWSER           // and the same in the other direction
  }
  return env
}

function managedBrowserPresent(family) {
  const prefix = family === 'chromium-headless-shell' ? 'chromium_headless_shell-' : `${family}-`
  try { return readdirSync(playwrightBrowsersDir()).some((entry) => entry.startsWith(prefix)) } catch { return false }
}

// Runtime args are derived, never inherited from the spec: the scratch output goes to
// the repo's gitignored out/mcp (absolute -- spawn sets no cwd), and headless follows
// the display situation so a GUI-less host works out of the box. The envs WINNING over
// flags is not true in @playwright/mcp, so anything that must override belongs in env;
// these two are launcher-owned and stable, hence args.
function runtimeArgs() {
  const args = [...serverArgs]
  if (spec?.browser) {
    const force = process.env.MODUKIT_MCP_FORCE_HEADLESS
    const displayless = !process.env.DISPLAY && !process.env.WAYLAND_DISPLAY
    const headless = force === '1' || (force !== '0' && displayless)
    args.push('--output-dir', join(root, 'out', 'mcp'))
    if (headless) args.push('--headless')
  }
  return args
}

const child = spawn(binPath, runtimeArgs(), { stdio: 'inherit', env: exportedEnv() })
child.on('error', (err) => { console.error(`[mcp-serve] failed to start ${bin}:`, err.message); process.exit(1) })
child.on('exit', (code) => process.exit(code ?? 1))
