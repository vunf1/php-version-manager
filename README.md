# PHP Version Manager

A cross-platform PHP version manager with a modern GUI interface, built with Rust for memory safety and low resource usage.

## Features

- **Install/Remove PHP versions**: Download and manage multiple PHP versions
- **Switch versions**: Easily switch between installed PHP versions
- **Cross-platform**: Works on Windows and Linux
- **Graphical interface**: Modern, user-friendly GUI built with Tauri and React
- **Memory safe**: Built with Rust for safety and performance
- **Low resource**: Fast startup, minimal memory footprint

## Project Structure

```
php-version-manager/
├── phpvm-core/          # Core library
│   ├── src/
│   │   ├── config.rs    # Configuration management
│   │   ├── download.rs  # Download and caching
│   │   ├── install.rs   # Installation logic
│   │   ├── manager.rs   # Main PHP manager interface
│   │   ├── platform.rs  # OS-specific operations
│   │   ├── state.rs     # State management
│   │   └── version.rs   # Version parsing and handling
│   └── Cargo.toml
├── phpvm-gui/           # Graphical user interface (Tauri + React)
│   ├── src/             # React frontend
│   ├── src-tauri/       # Tauri backend (Rust)
│   └── package.json
├── build.sh             # Build script (Linux/macOS)
├── build.ps1            # Build script (Windows PowerShell)
└── build.bat            # Build script (Windows CMD)
```

## Architecture

The project uses a core library (`phpvm-core`) that contains all the business logic for managing PHP versions. The GUI application depends on this core library, ensuring:

- **No code duplication**: All PHP management logic is in one place
- **Maintainability**: Changes to core logic are centralized

### Core Library (`phpvm-core`)

The core library provides:
- `PhpManager`: Main interface for all PHP operations
- `Config`: Configuration management (JSON-based)
- `PhpState`: State tracking (installed versions, active version)
- `Installer`: Handles download, extraction, and installation
- `Downloader`: Manages downloads with caching and checksum verification
- Platform-specific utilities for PATH management

### GUI (`phpvm-gui`)

Graphical interface built with:
- **Frontend**: React + Vite
- **Backend**: Tauri (Rust)
- **Communication**: Tauri commands that call into `phpvm-core`

## Building

### Development Build (Recommended for Development)

Build in debug mode and automatically open the GUI:

**Windows:**
```powershell
.\build-dev.ps1
```

**Linux/macOS:**
```bash
chmod +x build-dev.sh
./build-dev.sh
```

This compiles in debug mode (faster builds) and opens the application automatically.

### Production Build

### Prerequisites

The build scripts will automatically check for prerequisites and provide installation instructions if anything is missing. See [PREREQUISITES.md](PREREQUISITES.md) for detailed setup instructions.

**Required:**
- **Rust (Cargo)** - For compiling the core library and CLI
- **Node.js and npm** - For building the GUI frontend
- **Platform-specific build tools**:
  - Windows: Visual Studio Build Tools with C++ workload
  - Linux: Build essentials and WebKit/GTK development libraries

### Windows

```powershell
.\build.ps1
```

Or using CMD:
```cmd
build.bat
```

### Linux/macOS

```bash
chmod +x build.sh
./build.sh
```

### Manual Build

#### Build Core Library
```bash
cd phpvm-core
cargo build --release
```

#### Build GUI
```bash
cd phpvm-gui
npm install
npm run tauri build
```

## Usage

### GUI

The GUI compiles to a **native desktop executable** (not a browser application). After building, launch the executable:

- **Windows**: `phpvm-gui\src-tauri\target\release\phpvm-gui.exe`
- **Linux**: `phpvm-gui/src-tauri/target/release/phpvm-gui`
- **macOS**: `phpvm-gui/src-tauri/target/release/bundle/macos/PHP Version Manager.app`

The application will open as a native desktop window. Use the graphical interface to:
- Install PHP versions
- Switch between versions
- Remove versions
- View installed and available versions

## Design Decisions

### Why Rust?

- **Memory safety**: Zero-cost abstractions with compile-time safety
- **Performance**: Low resource usage, fast startup
- **Cross-platform**: Excellent Windows and Linux support
- **Ecosystem**: Great CLI libraries (clap) and GUI framework (Tauri)

### Why Separate Folders?

- **Clear separation**: Core library and GUI are distinct
- **Independent builds**: Can build core without GUI dependencies
- **Maintainability**: Core logic is separated from UI concerns

### Why Tauri for GUI?

- **Lightweight**: Much smaller than Electron
- **Rust backend**: Can directly use `phpvm-core` without FFI
- **Native performance**: Uses system webview
- **Security**: Smaller attack surface than Electron

## Security

- Checksum verification for downloads
- Atomic installs (download → verify → stage → activate)
- Reversible switching with last-known-good fallback
- User-scoped by default (no silent privilege escalation)

## License

MIT
