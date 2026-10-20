/* examples/cpp/host_sup_demo.cc - the Stage-4 M2 exe-rung demo with the
   ORPHAN-PROOF from a C++ host (plan §5: "ORPHAN-PROOF-style demo gate
   from the C++ host"). Two modes of one binary:

     (default) WALK : the supervised host. Drives the generated wrapper's
       sup_* arm (bindings/cpp face B): sup_install (real runner spawn +
       HELLO + start), sup_state(ACTIVE), sup_step(idle), sup_dial(REAL
       domain -> typed refusal word), then sleeps until killed.
     --proof   PARENT: fork+exec self in WALK mode, wait for the READY
       handshake line, SIGKILL the child host, then scan /proc: the
       runner must have self-reaped on the control-pipe EOF (S2-M5a
       machinery, now proven under a C++-authored host). Zero survivors
       prints `SUP-DEMO PASS`; any survivor prints the pids and FAILs.

   Sabotage (DEMO_STAGE4_M2_SABOTAGE=1) exists at exactly ONE named point
   below: the parent stops (SIGSTOP) instead of killing the child, so the
   runner legitimately survives - the census MUST go red on it.
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

#include "modukit_embed.hpp"

namespace {

struct Paths {
  std::string so;     // plugin artifact (exe manifest `artifact`)
  std::string run;    // shadow-runner binary (manifest `runner`)
};

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
  if (dial.ok()) { // REAL-domain child: dial answers typed refusal, never
    std::fprintf(stderr, "sup_dial must refuse on a real-clock child\n");
    return 1;
  }                // a silent no-op (header word, same as M1's clock rule)
  std::printf("WALK WORDS OK (install/ACTIVE/step/dial-refusal %d)\n", dial.raw());
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
  const bool sabotage = std::getenv("DEMO_STAGE4_M2_SABOTAGE") != nullptr;
  if (sabotage) {
    // THE ONE named sabotage point: stopping the host instead of killing
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
