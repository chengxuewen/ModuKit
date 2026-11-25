# c-example-bundle - the positive dlopen fixture for the C band (onboarding
# plan E2, ruling A1: the C twin of the cpp-example-bundle template, D23
# template standing). Pure C: the kernel-facing surface is exactly the v1
# handshake (contracts/modukit_bundle.h) plus ONE service registration -
# no generated shelf, no C++ runtime, nothing but this one source file.
add_library(c-example-bundle SHARED src/hello.c)
set_target_properties(c-example-bundle PROPERTIES
    C_STANDARD 11 C_STANDARD_REQUIRED ON
    FOLDER "test-bundles/c/c-example-bundle")
target_include_directories(c-example-bundle PRIVATE
    "${CMAKE_SOURCE_DIR}/contracts")
