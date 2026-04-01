# Rebuild Windows shell icon cache so Task Manager / taskbar pick up a new .exe icon after tauri build.
# Safe to run; restarts Explorer (taskbar flashes ~1s). Close unsaved work in Explorer windows first.
$ErrorActionPreference = "Stop"
Write-Host "Stopping Explorer to clear icon cache..." -ForegroundColor Yellow
Stop-Process -Name "explorer" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 2

Remove-Item -Path "$env:LOCALAPPDATA\IconCache.db" -Force -ErrorAction SilentlyContinue
Get-ChildItem "$env:LOCALAPPDATA\Microsoft\Windows\Explorer" -Filter "iconcache*.db" -ErrorAction SilentlyContinue |
    Remove-Item -Force -ErrorAction SilentlyContinue

Write-Host "Starting Explorer..." -ForegroundColor Green
Start-Process "explorer.exe"
Write-Host "Done. Re-open Task Manager and check your app. If still old, sign out or reboot once." -ForegroundColor Cyan
