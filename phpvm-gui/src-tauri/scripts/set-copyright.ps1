# PowerShell script to set copyright metadata in Windows executable
# This is called as a post-build step to avoid VERSION resource conflicts with Tauri

$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path
# Path from scripts folder: ..\..\..\..\target\release\phpvm-gui.exe
# scripts -> src-tauri -> phpvm-gui -> workspace root -> target
$workspaceRoot = Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $scriptPath))
$exePath = Join-Path $workspaceRoot "target\release\phpvm-gui.exe"

if (-not (Test-Path $exePath)) {
    Write-Host "Executable not found, skipping copyright metadata update." -ForegroundColor Yellow
    Write-Host "Expected path: $exePath" -ForegroundColor Gray
    exit 0
}

# Use Unicode escape for copyleft symbol (U+1F12F)
$copyrightText = "Copyleft " + [char]::ConvertFromUtf32(0x1F12F) + " JMSIT.cloud"

# Try to use rcedit via node_modules
$nodeModulesRcedit = Join-Path $scriptPath "..\node_modules\.bin\rcedit.cmd"
if (Test-Path $nodeModulesRcedit) {
    $result = & $nodeModulesRcedit $exePath --set-version-string LegalCopyright $copyrightText 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Copyright metadata set successfully: $copyrightText" -ForegroundColor Green
        exit 0
    } else {
        Write-Host "Warning: rcedit failed to set copyright metadata." -ForegroundColor Yellow
    }
}

# Fallback: Check if Resource Hacker is available
$resourceHacker = Get-Command "ResourceHacker.exe" -ErrorAction SilentlyContinue
if ($resourceHacker) {
    Write-Host "Using Resource Hacker to set copyright metadata..." -ForegroundColor Cyan
    
    # Create RC file content
    $rcLines = @(
        "1 VERSIONINFO",
        "FILEVERSION     0,1,0,0",
        "PRODUCTVERSION  0,1,0,0",
        "FILEOS          0x00040004L",
        "FILETYPE        0x00000001L",
        "BEGIN",
        "  BLOCK `"StringFileInfo`"",
        "  BEGIN",
        "    BLOCK `"040904B0`"",
        "    BEGIN",
        "      VALUE `"LegalCopyright`", `"$copyrightText`"",
        "    END",
        "  END",
        "  BLOCK `"VarFileInfo`"",
        "  BEGIN",
        "    VALUE `"Translation`", 0x0409, 1200",
        "  END",
        "END"
    )
    
    $tempRc = [System.IO.Path]::GetTempFileName() + ".rc"
    $rcLines | Out-File -FilePath $tempRc -Encoding ASCII
    
    & ResourceHacker.exe -open $exePath -save $exePath -action modify -res $tempRc
    Remove-Item $tempRc -ErrorAction SilentlyContinue
    Write-Host "Copyright metadata set successfully: $copyrightText" -ForegroundColor Green
} else {
    Write-Host "Warning: Could not set copyright metadata automatically." -ForegroundColor Yellow
    Write-Host "To enable this feature:" -ForegroundColor Yellow
    Write-Host "  1. Install Resource Hacker from: http://www.angusj.com/resourcehacker/" -ForegroundColor Cyan
    Write-Host "  2. Or manually set copyright using Resource Hacker GUI" -ForegroundColor Cyan
    Write-Host "  Expected copyright value: $copyrightText" -ForegroundColor Gray
}
