/* cpp-example-plugin - positive dlopen fixture, C++ twin of the cargo
   example-plugin (D23 template standing). The kernel-facing surface is
   exactly the v1 handshake (contracts/modukit_plugin.h) plus ONE service
   registration; message types ride stock protoc C++ gencode and the
   emitter's plugin-face skeleton (ruling C++-3 face A) - nothing here
   re-implements the dispatch or the messages. */
#include <cstring>

#include "modukit_plugin.h"
#include "com_modukit_demo.hpp"

namespace {

/* The author's implementation of the generated skeleton: Echo answers with
   the request payload verbatim, Describe reports the registration identity. */
class DemoServiceImpl final : public com::modukit::demo::DemoServiceSkeleton {
 public:
  ::com::modukit::demo::EchoReply Echo(
      const ::com::modukit::demo::EchoRequest &request) override {
    ::com::modukit::demo::EchoReply reply;
    reply.set_payload(request.payload());
    reply.set_status(::com::modukit::demo::DEMO_STATUS_OK);
    return reply;
  }

  ::com::modukit::demo::MetaReply Describe(
      const ::com::modukit::demo::MetaRequest &request) override {
    (void)request;
    ::com::modukit::demo::MetaReply reply;
    reply.set_service_id(kServiceId);
    reply.set_method_count(static_cast<uint32_t>(kMethodCount));
    reply.set_state(::com::modukit::demo::DEMO_STATUS_OK);
    return reply;
  }
};

DemoServiceImpl g_service;

/* The one service's call slot (mirrors abi::ServiceCallFn): the byte door
   of the generated skeleton, typed in/out through protoc messages. */
extern "C" int32_t demo_call(const char *method, const uint8_t *input,
                             size_t input_len, uint8_t *output,
                             size_t output_cap) {
  if (method == nullptr || output == nullptr || (input_len > 0 && input == nullptr)) {
    return DemoServiceImpl::kBadRequest;
  }
  return g_service.Dispatch(method, input, input_len, output, output_cap);
}

/* Activation: register exactly one service through the borrowed host table
   and keep nothing (01 §3 rule 2: no ambient authority past this call). */
extern "C" int32_t demo_start(const modukit_host_abi_v1 *host) {
  if (host == nullptr) {
    return -1;
  }
  if (host->size < sizeof(modukit_host_abi_v1)) {
    return -2;
  }
  const int64_t rid =
      host->register_service(host->user, DemoServiceImpl::kServiceId, demo_call);
  return rid < 0 ? -3 : 0;
}

extern "C" int32_t demo_stop(void) {
  return 0;  /* cooperative no-op; idempotent by construction (01 §3 rule 6) */
}

const modukit_plugin_abi_v1 DESCRIPTOR = {
    MODUKIT_ABI_VERSION,   /* abi_version - guard 2 */
    sizeof(modukit_plugin_abi_v1), /* size - guard 3 */
    "cpp-example-plugin",  /* name */
    DemoServiceImpl::kServiceId, /* id == registered service id, mirroring the rs twin (D9) */
    "0.1.0",               /* version - plugin's own semver */
    demo_start,            /* start - guard 4 */
    demo_stop,             /* stop - guard 4 */
};

}  // namespace

/* The single exported entry symbol (04 §3 symbol discipline). */
extern "C" const modukit_plugin_abi_v1 *modukit_plugin_init(void) {
  return &DESCRIPTOR;
}
