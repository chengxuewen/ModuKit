# examples/cpp/tutorial/ - the progressive C++ tutorial series (onboarding
# deepening follow-up: the CppMS doc/src/tutorial shape - one minimal
# project per step, each compiled by the cmake graph, walked by the
# docs/tutorials chapters). t1 hello / t2 echo / t3 bus / t4 the driver.
#
# The series rides the cpp band's environment: it needs the generated
# embed wrapper (t4) but the PLUGINS are deliberately shelf-free raw ABI
# (t1-t3 include contracts/modukit_plugin.h only) - the tutorial shows the
# handshake itself, not the codegen.
if(NOT TARGET modukit_cpp OR NOT TARGET modukit-core-static)
  message(STATUS "examples/cpp/tutorial: cpp shelf/env incomplete -- skipped (auto-skip)")
  return()
endif()

add_subdirectory(t1-hello-plugin)
add_subdirectory(t2-echo-plugin)
add_subdirectory(t3-bus-plugin)
add_subdirectory(t4-driver)
