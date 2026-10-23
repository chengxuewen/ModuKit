@echo off
rem bootstrap.bat - one-time Windows-native dev-environment setup for ModuKit
rem (user-level, no admin). Same steps as bootstrap.sh:
rem   1. ensure mise   (official GitHub release zip -> %LOCALAPPDATA%\mise\bin)
rem   2. ensure user PATH contains that bin dir (via .NET User-scope API, not setx)
rem   3. mise install  (toolchain pins from mise.toml; no-op when satisfied)
rem   4. ensure pixi   (delegated to PolyOrch\scripts\pixi-init.bat: pinned
rem      auditable installer, already-satisfied no-op, lock-drift retry)
rem   5. git hooks: C3 commit-shape pre-flight (2026-10-23 doc-audit, mirrors
rem      bootstrap.sh stage 6; Windows first-run debt stands)
rem
rem Windows first-run verification DEBT (D104): the delegation below mirrors
rem the sh path 1:1 but has NEVER been executed on Windows - verify on real
rem Windows hardware before trusting it.
rem After first run: open a NEW terminal. Linux/macOS: use bootstrap.sh.
setlocal EnableExtensions
cd /d "%~dp0"
set "MB=%LOCALAPPDATA%\mise\bin"

echo [bootstrap] step 1/5: ensuring mise ...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $bin='%MB%'; if (-not (Get-Command mise -ErrorAction SilentlyContinue) -and -not (Test-Path (Join-Path $bin 'mise.exe'))) { Write-Host 'installing mise (GitHub release zip)'; $r=Invoke-RestMethod 'https://api.github.com/repos/jdx/mise/releases/latest'; $a=$r.assets | Where-Object { $_.name -match 'windows-x64\.zip$' } | Select-Object -First 1; if (-not $a) { throw 'no windows-x64 asset on latest release' }; $z=Join-Path $env:TEMP $a.name; Invoke-WebRequest -UseBasicParsing $a.browser_download_url -OutFile $z; $x=Join-Path $env:TEMP 'mise-extract'; if (Test-Path $x) { Remove-Item $x -Recurse -Force }; Expand-Archive $z $x; $e=Get-ChildItem -Recurse -Path $x -Filter mise.exe | Select-Object -First 1; if (-not $e) { throw 'mise.exe not found inside archive' }; New-Item -ItemType Directory -Force -Path $bin | Out-Null; Copy-Item $e.FullName (Join-Path $bin 'mise.exe') -Force }"
if errorlevel 1 (
    echo [bootstrap] ERROR: mise install step failed - see message above 1>&2
    exit /b 1
)
set "MISE=mise"
where mise >nul 2>&1 || set "MISE=%MB%\mise.exe"
if not exist "%MISE%" if "%MISE%"=="mise" (
    echo [bootstrap] ERROR: mise not resolvable after install 1>&2
    exit /b 1
)

echo [bootstrap] step 2/5: ensuring user PATH entry ...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$bin='%MB%'; $up=[Environment]::GetEnvironmentVariable('Path','User'); if (($up -split ';') -notcontains $bin) { [Environment]::SetEnvironmentVariable('Path', ($bin + ';' + $up), 'User'); Write-Host 'added %MB% to user PATH (restart terminal to pick up)' } else { Write-Host 'user PATH already contains mise bin' }"

echo [bootstrap] step 3/5: applying mise.toml pins ...
"%MISE%" install
if errorlevel 1 (
    echo [bootstrap] ERROR: mise install failed 1>&2
    exit /b 1
)

echo [bootstrap] step 4/5: ensuring pixi via PolyOrch standard script ...
rem guard: the supplier script must exist before delegation - initialise the
rem submodule if it is not checked out, fail loudly when it is still missing.
if not exist "PolyOrch\scripts\pixi-init.bat" (
    echo [bootstrap] PolyOrch\scripts\pixi-init.bat missing - running git submodule update --init --recursive PolyOrch
    git submodule update --init --recursive PolyOrch
    if errorlevel 1 (
        echo [bootstrap] ERROR: submodule init failed - check network/git, clone PolyOrch manually and retry 1>&2
        exit /b 1
    )
    if not exist "PolyOrch\scripts\pixi-init.bat" (
        echo [bootstrap] ERROR: supplier script still missing after submodule init 1>&2
        exit /b 1
    )
)
call PolyOrch\scripts\pixi-init.bat "%~dp0pixi.toml"
if errorlevel 1 (
    echo [bootstrap] ERROR: pixi bootstrap delegation failed - see supplier message above 1>&2
    exit /b 1
)

echo [bootstrap] step 5/5: repo-local git hooks (C3 pre-flight) ...
git config core.hooksPath scripts/hooks
rem reserved host-policy slot: China cargo mirror (rsproxy) would go here -
rem maintainer network-policy call (D104); deliberately not the supplier job.

echo.
echo [bootstrap] done. Open a NEW terminal, then verify:  node -v  (expect v22.x pin),  pixi --version
