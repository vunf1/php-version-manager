# PowerShell development build script for Windows
# Builds GUI in debug mode and opens it automatically

$ErrorActionPreference = "Stop"

Write-Host "Development Build - PHP Version Manager GUI" -ForegroundColor Green
Write-Host ""

# Check prerequisites
Write-Host "Checking prerequisites..." -ForegroundColor Yellow

# Check for Cargo (Rust)
$cargoCheck = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargoCheck) {
    Write-Host "ERROR: Cargo (Rust) is not installed or not in PATH!" -ForegroundColor Red
    exit 1
}
$cargoVersion = (cargo --version 2>&1 | Out-String).Trim()
Write-Host "  [OK] Cargo (Rust) found: $cargoVersion" -ForegroundColor Green

# Check for Node.js
$nodeCheck = Get-Command node -ErrorAction SilentlyContinue
if (-not $nodeCheck) {
    Write-Host "ERROR: Node.js is not installed or not in PATH!" -ForegroundColor Red
    exit 1
}
$nodeVersion = (node --version 2>&1 | Out-String).Trim()
Write-Host "  [OK] Node.js found: $nodeVersion" -ForegroundColor Green

# Check for npm
$npmCheck = Get-Command npm -ErrorAction SilentlyContinue
if (-not $npmCheck) {
    Write-Host "ERROR: npm is not installed or not in PATH!" -ForegroundColor Red
    exit 1
}
$npmVersion = (npm --version 2>&1 | Out-String).Trim()
Write-Host "  [OK] npm found: $npmVersion" -ForegroundColor Green
Write-Host ""

# Set environment variables for optimal hot reload development
# IMPORTANT: No optimization flags for dev mode - they slow down compilation
# Hot reload requires fast incremental compilation, not optimized code

# Check for sccache (compiler cache) - optional but recommended
# NOTE: For hot reload, incremental compilation is REQUIRED and takes priority over sccache
# sccache conflicts with incremental compilation, so we disable it in dev mode
$sccachePath = Get-Command sccache -ErrorAction SilentlyContinue
if ($sccachePath) {
    Write-Host "  [INFO] sccache detected but disabled for dev mode (incremental compilation required)" -ForegroundColor Yellow
    Write-Host "         sccache will be used for release builds" -ForegroundColor Yellow
} else {
    Write-Host "  [INFO] sccache not found - install with: cargo install sccache" -ForegroundColor Yellow
    Write-Host "         Note: sccache is disabled in dev mode for hot reload compatibility" -ForegroundColor Yellow
}

# ALWAYS enable incremental compilation for hot reload
# This is REQUIRED for fast rebuilds during development
# sccache is disabled in dev mode because it conflicts with incremental compilation
$env:CARGO_INCREMENTAL = "1"
# Explicitly unset RUSTC_WRAPPER to ensure sccache doesn't interfere
$env:RUSTC_WRAPPER = $null
Write-Host "  [INFO] Incremental compilation enabled for hot reload" -ForegroundColor Green

# Enable faster linking and codegen for dev mode
# More codegen units = faster compilation but slower runtime (perfect for dev mode)
# 16 is a good balance - fast compilation without too much runtime overhead
$env:CARGO_BUILD_RUSTFLAGS = "-C codegen-units=16 -C embed-bitcode=no"

# Enable dev mode optimizations
$env:TAURI_DEV = "true"

# Check for lld (LLVM linker) - faster linking on Windows
$llvmBinPath = "C:\Program Files\LLVM\bin"
$lldPath = Join-Path $llvmBinPath "lld-link.exe"
if (Test-Path $lldPath) {
    # Add LLVM bin to PATH if not already there (for lld-link to be found)
    if ($env:PATH -notlike "*$llvmBinPath*") {
        $env:PATH = "$llvmBinPath;$env:PATH"
    }
    Write-Host "  [INFO] lld-link detected - faster linking enabled" -ForegroundColor Green
} else {
    Write-Host "  [INFO] lld-link not found - install LLVM for faster linking" -ForegroundColor Yellow
}

# Get CPU core count for parallel builds
$cpuCores = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
Write-Host "  [INFO] Using $cpuCores CPU cores for parallel compilation" -ForegroundColor Cyan
Write-Host ""

# Build the core library (only if not already built)
Write-Host "Checking core library..." -ForegroundColor Yellow
Push-Location phpvm-core
$ErrorActionPreference = 'Continue'
cargo check --jobs $cpuCores --quiet 2>&1 | Out-Null
$ErrorActionPreference = 'Stop'
Pop-Location

# Build and run GUI in development mode with hot reload
Write-Host "Building and running GUI (development mode with hot reload)..." -ForegroundColor Yellow
Write-Host "This will:" -ForegroundColor Cyan
Write-Host "  - Compile in debug mode (fast, unoptimized)" -ForegroundColor Cyan
Write-Host "  - Enable hot reload for frontend (instant updates)" -ForegroundColor Cyan
Write-Host "  - Enable watch mode for Rust (auto-recompile on changes)" -ForegroundColor Cyan
Write-Host "  - Open the application automatically" -ForegroundColor Cyan
Write-Host ""

Push-Location phpvm-gui

# Install dependencies if needed
if (-not (Test-Path "node_modules")) {
    Write-Host "Installing npm dependencies..." -ForegroundColor Cyan
    npm install
    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [FAIL] Failed to install dependencies!" -ForegroundColor Red
        Pop-Location
        exit 1
    }
}

# Set additional environment variables for Tauri dev mode hot reload
$env:TAURI_DEV = "1"
$env:RUST_BACKTRACE = "1"  # Better error messages in dev
$env:RUST_LOG = "info"      # Control logging level

# Run Tauri dev mode with watch (builds, opens, and watches for changes)
Write-Host "Starting development build with hot reload..." -ForegroundColor Cyan
Write-Host "Frontend: Vite HMR will update instantly on file changes" -ForegroundColor Gray
Write-Host "Backend: Tauri will watch Rust files and recompile automatically" -ForegroundColor Gray
Write-Host ""
$ErrorActionPreference = 'Continue'
npm run tauri:dev
$ErrorActionPreference = 'Stop'
Pop-Location

Write-Host ""
Write-Host "Development build completed." -ForegroundColor Green
