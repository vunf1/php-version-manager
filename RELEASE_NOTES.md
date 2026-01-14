# Release Notes

## Version 0.1.0 - Initial Release

### 🎉 Overview

Initial release of PHP Version Manager, a cross-platform tool for managing multiple PHP versions with a modern graphical interface.

### 📦 Installation

**Windows (64-bit)**

Two installer options are available:

- **MSI Installer**: `PHP Version Manager_0.1.0_x64_en-US.msi`
  - Recommended for enterprise deployments
  - Supports silent installation
  - Integrates with Windows Installer service

- **Setup Executable**: `PHP Version Manager_0.1.0_x64-setup.exe`
  - User-friendly installation wizard
  - Includes automatic dependency checks
  - Standard Windows installer experience

### ✨ Features

#### Core Functionality
- **Install PHP Versions**: Download and install multiple PHP versions from official sources
- **Version Switching**: Easily switch between installed PHP versions
- **Version Management**: View installed versions, remove unused versions, and manage your PHP environment
- **Cache Management**: Efficient download caching with checksum verification

#### User Interface
- **Modern GUI**: Clean, intuitive graphical interface built with Tauri
- **Tabbed Interface**: Organized views for Installed Versions, Available Versions, Cache, and Settings
- **Progress Tracking**: Real-time progress indicators for downloads and installations
- **Notifications**: Clear feedback for all operations and errors

#### Platform Support
- **Windows**: Full support for Windows 10/11 (64-bit)
- **Linux**: Native Linux support (see source build instructions)
- **Cross-Platform**: Consistent experience across supported platforms

### 🛠️ System Requirements

**Windows:**
- Windows 10 or later (64-bit)
- Administrator privileges for installation (optional for user-scoped installs)
- Internet connection for downloading PHP versions

### 🚀 Getting Started

1. **Install the Application**
   - Run the MSI or Setup executable
   - Follow the installation wizard
   - Launch PHP Version Manager from the Start menu

2. **Install Your First PHP Version**
   - Open the "Available Versions" tab
   - Select a PHP version to install
   - Wait for download and installation to complete

3. **Switch PHP Versions**
   - Go to the "Installed Versions" tab
   - Click "Set Active" on your desired version
   - The system PATH will be updated automatically

### 🔒 Security

- **Checksum Verification**: All downloads are verified for integrity
- **Atomic Operations**: Installations are atomic with rollback support
- **User-Scoped**: No silent privilege escalation required
- **Memory Safe**: Built with Rust for security and reliability

### 📝 Notes

- This is the initial release with core functionality
- Supports Windows 64-bit platforms
- Memory-safe implementation using Rust
- Low resource footprint for fast startup and minimal memory usage
- All PHP versions are downloaded from official sources

### 🐛 Known Issues

None at this time. Please report any issues on the project repository.

---

**Release Date**: Initial Release  
**Version**: 0.1.0  
**Platform**: Windows x64  
**License**: MIT
