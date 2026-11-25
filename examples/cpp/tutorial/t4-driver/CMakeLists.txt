# t4 - the driver (see t4_driver.cc for the lesson). Links the generated
# wrapper + the core staticlib exactly like the host demos; ctest runs it
# against BOTH tutorial bundles (t2 exercises the echo scene, t1 the
# bare-lifecycle walk).
add_executable(t4-driver t4_driver.cc)
set_target_properties(t4-driver PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "${MODUKIT_IDE_ROOT}examples/cpp/tutorial")
target_link_libraries(t4-driver PRIVATE modukit_cpp modukit-core-static ${CMAKE_DL_LIBS})
add_dependencies(t4-driver t1-hello-bundle t2-echo-bundle t3-bus-bundle modukit-core-build)

add_test(NAME t4-driver-t2
    COMMAND t4-driver "$<TARGET_FILE:t2-echo-bundle>")
set_tests_properties(t4-driver-t2 PROPERTIES
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    SKIP_RETURN_CODE 77
    LABELS "demo;cpp;headless")
add_test(NAME t4-driver-t1
    COMMAND t4-driver "$<TARGET_FILE:t1-hello-bundle>")
set_tests_properties(t4-driver-t1 PROPERTIES
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    SKIP_RETURN_CODE 77
    LABELS "demo;cpp;headless")
