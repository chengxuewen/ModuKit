# ModuKit Tutorials

Hands-on learning paths. Every command below runs a standing gate leg - the
repo replays them nightly, so what you copy is what CI proves. Code blocks
are quoted verbatim from compiled fixtures, with `file:line` provenance (the
A5 discipline of `docs/onboarding-plan.md`).

## Who this is for

| You are | Start at | Then |
|---|---|---|
| A newcomer ("what is this thing?") | chapter 01 | the [feature gallery](../features.md) once it lands |
| A bundle author (C / C++ / Python / JS) | chapter 02 | chapters 03-04, then 06 if you write script bundles |
| An embedder (your app drives the kernel) | chapter 05 | chapters 03-04 for the model behind the verbs |

## Platform note

Supported: Linux native, Windows (pixi env; the bash demo legs run under
WSL, the cmake/build face works natively). macOS is not a pixi platform
today - `pixi.toml` declares linux-64/win-64 only.

## The band-choice table

| Band | Bundle face | Host/embed face | Placement |
|---|---|---|---|
| C | `test-bundles/c/c-example-bundle` (pure C, one file) | `examples/c/host_demo.c` | dl |
| C++ | `test-bundles/cpp/cpp-example-bundle` (generated shelf) | `examples/cpp/host_demo.cc` | dl |
| Python | `test-bundles/py/py_echo.py` (script, exe wall) | `examples/py/host_demo.py` (ctypes) | exe |
| JS | `test-bundles/js/js_echo.mjs` (script, exe wall) | `examples/js/host_demo.js` (napi) | exe |

## Chapters

1. [First run](01-first-run.md) - 60 seconds, full lifecycle.
2. [Your first bundle](02-first-bundle.md) - manifest, install gates, the
   sabotage levers (C and C++ bands). Sample manifest:
   [fixtures/manifest-sample.toml](fixtures/manifest-sample.toml).
2b. [The C++ tutorial series](02b-cpp-tutorial-series.md) - four minimal
   projects (hello / echo / bus / driver), CppMS-style progressive steps.
3. [The bus](03-the-bus.md) - topics, late-join, the clock bell.
4. [The exe wall](04-the-exe-wall.md) - processes, crash/revive,
   ORPHAN-PROOF.
5. [Embed from your language](05-embed-from-your-language.md) - C, C++,
   Python, JS hosts on the same verb walk.
6. [Script bundles](06-script-bundles.md) - the Python/JS bundle door and
   the four hidden contracts.

## Where to go next

- The deep-dive canon: [../modules/00-overview.md](../modules/00-overview.md)
  (reading order for newcomer / bundle author / comm-stack).
- The environment: [../env.md](../env.md).
- Everything the kernel can do: the [feature gallery](../features.md) and
  [../capability-ledger.md](../capability-ledger.md).
