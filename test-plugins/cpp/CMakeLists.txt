# test-plugins/cpp/ - the C++ fixture band (Stage-4 M1: the dl fixtures the
# bindings/cpp probe dlopens). NOT cargo members - this band rides ONLY the
# cmake graph, mirroring test-plugins/{example,bad}-plugin one-for-one:
# cpp-example-plugin the positive dlopen template (D23), cpp-bad-plugin the
# ABI-lie negative fixture the version guard must reject.
#
# The message layer and the dispatch skeleton come from the bindings/cpp
# shelf targets (stock protoc gencode + emitter skeleton, generated at
# build time - nothing hand-copied, contracts/ stays read-only).
if(NOT TARGET modukit_cpp)
  message(STATUS "test-plugins/cpp: bindings/cpp shelf skipped -- fixture band skipped")
  return()
endif()

add_subdirectory(cpp-example-plugin)
add_subdirectory(cpp-bad-plugin)
