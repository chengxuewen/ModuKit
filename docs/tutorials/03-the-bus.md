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
bundles publish through their host table (chapter 06) or the embed face.
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
(`test-bundles/rs/busv2-bundle`) on the OTHER side of the process wall:
the bundle declares ABI v2 (D96), the kernel hands the `HostAbiV2` table
(the v1 prefix plus four mailbox slots), and the bundle publishes through
the mailbox. The self-echo discipline is quoted from
`test-bundles/rs/busv2-bundle/src/lib.rs:73-88`:

```rust
fn mailbox_walk(h: &HostAbiV2) -> Result<(), i32> {
    // 1. subscribe first: the publisher must hear its own publish (ruling 6).
    let qid = h.bus_subscribe(TOPIC).map_err(|_| -101)?;
    if qid <= 0 {
        return Err(-101);
    }
    // 2. publish latest-value bytes -> 0 (broadcast, never blocks).
    if h.bus_publish(TOPIC, QOS_LATEST, b"self-echo") != 0 {
        return Err(-102);
    }
    // 3. self-echo is visible on the same call, no second publisher anywhere.
    let mut buf = [0u8; 32];
    let written = h.bus_recv(qid, &mut buf).map_err(|_| -103)?;
    if written != 9 || &buf[..written] != b"self-echo" {
        return Err(-103);
    }
```

A v2 bundle that publishes at start is the chapter 02 shape plus four
slots - the C++ twin (`test-bundles/cpp/cpp-busv2-bundle`) shows the same
door in 70 lines, and the cpp host demo's scene [11] reads its seed back
via late-join.

## The timer bell

```bash
bash scripts/demo-wave2-clock.sh
```

A v3 bundle schedules one absolute-deadline job (`timer_schedule`, D99);
the kernel fires it on the clock's own authority law and rings the per-
bundle topic `org.modukit.timer.<bundle-id>` (payload: token u64 LE ++
label UTF-8, `Lossy(16)`). The schedule call is quoted from
`test-bundles/rs/timerv3-bundle/src/lib.rs:100-106`:

```rust
    // 3. Schedule the one job the census and the bell ride on. The token is
    //    the kernel clock's monotonic counter; the door test decodes the
    //    exact value from the bell payload, this cell proves it came back.
    let token = h3.timer_schedule(deadline, JOB_LABEL).map_err(|_| -107)?;
    if token == 0 {
        return Err(-108);
    }
```

There is no cancel verb - the kernel releases a stopped bundle's tokens
itself (D99 §3), and the door must never half-schedule on bad input
(the fixture's refusal cells prove it). The fixture is
`test-bundles/rs/timerv3-bundle`.

## Configuration rides the same rails

A bundle that declares `config_live = ["key", ...]` receives value
changes as ordinary topics - same publish/subscribe grammar, no second
mechanism (D29; the face landed in S3-2,
[../modules/05-manifest-and-deployment.md](../modules/05-manifest-and-deployment.md)
§6). The demo leg for it is a recorded future item - the canon link is
the honest path until a leg ships.

## Next

Chapter 04: the exe wall - why some bundles run as processes, what the
supervisor does when they die, and why the census counts orphans.
