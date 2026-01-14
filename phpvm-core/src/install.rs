use crate::config;
use crate::state::{InstallMetadata, PhpState};
use crate::version::PhpVersion;
use crate::download::Downloader;
use anyhow::Context;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;

pub struct Installer {
    downloader: Downloader,
    config: config::Config,
}

impl Installer {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Installer {
            downloader: Downloader::new()?,
            config: config::Config::load()?,
        })
    }

    pub async fn install_version(
        &self,
        version: &PhpVersion,
        source_url: Option<&str>,
        thread_safe: Option<bool>,
        progress_callback: Option<Box<dyn FnMut(u64, u64, f64) + Send + Sync>>,
    ) -> anyhow::Result<PathBuf> {
        let version_str = version.to_string();
        // Log the thread_safe parameter to debug
        crate::logging::log_message("DEBUG", &format!("Install request: version={}, thread_safe={:?} (is_some={}, is_none={})", 
            version_str, thread_safe, thread_safe.is_some(), thread_safe.is_none()));
        
        // CRITICAL: Handle the thread_safe parameter correctly
        // If it's Some(false), we want NTS
        // If it's Some(true), we want TS
        // If it's None, default to TS
        let thread_safe_flag = match thread_safe {
            Some(false) => {
                crate::logging::log_message("INFO", "✓ thread_safe is Some(false) - INSTALLING NTS");
                eprintln!("[Installer] ✓ Confirmed: Installing NTS variant (thread_safe=false)");
                false
            },
            Some(true) => {
                crate::logging::log_message("INFO", "✓ thread_safe is Some(true) - INSTALLING TS");
                eprintln!("[Installer] ✓ Confirmed: Installing TS variant (thread_safe=true)");
                true
            },
            None => {
                crate::logging::log_message("WARN", "⚠ thread_safe is None - DEFAULTING TO TS (this might be a bug!)");
                eprintln!("[Installer] ⚠ WARNING: thread_safe is None - defaulting to TS. If user selected NTS, this is a BUG!");
                true
            }
        };
        
        let variant_suffix = if thread_safe_flag { "ts" } else { "nts" };
        let install_path = self.config.install_dir.join(format!("php-{}-{}", version_str, variant_suffix));
        
        crate::logging::log_message("DEBUG", &format!("Installing variant: {} (path: {:?})", variant_suffix.to_uppercase(), install_path));

        // Check if this specific variant is already installed (verify it's complete)
        if install_path.exists() {
            let php_exe = crate::platform::get_php_executable_path(&install_path);
            if php_exe.exists() {
                anyhow::bail!("PHP {} ({}) is already installed", version_str, variant_suffix.to_uppercase());
            } else {
                // Directory exists but installation is incomplete, clean it up
                crate::logging::log_message("WARN", &format!("Found incomplete installation for {} ({}), cleaning up", version_str, variant_suffix));
                if let Err(e) = fs::remove_dir_all(&install_path) {
                    crate::logging::log_message("ERROR", &format!("Failed to clean up incomplete installation: {}", e));
                    anyhow::bail!("Found incomplete installation for {} ({}). Please manually remove {:?} and try again", version_str, variant_suffix, install_path);
                }
            }
        }

        tracing::info!("Installing PHP {}", version_str);
        crate::logging::log_message("INFO", &format!("Installing PHP {}", version_str));

        let url = source_url.map(|s| s.to_string()).unwrap_or_else(|| {
            // Determine Visual Studio/Visual C++ version based on PHP version
            // PHP 8.4+ uses vs17 (Visual Studio 2017)
            // PHP 8.0-8.3 uses vs16 (Visual Studio 2016)
            // PHP 7.x uses vc15 (Visual C++ 2015)
            let vs_version = if version.major > 8 || (version.major == 8 && version.minor >= 4) {
                "vs17"
            } else if version.major == 8 {
                "vs16"
            } else if version.major == 7 {
                "vc15"
            } else {
                // For PHP 5.x and earlier, try vc15 as fallback
                "vc15"
            };
            
            // Use thread_safe_flag to determine TS or NTS build
            // CORRECT URL FORMAT:
            // TS:  php-{version}-Win32-{vs}-x64.zip
            // NTS: php-{version}-nts-Win32-{vs}-x64.zip (nts comes AFTER version, BEFORE Win32)
            let url = if thread_safe_flag {
                let u = format!("https://windows.php.net/downloads/releases/php-{}-Win32-{}-x64.zip", version_str, vs_version);
                crate::logging::log_message("DEBUG", &format!("Building TS URL (thread_safe_flag=true): {}", u));
                eprintln!("[Installer] Building TS URL: {}", u);
                u
            } else {
                let u = format!("https://windows.php.net/downloads/releases/php-{}-nts-Win32-{}-x64.zip", version_str, vs_version);
                crate::logging::log_message("DEBUG", &format!("Building NTS URL (thread_safe_flag=false): {}", u));
                eprintln!("[Installer] Building NTS URL: {}", u);
                u
            };
            url
        });
        
        crate::logging::log_message("INFO", &format!("Final download URL: {}", url));
        eprintln!("[Installer] Final download URL: {}", url);
        eprintln!("[Installer] URL contains '-nts': {}", url.contains("-nts"));
        eprintln!("[Installer] URL contains 'x64.zip' (TS): {}", url.ends_with("x64.zip") && !url.contains("-nts"));
        crate::logging::log_message("INFO", &format!("Downloading PHP {} ({})", version_str, variant_suffix.to_uppercase()));

        let archive_path = self
            .downloader
            .download_file(&url, None, progress_callback)
            .await
            .with_context(|| format!("Failed to download PHP archive from: {}", url))?;
        
        crate::logging::log_message("INFO", &format!("Archive downloaded to: {:?}", archive_path));

        tracing::info!("Extracting archive...");
        crate::logging::log_message("INFO", &format!("Extracting archive from: {:?} to: {:?}", archive_path, install_path));
        
        // Extract archive, and clean up on failure
        let extract_result = self.extract_archive(&archive_path, &install_path)
            .with_context(|| format!("Failed to extract archive from {:?} to {:?}", archive_path, install_path));
        
        if let Err(ref e) = extract_result {
            crate::logging::log_message("ERROR", &format!("Extraction failed: {}", e));
            // Clean up partial installation
            if install_path.exists() {
                crate::logging::log_message("INFO", &format!("Cleaning up partial installation at: {:?}", install_path));
                let _ = fs::remove_dir_all(&install_path);
            }
        }
        
        extract_result?;
        
        // Verify installation is complete by checking for PHP executable
        let php_exe = crate::platform::get_php_executable_path(&install_path);
        if !php_exe.exists() {
            crate::logging::log_message("ERROR", &format!("PHP executable not found after extraction: {:?}", php_exe));
            // Clean up incomplete installation
            if install_path.exists() {
                crate::logging::log_message("INFO", &format!("Cleaning up incomplete installation at: {:?}", install_path));
                let _ = fs::remove_dir_all(&install_path);
            }
            anyhow::bail!("Installation incomplete: PHP executable not found at {:?}", php_exe);
        }
        
        crate::logging::log_message("INFO", &format!("PHP executable found at: {:?}", php_exe));

        let checksum = self.calculate_checksum(&install_path).await?;

        // Store version with variant suffix for identification
        let version_with_variant = format!("{}-{}", version_str, variant_suffix);
        
        let metadata = InstallMetadata {
            version: version_with_variant.clone(),
            install_path: install_path.clone(),
            installed_at: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                .to_string(),
            checksum: Some(checksum),
            source: url.to_string(),
        };

        let mut state = PhpState::load()?;
        state.add_version(version_with_variant.clone(), metadata);
        state.save()?;

        tracing::info!("Successfully installed PHP {}", version_with_variant);
        Ok(install_path)
    }

    pub async fn remove_version(&self, version: &PhpVersion) -> anyhow::Result<()> {
        let version_str = version.to_string();
        
        // Check if version string includes variant suffix (e.g., "8.5.1-ts")
        let (base_version_str, variant_suffix) = if version_str.contains("-ts") || version_str.contains("-nts") {
            let parts: Vec<&str> = version_str.split('-').collect();
            if parts.len() >= 4 {
                // Format: "8.5.1-ts" or "8.5.1-nts"
                let base = parts[0..3].join(".");
                let variant = parts[3];
                (base, Some(variant))
            } else {
                (version_str.clone(), None)
            }
        } else {
            (version_str.clone(), None)
        };
        
        // Determine install path based on variant
        let install_path = if let Some(variant) = variant_suffix {
            self.config.install_dir.join(format!("php-{}-{}", base_version_str, variant))
        } else {
            // Try to find which variant exists, or use old format
            let ts_path = self.config.install_dir.join(format!("php-{}-ts", base_version_str));
            let nts_path = self.config.install_dir.join(format!("php-{}-nts", base_version_str));
            if ts_path.exists() {
                ts_path
            } else if nts_path.exists() {
                nts_path
            } else {
                // Fallback to old format for backward compatibility
                self.config.install_dir.join(format!("php-{}", base_version_str))
            }
        };
        
        let version_with_variant = if let Some(variant) = variant_suffix {
            format!("{}-{}", base_version_str, variant)
        } else {
            // Try to detect from path
            let path_str = install_path.to_string_lossy();
            if path_str.contains("-ts") {
                format!("{}-ts", base_version_str)
            } else if path_str.contains("-nts") {
                format!("{}-nts", base_version_str)
            } else {
                base_version_str.clone()
            }
        };

        if !install_path.exists() {
            anyhow::bail!("Version {} is not installed", version_with_variant);
        }

        let mut state = PhpState::load()?;
        // Check if this specific variant is active
        if state.active_version.as_deref() == Some(&version_with_variant) {
            anyhow::bail!("Cannot remove active version. Switch to another version first.");
        }

        let variant_display = variant_suffix.unwrap_or("");
        tracing::info!("Removing PHP {} ({})", base_version_str, variant_display);
        fs::remove_dir_all(&install_path)
            .context("Failed to remove installation directory")?;

        state.remove_version(&version_with_variant);
        state.save()?;

        tracing::info!("Successfully removed PHP {} ({})", base_version_str, variant_display);
        Ok(())
    }

    fn extract_archive(&self, archive_path: &PathBuf, target_dir: &PathBuf) -> anyhow::Result<()> {
        // Verify archive exists
        if !archive_path.exists() {
            anyhow::bail!("Archive file does not exist: {:?}", archive_path);
        }
        
        // Check archive size
        let metadata = fs::metadata(archive_path)
            .with_context(|| format!("Failed to read archive metadata: {:?}", archive_path))?;
        if metadata.len() == 0 {
            anyhow::bail!("Archive file is empty: {:?}", archive_path);
        }
        
        crate::logging::log_message("DEBUG", &format!("Archive size: {} bytes", metadata.len()));
        
        fs::create_dir_all(target_dir)
            .with_context(|| format!("Failed to create target directory: {:?}", target_dir))?;

        #[cfg(target_os = "windows")]
        {
            use zip::ZipArchive;

            crate::logging::log_message("DEBUG", "Opening ZIP archive...");
            let file = fs::File::open(archive_path)
                .with_context(|| format!("Failed to open archive file: {:?}", archive_path))?;
            
            let mut archive = ZipArchive::new(file)
                .with_context(|| format!("Failed to read ZIP archive (file may be corrupted): {:?}", archive_path))?;
            
            let file_count = archive.len();
            crate::logging::log_message("DEBUG", &format!("Archive contains {} files", file_count));

            for i in 0..file_count {
                let mut file = archive.by_index(i)
                    .with_context(|| format!("Failed to read file {} from archive", i))?;
                
                let file_name = file.name().to_string();
                let outpath = target_dir.join(file.mangled_name());

                if file.name().ends_with('/') {
                    fs::create_dir_all(&outpath)
                        .with_context(|| format!("Failed to create directory: {:?}", outpath))?;
                } else {
                    if let Some(p) = outpath.parent() {
                        fs::create_dir_all(p)
                            .with_context(|| format!("Failed to create parent directory: {:?}", p))?;
                    }
                    
                    crate::logging::log_message("DEBUG", &format!("Extracting: {} -> {:?}", file_name, outpath));
                    
                    let mut outfile = fs::File::create(&outpath)
                        .with_context(|| format!("Failed to create file: {:?}", outpath))?;
                    
                    std::io::copy(&mut file, &mut outfile)
                        .with_context(|| format!("Failed to extract file: {} to {:?}", file_name, outpath))?;
                }
            }
            
            crate::logging::log_message("INFO", &format!("Successfully extracted {} files", file_count));
        }

        #[cfg(not(target_os = "windows"))]
        {
            use flate2::read::GzDecoder;
            use tar::Archive;

            let file = fs::File::open(archive_path)?;
            let tar = GzDecoder::new(file);
            let mut archive = Archive::new(tar);
            archive.unpack(target_dir)?;
        }

        Ok(())
    }

    async fn calculate_checksum(&self, path: &PathBuf) -> anyhow::Result<String> {
        use sha2::{Digest, Sha256};
        use std::io::Read;

        let mut hasher = Sha256::new();
        let mut buffer = vec![0u8; 8192];

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let mut file = fs::File::open(&path)?;
                loop {
                    let n = file.read(&mut buffer)?;
                    if n == 0 {
                        break;
                    }
                    hasher.update(&buffer[..n]);
                }
            }
        }

        Ok(hex::encode(hasher.finalize()))
    }
}
