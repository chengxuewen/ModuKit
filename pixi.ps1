# pixi.ps1 -- activate the pixi workspace that lives in THIS directory
# (PowerShell / pwsh, in place).
#
# Installed next to pixi.toml / .pixi\ by polyorch_pixi_init(COPY_SCRIPTS)
# or polyorch_pixi_scripts_install(). The file is managed by PolyOrch; local
# edits are overwritten on the next install.
#
# Usage:   . .\pixi.ps1        (dot-source it; running it activates a child only)
# For a subshell model instead, use pixi.bat (`pixi shell`).
#
# NOTE: authored on Linux, not yet exercised on a real Windows machine
# (best-effort). Requires pixi; if absent, provision it first with
# polyorch_pixi_tool_ensure() (see examples/pixi-bootstrap/).

$dir = Split-Path -Parent $PSCommandPath

$pixi = (Get-Command pixi -ErrorAction SilentlyContinue).Source
if (-not $pixi) {
    $cand = Join-Path $env:USERPROFILE ".pixi\bin\pixi.exe"
    if (Test-Path $cand) { $pixi = $cand }
}
if (-not $pixi) {
    Write-Error "pixi not found -- provision it first (polyorch_pixi_tool_ensure)"
    exit 1
}

$man = Join-Path $dir "pixi.toml"
if (-not (Test-Path $man)) { $man = Join-Path $dir "pyproject.toml" }
if (-not (Test-Path $man)) {
    Write-Error "no pixi.toml or pyproject.toml beside this script: $dir"
    exit 1
}

& $pixi shell-hook --manifest-path $man --shell pwsh | Invoke-Expression
if ($IsMacOS -or $IsLinux) { Remove-Variable -Name dir, pixi, man, cand -ErrorAction SilentlyContinue }
