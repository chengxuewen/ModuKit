/* examples/cpp/host_demo.cc - the Stage-4 M1 C++ embed demo (plan §5 M1:
   "one host embedding the kernel, walking the full v1 word list"; proof
   that the HostEmbedV1 face is not paper). The application calls ONLY the
   generated wrapper (bindings/cpp shelf, face B: modukit_embed.hpp) and
   the stock protoc messages (C++-1); the kernel behind the symbols is the
   ambient Rust process-global in libmodukit_core.a (crates/modukit-core/
   src/embed.rs). Fixture leg: the real built C++ plugin .so (argv[1],
   baked by ctest via $<TARGET_FILE:cpp-example-plugin>).

   Word list, one line per verb, in one honest chain:
     embed_abi_version -> install -> resolve -> start -> state
     -> service_call (EchoRequest/EchoReply over the demo contract)
     -> bus_publish -> bus_subscribe -> bus_recv -> bus_unsubscribe
     -> clock_now -> clock_advance (Real domain: pinned REFUSAL, not a
        silent no-op - the header's rule, asserted as an expected code)
     -> introspect_snapshot -> introspect_events (the opaque-bytes codec
        documented in embed.rs/ embed_codec.rs is parsed field by field)
     -> stop -> uninstall.
   Prints `CPP-DEMO PASS` only when every step asserted clean; exit 77 =
   skip when the fixture is absent; DEMO_STAGE4_SABOTAGE=1 corrupts the
   manifest at exactly ONE named point (scene [2]) so the gate must RED. */
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#include <filesystem>

#include "com.modukit.demo.pb.h"   /* stock protoc messages (C++-1) */
#include "modukit_embed.hpp"       /* face B: the generated embed wrapper */

namespace {

const std::string kPluginId = "com.example.cpp.demo";
const char *const kServiceId = "com.modukit.demo.DemoService";
const char *const kTopic = "com.example.cpp.demo.status";
const char *const kPayload = "embed-demo-payload";

int failures = 0;

void check(bool ok, const char *what) {
  if (!ok) {
    std::printf("FAIL: %s\n", what);
    ++failures;
  } else {
    std::printf("ok:   %s\n", what);
  }
}

/* Little-endian field readers for the documented event record codec
   (embed_codec.rs: i32 kind | u64 seq | u64 ts_ns | u64 len | payload). */
uint32_t rd32le(const uint8_t *p) {
  return static_cast<uint32_t>(p[0]) | (static_cast<uint32_t>(p[1]) << 8) |
         (static_cast<uint32_t>(p[2]) << 16) | (static_cast<uint32_t>(p[3]) << 24);
}
uint64_t rd64le(const uint8_t *p) {
  uint64_t v = 0;
  for (int i = 7; i >= 0; --i) v = (v << 8) | p[i];
  return v;
}

/* scene [2] build point: the deploy-tier manifest TOML for the fixture. */
std::string manifest_for(const std::string &so_path) {
  if (std::getenv("DEMO_STAGE4_SABOTAGE") != nullptr) {
    /* THE single sabotage point (documented in scripts/demo-stage4-m1.sh):
       an unknown placement must be refused at the install door. */
    std::printf("[sabotage] DEMO_STAGE4_SABOTAGE=1 -> corrupting manifest\n");
    return "id = \"" + kPluginId + "\"\nversion = \"0.1.0\"\n"
           "[entry]\nkind = \"dlx\"\nartifact = \"" + so_path + "\"\n";
  }
  return "id = \"" + kPluginId + "\"\nversion = \"0.1.0\"\n"
         "[entry]\nkind = \"dl\"\nartifact = \"" + so_path + "\"\n";
}

}  // namespace

int main(int argc, char **argv) {
  if (argc < 2) {
    std::printf("SKIP: usage: host_demo <path-to-libcpp-example-plugin.so>\n");
    return 77;
  }
  const std::string so_path = argv[1];
  if (!std::filesystem::exists(so_path)) {
    std::printf("SKIP: fixture .so missing at %s\n", so_path.c_str());
    return 77;
  }

  std::printf("=== ModuKit Stage-4 M1 C++ embed demo ===\n");
  const modukit::Kernel kernel;  /* the ambient process-global kernel */

  /* [1] embed_abi_version: the wrapper's own handshake probe. */
  check(kernel.abi_version() == MODUKIT_EMBED_ABI_VERSION, "[1] embed_abi_version == 1");

  /* [2] install: manifest-v0 TOML in, kernel-owned canonical id out. */
  const std::string toml = manifest_for(so_path);
  const char *plugin_id = nullptr;
  check(kernel.install(toml.c_str(), &plugin_id).ok() && plugin_id != nullptr &&
            std::strcmp(plugin_id, kPluginId.c_str()) == 0,
        "[2] install -> canonical id (kernel-owned bytes)");
  if (failures != 0) {
    std::printf("CPP-DEMO FAIL (%d, install door)\n", failures);
    return 1;  /* nothing downstream is reachable without a plugin */
  }

  /* [3] resolve + [4] start + [5] state: the dl handshake and the walk. */
  check(kernel.resolve(plugin_id).ok(), "[3] resolve -> ABI v1 handshake green");
  check(kernel.start(plugin_id).ok(), "[4] start -> ACTIVE");
  int32_t state = -1;
  check(kernel.state(plugin_id, &state).ok() &&
            state == static_cast<int32_t>(modukit::PluginState::STATE_ACTIVE),
        "[5] state == ACTIVE (4)");

  /* [6] service_call: typed round trip over the frozen message shape. */
  com::modukit::demo::EchoRequest req;
  req.set_payload(kPayload);
  const std::string req_bytes = req.SerializeAsString();
  std::vector<uint8_t> resp(sizeof(com::modukit::demo::EchoRequest) + 256, 0);
  const modukit::Status call =
      kernel.service_call(kServiceId, "Echo",
                          reinterpret_cast<const uint8_t *>(req_bytes.data()),
                          req_bytes.size(), resp.data(), resp.size());
  bool echo_ok = false;
  if (call.ok()) {
    com::modukit::demo::EchoReply reply;
    echo_ok = reply.ParseFromArray(resp.data(), call.raw()) &&
              reply.payload() == kPayload &&
              reply.status() == com::modukit::demo::DEMO_STATUS_OK;
  }
  check(echo_ok, "[6] service_call Echo round-trips the payload (status OK)");
  check(kernel.service_call("com.example.nope", "Echo",
                            reinterpret_cast<const uint8_t *>(req_bytes.data()),
                            req_bytes.size(), resp.data(), resp.size()).code() ==
            modukit::KernelCode::NOT_FOUND,
        "[6] service_call unknown id -> NOT_FOUND(-14)");

  /* [7] bus: publish, late-join subscribe, recv (cap discipline honored). */
  const std::string bus_value = "alpha-signal";
  check(kernel.bus_publish(kTopic, static_cast<int32_t>(modukit::Qos::QOS_LATEST_VALUE),
                           reinterpret_cast<const uint8_t *>(bus_value.data()),
                           bus_value.size())
            .ok(),
        "[7] bus_publish latest-value");
  int32_t queue_id = -1;
  const modukit::Status sub = kernel.bus_subscribe(kTopic, &queue_id);
  check(sub.ok() && sub.raw() == queue_id && queue_id >= 1,
        "[7] bus_subscribe -> kernel-owned queue id");
  size_t got_len = 0;
  uint8_t small[4];
  check(kernel.bus_recv(queue_id, small, sizeof(small), &got_len)
                .code() == modukit::KernelCode::QUOTA_EXCEEDED &&
            got_len == bus_value.size(),
        "[7] bus_recv too-small cap -> QUOTA_EXCEEDED, need reported");
  std::vector<uint8_t> out(bus_value.size() + 16, 0);
  const modukit::Status recv =
      kernel.bus_recv(queue_id, out.data(), out.size(), &got_len);
  check(recv.ok() && got_len == bus_value.size() &&
            std::memcmp(out.data(), bus_value.data(), bus_value.size()) == 0,
        "[7] bus_recv (late join) gets the exact value back");
  check(kernel.bus_unsubscribe(queue_id).ok() &&
            kernel.bus_recv(queue_id, out.data(), out.size(), &got_len)
                    .code() == modukit::KernelCode::NOT_FOUND,
        "[7] bus_unsubscribe retires the queue id");

  /* [8] clock: the ambient kernel runs the Real domain (embed.rs decision
     1), so now() answers and the dial REFUSES — pinned code, documented. */
  int64_t now_ns = -1;
  check(kernel.clock_now(&now_ns).ok() && now_ns >= 0, "[8] clock_now >= 0 ns");
  check(kernel.clock_advance(1000000).code() == modukit::KernelCode::CLOCK_DIAL_REFUSED,
        "[8] clock_advance on Real -> CLOCK_DIAL_REFUSED (never a no-op)");

  /* [9] introspect_snapshot: size query then exact copy; the documented
     line codec parses and carries this plugin's ACTIVE row. */
  size_t snap_len = 0;
  const modukit::Status probe = kernel.introspect_snapshot(nullptr, 0, &snap_len);
  check(probe.ok() && probe.raw() == static_cast<int32_t>(snap_len) && snap_len > 0,
        "[9] introspect_snapshot size query (NULL buffer)");
  std::vector<uint8_t> snap(snap_len, 0);
  size_t copied = 0;
  const modukit::Status take = kernel.introspect_snapshot(snap.data(), snap.size(), &copied);
  bool snap_ok = false;
  if (take.ok() && copied == snap_len) {
    const std::string text(reinterpret_cast<const char *>(snap.data()), snap_len);
    snap_ok = text.find("plugin\t" + kPluginId + "\tACTIVE") != std::string::npos &&
              text.find("service\torg.modukit.snapshot\t") != std::string::npos &&
              text.find("counters\t") != std::string::npos;
  }
  check(snap_ok, "[9] introspect_snapshot: documented line codec parses");

  /* [10] introspect_events: resumable LE record walk; the lifecycle
          transitions of THIS plugin must be on the ring. */
  size_t ev_len = 0;
  uint64_t cursor = 0;
  const modukit::Status ev_probe =
      kernel.introspect_events(0, nullptr, 0, &ev_len, &cursor);
  check(ev_probe.ok() && ev_len > 28, "[10] introspect_events size query");
  std::vector<uint8_t> events(ev_len, 0);
  size_t ev_copied = 0;
  const modukit::Status ev_take =
      kernel.introspect_events(0, events.data(), events.size(), &ev_copied, &cursor);
  bool saw_lifecycle = false, tiled = false;
  if (ev_take.ok()) {
    size_t off = 0;
    uint64_t last_seq = 0;
    while (off + 28 <= ev_copied) {
      const uint32_t kind = rd32le(events.data() + off);
      const uint64_t seq = rd64le(events.data() + off + 4);
      const uint64_t plen = rd64le(events.data() + off + 20);
      if (kind == static_cast<uint32_t>(modukit::EventKind::EVENT_LIFECYCLE) &&
          std::memcmp(events.data() + off + 28, kPluginId.c_str(), kPluginId.size()) == 0) {
        saw_lifecycle = true;
      }
      last_seq = seq;
      off += 28 + static_cast<size_t>(plen);
    }
    tiled = off == ev_copied && cursor == last_seq;
  }
  check(saw_lifecycle && tiled,
        "[10] introspect_events: LE records tile the buffer; lifecycle rows found");

  /* [11] stop + uninstall close the walk (and the plugin's services). */
  check(kernel.stop(plugin_id).ok(), "[11] stop -> RESOLVED");
  check(kernel.state(plugin_id, &state).ok() &&
            state == static_cast<int32_t>(modukit::PluginState::STATE_RESOLVED),
        "[11] state == RESOLVED (6)");
  check(kernel.uninstall(plugin_id).ok(), "[11] uninstall -> UNINSTALLED");
  check(kernel.state(plugin_id, &state).ok() &&
            state == static_cast<int32_t>(modukit::PluginState::STATE_UNINSTALLED),
        "[11] state == UNINSTALLED (0)");
  check(kernel.resolve(plugin_id).code() == modukit::KernelCode::CONTRACT_VIOLATION,
        "[11] resolve from UNINSTALLED -> CONTRACT_VIOLATION (table guard)");

  if (failures != 0) {
    std::printf("CPP-DEMO FAIL (%d)\n", failures);
    return 1;
  }
  std::printf("CPP-DEMO PASS\n");
  return 0;
}
