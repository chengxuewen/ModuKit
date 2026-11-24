# t1 - the minimal plugin (see t1.cpp for the lesson).
add_library(t1-hello-plugin SHARED t1.cpp)
set_target_properties(t1-hello-plugin PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "${MODUKIT_IDE_ROOT}examples/cpp/tutorial")
target_include_directories(t1-hello-plugin PRIVATE
    "${CMAKE_SOURCE_DIR}/contracts")
