/** examples/js/pin_test.js - wave-2 JS volume M1 node:test harness (D102
 * R3-A gate 3): pins the generated facade against the REAL built bridge
 * .node + kernel .so. Run via `node --test examples/js/pin_test.js` (or
 * scripts/demo-wave2-js.sh). Paths default to the workspace build outputs;
 * override with MODUKIT_JS_BRIDGE / MODUKIT_CORE_SO.
 */
'use strict';

const { test } = require('node:test');
const assert = require('node:assert');
const path = require('node:path');
const fs = require('node:fs');

const REPO = path.resolve(__dirname, '..', '..');
process.env.MODUKIT_CORE_SO ??= path.join(REPO, 'target', 'debug', 'libmodukit_core.so');
process.env.MODUKIT_JS_BRIDGE ??= path.join(REPO, 'target', 'js', 'modukit_js.node');

if (!fs.existsSync(process.env.MODUKIT_JS_BRIDGE) || !fs.existsSync(process.env.MODUKIT_CORE_SO)) {
  console.log('SKIP: bridge .node or kernel .so not built (run scripts/demo-wave2-js.sh first)');
  process.exit(77);
}
const skin = require(path.join(REPO, 'target', 'js', 'modukit_embed.js'));

/* the 16 rung-0 verbs, names verbatim from contracts/modukit_embed.h
 * (declaration lines cited; re-derived independently of the emitter) */
const RUNG0_VERBS = [
  ['modukit_embed_abi_version', 128],
  ['modukit_install', 139],
  ['modukit_uninstall', 144],
  ['modukit_resolve', 149],
  ['modukit_start', 153],
  ['modukit_stop', 157],
  ['modukit_state', 160],
  ['modukit_service_call', 172],
  ['modukit_bus_publish', 184],
  ['modukit_bus_subscribe', 193],
  ['modukit_bus_recv', 201],
  ['modukit_bus_unsubscribe', 206],
  ['modukit_clock_now', 212],
  ['modukit_clock_advance', 217],
  ['modukit_introspect_snapshot', 227],
  ['modukit_introspect_events', 235],
];

test('facade exports all 16 rung-0 verbs', () => {
  assert.strictEqual(RUNG0_VERBS.length, 16, 'rung-0 verb count moved (16 at M1)');
  for (const [name, line] of RUNG0_VERBS) {
    assert.strictEqual(typeof skin[name], 'function', `facade lacks ${name} (embed.h:${line})`);
    const decl = fs
      .readFileSync(path.join(REPO, 'contracts', 'modukit_embed.h'), 'utf8')
      .split('\n')[line - 1];
    assert.ok(decl.trim().startsWith(`int32_t ${name}(`), `citation drift: embed.h:${line}`);
  }
});

test('enum mirrors match the header values', () => {
  // Values copied verbatim from contracts/modukit_embed.h (lines cited in
  // the ledger-style blocks: kernel_code :36-70, plugin_state :74-83,
  // clock_domain :86-90, qos :93-98, event_kind :102-110).
  assert.deepStrictEqual({ ...skin.PluginState }, {
    STATE_UNINSTALLED: 0,
    STATE_INSTALLED: 1,
    STATE_RESOLVING: 2,
    STATE_STARTING: 3,
    STATE_ACTIVE: 4,
    STATE_STOPPING: 5,
    STATE_RESOLVED: 6,
    STATE_FAILED: 7,
  });
  assert.deepStrictEqual({ ...skin.ClockDomain }, { CLOCK_REAL: 0, CLOCK_SIM: 1, CLOCK_STEP: 2 });
  assert.deepStrictEqual({ ...skin.Qos }, {
    QOS_LATEST_VALUE: 0,
    QOS_KEEP_ALL: 1,
    QOS_LOSSY: 2,
    QOS_BOUNDED: 3,
  });
  assert.deepStrictEqual({ ...skin.EventKind }, {
    EVENT_LIFECYCLE: 0,
    EVENT_REGISTRY: 1,
    EVENT_MODULE_LOADED: 2,
    EVENT_TIMER_DONE: 3,
    EVENT_EDGE_REMOVED: 4,
    EVENT_MARKER: 5,
    EVENT_LOG: 6,
  });
  assert.strictEqual(skin.KernelCode.NOT_FOUND, 14);
  assert.strictEqual(skin.KernelCode.MANIFEST_INVALID, 1);
  assert.strictEqual(skin.KernelCode.CLOCK_DIAL_REFUSED, 27);
  assert.strictEqual(skin.KernelCode.KERNEL_CODE_UNSPECIFIED, 0);
  assert.strictEqual(Object.keys(skin.KernelCode).length, 28, 'D51 dictionary is 28 names');
  assert.strictEqual(skin.MODUKIT_EMBED_ABI_VERSION, 1, 'embed.h:28 define mirror');
});

test('embed_abi_version through the facade returns 1', () => {
  assert.strictEqual(skin.modukit_embed_abi_version(), 1);
});

test('error narrowing returns the named variant', () => {
  // A real kernel-domain failure (unknown service id on the call door -
  // the host_demo scene [6] negative, same contract) arrives as the raw
  // negative D51 code; the facade table narrows it to the named variant.
  const bad = skin.modukit_service_call('com.example.nope', 'echo', Buffer.alloc(0), 64);
  assert.strictEqual(bad.raw, -14, 'unknown id -> NOT_FOUND(-14) at the bridge');
  assert.strictEqual(skin.EMBED_ERRORS[-bad.raw], 'NOT_FOUND');
});
