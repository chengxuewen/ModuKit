# t3 - the bus (see t3.cpp for the lesson).
add_library(t3-bus-plugin SHARED t3.cpp)
set_target_properties(t3-bus-plugin PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "${MODUKIT_IDE_ROOT}examples/cpp/tutorial")
target_include_directories(t3-bus-plugin PRIVATE
    "${CMAKE_SOURCE_DIR}/contracts")
