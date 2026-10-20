add_library(cpp-example-plugin SHARED src/plugin.cc)
set_target_properties(cpp-example-plugin PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "test-plugins/cpp/cpp-example-plugin")
target_link_libraries(cpp-example-plugin PRIVATE modukit_cpp)
