#!/usr/bin/env node
/** examples/js/host_demo.js - the wave-2 JS embed demo (D102 volume M1;
 * mirror of examples/cpp/host_demo.cc scenes [1]-[11], one-for-one, same
 * scene labels and success-string shape). Proof that the HostEmbedV1 face
 * is not paper for JS either: the application calls ONLY the generated
 * facade (target/js/modukit_embed.js, re-emitted from
 * contracts/modukit_embed.h by scripts/demo-wave2-js.sh) over the generated
 * napi bridge (.node), which dlopens the ambient Rust process-global kernel
 * in libmodukit_core.so (the D100 M0 cdylib). Fixture leg: the same real
 * C++ plugin .so the cpp and Python M1 demos drive (argv[3]).
 *
 * Word list, one line per verb, in one honest chain (16 rung-0 verbs over
 * 11 scenes - some scenes merge verbs, exactly like the cpp demo):
 *     embed_abi_version -> install -> resolve -> start -> state
 *     -> service_call (EchoRequest/EchoReply over the demo contract)
 *     -> bus_publish -> bus_subscribe -> bus_recv -> bus_unsubscribe
 *     -> clock_now -> clock_advance (Real domain: pinned REFUSAL, not a
 *        silent no-op - the header's rule, asserted as an expected code)
 *     -> introspect_snapshot -> introspect_events (the opaque-bytes codecs
 *        documented in embed.rs/ embed_codec.rs are parsed field by field)
 *     -> stop -> uninstall.
 * Prints `JS-DEMO PASS` only when every step asserted clean; exit 77 = skip
 * when the kernel .so / bridge .node / facade / fixture .so is absent;
 * DEMO_JS_SABOTAGE=1 corrupts the manifest at exactly ONE named point
 * (scene [2]) so the gate must RED.
 *
 * Payload codec note: the pre-drawn hand-rolled minimal fixture codec (the
 * python-era precedent, host_demo.py:64-116 - the cpp demo carries its own
 * hand-parsed codecs at host_demo.cc:184-218). Payload bytes are pinned to
 * examples/cpp/host_demo.cc:39 kPayload. Integer crossing: 64-bit values
 * arrive as JS numbers via the bridge's i64 channel (see the facade banner).
 */
'use strict';

const fs = require('node:fs');
const path = require('node:path');
const { Buffer } = require('node:buffer');

const REPO = path.resolve(__dirname, '..', '..');

const K_PLUGIN_ID = 'com.example.cpp.demo';
const K_SERVICE_ID = 'com.modukit.demo.DemoService';
const K_TOPIC = 'com.example.cpp.demo.status';
const K_PAYLOAD = Buffer.from('embed-demo-payload'); // pinned to host_demo.cc:39
const K_BUS_VALUE = Buffer.from('alpha-signal');

let failures = 0;

function check(ok, what) {
  if (!ok) {
    console.log(`FAIL: ${what}`);
    failures += 1;
  } else {
    console.log(`ok:   ${what}`);
  }
}

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

function encodeEchoRequest(payload) {
  /* EchoRequest{ bytes payload = 1 } (com.modukit.demo.proto): tag
   * field 1, wire type 2, then the length-delimited bytes. */
  return Buffer.concat([Buffer.from([0x0a]), putVarint(payload.length), payload]);
}

function decodeEchoReply(buf) {
  /* EchoReply{ bytes payload = 1; DemoStatus status = 2 } -> [payload,
   * status]. Unknown fields are walked and skipped (proto2 wire discipline,
   * the decoder dogfoods D12 rule 2 like the codegen's own reader). */
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

/* scene [2] build point: the deploy-tier manifest TOML for the fixture. */
function manifestFor(soPath) {
  if (process.env.DEMO_JS_SABOTAGE !== undefined) {
    /* THE single sabotage point (documented in scripts/demo-wave2-js.sh):
     * an unknown placement must be refused at the install door. */
    console.log('[sabotage] DEMO_JS_SABOTAGE=1 -> corrupting manifest');
    return `id = "${K_PLUGIN_ID}"\nversion = "0.1.0"\n[entry]\nkind = "dlx"\nartifact = "${soPath}"\n`;
  }
  return `id = "${K_PLUGIN_ID}"\nversion = "0.1.0"\n[entry]\nkind = "dl"\nartifact = "${soPath}"\n`;
}

function main() {
  if (process.argv.length < 5) {
    console.log('SKIP: usage: host_demo.js <libmodukit_core.so> <modukit_js.node> <path-to-libcpp-example-plugin.so>');
    return 77;
  }
  const [coreSo, bridgeNode, fixtureSo] = process.argv.slice(2);
  for (const missing of [coreSo, bridgeNode, fixtureSo]) {
    if (!fs.existsSync(missing)) {
      console.log(`SKIP: file missing at ${missing}`);
      return 77;
    }
  }
  const facadePath = path.join(REPO, 'target', 'js', 'modukit_embed.js');
  if (!fs.existsSync(facadePath)) {
    console.log(`SKIP: generated facade missing at ${facadePath} (run scripts/demo-wave2-js.sh)`);
    return 77;
  }
  process.env.MODUKIT_CORE_SO = coreSo;
  process.env.MODUKIT_JS_BRIDGE = bridgeNode;
  const skin = require(facadePath); // the generated facade; nothing else

  console.log('=== ModuKit wave-2 M1 JS embed demo ===');

  /* [1] embed_abi_version: the facade's own handshake probe. */
  check(
    skin.modukit_embed_abi_version() === 1, // MODUKIT_EMBED_ABI_VERSION, embed.h:28
    '[1] embed_abi_version == 1',
  );

  /* [2] install: manifest-v0 TOML in, kernel-owned canonical id out. */
  const inst = skin.modukit_install(manifestFor(fixtureSo));
  check(
    inst.raw === 0 && inst.out_plugin_id === K_PLUGIN_ID,
    '[2] install -> canonical id (kernel-owned bytes)',
  );
  if (failures !== 0) {
    console.log(`JS-DEMO FAIL (${failures}, install door)`);
    return 1; // nothing downstream is reachable without a plugin
  }
  const pid = inst.out_plugin_id;

  /* [3] resolve + [4] start + [5] state: the dl handshake and the walk. */
  check(skin.modukit_resolve(pid) === 0, '[3] resolve -> ABI v1 handshake green');
  check(skin.modukit_start(pid) === 0, '[4] start -> ACTIVE');
  const st = skin.modukit_state(pid);
  check(
    st.raw === 0 && st.out_state === skin.PluginState.STATE_ACTIVE,
    '[5] state == ACTIVE (4)',
  );

  /* [6] service_call: typed round trip over the frozen message shape. */
  const reqBytes = encodeEchoRequest(K_PAYLOAD);
  const call = skin.modukit_service_call(K_SERVICE_ID, 'echo', reqBytes, reqBytes.length + 256);
  let echoOk = false;
  if (call.raw >= 0) {
    const [payload, status] = decodeEchoReply(call.response);
    echoOk = payload.equals(K_PAYLOAD) && status === 1; // DEMO_STATUS_OK
  }
  check(echoOk, '[6] service_call Echo round-trips the payload (status OK)');
  const bad = skin.modukit_service_call('com.example.nope', 'echo', reqBytes, reqBytes.length + 256);
  check(
    bad.raw === -skin.KernelCode.NOT_FOUND,
    '[6] service_call unknown id -> NOT_FOUND(-14)',
  );

  /* [7] bus: publish, late-join subscribe, recv (cap discipline honored). */
  check(
    skin.modukit_bus_publish(K_TOPIC, skin.Qos.QOS_LATEST_VALUE, K_BUS_VALUE) === 0,
    '[7] bus_publish latest-value',
  );
  const sub = skin.modukit_bus_subscribe(K_TOPIC);
  check(
    sub.raw === sub.out_queue_id && sub.out_queue_id >= 1,
    '[7] bus_subscribe -> kernel-owned queue id',
  );
  const small = skin.modukit_bus_recv(sub.out_queue_id, 4);
  check(
    small.raw === -skin.KernelCode.QUOTA_EXCEEDED && Number(small.out_len) === K_BUS_VALUE.length,
    '[7] bus_recv too-small cap -> QUOTA_EXCEEDED, need reported',
  );
  const got = skin.modukit_bus_recv(sub.out_queue_id, K_BUS_VALUE.length + 16);
  check(
    got.raw >= 0 && Number(got.out_len) === K_BUS_VALUE.length && got.out_value.equals(K_BUS_VALUE),
    '[7] bus_recv (late join) gets the exact value back',
  );
  check(
    skin.modukit_bus_unsubscribe(sub.out_queue_id) === 0 &&
      skin.modukit_bus_recv(sub.out_queue_id, K_BUS_VALUE.length + 16).raw ===
        -skin.KernelCode.NOT_FOUND,
    '[7] bus_unsubscribe retires the queue id',
  );

  /* [8] clock: the ambient kernel runs the Real domain (embed.rs decision
   * 1), so now() answers and the dial REFUSES - pinned code, documented. */
  const now = skin.modukit_clock_now();
  check(now.raw === 0 && now.out_now_ns >= 0, '[8] clock_now >= 0 ns');
  check(
    skin.modukit_clock_advance(1000000) === -skin.KernelCode.CLOCK_DIAL_REFUSED,
    '[8] clock_advance on Real -> CLOCK_DIAL_REFUSED (never a no-op)',
  );

  /* [9] introspect_snapshot: size query then exact copy; the documented
   * line codec parses and carries this plugin's ACTIVE row. */
  const sProbe = skin.modukit_introspect_snapshot(0); // cap 0 = the NULL size query
  check(
    sProbe.raw === sProbe.out_len && sProbe.out_len > 0,
    '[9] introspect_snapshot size query (NULL buffer)',
  );
  const sTake = skin.modukit_introspect_snapshot(sProbe.out_len);
  let snapOk = false;
  if (sTake.raw >= 0 && Number(sTake.out_len) === sProbe.out_len) {
    const text = sTake.out_buf.toString('utf8');
    snapOk =
      text.includes(`plugin\t${K_PLUGIN_ID}\tACTIVE`) &&
      text.includes('service\torg.modukit.snapshot\t') &&
      text.includes('counters\t');
  }
  check(snapOk, '[9] introspect_snapshot: documented line codec parses');

  /* [10] introspect_events: resumable LE record walk; the lifecycle
   * transitions of THIS plugin must be on the ring. */
  const eProbe = skin.modukit_introspect_events(0, 0); // cap 0 = the NULL size query
  check(eProbe.raw >= 0 && eProbe.out_len > 28, '[10] introspect_events size query');
  const eTake = skin.modukit_introspect_events(0, eProbe.out_len);
  let sawLifecycle = false;
  let tiled = false;
  if (eTake.raw >= 0) {
    const buf = eTake.out_buf;
    const copied = Number(eTake.out_len);
    let off = 0;
    let lastSeq = 0;
    while (off + 28 <= copied) {
      const kind = buf.readInt32LE(off);
      const seq = Number(buf.readBigUInt64LE(off + 4));
      const plen = Number(buf.readBigUInt64LE(off + 20));
      if (
        kind === skin.EventKind.EVENT_LIFECYCLE &&
        buf.subarray(off + 28, off + 28 + K_PLUGIN_ID.length).toString('utf8') === K_PLUGIN_ID
      ) {
        sawLifecycle = true;
      }
      lastSeq = seq;
      off += 28 + plen;
    }
    tiled = off === copied && Number(eTake.out_seq) === lastSeq;
  }
  check(sawLifecycle && tiled, '[10] introspect_events: LE records tile the buffer; lifecycle rows found');

  /* [11] stop + uninstall close the walk (and the plugin's services). */
  check(skin.modukit_stop(pid) === 0, '[11] stop -> RESOLVED');
  const stR = skin.modukit_state(pid);
  check(
    stR.raw === 0 && stR.out_state === skin.PluginState.STATE_RESOLVED,
    '[11] state == RESOLVED (6)',
  );
  check(skin.modukit_uninstall(pid) === 0, '[11] uninstall -> UNINSTALLED');
  const stU = skin.modukit_state(pid);
  check(
    stU.raw === 0 && stU.out_state === skin.PluginState.STATE_UNINSTALLED,
    '[11] state == UNINSTALLED (0)',
  );
  check(
    skin.modukit_resolve(pid) === -skin.KernelCode.CONTRACT_VIOLATION,
    '[11] resolve from UNINSTALLED -> CONTRACT_VIOLATION (table guard)',
  );

  if (failures !== 0) {
    console.log(`JS-DEMO FAIL (${failures})`);
    return 1;
  }
  console.log('JS-DEMO PASS');
  return 0;
}

process.exit(main());
