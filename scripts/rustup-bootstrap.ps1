# Dot-source from repo-root PowerShell build scripts.
# Probes cargo via cmd (avoids ErrorAction Stop on rustup stderr) and can fix missing default toolchain.

function Get-CargoVersionProbe {
    $cargoOut = cmd /c "cargo --version 2>nul" 2>&1
    $code = $LASTEXITCODE
    $line = if ($null -eq $cargoOut) { "" } else { "$cargoOut".Trim() }
    return @{ Code = $code; Line = $line }
}

function Ensure-CargoUsable {
    $p = Get-CargoVersionProbe
    if ($p.Code -eq 0 -and $p.Line) {
        return $p
    }

    $rustup = Get-Command rustup -ErrorAction SilentlyContinue
    if (-not $rustup) {
        return $p
    }

    Write-Host "Rust toolchain: no working default. Running rustup (install stable, set default). This can take a few minutes." -ForegroundColor Cyan
    cmd /c "rustup toolchain install stable"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Note: rustup toolchain install stable exited with code $LASTEXITCODE (continuing to set default)." -ForegroundColor Yellow
    }
    cmd /c "rustup default stable"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "rustup default stable failed (exit $LASTEXITCODE)." -ForegroundColor Red
    }
    return (Get-CargoVersionProbe)
}
