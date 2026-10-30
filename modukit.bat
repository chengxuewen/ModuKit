@echo off
rem modukit.bat -- execution front door for Windows (best-effort; mirrors modukit.sh).
rem
rem Pure passthrough:  modukit.bat <task> [args...]   ==   pixi run <task> [args...]
rem No task table here on purpose -- pixi.toml is the single source of task names.
rem Not source-able; use pixi.bat when you want an activated sub-shell.
rem
rem Exit codes owned by this file: -h/--help -> 0, no args -> 2, pixi missing -> 127.
rem Windows first-run debt: never exercised on a real Windows box; pixi run remains
rem the supported path if this wrapper misbehaves.

setlocal
cd /d "%~dp0"

if "%~1"=="" goto usage2
if /i "%~1"=="-h" goto usage0
if /i "%~1"=="--help" goto usage0

set "PIXI="
where pixi >nul 2>nul && set "PIXI=pixi"
if not defined PIXI if exist "%USERPROFILE%\.pixi\bin\pixi.exe" set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"
if not defined PIXI (
    echo modukit.bat: pixi not found ^(run bootstrap.bat^) 1>&2
    exit /b 127
)

"%PIXI%" run --manifest-path "%~dp0pixi.toml" %*
exit /b %ERRORLEVEL%

:usage0
echo usage: modukit.bat ^<task^> [args...]
echo.
echo common:
echo   modukit.bat mcp status      MCP: pin/installed versions + index state
echo   modukit.bat mcp setup       install/repair MCP packages at the pin
echo   modukit.bat mcp sync        incremental MCP sync
echo   modukit.bat mcp index       build ^(first run^) or rebuild .codegraph
echo   modukit.bat agents sync     agent config sync ^(rules/mcp/memory/skills^)
echo   modukit.bat agents check    verify every generated layer ^(non-zero on drift^)
echo   modukit.bat gate-agents     single-source consistency gate
echo   modukit.bat ci              the whole local gate
echo.
echo docs (the book, output in www/book/en):
echo   modukit.bat book-build      build the site ^(--public drops internal pages^)
echo   modukit.bat book-serve      serve it on :3000 and rebuild on change
echo   modukit.bat book-check      static checks on the built site
echo   modukit.bat book-graph      refresh graph.json/meta.json only
echo   modukit.bat spec-trace      SDD clauses ^<^-^> spec tags
echo.
echo   pixi task list              every task this workspace defines
echo.
echo pixi run ^<task^> is always equivalent.
exit /b 0

:usage2
echo usage: modukit.bat ^<task^> [args...]   ^(see: modukit.bat -h^) 1>&2
exit /b 2
