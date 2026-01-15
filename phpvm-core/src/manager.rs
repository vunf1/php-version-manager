use crate::config;
use crate::install::Installer;
use crate::logging;
use crate::platform;
use crate::provider::Provider;
use crate::state::PhpState;
use crate::version::PhpVersion;
use anyhow::Context;
use std::fs;

pub struct PhpManager {
    installer: Installer,
    config: config::Config,
}

impl PhpManager {
    pub fn new() -> anyhow::Result<Self> {
        // Initialize logging
        let _ = logging::init();
        
        logging::log_message("INFO", "PHP Manager initialized");
        
        Ok(PhpManager {
            installer: Installer::new()?,
            config: config::Config::load()?,
        })
    }

    pub async fn install(&self, version_str: &str, thread_safe: Option<bool>, progress_callback: Option<Box<dyn FnMut(u64, u64, f64) + Send + Sync>>) -> anyhow::Result<()> {
        logging::log_message("INFO", &format!("Starting installation of PHP {}", version_str));
        let version = PhpVersion::from_string(version_str)?;
        match self.installer.install_version(&version, None, thread_safe, progress_callback).await {
            Ok(_) => {
                logging::log_message("INFO", &format!("Successfully installed PHP {}", version_str));
                Ok(())
            }
            Err(e) => {
                logging::log_message("ERROR", &format!("Failed to install PHP {}: {}", version_str, e));
                Err(e)
            }
        }
    }

    pub async fn remove(&self, version_str: &str) -> anyhow::Result<()> {
        // version_str can be "8.5.1-ts" or "8.5.1-nts"
        // Extract base version for PhpVersion parsing (first 3 parts: major.minor.patch)
        let base_version_str = if version_str.contains("-ts") || version_str.contains("-nts") {
            version_str.split('-').take(3).collect::<Vec<_>>().join(".")
        } else {
            version_str.to_string()
        };
        let version = PhpVersion::from_string(&base_version_str)?;
        // Pass the full version string (with variant) to remove_version
        // We'll need to update remove_version signature, but for now, store it in version's suffix
        let mut version_with_variant = version.clone();
        if version_str.contains("-ts") {
            version_with_variant.suffix = Some("ts".to_string());
        } else if version_str.contains("-nts") {
            version_with_variant.suffix = Some("nts".to_string());
        }
        self.installer.remove_version(&version_with_variant).await?;
        Ok(())
    }

    pub async fn switch(&self, version_str: &str) -> anyhow::Result<()> {
        // version_str can be "8.5.1-ts" or "8.5.1-nts" or just "8.5.1" (use first available)
        let state = PhpState::load()?;
        
        // Find the installed version (with variant suffix)
        let installed_version = if state.installed_versions.contains(&version_str.to_string()) {
            version_str.to_string()
        } else {
            // Try to find any variant of this version
            state.installed_versions
                .iter()
                .find(|v| v.starts_with(&format!("{}", version_str.split('-').next().unwrap_or(version_str))))
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("Version {} is not installed", version_str))?
        };

        let version_dir = self
            .config
            .install_dir
            .join(format!("php-{}", installed_version));

        if !version_dir.exists() {
            anyhow::bail!("Version directory does not exist: {:?}", version_dir);
        }

        let php_exe = platform::get_php_executable_path(&version_dir);
        if !php_exe.exists() {
            anyhow::bail!("PHP executable not found: {:?}", php_exe);
        }

        tracing::info!("Switching to PHP {}", installed_version);

        let current_path = platform::get_current_path();
        let current_dir = current_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid current path"))?;
        fs::create_dir_all(current_dir)?;

        #[cfg(target_os = "windows")]
        {
            let current_dir = current_path.parent()
                .ok_or_else(|| anyhow::anyhow!("Invalid current path"))?;
            
            // Create php.exe in current directory for IDE compatibility
            // IDEs (like VS Code) expect php.exe directly in the current directory
            // They check: C:\Users\...\phpvm\current\php.exe
            let php_exe_in_current = current_dir.join("php.exe");
            
            // Remove any existing php.exe and DLLs if they exist
            if php_exe_in_current.exists() {
                let _ = fs::remove_file(&php_exe_in_current);
            }
            
            // Clean up old DLL files from previous version
            if let Ok(entries) = fs::read_dir(current_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(ext) = path.extension() {
                                if ext == "dll" || ext == "DLL" {
                                    let _ = fs::remove_file(&path);
                                }
                            }
                        }
                    }
                }
            }
            
            // Copy php.exe to current directory
            // This is the most reliable method that works without admin privileges
            // and ensures IDEs can find and validate the PHP executable
            fs::copy(&php_exe, &php_exe_in_current)
                .context("Failed to copy php.exe to current directory for IDE compatibility")?;
            
            tracing::info!("Copied php.exe to current directory for IDE compatibility: {:?}", php_exe_in_current);
            
            // Copy all DLL files from the version directory to current directory
            // PHP requires DLLs to be in the same directory or in PATH
            if let Ok(entries) = fs::read_dir(&version_dir) {
                for entry in entries {
                    if let Ok(entry) = entry {
                        let path = entry.path();
                        if path.is_file() {
                            if let Some(file_name) = path.file_name() {
                                if let Some(ext) = path.extension() {
                                    if ext == "dll" || ext == "DLL" {
                                        let dll_in_current = current_dir.join(file_name);
                                        if let Err(e) = fs::copy(&path, &dll_in_current) {
                                            tracing::warn!("Failed to copy DLL {:?} to current directory: {}", path, e);
                                        } else {
                                            tracing::info!("Copied DLL {:?} to current directory", file_name);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Also create php.bat for command-line compatibility (backward compatibility)
            let php_exe_str = php_exe.to_string_lossy().replace("\\", "\\\\");
            let batch_content = format!(
                "@echo off\n\"{}\" %*",
                php_exe_str
            );
            fs::write(&current_path, batch_content)?;
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On Unix, create a symlink or shell script
            if current_path.exists() {
                fs::remove_file(&current_path)?;
            }
            std::os::unix::fs::symlink(&php_exe, &current_path)?;
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&current_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&current_path, perms)?;
        }

        let current_dir_buf = current_dir.to_path_buf();
        platform::add_to_path(&current_dir_buf)
            .context("Failed to add PHP to PATH")?;

        let mut state = PhpState::load()?;
        state.set_active(installed_version.clone());
        state.save()?;

        // Update config (store base version without variant for compatibility)
        let mut config = config::Config::load()?;
        let base_version = installed_version.split('-').next().unwrap_or(&installed_version).to_string();
        config.active_version = Some(base_version);
        config.save()?;

        tracing::info!("Successfully switched to PHP {}", installed_version);
        Ok(())
    }

    pub fn list_installed(&self) -> anyhow::Result<Vec<String>> {
        
        let mut installed = Vec::new();
        
        // First, get versions from state
        let state = PhpState::load()?;
        let state_versions: std::collections::HashSet<String> = state.installed_versions.iter().cloned().collect();
        
        // Then, check filesystem for actual installations
        if self.config.install_dir.exists() {
            let entries = fs::read_dir(&self.config.install_dir)?;
            for entry in entries {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    // Try to parse directory name as PHP version (supports php-8.5.1-ts and php-8.5.1-nts)
                    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                        if dir_name.starts_with("php-") {
                            let version_with_variant = dir_name.strip_prefix("php-").unwrap_or(dir_name);
                            // Check if PHP executable exists in this directory
                            let php_exe = platform::get_php_executable_path(&path);
                            if php_exe.exists() {
                                installed.push(version_with_variant.to_string());
                                logging::log_message("DEBUG", &format!("Found installed version on disk: {}", version_with_variant));
                            } else {
                                logging::log_message("WARN", &format!("Directory {} exists but PHP executable not found, cleaning up", dir_name));
                                // Clean up incomplete installation
                                if let Err(e) = fs::remove_dir_all(&path) {
                                    logging::log_message("ERROR", &format!("Failed to clean up incomplete installation {}: {}", dir_name, e));
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Merge with state versions (in case state has versions not on disk)
        for version in state_versions {
            if !installed.contains(&version) {
                // Check if it's actually installed on disk (version includes variant suffix like "8.5.1-ts")
                let version_dir = self.config.install_dir.join(format!("php-{}", version));
                if version_dir.exists() {
                    let php_exe = platform::get_php_executable_path(&version_dir);
                    if php_exe.exists() {
                        installed.push(version);
                    } else {
                        logging::log_message("WARN", &format!("Version {} in state but PHP executable not found", version));
                    }
                } else {
                    logging::log_message("WARN", &format!("Version {} in state but directory not found", version));
                }
            }
        }
        
        // Sort versions
        installed.sort();
        
        logging::log_message("DEBUG", &format!("Listed {} installed versions", installed.len()));
        Ok(installed)
    }

    pub fn get_active(&self) -> anyhow::Result<Option<String>> {
        let state = PhpState::load()?;
        Ok(state.active_version)
    }
    
    pub fn get_version_metadata(&self, version_str: &str) -> anyhow::Result<Option<crate::state::InstallMetadata>> {
        let state = PhpState::load()?;
        Ok(state.get_metadata(version_str).cloned())
    }

    pub async fn list_available(&self) -> anyhow::Result<Vec<String>> {
        logging::log_message("DEBUG", "Fetching available PHP versions");
        let provider = Provider::new()?;
        // Increased limit to 20 to show all major.minor branches (currently ~12 from 5.6 to 8.5)
        // This ensures all versions from versionlog.com are displayed
        let versions = provider.get_top_versions(20).await?;
        let version_strings: Vec<String> = versions.iter().map(|v| v.version.clone()).collect();
        logging::log_message("DEBUG", &format!("Found {} available versions", version_strings.len()));
        Ok(version_strings)
    }
    
    pub async fn get_version_info(&self, version_str: &str) -> anyhow::Result<Option<crate::provider::VersionInfo>> {
        let provider = Provider::new()?;
        let versions = provider.fetch_available_versions().await?;
        
        // Try to find version in fetched list
        if let Some(mut info) = versions.into_iter().find(|v| v.version == version_str) {
            // Ensure EOL date is populated even if parsing failed
            if info.eol_date.is_none() {
                if let Ok(version) = PhpVersion::from_string(version_str) {
                    info.eol_date = Provider::get_eol_date(version.major, version.minor);
                }
            }
            // Ensure download URL is populated
            if info.download_url.is_none() {
                if let Ok(version) = PhpVersion::from_string(version_str) {
                    info.download_url = Some(Provider::generate_download_url(version_str, version.major, version.minor));
                }
            }
            return Ok(Some(info));
        }
        
        // Version not in fetched list - create VersionInfo with EOL date from get_eol_date
        if let Ok(version) = PhpVersion::from_string(version_str) {
            let eol_date = Provider::get_eol_date(version.major, version.minor);
            let download_url = Some(Provider::generate_download_url(version_str, version.major, version.minor));
            
            Ok(Some(crate::provider::VersionInfo {
                version: version_str.to_string(),
                release_date: None,
                eol_date,
                download_url,
                checksum: None,
            }))
        } else {
            Ok(None)
        }
    }
    
    pub fn is_path_configured(&self) -> anyhow::Result<bool> {
        let current_path = platform::get_current_path();
        let current_dir = current_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid current path"))?;
        platform::is_path_set(&current_dir.to_path_buf())
    }
    
    pub fn ensure_path_set(&self) -> anyhow::Result<()> {
        let current_path = platform::get_current_path();
        let current_dir = current_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid current path"))?;
        platform::add_to_path(&current_dir.to_path_buf())
            .context("Failed to add PHP to PATH")
    }
}
