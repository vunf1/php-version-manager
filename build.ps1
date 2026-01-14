# PowerShell build script for Windows

$ErrorActionPreference = "Stop"

Write-Host "Building PHP Version Manager..." -ForegroundColor Green
Write-Host ""

# Check prerequisites
Write-Host "Checking prerequisites..." -ForegroundColor Yellow

# Check for Cargo (Rust)
$cargoCheck = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargoCheck) {
    Write-Host "ERROR: Cargo (Rust) is not installed or not in PATH!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install Rust from: https://www.rust-lang.org/tools/install" -ForegroundColor Yellow
    Write-Host "After installation, restart your terminal and try again." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To verify installation, run: cargo --version" -ForegroundColor Cyan
    exit 1
}

$cargoVersion = (cargo --version 2>&1 | Out-String).Trim()
Write-Host "  [OK] Cargo (Rust) found: $cargoVersion" -ForegroundColor Green

# Check for Node.js
$nodeCheck = Get-Command node -ErrorAction SilentlyContinue
if (-not $nodeCheck) {
    Write-Host "ERROR: Node.js is not installed or not in PATH!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install Node.js from: https://nodejs.org/" -ForegroundColor Yellow
    Write-Host "After installation, restart your terminal and try again." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "To verify installation, run: node --version" -ForegroundColor Cyan
    exit 1
}

Write-Host "  [OK] Node.js found: $($nodeCheck.Version)" -ForegroundColor Green

# Check for npm
$npmCheck = Get-Command npm -ErrorAction SilentlyContinue
if (-not $npmCheck) {
    Write-Host "ERROR: npm is not installed or not in PATH!" -ForegroundColor Red
    Write-Host ""
    Write-Host "npm should come with Node.js. Please reinstall Node.js from: https://nodejs.org/" -ForegroundColor Yellow
    exit 1
}

$npmVersion = (npm --version 2>&1 | Out-String).Trim()
Write-Host "  [OK] npm found: $npmVersion" -ForegroundColor Green
Write-Host ""

# Set environment variables for faster builds
$env:RUSTFLAGS = "-C target-cpu=native"  # Use CPU-specific optimizations

# Check for sccache (compiler cache) - optional but recommended
$sccachePath = Get-Command sccache -ErrorAction SilentlyContinue
if ($sccachePath) {
    $env:RUSTC_WRAPPER = "sccache"
    # sccache provides better caching than incremental compilation
    # Disable incremental when using sccache (sccache handles caching)
    $env:CARGO_INCREMENTAL = "0"
    Write-Host "  [INFO] sccache detected - compiler caching enabled" -ForegroundColor Green
    Write-Host "         Note: Incremental compilation disabled (sccache provides better caching)" -ForegroundColor Cyan
    Write-Host "         First build will be normal speed, subsequent builds will be much faster" -ForegroundColor Cyan
} else {
    # Only enable incremental compilation if sccache is not available
    $env:CARGO_INCREMENTAL = "1"
    Write-Host "  [INFO] sccache not found - install with: cargo install sccache" -ForegroundColor Yellow
    Write-Host "         This will speed up subsequent builds by caching compiled artifacts" -ForegroundColor Yellow
}

# Check for lld (LLVM linker) - faster linking on Windows
$llvmBinPath = "C:\Program Files\LLVM\bin"
$lldPath = Join-Path $llvmBinPath "lld-link.exe"
if (Test-Path $lldPath) {
    # Add LLVM bin to PATH if not already there (for lld-link to be found)
    if ($env:PATH -notlike "*$llvmBinPath*") {
        $env:PATH = "$llvmBinPath;$env:PATH"
    }
    Write-Host "  [INFO] lld-link detected - faster linking enabled" -ForegroundColor Green
    Write-Host "         Location: $lldPath" -ForegroundColor Cyan
} else {
    Write-Host "  [INFO] lld-link not found - install LLVM for faster linking" -ForegroundColor Yellow
    Write-Host "         Download from: https://github.com/llvm/llvm-project/releases" -ForegroundColor Yellow
}

# Get CPU core count for parallel builds
$cpuCores = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
Write-Host "  [INFO] Using $cpuCores CPU cores for parallel compilation" -ForegroundColor Cyan

# Build the core library
Write-Host "Building core library..." -ForegroundColor Yellow
Push-Location phpvm-core
$ErrorActionPreference = 'Continue'
$output = cargo build --release --jobs $cpuCores 2>&1
$buildSuccess = $LASTEXITCODE -eq 0
$ErrorActionPreference = 'Stop'
if (-not $buildSuccess) {
    Write-Host "  [FAIL] Core library build failed!" -ForegroundColor Red
    Pop-Location
    exit 1
}
Write-Host "  [OK] Core library built successfully" -ForegroundColor Green
Pop-Location

# Build the GUI
Write-Host "Building GUI..." -ForegroundColor Yellow

# Check if phpvm-gui.exe is running and close it if needed
$guiExePath = "phpvm-gui\src-tauri\target\release\phpvm-gui.exe"
if (Test-Path $guiExePath) {
    $guiProcesses = Get-Process | Where-Object {$_.Path -like "*phpvm-gui.exe" -or $_.ProcessName -like "*phpvm*"}
    if ($guiProcesses) {
        Write-Host "  [WARN] PHP Version Manager GUI is running. Attempting to close it..." -ForegroundColor Yellow
        foreach ($proc in $guiProcesses) {
            try {
                Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
                Write-Host "  [OK] Closed process: $($proc.ProcessName) (PID: $($proc.Id))" -ForegroundColor Green
            } catch {
                Write-Host "  [WARN] Could not close process: $($proc.ProcessName)" -ForegroundColor Yellow
            }
        }
        Start-Sleep -Seconds 2
    }
}

Push-Location phpvm-gui

# Ensure environment variables are set for Tauri build (it uses cargo internally)
# RUSTFLAGS is already set at the top
# CARGO_INCREMENTAL is already set/unset based on sccache detection above
# Don't override it here - respect the sccache setting

# Check if node_modules exists, if not, install dependencies
if (-not (Test-Path "node_modules")) {
    Write-Host "  Installing npm dependencies..." -ForegroundColor Cyan
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [FAIL] Failed to install npm dependencies!" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Write-Host "  [OK] Dependencies installed" -ForegroundColor Green
}

# Build Tauri app (will use cargo with optimizations from .cargo/config.toml and env vars)
Write-Host "  Building Tauri application (this may take a while)..." -ForegroundColor Cyan
Write-Host "  Note: Rust compilation will use $cpuCores CPU cores for parallel builds" -ForegroundColor Cyan
$ErrorActionPreference = 'Continue'
$output = npm run tauri:build 2>&1
$buildSuccess = $LASTEXITCODE -eq 0
$ErrorActionPreference = 'Stop'
if (-not $buildSuccess) {
    Write-Host "  [FAIL] GUI build failed!" -ForegroundColor Red
    Write-Host ""
    
    # Check for specific error types
    $outputStr = $output | Out-String
    if ($outputStr -match "Access is denied" -or $outputStr -match "failed to remove file") {
        Write-Host "  TROUBLESHOOTING - File locked error:" -ForegroundColor Yellow
        Write-Host "  1. Close PHP Version Manager GUI if it's running" -ForegroundColor Cyan
        Write-Host "  2. Close any antivirus or Windows Defender that might be scanning the file" -ForegroundColor Cyan
        Write-Host "  3. Try manually deleting: $guiExePath" -ForegroundColor Cyan
        Write-Host "  4. Restart your terminal and try again" -ForegroundColor Cyan
    } else {
        Write-Host "  Common issues:" -ForegroundColor Yellow
        Write-Host "  - Make sure you have Visual Studio Build Tools installed" -ForegroundColor Yellow
        Write-Host "  - Download from: https://visualstudio.microsoft.com/downloads/" -ForegroundColor Yellow
        Write-Host "  - Install 'Desktop development with C++' workload" -ForegroundColor Yellow
    }
    Pop-Location
    exit 1
}
Write-Host "  [OK] GUI built successfully" -ForegroundColor Green
Pop-Location

Write-Host ""
Write-Host "Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Output files:" -ForegroundColor Cyan
Write-Host "  GUI: phpvm-gui\src-tauri\target\release\phpvm-gui.exe" -ForegroundColor White
Write-Host ""
Write-Host "Installers:" -ForegroundColor Cyan
Write-Host "  MSI: phpvm-gui\src-tauri\target\release\bundle\msi\PHP Version Manager_0.1.0_x64_en-US.msi" -ForegroundColor White
Write-Host "  NSIS: phpvm-gui\src-tauri\target\release\bundle\nsis\PHP Version Manager_0.1.0_x64-setup.exe" -ForegroundColor White
Write-Host ""
