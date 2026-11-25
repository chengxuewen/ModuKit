add_library(cpp-example-bundle SHARED src/plugin.cc)
set_target_properties(cpp-example-bundle PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "test-plugins/cpp/cpp-example-bundle")
target_link_libraries(cpp-example-bundle PRIVATE modukit_cpp)
