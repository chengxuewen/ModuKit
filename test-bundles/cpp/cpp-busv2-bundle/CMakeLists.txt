# cpp-busv2-bundle - the D96 v2-door C++ fixture (onboarding deepening D2;
# the C++ twin of the rs busv2-bundle, D23 template standing). Declares
# ABI v2: the kernel hands the HostAbiV2 table (the v1 prefix plus the four
# mailbox slots) at start, and the plugin publishes ONE seed value on its
# own topic - the LatestValue snapshot a late-join host then reads. The
# registered echo service proves the v1 prefix still answers calls.
add_library(cpp-busv2-bundle SHARED src/busv2_plugin.cc)
set_target_properties(cpp-busv2-bundle PROPERTIES
    CXX_STANDARD 20 CXX_STANDARD_REQUIRED ON
    FOLDER "test-plugins/cpp/cpp-busv2-bundle")
target_include_directories(cpp-busv2-bundle PRIVATE
    "${CMAKE_SOURCE_DIR}/contracts")
