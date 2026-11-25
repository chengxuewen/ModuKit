# test-bundles/cpp/ - the C++ fixture band (Stage-4 M1: the dl fixtures the
# bindings/cpp probe dlopens). NOT cargo members - this band rides ONLY the
# cmake graph, mirroring test-bundles/{example,bad}-bundle one-for-one:
# cpp-example-bundle the positive dlopen template (D23), cpp-bad-bundle the
# ABI-lie negative fixture the version guard must reject.
#
# The message layer and the dispatch skeleton come from the bindings/cpp
# shelf targets (stock protoc gencode + emitter skeleton, generated at
# build time - nothing hand-copied, contracts/ stays read-only).
if(NOT TARGET modukit_cpp)
  message(STATUS "test-bundles/cpp: bindings/cpp shelf skipped -- fixture band skipped")
  return()
endif()

add_subdirectory(cpp-example-bundle)
add_subdirectory(cpp-busv2-bundle)   # onboarding D2: the D96 v2-door twin (mailbox slots at start)
add_subdirectory(cpp-bad-bundle)
