/* tutorial t2 - the first service: register ONE echo service through the
 * borrowed host table (the CppMS "register a service" step, raw-ABI shape
 * - no generated shelf, so every line of the handshake is visible). The
 * echo answers the request payload verbatim; the unknown-method refusal
 * is the bundle's own authority (D27 author code space). */
#include <cstring>

#include "modukit_bundle.h"

namespace {

const char *const kServiceId = "com.example.t2.echo";
const int32_t kBadRequest = -101;

extern "C" int32_t t2_call(const char *method, const uint8_t *input,
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

extern "C" int32_t t2_stop(void) { return 0; }

const modukit_bundle_abi_v1 T2_DESCRIPTOR = {
    MODUKIT_ABI_VERSION,
    sizeof(modukit_bundle_abi_v1),
    "t2-echo-bundle",
    kServiceId, /* id == registered service id (D9), mirroring the twins */
    "0.1.0",
    t2_start,
    t2_stop,
};

}  // namespace

extern "C" const modukit_bundle_abi_v1 *modukit_bundle_init(void) {
  return &T2_DESCRIPTOR;
}
