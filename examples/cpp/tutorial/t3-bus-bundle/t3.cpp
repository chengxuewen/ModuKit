/* tutorial t3 - the bus: subscribe your own topic FIRST, then publish,
 * and read your own publish back (the D96 ruling-6 self-echo discipline,
 * the CppMS "talk between bundles" step, mailbox shape). Everything runs
 * inside start - the host table is borrowed for that call only (01 §3
 * rule 2), so the walk must complete here. */
#include <cstring>

#include "modukit_bundle.h"

namespace {

const char *const kTopic = "com.example.t3.bus";
const char *const kServiceId = "com.example.t3.echo";
const char *const kBundleId = "com.example.t3.bus";
const char *const kPayload = "t3-self-echo";
const int32_t kQosLatestValue = 0; /* the embed modukit_qos vocabulary */
const int32_t kBadRequest = -101;

/* steps of the mailbox walk; each failure names its step. */
enum : int32_t {
  kSub = -111,
  kPub = -112,
  kRecv = -113,
};

extern "C" int32_t t3_call(const char *method, const uint8_t *input,
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

extern "C" int32_t t3_start(const modukit_host_abi_v1 *host) {
  if (host == nullptr) {
    return -1;
  }
  if (host->size < sizeof(modukit_host_abi_v2)) {
    return -2; /* the mailbox slots need the v2 tail */
  }
  const auto *v2 = reinterpret_cast<const modukit_host_abi_v2 *>(host);

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
  /* 3. read the self-echo back (the retained snapshot, late-join style). */
  uint8_t buf[64];
  size_t len = 0;
  if (v2->bus_recv(v2->user, queue, buf, sizeof(buf), &len) !=
          static_cast<int32_t>(std::strlen(kPayload)) ||
      std::memcmp(buf, kPayload, len) != 0) {
    return kRecv;
  }

  /* the v1 prefix still registers services after the mailbox work. */
  const int64_t rid = v2->register_service(v2->user, kServiceId, t3_call);
  return rid < 0 ? -3 : 0;
}

extern "C" int32_t t3_stop(void) { return 0; }

const modukit_bundle_abi_v1 T3_DESCRIPTOR = {
    MODUKIT_ABI_VERSION_V2, /* the D96 v2 door - declared major 2 */
    sizeof(modukit_bundle_abi_v1),
    "t3-bus-bundle",
    kBundleId,
    "0.1.0",
    t3_start,
    t3_stop,
};

}  // namespace

extern "C" const modukit_bundle_abi_v1 *modukit_bundle_init(void) {
  return &T3_DESCRIPTOR;
}
