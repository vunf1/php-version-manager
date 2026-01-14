#!/bin/bash

set -e

echo "Building PHP Version Manager..."
echo ""

# Check prerequisites
echo "Checking prerequisites..."

# Check for Cargo (Rust)
if ! command -v cargo &> /dev/null; then
    echo "ERROR: Cargo (Rust) is not installed or not in PATH!"
    echo ""
    echo "Please install Rust from: https://www.rust-lang.org/tools/install"
    echo "After installation, restart your terminal and try again."
    echo ""
    echo "To verify installation, run: cargo --version"
    exit 1
fi

CARGO_VERSION=$(cargo --version)
echo "  ✓ Cargo (Rust) found: $CARGO_VERSION"

# Check for Node.js
if ! command -v node &> /dev/null; then
    echo "ERROR: Node.js is not installed or not in PATH!"
    echo ""
    echo "Please install Node.js from: https://nodejs.org/"
    echo "After installation, restart your terminal and try again."
    echo ""
    echo "To verify installation, run: node --version"
    exit 1
fi

NODE_VERSION=$(node --version)
echo "  ✓ Node.js found: $NODE_VERSION"

# Check for npm
if ! command -v npm &> /dev/null; then
    echo "ERROR: npm is not installed or not in PATH!"
    echo ""
    echo "npm should come with Node.js. Please reinstall Node.js from: https://nodejs.org/"
    exit 1
fi

NPM_VERSION=$(npm --version)
echo "  ✓ npm found: $NPM_VERSION"
echo ""

# Set environment variables for faster builds
export RUSTFLAGS="-C target-cpu=native"

# Check for sccache (compiler cache) - optional but recommended
if command -v sccache &> /dev/null; then
    export RUSTC_WRAPPER="sccache"
    # sccache provides better caching than incremental compilation
    # Disable incremental when using sccache (sccache handles caching)
    export CARGO_INCREMENTAL=0
    echo "  [INFO] sccache detected - compiler caching enabled"
    echo "         Note: Incremental compilation disabled (sccache provides better caching)"
    echo "         First build will be normal speed, subsequent builds will be much faster"
else
    # Only enable incremental compilation if sccache is not available
    export CARGO_INCREMENTAL=1
    echo "  [INFO] sccache not found - install with: cargo install sccache"
    echo "         This will speed up subsequent builds by caching compiled artifacts"
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

# Build the core library
echo "Building core library..."
cd phpvm-core
if ! cargo build --release --jobs "$CPU_CORES"; then
    echo "  ✗ Core library build failed!"
    exit 1
fi
echo "  ✓ Core library built successfully"
cd ..

# Build the GUI
echo "Building GUI..."

# Ensure environment variables are set for Tauri build (it uses cargo internally)
# These are already set at the top, but ensure they persist
export CARGO_INCREMENTAL=1
export RUSTFLAGS="-C target-cpu=native"

cd phpvm-gui

# Check if node_modules exists, if not, install dependencies
if [ ! -d "node_modules" ]; then
    echo "  Installing npm dependencies..."
    if ! npm install; then
        echo "  ✗ Failed to install npm dependencies!"
        exit 1
    fi
    echo "  ✓ Dependencies installed"
fi

# Build Tauri app (will use cargo with optimizations from .cargo/config.toml and env vars)
echo "  Building Tauri application (this may take a while)..."
echo "  Note: Rust compilation will use $CPU_CORES CPU cores for parallel builds"
if ! npm run tauri:build; then
    echo "  ✗ GUI build failed!"
    echo ""
    echo "  Common issues:"
    echo "  - Make sure you have build essentials installed:"
    echo "    sudo apt-get install build-essential libwebkit2gtk-4.0-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev"
    echo "  - Or on Fedora:"
    echo "    sudo dnf install webkit2gtk3-devel openssl-devel gtk3-devel libappindicator-gtk3 librsvg2-devel"
    exit 1
fi
echo "  ✓ GUI built successfully"
cd ..

echo ""
echo "Build complete!"
echo ""
echo "Output files:"
echo "  GUI: phpvm-gui/src-tauri/target/release/phpvm-gui"
echo ""
