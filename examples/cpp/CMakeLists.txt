# examples/cpp/ - the C++ embed demo band (Stage-4 M1, ruling C++-4 two-proof
# wording: one host embedding the kernel, one C++ plugin). Mirrors the
# bindings/cpp + test-plugins/cpp shelf grammar: hard dependencies on the
# already-mounted targets, auto-skip when the graph carries none of them.
#
# The demo links the GENERATED embed wrapper (modukit_cpp) and the kernel's
# HostEmbedV1 staticlib (modukit-core-static, the STATIC lane handle in
# cmake/ModuKitPolyOrchCrates.cmake) and dlopens the real cpp fixture .so -
# the same link/environment shape the bindings/cpp probe proves.

if(NOT TARGET modukit_cpp OR NOT TARGET cpp-example-plugin OR NOT TARGET modukit-core-static)
  message(STATUS "examples/cpp: cpp shelf/env incomplete -- embed demo skipped (auto-skip)")
  return()
endif()

add_executable(modukit-cpp-host-demo host_demo.cc)
set_target_properties(modukit-cpp-host-demo PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "examples/cpp")
target_link_libraries(modukit-cpp-host-demo PRIVATE
    modukit_cpp modukit-core-static ${CMAKE_DL_LIBS})
# the demo dlopens the real built fixture - it must exist before the test runs
add_dependencies(modukit-cpp-host-demo cpp-example-plugin)

# ctest bridge follows the crates-face grammar: real target = name = test
# entry, SKIP_RETURN_CODE 77 (fixture absent -> host_demo returns 77), and
# the fixture path rides argv[1] (the probe bakes it via a define; the demo
# takes it as the documented argument so a manual run works identically).
add_test(NAME modukit-cpp-host-demo
    COMMAND modukit-cpp-host-demo "$<TARGET_FILE:cpp-example-plugin>")
set_tests_properties(modukit-cpp-host-demo PROPERTIES
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    SKIP_RETURN_CODE 77
    LABELS "demo;cpp;headless")
