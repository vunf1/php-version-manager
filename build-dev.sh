#!/bin/bash

# Development build script for Linux/macOS
# Builds GUI in debug mode and opens it automatically

set -e

echo "Development Build - PHP Version Manager GUI"
echo ""

# Check prerequisites
echo "Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo "ERROR: Cargo (Rust) is not installed or not in PATH!"
    exit 1
fi

if ! command -v node &> /dev/null; then
    echo "ERROR: Node.js is not installed or not in PATH!"
    exit 1
fi

if ! command -v npm &> /dev/null; then
    echo "ERROR: npm is not installed or not in PATH!"
    exit 1
fi

echo "  ✓ All prerequisites found"
echo ""

# Set environment variables for faster dev builds
export RUSTFLAGS="-C target-cpu=native"

# Check for sccache (compiler cache) - optional but recommended
if command -v sccache &> /dev/null; then
    export RUSTC_WRAPPER="sccache"
    # sccache provides better caching than incremental compilation
    # Disable incremental when using sccache (sccache handles caching)
    export CARGO_INCREMENTAL=0
    echo "  [INFO] sccache detected - compiler caching enabled"
    echo "         Note: Incremental compilation disabled (sccache provides better caching)"
else
    # Only enable incremental compilation if sccache is not available
    export CARGO_INCREMENTAL=1
    echo "  [INFO] sccache not found - install with: cargo install sccache"
fi

# Check for lld (LLVM linker) - faster linking on Linux
if command -v ld.lld &> /dev/null || command -v lld &> /dev/null; then
    echo "  [INFO] lld detected - faster linking enabled"
else
    echo "  [INFO] lld not found - install LLVM for faster linking"
fi

# Get CPU core count for parallel builds
CPU_CORES=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo "4")
echo "  [INFO] Using $CPU_CORES CPU cores for parallel compilation"
echo ""

# Check core library
echo "Checking core library..."
cd phpvm-core
cargo check --jobs "$CPU_CORES" --quiet || true
cd ..

# Build and run GUI in development mode
echo "Building and running GUI (development mode)..."
echo "This will compile in debug mode and open the application automatically."
echo ""

cd phpvm-gui

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "Installing npm dependencies..."
    npm install
fi

# Run Tauri dev mode (builds and opens automatically)
echo "Starting development build..."
npm run tauri:dev

cd ..

echo ""
echo "Development build completed."
