/* examples/c/host_demo.c - the C embed demo (onboarding-plan E2, ruling
 * A1 B-expanded; deepened 2026-11-24 to the FULL 17-verb walk, D1 of the
 * deepening batch). Plain C, no generated wrapper: the application calls
 * the modukit_embed.h verbs directly, exactly the identity story "the C
 * ABI is the only cross-language exit". The kernel behind the symbols is
 * the ambient Rust process-global in libmodukit_core.a
 * (crates/modukit-core/src/embed.rs). Fixture leg: the real built C bundle
 * .so (argv[1], baked by ctest via $<TARGET_FILE:c-example-bundle>).
 *
 * Walk (12 scenes, the whole rung-0 face the C band can reach):
 *   [1] embed_abi_version
 *   [2] install (manifest-v0 TOML built here; DEMO_C_SABOTAGE=1 corrupts
 *       it at this one point - the gate must RED)
 *   [3] resolve -> [4] start -> [5] state == ACTIVE
 *   [6] service_call ("hello" echoes the payload verbatim; unknown id
 *       answers negated NOT_FOUND)
 *   [7] find_services (S1 property-filter query: size probe, exact copy)
 *   [8] bus: latest-value publish -> late-join subscribe -> too-small
 *       recv reports QUOTA_EXCEEDED without consuming -> exact recv ->
 *       unsubscribe retires the queue
 *   [9] clock: now() >= 0; advance() on the Real domain answers the
 *       pinned CLOCK_DIAL_REFUSED (never a silent no-op)
 *   [10] introspect_snapshot: size probe, exact copy, documented line
 *        codec parsed (this bundle's ACTIVE row present)
 *   [11] introspect_events: size probe, LE records tile the buffer, this
 *        bundle's lifecycle rows found
 *   [12] stop -> uninstall -> resolve from UNINSTALLED answers the table
 *        guard (CONTRACT_VIOLATION)
 * Prints `C-DEMO PASS` only when every step asserted clean; exit 77 =
 * skip when the fixture is absent. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "modukit_embed.h"

#define BUNDLE_ID "com.example.c.demo"
#define SERVICE_ID "com.modukit.demo.hello"
#define PAYLOAD "c-demo-payload"
#define TOPIC "com.example.c.demo.status"
#define SIGNAL "c-demo-signal"

static int failures = 0;

static void check(int ok, const char *what) {
  if (!ok) {
    printf("FAIL: %s\n", what);
    ++failures;
  } else {
    printf("ok:   %s\n", what);
  }
}

/* Little-endian field readers for the documented event record codec
 * (embed_codec.rs: i32 kind | u64 seq | u64 ts_ns | u64 len | payload). */
static uint32_t rd32le(const uint8_t *p) {
  return (uint32_t)p[0] | ((uint32_t)p[1] << 8) | ((uint32_t)p[2] << 16) |
         ((uint32_t)p[3] << 24);
}
static uint64_t rd64le(const uint8_t *p) {
  uint64_t v = 0;
  for (int i = 7; i >= 0; --i) v = (v << 8) | p[i];
  return v;
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
           "id = \"" BUNDLE_ID "\"\nversion = \"0.1.0\"\n"
           "[entry]\nkind = \"%s\"\nartifact = \"%s\"\n",
           kind, so_path);
}

int main(int argc, char **argv) {
  if (argc < 2) {
    printf("SKIP: usage: host_demo <path-to-libc-example-bundle.so>\n");
    return 77;
  }
  const char *so_path = argv[1];
  FILE *probe = fopen(so_path, "rb");
  if (probe == NULL) {
    printf("SKIP: fixture .so missing at %s\n", so_path);
    return 77;
  }
  fclose(probe);

  printf("=== ModuKit E2 C embed demo (full rung-0 walk) ===\n");

  /* [1] embed_abi_version: the handshake probe. */
  check(modukit_embed_abi_version() == MODUKIT_EMBED_ABI_VERSION,
        "[1] embed_abi_version == 1");

  /* [2] install: manifest-v0 TOML in, kernel-owned canonical id out. */
  char toml[1024];
  manifest_for(so_path, toml, sizeof(toml));
  const char *bundle_id = NULL;
  const int32_t installed = modukit_install(toml, &bundle_id);
  check(installed == 0 && bundle_id != NULL &&
            strcmp(bundle_id, BUNDLE_ID) == 0,
        "[2] install -> canonical id (kernel-owned bytes)");
  if (failures != 0) {
    printf("C-DEMO FAIL (%d, install door)\n", failures);
    return 1; /* nothing downstream is reachable without a bundle */
  }

  /* [3] resolve + [4] start + [5] state. */
  check(modukit_resolve(bundle_id) == 0,
        "[3] resolve -> ABI v1 handshake green");
  check(modukit_start(bundle_id) == 0, "[4] start -> ACTIVE");
  int32_t state = -1;
  check(modukit_state(bundle_id, &state) == 0 && state == MODUKIT_STATE_ACTIVE,
        "[5] state == ACTIVE (4)");

  /* [6] service_call: the "hello" echo round-trips the payload; an
   * unknown service id answers the negated dictionary code, never a
   * partial answer. */
  const uint8_t req[] = PAYLOAD;
  uint8_t resp[512];
  const int32_t call = modukit_service_call(SERVICE_ID, "hello", req,
                                            sizeof(req) - 1, resp,
                                            sizeof(resp));
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
  const int32_t need = modukit_embed_find_services(filter, NULL, 0, &find_len);
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
       * bundle id (embed_codec.rs render_find). */
      find_ok = strstr((const char *)found, "service\t") != NULL &&
                strstr((const char *)found, SERVICE_ID) != NULL;
    }
    free(found);
  }
  check(find_ok, "[7] find_services: the C service's row rides the codec");

  /* [8] bus: publish, late-join subscribe, cap discipline, exact recv,
   * unsubscribe retires the queue id. */
  check(modukit_bus_publish(TOPIC, MODUKIT_QOS_LATEST_VALUE,
                            (const uint8_t *)SIGNAL,
                            sizeof(SIGNAL) - 1) == 0,
        "[8] bus_publish latest-value");
  int32_t queue_id = -1;
  /* the contract: bus_subscribe RETURNS the queue id (embed.h:186) - not
   * a zero status; asserting the id equality is the honest check. */
  check(modukit_bus_subscribe(TOPIC, &queue_id) == queue_id && queue_id >= 1,
        "[8] bus_subscribe -> kernel-owned queue id");
  size_t got_len = 0;
  uint8_t small[4];
  check(modukit_bus_recv(queue_id, small, sizeof(small), &got_len) ==
                -MODUKIT_QUOTA_EXCEEDED &&
            got_len == sizeof(SIGNAL) - 1,
        "[8] bus_recv too-small cap -> QUOTA_EXCEEDED, need reported");
  uint8_t out[128];
  /* success returns the byte count (embed.rs bus_recv `return n`), not a
   * zero status - the value itself is what the assertions compare. */
  check(modukit_bus_recv(queue_id, out, sizeof(out), &got_len) ==
                (int32_t)(sizeof(SIGNAL) - 1) &&
            got_len == sizeof(SIGNAL) - 1 &&
            memcmp(out, SIGNAL, got_len) == 0,
        "[8] bus_recv (late join) gets the exact value back");
  check(modukit_bus_unsubscribe(queue_id) == 0 &&
            modukit_bus_recv(queue_id, out, sizeof(out), &got_len) ==
                -MODUKIT_NOT_FOUND,
        "[8] bus_unsubscribe retires the queue id");

  /* [9] clock: the ambient kernel runs the Real domain, so now() answers
   * and advance() answers the pinned refusal (never a silent no-op). */
  int64_t now_ns = -1;
  check(modukit_clock_now(&now_ns) == 0 && now_ns >= 0,
        "[9] clock_now >= 0 ns");
  check(modukit_clock_advance(1000000) == -MODUKIT_CLOCK_DIAL_REFUSED,
        "[9] clock_advance on Real -> CLOCK_DIAL_REFUSED (never a no-op)");

  /* [10] introspect_snapshot: size query then exact copy; the documented
   * line codec parses and carries this bundle's ACTIVE row. */
  size_t snap_len = 0;
  check(modukit_introspect_snapshot(NULL, 0, &snap_len) > 0 && snap_len > 0,
        "[10] introspect_snapshot size query (NULL buffer)");
  uint8_t *snap = malloc(snap_len);
  check(snap != NULL, "[10] snapshot buffer alloc");
  int snap_ok = 0;
  if (snap != NULL) {
    size_t copied = 0;
    if (modukit_introspect_snapshot(snap, snap_len, &copied) > 0 &&
        copied == snap_len) {
      snap[snap_len] = 0;
      snap_ok = strstr((const char *)snap, "bundle\t" BUNDLE_ID "\tACTIVE") !=
                    NULL &&
                strstr((const char *)snap, "service\t") != NULL &&
                strstr((const char *)snap, "counters\t") != NULL;
    }
    free(snap);
  }
  check(snap_ok, "[10] introspect_snapshot: documented line codec parses");

  /* [11] introspect_events: size probe, LE records tile the buffer, this
   * bundle's lifecycle rows must be on the ring. */
  size_t ev_len = 0;
  uint64_t cursor = 0;
  check(modukit_introspect_events(0, NULL, 0, &ev_len, &cursor) >= 0 &&
            ev_len > 28,
        "[11] introspect_events size query");
  uint8_t *events = malloc(ev_len);
  check(events != NULL, "[11] events buffer alloc");
  int ev_ok = 0;
  if (events != NULL) {
    size_t ev_copied = 0;
    cursor = 0;
    if (modukit_introspect_events(0, events, ev_len, &ev_copied, &cursor) >=
        0) {
      size_t off = 0;
      uint64_t last_seq = 0;
      int saw_lifecycle = 0, tiled = 0;
      while (off + 28 <= ev_copied) {
        const uint32_t kind = rd32le(events + off);
        const uint64_t seq = rd64le(events + off + 4);
        const uint64_t plen = rd64le(events + off + 20);
        if (kind == MODUKIT_EVENT_LIFECYCLE &&
            ev_copied >= off + 28 + strlen(BUNDLE_ID) &&
            memcmp(events + off + 28, BUNDLE_ID, strlen(BUNDLE_ID)) == 0) {
          saw_lifecycle = 1;
        }
        last_seq = seq;
        off += 28 + (size_t)plen;
      }
      tiled = off == ev_copied && cursor == last_seq;
      ev_ok = saw_lifecycle && tiled;
    }
    free(events);
  }
  check(ev_ok, "[11] introspect_events: LE records tile; lifecycle rows found");

  /* [12] stop + uninstall close the walk (and the bundle's services);
   * the table guard answers resolve-from-UNINSTALLED. */
  check(modukit_stop(bundle_id) == 0, "[12] stop -> RESOLVED");
  check(modukit_state(bundle_id, &state) == 0 &&
            state == MODUKIT_STATE_RESOLVED,
        "[12] state == RESOLVED (6)");
  check(modukit_uninstall(bundle_id) == 0, "[12] uninstall -> UNINSTALLED");
  check(modukit_state(bundle_id, &state) == 0 &&
            state == MODUKIT_STATE_UNINSTALLED,
        "[12] state == UNINSTALLED (0)");
  check(modukit_resolve(bundle_id) == -MODUKIT_CONTRACT_VIOLATION,
        "[12] resolve from UNINSTALLED -> CONTRACT_VIOLATION(-10)");

  if (failures != 0) {
    printf("C-DEMO FAIL (%d)\n", failures);
    return 1;
  }
  printf("C-DEMO PASS\n");
  return 0;
}
