add_library(cpp-bad-bundle SHARED src/bundle.cc)
set_target_properties(cpp-bad-bundle PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "test-bundles/cpp/cpp-bad-bundle")
# deliberately no protobuf and no shelf link: the ABI-lie story is the
# descriptor alone (twin of the rs bad-bundle)
target_include_directories(cpp-bad-bundle PRIVATE "${CMAKE_SOURCE_DIR}/contracts")
