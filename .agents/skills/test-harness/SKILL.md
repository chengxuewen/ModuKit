---
name: test-harness
description: "Multi-language automated test harness. Generates test skeletons from ModuKit SDD clauses (Rust/TS/Python/C++/C), enforces the AAA pattern, reverse-traces tests to clauses, produces coverage reports. Interactive menu-driven. Stage-aware (skips not-yet-landed clauses). Use when a module needs a test skeleton from its clause, or coverage must be reported."
type: tooling
notes: "SDD to tests harness"
---

# Test Harness

Automated test generation and verification for multi-language projects. Test code comes straight out of the ModuKit SDD clauses, with AAA pattern, stage alignment, and language-idiom consistency guaranteed.

**Philosophy**: tests are not an afterthought bolted onto finished code — they grow directly from the spec. A good test file = the executable copy of the spec.

---

## Entry: test task types

### `/test-harness` (no arguments)
Pops up the task-type menu:

```
[1] SDD→tests — generate test files from clauses (stubs / AAA skeletons / fully filled)
[2] tests→spec — reverse tracing (coverage verification / missing scenarios)
[3] run cases — execute tests and fix failing ones
[4] incremental tests — produce incremental tests from the current git diff
[5] project init — set up test infrastructure for a new module
[6] coverage report — generate coverage analysis and gaps
[7] selective testing — run only tests affected by changes (git-diff driven)
```

### `/test-harness generate`
Skips the menu; goes straight into SDD→test generation mode.

### `/test-harness run`
Skips the menu; runs tests directly.

### `/test-harness quick`
Only fixes currently failing tests; generates no new tests.

---

## ModuKit SDD namespaces (B6)

The five clause prefixes are `KERNEL`, `EMBED`, `WIRE`, `BUS`, `TRANSPORT`
(`docs/sdd/`). A clause is an index entry; the existing gate stays the authority.

Tag convention: `// spec: PREFIX-NN` in Rust/C/C++/JS, `# spec: PREFIX-NN` in
Python/shell. Scan roots: `crates/*/src`, `crates/*/tests`, `test-plugins/`,
`examples/`. A clause whose proof is a demo script is named in the ownership table,
not forced into a tag.

## Multi-language strategy

This project is multi-language. Test generation must adapt to each language's idiom.

### Language detection

Detect the target language automatically from project structure:

| Signal | Detected language |
|------|---------|
| `Cargo.toml` exists | Rust |
| `package.json` + `tsconfig.json` | TypeScript |
| `pyproject.toml` / `setup.py` | Python |
| `CMakeLists.txt` + `.cpp`/`.hpp` | C++ |
| `CMakeLists.txt` + `.c`/`.h` (no .cpp) | C |
| multiple signals | interactive choice |

### Per-language test idioms

#### Rust
- **Unit tests**: `#[cfg(test)] mod tests { ... }` inline in the source file
- **Integration tests**: `tests/` directory, each file an independent crate
- **Mocks**: `mockall` crate, `mock!` macro
- **Assertions**: `assert_eq!` / `assert!` / `assert!(matches!(...))`
- **Run**: `cargo test`
- **Idioms**:
  - function naming: `test_<module>_<scenario>`
  - no `unwrap()` / `expect()` — use `?` or `assert!(result.is_ok())`
  - test module starts with `use super::*;`

#### TypeScript
- **Unit tests**: `*.test.ts` or `*.spec.ts` alongside the source file
- **Framework**: Vitest (preferred) / Jest
- **Mocks**: `vi.mock()` / `jest.mock()`
- **Assertions**: `expect(x).toBe(y)` / `expect(x).toEqual(y)`
- **Run**: `npx vitest` / `npx jest`
- **Idioms**:
  - `describe('ModuleName', () => { it('should ...', () => { ... }) })`
  - AAA comments: `// Arrange` / `// Act` / `// Assert`

#### Python
- **Unit tests**: `test_*.py` alongside the source file or in `tests/`
- **Framework**: pytest
- **Mocks**: `unittest.mock` / `pytest-mock`
- **Assertions**: plain `assert` (pytest style)
- **Run**: `pytest`
- **Idioms**:
  - function naming: `test_<module>_<scenario>`
  - class organization: `class Test<Module>:`
  - fixtures: `@pytest.fixture`

#### C++
- **Unit tests**: `*_test.cpp` in the `tests/` directory
- **Framework**: GoogleTest (gtest/gmock)
- **Mocks**: `MOCK_METHOD` macro
- **Assertions**: `EXPECT_EQ` / `ASSERT_TRUE`
- **Run**: `ctest` or `cmake --build build && ctest --test-dir build`
- **Idioms**:
  - `TEST(TestSuiteName, TestName) { ... }`
  - AAA comments

#### C
- **Unit tests**: `test_*.c` in the `tests/` directory
- **Framework**: Unity / CMock
- **Run**: `ctest` or `make test`
- **Idioms**:
  - `TEST_ASSERT_EQUAL(expected, actual)`
  - `setUp() / tearDown()` lifecycle

---

## Workflow

### Mode 1: SDD → test generation

```
/test-harness generate
```

#### Generation tiers (asked every time)

1. **stubs**: function signatures only + `todo!()` / `fail()` / `pytest.fail()` — compiles, tests fail
2. **AAA skeleton**: Arrange/Act/Assert comments + placeholders → structure ready, assertions to fill
3. **full fill**: concrete values extracted from the spec, assertions complete and runnable — expected to pass directly

#### Step 1: identify spec sources

Scan the `docs/sdd/` directory automatically:
```
docs/sdd/
├── kernel.md       → KERNEL-01 lifecycle states/edges
├── embed.md        → EMBED-01 24-verb ordered parity
├── wire.md         → WIRE-01 frame header + wire version
├── bus.md          → BUS-01 MSG_FORWARD/MSG_DELIVER
└── transport.md    → TRANSPORT-01 SCM_RIGHTS/CTRUNC
```

Let the user choose clause files (single or multi-select).

#### Step 2: Stage-aware filtering

Read `status.md`'s `## Phase` paragraph and the landed `docs/stage-*.md` plans.
The five clauses above are the only current SDD surface; `scripts/spec-trace.sh`
proves each has a test tag.

| Stage / volume | Usable clauses |
|-------|---------|
| Stage 1-3 | KERNEL-01 |
| Stage 4 / wave-2 | KERNEL-01 + EMBED-01 + WIRE-01 + BUS-01 + TRANSPORT-01, as landed |
| Future clause | not landed — emit a stub and mark it |

Filter clause items automatically:
- clause has a `// spec:` tag (or `# spec:` in a non-Rust test) → generate first, fully filled
- contract exists but no test tag → AAA skeleton; recommend `spec-trace`
- future/unlisted clause → stub placeholder with a stage note

#### Step 3: generate test files

For each spec item, generate a test function in the target language:

```
input: KERNEL-01
  - preconditions: initial state
  - operation: spawn → stop
  - expected: legal state edges only
  - test mapping: test_kernel_01_lifecycle

output (Rust):
  #[test]
  fn test_type_01_bool_roundtrip() {
      // Arrange
      let values = vec![true, false];
      // Act
      for val in values {
          let encoded = encode_bool(val);
          let decoded = decode_bool(&encoded);
          // Assert
          assert_eq!(val, decoded, "Bool roundtrip failed");
      }
  }
```

#### Step 4: write and verify

1. write the test file (inline unit test / new integration-test file)
2. run the compile check (no syntax errors)
3. run the tests (expectation: some pass, some fail → mark the to-be-implemented)
4. report: `generated N test functions → M passed / K failed / P skipped`

---

### Mode 2: tests → spec reverse tracing

```
/test-harness trace
```

Check existing test coverage, cross-referenced against the SDD clauses:

1. run `bash scripts/spec-trace.sh` to confirm clause→tag pairs
2. scan all test files; extract clause tags (`// spec: KERNEL-01`, etc.)
3. produce a coverage matrix:

```
Clause     | Test function                       | Status
-----------|------------------------------------|------
KERNEL-01  | exe_spawn_green_walks_full_...      | ✅
EMBED-01   | plugin_table_field_lists...         | ✅
WIRE-01    | frame_codec_stamps_the_wire_...     | ✅
BUS-01     | wall_out_runs_the_same_walk...      | ✅
TRANSPORT-01 | the_fdpass_shim_round_trips...    | ✅
```

4. mark:
   - ❌ missing → recommend generating from the clause and adding a `// spec:` tag
   - ⚠️ incomplete → boundary conditions uncovered
   - ✅ complete

---

### Mode 3: case execution and repair

```
/test-harness run
```

1. run the current project test suite
2. collect the failure list
3. for each failure:
   - read the test source
   - read the corresponding implementation source
   - adjudicate root cause: test error vs implementation error
   - auto-fix test errors (wrong assertions, missing mocks)
   - mark implementation errors → report to the user

**Root-cause adjudication rules**:
- test logic disagrees with the spec → test error
- wrong assertion value (expects 1 but the spec says 2) → test error
- implementation incomplete / interface changed → implementation error

**Never**: modify implementation code to make tests pass (unless the user explicitly asks).

---

### Mode 4: incremental tests

```
/test-harness incremental
```

Identify changes from `git diff` → generate matching tests:

1. `git diff --name-only` to list changed files
2. reverse-map files to SDD clauses (file path → module → clause ID)
3. generate incremental tests only for clauses related to the changes
4. if a changed file has no tests yet → initialize a test file
(note: Mode 7 runs affected tests; Mode 4 generates incremental tests)

---

### Mode 5: project initialization

```
/test-harness init
```

Create test infrastructure for new modules:

**Rust**:
- `tests/` directory + integration-test entry points
- crate-level `#[cfg(test)]` helper module
- `mockall` dependency check

**TypeScript**:
- `vitest.config.ts` / `jest.config.ts`
- `__tests__/` directory
- test setup file

**Python**:
- `tests/__init__.py` + `conftest.py`
- pytest config (pyproject.toml)

**C++**:
- `CMakeLists.txt` test targets
- `tests/` directory + CMakeLists.txt
- gtest integration

**C**:
- `CMakeLists.txt` test targets
- Unity/CMock framework download

---

### Mode 6: coverage report

```
/test-harness coverage
```

1. run tests with coverage
2. aggregate and display per module:

```
Module              | Line cov   | Branch cov | Clause coverage
-------------------|----------|----------|----------
modukit-core       | ...      | ...      | KERNEL-01 / EMBED-01 / BUS-01
modukit-transport  | ...      | ...      | TRANSPORT-01
modukit-codegen    | ...      | ...      | WIRE-01
modukit-js         | ...      | ...      | cross-cutting
───────────────────|──────────|──────────|────────
TOTAL              | ...      | ...      | 5 clauses
```

3. rank the gaps: uncovered clause items / low-branch-coverage functions
4. recommend: the N tests to fill first

---

### Mode 7: selective test runner

```
/test-harness selective
```

> Note: early-stage crates may still be stubs with no real test targets yet. Mode 7 uses `cargo metadata` to check crate existence before suggesting test commands.
Identify changes from `git diff` → run only the affected tests (not a gate; a scheduling tool):

1. `git diff --name-only HEAD` to list changed files
2. map files to test targets:

| Changed-path match | Test command |
|---|---|
| `crates/<name>/` | `cargo test -p <name>` |
| `Cargo.toml` or `Cargo.lock` | `cargo test --workspace` |
| cross-crate (3+ crates) | `bash scripts/check.sh` |

3. run tests (one layer at a time, avoiding cargo lock contention)
4. report: layer / command / test count / passed / failed / duration

**Never**: run `cargo test` for multiple crates simultaneously. Don't diagnose failures — report only. >20 changed files → suggest `cargo test --workspace`.

---

## Interactive mode

All test-generation operations show a diff preview before writing files, for user confirmation:

```
The following changes will be generated:
  crates/<name>/src/types.rs +45 (inline test module)
  tests/integration/test_type_roundtrip.rs (new file, 150 lines)
  tests/integration/test_qos_security.rs (new file, 80 lines)

Total: 3 files, +275 lines, 30 test functions

Execute? [Y/n]
```

---

## Stage-aware rules

Read `status.md`'s `## Phase` paragraph and the current `docs/stage-*.md` plan.

| Checkpoint | Behavior |
|--------|------|
| a clause-required trait isn't landed yet | generate a stub or skip; annotate "⏭️ not landed" |
| clause has a test tag | full fill + assertions |
| clause exists without a tag | AAA skeleton; recommend `spec-trace` |
| future/unlisted clause | stub placeholder |

When a new clause lands, re-run `generate` → previously skipped tests get filled automatically.

---

## Quality rules

1. **one assertion goal per test** — one test function verifies one spec item
2. **AAA comments must be explicit** — `// Arrange` / `// Act` / `// Assert` never omitted
3. **never edit tests to let an implementation pass** — spec > test > implementation (priority order)
4. **no unwrap/expect** — avoid even in tests; use `assert!(result.is_ok())`
5. **boundary conditions first** — every entry in the clause's boundary-conditions field → its own test
6. **traceable naming** — test function name includes the clause ID (e.g. `test_kernel_01_*`)
7. **Stage alignment** — never generate tests the current stage cannot run

---

## Quick reference

```
/test-harness               → choose the task type
/test-harness generate      → SDD→test generation
/test-harness run           → run and repair tests
/test-harness trace         → tests→spec reverse tracing
/test-harness incremental   → incremental tests (git diff)
/test-harness init          → initialize test infrastructure
/test-harness coverage      → coverage report
/test-harness selective    → selective test runner (git diff)
```

### Language command cheat sheet

| Language | Run tests | Coverage | Mock library |
|------|---------|---------|---------|
| Rust | `cargo test` | `cargo tarpaulin` | mockall |
| TypeScript | `npx vitest` | `npx vitest --coverage` | vi.mock() |
| Python | `pytest` | `pytest --cov` | pytest-mock |
| C++ | `ctest` | `gcov + lcov` | gmock |
| C | `ctest` | `gcov + lcov` | CMock |

## Rationalizations (anti-excuse)

| Excuse | Rebuttal |
|--------|----------|
| "tests pass, we're done" | Passing tests cover the schema, not the edge cases; check the uncovered paths. |
| "skip the slow integration test" | The gate runs the suite; a skipped test is a silent hole. |
| "snapshot is enough" | Snapshots prove shape, not behavior; assert the behavior too. |
| "the property test will be flaky" | A flaky property is a shrinking bug; seed the case and pin it. |

## Red Flags

- A new code path with no test at all.
- `#[ignore]` / `skip` added to make the suite green.
- Assertions weakened or deleted.
- Snapshot updated without reading the diff.
- Tests that hit the network/clock without a seam.

## Verification (evidence)

- The exact `cargo test` / `npx vitest` / `pytest` output (green, with counts).
- For a new path: the test name plus the failing-without-fix proof.
- Coverage delta or the specific edge cases enumerated.
