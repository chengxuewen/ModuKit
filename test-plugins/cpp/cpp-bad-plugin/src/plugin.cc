/* cpp-bad-plugin - negative fixture, C++ twin of the cargo bad-plugin: a
   deliberately mismatched handshake. The descriptor reports ABI v999 with
   an otherwise well-formed struct, so the version guard must reject it
   (04 §3 "unknown major version -> refused") - never a segfault, never a
   silent accept. start/stop are present (a NULL would trip the incomplete
   guard instead of the version guard) and carry poison-returning statuses
   so any partial-start attempt is visible. No protobuf here on purpose:
   the ABI-lie story needs no message layer. */
#include "modukit_plugin.h"

namespace {

/* Unknown major version (mirrors the rs bad-plugin const). */
constexpr uint32_t BAD_ABI_VERSION = 999;

extern "C" int32_t lie_start(const modukit_host_abi_v1 *) {
  return -113;  /* running at all is the bug the version guard prevents */
}

extern "C" int32_t lie_stop(void) { return -113; }

const modukit_plugin_abi_v1 DESCRIPTOR = {
    BAD_ABI_VERSION,
    sizeof(modukit_plugin_abi_v1),
    "cpp-bad-plugin",
    "com.example.modukit.cppbad",
    "0.1.0",
    lie_start,
    lie_stop,
};

}  // namespace

extern "C" const modukit_plugin_abi_v1 *modukit_plugin_init(void) {
  return &DESCRIPTOR;
}
