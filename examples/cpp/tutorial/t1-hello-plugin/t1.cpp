/* tutorial t1 - the minimal plugin: a descriptor and nothing else
 * (the CppMS "install and start a bundle" step, ModuKit shape). The
 * handshake guards (04 §3) need exactly: the entry symbol, abi_version,
 * size, non-NULL start/stop. No service, no bus - just prove the plugin
 * installs, resolves, starts and stops. */
#include "modukit_bundle.h"

namespace {

extern "C" int32_t t1_start(const modukit_host_abi_v1 *host) {
  (void)host; /* nothing to register yet - that is step t2 */
  return 0;
}

extern "C" int32_t t1_stop(void) { return 0; }

const modukit_bundle_abi_v1 T1_DESCRIPTOR = {
    MODUKIT_ABI_VERSION,           /* abi_version - handshake guard 2 */
    sizeof(modukit_bundle_abi_v1), /* size - handshake guard 3 */
    "t1-hello-plugin",             /* name */
    "com.example.t1.hello",        /* id (D9 canonical reverse-DNS) */
    "0.1.0",                       /* version */
    t1_start,                      /* start - guard 4 */
    t1_stop,                       /* stop - guard 4 */
};

}  // namespace

extern "C" const modukit_bundle_abi_v1 *modukit_bundle_init(void) {
  return &T1_DESCRIPTOR;
}
