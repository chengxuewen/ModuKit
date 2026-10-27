#!/usr/bin/env bash
# Wave-2 plugin-door NODE leg (leg thirteen; D106 batch 3, plan T3.3):
# the hand-written skin bindings/js/plugin_runtime.mjs walks the exe wall as
# a kernel-spawned script child (runner = the absolute node path, artifact =
# a staged js_echo.mjs copy). Stages mirror demo-wave2-plugin-py.sh exactly:
# [0] RUNNER line; [1] conformance BEFORE launching anything (all 14 golden
# frames re-encoded byte-exact through the skin's framing helpers + two
# hostile-frame plants); [2] embed-driver lifecycle - install -> FIDELITY
# (the cross-volume constant, byte-verbatim) -> caps -> BUS ROUNDTRIP (seed
# crosses out via FORWARD, self-echo crosses back via DELIVER, the
# derivative returns through `seen`) -> DIAL clean refusal (-27, child
# alive) -> cooperative stop -> liar answers the typed ServiceCallFailed
# (-25) twice while staying ACTIVE -> stop -> the next echo answers
# byte-exact again; [3] SIGKILL the driver, /proc census (the house law:
# never pgrep -f) asserts zero strays - the EOF path of the kernel's OWN
# child, not a driver-managed subprocess; [4] the C0 lever
# DEMO_PLUGIN_JS_SABOTAGE=1 skips the skin's REGISTER push at its one named
# point: the lifecycle must go red, no PASS line, exit 1 (the green run
# proves it inline; an externally-set lever reds the whole leg before
# stage [3]).
# PASS-line grammar mirrors scripts/demo-wave2-py-sup.sh; the cargo legs
# route through pixi per D65. Auto-SKIP only when an instrument is absent
# (no node, no build toolchain at all, no pinned-env protoc - the cpp-band
# instrument contract) - a present toolchain that fails to build is RED,
# never a silent skip (C0). The PLUGIN child needs no .so and no npm (pure
# node stdlib skin); the driver loads libmodukit_core.so through the
# generated napi bridge, no LD_PRELOAD anywhere in this leg (no cpp fixture
# chain: the plugin child is a .mjs file).
set -uo pipefail
cd "$(dirname "$0")/.."

export PATH="$HOME/.pixi/bin:$PATH"
# T3.1 runner-origin probe (wave-2 benchmark plan section 4): pixi default
# env node first, PATH fallback; neither present keeps the loud SKIP
# (marker string byte-identical). One RUNNER line per invocation, version
# read from the interpreter itself.
if [ -x ".pixi/envs/default/bin/node" ]; then
  NBIN=".pixi/envs/default/bin/node"
  NSRC="pixi-env"
elif command -v node >/dev/null 2>&1; then
  NBIN="node"
  NSRC="PATH fallback - pixi env not warmed"
else
  echo "demo-wave2-plugin-js: SKIP (no node: the skin has no interpreter to run in)"
  exit 0
fi
echo "RUNNER: node $("$NBIN" --version 2>&1) ($NSRC)"
if command -v pixi >/dev/null 2>&1; then
  CARGO=(pixi run cargo)
elif command -v cargo >/dev/null 2>&1; then
  CARGO=(cargo)
else
  echo "demo-wave2-plugin-js: SKIP (no pixi and no cargo: no build toolchain on this box)"
  exit 0
fi
# Protoc pin (the instrument the cpp band auto-skips on) - same contract as
# scripts/demo-wave2-js.sh so the skins rise and fall together.
protoc_found=0
[ -x ".pixi/envs/default/bin/protoc" ] && protoc_found=1
command -v protoc >/dev/null 2>&1 && protoc_found=1
if [ "$protoc_found" -eq 0 ]; then
  echo "demo-wave2-plugin-js: SKIP (no pinned-env protoc: cpp-band instrument contract unmet)"
  exit 0
fi
# The skin + fixtures are tracked repo content (no ignore rule exists under
# bindings/js - nothing generated lands there, unlike bindings/py): absence
# = broken tree, RED by the same law as a failed build.
for f in bindings/js/plugin_runtime.mjs test-plugins/js/js_echo.mjs test-plugins/js/js_liar.mjs; do
  [ -f "$f" ] || { echo "demo-wave2-plugin-js: RED (tracked content missing: $f)"; exit 1; }
done

log=$(mktemp)
dlog=$(mktemp)
sablog=$(mktemp)
stage=$(mktemp -d)
trap 'rm -f "$log" "$dlog" "$sablog"; rm -rf "$stage"' EXIT

"${CARGO[@]}" build -p modukit-core >>"$log" 2>&1 || {
  echo "demo-wave2-plugin-js: RED (modukit-core cdylib build failed; last lines:)"
  tail -10 "$log"
  exit 1
}
core_so=""
for c in target/debug/libmodukit_core.so out/cargo-target/debug/libmodukit_core.so; do
  if [ -f "$c" ]; then core_so="$c"; break; fi
done
[ -n "$core_so" ] || { echo "demo-wave2-plugin-js: RED (no libmodukit_core.so after a green build)"; exit 1; }

# The live embed faces are REGENERATED through the emission door (D63/D102):
# the bridge + facade are generated, never hand-written; nothing lands under
# target/js/ by hand (same doors the js-sup leg rides).
scratch=$(mktemp -d)
"${CARGO[@]}" run -q -p modukit-codegen --bin protoc-gen-modukit -- \
  "@contracts/org.modukit.runtime.proto.fdset" \
  "@contracts/com.modukit.demo.proto.fdset" \
  --embed_header contracts/modukit_embed.h \
  --js_out "$scratch" \
  --output_dir "$scratch" >>"$log" 2>&1 || {
  echo "demo-wave2-plugin-js: RED (face regeneration failed; last lines:)"
  tail -10 "$log"
  rm -rf "$scratch"
  exit 1
}
mkdir -p target/js
cp "$scratch/modukit_embed_bridge.rs" crates/modukit-js/src/lib.rs || { echo "demo-wave2-plugin-js: RED (bridge face not copied)"; rm -rf "$scratch"; exit 1; }
cp "$scratch/modukit_embed.js" target/js/modukit_embed.js || { echo "demo-wave2-plugin-js: RED (facade not copied)"; rm -rf "$scratch"; exit 1; }
rm -rf "$scratch"
[ -f crates/modukit-js/src/lib.rs ] || { echo "demo-wave2-plugin-js: RED (crates/modukit-js/src/lib.rs not regenerated)"; exit 1; }
[ -f target/js/modukit_embed.js ] || { echo "demo-wave2-plugin-js: RED (target/js/modukit_embed.js not regenerated)"; exit 1; }
# Live bridge face must equal its golden, or the checked-in corpus drifted.
cmp -s crates/modukit-js/src/lib.rs crates/modukit-codegen/testdata/golden/modukit_embed_bridge.rs || {
  echo "demo-wave2-plugin-js: RED (crates/modukit-js/src/lib.rs drifted from the golden - regen + review)"
  exit 1
}

# The napi bridge addon (the facade's dlopen floor): same doors as the
# js-sup leg (the M0 probe grammar - the cdylib is the .node by another
# suffix).
"${CARGO[@]}" build -p modukit-js --offline >>"$log" 2>&1 || {
  echo "demo-wave2-plugin-js: RED (modukit-js addon build failed; last lines:)"
  tail -10 "$log"
  exit 1
}
[ -f target/debug/libmodukit_js.so ] || { echo "demo-wave2-plugin-js: RED (no libmodukit_js.so after a green build)"; exit 1; }
cp target/debug/libmodukit_js.so target/js/modukit_js.node

# Stage the plugin artifacts (the manifest names staged paths, runner and
# artifact both absolute and existing - the spawn doorway checks is_file).
cp test-plugins/js/js_echo.mjs test-plugins/js/js_liar.mjs "$stage/"
# Absolutize the runner path (the kernel is_file-checks the manifest runner
# - executor_proc.rs:225 refuses bare "node").
case "$NBIN" in
  /*) NABS="$NBIN";;
  */*) NABS="$PWD/$NBIN";;
  *) NABS=$(command -v "$NBIN");;
esac
[ -f "$NABS" ] || { echo "demo-wave2-plugin-js: RED (runner path not an existing file: $NABS)"; exit 1; }

# --- [1] conformance: BEFORE launching anything. Decode + re-encode every
# golden frame byte-exactly through the skin, then plant the two hostile
# frames (length over FRAME_MAX refused pre-alloc; trailing bytes refused).
cat > "$stage/conformance.mjs" <<'JSEOF'
// Scratch conformance probe for scripts/demo-wave2-plugin-js.sh stage [1]:
// the goldens (contracts/golden/wire-v1) are the ledger's published truth;
// the skin's own codecs must re-encode each one byte-for-byte. Args:
// <bindings-js-dir> <golden-dir>
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const rt = await import(`file://${path.join(process.argv[2], 'plugin_runtime.mjs')}`);
const GOLD = process.argv[3];

// Per-kind canonicalization: decode the payload, re-encode through the
// mirror encoder - byte identity proves BOTH directions of the codec.
function canonical(kind, payload) {
  if (kind === rt.MSG_START || kind === rt.MSG_STOP) {
    if (payload.length) {
      throw new rt.FrameReject(`${kind} must carry the EMPTY payload`);
    }
    return payload;
  }
  if (kind === rt.MSG_HELLO) {
    const h = rt.parseHello(payload);
    return rt.helloPayload(h.abi, h.pid, h.version, h.name, h.caps);
  }
  if (kind === rt.MSG_ACK) return rt.ackPayload(rt.parseAck(payload));
  if (kind === rt.MSG_REGISTER) {
    const r = rt.parseRegister(payload);
    return rt.registerPayload(r.serviceId, r.methods);
  }
  if (kind === rt.MSG_CALL) {
    const c = rt.parseCall(payload);
    return rt.callPayload(c.seq, c.method, c.input);
  }
  if (kind === rt.MSG_RESULT) {
    const r = rt.parseResult(payload);
    return rt.resultPayload(r.seq, r.status, r.output);
  }
  if (kind === rt.MSG_DIAL) return rt.dialPayload(rt.parseDial(payload));
  if (kind === rt.MSG_DUE) {
    const d = rt.parseDue(payload);
    return rt.duePayload(d.status, d.fired, d.jobs);
  }
  if (kind === rt.MSG_FORWARD) {
    const f = rt.parseForward(payload);
    return rt.forwardPayload(f.op, f.topic, f.qos, f.value);
  }
  if (kind === rt.MSG_DELIVER) {
    const d = rt.parseDeliver(payload);
    return rt.deliverPayload(d.topic, d.qos, d.value);
  }
  throw new rt.FrameReject(`golden carries an unknown kind ${kind}`);
}

const names = [
  'HELLO_plain', 'HELLO_caps', 'START', 'STOP', 'ACK', 'REGISTER', 'CALL',
  'RESULT', 'DIAL', 'DUE', 'FORWARD_PUB', 'FORWARD_SUB', 'FORWARD_UNSUB',
  'DELIVER',
];
let count = 0;
for (const n of names) {
  const raw = fs.readFileSync(path.join(GOLD, `${n}.frame`));
  const { payloadLen, wireVer, kind } = rt.decodeHeader(raw.subarray(0, 6));
  if (wireVer !== rt.WIRE_VERSION) {
    throw new rt.FrameReject(`${n}: foreign version ${wireVer}`);
  }
  if (raw.length !== 6 + payloadLen) {
    throw new rt.FrameReject(`${n}: declared ${payloadLen}, file holds ${raw.length - 6}`);
  }
  const payload = raw.subarray(6);
  if (!rt.encodeFrame(kind, canonical(kind, payload)).equals(raw)) {
    throw new rt.FrameReject(`${n}: re-encode is not byte-exact`);
  }
  count += 1;
}
if (count !== 14) {
  throw new Error(`expected 14 goldens, walked ${count}`);
}

// Socket-shaped reader: counts the LARGEST read request so the pre-alloc
// refusal is provable (a >FRAME_MAX length must never reach a payload
// read - the proc_wire.rs:140-149 posture).
class Shim {
  constructor(data) {
    this.data = data;
    this.off = 0;
    this.maxRequest = 0;
  }

  async read(n) {
    this.maxRequest = Math.max(this.maxRequest, n);
    const part = this.data.subarray(this.off, this.off + n);
    this.off += part.length;
    return part;
  }
}

// Hostile 1: length over FRAME_MAX - refused BEFORE any allocation.
const shim = new Shim(
  Buffer.concat([
    Buffer.from([0xff, 0xff, 0xff, 0xff, rt.WIRE_VERSION, rt.MSG_CALL]),
    Buffer.from('junk'),
  ]),
);
try {
  await rt.readFrame((n) => shim.read(n));
  throw new Error('hostile length accepted');
} catch (e) {
  if (!(e instanceof rt.FrameTooLarge)) throw e;
}
if (shim.maxRequest > 6) {
  throw new Error('payload read requested despite the over-ceiling length');
}

// Hostile 2: trailing bytes - refused by the strict parse (the ledger's
// "silence and lies are both disagreement" line).
try {
  const raw = fs.readFileSync(path.join(GOLD, 'REGISTER.frame'));
  rt.parseRegister(Buffer.concat([raw.subarray(6), Buffer.from([0])]));
  throw new Error('trailing bytes accepted');
} catch (e) {
  if (!(e instanceof rt.FrameReject)) throw e;
}

// Write-side pre-check too (the two-way ceiling): an over-ceiling OUTBOUND
// payload must be refused before any send (Bridge.talk precedent).
try {
  rt.encodeFrame(rt.MSG_FORWARD, Buffer.alloc(rt.FRAME_MAX + 1, 0x78));
  throw new Error('over-ceiling write accepted');
} catch (e) {
  if (!(e instanceof rt.FrameTooLarge)) throw e;
}

console.log('wave2-plugin-js: CONFORMANCE OK (14 frames + 2 hostile)');
JSEOF
"$NBIN" "$stage/conformance.mjs" "$PWD/bindings/js" "$PWD/contracts/golden/wire-v1" >"$log" 2>&1
status=$?
if [ "$status" -ne 0 ]; then
  cat "$log"
  echo "demo-wave2-plugin-js: RED (conformance node exited non-zero)"
  exit 1
fi
cat "$log"
grep -q 'wave2-plugin-js: CONFORMANCE OK (14 frames + 2 hostile)' "$log" || {
  echo "demo-wave2-plugin-js: RED (no CONFORMANCE OK line - the codec walk did not complete)"
  exit 1
}

# --- [2]+[3] driver. Runs the sup-rung lifecycle as its OWN process: the
# leg SIGKILLs it mid-life for the census, so the orphan walk exercises the
# kernel child's EOF path (the .so dies WITH the driver; the plugin child is
# the kernel's own child, not one this shell ever managed).
cat > "$stage/driver.mjs" <<'JSEOF'
// Scratch embed driver for scripts/demo-wave2-plugin-js.sh stage [2]: drives
// the GENERATED napi facade (target/js/modukit_embed.js) - sup_install on
// script-shaped exe manifests, the FIDELITY / BUS / liar / stop walk, then
// sleeps with a live child so the leg can SIGKILL it for the census. Args:
// <core-so> <bridge-node> <stage-dir> <runner-abs> <seed>
import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';

const [coreSo, bridgeNode, stage, runner] = process.argv.slice(2);
const seed = Buffer.from(process.argv[6] ?? '', 'utf-8');

process.env.MODUKIT_CORE_SO = coreSo;
process.env.MODUKIT_JS_BRIDGE = bridgeNode;
process.env.MODUKIT_JS_SKIN = path.join(process.cwd(), 'bindings', 'js');
process.env.MODUKIT_JS_SEED = seed.toString('utf-8');
const require = createRequire(import.meta.url);
const skin = require(path.join(process.cwd(), 'target', 'js', 'modukit_embed.js'));

const ID_ECHO = 'org.modukit.demo.jsecho';
const ID_LIAR = 'org.modukit.demo.jsliar';
const VERSION = '0.1.0';
const ACTIVE = skin.PluginState.STATE_ACTIVE;
// D51 codes as the ledger numbers them (modukit_embed.h, embed_codec.rs):
const NEG_NOT_FOUND = -14;
const NEG_SERVICE_CALL_FAILED = -25;
const NEG_CLOCK_DIAL_REFUSED = -27;
const K_DIAL_DELTA_NS = 1000000000;

if (skin.modukit_embed_abi_version() !== 1) {
  console.error(`driver: embed ABI ${skin.modukit_embed_abi_version()} != 1`);
  process.exit(1);
}

function fail(msg) {
  console.error(`PLUGIN-JS FAIL: ${msg}`);
  process.exit(1);
}

function manifest(idStr, artifact) {
  // The kernel cross-checks HELLO id/version against the manifest (strict,
  // executor_proc.rs:301-314): id = the skin's serve id, version = "0.1.0".
  return (
    `id = "${idStr}"\nversion = "${VERSION}"\n[entry]\nkind = "exe"\n` +
    `artifact = "${artifact}"\nrunner = "${runner}"\n`
  );
}

function install(idStr, artifact) {
  const r = skin.modukit_sup_install(manifest(idStr, artifact));
  if (r.raw !== 0 || r.out_plugin_id === undefined) {
    fail(`sup_install(${idStr}) -> ${r.raw}`);
  }
  return r.out_plugin_id;
}

function wantActive(pid, label) {
  const r = skin.modukit_sup_state(pid);
  if (r.raw !== 0 || r.out_state !== ACTIVE) {
    fail(`${label} must be ACTIVE(4), state=${r.out_state}`);
  }
}

function call(pidService, method, inp) {
  const respCap = Math.max(inp.length, 1) + 4096;
  const r = skin.modukit_sup_service_call(pidService, method, inp, respCap);
  return [r.raw, r.response.subarray(0, Math.max(r.raw, 0))];
}

function stopUninstall(pid, label) {
  // stop/uninstall carry no out-params: the raw i32 rides the return itself
  // (the {raw, out_*} object shape is only for out-param verbs).
  const rs = skin.modukit_sup_stop(pid);
  if (rs !== 0) fail(`${label} cooperative stop -> ${rs}`);
  const ru = skin.modukit_sup_uninstall(pid);
  if (ru !== 0) fail(`${label} uninstall -> ${ru}`);
}

// --- js_echo lifecycle. ONE service id at a time: sup_service_call acquires
// the top-ranked proxy (embed_sup.rs), so the liar only lives once echo is
// unregistered (this also proves the proxy teardown pairs with every stop).
const echoJs = path.join(stage, 'js_echo.mjs');
let pid = install(ID_ECHO, echoJs);
wantActive(pid, 'js_echo');

let [rc, out] = call(ID_ECHO, 'echo', seed);
if (rc !== seed.length || !out.equals(seed)) {
  fail(`FIDELITY echo -> rc=${rc} bytes=${out.toString('hex')} (the ${seed.length}-byte constant must come back verbatim)`);
}
console.log(`FIDELITY PASS (${rc} bytes verbatim through the script skin)`);

[rc, out] = call(ID_ECHO, 'caps', Buffer.alloc(0));
if (rc !== 1 || out[0] !== 1) {
  fail(`caps -> rc=${rc} out=${out.toString('hex')} want 01`);
}
console.log('CAPS PASS');

// BUS ROUNDTRIP: the seed crossed out via FORWARD(PUB) in the start window,
// the parent's self-echo (D96 ruling 6) sent it back as a DELIVER, the
// callback's derivative crossed out again - `seen` returns the last payload
// the skin's bus callback received.
[rc, out] = call(ID_ECHO, 'seen', Buffer.alloc(0));
const want = Buffer.concat([seed, Buffer.from('|bus')]);
if (rc !== want.length || !out.equals(want)) {
  fail(`BUS ROUNDTRIP -> rc=${rc} out=${out.toString('hex')} want ${want.toString('hex')} (crossing did not complete)`);
}
console.log('BUS ROUNDTRIP PASS (seed out, echo back, derivative in `seen`)');

// DIAL clean refusal: the skin answers DUE(DIAL_STATUS_REFUSED_REAL) and the
// parent maps it typed - the child must SURVIVE the refusal (a live,
// cooperative, undialable plugin - executor_proc.rs:850-855).
const dial = skin.modukit_sup_dial(pid, K_DIAL_DELTA_NS);
if (dial.raw !== NEG_CLOCK_DIAL_REFUSED) {
  fail(`sup_dial must surface the clean refusal -27, got ${dial.raw}`);
}
[rc, out] = call(ID_ECHO, 'echo', Buffer.from('alive?'));
if (rc !== 6 || !out.equals(Buffer.from('alive?'))) {
  fail(`child must answer after the refusal (rc=${rc})`);
}
console.log('DIAL REFUSED PASS (-27, child alive)');

// Cooperative stop (ACK path of the skin) then uninstall - the proxy leaves
// with the run, clearing the registry for the liar's single proxy.
stopUninstall(pid, 'js_echo');
console.log('STOP PASS (cooperative ack + uninstall)');

// --- js_liar: every call answers the author code -1; the door maps it to
// SERVICE_CALL_FAILED. Two calls both fail TYPED and the plugin stays
// ACTIVE between them - the i32 channel is a contract, not decoration.
const liarJs = path.join(stage, 'js_liar.mjs');
pid = install(ID_LIAR, liarJs);
wantActive(pid, 'js_liar');
[rc] = call(ID_LIAR, 'lie', Buffer.from('x'));
if (rc !== NEG_SERVICE_CALL_FAILED) {
  fail(`liar must answer -25 typed, got ${rc}`);
}
[rc] = call(ID_LIAR, 'lie', Buffer.from('x'));
wantActive(pid, 'js_liar after the lie');
if (rc !== NEG_SERVICE_CALL_FAILED) {
  fail(`second liar call -> ${rc}`);
}
console.log('LIAR PASS (typed -25 twice, plugin stayed ACTIVE)');

// The census needs a live child at the kill; the "next echo works" proof is
// js_echo re-installed (single-proxy door again after the liar unregisters).
stopUninstall(pid, 'js_liar');
pid = install(ID_ECHO, echoJs);
wantActive(pid, 'js_echo (post-liar)');
[rc, out] = call(ID_ECHO, 'echo', seed);
if (rc !== seed.length || !out.equals(seed)) {
  fail(`post-liar echo must answer byte-exact again (rc=${rc})`);
}
console.log('NEXT ECHO PASS (liar episode cost the family nothing)');

console.log(`READY ${process.pid}`);
console.log('LIFECYCLE PASS');
process.exitCode = 0;
// The leg SIGKILLs the driver; the plugin rides the EOF. The parked read in
// the addon's dlopened kernel does not keep the node loop alive on its own
// (napi threads are not loop refs), so park on a plain interval.
setInterval(() => {}, 1000);
JSEOF
"$NBIN" "$stage/driver.mjs" "$PWD/$core_so" "target/js/modukit_js.node" "$stage" "$NABS" \
  "sup-fidelity-payload" >"$dlog" 2>&1 &
dpid=$!
ok=""
for _ in $(seq 1 600); do
  if grep -q '^LIFECYCLE PASS' "$dlog" 2>/dev/null; then ok=1; break; fi
  if ! kill -0 "$dpid" 2>/dev/null; then break; fi
  sleep 0.1
done
if [ -z "$ok" ]; then
  cat "$dlog"
  echo "demo-wave2-plugin-js: RED (lifecycle driver failed or died before the marker)"
  exit 1
fi

# --- [3] orphan census: SIGKILL the driver mid-life (kernel .so gone with
# it), then the /proc walk asserts the staged children self-exited on the
# control EOF. House law: never pgrep -f (it can match this script's own
# command line); fs walk over /proc/<pid>/cmdline, mirror of the py leg's
# os.walk census (examples/py/host_sup_demo.py:122-140 shape).
# Kill + reap in bash (PIT-15: cleanup legs carry || true), then the scan.
kill -9 "$dpid" 2>/dev/null || true
wait "$dpid" 2>/dev/null || true
"$NBIN" - "$stage" <<'JSEOF'
// Scratch census for scripts/demo-wave2-plugin-js.sh stage [3]: the staged
// plugin children must self-exit on the control EOF within the deadline.
import fs from 'node:fs';
import process from 'node:process';

const stage = process.argv[2];
const needle = Buffer.from(`${stage}/js_`);
const deadline = Date.now() + 5000;
let survivors = [];
for (;;) {
  survivors = [];
  for (const entry of fs.readdirSync('/proc')) {
    if (!/^\d+$/.test(entry)) continue;
    let cmd;
    try {
      cmd = fs.readFileSync(`/proc/${entry}/cmdline`);
    } catch {
      continue; // vanished between readdir and read (the py-sup census twin)
    }
    if (cmd.includes(needle)) survivors.push(entry);
  }
  if (survivors.length === 0 || Date.now() > deadline) break;
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 50);
}
if (survivors.length) {
  console.error(`CENSUS FAIL: staged plugin pids survived: ${survivors.join(',')}`);
  process.exit(1);
}
console.log('CENSUS PASS (zero staged-plugin survivors after the driver SIGKILL)');
JSEOF
census_rc=$?
cat "$dlog"
if [ "$census_rc" -ne 0 ]; then
  echo "demo-wave2-plugin-js: RED (orphan census found strays - EOF path broken)"
  exit 1
fi

# --- [4] sabotage proof, inline every run (C0: a check is only trusted
# after it is OBSERVED red). The lever lands at the ONE named point in the
# skin (bindings/js/plugin_runtime.mjs, the REGISTER loop): skipping the
# announcement makes the driver's first service_call NOT_FOUND (-14), the
# lifecycle must die before the marker. Exit non-zero AND no marker = green.
DEMO_PLUGIN_JS_SABOTAGE=1 "$NBIN" "$stage/driver.mjs" "$PWD/$core_so" "target/js/modukit_js.node" \
  "$stage" "$NABS" "sup-fidelity-payload" >"$sablog" 2>&1
sab_rc=$?
if [ "$sab_rc" -eq 0 ] || grep -q '^LIFECYCLE PASS' "$sablog"; then
  cat "$sablog"
  echo "demo-wave2-plugin-js: RED (sabotage run went GREEN - the lever is not load-bearing)"
  exit 1
fi
if ! grep -q 'FIDELITY echo -> rc=-14' "$sablog"; then
  cat "$sablog"
  echo "demo-wave2-plugin-js: RED (sabotage run died at the wrong door - expect -14 NOT_FOUND)"
  exit 1
fi
echo "SABOTAGE PASS (REGISTER skipped -> lifecycle red, exit $sab_rc)"

echo "demo-wave2-plugin-js: PASS"
