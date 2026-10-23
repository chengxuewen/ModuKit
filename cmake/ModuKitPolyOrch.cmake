# ModuKitPolyOrch.cmake - thin wrapper over the PolyOrch helper surface (D66 shape).
# Keep this file small: it embeds PolyOrch and verifies the helpers exist. Per-lane
# wiring (rust / crates / wasm) grows here as modukit_* functions when those lanes land
# (the VisiaEngine PolyOrchCrates/PolyOrchWasm wrapper precedent).

# modukit_polyorch_version_floor(_mode): T2.1 version floor (wave-2 benchmark
#   plan section 3). Our lanes mint ctest entries via the supplier's CTEST
#   keyword, which landed in PolyOrch 0.7.1 - below that floor the lanes would
#   configure clean but never register a test. Reads INTERFACE_POLYORCH_VERSION
#   off the sentinel target (PolyOrch/CMakeLists.txt:25); a missing value counts
#   as below floor. Grammar mirrored from MediaServoPolyOrch.cmake:39-44/70-76.
#   Called from BOTH branches of modukit_ensure_polyorch() (reuse + self-embed).
function(modukit_polyorch_version_floor _mode)
  if(TARGET PolyOrch::PolyOrch)
    get_target_property(_ver PolyOrch::PolyOrch INTERFACE_POLYORCH_VERSION)
  else()
    set(_ver "NOTFOUND")   # bare sentinel without the target - treated as missing
  endif()
  set(_floor "0.7.1")   # single floor constant - the pin that ships the CTEST keyword
  if(NOT _ver OR _ver VERSION_LESS _floor)
    message(FATAL_ERROR
      "modukit: needs PolyOrch >= ${_floor} (the CTEST keyword our lanes use to mint "
      "ctest entries) but the ${_mode} copy reports '${_ver}'. Fix: raise the "
      "gitlink pin to a >= ${_floor} SHA (standalone chore(deps) bump, C2/D67); in "
      "reuse mode upgrade the superbuild's outer PolyOrch checkout instead.")
  endif()
endfunction()

# modukit_ensure_polyorch(): sentinel embed, first-wins (D66).
#   - A superbuild that already added PolyOrch wins; we reuse its copy.
#   - Otherwise self-embed the submodule at its gitlink pin (D67: the gitlink is the
#     sole version authority - this wrapper NEVER fetches, never tracks a branch).
function(modukit_ensure_polyorch)
  if(COMMAND polyorch_rust_setup)
    message(STATUS "modukit: reusing PolyOrch provided by the superbuild (first-wins, D66)")
    modukit_polyorch_version_floor("superbuild-provided (reuse)")
    return()
  endif()

  set(_po "${CMAKE_CURRENT_FUNCTION_LIST_DIR}/../PolyOrch")
  if(NOT EXISTS "${_po}/CMakeLists.txt")
    message(FATAL_ERROR
      "PolyOrch submodule missing at ${_po} - run: git submodule update --init (gitlink pin, D67)")
  endif()

  add_subdirectory("${_po}" "${CMAKE_BINARY_DIR}/_polyorch" SYSTEM)

  if(NOT COMMAND polyorch_rust_setup)
    message(FATAL_ERROR
      "PolyOrch embedded but its helpers are absent - check the gitlink pin (D67 bump is a standalone commit)")
  endif()
  message(STATUS "modukit: PolyOrch self-embedded at its gitlink pin (D66)")
  modukit_polyorch_version_floor("self-embedded gitlink pin")
endfunction()


option(MODUKIT_USE_PIXI "build toolchain comes from the pinned pixi env (D65)" ON)

# modukit_ensure_polyorch_rust(): T-B1 translation (VisiaEngine pattern) - maps the
# MODUKIT_USE_PIXI master switch onto PolyOrch's rust setup. Call AFTER
# modukit_ensure_polyorch(). FROM pixi REQUIRES the pixi helper face + the repo pixi.toml.
function(modukit_ensure_polyorch_rust)
  if(NOT COMMAND polyorch_rust_setup)
    message(FATAL_ERROR "call modukit_ensure_polyorch() first - PolyOrch helpers absent")
  endif()
  if(MODUKIT_USE_PIXI)
    if(NOT COMMAND polyorch_pixi_setup)
      message(FATAL_ERROR "MODUKIT_USE_PIXI needs the PolyOrch pixi helper face")
    endif()
    polyorch_pixi_setup(COPY_SCRIPTS REPORT)
    polyorch_rust_setup(FROM pixi REQUIRED)
  else()
    polyorch_rust_setup(FROM system REQUIRED)
  endif()
  message(STATUS "modukit: rust face ready (pixi=${MODUKIT_USE_PIXI}, cargo=${POLYORCH_RUST_CARGO})")
endfunction()