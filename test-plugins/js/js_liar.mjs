#!/usr/bin/env node
// test-plugins/js/js_liar.mjs - the wave-2 plugin-door liar fixture (D106
// batch 3, plan T3.2): declares ONE service and answers every CALL with
// RESULT status=-1 (a D27 author code) and empty output. THE LEG CONSUMES
// IT: the kernel-side proxy must surface the lie as the typed
// SERVICE_CALL_FAILED on the embed channel while the plugin stays ALIVE and
// cooperative - the i32 status channel is a contract, not decoration
// (proc_wire.rs:48-53 doc; proc_proxy.rs returns the author code without
// killing). No bus topics: this skin also proves the tail-less legacy
// HELLO grammar path end to end (caps 0 -> no 4-byte tail).
// Runtime resolution: same two-candidate rule as js_echo.mjs (env wins,
// in-place relative fallback).

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';

const ID = 'org.modukit.demo.jsliar';
const VERSION = '0.1.0';
const NAME = 'js-liar';

function findSkin() {
  const cands = [];
  const env = process.env.MODUKIT_JS_SKIN;
  if (env) cands.push(env);
  const here = path.dirname(fileURLToPath(import.meta.url));
  cands.push(path.normalize(path.join(here, '..', '..', 'bindings', 'js')));
  for (const c of cands) {
    if (fs.existsSync(path.join(c, 'plugin_runtime.mjs'))) return c;
  }
  process.exit(5);
}

const rt = await import(`file://${path.join(findSkin(), 'plugin_runtime.mjs')}`);

async function onCall(method, inp) {
  void method;
  void inp;
  return [-1, Buffer.alloc(0)]; // the lie, verbatim on the D27 channel
}

await rt.serve({
  id: ID,
  version: VERSION,
  name: NAME,
  services: { [ID]: ['lie'] },
  onCall,
});
