@echo off
rem init-mcp-codegraph.bat - Windows-native twin of init-mcp-codegraph.sh.
rem Check -> install (pinned) -> init -> serve --mcp, on primitives guaranteed
rem on every Windows: cmd.exe + powershell.exe. The release bundle vendors its
rem own Node runtime, so no system Node is required (verified install.ps1 header).
rem Select it by editing the mcp.local-codegraph command LOCALLY to:
rem   ["cmd", "/c", ".opencode\\init-mcp-codegraph.bat"]
setlocal
set "VERSION=v1.6.2"
if defined CODEGRAPH_VERSION set "VERSION=%CODEGRAPH_VERSION%"
for %%I in ("%~dp0..") do set "ROOT=%%~fI"
set "CG=%ROOT%\.opencode\codegraph\current\bin\codegraph.cmd"

if exist "%CG%" goto :graph
echo [codegraph] installing %VERSION% into .opencode\codegraph ... 1>&2
powershell -NoProfile -ExecutionPolicy Bypass -Command "$env:CODEGRAPH_VERSION='%VERSION%'; $env:CODEGRAPH_INSTALL_DIR='%ROOT%\.opencode\codegraph'; irm https://raw.githubusercontent.com/colbymchenry/codegraph/main/install.ps1 | iex" 1>&2
if not exist "%CG%" (
    echo [codegraph] ERROR: installer finished but launcher missing: "%CG%" 1>&2
    exit /b 1
)

:graph
if exist "%ROOT%\.codegraph\" goto :serve
echo [codegraph] no project graph yet - running init, empty repo = fast ... 1>&2
pushd "%ROOT%"
call "%CG%" init 1>&2
if errorlevel 1 echo [codegraph] WARNING: init failed; serve starts with an empty/stale graph 1>&2
popd

:serve
call "%CG%" serve --mcp
