#!/usr/bin/env node
/** examples/js/host_sup_demo.js - the wave-2 JS exe-rung demo (D102 volume
 * M2; a one-for-one mirror of examples/cpp/host_sup_demo.cc scenes, same
 * labels and success-string shape, and the Python host_sup_demo.py sibling).
 * Proof the sup_* exe rung is not paper for JS either: the application calls
 * ONLY the generated facade (target/js/modukit_embed.js, re-emitted from
 * contracts/modukit_embed.h by scripts/demo-wave2-js-sup.sh) over the
 * generated napi bridge (.node), which dlopens the ambient Rust Supervisor
 * kernel in libmodukit_core.so. The supervised child is the SAME real C++
 * fixture the cpp M2 demo drives (argv[3] .so + argv[4] runner binary).
 *
 * TWO modes of one script, exactly like the cpp binary:
 *   (default) WALK : the supervised host. Drives the facade's sup_* arm:
 *     modukit_sup_install (real runner spawn + HELLO + start) ->
 *     modukit_sup_state(ACTIVE) -> READY line -> modukit_sup_step(idle) ->
 *     modukit_sup_dial (step-domain child TURNS: typed status 0, the
 *     corrected word from host_sup_demo.cc:117-128) -> the M3-FIDELITY
 *     mirror: modukit_sup_service_call carries ONE non-empty typed protobuf
 *     payload ABI -> TL wire -> child service -> back and asserts the
 *     response payload bytes equal the pinned expected bytes EXACTLY before
 *     printing the FIDELITY PASS marker; then sleeps until killed.
 *   --proof PARENT : re-spawn node on this same file in WALK mode (the node
 *     analog of the cpp parent's fork + execv /proc/self/exe; the parent
 *     never loads the kernel), wait for the READY + FIDELITY markers over a
 *     side-channel status file (the node analog of the cpp parent reading the
 *     walk child's stdout pipe - node streams are async, so the handshake is
 *     a file token, not a blocking fgets), SIGKILL the child host, then a
 *     /proc cmdline census (fs.readdirSync + readFileSync, mirror of
 *     host_sup_demo.cc:57-84): the runner must have self-reaped on the
 *     control-pipe EOF. Zero survivors prints `JS-SUP-DEMO PASS`; any
 *     survivor prints the pid and FAILs (ORPHAN-PROOF under a JS host).
 *
 * Sabotage levers, each ONE named point (C0, mirror of the cpp two-lever
 * pattern; the Python host_sup_demo.py carries the same pair):
 *   DEMO_JS_M2_SABOTAGE=1 - the parent SIGSTOPs the walk child instead of
 *     killing it, so the runner legitimately survives: the /proc census MUST
 *     find the live runner and the gate goes RED with no PASS line.
 *   DEMO_JS_M3_SABOTAGE=1 - corrupts the fidelity stanza's expected bytes
 *     before the comparison: the walk child never prints the FIDELITY PASS
 *     marker, the parent goes RED with no JS-SUP-DEMO PASS line.
 * Exit 77 = kernel .so / bridge .node / facade / fixture .so / runner absent
 * (the skip contract of the cpp leg and examples/js/host_demo.js).
 *
 * Fidelity expectation, single-source pinned to the cpp stanza's own constant
 * at examples/cpp/host_sup_demo.cc:139 (`req.set_payload("sup-fidelity-payload")`)
 * and asserted by examples/py/host_sup_demo.py K_FIDELITY_PAYLOAD: JS asserts
 * THE SAME bytes. Payload codec: the demo-local EchoRequest/EchoReply walk
 * reused from examples/js/host_demo.js:58-113 (the python/cpp-era precedent).
 */
'use strict';

const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const { spawn } = require('node:child_process');
const { Buffer } = require('node:buffer');

const REPO = path.resolve(__dirname, '..', '..');

// Supervised record id == the fixture's canonical service id (mirror of
// host_sup_demo.cc:55 kServiceId and the manifest at :88 / :147-150).
const K_ID = 'com.modukit.demo.DemoService';
// Fidelity expectation, pinned to host_sup_demo.cc:139.
const K_FIDELITY_PAYLOAD = Buffer.from('sup-fidelity-payload');
// Step-domain dial delta the cpp stanza uses (host_sup_demo.cc:116, 1 s).
const K_DIAL_DELTA_NS = 1000000000;

function putVarint(value) {
  const out = [];
  for (;;) {
    const byte = value & 0x7f;
    value = Math.floor(value / 128);
    if (value) out.push(byte | 0x80);
    else { out.push(byte); return Buffer.from(out); }
  }
}

function readVarint(buf, off) {
  let value = 0;
  let shift = 1;
  for (;;) {
    const byte = buf[off];
    off += 1;
    value += (byte & 0x7f) * shift;
    if (!(byte & 0x80)) return [value, off];
    shift *= 128;
  }
}

// EchoRequest{ bytes payload = 1 } - byte-identical to protobuf
// SerializeAsString for this one-field message (reused from host_demo.js).
function encodeEchoRequest(payload) {
  return Buffer.concat([Buffer.from([0x0a]), putVarint(payload.length), payload]);
}

// EchoReply{ bytes payload = 1; DemoStatus status = 2 } -> [payload, status]
// (reused from host_demo.js:86-113; proto2 unknown fields walked + skipped).
function decodeEchoReply(buf) {
  let payload = Buffer.alloc(0);
  let status = null;
  let off = 0;
  while (off < buf.length) {
    const [key, o1] = readVarint(buf, off);
    off = o1;
    const field = Math.floor(key / 8);
    const wire = key & 7;
    if (wire === 2) {
      const [len, o2] = readVarint(buf, off);
      off = o2;
      const value = buf.subarray(off, off + len);
      off += len;
      if (field === 1) payload = value;
    } else if (wire === 0) {
      const [value, o2] = readVarint(buf, off);
      off = o2;
      if (field === 2) status = value;
    } else {
      return [payload, status]; // shape this fixture never emits
    }
  }
  return [payload, status];
}

/* exe manifest TOML for the fixture (mirror of host_sup_demo.cc:87-90 and the
 * Python manifest at host_sup_demo.py:147-151); kind "exe" + runner path. */
function manifestFor(fixtureSo, runPath) {
  return (
    'id = "com.modukit.demo.DemoService"\nversion = "0.1.0"\n' +
    '[entry]\nkind = "exe"\nartifact = "' + fixtureSo + '"\nrunner = "' + runPath + '"\n'
  );
}

// Append a handshake token for the parent (side-channel; only when the parent
// set DEMO_JS_STATUS_FILE, i.e. a proof-spawned walk child).
function signal(token) {
  const file = process.env.DEMO_JS_STATUS_FILE;
  if (file) fs.appendFileSync(file, token + '\n');
}

function walkMode(coreSo, bridgeNode, fixtureSo, runPath) {
  process.env.MODUKIT_CORE_SO = coreSo;
  process.env.MODUKIT_JS_BRIDGE = bridgeNode;
  const skin = require(path.join(REPO, 'target', 'js', 'modukit_embed.js'));

  if (skin.modukit_embed_abi_version() !== 1) {
    console.error(`sup: embed ABI ${skin.modukit_embed_abi_version()} != 1`);
    return 1;
  }
  const inst = skin.modukit_sup_install(manifestFor(fixtureSo, runPath));
  if (inst.raw !== 0 || inst.out_plugin_id !== K_ID) {
    console.error(`sup_install -> ${inst.raw}`);
    return 1;
  }
  const id = inst.out_plugin_id;
  const st = skin.modukit_sup_state(id);
  if (st.raw !== 0 || st.out_state !== skin.PluginState.STATE_ACTIVE) {
    console.error(`sup state=${st.out_state} want ACTIVE(4)`);
    return 1;
  }
  console.log(`READY ${process.pid}`);
  signal('READY');
  const step = skin.modukit_sup_step();
  if (step.raw !== 0 || step.out_actions !== 0) {
    console.error(`sup step actions=${step.out_actions} want 0`);
    return 1;
  }
  const dial = skin.modukit_sup_dial(id, K_DIAL_DELTA_NS);
  // modukit-run boots the Step domain (the supervisor's wire dial is the sole
  // time authority there), so this child TURNS: typed status 0. The corrected
  // word from host_sup_demo.cc:117-128 (a Real child would refuse instead).
  if (dial.raw !== 0 || dial.out_status !== 0) {
    console.error(`sup_dial must turn the step-domain child (raw=${dial.raw} status=${dial.out_status})`);
    return 1;
  }
  console.log(`WALK WORDS OK (install/ACTIVE/step/dial-turn status=${dial.out_status})`);

  /* M3 FIDELITY stanza (D94 W3 mirrored): the fixture's Echo answers with the
   * request payload verbatim, so the response payload BYTES must equal the
   * pinned expectation exactly (host_sup_demo.cc:132-169, host_sup_demo.py:188-211). */
  const reqBytes = encodeEchoRequest(K_FIDELITY_PAYLOAD);
  const call = skin.modukit_sup_service_call(K_ID, 'echo', reqBytes, reqBytes.length + 256);
  let want = K_FIDELITY_PAYLOAD;
  if (process.env.DEMO_JS_M3_SABOTAGE !== undefined) {
    /* THE single named sabotage point for the fidelity gate: corrupt the
     * expected bytes before the comparison - the gate MUST go red on a
     * byte-exact child (mirror of host_sup_demo.cc:147-154). */
    want = Buffer.from(want);
    want[want.length - 1] ^= 0x20;
    console.log('SABOTAGE: expected payload bytes corrupted');
  }
  let fidelityOk = call.raw > 0;
  if (fidelityOk) {
    const [payload, status] = decodeEchoReply(call.response);
    fidelityOk = payload.equals(want) && status === 1; // DEMO_STATUS_OK
  }
  if (!fidelityOk) {
    console.error(`FIDELITY FAIL (sup_service_call raw=${call.raw})`);
    return 1; // no marker: the parent loop + gate stay red
  }
  console.log(`FIDELITY PASS (${call.raw} typed bytes out, payload byte-exact)`);
  signal('FIDELITY');
  for (;;) {
    sleepMs(1000); // the parent kills us; the runner must survive us by no time
  }
}

// Census (mirror of host_sup_demo.cc:57-84 / host_sup_demo.py:122-140): the
// first pid other than self whose /proc cmdline carries the runner path
// substring, or -1 for none. A SIGKILLed host is a zombie with an empty
// cmdline, so it never counts; the runner is the only legitimate target.
function census(runPath) {
  const me = process.pid;
  for (const entry of fs.readdirSync('/proc')) {
    if (!/^\d+$/.test(entry)) continue;
    const pid = Number(entry);
    if (pid === me) continue;
    let cmd;
    try {
      cmd = fs.readFileSync(`/proc/${entry}/cmdline`);
    } catch {
      continue; // vanished between readdir and read (cpp fopen skip)
    }
    if (cmd.length > 0 && cmd.includes(runPath)) return pid;
  }
  return -1;
}

function sleepMs(ms) {
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, ms);
}

function alive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function killQuiet(pid, sig) {
  try {
    process.kill(pid, sig);
  } catch {
    /* already gone */
  }
}

function proofMode(coreSo, bridgeNode, fixtureSo, runPath) {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'modukit-js-sup-'));
  const status = path.join(dir, 'status');
  fs.writeFileSync(status, '');
  // Re-spawn node on this same file in WALK mode: the transcript rides the
  // shared stdout (stdio inherit -> the leg greps the markers there), and the
  // READY/FIDELITY handshake comes over the status file.
  const child = spawn(
    process.execPath,
    [__filename, coreSo, bridgeNode, fixtureSo, runPath],
    { stdio: ['ignore', 'inherit', 'inherit'], env: { ...process.env, DEMO_JS_STATUS_FILE: status } },
  );

  // Block on the status file until a token appears or the child dies before
  // it (the walk child either reaches the marker or dies - never hangs).
  function readUntil(token, deadlineMs) {
    let waited = 0;
    for (;;) {
      let text = '';
      try {
        text = fs.readFileSync(status, 'utf8');
      } catch {
        /* file not yet flushed */
      }
      if (text.includes(token)) return true;
      if (!alive(child.pid)) return false;
      if (waited >= deadlineMs) return false;
      sleepMs(50);
      waited += 50;
    }
  }

  if (!readUntil('READY', 30000)) {
    console.error('SUP-DEMO FAIL (walk child never reached READY)');
    killQuiet(child.pid, 'SIGKILL');
    return 1;
  }
  // The walk child must clear the fidelity stanza before the kill: a child
  // that dies without the token lands here red (gate load-bearing parent-side;
  // the runner's own empty-probe guard stays untouched - PIT-18).
  if (!readUntil('FIDELITY', 30000)) {
    console.error('SUP-DEMO FAIL (no FIDELITY PASS - walk child died at the door)');
    killQuiet(child.pid, 'SIGKILL');
    return 1;
  }

  const sabotage = process.env.DEMO_JS_M2_SABOTAGE !== undefined;
  if (sabotage) {
    // THE single named sabotage point for the orphan census: stopping the
    // host instead of killing it leaves the runner's control fd open - the
    // census MUST find a live survivor (mirror of host_sup_demo.cc:271-281).
    console.log('SABOTAGE: SIGSTOP instead of SIGKILL (runner expected live)');
    killQuiet(child.pid, 'SIGSTOP');
  } else {
    killQuiet(child.pid, 'SIGKILL');
  }

  const deadlineMs = sabotage ? 1000 : 5000;
  for (let waited = 0; ; waited += 100) {
    const survivor = census(runPath);
    if (survivor < 0) {
      if (sabotage) {
        console.error('SUP-DEMO FAIL (sabotage set but no survivor - census is not load-bearing)');
        killQuiet(child.pid, 'SIGKILL');
        return 1;
      }
      console.log('runner self-reaped under a JS host (orphan-proof)');
      console.log('JS-SUP-DEMO PASS');
      return 0;
    }
    if (waited >= deadlineMs) {
      if (sabotage) {
        console.log(`SABOTAGE confirmed: runner pid=${survivor} survived host stop`);
        killQuiet(child.pid, 'SIGKILL');
        return 1; // gate must be red under sabotage: it prints no PASS line
      }
      console.error(`SUP-DEMO FAIL: runner pid=${survivor} orphaned after host kill`);
      killQuiet(child.pid, 'SIGKILL');
      return 1;
    }
    sleepMs(100);
  }
}

function main() {
  const argv = process.argv.slice(2);
  const positional = argv.filter((a) => a !== '--proof');
  const proof = positional.length !== argv.length;
  if (positional.length < 4) {
    console.error('usage: host_sup_demo.js [--proof] <libmodukit_core.so> <modukit_js.node> <plugin.so> <runner>');
    return 2;
  }
  const [coreSo, bridgeNode, fixtureSo, runPath] = positional;
  const facadePath = path.join(REPO, 'target', 'js', 'modukit_embed.js');
  for (const missing of [coreSo, bridgeNode, fixtureSo, runPath, facadePath]) {
    if (!fs.existsSync(missing)) {
      console.log(`SKIP: file missing at ${missing}`);
      return 77;
    }
  }
  console.log(`=== ModuKit wave-2 M2 JS sup demo (${proof ? 'proof' : 'walk'}) ===`);
  return proof ? proofMode(coreSo, bridgeNode, fixtureSo, runPath)
    : walkMode(coreSo, bridgeNode, fixtureSo, runPath);
}

process.exit(main());
