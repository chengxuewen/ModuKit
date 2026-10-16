# Assembly layer (D66 - five-part order modeled on the VisiaEngine consumer shape).
# Binding lanes + examples + install ride this graph later per docs/stage-1-plan.md.

cmake_minimum_required(VERSION 3.25)   # PolyOrch embed floor (D66; VisiaEngine D39 equivalent)

# Version single source: the workspace Cargo.toml (do not copy iceoryx2's five-literal debt -
# mismatch = hard configure error; D67 grammar, VisiaEngine root pattern)
file(READ "${CMAKE_CURRENT_SOURCE_DIR}/Cargo.toml" _ws)
string(REGEX MATCH "version = \"([0-9]+\\.[0-9]+\\.[0-9]+)\"" _m "${_ws}")
if(NOT _m)
  message(FATAL_ERROR "Failed to read the Cargo.toml workspace version (single-source contract broken)")
endif()

project(modukit VERSION ${CMAKE_MATCH_1} LANGUAGES CXX)   # CXX identity required before the PolyOrch embed (PlatformSupport contract, observed first-red M1)

enable_testing()                       # must run before ALL add_subdirectory calls (measured lesson)

include(cmake/ModuKitPolyOrch.cmake)
modukit_ensure_polyorch()              # sentinel embed: reuse superbuild copy first-wins, else gitlink pin
modukit_ensure_polyorch_rust()         # T-B1 translation: MODUKIT_USE_PIXI -> polyorch_rust_setup(FROM pixi)

# add_subdirectory(crates/...) lanes arrive with M8.5 W2 - see docs/stage-1-plan.md section 9
