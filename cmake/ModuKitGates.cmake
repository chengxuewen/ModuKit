# ModuKitGates.cmake - M8.5 W4 probe helper: request the cmake File API
# codemodel so the FOLDER/naming table can be asserted from
# <build>/.cmake/api/v1/reply/target-*.json (plan section 9, W4 row:
# "FOLDER/naming table asserted via cmake file API probe"). Read-only
# evidence surface: nothing here changes any target.
#
# cmake-file-api(7) mechanism as measured on cmake 4.4.4 (2026-10-16): the
# request is an EMPTY shared-stateless query file
# <build>/.cmake/api/v1/query/codemodel-v2 (there is no file(API)
# subcommand); CMake reads the query when it generates the build system,
# but a query first written mid-configure yields replies only from the
# NEXT configure of the same tree - the evidence run configures twice,
# then reads <build>/.cmake/api/v1/reply/target-*.json. Idempotent touch.

function(modukit_gates_codemodel_probe)
  set(_qdir "${CMAKE_BINARY_DIR}/.cmake/api/v1/query")
  file(MAKE_DIRECTORY "${_qdir}")
  file(TOUCH "${_qdir}/codemodel-v2")
endfunction()
