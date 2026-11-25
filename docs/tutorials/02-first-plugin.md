# Tutorial 02 - Your first plugin

A plugin author writes exactly two things: the plugin binary that exports
one entry symbol, and a manifest that tells the kernel what it is and where
its binary lives. Everything else - verification, handshake, lifecycle -
is the kernel's job.

This chapter uses the two bands the repo compiles every night: **C++**
(the generated-shelf shape) and **C** (the raw-ABI shape, one source file).
Python and JS plugin authors jump to chapter 06.

## The manifest (three fields, one shape)

Quoted verbatim from `examples/cpp/host_demo.cc:73-74` (`manifest_for`):

```toml
id = "com.example.cpp.demo"
version = "0.1.0"

[entry]
kind = "dl"
artifact = "<path-to-the-built-plugin-.so>"
```

`id` is a canonical reverse-DNS id (D9). `kind = "dl"` means: the kernel
dlopens this artifact in-process. The full v0 grammar - `requires` with
version ranges, `on_down` policies, `config_live` - is
[../modules/05-manifest-and-deployment.md](../modules/05-manifest-and-deployment.md);
three fields are all a first plugin needs.

## The plugin binary (C++ band)

Quoted verbatim from
`test-plugins/cpp/cpp-example-bundle/src/plugin.cc:68-76` - the entire
self-report descriptor:

```cpp
const modukit_bundle_abi_v1 DESCRIPTOR = {
    MODUKIT_ABI_VERSION,   /* abi_version - guard 2 */
    sizeof(modukit_bundle_abi_v1), /* size - guard 3 */
    "cpp-example-bundle",  /* name */
    DemoServiceImpl::kServiceId, /* id == registered service id, mirroring the rs twin (D9) */
    "0.1.0",               /* version - plugin's own semver */
    demo_start,            /* start - guard 4 */
    demo_stop,             /* stop - guard 4 */
};
```

The handshake guards (04 §3) check exactly: the entry symbol resolves and
returns non-NULL; `abi_version` equals the host's; `size` is at least the
host's compiled size; `start`/`stop` are non-NULL. The generated shelf
(`bindings/cpp`) carries the message dispatch; the author implements the
service class above it (`plugin.cc:16-35`).

## The plugin binary (C band)

The C twin is the whole story in one hand-written file:
`test-plugins/c/c-example-bundle/src/hello.c` (43 lines, no generated
shelf, no C++ runtime). The descriptor is the same struct; the service is
`com.modukit.demo.hello` with one method `hello` that echoes the request
payload; the call slot returns a plugin-author negative code on anything
else (D27 code space). Read it top to bottom - it is the minimal complete
plugin.

## Run the walk (and trip the gate on purpose)

```bash
bash scripts/demo-stage4-m1.sh        # CPP-DEMO PASS + C-DEMO PASS
DEMO_STAGE4_SABOTAGE=1 bash scripts/demo-stage4-m1.sh   # expect RED, exit 1
```

The sabotage lever corrupts the manifest's `entry.kind` at exactly one
named point (`examples/cpp/host_demo.cc:66-71`) - watch the install door
refuse it, then re-run without the lever and watch it pass. The C host demo
carries the same lever (`DEMO_C_SABOTAGE=1`, `examples/c/host_demo.c`,
`manifest_for`). Falsifiability is part of the proof: every gallery leg
documents how to make it RED.

## The negative fixtures are canonical

`test-plugins/cpp/cpp-bad-bundle` and the rust `bad-bundle` are the
ABI-lie liars: a descriptor that lies about its ABI version parks the
plugin on the FAILED edge at resolve (D3's Failed edge exists for exactly
this). D23 ruled these fixtures the official plugin-starting answer - when
in doubt, copy the positive one and break it on purpose.

## Where the grammar lives

[../modules/05-manifest-and-deployment.md](../modules/05-manifest-and-deployment.md)
(grammar + the deploy-tier plan),
[../modules/02-service-model.md](../modules/02-service-model.md)
(registration semantics), and the walkthrough
[../modules/08-end-to-end-walkthrough.md](../modules/08-end-to-end-walkthrough.md)
§9 for the fixtures' standing.

## Next

Chapter 03: the bus - what late-join was doing in chapter 01, and how
topics and timers cross the process wall.
