# PHP Version Manager

<div align="center">

**A modern, cross-platform PHP version manager with an intuitive graphical interface**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey)](https://github.com/yourusername/php-version-manager)

Built with Rust for memory safety, performance, and low resource usage

</div>

---

## 🌟 Overview

PHP Version Manager (phpvm) is a cross-platform tool that simplifies managing multiple PHP versions on your system. Whether you're a developer switching between PHP versions for different projects or a system administrator managing PHP installations, phpvm provides a modern, user-friendly solution.

### Key Highlights

- 🎨 **Modern GUI**: Clean, intuitive interface built with Tauri and React
- ⚡ **Fast & Lightweight**: Native performance with minimal resource footprint
- 🔒 **Memory Safe**: Built with Rust for security and reliability
- 🌍 **Cross-Platform**: Works seamlessly on Windows and Linux
- 🔄 **Automatic PATH Management**: Seamless version switching with automatic system PATH updates
- 📦 **Smart Caching**: Efficient download caching with checksum verification
- 🔐 **Secure**: Atomic operations with rollback support and checksum verification

---

## ✨ Features

### Core Functionality

- **📥 Install PHP Versions**: Download and install multiple PHP versions from official sources
- **🔄 Version Switching**: Easily switch between installed PHP versions with a single click
- **🗑️ Version Management**: Remove unused versions to free up disk space
- **👀 Version Discovery**: Browse available PHP versions across different releases
- **📊 Version Status**: View detailed information about installed and active versions

### User Interface

- **📑 Tabbed Interface**: Organized views for Installed Versions, Available Versions, Cache, and Settings
- **📈 Progress Tracking**: Real-time progress indicators for downloads and installations
- **🔔 Notifications**: Clear feedback for all operations, successes, and errors
- **🎨 Modern Design**: Clean, responsive interface with smooth animations

### Platform Features

- **💾 Cache Management**: View and manage downloaded PHP archives with efficient caching
- **🔧 Settings**: Configure installation paths, cache location, and update preferences
- **🔄 Auto-Updates**: Automatic update checking and one-click updates (when available)
- **🔍 Thread-Safe Variants**: Support for both Thread-Safe (TS) and Non-Thread-Safe (NTS) PHP builds on Windows

### Security & Reliability

- **✅ Checksum Verification**: All downloads are verified for integrity
- **🔄 Atomic Operations**: Installations are atomic with rollback support
- **👤 User-Scoped**: No silent privilege escalation required
- **🛡️ Memory Safe**: Built with Rust for enhanced security

---

## 📦 Installation

### Pre-built Packages

#### Windows (64-bit)

Two installer options are available from the [Releases](https://github.com/yourusername/php-version-manager/releases) page:

- **MSI Installer** (`*.msi`)
  - Recommended for enterprise deployments
  - Supports silent installation
  - Integrates with Windows Installer service

- **Setup Executable** (`*-setup.exe`)
  - User-friendly installation wizard
  - Includes automatic dependency checks
  - Standard Windows installer experience

#### Linux

- **AppImage**: Download the `.AppImage` file, make it executable (`chmod +x`), and run it
- **Debian/Ubuntu**: Install the `.deb` package with `sudo dpkg -i *.deb`
- **Fedora/RHEL**: Install the `.rpm` package with `sudo rpm -i *.rpm` or `sudo dnf install *.rpm`

### Build from Source

#### Prerequisites

**Required:**
- **Rust (Cargo)** - Latest stable version ([Install Rust](https://www.rust-lang.org/tools/install))
- **Node.js and npm** - Version 18+ ([Install Node.js](https://nodejs.org/))
- **Platform-specific build tools**:
  - **Windows**: Visual Studio Build Tools with C++ workload
  - **Linux**: Build essentials and WebKit/GTK development libraries

#### Quick Start

**Windows (PowerShell):**
```powershell
# Development build (faster, opens automatically)
.\build-dev.ps1

# Production build
.\build.ps1
```

**Windows (CMD):**
```cmd
build-dev.bat
build.bat
```

**Linux/macOS:**
```bash
# Development build
chmod +x build-dev.sh
./build-dev.sh

# Production build
chmod +x build.sh
./build.sh
```

#### Manual Build

1. **Build Core Library:**
   ```bash
   cd phpvm-core
   cargo build --release
   ```

2. **Build GUI:**
   ```bash
   cd phpvm-gui
   npm install
   npm run tauri build
   ```

3. **Run Application:**
   - **Windows**: `phpvm-gui\src-tauri\target\release\phpvm-gui.exe`
   - **Linux**: `phpvm-gui/src-tauri/target/release/phpvm-gui`
   - **macOS**: `phpvm-gui/src-tauri/target/release/bundle/macos/PHP Version Manager.app`

---

## 🚀 Getting Started

### 1. Install Your First PHP Version

1. Launch PHP Version Manager
2. Open the **"Available Versions"** tab
3. Select a PHP version from the list
4. Click **"Install"** and wait for the download and installation to complete

### 2. Switch PHP Versions

1. Go to the **"Installed Versions"** tab
2. Find your desired version in the list
3. Click **"Set Active"** on the version you want to use
4. The system PATH will be updated automatically

### 3. Manage Versions

- **Remove Versions**: Click **"Remove"** on any installed version to delete it
- **View Cache**: Check the **"Cache"** tab to see downloaded archives
- **Clear Cache**: Remove cached downloads to free up disk space
- **Configure Settings**: Adjust paths and preferences in the **"Settings"** tab

---

## 🏗️ Architecture

### Project Structure

```
php-version-manager/
├── phpvm-core/          # Core library (Rust)
│   ├── src/
│   │   ├── config.rs    # Configuration management
│   │   ├── download.rs  # Download and caching
│   │   ├── install.rs   # Installation logic
│   │   ├── manager.rs   # Main PHP manager interface
│   │   ├── platform.rs  # OS-specific operations (PATH management)
│   │   ├── provider.rs  # PHP version provider/API
│   │   ├── state.rs     # State management
│   │   └── version.rs   # Version parsing and handling
│   └── Cargo.toml
├── phpvm-gui/           # Graphical user interface
│   ├── src/             # React frontend
│   │   ├── components/  # UI components
│   │   ├── hooks/       # React hooks
│   │   ├── services/    # API services
│   │   └── styles/      # CSS stylesheets
│   ├── src-tauri/       # Tauri backend (Rust)
│   │   ├── src/
│   │   │   ├── commands.rs  # Tauri command handlers
│   │   │   ├── main.rs      # Application entry point
│   │   │   └── update.rs    # Auto-update functionality
│   │   └── tauri.conf.json
│   └── package.json
├── build.sh             # Build script (Linux/macOS)
├── build.ps1            # Build script (Windows PowerShell)
└── build.bat            # Build script (Windows CMD)
```

### Design Philosophy

The project uses a **modular architecture** with clear separation between core logic and user interface:

- **`phpvm-core`**: Contains all business logic for managing PHP versions
  - Platform-independent core operations
  - Platform-specific implementations for Windows and Linux
  - No UI dependencies
  
- **`phpvm-gui`**: Provides the graphical user interface
  - React frontend for UI rendering
  - Tauri backend that directly uses `phpvm-core`
  - Communication via Tauri commands

**Benefits:**
- ✅ **No code duplication**: All PHP management logic is centralized
- ✅ **Maintainability**: Changes to core logic are isolated
- ✅ **Testability**: Core library can be tested independently
- ✅ **Future CLI**: Core library can be reused for a future CLI tool

### Core Components

#### `PhpManager`
Main interface for all PHP operations:
- Install/remove PHP versions
- Switch between versions
- List installed and available versions
- Manage cache

#### `Installer`
Handles the complete installation process:
- Download verification
- Archive extraction
- File installation
- PATH management

#### `Downloader`
Manages downloads with:
- Efficient caching
- Checksum verification
- Progress reporting
- Resume support

#### `PhpState`
Tracks application state:
- Installed versions
- Active version
- Configuration settings

---

## 🛠️ System Requirements

### Windows
- **OS**: Windows 10 or later (64-bit)
- **Privileges**: Administrator privileges for installation (optional for user-scoped installs)
- **Network**: Internet connection for downloading PHP versions
- **Disk Space**: ~500MB per PHP version (plus cache space)

### Linux
- **OS**: Linux distribution with GTK 3.0+ and WebKitGTK support
- **Network**: Internet connection for downloading PHP versions
- **Disk Space**: ~500MB per PHP version (plus cache space)

---

## 🎯 Use Cases

- **Web Developers**: Switch between PHP versions for different projects
- **System Administrators**: Manage PHP installations across multiple environments
- **CI/CD**: Use in automated build pipelines
- **Testing**: Test applications against multiple PHP versions
- **Learning**: Experiment with different PHP versions and features

---

## 🔧 Configuration

PHP Version Manager stores configuration in:
- **Windows**: `%APPDATA%\phpvm\config.json`
- **Linux**: `~/.config/phpvm/config.json`

Default installation paths:
- **Windows**: `%LOCALAPPDATA%\phpvm\versions\`
- **Linux**: `~/.local/share/phpvm/versions/`

These can be configured through the Settings tab in the application.

---

## 🐛 Troubleshooting

### Installation Issues

**Problem**: PHP version fails to install
- **Solution**: Check internet connection and available disk space
- **Solution**: Verify checksums are valid (check logs)

**Problem**: PATH not updating after switching versions
- **Solution**: Restart your terminal/command prompt
- **Solution**: On Windows, restart the application with administrator privileges if needed

### Build Issues

**Problem**: Build fails with missing dependencies
- **Solution**: Ensure all prerequisites are installed (see Prerequisites section)
- **Solution**: On Linux, install all required development libraries
- **Solution**: On Windows, ensure Visual Studio Build Tools are installed with C++ workload

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

---

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Built with [Tauri](https://tauri.app/) for the GUI framework
- Powered by [Rust](https://www.rust-lang.org/) for the core logic
- UI built with [React](https://react.dev/) and modern web technologies

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/php-version-manager/issues)
- **Releases**: [GitHub Releases](https://github.com/yourusername/php-version-manager/releases)

---

<div align="center">

**Made with ❤️ using Rust and modern web technologies**

[⬆ Back to Top](#php-version-manager)

</div>
