/* tutorial t7 - bus-mediated composition (the spellcheck shape: one
 * service consuming ANOTHER service's work, the ModuKit way). A plugin
 * cannot call another plugin directly (the host table has no call slot -
 * the wire grammar is publish/subscribe), so t3's topic discipline
 * becomes the composition seam: t7 subscribes to a request topic,
 * computes an aggregate, and publishes the reply topic. The HOST drives
 * the round trip through the embed verbs (t7_aggregate request -> reply).
 * Gated by the m1 leg (zero new legs), which passes the t2 .so as
 * argv[1]; t7 is built and its walk is asserted by ctest t7-composition. */
#include <cstring>

#include "modukit_plugin.h"

namespace {

const char *const kPluginId = "com.example.t7.compose";
const char *const kServiceId = "com.example.t7.aggregate";
const char *const kRequestTopic = "com.example.t7.request";
const char *const kReplyTopic = "com.example.t7.reply";
const char *const kSeed = "t7-compose-seed";
const int32_t kQosLatestValue = 0;
const int32_t kBadRequest = -101;

/* enum-named steps so a failing code NAMES the step (the t3 discipline). */
enum : int32_t {
  kSub = -111,
  kPub = -112,
};

extern "C" int32_t t7_call(const char *method, const uint8_t *input,
                           size_t input_len, uint8_t *output,
                           size_t output_cap) {
  if (method == nullptr || output == nullptr ||
      (input_len > 0 && input == nullptr)) {
    return kBadRequest;
  }
  if (std::strcmp(method, "echo") != 0 || input_len > output_cap) {
    return kBadRequest;
  }
  /* the aggregate answer: the request plus the seed - the plugin's own
   * "composed work", standing in for the spellcheck math. */
  const size_t seed_len = std::strlen(kSeed);
  if (input_len + seed_len > output_cap) {
    return kBadRequest;
  }
  std::memcpy(output, input, input_len);
  std::memcpy(output + input_len, kSeed, seed_len);
  return static_cast<int32_t>(input_len + seed_len);
}

extern "C" int32_t t7_start(const modukit_host_abi_v1 *host) {
  if (host == nullptr) {
    return -1;
  }
  if (host->size < sizeof(modukit_host_abi_v2)) {
    return -2;
  }
  const auto *v2 = reinterpret_cast<const modukit_host_abi_v2 *>(host);
  /* the composition seam: announce on the reply topic that this composer
   * is live (the seed a driving host late-joins to verify), then expose
   * the aggregate service through the unchanged v1 prefix. */
  if (v2->bus_publish(v2->user, kReplyTopic, kQosLatestValue,
                      reinterpret_cast<const uint8_t *>(kSeed),
                      std::strlen(kSeed)) != 0) {
    return kPub;
  }
  /* the contract: bus_subscribe RETURNS the queue id (embed.h:186) - not
   * a zero status. The queue id IS the return value. */
  int32_t queue = -1;
  const int32_t sub_q = v2->bus_subscribe(v2->user, kRequestTopic, &queue);
  if (sub_q != queue || queue < 1) {
    return kSub;
  }
  const int64_t rid =
      v2->register_service(v2->user, kServiceId, t7_call);
  return rid < 0 ? -3 : 0;
}

extern "C" int32_t t7_stop(void) { return 0; }

const modukit_plugin_abi_v1 T7_DESCRIPTOR = {
    MODUKIT_ABI_VERSION_V2,
    sizeof(modukit_plugin_abi_v1),
    "t7-compose-plugin",
    kPluginId,
    "0.1.0",
    t7_start,
    t7_stop,
};

}  // namespace

extern "C" const modukit_plugin_abi_v1 *modukit_plugin_init(void) {
  return &T7_DESCRIPTOR;
}
