@echo off
rem bootstrap.bat - one-time Windows-native dev setup (pixi only; no admin).
rem Mirrors bootstrap.sh:
rem   1. submodule guard for the PolyOrch pixi-init vendor
rem   2. pixi + repo envs via PolyOrch\scripts\pixi-init.bat
rem   3. locate pixi (where / %USERPROFILE%\.pixi\bin)
rem   4. repo git hooks (core.hooksPath=scripts/hooks)
rem   5. advisory agent-layer sync
rem
rem Windows first-run verification DEBT: this path mirrors the sh twin but has
rem NEVER been executed on Windows - verify on real hardware before trusting it.
setlocal EnableExtensions
cd /d "%~dp0"

echo [bootstrap] step 1/5: ensuring PolyOrch pixi-init ...
if not exist "PolyOrch\scripts\pixi-init.bat" (
    echo [bootstrap] PolyOrch\scripts\pixi-init.bat missing - initialising the submodule
    git submodule update --init --recursive PolyOrch
    if errorlevel 1 (
        echo [bootstrap] ERROR: submodule init failed - check network/git 1>&2
        exit /b 1
    )
    if not exist "PolyOrch\scripts\pixi-init.bat" (
        echo [bootstrap] ERROR: supplier script still missing after init 1>&2
        exit /b 1
    )
)
call PolyOrch\scripts\pixi-init.bat "%~dp0pixi.toml"
if errorlevel 1 (
    echo [bootstrap] ERROR: pixi bootstrap delegation failed 1>&2
    exit /b 1
)

echo [bootstrap] step 2/5: locating pixi ...
set "PIXI="
where pixi >nul 2>nul && set "PIXI=pixi"
if not defined PIXI if exist "%USERPROFILE%\.pixi\bin\pixi.exe" set "PIXI=%USERPROFILE%\.pixi\bin\pixi.exe"
if not defined PIXI (
    echo [bootstrap] ERROR: pixi not resolvable after install 1>&2
    exit /b 1
)

echo [bootstrap] step 3/5: repo git hooks ...
git config core.hooksPath scripts/hooks

echo [bootstrap] step 4/5: advisory agent sync ...
"%PIXI%" run agents sync
if errorlevel 1 echo [bootstrap] WARN: agent sync failed; run: pixi run agents sync

echo [bootstrap] step 5/5: done.
echo   verify:  pixi --version  /  pixi run cargo --version  /  pixi run node --version
echo   mise and rust-toolchain.toml are retired; pixi is the single toolchain.
