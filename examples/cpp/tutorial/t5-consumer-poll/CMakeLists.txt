# t5 - the naive consumer (see t5_consumer.cc for the lesson). A host-side
# driver like t4: links the generated wrapper + the core staticlib.
add_executable(t5-consumer t5_consumer.cc)
set_target_properties(t5-consumer PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "${MODUKIT_IDE_ROOT}examples/cpp/tutorial")
target_link_libraries(t5-consumer PRIVATE modukit_cpp modukit-core-static ${CMAKE_DL_LIBS})
add_dependencies(t5-consumer t2-echo-plugin modukit-core-build)
add_test(NAME t5-consumer
    COMMAND t5-consumer "$<TARGET_FILE:t2-echo-plugin>")
set_tests_properties(t5-consumer PROPERTIES
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    SKIP_RETURN_CODE 77
    LABELS "demo;cpp;headless")
