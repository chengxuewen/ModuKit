# Tutorial 02b - the C++ tutorial series (progressive)

CppMicroServices-style: ONE minimal project per step, each compiled by the
cmake graph, driven by one shared `t4-driver`. The projects live in
`examples/cpp/tutorial/` - this page is the guided tour; the code is the
truth (read each file top to bottom, it is all there is).

## The four steps

| Step | Project | The lesson | Source |
|---|---|---|---|
| t1 | `t1-hello-plugin/` | a plugin is a DESCRIPTOR: entry symbol + abi_version + size + start/stop. Nothing else is required | `t1.cpp` |
| t2 | `t2-echo-plugin/` | a service is ONE register call through the borrowed host table; the call slot is your whole protocol | `t2.cpp` |
| t3 | `t3-bus-plugin/` | the v2 door: subscribe-your-own-topic first, publish, read the self-echo back - all inside start | `t3.cpp` |
| t4 | `t4-driver/` | the HOST side: the generated wrapper walks install -> resolve -> start -> state -> service_call -> stop -> uninstall | `t4_driver.cc` |
| t5 | `t5-consumer-poll/` | the NAIVE consumer: call by well-known id; the typed NOT_FOUND while the provider is absent is the lesson (dictionaryclient) | `t5_consumer.cc` |
| t6 | `t6-consumer-filter/` | the FILTER consumer: discover with the S1 query first (empty set = data, never an error), then call (dictionaryclient3) | `t6_consumer.cc` |

## Run it

```bash
bash scripts/demo-stage4-m1.sh    # ends: PASS (cpp + c + tutorial)
```

The leg builds the whole series and runs the drivers four times: t4
against t2 (the echo scene exercises `service_call`) and t1 (the bare
lifecycle walk - the echo scene self-skips because t1 registers nothing),
then the consumer evolution t5 (typed absence -> same call round-trips)
and t6 (discover, then call). Each run prints its own PASS line.

## What each step teaches (quoted, with provenance)

**t1** - the whole plugin, quoted from
`examples/cpp/tutorial/t1-hello-plugin/t1.cpp:33-41`:

```cpp
const modukit_plugin_abi_v1 T1_DESCRIPTOR = {
    MODUKIT_ABI_VERSION,           /* abi_version - handshake guard 2 */
    sizeof(modukit_plugin_abi_v1), /* size - handshake guard 3 */
    "t1-hello-plugin",             /* name */
    "com.example.t1.hello",        /* id (D9 canonical reverse-DNS) */
    "0.1.0",                       /* version */
    t1_start,                      /* start - guard 4 */
    t1_stop,                       /* stop - guard 4 */
};
```

**t2** - the activation, quoted from
`examples/cpp/tutorial/t2-echo-plugin/t2.cpp:44-51`:

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

**t3** - the self-echo discipline, quoted from
`examples/cpp/tutorial/t3-bus-plugin/t3.cpp:50-63`:

```cpp
  /* 1. subscribe first: the publisher must hear its own publish. */
  int32_t queue = -1;
  if (v2->bus_subscribe(v2->user, kTopic, &queue) != 0 || queue < 1) {
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

## Where this fits

- Chapter 02 taught the SHAPE with the canonical fixtures; this series is
  the SAME handshake, minimal by construction, four steps instead of one
  84-line file.
- The raw-ABI choice is deliberate: t1-t3 include
  `contracts/modukit_plugin.h` only - no generated shelf - so every line
  of the handshake is visible. The shelf (and when to prefer it) is
  chapter 02's C++ band section.
- Next: chapter 03 (the bus, with the rs mailbox walk), chapter 04 (the
  exe wall), chapter 05 (embedding from your language).
