# t6 - the filter consumer (see t6_consumer.cc for the lesson).
add_executable(t6-consumer t6_consumer.cc)
set_target_properties(t6-consumer PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "${MODUKIT_IDE_ROOT}examples/cpp/tutorial")
target_link_libraries(t6-consumer PRIVATE modukit_cpp modukit-core-static ${CMAKE_DL_LIBS})
add_dependencies(t6-consumer t2-echo-plugin modukit-core-build)
add_test(NAME t6-consumer
    COMMAND t6-consumer "$<TARGET_FILE:t2-echo-plugin>")
set_tests_properties(t6-consumer PROPERTIES
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    SKIP_RETURN_CODE 77
    LABELS "demo;cpp;headless")
