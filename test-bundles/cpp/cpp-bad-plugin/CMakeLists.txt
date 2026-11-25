add_library(cpp-bad-plugin SHARED src/plugin.cc)
set_target_properties(cpp-bad-plugin PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "test-plugins/cpp/cpp-bad-plugin")
# deliberately no protobuf and no shelf link: the ABI-lie story is the
# descriptor alone (twin of the rs bad-plugin)
target_include_directories(cpp-bad-plugin PRIVATE "${CMAKE_SOURCE_DIR}/contracts")
