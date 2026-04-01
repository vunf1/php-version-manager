use crate::config;
use crate::provider::Provider;
use crate::state::{InstallMetadata, PhpState};
use crate::version::PhpVersion;
use crate::download::{hash_url, Downloader};
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

/// Official Windows `php.exe` is much larger; catches truncated downloads or corrupt zips.
#[cfg(target_os = "windows")]
const MIN_PHP_EXECUTABLE_BYTES: u64 = 32 * 1024;
#[cfg(not(target_os = "windows"))]
const MIN_PHP_EXECUTABLE_BYTES: u64 = 32 * 1024;

/// Smoke-test extracted tree: executable present, plausible size, and `php -n -v` succeeds.
pub(crate) fn verify_extracted_php_bundle(install_root: &Path) -> anyhow::Result<()> {
    let exe = crate::platform::get_php_executable_path(&install_root.to_path_buf());
    if !exe.is_file() {
        anyhow::bail!("PHP executable missing after extract: {:?}", exe);
    }
    let len = fs::metadata(&exe)?.len();
    if len < MIN_PHP_EXECUTABLE_BYTES {
        anyhow::bail!(
            "PHP executable looks incomplete ({} bytes; expected at least {}): {:?}",
            len,
            MIN_PHP_EXECUTABLE_BYTES,
            exe
        );
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let output = Command::new(&exe)
            .args(["-n", "-v"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .with_context(|| format!("Failed to run {:?} -n -v", exe))?;
        if !output.status.success() {
            anyhow::bail!(
                "php -n -v failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let out = String::from_utf8_lossy(&output.stdout);
        if !out.contains("PHP") {
            anyhow::bail!("Unexpected output from php -n -v: {}", out.trim());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(&exe)?.permissions().mode();
        if mode & 0o111 == 0 {
            anyhow::bail!("PHP binary is not executable: {:?}", exe);
        }
        let output = Command::new(&exe)
            .args(["-n", "-v"])
            .output()
            .with_context(|| format!("Failed to run {:?} -n -v", exe))?;
        if !output.status.success() {
            anyhow::bail!(
                "php -n -v failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let out = String::from_utf8_lossy(&output.stdout);
        if !out.contains("PHP") {
            anyhow::bail!("Unexpected output from php -n -v: {}", out.trim());
        }
    }

    Ok(())
}

/// Extract a Windows `.zip` or Unix `.tar.gz` PHP archive into `target_dir`.
/// On Windows, a single top-level folder in the zip (e.g. `php-8.x.x/`) is stripped
/// so `php.exe` ends up directly under `target_dir`.
pub(crate) fn extract_php_archive(archive_path: &PathBuf, target_dir: &PathBuf) -> anyhow::Result<()> {
    if !archive_path.exists() {
        anyhow::bail!("Archive file does not exist: {:?}", archive_path);
    }

    let metadata = fs::metadata(archive_path)
        .with_context(|| format!("Failed to read archive metadata: {:?}", archive_path))?;
    if metadata.len() == 0 {
        anyhow::bail!("Archive file is empty: {:?}", archive_path);
    }

    crate::logging::log_message(
        "DEBUG",
        &format!("Archive size: {} bytes", metadata.len()),
    );

    // Always start from an empty tree so files from a failed or manual partial install
    // cannot remain alongside the new extract.
    if target_dir.exists() {
        fs::remove_dir_all(target_dir).with_context(|| {
            format!(
                "Failed to clear target directory before extract: {:?}",
                target_dir
            )
        })?;
    }
    fs::create_dir_all(target_dir)
        .with_context(|| format!("Failed to create target directory: {:?}", target_dir))?;

    #[cfg(target_os = "windows")]
    {
        use zip::ZipArchive;

        crate::logging::log_message("DEBUG", "Opening ZIP archive...");
        let file = fs::File::open(archive_path)
            .with_context(|| format!("Failed to open archive file: {:?}", archive_path))?;

        let mut archive = ZipArchive::new(file).with_context(|| {
            format!(
                "Failed to read ZIP archive (file may be corrupted): {:?}",
                archive_path
            )
        })?;

        let file_count = archive.len();
        crate::logging::log_message(
            "DEBUG",
            &format!("Archive contains {} files", file_count),
        );

        let mut file_names: Vec<String> = Vec::new();

        for i in 0..file_count {
            let file_entry = archive
                .by_index(i)
                .with_context(|| format!("Failed to read file {} from archive", i))?;
            let file_name = file_entry.name().to_string();
            file_names.push(file_name);
        }

        let mut common_prefix: Option<String> = None;
        for file_name in &file_names {
            if let Some(first_slash) = file_name.find('/') {
                let prefix = &file_name[..first_slash + 1];
                match &common_prefix {
                    None => common_prefix = Some(prefix.to_string()),
                    Some(existing) if existing != prefix => {
                        common_prefix = None;
                        break;
                    }
                    _ => {}
                }
            } else {
                common_prefix = None;
                break;
            }
        }

        if let Some(ref prefix) = common_prefix {
            crate::logging::log_message(
                "INFO",
                &format!(
                    "Detected common prefix in ZIP: '{}' - stripping it",
                    prefix
                ),
            );
            crate::logging::log_message(
                "INFO",
                "Files will be extracted directly to target directory",
            );
        } else {
            crate::logging::log_message(
                "DEBUG",
                "No common prefix detected, preserving archive structure",
            );
        }

        for i in 0..file_count {
            let mut file = archive
                .by_index(i)
                .with_context(|| format!("Failed to read file {} from archive", i))?;

            let original_name = &file_names[i];
            let mut file_name = original_name.clone();

            if let Some(ref prefix) = common_prefix {
                if file_name.starts_with(prefix) {
                    file_name = file_name[prefix.len()..].to_string();
                }
            }

            let clean_name = file_name.replace('\\', "/");
            let outpath = target_dir.join(&clean_name);

            if clean_name.ends_with('/') {
                fs::create_dir_all(&outpath)
                    .with_context(|| format!("Failed to create directory: {:?}", outpath))?;
            } else {
                if let Some(p) = outpath.parent() {
                    fs::create_dir_all(p)
                        .with_context(|| format!("Failed to create parent directory: {:?}", p))?;
                }

                crate::logging::log_message(
                    "DEBUG",
                    &format!("Extracting: {} -> {:?}", original_name, outpath),
                );

                let mut outfile = fs::File::create(&outpath)
                    .with_context(|| format!("Failed to create file: {:?}", outpath))?;

                std::io::copy(&mut file, &mut outfile).with_context(|| {
                    format!(
                        "Failed to extract file: {} to {:?}",
                        original_name, outpath
                    )
                })?;
            }
        }

        crate::logging::log_message(
            "INFO",
            &format!("Successfully extracted {} files", file_count),
        );
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

pub struct Installer {
    downloader: Downloader,
    config: config::Config,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheMetadataEntry {
    version: String,
    source_url: String,
    stored_at: String,
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
        let base_semver = format!("{}.{}.{}", version.major, version.minor, version.patch);
        // Log the thread_safe parameter to debug
        crate::logging::log_message("DEBUG", &format!("Install request: version={}, base_semver={}, thread_safe={:?} (is_some={}, is_none={})", 
            version_str, base_semver, thread_safe, thread_safe.is_some(), thread_safe.is_none()));
        
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
        let install_path = self
            .config
            .install_dir
            .join(format!("php-{}-{}", base_semver, variant_suffix));
        
        crate::logging::log_message("DEBUG", &format!("Installing variant: {} (path: {:?})", variant_suffix.to_uppercase(), install_path));

        // Check if this specific variant is already installed (verify it's complete)
        if install_path.exists() {
            let php_exe = crate::platform::get_php_executable_path(&install_path);
            if php_exe.exists() {
                anyhow::bail!("PHP {} ({}) is already installed", base_semver, variant_suffix.to_uppercase());
            } else {
                // Directory exists but installation is incomplete, clean it up
                crate::logging::log_message("WARN", &format!("Found incomplete installation for {} ({}), cleaning up", version_str, variant_suffix));
                if let Err(e) = fs::remove_dir_all(&install_path) {
                    crate::logging::log_message("ERROR", &format!("Failed to clean up incomplete installation: {}", e));
                    anyhow::bail!("Found incomplete installation for {} ({}). Please manually remove {:?} and try again", base_semver, variant_suffix, install_path);
                }
            }
        }

        tracing::info!("Installing PHP {}", base_semver);
        crate::logging::log_message("INFO", &format!("Installing PHP {}", base_semver));

        // Store version with variant suffix for identification (always `X.Y.Z-ts|nts`)
        let version_with_variant = format!("{}-{}", base_semver, variant_suffix);

        let url = source_url.map(|s| s.to_string()).unwrap_or_else(|| {
            let v = PhpVersion::new(version.major, version.minor, version.patch);
            let u = Provider::build_official_windows_php_zip_url(&v, thread_safe_flag);
            if thread_safe_flag {
                crate::logging::log_message("DEBUG", &format!("Building TS URL: {}", u));
                eprintln!("[Installer] Building TS URL: {}", u);
            } else {
                crate::logging::log_message("DEBUG", &format!("Building NTS URL: {}", u));
                eprintln!("[Installer] Building NTS URL: {}", u);
            }
            u
        });
        
        crate::logging::log_message("INFO", &format!("Final download URL: {}", url));
        eprintln!("[Installer] Final download URL: {}", url);
        eprintln!("[Installer] URL contains '-nts': {}", url.contains("-nts"));
        eprintln!("[Installer] URL contains 'x64.zip' (TS): {}", url.ends_with("x64.zip") && !url.contains("-nts"));
        crate::logging::log_message("INFO", &format!("Downloading PHP {} ({})", base_semver, variant_suffix.to_uppercase()));

        let archive_path = self
            .downloader
            .download_file(&url, None, progress_callback)
            .await
            .with_context(|| format!("Failed to download PHP archive from: {}", url))?;
        
        if let Err(e) = self.write_cache_metadata(&hash_url(&url), &version_with_variant, &url) {
            crate::logging::log_message("WARN", &format!("Failed to write cache metadata: {}", e));
        }

        crate::logging::log_message("INFO", &format!("Archive downloaded to: {:?}", archive_path));

        tracing::info!("Extracting archive...");
        crate::logging::log_message("INFO", &format!("Extracting archive from: {:?} to: {:?}", archive_path, install_path));
        
        // Extract archive, and clean up on failure
        let extract_result = extract_php_archive(&archive_path, &install_path)
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

        if let Err(e) = verify_extracted_php_bundle(&install_path) {
            crate::logging::log_message(
                "ERROR",
                &format!("Extracted PHP failed verification: {}", e),
            );
            if install_path.exists() {
                crate::logging::log_message(
                    "INFO",
                    &format!("Cleaning up failed verification at: {:?}", install_path),
                );
                let _ = fs::remove_dir_all(&install_path);
            }
            return Err(e.context("PHP bundle verification failed after extract"));
        }

        crate::logging::log_message(
            "INFO",
            "PHP executable verified (size check and php -n -v)",
        );

        let checksum = self.calculate_checksum(&install_path).await?;

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

    fn write_cache_metadata(
        &self,
        hash: &str,
        version: &str,
        source_url: &str,
    ) -> anyhow::Result<()> {
        Self::write_cache_metadata_at(&self.config.download_cache, hash, version, source_url)
    }

    fn write_cache_metadata_at(
        cache_dir: &Path,
        hash: &str,
        version: &str,
        source_url: &str,
    ) -> anyhow::Result<()> {
        fs::create_dir_all(cache_dir)?;

        let metadata_path = cache_dir.join("cache_metadata.json");
        let mut metadata_map: HashMap<String, CacheMetadataEntry> = if metadata_path.exists() {
            match fs::read_to_string(&metadata_path) {
                Ok(content) => match serde_json::from_str(&content) {
                    Ok(map) => map,
                    Err(e) => {
                        crate::logging::log_message(
                            "WARN",
                            &format!("Invalid cache metadata file, recreating: {}", e),
                        );
                        HashMap::new()
                    }
                },
                Err(_) => HashMap::new(),
            }
        } else {
            HashMap::new()
        };

        let stored_at = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .to_string();

        metadata_map.insert(
            hash.to_string(),
            CacheMetadataEntry {
                version: version.to_string(),
                source_url: source_url.to_string(),
                stored_at,
            },
        );

        let content = serde_json::to_string_pretty(&metadata_map)?;
        fs::write(&metadata_path, content)?;
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_cache_metadata_at_creates_and_updates_file() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path();

        Installer::write_cache_metadata_at(
            cache_dir,
            "hash1",
            "8.2.1-nts",
            "https://example.com/php-8.2.1-nts.zip",
        )
        .unwrap();

        Installer::write_cache_metadata_at(
            cache_dir,
            "hash2",
            "8.2.2-ts",
            "https://example.com/php-8.2.2-ts.zip",
        )
        .unwrap();

        let metadata_path = cache_dir.join("cache_metadata.json");
        let content = fs::read_to_string(metadata_path).unwrap();
        let map: HashMap<String, CacheMetadataEntry> = serde_json::from_str(&content).unwrap();

        let entry1 = map.get("hash1").unwrap();
        assert_eq!(entry1.version, "8.2.1-nts");
        assert_eq!(entry1.source_url, "https://example.com/php-8.2.1-nts.zip");
        assert!(!entry1.stored_at.is_empty());

        let entry2 = map.get("hash2").unwrap();
        assert_eq!(entry2.version, "8.2.2-ts");
        assert_eq!(entry2.source_url, "https://example.com/php-8.2.2-ts.zip");
        assert!(!entry2.stored_at.is_empty());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_extract_windows_zip_strips_common_prefix_and_exposes_php_exe() {
        use std::io::Write;
        use zip::write::{FileOptions, ZipWriter};

        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("fixture.zip");
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts = FileOptions::default();
        zip.start_file("php-8.0.0/php.exe", opts).unwrap();
        zip.write_all(b"MZdummy").unwrap();
        zip.finish().unwrap();

        let out = tmp.path().join("extracted");
        extract_php_archive(&zip_path, &out).unwrap();
        assert!(
            out.join("php.exe").is_file(),
            "expected php.exe at {:?}",
            out.join("php.exe")
        );
    }

    #[cfg(target_os = "windows")]
    fn list_files_recursive_sorted(root: &Path) -> Vec<(String, u64)> {
        let mut out = Vec::new();
        fn walk(cur: &Path, base: &Path, out: &mut Vec<(String, u64)>) {
            let Ok(entries) = fs::read_dir(cur) else {
                return;
            };
            for e in entries.flatten() {
                let p = e.path();
                let rel = p
                    .strip_prefix(base)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                if p.is_dir() {
                    walk(&p, base, out);
                } else if let Ok(m) = e.metadata() {
                    out.push((rel, m.len()));
                }
            }
        }
        walk(root, root, &mut out);
        out.sort();
        out
    }

    /// Leftovers from a partial or manual tree must not survive a new extract.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_extract_clears_stale_files_from_previous_partial_tree() {
        use std::io::Write;
        use zip::write::{FileOptions, ZipWriter};

        let tmp = TempDir::new().unwrap();
        let out = tmp.path().join("target");
        fs::create_dir_all(&out).unwrap();
        fs::write(out.join("stale_from_old_install.txt"), b"junk").unwrap();

        let zip_path = tmp.path().join("bundle.zip");
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts = FileOptions::default();
        zip.start_file("php-8.0.0/php.exe", opts).unwrap();
        zip.write_all(b"MZdummy").unwrap();
        zip.start_file("php-8.0.0/license.txt", opts).unwrap();
        zip.write_all(b"MIT").unwrap();
        zip.finish().unwrap();

        extract_php_archive(&zip_path, &out).unwrap();
        assert!(
            !out.join("stale_from_old_install.txt").exists(),
            "leftover files from a prior tree must be removed before extract"
        );
        assert!(out.join("php.exe").is_file());
        assert!(out.join("license.txt").is_file());
    }

    /// Every non-directory entry in the zip (after prefix strip) must appear on disk with the same size.
    #[cfg(target_os = "windows")]
    #[test]
    fn test_extract_writes_all_zip_files_none_missing() {
        use std::io::Write;
        use zip::write::{FileOptions, ZipWriter};

        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("m.zip");
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let opts = FileOptions::default();
        zip.start_file("php-1.0.0/a.txt", opts).unwrap();
        zip.write_all(b"aaa").unwrap();
        zip.start_file("php-1.0.0/sub/b.txt", opts).unwrap();
        zip.write_all(b"bb").unwrap();
        zip.finish().unwrap();

        let out = tmp.path().join("out");
        extract_php_archive(&zip_path, &out).unwrap();
        let files = list_files_recursive_sorted(&out);
        assert_eq!(files, vec![
            ("a.txt".to_string(), 3),
            ("sub/b.txt".to_string(), 2),
        ]);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_verify_rejects_undersized_php_exe() {
        let tmp = TempDir::new().unwrap();
        let exe = tmp.path().join("php.exe");
        fs::write(&exe, vec![b'x'; 1024]).unwrap();
        assert!(
            verify_extracted_php_bundle(tmp.path()).is_err(),
            "undersized php.exe must fail verification"
        );
    }

    #[cfg(all(not(target_os = "windows"), unix))]
    #[test]
    fn test_verify_rejects_undersized_php_binary_unix() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().unwrap();
        let bin = tmp.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let exe = bin.join("php");
        fs::write(&exe, vec![b'y'; 1024]).unwrap();
        let mut perms = fs::metadata(&exe).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&exe, perms).unwrap();
        assert!(
            verify_extracted_php_bundle(tmp.path()).is_err(),
            "undersized php binary must fail verification"
        );
    }

    /// End-to-end: real zip from windows.php.net, extract, run `php -n -v`.
    #[cfg(target_os = "windows")]
    #[tokio::test]
    #[ignore = "network: cargo test -p phpvm-core smoke_download_extract_verify -- --ignored"]
    async fn smoke_download_extract_verify_php_runs() {
        let tmp = TempDir::new().unwrap();
        let zip_path = tmp.path().join("php.zip");
        let url = crate::provider::Provider::build_official_windows_php_zip_url(
            &crate::version::PhpVersion::new(8, 3, 16),
            true,
        );
        let client = reqwest::Client::builder()
            .user_agent("phpvm-verify-smoke")
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .unwrap();
        let resp = client.get(&url).send().await.unwrap();
        assert!(
            resp.status().is_success(),
            "GET {} -> {}",
            url,
            resp.status()
        );
        let bytes = resp.bytes().await.unwrap();
        fs::write(&zip_path, &bytes).unwrap();

        let out = tmp.path().join("extracted");
        extract_php_archive(&zip_path, &out).unwrap();
        verify_extracted_php_bundle(&out).unwrap();
    }
}
