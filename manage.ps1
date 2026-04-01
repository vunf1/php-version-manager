# PHP Version Manager - management console
# Launches the Node.js Ink TUI: node scripts/manage.mjs [action]
#
# Actions (run TUI with no args, or pass first arg, e.g. .\manage.ps1 dev):
#   dev       Tauri dev + hot reload (build-dev.ps1 / build-dev.sh)
#   build     Release build (build.ps1 / build.sh)
#   test      cargo test --workspace
#   check     cargo check --workspace
#   status    cargo / node / npm
#   deps      npm install in phpvm-gui
#   clean     Remove phpvm-gui/dist, Vite cache, cargo clean --workspace
#   help      Print actions
#
# Usage: .\manage.ps1              (interactive TUI)
#        .\manage.ps1 dev          (run action directly)
#
# Requires: Node.js 18+ (npm install once at repo root for TUI deps)

param([Parameter(Position = 0)][string]$Action = "")

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

if (-not (Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "Node.js not found. Install Node 18+ and ensure it is on PATH." -ForegroundColor Red
    exit 1
}

$manageScript = Join-Path $PSScriptRoot "scripts\manage.mjs"
if (-not (Test-Path $manageScript)) {
    Write-Host "Missing scripts/manage.mjs" -ForegroundColor Red
    exit 1
}

$inkPath = Join-Path $PSScriptRoot "node_modules\ink\package.json"
if (-not (Test-Path $inkPath)) {
    Write-Host "Installing manage console dependencies (first run)..." -ForegroundColor Cyan
    npm install --omit=dev --prefix $PSScriptRoot
    if ($LASTEXITCODE -ne 0) {
        Write-Host "npm install failed." -ForegroundColor Red
        exit 1
    }
}

$nodeArgs = @((Resolve-Path $manageScript).Path)
if ($Action -ne "") {
    $nodeArgs += $Action
}

& node $nodeArgs
exit $LASTEXITCODE
