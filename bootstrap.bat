@echo off
rem bootstrap.bat - one-time Windows-native dev-environment setup for ModuKit
rem (user-level, no admin). Same three steps as bootstrap.sh:
rem   1. ensure mise   (official GitHub release zip -> %LOCALAPPDATA%\mise\bin)
rem   2. ensure user PATH contains that bin dir (via .NET User-scope API, not setx)
rem   3. mise install  (toolchain pins from mise.toml; no-op when satisfied)
rem After first run: open a NEW terminal. Linux/macOS: use bootstrap.sh.
setlocal EnableExtensions
cd /d "%~dp0"
set "MB=%LOCALAPPDATA%\mise\bin"

echo [bootstrap] step 1/3: ensuring mise ...
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

echo [bootstrap] step 2/3: ensuring user PATH entry ...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$bin='%MB%'; $up=[Environment]::GetEnvironmentVariable('Path','User'); if (($up -split ';') -notcontains $bin) { [Environment]::SetEnvironmentVariable('Path', ($bin + ';' + $up), 'User'); Write-Host 'added %MB% to user PATH (restart terminal to pick up)' } else { Write-Host 'user PATH already contains mise bin' }"

echo [bootstrap] step 3/3: applying mise.toml pins ...
"%MISE%" install
if errorlevel 1 (
    echo [bootstrap] ERROR: mise install failed 1>&2
    exit /b 1
)

echo.
echo [bootstrap] done. Open a NEW terminal, then verify:  node -v  (expect v22.x pin)
