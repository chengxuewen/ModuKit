/* tutorial t6 - the filter consumer (the CppMS dictionaryclient3 step):
 * DISCOVER before you call. The naive consumer (t5) hardcodes the
 * service id; this consumer queries the registry with the S1
 * property-filter face (modukit_embed_find_services), parses the
 * documented line codec, and only then calls. The empty result is the
 * second lesson: an unmatched filter is an EMPTY SET, never an error. */
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <string>

#include "modukit_embed.hpp"

namespace {

const char *const kConsumerId = "com.example.t6.consumer";
const char *const kEchoService = "com.example.t2.echo";
const char *const kPayload = "t6-consumer-call";
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
    std::printf("SKIP: usage: t6-consumer <path-to-t2-echo-plugin.so>\n");
    return 77;
  }
  const std::string so_path = argv[1];
  if (!std::filesystem::exists(so_path)) {
    std::printf("SKIP: plugin .so missing at %s\n", so_path.c_str());
    return 77;
  }

  const modukit::Kernel kernel;
  const std::string toml = manifest_for(so_path);
  const char *plugin_id = nullptr;
  check(kernel.install(toml.c_str(), &plugin_id).ok(), "install the provider");
  check(kernel.resolve(plugin_id).ok() && kernel.start(plugin_id).ok(),
        "provider ACTIVE");

  /* discovery: the S1 filter query, size probe then exact copy. The
   * wrapper method carries the embed_ prefix (the verb's own name). */
  const std::string filter = "(service.id=" + std::string(kEchoService) + ")";
  size_t need = 0;
  check(kernel.embed_find_services(filter.c_str(), nullptr, 0, &need).raw() >
                    0 &&
                need > 0,
        "find_services size probe (NULL buffer)");
  std::string rows(static_cast<size_t>(need), 0);
  check(kernel.embed_find_services(
                filter.c_str(), reinterpret_cast<uint8_t *>(rows.data()), need,
                &need)
                    .raw() > 0 &&
            rows.find(kEchoService) != std::string::npos,
        "the filter query names the echo service");

  /* the empty-set lesson: an unmatched filter answers EMPTY, not an error
   * (the liar-family shape - absence is data). */
  size_t ghost = 123;
  check(kernel.embed_find_services("(service.id=com.example.ghost)", nullptr,
                                   0, &ghost)
                    .raw() == 0 &&
            ghost == 0,
        "an unmatched filter is an empty set (0 bytes), never an error");

  /* now - and only now - the call. */
  const uint8_t req[] = {'t', '6', '-', 'c', 'a', 'l', 'l', 0};
  const size_t req_len = sizeof(req) - 1;
  uint8_t resp[256];
  const modukit::Status call = kernel.service_call(
      kEchoService, "echo", req, req_len, resp, sizeof(resp));
  check(call.ok() && call.raw() == static_cast<int32_t>(req_len) &&
            std::memcmp(resp, req, req_len) == 0,
        "discovered, then called");

  check(kernel.stop(plugin_id).ok() && kernel.uninstall(plugin_id).ok(),
        "closed");

  if (failures != 0) {
    std::printf("T6-CONSUMER FAIL (%d)\n", failures);
    return 1;
  }
  std::printf("T6-CONSUMER PASS\n");
  return 0;
}
