# t2 - the first service (see t2.cpp for the lesson).
add_library(t2-echo-bundle SHARED t2.cpp)
set_target_properties(t2-echo-bundle PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "${MODUKIT_IDE_ROOT}examples/cpp/tutorial")
target_include_directories(t2-echo-bundle PRIVATE
    "${CMAKE_SOURCE_DIR}/contracts")
