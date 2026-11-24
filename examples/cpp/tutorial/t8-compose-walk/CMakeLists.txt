# t8 - the composition walk (see t8_compose.cc for the lesson). Links the
# generated wrapper + the core staticlib like the other drivers; ctest
# drives it against the t7 composer.
add_executable(t8-compose t8_compose.cc)
set_target_properties(t8-compose PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "${MODUKIT_IDE_ROOT}examples/cpp/tutorial")
target_link_libraries(t8-compose PRIVATE modukit_cpp modukit-core-static ${CMAKE_DL_LIBS})
add_dependencies(t8-compose t7-compose-plugin modukit-core-build)
add_test(NAME t8-compose
    COMMAND t8-compose "$<TARGET_FILE:t7-compose-plugin>")
set_tests_properties(t8-compose PROPERTIES
    WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
    SKIP_RETURN_CODE 77
    LABELS "demo;cpp;headless")
