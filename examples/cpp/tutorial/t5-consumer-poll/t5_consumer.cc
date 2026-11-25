/* tutorial t5 - the naive consumer (the CppMS dictionaryclient step):
 * a HOST-side consumer that wants a service and does not care who
 * provides it. Step one of the consumer evolution (t5 poll -> t6
 * filter): call by well-known id, handle the typed NOT_FOUND while the
 * provider is absent, retry after it appears. The typed refusal is the
 * lesson - a missing service is an ANSWER, never a crash. */
#include <cstdio>
#include <cstring>
#include <filesystem>
#include <string>

#include "modukit_embed.hpp"

namespace {

const char *const kConsumerId = "com.example.t5.consumer";
const char *const kEchoService = "com.example.t2.echo";
const char *const kPayload = "t5-consumer-call";
int failures = 0;

void check(bool ok, const char *what) {
  std::printf("%s: %s\n", ok ? "ok  " : "FAIL", what);
  if (!ok) {
    ++failures;
  }
}

std::string manifest_for(const std::string &so_path) {
  return "id = \"" + std::string(kConsumerId) + "\"\nversion = \"0.1.0\"\n"
         "[entry]\nkind = \"dl\"\nartifact = \"" + so_path + "\"\n";
}

}  // namespace

int main(int argc, char **argv) {
  if (argc < 2) {
    std::printf("SKIP: usage: t5-consumer <path-to-t2-echo-bundle.so>\n");
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
  check(kernel.install(toml.c_str(), &bundle_id).ok(), "install the provider");
  check(kernel.resolve(bundle_id).ok(), "resolve (handshake, NOT started)");

  /* the naive consumer's first lesson: the typed absence. */
  const uint8_t req[] = {'t', '5', '-', 'c', 'a', 'l', 'l', 0};
  const size_t req_len = sizeof(req) - 1;
  uint8_t resp[256];
  check(kernel.service_call(kEchoService, "echo", req, req_len, resp,
                            sizeof(resp))
                .code() == modukit::KernelCode::NOT_FOUND,
        "service_call while the provider is stopped -> NOT_FOUND(-14)");

  /* the provider appears; the SAME call now round-trips. */
  check(kernel.start(bundle_id).ok(), "start the provider");
  const modukit::Status call =
      kernel.service_call(kEchoService, "echo", req, req_len, resp, sizeof(resp));
  check(call.ok() && call.raw() == static_cast<int32_t>(req_len) &&
            std::memcmp(resp, req, req_len) == 0,
        "the SAME service_call now echoes (the consumer never changed)");

  check(kernel.stop(bundle_id).ok() && kernel.uninstall(bundle_id).ok(),
        "the provider leaves; the consumer's lesson is complete");

  if (failures != 0) {
    std::printf("T5-CONSUMER FAIL (%d)\n", failures);
    return 1;
  }
  std::printf("T5-CONSUMER PASS\n");
  return 0;
}
