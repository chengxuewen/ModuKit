@echo off
REM modukit.bat -- execution front door (Windows, best-effort; mirrors modukit.sh).
REM Pure passthrough to `pixi run <task> [args...]`.
setlocal
set "DIR=%~dp0"
set "PIXI="
where pixi >nul 2>nul && set "PIXI=pixi"
if not defined PIXI if exist "%USERPROFILE%\.pixi\bin\pixi.exe" set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"
if not defined PIXI (
    echo modukit.bat: pixi not found ^(run bootstrap.bat^) 1>&2
    exit /b 127
)
"%PIXI%" run --manifest-path "%DIR%pixi.toml" %*
