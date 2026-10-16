# ModuKitPolyOrch.cmake - thin wrapper over the PolyOrch helper surface (D66 shape).
# Keep this file small: it embeds PolyOrch and verifies the helpers exist. Per-lane
# wiring (rust / crates / wasm) grows here as modukit_* functions when those lanes land
# (the VisiaEngine PolyOrchCrates/PolyOrchWasm wrapper precedent).

# modukit_ensure_polyorch(): sentinel embed, first-wins (D66).
#   - A superbuild that already added PolyOrch wins; we reuse its copy.
#   - Otherwise self-embed the submodule at its gitlink pin (D67: the gitlink is the
#     sole version authority - this wrapper NEVER fetches, never tracks a branch).
function(modukit_ensure_polyorch)
  if(COMMAND polyorch_rust_setup)
    message(STATUS "modukit: reusing PolyOrch provided by the superbuild (first-wins, D66)")
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