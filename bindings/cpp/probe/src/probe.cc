/* modukit-cpp-probe - the bindings/cpp ctest probe (Stage-4 M1): loads the
   REAL built C++ fixtures through dlopen and walks the frozen v1 handshake
   from the host side (executor.rs `open_validated`/`validate` guard order,
   04 §3):
     good arm  - entry symbol + descriptor guards, start() over a minimal
                 filled host table, one typed protobuf round-trip through
                 the register-captured ServiceCallFn, stop() twice;
     bad arm   - the abi_version guard must REFUSE the 999 descriptor before
                 start is ever called (mirrors the rs bad-plugin intent).
   Also includes both generated faces (plugin skeleton + embed wrapper) in
   one TU - the header coexistence the contract header promises.
   Prints `CPP-PROBE PASS` on success; 77 = skip if a fixture is absent. */
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>

#include <dlfcn.h>

#include "modukit_plugin.h"
#include "modukit_embed.hpp"        /* face B: compiled, never called here */
#include "com_modukit_demo.hpp"     /* face A: the id + dispatch codes */
#include "com.modukit.demo.pb.h"    /* stock protoc messages (C++-1) */

namespace {

int failures = 0;

void check(bool ok, const char *what) {
  if (!ok) {
    std::printf("FAIL: %s\n", what);
    ++failures;
  }
}

/* Host-side registration capture: the probe plays the kernel for one call */
struct Captured {
  const char *service_id = nullptr;
  modukit_service_call_fn_t call = nullptr;
  int64_t next_id = 0;
};

int64_t capture_register(void *user, const char *service_id,
                         modukit_service_call_fn_t call) {
  auto *cap = static_cast<Captured *>(user);
  cap->service_id = service_id;
  cap->call = call;
  return cap->next_id++;
}

using init_fn = const modukit_plugin_abi_v1 *(*)();

/* The v1 guard sequence, verbatim order (04 §3): returns nullptr on
   refusal; *refusal names the guard that fired when non-null is returned. */
const modukit_plugin_abi_v1 *open_and_guard(const char *path,
                                            const char **refusal) {
  *refusal = nullptr;
  void *handle = dlopen(path, RTLD_NOW | RTLD_LOCAL);
  if (handle == nullptr) {
    std::printf("SKIP: dlopen(%s) failed: %s\n", path, dlerror());
    std::exit(77);
  }
  auto init = reinterpret_cast<init_fn>(dlsym(handle, MODUKIT_ENTRY_SYMBOL));
  if (init == nullptr) {
    *refusal = "entry symbol missing";
    return nullptr;
  }
  const modukit_plugin_abi_v1 *d = init();
  if (d == nullptr) {
    *refusal = "init returned NULL";
    return nullptr;
  }
  if (d->abi_version != MODUKIT_ABI_VERSION) {
    *refusal = "ABI_MISMATCH";
    return d;  /* caller inspects the lie */
  }
  if (d->size < sizeof(modukit_plugin_abi_v1)) {
    *refusal = "DESCRIPTOR_TOO_SMALL";
    return d;
  }
  if (d->start == nullptr || d->stop == nullptr) {
    *refusal = "HANDSHAKE_INCOMPLETE";
    return d;
  }
  return d;
}

void good_arm(const char *path) {
  const char *refusal = nullptr;
  const modukit_plugin_abi_v1 *d = open_and_guard(path, &refusal);
  check(refusal == nullptr && d != nullptr, "good fixture passes the v1 guards");
  if (refusal != nullptr) {
    std::printf("[good] refused at: %s\n", refusal);
    return;
  }
  std::printf("[good] descriptor: abi_version=%u size=%u name=%s id=%s version=%s\n",
              d->abi_version, d->size, d->name, d->id, d->version);

  Captured cap;
  modukit_host_abi_v1 host = {
      sizeof(modukit_host_abi_v1), &cap, capture_register,
  };
  check(d->start(&host) == 0, "start() returns 0");
  check(cap.call != nullptr, "register_service captured the call slot");
  check(cap.service_id != nullptr &&
                std::strcmp(cap.service_id,
                            com::modukit::demo::DemoServiceSkeleton::kServiceId) == 0,
        "registered id == skeleton kServiceId");

  /* Typed round-trip through the captured ServiceCallFn (D27 int channel). */
  com::modukit::demo::EchoRequest req;
  req.set_payload("cpp-probe-payload");
  const std::string bytes = req.SerializeAsString();
  std::vector<uint8_t> out(bytes.size() + 128);
  const int32_t n = cap.call("Echo",
                             reinterpret_cast<const uint8_t *>(bytes.data()),
                             bytes.size(), out.data(), out.size());
  check(n >= 0, "Echo call returns byte count >= 0");
  com::modukit::demo::EchoReply reply;
  check(n >= 0 && reply.ParseFromArray(out.data(), n), "reply parses");
  check(reply.payload() == req.payload(), "payload round-trips");
  check(reply.status() == com::modukit::demo::DEMO_STATUS_OK, "status OK");
  std::printf("[good] echo round-trip: %d bytes, payload verified, status=OK\n", n);

  const int32_t bad = cap.call("Nope", nullptr, 0, out.data(), out.size());
  check(bad == com::modukit::demo::DemoServiceSkeleton::kUnknownMethod,
        "unknown method -> kUnknownMethod");

  check(d->stop() == 0 && d->stop() == 0, "stop() idempotent (0,0)");
  std::printf("[good] stop x2: 0 0 (idempotent)\n");
}

void bad_arm(const char *path) {
  const char *refusal = nullptr;
  const modukit_plugin_abi_v1 *d = open_and_guard(path, &refusal);
  /* The guard sequence must fire the VERSION refusal (start/stop present,
     size right) - anything else hides what this fixture proves. */
  check(refusal != nullptr && std::strcmp(refusal, "ABI_MISMATCH") == 0,
        "bad fixture refused at the version guard");
  check(d != nullptr && d->abi_version != MODUKIT_ABI_VERSION,
        "abi_version != MODUKIT_ABI_VERSION detected");
  std::printf("[bad] abi_version=%u != %u -> REFUSED (%s), start never called\n",
              d ? d->abi_version : 0u, MODUKIT_ABI_VERSION,
              refusal ? refusal : "ACCEPTED (BUG)");
}

}  // namespace

int main() {
#ifndef MODUKIT_CPP_PROBE_GOOD
#error "MODUKIT_CPP_PROBE_GOOD/BAD paths must be defined by the build"
#endif
  good_arm(MODUKIT_CPP_PROBE_GOOD);
  bad_arm(MODUKIT_CPP_PROBE_BAD);
  if (failures != 0) {
    std::printf("CPP-PROBE FAIL (%d)\n", failures);
    return 1;
  }
  std::printf("CPP-PROBE PASS\n");
  return 0;
}
