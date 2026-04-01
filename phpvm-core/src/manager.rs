use crate::config;
use crate::install::Installer;
use crate::logging;
use crate::platform;
use crate::provider::Provider;
use crate::state::PhpState;
use crate::version::PhpVersion;
use anyhow::Context;
use std::fs;
use std::collections::HashMap;
use std::path::Path;

/// Resolve `version_str` to an installed version id (`X.Y.Z-ts` / `X.Y.Z-nts`).
/// Returns an error if both TS and NTS are installed for the same semver but the request is ambiguous (e.g. `8.2.0` only).
pub(crate) fn resolve_switch_target(
    installed_versions: &[String],
    version_str: &str,
) -> anyhow::Result<String> {
    if installed_versions.iter().any(|v| v == version_str) {
        return Ok(version_str.to_string());
    }
    let base = version_str.split('-').next().unwrap_or(version_str);
    let prefix = format!("{}-", base);
    let candidates: Vec<String> = installed_versions
        .iter()
        .filter(|v| v.as_str() == base || v.starts_with(&prefix))
        .cloned()
        .collect();
    match candidates.len() {
        0 => Err(anyhow::anyhow!("Version {} is not installed", version_str)),
        1 => Ok(candidates.into_iter().next().expect("one candidate")),
        _ => Err(anyhow::anyhow!(
            "Multiple variants installed for {} ({}). Specify {}-ts or {}-nts explicitly.",
            base,
            candidates.join(", "),
            base,
            base
        )),
    }
}

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
        let installed_version = resolve_switch_target(&state.installed_versions, version_str)?;

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
            // Replace the current directory with a full copy of the version directory.
            // This ensures the active version is fully available at:
            // C:\Users\...\AppData\Local\phpvm\current\
            if current_dir.exists() {
                fs::remove_dir_all(current_dir)
                    .context("Failed to remove existing current directory")?;
            }
            fs::create_dir_all(current_dir)
                .context("Failed to create current directory")?;

            copy_dir_recursive(&version_dir, current_dir)
                .context("Failed to copy PHP version into current directory")?;

            verify_dir_mirror(&version_dir, current_dir)
                .context("Current directory does not perfectly mirror the version directory")?;

            // Verify the active copy contains php.exe for IDE compatibility.
            let php_exe_in_current = current_dir.join("php.exe");
            if !php_exe_in_current.exists() {
                anyhow::bail!("php.exe not found in current directory after copy: {:?}", php_exe_in_current);
            }
            
            tracing::info!("Copied full PHP version to current directory: {:?}", current_dir);
            
            // Also create php.bat for command-line compatibility (backward compatibility)
            let php_exe_str = php_exe_in_current.to_string_lossy().replace("\\", "\\\\");
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

fn copy_dir_recursive(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if !src.is_dir() {
        anyhow::bail!("Source path is not a directory: {:?}", src);
    }
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let target = dst.join(entry.file_name());
        if path.is_dir() {
            copy_dir_recursive(&path, &target)?;
        } else {
            fs::copy(&path, &target)
                .with_context(|| format!("Failed to copy file {:?} to {:?}", path, target))?;
        }
    }
    Ok(())
}

fn verify_dir_mirror(src: &Path, dst: &Path) -> anyhow::Result<()> {
    if !src.is_dir() || !dst.is_dir() {
        anyhow::bail!("Both source and destination must be directories");
    }

    let src_map = build_file_map(src)?;
    let dst_map = build_file_map(dst)?;

    if src_map.len() != dst_map.len() {
        anyhow::bail!(
            "File count mismatch: source has {}, destination has {}",
            src_map.len(),
            dst_map.len()
        );
    }

    for (rel_path, src_meta) in src_map {
        match dst_map.get(&rel_path) {
            Some(dst_meta) => {
                if src_meta.is_dir != dst_meta.is_dir {
                    anyhow::bail!("Type mismatch for {}", rel_path);
                }
                if !src_meta.is_dir && src_meta.size != dst_meta.size {
                    anyhow::bail!(
                        "Size mismatch for {} (source {}, destination {})",
                        rel_path,
                        src_meta.size,
                        dst_meta.size
                    );
                }
            }
            None => {
                anyhow::bail!("Missing in destination: {}", rel_path);
            }
        }
    }

    Ok(())
}

#[derive(Clone, Debug)]
struct FileMeta {
    is_dir: bool,
    size: u64,
}

fn build_file_map(root: &Path) -> anyhow::Result<HashMap<String, FileMeta>> {
    let mut map = HashMap::new();
    walk_dir(root, root, &mut map)?;
    Ok(map)
}

fn walk_dir(root: &Path, current: &Path, map: &mut HashMap<String, FileMeta>) -> anyhow::Result<()> {
    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        let metadata = entry.metadata()?;
        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };
        map.insert(rel, FileMeta { is_dir, size });
        if is_dir {
            walk_dir(root, &path, map)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod resolve_switch_tests {
    use super::resolve_switch_target;

    #[test]
    fn exact_match_wins() {
        let installed = vec!["8.2.0-nts".to_string(), "8.2.0-ts".to_string()];
        assert_eq!(
            resolve_switch_target(&installed, "8.2.0-nts").unwrap(),
            "8.2.0-nts"
        );
    }

    #[test]
    fn single_variant_by_base_semver() {
        let installed = vec!["8.2.0-nts".to_string()];
        assert_eq!(
            resolve_switch_target(&installed, "8.2.0").unwrap(),
            "8.2.0-nts"
        );
    }

    #[test]
    fn ambiguous_requires_explicit_variant() {
        let installed = vec!["8.2.0-nts".to_string(), "8.2.0-ts".to_string()];
        let err = resolve_switch_target(&installed, "8.2.0").unwrap_err();
        assert!(
            err.to_string().contains("Multiple variants installed"),
            "{}",
            err
        );
    }

    #[test]
    fn legacy_unsuffixed_directory_name() {
        let installed = vec!["8.2.0".to_string()];
        assert_eq!(
            resolve_switch_target(&installed, "8.2.0").unwrap(),
            "8.2.0"
        );
    }
}
