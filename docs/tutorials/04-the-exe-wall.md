# Tutorial 04 - The exe wall

Chapter 02's bundles were `dl`-placed: the kernel dlopens them into its own
process. The second placement is `exe`: the kernel SPAWNS the bundle as a
child process, and everything between them crosses a wire - a narrow,
frozen frame set (D87) with 11 wire kinds. This chapter runs that wall and
then breaks it on purpose.

## The model (one paragraph)

An `exe` manifest names a `runner` (an absolute path to the process
launcher - typically `modukit-run`) plus the artifact the runner loads.
At resolve the kernel spawns the child and waits for the HELLO handshake,
which self-reports identity and is cross-checked against the manifest
(the D7 cross-check: silence or a lie is refusal). At start the runner
receives START and registers services back through a proxy - kernel-side
consumers cannot tell an exe service from a dl one. The supervisor owns
the rung: crash-watch, restart budget (D3), and the ORPHAN-PROOF census.
Canon: [../modules/01-bundle-placement-model.md](../modules/01-bundle-placement-model.md),
[../modules/03-lifecycle-state-machine.md](../modules/03-lifecycle-state-machine.md) §4.

## See the wall hold

```bash
bash scripts/demo-stage2.sh
```

The transcript walks: exe bundle -> HELLO cross-check -> proxy echo
kernel-side -> wire dial -> crash/revive budget -> ORPHAN-PROOF (SIGKILL
the host, the runner self-dies, zero stray processes). The census at the
end is the point: a killed host leaves zero orphans. The proof's own
arrangement is quoted from `examples/rs/stage2_demo.rs:291-298`:

```rust
    // [12] ORPHAN-PROOF (new proof, nothing like it anywhere): modukit-host
    // as a REAL child process; its runner grandchild must die ALONE when the
    // host is SIGKILLed (runner control-EOF exit; F1 semantics).
    // Arrange: a fresh manifest for the host + a clean runner census.
    if count_runner_procs() != 0 {
        fail("runner census non-zero before the orphan-proof (leak from an earlier scene)");
    }
```

The census is checked BEFORE the kill (a leak from an earlier scene must
fail the proof, not silently pass into it) and again after - the demo
does not trust its own hygiene, it counts.

## See it from a C++-authored host

```bash
bash scripts/demo-stage4-m2.sh
```

`examples/cpp/host_sup_demo.cc` drives the sup_* verbs (sup_install ->
dial -> READY -> sup_service_call, the FIDELITY stanza byte-pinned to the
cpp constant), then kills the host and proves the runner self-reap census
again - ORPHAN-PROOF under a foreign host. The m1 leg of this chapter
pair also carries the C band: the same leg ends with `C-DEMO PASS`
(chapter 02).

## The sabotage levers

Every exe leg documents how to make it RED: stage2 and m2 carry
`DEMO_STAGE2_SABOTAGE`-class hooks, and the script-bundle legs prove
theirs inline on every green run (chapter 06). Run a leg with its lever
set, watch the typed refusal, then re-run green - the refusals are the
product, not the failure.

## Why a wall at all

The wall buys what dl cannot give: a crashed or crashed-ON-PURPOSE bundle
cannot take the kernel down (01 §4: the host survives, the bundle lands
on the FAILED edge, the supervisor revives it inside the D3 budget). It
is also the seam the script languages cross - chapter 06's Python and JS
bundles are exe children speaking the same 11 wire kinds. The honest
asymmetry (native code cannot be preempted; stop budgets are
documentation at v0 for `dl`) is written down in
[../modules/03-lifecycle-state-machine.md](../modules/03-lifecycle-state-machine.md) §5.

## Next

Chapter 05: embed the kernel from YOUR language - including plain C.
