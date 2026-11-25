/* tutorial t4 - the host side: drive a bundle through the full lifecycle
 * over the GENERATED wrapper (the CppMS "run the framework" step). This
 * is the tutorial's driver: it installs one of the t1/t2 bundles passed
 * as argv[1], walks install -> resolve -> start -> state -> service_call
 * (t2) -> stop -> uninstall, and prints T4-DRIVER PASS. The reduced walk
 * is deliberate: chapters teach the verbs one at a time; this driver
 * proves the tutorial bundles the same way the gate legs prove the
 * fixtures. */
#include <cstdio>
#include <cstring>
#include <filesystem>
#include <string>

#include "modukit_embed.hpp"

namespace {

const char *const kEchoService = "com.example.t2.echo";
const char *const kPayload = "t4-driver-payload";
int failures = 0;

void check(bool ok, const char *what) {
  std::printf("%s: %s\n", ok ? "ok  " : "FAIL", what);
  if (!ok) {
    ++failures;
  }
}

std::string manifest_for(const std::string &so_path) {
  return "id = \"com.example.t4.walk\"\nversion = \"0.1.0\"\n"
         "[entry]\nkind = \"dl\"\nartifact = \"" + so_path + "\"\n";
}

}  // namespace

int main(int argc, char **argv) {
  if (argc < 2) {
    std::printf("SKIP: usage: t4-driver <path-to-a-tutorial-bundle.so>\n");
    return 77;
  }
  const std::string so_path = argv[1];
  if (!std::filesystem::exists(so_path)) {
    std::printf("SKIP: bundle .so missing at %s\n", so_path.c_str());
    return 77;
  }

  const modukit::Kernel kernel;

  const std::string toml = manifest_for(so_path);
  const char *bundle_id = nullptr;
  check(kernel.install(toml.c_str(), &bundle_id).ok() &&
            bundle_id != nullptr,
        "install");
  if (failures != 0) {
    std::printf("T4-DRIVER FAIL (install door)\n");
    return 1;
  }
  check(kernel.resolve(bundle_id).ok(), "resolve");
  check(kernel.start(bundle_id).ok(), "start");
  int32_t state = -1;
  check(kernel.state(bundle_id, &state).ok() &&
            state == static_cast<int32_t>(modukit::BundleState::STATE_ACTIVE),
        "state == ACTIVE");

  /* the echo scene only makes sense against the t2 bundle (it registers
   * the echo service); the t1 walk ends here by design. */
  if (std::strcmp(bundle_id, "com.example.t2.echo") == 0) {
    const uint8_t req[] = {'t', '4', '-', 'd', 'r', 'i', 'v', 'e', 'r',
                           '-', 'p', 'a', 'y', 'l', 'o', 'a', 'd', 0};
    const size_t req_len = sizeof(req) - 1;
    uint8_t resp[256];
    const modukit::Status call = kernel.service_call(
        kEchoService, "echo", req, req_len, resp, sizeof(resp));
    check(call.ok() && call.raw() == static_cast<int32_t>(req_len) &&
              std::memcmp(resp, req, req_len) == 0,
          "service_call echoes the payload");
  }

  check(kernel.stop(bundle_id).ok(), "stop");
  check(kernel.uninstall(bundle_id).ok(), "uninstall");

  if (failures != 0) {
    std::printf("T4-DRIVER FAIL (%d)\n", failures);
    return 1;
  }
  std::printf("T4-DRIVER PASS\n");
  return 0;
}
