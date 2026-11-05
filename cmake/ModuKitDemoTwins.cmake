# ModuKitDemoTwins.cmake - ctest twins for the thirteen demo legs (repo-audit
# HIGH 2026-11-04: the scripts ran only inside check.sh, so the IDE/ctest
# surface could not reach them). Each twin wraps scripts/<leg>.sh and asserts
# the printed PASS token - the same grep grammar the check.sh leg uses, not
# the exit code. Twins carry LABELS "demo-twin": the fast gate (check.sh
# ctest-stage4) CENSUSES these names and excludes them from its run; full
# execution is `ctest -L demo-twin` - acceptance-grade, minutes, like
# `pixi run vendor-test`. Missing scripts mint nothing (fresh-clone safe).
set(_modukit_demo_twins
    demo-stage1 demo-stage2 demo-stage4-m1 demo-stage4-m2 demo-stage5-n2
    demo-wave2-reader demo-wave2-clock demo-wave2-py demo-wave2-py-sup
    demo-wave2-js demo-wave2-js-sup demo-wave2-plugin-py demo-wave2-plugin-js)

function(modukit_add_demo_twins)
  foreach(_leg ${_modukit_demo_twins})
    if(EXISTS "${CMAKE_SOURCE_DIR}/scripts/${_leg}.sh")
      add_test(NAME twin-${_leg}
          COMMAND ${CMAKE_COMMAND} -E env
                  "TMPDIR=${CMAKE_SOURCE_DIR}/out/tmp"
                  bash -c "bash ${CMAKE_SOURCE_DIR}/scripts/${_leg}.sh 2>/dev/null | grep -q '${_leg}: PASS'")
      set_tests_properties(twin-${_leg} PROPERTIES
          WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
          SKIP_RETURN_CODE 77
          LABELS "demo-twin;acceptance")
    endif()
  endforeach()
endfunction()
