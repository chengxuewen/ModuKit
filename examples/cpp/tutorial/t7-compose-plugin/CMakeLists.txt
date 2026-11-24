# t7 - bus-mediated composition (see t7.cpp for the lesson).
add_library(t7-compose-plugin SHARED t7.cpp)
set_target_properties(t7-compose-plugin PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "${MODUKIT_IDE_ROOT}examples/cpp/tutorial")
target_include_directories(t7-compose-plugin PRIVATE
    "${CMAKE_SOURCE_DIR}/contracts")
