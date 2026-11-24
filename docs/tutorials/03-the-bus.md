# Tutorial 03 - The bus

Chapter 01 showed a late-joiner receiving a value that was published before
it subscribed. This chapter is about that machine: topics, the
latest-value queue discipline, and the timer bell - including across the
process wall.

## The model (one paragraph)

Topics are named channels (`com.yourco.hmi.Speed`-style reverse-DNS ids).
Every topic keeps its semantics from the QoS the subscriber chooses;
`QoS::LatestValue` (the default) keeps only the current value, so a
late-joiner immediately receives it. Publishing is a host-level act;
plugins publish through their host table (chapter 06) or the embed face.
The canon is [../modules/06-topics-and-bus.md](../modules/06-topics-and-bus.md).

## See it in-process

```bash
bash scripts/demo-stage1.sh
```

Scene [6] of the transcript (`examples/rs/stage1_demo.rs`) is the
late-join: publish first, subscribe second, the value still arrives. That
is the whole v1 story - retention lives at the topic, not in the
subscriber.

## See it cross the wall

```bash
bash scripts/demo-stage5-n2.sh
```

The Stage-5 N2 leg hosts the `busv2` fixture
(`test-plugins/rs/busv2-plugin`) on the OTHER side of the process wall:
the plugin publishes through the mailbox slots of the v2 host table
(D96), the host pump forwards it kernel-side, and the demo asserts the
topic arrives. This is the exe-wall preview - chapter 04 explains the
wall itself.

## The timer bell

```bash
bash scripts/demo-wave2-clock.sh
```

A v3 plugin schedules one absolute-deadline job (`timer_schedule`, D99);
the kernel fires it on the clock's own authority law and rings the per-
plugin topic `org.modukit.timer.<plugin-id>` (payload: token u64 LE ++
label UTF-8, `Lossy(16)`). There is no cancel verb - the kernel releases
a stopped plugin's tokens itself (D99 §3). The fixture is
`test-plugins/rs/timerv3-plugin`.

## Configuration rides the same rails

A plugin that declares `config_live = ["key", ...]` receives value
changes as ordinary topics - same publish/subscribe grammar, no second
mechanism (D29; the face landed in S3-2,
[../modules/05-manifest-and-deployment.md](../modules/05-manifest-and-deployment.md)
§6). The demo leg for it is a recorded future item - the canon link is
the honest path until a leg ships.

## Next

Chapter 04: the exe wall - why some plugins run as processes, what the
supervisor does when they die, and why the census counts orphans.
