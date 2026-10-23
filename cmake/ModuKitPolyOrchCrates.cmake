# ModuKitPolyOrchCrates.cmake - M8.5 W2 crates lane (D66 fulfillment: cargo stays
# the build engine, the CMake graph DRIVES it; zero compilation rules here).
# Shape copied from the measured VisiaEngine crates face (cmake/VisiaEngineCrates.cmake,
# 2026-10-16): table form, shared BASE_DIR across build/test/clean, FOLDER = the
# position mirror on disk, per-lane -test/-clean buttons, host-side aggregates
# deliberately FOLDER-less (R10 naming: real target = name = ctest entry, button
# suffixes -run/-test/-clean).
#
# ctest note (benchmark batch 1, D104, 2026-10-23): the verb's own CTEST keyword
# (supplier D45; present at the pin - PolyOrch project version 0.7.1) mints each
# lane's ctest entry from the SAME command vector as the button, so entry and
# button can never drift in argv or env hygiene. The former W4 bridge (two
# sanctioned-internal reads + a hand add_test) is RETIRED here: this file
# touches no supplier internals. The twin project retired the identical pattern
# citing this file's W4 as its own precedent - that debt is now cleared.

option(MODUKIT_POLYORCH_CRATES "register the PolyOrch-driven cargo lanes in the graph" ON)

# Lane table - <package>@<position-mirror folder>@<kind lib|shared>@<cargo lib name>.
# The lib name is the [lib] name from the member Cargo.toml (cargo normalizes
# dashes to underscores); it is what the helper's artifact-name table stamps.
set(MODUKIT_RUST_LANES
    "modukit-core@crates/modukit-core@lib@modukit_core"
    "modukit-transport@crates/modukit-transport@lib@modukit_transport"
    "modukit-codegen@crates/modukit-codegen@lib@modukit_codegen"
    "example-plugin@test-plugins/example-plugin@shared@example_plugin"
    "bad-plugin@test-plugins/bad-plugin@shared@bad_plugin")

# modukit_polyorch_crates() - call AFTER modukit_ensure_polyorch_rust().
function(modukit_polyorch_crates)
  if(NOT MODUKIT_POLYORCH_CRATES)
    return()
  endif()
  if(NOT COMMAND polyorch_rust_test OR NOT POLYORCH_RUST_FOUND)
    message(STATUS "modukit crates: rust face not available (sentinel/skip path) -- off")
    return()
  endif()

  # Shared cargo cache for every lane surface (build/test/clean ride ONE dir;
  # git-ignored by the out/ rule, D66: no artifacts outside out/).
  set(_base "${CMAKE_SOURCE_DIR}/out/cargo-target")
  set(_mediators "")

  foreach(_entry ${MODUKIT_RUST_LANES})
    string(REPLACE "@" ";" _kv "${_entry}")
    list(GET _kv 0 _pkg)
    list(GET _kv 1 _fold)
    list(GET _kv 2 _kind)
    list(GET _kv 3 _crate)
    string(TOUPPER "${_kind}" _kindw)
    set(_manifest "${CMAKE_SOURCE_DIR}/${_fold}/Cargo.toml")

    # build lane: handle <pkg> (LIB = rlib ordering handle, SHARED = cdylib
    # artifact handle) + mediator <pkg>-build owning the cargo rule.
    polyorch_rust_build(
        TARGET   ${_pkg}
        PACKAGE  ${_pkg}
        CRATE    ${_crate}
        ${_kindw}
        MANIFEST "${_manifest}"
        BASE_DIR "${_base}"
        FOLDER   "${_fold}")

    # -test button + ctest twin from the one vector (default entry name
    # <package>-test matches the button NAME below - census unchanged).
    polyorch_rust_test(
        PACKAGE  ${_pkg}
        NAME     ${_pkg}-test
        TARGET   ${_pkg}
        MANIFEST "${_manifest}"
        BASE_DIR "${_base}"
        FOLDER   "${_fold}"
        CTEST CTEST_LABELS "rust;headless"
            CTEST_WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
            CTEST_SKIP_RETURN_CODE 77)

    # -clean button: every lane's clean clears the SHARED face (rm -rf of the
    # one cache dir; idempotent across lanes - the honest action, not a
    # sibling wipe; same semantics VisiaEngine documents for its rlib face).
    polyorch_rust_clean(
        NAME     ${_pkg}-clean
        BASE_DIR "${_base}"
        FOLDER   "${_fold}")

    list(APPEND _mediators ${_pkg}-build)
  endforeach()

  # modukit-codegen also ships the protoc plugin bin (plan W2 row: lib+bin).
  # Separate handle on the same lane folder; the lane's -test already covers
  # the package's test suites (cargo test builds bins too).
  polyorch_rust_build(
      TARGET   modukit-codegen-bin
      PACKAGE  modukit-codegen
      CRATE    protoc-gen-modukit
      BINARY
      MANIFEST "${CMAKE_SOURCE_DIR}/crates/modukit-codegen/Cargo.toml"
      BASE_DIR "${_base}"
      FOLDER   "crates/modukit-codegen")
  list(APPEND _mediators modukit-codegen-bin-build)

  # Stage-4 M1 embed demo lane: the HostEmbedV1 staticlib exit
  # (libmodukit_core.a) that examples/cpp links. Same crate + lane folder as
  # the LIB handle in the table above, STATIC kind (the codegen-bin
  # precedent: one crate, a second kind-specific handle).
  polyorch_rust_build(
      TARGET   modukit-core-static
      PACKAGE  modukit-core
      CRATE    modukit_core
      STATIC
      MANIFEST "${CMAKE_SOURCE_DIR}/crates/modukit-core/Cargo.toml"
      BASE_DIR "${_base}"
      FOLDER   "crates/modukit-core")
  list(APPEND _mediators modukit-core-static-build)

  # W3: stage1_demo as a lane (cargo example of modukit-core; EXAMPLE kind).
  # Run button = polyorch_rust_run (direct staged-binary exec + the mediator
  # auto-build edge - deliberately not `cargo run`). The demo is argv-free:
  # it self-stages its fixture cdylibs via cargo at runtime (scripts/
  # demo-stage1.sh stays the single script home for the PASS-line gate;
  # nothing is duplicated here - the ctest entry just execs the binary).
  set(_demo_manifest "${CMAKE_SOURCE_DIR}/examples/Cargo.toml")
  polyorch_rust_build(
      TARGET   modukit-example-stage1_demo
      PACKAGE  modukit-examples
      CRATE    stage1_demo
      EXAMPLE
      MANIFEST "${_demo_manifest}"
      BASE_DIR "${_base}"
      FOLDER   "examples/rs")
  polyorch_rust_run(
      TARGET modukit-example-stage1_demo
      FOLDER "examples/rs")
  # ctest entry: exec the staged binary with the routed toolchain in its
  # environment. Two prefixes are load-bearing (measured first-red of this
  # wiring: the demo's internal fixture build died on the mise rustc shim):
  #   CARGO=<routed cargo>          - the demo falls back to bare "cargo";
  #   PATH=<pixi bin dir> first     - cargo resolves its rustc SIBLING via
  #                                     PATH, so the shim dir must not answer
  # (same env hygiene the supplier command wrapper applies; the semicolon
  # escape is inert on POSIX, needed where PATH entries carry ';').
  if(WIN32)
    set(_sep ";")
  else()
    set(_sep ":")
  endif()
  set(_ppath "PATH=${POLYORCH_RUST_BIN_DIR}${_sep}$ENV{PATH}")
  string(REPLACE ";" "\\;" _ppath "${_ppath}")
  add_test(NAME example_stage1_demo
      COMMAND ${CMAKE_COMMAND} -E env "CARGO=${POLYORCH_RUST_CARGO}" "${_ppath}"
              $<TARGET_FILE:modukit-example-stage1_demo>)
  set_tests_properties(example_stage1_demo PROPERTIES
      WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
      SKIP_RETURN_CODE 77
      LABELS "example;rust;headless")
  list(APPEND _mediators modukit-example-stage1_demo-build)

  # Tree-level aggregate: the default build reaches every lane through the
  # graph (the mediators themselves stay non-ALL, T-H10). Deliberately
  # FOLDER-less like the VisiaEngine face aggregates - the per-lane buttons
  # are the user-facing entries.
  add_custom_target(modukit-rust-build ALL
      COMMENT "modukit: building all cargo lanes through the graph (D66)"
      VERBATIM)
  add_dependencies(modukit-rust-build ${_mediators})
endfunction()
