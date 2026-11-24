# Tutorial 01 - First run

Sixty seconds from a fresh clone to a full plugin lifecycle, and the mental
model that explains every line of output.

## Prerequisites

Linux (or WSL on Windows). One command, idempotent:

```bash
./bootstrap.sh
```

What it does: self-heals the `PolyOrch` submodule, installs pixi, and
provisions the pinned toolchain into `.pixi/envs/default` (cmake, ninja,
protoc, the triplet gcc, rust). First run costs a few minutes of downloads;
after that everything is offline.

Open a fresh terminal afterwards (bootstrap tells you), so `pixi` is on
PATH. The full environment walkthrough lives in
[../env.md](../env.md) - this chapter runs the demo, that page explains the
territories.

## Run it

```bash
pixi run demo stage1
```

(or the long form: `bash scripts/demo-stage1.sh`). First run includes a
cold cargo build of the core; warm runs are seconds. You should see the
transcript end with:

```text
STAGE1-DEMO PASS
```

## What just happened

The Rust demo `examples/rs/stage1_demo.rs` (quoted from, see that file for
the authoritative transcript) walks, in order:

1. **A fixture cdylib is built** - the plugin is an ordinary shared library.
2. **The D28 install protocol** - a bundle archive is staged, verified, and
   installed over a fresh host tree.
3. **A tampered bundle is refused at the install door** - RED before the
   install root is touched (negatives are part of the proof).
4. **Lifecycle walk** - install -> resolve (the dl handshake) -> start ->
   ACTIVE -> stop -> uninstall, the six-state machine of
   [../modules/03-lifecycle-state-machine.md](../modules/03-lifecycle-state-machine.md).
5. **Service echo** - the plugin registered a service; the host calls it.
6. **Bus late-join** - a subscriber that arrives after the publish still
   gets the current value (`QoS::LatestValue`,
   [../modules/06-topics-and-bus.md](../modules/06-topics-and-bus.md)).

The two most important ideas for everything that follows:

- **The install door is a gate, not a formality**: identity, placement, and
  manifest grammar are checked before the plugin root exists. Chapter 02
  makes you trip it on purpose.
- **Everything is a service**: the plugin published one; the host consumed
  one; the whole registry is queryable (chapter 03's tracker and the S1
  filter ride the same registry).

## One command for every demo

`pixi run demo <leg>` runs any gallery leg (`bash scripts/demo.sh` under
the hood). A bare `pixi run demo` runs this chapter's leg; an unknown leg
prints the full list. The same legs are standing members of the local gate
(`bash scripts/check.sh`) - what you just ran is what CI replays.

## Next

Chapter 02: write your own plugin - and watch the gate refuse a bad one.
