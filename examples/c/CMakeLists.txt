# examples/c/ - the C embed demo band (onboarding-plan E2, ruling A1
# B-expanded: "embed from your language" ships a C cell too - a host
# written in plain C, linking the kernel's HostEmbedV1 staticlib and
# driving the C fixture band). Auto-skips unless the test-plugins/c band
# is on AND the core staticlib target exists (the cpp band's guard
# grammar, keyed on the C band's own targets).
if(NOT TARGET c-example-plugin OR NOT TARGET modukit-core-static)
  message(STATUS "examples/c: c band/env incomplete -- embed demo skipped (auto-skip)")
  return()
endif()

add_executable(c-host-demo host_demo.c)
set_target_properties(c-host-demo PROPERTIES
    C_STANDARD 11 C_STANDARD_REQUIRED ON
    FOLDER "${MODUKIT_IDE_ROOT}examples/c")
target_link_libraries(c-host-demo PRIVATE modukit-core-static ${CMAKE_DL_LIBS})
target_include_directories(c-host-demo PRIVATE "${CMAKE_SOURCE_DIR}/contracts")
# the demo dlopens the real built C fixture - it must exist before the test
# runs; the core staticlib comes from the lane mediator (same cold-parse
# reason as the cpp band: the .a is a plain ninja input with no producer
# edge, so the single-target build reaches the cargo rule through this).
add_dependencies(c-host-demo c-example-plugin modukit-core-build)

# ctest bridge, cpp-band grammar: real target = name = test entry,
# SKIP_RETURN_CODE 77 (fixture absent -> host_demo returns 77), the
# fixture path rides argv[1].
add_test(NAME c-host-demo
    COMMAND c-host-demo "$<TARGET_FILE:c-example-plugin>")
set_tests_properties(c-host-demo PROPERTIES
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    SKIP_RETURN_CODE 77
    LABELS "demo;c;headless")
