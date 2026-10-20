add_executable(modukit-cpp-probe src/probe.cc)
set_target_properties(modukit-cpp-probe PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "bindings/cpp/probe")
target_compile_definitions(modukit-cpp-probe PRIVATE
    MODUKIT_CPP_PROBE_GOOD="$<TARGET_FILE:cpp-example-plugin>"
    MODUKIT_CPP_PROBE_BAD="$<TARGET_FILE:cpp-bad-plugin>")
target_link_libraries(modukit-cpp-probe PRIVATE modukit_cpp ${CMAKE_DL_LIBS})
# the probe dlopens the REAL built fixtures - they must exist before it runs
add_dependencies(modukit-cpp-probe cpp-example-plugin cpp-bad-plugin)

# ctest bridge follows the crates-face grammar (ModuKitPolyOrchCrates.cmake):
# real target = name = test entry, SKIP_RETURN_CODE 77, headless labels.
add_test(NAME modukit-cpp-probe COMMAND modukit-cpp-probe)
set_tests_properties(modukit-cpp-probe PROPERTIES
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    SKIP_RETURN_CODE 77
    LABELS "probe;cpp;headless")
