# bindings/cpp/ - the C++ binding shelf (Stage-4 M1, rulings C++-3/C++-5;
# VisiaEngine bindings grammar: include/ + a probe with its own CMakeLists).
#
# Nothing generated is checked in: the shelf materializes at build time in
# this directory's BINARY tree. Two generators, both custom commands:
#   1. stock protoc --cpp_out from contracts/com.modukit.demo.proto (+ the
#      option-vocabulary modukit.proto) - the message layer consumed AS-IS
#      (C++-1: protoc output only, never a hand-built message layer);
#   2. protoc-gen-modukit from the checked-in fdsets - the plugin-face
#      skeleton (com_modukit_demo.hpp, face A) and the embed-face wrapper
#      (modukit_embed.hpp, face B, from the contract header). Same recipe
#      as scripts/regen-golden.sh; the goldens themselves stay untouched.
#
# Auto-skip: when the pinned env carries no protoc/libprotobuf (or the
# cargo lanes are off), the whole shelf plus its ctest probe skip with a
# STATUS line - the graph stays green on hosts without the C++ deps.
#
# protobuf duplicate-registration note: the gencode compiles ONCE into the
# shared modukit-demo-gencode library that every consumer links, so a probe
# process and a dlopened fixture share one descriptor-pool registration.

set(_prefix "")
if(DEFINED POLYORCH_RUST_BIN_DIR)
  get_filename_component(_prefix "${POLYORCH_RUST_BIN_DIR}" DIRECTORY)
endif()
if(_prefix STREQUAL "" AND DEFINED ENV{CONDA_PREFIX})
  set(_prefix "$ENV{CONDA_PREFIX}")
endif()

find_program(MODUKIT_CPP_PROTOC NAMES protoc PATHS "${_prefix}/bin" NO_DEFAULT_PATH)
find_library(MODUKIT_CPP_PROTOBUF NAMES protobuf PATHS "${_prefix}/lib" NO_DEFAULT_PATH)

if(NOT MODUKIT_CPP_PROTOC OR NOT MODUKIT_CPP_PROTOBUF)
  message(STATUS "bindings/cpp: no protoc/libprotobuf in the pinned env -- shelf skipped (auto-skip)")
  return()
endif()
if(NOT TARGET modukit-codegen-bin)
  message(STATUS "bindings/cpp: cargo lanes off (no modukit-codegen-bin) -- shelf skipped (auto-skip)")
  return()
endif()

set(_gen "${CMAKE_CURRENT_BINARY_DIR}/include")
file(MAKE_DIRECTORY "${_gen}")
set(_contracts "${CMAKE_SOURCE_DIR}/contracts")

# 1. stock protoc C++ gencode (M1 PROBE recipe: the pinned env's protoc over
#    the checked-in contract originals; modukit.proto rides as the import).
add_custom_command(
    OUTPUT "${_gen}/com.modukit.demo.pb.h" "${_gen}/com.modukit.demo.pb.cc"
           "${_gen}/modukit.pb.h" "${_gen}/modukit.pb.cc"
    COMMAND "${MODUKIT_CPP_PROTOC}" -I "${_contracts}" --cpp_out "${_gen}"
            com.modukit.demo.proto modukit.proto
    DEPENDS "${_contracts}/com.modukit.demo.proto" "${_contracts}/modukit.proto"
    COMMENT "bindings/cpp: protoc --cpp_out (stock gencode, C++-1)"
    VERBATIM)

# 2. emitter faces (the same two inputs regen-golden.sh feeds the goldens).
add_custom_command(
    OUTPUT "${_gen}/com_modukit_demo.hpp" "${_gen}/modukit_embed.hpp"
    COMMAND $<TARGET_FILE:modukit-codegen-bin>
            "@${_contracts}/org.modukit.runtime.proto.fdset"
            "@${_contracts}/com.modukit.demo.proto.fdset"
            --embed_header "${_contracts}/modukit_embed.h"
            --output_dir "${_gen}"
    DEPENDS "${_contracts}/com.modukit.demo.proto.fdset"
            "${_contracts}/org.modukit.runtime.proto.fdset"
            "${_contracts}/modukit_embed.h"
    COMMENT "bindings/cpp: protoc-gen-modukit (plugin skeleton + embed wrapper)"
    VERBATIM)

add_custom_target(modukit-cpp-gen
    DEPENDS "${_gen}/com.modukit.demo.pb.h" "${_gen}/com.modukit.demo.pb.cc"
            "${_gen}/modukit.pb.cc" "${_gen}/com_modukit_demo.hpp"
            "${_gen}/modukit_embed.hpp")
set_target_properties(modukit-cpp-gen PROPERTIES
FOLDER "bindings/cpp")
add_dependencies(modukit-cpp-gen modukit-codegen-bin)  # the emitter binary is a cargo lane

# The ONE gencode compilation (see the duplicate-registration note above).
add_library(modukit-demo-gencode SHARED
    "${_gen}/com.modukit.demo.pb.cc" "${_gen}/modukit.pb.cc")
add_dependencies(modukit-demo-gencode modukit-cpp-gen)
set_target_properties(modukit-demo-gencode PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    POSITION_INDEPENDENT_CODE ON
    FOLDER "bindings/cpp")
target_include_directories(modukit-demo-gencode PUBLIC "${_gen}")
target_link_libraries(modukit-demo-gencode PUBLIC "${MODUKIT_CPP_PROTOBUF}")

# modukit_cpp: the header-only C++ face (shelf grammar: an INTERFACE target
# over generated include/ + the contract originals it includes).
add_library(modukit_cpp INTERFACE)
set_target_properties(modukit_cpp PROPERTIES FOLDER "bindings/cpp")
target_include_directories(modukit_cpp INTERFACE
    "${_gen}" "${_contracts}")
target_compile_features(modukit_cpp INTERFACE cxx_std_20)
target_link_libraries(modukit_cpp INTERFACE modukit-demo-gencode)

add_subdirectory(probe)
