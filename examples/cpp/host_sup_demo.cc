/* examples/cpp/host_sup_demo.cc - the Stage-4 M2 exe-rung demo with the
   ORPHAN-PROOF from a C++ host (plan §5: "ORPHAN-PROOF-style demo gate
   from the C++ host"), plus the M3 fidelity stanza (D94 W3). Two modes
   of one binary:

     (default) WALK : the supervised host. Drives the generated wrapper's
       sup_* arm (bindings/cpp face B): sup_install (real runner spawn +
       HELLO + start), sup_state(ACTIVE), sup_step(idle), sup_dial(step-
       domain -> typed turn word), then the M3 FIDELITY stanza:
       sup_service_call carries ONE non-empty typed protobuf payload
       through ABI -> TL wire -> child service -> back and asserts the
       response's payload bytes equal the request's EXACTLY before
       printing the FIDELITY PASS marker; then sleeps until killed.
     --proof   PARENT: fork+exec self in WALK mode, wait for the READY
       handshake line and the FIDELITY PASS marker (a walk child that
       dies before the marker failed the door - no PASS is printed),
       SIGKILL the child host, then scan /proc: the
       runner must have self-reaped on the control-pipe EOF (S2-M5a
       machinery, now proven under a C++-authored host). Zero survivors
       prints `SUP-DEMO PASS`; any survivor prints the pids and FAILs.

   Sabotage hooks, each ONE named point (C0):
     DEMO_STAGE4_M2_SABOTAGE=1 - the parent stops (SIGSTOP) instead of
       killing the child, so the runner legitimately survives - the
       census MUST go red on it.
     DEMO_STAGE4_M3_SABOTAGE=1 - corrupts the fidelity stanza's expected-
       bytes comparison, so the gate MUST go red on a byte-exact child.
   Exit 77 = runner .so/binary absent (ctest SKIP_RETURN_CODE contract). */
#include <algorithm>
#include <csignal>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <filesystem>
#include <string>
#include <vector>

#include <fcntl.h>
#include <sys/wait.h>
#include <unistd.h>

#include "com.modukit.demo.pb.h"   /* stock protoc messages (C++-1, as on the dl face) */
#include "modukit_embed.hpp"       /* face B: the generated embed wrapper */

namespace {

struct Paths {
  std::string so;     // plugin artifact (exe manifest `artifact`)
  std::string run;    // shadow-runner binary (manifest `runner`)
};

// The fixture's canonical service id (D9 mirror inside the cpp plugin;
// the same id the dl face of host_demo.cc calls).
const char *const kServiceId = "com.modukit.demo.DemoService";

int census(const std::string &run_path) {
  // Any live process whose cmdline contains the runner path substring?
  // Returns the first such pid, or -1 for "none found".
  for (auto &ent : std::filesystem::directory_iterator("/proc")) {
    const std::string pid = ent.path().filename().string();
    if (!std::all_of(pid.begin(), pid.end(), ::isdigit)) {
      continue;
    }
    const auto p = std::atoi(pid.c_str());
    if (p == ::getpid()) {
      continue;
    }
    std::FILE *f = std::fopen((ent.path() / "cmdline").string().c_str(), "rb");
    if (!f) {
      continue;
    }
    char buf[4096];
    const size_t n = std::fread(buf, 1, sizeof(buf) - 1, f);
    std::fclose(f);
    if (n > 0) {
      buf[n] = '\0';
      if (std::strstr(buf, run_path.c_str()) != nullptr) {
        return p;
      }
    }
  }
  return -1;
}

int walk_mode(const Paths &p) {
  const std::string manifest =
      "id = \"com.modukit.demo.DemoService\"\nversion = \"0.1.0\"\n"
      "[entry]\nkind = \"exe\"\nartifact = \"" +
      p.so + "\"\nrunner = \"" + p.run + "\"\n";
  const modukit::Kernel host;
  if (host.abi_version() != 1) {
    std::fprintf(stderr, "sup: embed ABI %d != 1\n", host.abi_version());
    return 1;
  }
  const char *id_out = nullptr;
  const auto ins = host.sup_install(manifest.c_str(), &id_out);
  if (!ins.ok() || id_out == nullptr) {
    std::fprintf(stderr, "sup_install -> %d\n", ins.raw());
    return 1;
  }
  std::int32_t state = -1;
  if (!host.sup_state(id_out, &state).ok() || state != 4 /*ACTIVE*/) {
    std::fprintf(stderr, "sup state=%d want ACTIVE(4)\n", state);
    return 1;
  }
  std::printf("READY %d\n", ::getpid());
  std::fflush(stdout);
  std::int32_t actions = -1;
  if (!host.sup_step(&actions).ok() || actions != 0) {
    std::fprintf(stderr, "sup step actions=%d want 0\n", actions);
    return 1;
  }
  std::int32_t dstatus = 0;
  std::uint64_t dfired = 0;
  const auto dial = host.sup_dial(id_out, 1000000000, &dstatus, &dfired);
  // modukit-run boots the Step domain by default (the supervisor's wire
  // dial is the sole time authority there), so this child TURNS: typed
  // status 0 and a fired stamp. (The M2-era word demanded refusal on an
  // assumed Real-domain child - a wrong premise that was never observed
  // because the parent killed right after READY; the M3 fidelity gate
  // makes these words reachable, so the word is corrected to reality.
  // A Real-domain child takes the ClockDialRefused arm instead.)
  if (!dial.ok() || dstatus != 0) {
    std::fprintf(stderr, "sup_dial must turn the step-domain child (raw=%d status=%d)\n",
                 dial.raw(), dstatus);
    return 1;
  }
  std::printf("WALK WORDS OK (install/ACTIVE/step/dial-turn status=%d)\n", dstatus);
  std::fflush(stdout);

  /* M3 FIDELITY stanza (D94 W3): the cpp fixture's Echo answers with the
     request payload verbatim, so the response's payload BYTES must equal
     the request's exactly - typed protobuf bytes riding ABI -> TL wire ->
     child service -> back. Parent-side claim, parent-side gate: the
     runner's own self-check guard stays byte-agnostic (PIT-18 law: an
     empty probe + pass-on-Ok, never re-codify one fixture's dialect). */
  com::modukit::demo::EchoRequest req;
  req.set_payload("sup-fidelity-payload");
  const std::string req_bytes = req.SerializeAsString();
  std::vector<std::uint8_t> resp(req_bytes.size() + 256, 0);
  const auto call = host.sup_service_call(
      kServiceId, "echo",
      reinterpret_cast<const std::uint8_t *>(req_bytes.data()), req_bytes.size(),
      resp.data(), resp.size());
  std::string want = req.payload();
  if (std::getenv("DEMO_STAGE4_M3_SABOTAGE") != nullptr) {
    /* THE single named sabotage point for the fidelity gate (documented
       in scripts/demo-stage4-m2.sh): corrupt the expected bytes before
       the comparison - the gate MUST go red on a byte-exact child. */
    want.back() = static_cast<char>(want.back() ^ 0x20);
    std::printf("SABOTAGE: expected payload bytes corrupted\n");
    std::fflush(stdout);
  }
  bool fidelity_ok = call.ok() && call.raw() > 0;
  if (fidelity_ok) {
    com::modukit::demo::EchoReply reply;
    fidelity_ok = reply.ParseFromArray(resp.data(), call.raw()) &&
                  reply.payload().size() == want.size() &&
                  std::memcmp(reply.payload().data(), want.data(), want.size()) == 0 &&
                  reply.status() == com::modukit::demo::DEMO_STATUS_OK;
  }
  if (!fidelity_ok) {
    std::fprintf(stderr, "FIDELITY FAIL (sup_service_call raw=%d)\n", call.raw());
    return 1; // no marker: the parent loop + gate stay red
  }
  std::printf("FIDELITY PASS (%zu typed bytes out, payload byte-exact)\n",
              static_cast<std::size_t>(call.raw()));
  std::fflush(stdout);
  for (;;) {
    ::sleep(1); // the parent kills us; the runner must survive us by no time
  }
}

} // namespace

int main(int argc, char **argv) {
  bool proof = false;
  std::vector<char *> rest;
  for (int i = 1; i < argc; ++i) {
    if (std::strcmp(argv[i], "--proof") == 0) {
      proof = true;
    } else {
      rest.push_back(argv[i]);
    }
  }
  Paths p;
  if (!rest.empty()) {
    p.so = rest[0];
  }
  if (rest.size() > 1) {
    p.run = rest[1];
  } else if (const char *env = std::getenv("MODUKIT_RUNNER"); env != nullptr) {
    p.run = env;
  } else {
    p.run = "out/cargo-target/debug/modukit-run";
  }
  if (p.so.empty()) {
    std::fprintf(stderr, "usage: host_sup_demo [--proof] <plugin.so> [runner]\n");
    return 2;
  }
  if (!std::filesystem::exists(p.so) || !std::filesystem::exists(p.run)) {
    std::fprintf(stderr, "sup demo: fixture or runner absent (%s | %s) - skip\n",
                 p.so.c_str(), p.run.c_str());
    return 77;
  }
  if (!proof) {
    return walk_mode(p);
  }

  // ---- parent: fork WALK child, handshake, SIGKILL, census ----
  int fds[2];
  if (::pipe(fds) != 0) {
    return 1;
  }
  const pid_t kid = ::fork();
  if (kid == 0) {
    ::dup2(fds[1], STDOUT_FILENO);
    ::close(fds[0]);
    ::close(fds[1]);
    std::vector<char *> args;
    args.push_back(argv[0]);
    for (auto *a : rest) {
      args.push_back(a);
    }
    args.push_back(nullptr);
    ::execv("/proc/self/exe", args.data());
    std::_Exit(127);
  }
  if (kid < 0) {
    return 1;
  }
  ::close(fds[1]);
  char line[256];
  std::FILE *rd = ::fdopen(fds[0], "r");
  bool ready = false;
  while (std::fgets(line, sizeof(line), rd) != nullptr) {
    std::fputs(line, stdout);
    if (std::strncmp(line, "READY ", 6) == 0) {
      ready = true;
      break;
    }
  }
  if (!ready) {
    std::fprintf(stderr, "SUP-DEMO FAIL (walk child never reached READY)\n");
    ::kill(kid, SIGKILL);
    return 1;
  }
  // M3: the walk child must clear the fidelity stanza before the kill.
  // A child that dies without printing the marker (call refused, bytes
  // mismatched, wire timeout) lands here red - the gate is load-bearing
  // parent-side; the runner's own empty-probe guard stays untouched
  // (PIT-18). shortcut: no read timeout - the walk child always reaches
  // marker-or-death (bounded wire/dial calls), so fgets ends on EOF.
  bool fidelity = false;
  while (std::fgets(line, sizeof(line), rd) != nullptr) {
    std::fputs(line, stdout);
    if (std::strncmp(line, "FIDELITY PASS", 13) == 0) {
      fidelity = true;
      break;
    }
  }
  if (!fidelity) {
    std::fprintf(stderr,
                 "SUP-DEMO FAIL (no FIDELITY PASS - walk child died at the door)\n");
    ::kill(kid, SIGKILL);
    ::waitpid(kid, nullptr, 0);
    return 1;
  }
  std::fflush(stdout);
  const bool sabotage = std::getenv("DEMO_STAGE4_M2_SABOTAGE") != nullptr;
  if (sabotage) {
    // THE named M2 sabotage point: stopping the host instead of killing
    // it leaves the runner's control fd open - the census below MUST find
    // a live runner, proving the census is load-bearing.
    std::printf("SABOTAGE: SIGSTOP instead of SIGKILL (runner expected live)\n");
    ::kill(kid, SIGSTOP);
  } else {
    ::kill(kid, SIGKILL);
    ::waitpid(kid, nullptr, 0);
  }
  const int deadline_ms = sabotage ? 1000 : 5000;
  for (int waited = 0;; waited += 100) {
    const int survivor = census(p.run);
    if (survivor < 0) {
      if (sabotage) {
        std::fprintf(stderr, "SUP-DEMO FAIL (sabotage set but no survivor - "
                             "census is not load-bearing)\n");
        ::kill(kid, SIGKILL);
        return 1;
      }
      std::printf("runner self-reaped under a C++ host (orphan-proof)\n");
      std::printf("SUP-DEMO PASS\n");
      return 0;
    }
    if (waited >= deadline_ms) {
      if (sabotage) {
        std::printf("SABOTAGE confirmed: runner pid=%d survived host stop\n", survivor);
        ::kill(kid, SIGKILL);
        return 1; // gate must be red under sabotage: it prints no PASS line
      }
      std::fprintf(stderr, "SUP-DEMO FAIL: runner pid=%d orphaned after host kill\n",
                   survivor);
      ::kill(kid, SIGKILL);
      return 1;
    }
    if (sabotage) {
      continue; // sabotage path must OBSERVE the survivor; give it the wait
    }
    ::usleep(100 * 1000);
  }
}
