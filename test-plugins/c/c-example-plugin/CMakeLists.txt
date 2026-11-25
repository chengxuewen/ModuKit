# c-example-plugin - the positive dlopen fixture for the C band (onboarding
# plan E2, ruling A1: the C twin of the cpp-example-plugin template, D23
# template standing). Pure C: the kernel-facing surface is exactly the v1
# handshake (contracts/modukit_bundle.h) plus ONE service registration -
# no generated shelf, no C++ runtime, nothing but this one source file.
add_library(c-example-plugin SHARED src/hello.c)
set_target_properties(c-example-plugin PROPERTIES
    C_STANDARD 11 C_STANDARD_REQUIRED ON
    FOLDER "test-plugins/c/c-example-plugin")
target_include_directories(c-example-plugin PRIVATE
    "${CMAKE_SOURCE_DIR}/contracts")
