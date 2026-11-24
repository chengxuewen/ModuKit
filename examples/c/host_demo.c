/* examples/c/host_demo.c - the C embed demo (onboarding-plan E2, ruling
 * A1 B-expanded: "embed from your language" has a C cell). Plain C, no
 * generated wrapper: the application calls the modukit_embed.h verbs
 * directly, exactly the identity story "the C ABI is the only
 * cross-language exit". The kernel behind the symbols is the ambient Rust
 * process-global in libmodukit_core.a (crates/modukit-core/src/embed.rs).
 * Fixture leg: the real built C plugin .so (argv[1], baked by ctest via
 * $<TARGET_FILE:c-example-plugin>).
 *
 * Walk (the reduced E2 chain the plan names: install -> resolve -> start
 * -> state -> service_call -> find_services -> stop):
 *   install (manifest-v0 TOML built here) -> resolve -> start -> state
 *   -> service_call ("hello" echoes the payload verbatim; unknown id
 *      answers negated NOT_FOUND)
 *   -> find_services (property-filter query, S1: size query then exact
 *      copy; the documented line codec carries this service's row)
 *   -> stop -> uninstall.
 * Prints `C-DEMO PASS` only when every step asserted clean; exit 77 =
 * skip when the fixture is absent; DEMO_C_SABOTAGE=1 corrupts the
 * manifest at exactly ONE named point so the gate must RED. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "modukit_embed.h"

#define PLUGIN_ID "com.example.c.demo"
#define SERVICE_ID "com.modukit.demo.hello"
#define PAYLOAD "c-demo-payload"

static int failures = 0;

static void check(int ok, const char *what) {
  if (!ok) {
    printf("FAIL: %s\n", what);
    ++failures;
  } else {
    printf("ok:   %s\n", what);
  }
}

/* The deploy-tier manifest TOML for the fixture. DEMO_C_SABOTAGE=1 is THE
 * single sabotage point (mirrors DEMO_STAGE4_SABOTAGE in the cpp band): an
 * unknown placement must be refused at the install door. */
static void manifest_for(const char *so_path, char *out, size_t cap) {
  const char *kind = "dl";
  if (getenv("DEMO_C_SABOTAGE") != NULL) {
    printf("[sabotage] DEMO_C_SABOTAGE=1 -> corrupting manifest\n");
    kind = "dlx";
  }
  snprintf(out, cap,
           "id = \"" PLUGIN_ID "\"\nversion = \"0.1.0\"\n"
           "[entry]\nkind = \"%s\"\nartifact = \"%s\"\n",
           kind, so_path);
}

int main(int argc, char **argv) {
  if (argc < 2) {
    printf("SKIP: usage: host_demo <path-to-libc-example-plugin.so>\n");
    return 77;
  }
  const char *so_path = argv[1];
  FILE *probe = fopen(so_path, "rb");
  if (probe == NULL) {
    printf("SKIP: fixture .so missing at %s\n", so_path);
    return 77;
  }
  fclose(probe);

  printf("=== ModuKit E2 C embed demo ===\n");

  /* [1] embed_abi_version: the handshake probe. */
  check(modukit_embed_abi_version() == MODUKIT_EMBED_ABI_VERSION,
        "[1] embed_abi_version == 1");

  /* [2] install: manifest-v0 TOML in, kernel-owned canonical id out. */
  char toml[1024];
  manifest_for(so_path, toml, sizeof(toml));
  const char *plugin_id = NULL;
  const int32_t installed =
      modukit_install(toml, &plugin_id);
  check(installed == 0 && plugin_id != NULL &&
            strcmp(plugin_id, PLUGIN_ID) == 0,
        "[2] install -> canonical id (kernel-owned bytes)");
  if (failures != 0) {
    printf("C-DEMO FAIL (%d, install door)\n", failures);
    return 1; /* nothing downstream is reachable without a plugin */
  }

  /* [3] resolve + [4] start + [5] state. */
  check(modukit_resolve(plugin_id) == 0, "[3] resolve -> ABI v1 handshake green");
  check(modukit_start(plugin_id) == 0, "[4] start -> ACTIVE");
  int32_t state = -1;
  check(modukit_state(plugin_id, &state) == 0 && state == MODUKIT_STATE_ACTIVE,
        "[5] state == ACTIVE (4)");

  /* [6] service_call: the "hello" echo round-trips the payload; an
   * unknown service id answers the negated dictionary code, never a
   * partial answer. */
  const uint8_t req[] = PAYLOAD;
  uint8_t resp[512];
  const int32_t call = modukit_service_call(
      SERVICE_ID, "hello", req, sizeof(req) - 1, resp, sizeof(resp));
  check(call == (int32_t)(sizeof(req) - 1) &&
            memcmp(resp, req, sizeof(req) - 1) == 0,
        "[6] service_call hello echoes the payload verbatim");
  check(modukit_service_call("com.example.nope", "hello", req,
                             sizeof(req) - 1, resp, sizeof(resp)) ==
            -MODUKIT_NOT_FOUND,
        "[6] service_call unknown id -> NOT_FOUND(-14)");

  /* [7] find_services (S1): the property-filter query - size query then
   * exact copy; the documented line codec carries this service's row. */
  size_t find_len = 0;
  const char *filter = "(service.id=" SERVICE_ID ")";
  const int32_t need = modukit_embed_find_services(
      filter, NULL, 0, &find_len);
  check(need > 0 && (size_t)need == find_len,
        "[7] find_services size query (NULL buffer)");
  uint8_t *found = malloc(find_len + 1); /* +1 for the NUL we append */
  check(found != NULL, "[7] find_services buffer alloc");
  int find_ok = 0;
  if (found != NULL) {
    const int32_t taken =
        modukit_embed_find_services(filter, found, find_len, &find_len);
    find_ok = taken == need;
    if (find_ok) {
      found[find_len] = 0;
      /* the row is `service\t<numeric rid>\t<service_id>\t<ranking>` -
       * the second field is the kernel-owned registration number, not a
       * plugin id (embed_codec.rs render_find). */
      find_ok = strstr((const char *)found, "service\t") != NULL &&
                strstr((const char *)found, SERVICE_ID) != NULL;
    }
    free(found);
  }
  check(find_ok, "[7] find_services: the C service's row rides the codec");

  /* [8] stop + uninstall close the walk (and the plugin's services). */
  check(modukit_stop(plugin_id) == 0, "[8] stop -> RESOLVED");
  check(modukit_state(plugin_id, &state) == 0 &&
            state == MODUKIT_STATE_RESOLVED,
        "[8] state == RESOLVED (6)");
  check(modukit_uninstall(plugin_id) == 0, "[8] uninstall -> UNINSTALLED");
  check(modukit_state(plugin_id, &state) == 0 &&
            state == MODUKIT_STATE_UNINSTALLED,
        "[8] state == UNINSTALLED (0)");

  if (failures != 0) {
    printf("C-DEMO FAIL (%d)\n", failures);
    return 1;
  }
  printf("C-DEMO PASS\n");
  return 0;
}
