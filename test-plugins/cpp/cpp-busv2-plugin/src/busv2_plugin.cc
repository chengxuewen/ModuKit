/* cpp-busv2-plugin - the D96 v2-door C++ fixture (onboarding deepening D2;
 * the C++ twin of the rs busv2-plugin, D23 template standing). Declares
 * ABI_VERSION_V2: the kernel hands the HostAbiV2 table (the v1 prefix plus
 * the four mailbox slots) at start, and the plugin publishes ONE seed
 * value on its own topic - the LatestValue snapshot a late-join host then
 * reads. The whole publish runs INSIDE start (01 §3 rule 2: the table is
 * borrowed for the call); the registered echo service proves the v1
 * prefix still answers calls after the mailbox work. */
#include <cstring>

#include "modukit_plugin.h"

namespace {

const char *const kPluginId = "com.example.cpp.busv2";
const char *const kTopic = "com.example.cpp.busv2";
const char *const kServiceId = "com.example.cpp.busv2.echo";
const char *const kSeed = "cpp-busv2-seed";
const int32_t kQosLatestValue = 0; /* the embed modukit_qos vocabulary */
const int32_t kBadRequest = -101;  /* plugin-author code space (D27) */

/* The v1-prefix echo: message-shaped in/out, nothing else. */
extern "C" int32_t busv2_call(const char *method, const uint8_t *input,
                              size_t input_len, uint8_t *output,
                              size_t output_cap) {
  if (method == nullptr || output == nullptr ||
      (input_len > 0 && input == nullptr)) {
    return kBadRequest;
  }
  if (std::strcmp(method, "echo") != 0 || input_len > output_cap) {
    return kBadRequest;
  }
  std::memcpy(output, input, input_len);
  return static_cast<int32_t>(input_len);
}

/* Activation through the v2 door: publish the seed via the mailbox slot,
 * then register the echo service through the unchanged v1 slot. */
extern "C" int32_t busv2_start(const modukit_host_abi_v1 *host) {
  if (host == nullptr) {
    return -1;
  }
  /* the v2 tail is what this fixture exists to exercise; a v1 host table
   * would fail the size guard here, named. */
  if (host->size < sizeof(modukit_host_abi_v2)) {
    return -2;
  }
  const auto *v2 = reinterpret_cast<const modukit_host_abi_v2 *>(host);
  if (v2->bus_publish(v2->user, kTopic, kQosLatestValue,
                      reinterpret_cast<const uint8_t *>(kSeed),
                      std::strlen(kSeed)) != 0) {
    return -4;
  }
  const int64_t rid = v2->register_service(v2->user, kServiceId, busv2_call);
  return rid < 0 ? -3 : 0;
}

extern "C" int32_t busv2_stop(void) {
  return 0; /* cooperative no-op; idempotent by construction (01 §3 rule 6) */
}

const modukit_plugin_abi_v1 BUSV2_DESCRIPTOR = {
    MODUKIT_ABI_VERSION_V2,        /* abi_version - the v2 door (D96) */
    sizeof(modukit_plugin_abi_v1), /* size - handshake guard 3 */
    "cpp-busv2-plugin",            /* name */
    kPluginId,                     /* id == published topic owner (D9) */
    "0.1.0",                       /* version */
    busv2_start,                   /* start - guard 4 */
    busv2_stop,                    /* stop - guard 4 */
};

}  // namespace

extern "C" const modukit_plugin_abi_v1 *modukit_plugin_init(void) {
  return &BUSV2_DESCRIPTOR;
}
