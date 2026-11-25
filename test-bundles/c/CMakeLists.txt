# test-bundles/c/ - the C fixture band (onboarding-plan E2, ruling A1
# B-expanded: the bundle face ships for C too, mirroring the cpp band's
# guard STYLE one-for-one). NOT cargo members; the band rides only the
# cmake graph. The C language is enabled at the ROOT scope behind this
# band's own option (a subdirectory-level enable leaves CMAKE_C_COMPILE_
# OBJECT unset - observed cold 2026-11-24), so a MODUKIT_BUILD_C=OFF
# configure never probes a C compiler.
if(NOT MODUKIT_BUILD_C)
  message(STATUS "test-bundles/c: MODUKIT_BUILD_C=OFF -- fixture band skipped")
  return()
endif()

add_subdirectory(c-example-bundle)
