#!/usr/bin/env node
// test-plugins/js/js_echo.mjs - the wave-2 plugin-door echo fixture (D106
// batch 3, plan T3.2: docs/wave-2-plugin-door-plan.md section 5). The kernel
// spawns it as `node <this-file> --fd=0` (manifest kind=exe,
// runner=<absolute node path>); the control socket IS fd 0 and
// stdout/stderr are nulled - this file must never print.
//
// Three services' worth of behavior behind ONE service id: the embed face's
// sup_service_call door acquires the TOP-RANKED proxy only (embed_sup.rs -
// a second id answers NOT_FOUND), so the leg's addressable word is
// method-scoped: echo = verbatim bytes (the fidelity constant rides it),
// caps = one fixed byte, seen = the last payload the bus callback received
// (the roundtrip's read-back leg).
//
// The bus topic carries a seed: the seed bytes arrive through the spawn
// environment (MODUKIT_JS_SEED) because the embed face has NO bus verb on
// the sup rung - the two rungs own separate kernels (embed_sup.rs header:
// "buses... separate") and the wall is transparent only inside the start
// window (stage-5-plan section 3). The runtime publishes the seed IN that
// window; the parent's self-echo (D96 ruling 6) sends it back as a DELIVER,
// on_bus fires here, and the derivative goes out the same way - two full
// crossings of driver-originated bytes, observable back via `seen`.
//
// Runtime resolution (the robust minimal pick, stated): the MODUKIT_JS_SKIN
// env var wins (the leg stages this file OUTSIDE the repo tree, where a
// __dirname-relative default cannot reach bindings/js); the fallback is the
// in-place repo layout (<repo>/test-plugins/js/x -> <repo>/bindings/js), so
// the fixture also runs straight from a checkout without any env. Byte law
// (plan T3.2): echo never stringifies - the fidelity constant survives as
// bytes, no toString() anywhere near the payload.

import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const ID = 'org.modukit.demo.jsecho';
const VERSION = '0.1.0';
const NAME = 'js-echo';
const TOPIC = 'org.modukit.demo.wire-echo';
const DERIV = Buffer.from('|bus'); // the derivative marker the driver asserts on

function findSkin() {
  const cands = [];
  const env = process.env.MODUKIT_JS_SKIN;
  if (env) cands.push(env);
  const here = path.dirname(fileURLToPath(import.meta.url));
  cands.push(path.normalize(path.join(here, '..', '..', 'bindings', 'js')));
  for (const c of cands) {
    if (fs.existsSync(path.join(c, 'plugin_runtime.mjs'))) return c;
  }
  process.exit(5); // no console to complain to: the code IS the diagnosis
}

const require = createRequire(import.meta.url);
const rt = await import(`file://${path.join(findSkin(), 'plugin_runtime.mjs')}`);
void require; // createRequire kept only for the require-shaped import idiom parity

const state = { seen: Buffer.alloc(0), republished: false };

async function onCall(method, inp) {
  if (method === 'echo') return [0, inp];
  if (method === 'caps') return [0, Buffer.from([1])];
  if (method === 'seen') return [0, state.seen];
  return [-1, Buffer.alloc(0)]; // the child's own code is the unknown-method authority
}

async function onBus(topic, payload) {
  state.seen = payload;
  if (topic === TOPIC && !state.republished) {
    // Republish exactly once: the derivative rides the same topic and
    // re-enters through the same self-echo - an unguarded callback would
    // loop the start window against its own drain.
    state.republished = true;
    rt.publish(TOPIC, Buffer.concat([payload, DERIV]));
  }
}

const seed = process.env.MODUKIT_JS_SEED ?? '';
// Seeded entry = interest + initial in-window publish (runtime T2.1 shape);
// seedless stays a plain SUB (the wall is then quiet, as in the flag-off
// runner columns of abi_v2_cross.rs).
const topics = seed ? [[TOPIC, Buffer.from(seed, 'utf-8')]] : [TOPIC];

await rt.serve({
  id: ID,
  version: VERSION,
  name: NAME,
  services: { [ID]: ['echo', 'caps', 'seen'] },
  onCall,
  onBus,
  topics,
});
