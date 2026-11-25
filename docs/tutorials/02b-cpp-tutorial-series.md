# Tutorial 02b - the C++ tutorial series (progressive)

CppMicroServices-style: ONE minimal project per step, each compiled AND
executed by the cmake graph (the admission rule: a project joins the
series iff a standing leg runs it), driven by the shared drivers. The
projects live in `examples/cpp/tutorial/` - this page is the guided tour;
the code is the truth (read each file top to bottom, it is all there is).

## The eight steps

| Step | Project | The lesson | Source |
|---|---|---|---|
| t1 | `t1-hello-bundle/` | a plugin is a DESCRIPTOR: entry symbol + abi_version + size + start/stop. Nothing else is required | `t1.cpp`, the `T1_DESCRIPTOR` initializer |
| t2 | `t2-echo-bundle/` | a service is ONE register call through the borrowed host table; the call slot is your whole protocol | `t2.cpp`, the `t2_start` body |
| t3 | `t3-bus-bundle/` | the v2 door: subscribe-your-own-topic first (the slot RETURNS the queue id), publish, read the self-echo back - all inside start | `t3.cpp`, the `t3_start` mailbox walk |
| t4 | `t4-driver/` | the HOST side: the generated wrapper walks install -> resolve -> start -> state -> service_call -> stop -> uninstall | `t4_driver.cc`, `main` |
| t5 | `t5-consumer-poll/` | the NAIVE consumer: call by well-known id; the typed NOT_FOUND while the provider is absent is the lesson (dictionaryclient) | `t5_consumer.cc`, `main` |
| t6 | `t6-consumer-filter/` | the FILTER consumer: discover with the S1 query first (empty set = data, never an error), then call (dictionaryclient3) | `t6_consumer.cc`, `main` |
| t7 | `t7-compose-bundle/` | COMPOSITION across the wire: a plugin cannot call another plugin directly (no call slot on the host table), so the topic is the seam - announce on the reply topic, subscribe the request topic, expose the aggregate service (spellcheckservice) | `t7.cpp`, the `t7_start` body |
| t8 | `t8-compose-walk/` | the composed WALK: install the composer, late-join its announcement, drive the aggregate service (request + seed), verify the request topic, close (spellcheckclient) | `t8_compose.cc`, `main` |

## Run it

```bash
bash scripts/demo-stage4-m1.sh    # ends: PASS (cpp + c + tutorial)
```

The leg builds the whole series and runs the drivers five times: t4
against t2 (the echo scene exercises `service_call`) and t1 (the bare
lifecycle walk - the echo scene self-skips because t1 registers nothing),
the consumer evolution t5 (typed absence -> same call round-trips) and
t6 (discover, then call), then the composition pair t8 against t7 (the
announcement late-join, the composed call). Each run prints its own PASS
line; t7 itself is exercised by t8 (a plugin's start is not driven
directly).

## What each step teaches (quoted, with provenance)

Provenance anchors name the SYMBOL to read (line numbers rot; symbols do
not - ruling B3). Every quote is verbatim from the compiled fixture.

**t1** - the whole plugin, the `T1_DESCRIPTOR` initializer in
`examples/cpp/tutorial/t1-hello-bundle/t1.cpp`:

```cpp
const modukit_bundle_abi_v1 T1_DESCRIPTOR = {
    MODUKIT_ABI_VERSION,           /* abi_version - handshake guard 2 */
    sizeof(modukit_bundle_abi_v1), /* size - handshake guard 3 */
    "t1-hello-bundle",             /* name */
    "com.example.t1.hello",        /* id (D9 canonical reverse-DNS) */
    "0.1.0",                       /* version */
    t1_start,                      /* start - guard 4 */
    t1_stop,                       /* stop - guard 4 */
};
```

**t2** - the activation, the `t2_start` body in
`examples/cpp/tutorial/t2-echo-bundle/t2.cpp`:

```cpp
extern "C" int32_t t2_start(const modukit_host_abi_v1 *host) {
  if (host == nullptr) {
    return -1;
  }
  if (host->size < sizeof(modukit_host_abi_v1)) {
    return -2;
  }
  const int64_t rid = host->register_service(host->user, kServiceId, t2_call);
  return rid < 0 ? -3 : 0;
}
```

**t3** - the self-echo discipline, the mailbox walk in
`examples/cpp/tutorial/t3-bus-bundle/t3.cpp`:

```cpp
  /* 1. subscribe first: the publisher must hear its own publish. */
  int32_t queue = -1;
  /* the contract: bus_subscribe RETURNS the queue id (embed.h:186). */
  const int32_t sub_q = v2->bus_subscribe(v2->user, kTopic, &queue);
  if (sub_q != queue || queue < 1) {
    return kSub;
  }
  /* 2. publish the latest-value payload. */
  if (v2->bus_publish(v2->user, kTopic, kQosLatestValue,
                      reinterpret_cast<const uint8_t *>(kPayload),
                      std::strlen(kPayload)) != 0) {
    return kPub;
  }
```

**t4** - the driver's walk is the same chain the C host demo teaches
(chapter 05's C subsection); here it rides the generated wrapper.

**t7** - the composition seam, the `t7_start` body in
`examples/cpp/tutorial/t7-compose-bundle/t7.cpp`:

```cpp
  if (v2->bus_publish(v2->user, kReplyTopic, kQosLatestValue,
                      reinterpret_cast<const uint8_t *>(kSeed),
                      std::strlen(kSeed)) != 0) {
    return kPub;
  }
  int32_t queue = -1;
  const int32_t sub_q = v2->bus_subscribe(v2->user, kRequestTopic, &queue);
  if (sub_q != queue || queue < 1) {
    return kSub;
  }
  const int64_t rid = v2->register_service(v2->user, kServiceId, t7_call);
  return rid < 0 ? -3 : 0;
```

**t8** - the composed call, `main` in
`examples/cpp/tutorial/t8-compose-walk/t8_compose.cc`:

```cpp
  const modukit::Status call = kernel.service_call(
      kServiceId, "echo", reinterpret_cast<const uint8_t *>(req.data()),
      req.size(), resp.data(), resp.size());
  const std::string want = std::string(kRequest) + kSeed;
  check(call.ok() && call.raw() == static_cast<int32_t>(want.size()) &&
            std::memcmp(resp.data(), want.data(), want.size()) == 0,
        "the aggregate service composes request + seed");
```

## What the series does NOT teach yet (ruling B4)

The tracker, the dependency monitor (`on_down`), the config hot lane, and
the service-event stream are shipped ENGINE surfaces - but their faces
are Rust-only today (no embed verbs for them, so no C++/py/js band can
reach them from a tutorial). The honest paths for a C++ reader today:
the rs topic examples (`examples/rs/{tracker,dm,config,ops}_demo.rs` on
the `demo-rs-examples` leg) show each engine at work; the canon rows live
in [../features.md](../features.md). When an embed tail-add lands for
any of these, the series grows a step and this block shrinks.

## Where this fits

- Chapter 02 taught the SHAPE with the canonical fixtures; this series is
  the SAME handshake, minimal by construction, eight steps instead of one
  84-line file.
- The raw-ABI choice is deliberate: t1-t3 and t7 include
  `contracts/modukit_bundle.h` only - no generated shelf - so every line
  of the handshake is visible. The shelf (and when to prefer it) is
  chapter 02's C++ band section.
- Next: chapter 03 (the bus, with the rs mailbox walk), chapter 04 (the
  exe wall), chapter 05 (embedding from your language).
