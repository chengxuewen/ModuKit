# Tutorial 05 - Embed from your language

One kernel, four ways to drive it. Every band walks the SAME verb chain -
install -> resolve -> start -> state -> service_call -> ... -> stop - and
every host demo below is a standing gate leg, so what you read is what CI
runs. Counts and the full verb list live in
[../capability-ledger.md](../capability-ledger.md) (never in prose).

The kernel behind all four faces is the ambient Rust process-global in
`libmodukit_core` (`crates/modukit-core/src/embed.rs`); the bands differ
only in how they reach it. Chapter 02's manifest applies unchanged - each
demo builds one via its `manifest_for` helper.

## C - the identity story (plain C, no wrapper)

`examples/c/host_demo.c` includes the contract header and calls the verbs
directly - there is no generated layer to trust. The walk is quoted from
`examples/c/host_demo.c:73-85`:

```c
  /* [1] embed_abi_version: the handshake probe. */
  check(modukit_embed_abi_version() == MODUKIT_EMBED_ABI_VERSION,
        "[1] embed_abi_version == 1");

  /* [2] install: manifest-v0 TOML in, kernel-owned canonical id out. */
  manifest_for(so_path, toml, sizeof(toml));
  const char *plugin_id = NULL;
  const int32_t installed =
      modukit_install(toml, &plugin_id);
  check(installed == 0 && plugin_id != NULL &&
            strcmp(plugin_id, PLUGIN_ID) == 0,
        "[2] install -> canonical id (kernel-owned bytes)");
```

It drives install/resolve/start/state/service_call (the `hello` echo of
the C fixture)/find_services (the S1 filter query)/stop, then uninstalls.

```bash
bash scripts/demo-stage4-m1.sh    # ends: C-DEMO PASS (rides the cpp leg)
```

## C++ - the generated wrapper

`examples/cpp/host_demo.cc` drives the generated `HostEmbedV1` wrapper
(`bindings/cpp`, face B): typed `Status` results, the protoc messages, and
the same scenes the C demo mirrors. Run:

```bash
bash scripts/demo-stage4-m1.sh    # ends: CPP-DEMO PASS
```

The manifest build point is quoted from
`examples/cpp/host_demo.cc:73-74`; the walk starts at :93.

## Python - the generated ctypes floor

`examples/py/host_demo.py` uses `bindings/py/modukit_embed.py` - a
GENERATED ctypes floor (D100: re-emitted from `contracts/modukit_embed.h`
at gate time; DO NOT EDIT, D63). Run:

```bash
bash scripts/demo-wave2-py.sh     # the M1 walk (auto-skips without the cpp fixture)
```

The bind is quoted from `examples/py/host_demo.py:146-150` - the ctypes
floor wraps the loaded kernel in one call:

```python
    try:
        lib = skin.bind(ctypes.CDLL(core_so))
    except OSError as exc:
        print("SKIP: kernel .so not loadable (%s)" % exc)
        return 77
```

The manifest build point is quoted from `examples/py/host_demo.py:119-132`
(`manifest_for`, including the `DEMO_PY_SABOTAGE` lever). Note the auto-
SKIP copy on the leg: it tells you to run chapter 02's m1 first - the py
walk drives the real built cpp fixture `.so`.

## JS - the generated napi bridge

`examples/js/host_demo.js` uses the napi bridge (`crates/modukit-js`,
D102) through the generated facade `target/js/modukit_embed.js`. Run:

```bash
bash scripts/demo-wave2-js.sh     # the M1 walk (its SKIP copy names the prerequisite)
```

The import chain is quoted from `examples/js/host_demo.js:6-8` and
`:35-37` - the facade over the napi bridge, loaded like any module:

```js
 * facade (target/js/modukit_embed.js, re-emitted from
 * contracts/modukit_embed.h by scripts/demo-wave2-js.sh) over the generated
 * napi bridge (.node), which dlopens the ambient Rust process-global kernel
```

```js
const fs = require('node:fs');
const path = require('node:path');
const { Buffer } = require('node:buffer');
```

The word-list comment quoted from `examples/js/host_demo.js:12-16`
describes the honest chain; the leg header documents the pinned-env
`LD_PRELOAD` the fixture chain rides (Linux-only, WSL on Windows).

## What each band proves

| Band | Floor | Proven by |
|---|---|---|
| C | hand-written calls on `modukit_embed.h` | `examples/c/host_demo.c` (E2) |
| C++ | generated `modukit_cpp` wrapper | `examples/cpp/host_demo.cc` (m1 leg) |
| Python | generated ctypes floor | `examples/py/host_demo.py` (wave2-py leg) |
| JS | generated napi bridge + facade | `examples/js/host_demo.js` (wave2-js leg) |

## Next

Chapter 06: flip the roles - your Python or JS program BECOMES the plugin,
spawned by the kernel across the exe wall.
