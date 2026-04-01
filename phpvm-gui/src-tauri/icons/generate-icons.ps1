#Requires -Version 5.1
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot
$script = Join-Path $PSScriptRoot "generate_icons.py"

function Test-PythonWorks {
    param([string]$Command, [string[]]$PrefixArgs = @())
    $cmd = Get-Command $Command -ErrorAction SilentlyContinue
    if (-not $cmd) { return $false }
    & $Command @($PrefixArgs + @("-c", "import struct,zlib")) 2>$null | Out-Null
    return ($LASTEXITCODE -eq 0)
}

if (Test-PythonWorks "python") {
    & python $script
    exit $LASTEXITCODE
}
if (Test-PythonWorks "py" @("-3")) {
    & py -3 $script
    exit $LASTEXITCODE
}

Write-Error "generate-icons.ps1: Python 3 not found (tried 'python' and 'py -3'). Install from https://www.python.org/"
exit 1
