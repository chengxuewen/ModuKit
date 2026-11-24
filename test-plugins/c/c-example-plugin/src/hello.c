/* c-example-plugin - the positive dlopen fixture of the C band
 * (onboarding-plan E2, ruling A1 B-expanded; the C twin of the
 * cpp-example-plugin D23 template). The kernel-facing surface is exactly
 * the v1 handshake (contracts/modukit_plugin.h) plus ONE service
 * registration - hand-written, no generated shelf: the whole point of the
 * C twin is that a plugin author needs nothing but this header and one
 * source file.
 *
 * Service: "com.modukit.demo.hello", one method "hello" that answers with
 * the request payload verbatim (the echo shape the cpp twin teaches,
 * minus the codegen: the byte door IS the whole protocol here). */
#include <string.h>

#include "modukit_plugin.h"

#define HELLO_SERVICE_ID "com.modukit.demo.hello"
/* Plugin-author negative code space (D27): below -100 stays out of the
 * kernel's own dictionary; the cpp twin uses -101/-102/-103 for the same
 * three refusals. */
#define HELLO_BAD_REQUEST (-101)

static int32_t hello_call(const char *method, const uint8_t *input,
                          size_t input_len, uint8_t *output,
                          size_t output_cap) {
  if (method == NULL || output == NULL || (input_len > 0 && input == NULL)) {
    return HELLO_BAD_REQUEST;
  }
  if (strcmp(method, "hello") != 0) {
    return HELLO_BAD_REQUEST; /* unknown method - author code space */
  }
  if (input_len > output_cap) {
    return HELLO_BAD_REQUEST; /* needing more is a negative, never a
                                 truncation (modukit_plugin.h call shape) */
  }
  memcpy(output, input, input_len);
  return (int32_t)input_len;
}

/* Activation: register exactly one service through the borrowed host table
 * and keep nothing (01 §3 rule 2: no ambient authority past this call). */
static int32_t hello_start(const struct modukit_host_abi_v1 *host) {
  if (host == NULL) {
    return -1;
  }
  if (host->size < sizeof(modukit_host_abi_v1)) {
    return -2;
  }
  const int64_t rid =
      host->register_service(host->user, HELLO_SERVICE_ID, hello_call);
  return rid < 0 ? -3 : 0;
}

static int32_t hello_stop(void) {
  return 0; /* cooperative no-op; idempotent by construction (01 §3 rule 6) */
}

static const modukit_plugin_abi_v1 HELLO_DESCRIPTOR = {
    MODUKIT_ABI_VERSION,            /* abi_version - handshake guard 2 */
    sizeof(modukit_plugin_abi_v1),  /* size - handshake guard 3 */
    "c-example-plugin",             /* name */
    HELLO_SERVICE_ID,               /* id == registered service id (D9),
                                       mirroring the cpp/rs twins */
    "0.1.0",                        /* version - the plugin's own semver */
    hello_start,                    /* start - handshake guard 4 */
    hello_stop,                     /* stop - handshake guard 4 */
};

/* The single exported entry symbol (04 §3 symbol discipline). */
const modukit_plugin_abi_v1 *modukit_plugin_init(void) {
  return &HELLO_DESCRIPTOR;
}
