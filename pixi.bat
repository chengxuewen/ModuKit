@echo off
REM pixi.bat -- open a child cmd with the pixi workspace in THIS directory active.
REM
REM Installed next to pixi.toml / .pixi\ by polyorch_pixi_init(COPY_SCRIPTS)
REM or polyorch_pixi_scripts_install(). The file is managed by PolyOrch; local
REM edits are overwritten on the next install.
REM
REM cmd has no source/eval model: the peer of `pixi shell-hook` here is
REM `pixi shell`, which spawns an already-activated child shell; `exit` returns.
REM Use pixi.ps1 for an in-place PowerShell activation.
REM
REM NOTE: authored on Linux, not yet exercised on a real Windows machine
REM (best-effort). Requires pixi; if absent, provision it first with
REM polyorch_pixi_tool_ensure() (see examples/pixi-bootstrap/).

set "SCRIPT_DIR=%~dp0"

set "PIXI="
where pixi >nul 2>nul && set "PIXI=pixi"
if not defined PIXI if exist "%USERPROFILE%\.pixi\bin\pixi.exe" set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"
if not defined PIXI (
    echo pixi not found -- provision it first ^(polyorch_pixi_tool_ensure^). 1>&2
    exit /b 1
)

set "MANIFEST=%SCRIPT_DIR%pixi.toml"
if not exist "%MANIFEST%" set "MANIFEST=%SCRIPT_DIR%pyproject.toml"
if not exist "%MANIFEST%" (
    echo no pixi.toml or pyproject.toml beside this script: %SCRIPT_DIR% 1>&2
    exit /b 1
)

"%PIXI%" shell --manifest-path "%MANIFEST%"
