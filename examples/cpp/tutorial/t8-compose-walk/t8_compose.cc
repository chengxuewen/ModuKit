/* tutorial t8 - the composition WALK (the spellcheckclient shape): the
 * host drives t7's aggregate service end to end - install the composer,
 * read its start-time announcement via late-join, call the aggregate
 * service (request + seed = the composed answer), verify the request
 * topic carries the composer's subscription side, and close. Gated by
 * the m1 leg (argv[1] = the t7 .so). Prints T8-COMPOSE PASS. */
#include <cstdio>
#include <cstring>
#include <filesystem>
#include <string>
#include <vector>

#include "modukit_embed.hpp"

namespace {

const char *const kBundleId = "com.example.t7.compose";
const char *const kServiceId = "com.example.t7.aggregate";
const char *const kReplyTopic = "com.example.t7.reply";
const char *const kSeed = "t7-compose-seed";
const char *const kRequest = "req:";
int failures = 0;

void check(bool ok, const char *what) {
  std::printf("%s: %s\n", ok ? "ok  " : "FAIL", what);
  if (!ok) {
    ++failures;
  }
}

std::string manifest_for(const std::string &so_path) {
  return "id = \"" + std::string(kBundleId) + "\"\nversion = \"0.1.0\"\n"
         "[entry]\nkind = \"dl\"\nartifact = \"" + so_path + "\"\n";
}

}  // namespace

int main(int argc, char **argv) {
  if (argc < 2) {
    std::printf("SKIP: usage: t8-compose <path-to-t7-compose-plugin.so>\n");
    return 77;
  }
  const std::string so_path = argv[1];
  if (!std::filesystem::exists(so_path)) {
    std::printf("SKIP: plugin .so missing at %s\n", so_path.c_str());
    return 77;
  }

  const modukit::Kernel kernel;
  const std::string toml = manifest_for(so_path);
  const char *bundle_id = nullptr;
  check(kernel.install(toml.c_str(), &bundle_id).ok(), "install the composer");
  check(kernel.resolve(bundle_id).ok(), "resolve (the v2 handshake)");
  const modukit::Status started = kernel.start(bundle_id);
  char msg[160];
  std::snprintf(msg, sizeof(msg), "start (the announcement publishes) rc=%d", started.code());
  check(started.ok(), msg);

  /* the composer's start-time announcement, read via late-join. */
  int32_t queue = -1;
  check(kernel.bus_subscribe(kReplyTopic, &queue).ok(),
        "late-join the reply topic");
  size_t len = 0;
  std::vector<uint8_t> seed(64, 0);
  check(kernel.bus_recv(queue, seed.data(), seed.size(), &len).ok() &&
            std::memcmp(seed.data(), kSeed, len) == 0,
        "the start-time announcement arrives (LatestValue)");

  /* the composed call: request + seed = the aggregate answer. */
  const std::string req = kRequest;
  std::vector<uint8_t> resp(256, 0);
  const modukit::Status call = kernel.service_call(
      kServiceId, "echo", reinterpret_cast<const uint8_t *>(req.data()),
      req.size(), resp.data(), resp.size());
  const std::string want = std::string(kRequest) + kSeed;
  check(call.ok() && call.raw() == static_cast<int32_t>(want.size()) &&
            std::memcmp(resp.data(), want.data(), want.size()) == 0,
        "the aggregate service composes request + seed");

  /* the request topic exists with a subscriber (the composer's own
   * subscription side); publish is accepted kernel-side. */
  check(kernel
            .bus_publish("com.example.t7.request",
                         static_cast<int32_t>(modukit::Qos::QOS_LATEST_VALUE),
                         reinterpret_cast<const uint8_t *>(req.data()),
                         req.size())
            .ok(),
        "the request topic accepts publishes (the composer listens)");

  check(kernel.stop(bundle_id).ok() && kernel.uninstall(bundle_id).ok(),
        "the composer closes");

  if (failures != 0) {
    std::printf("T8-COMPOSE FAIL (%d)\n", failures);
    return 1;
  }
  std::printf("T8-COMPOSE PASS\n");
  return 0;
}
