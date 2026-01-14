use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

const GITHUB_REPO: &str = "vunf1/php-version-manager";
const GITHUB_API_BASE: &str = "https://api.github.com/repos";

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub download_url: Option<String>,
    pub release_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Get the current application version from Cargo.toml
pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Check if an asset name matches the application name patterns
fn matches_app_name(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    name_lower.contains("php.version.manager") ||
    name_lower.contains("php version manager") ||
    name_lower.contains("php-version-manager") ||
    name_lower.contains("phpvm") ||
    name_lower.contains("php_version_manager")
}

/// Detect Windows asset (prefers .exe NSIS installer, falls back to .msi)
#[cfg(windows)]
fn detect_platform_asset(assets: &[GitHubAsset]) -> Option<String> {
    // First try to find NSIS installer (.exe with setup or x64-setup in name)
    assets.iter()
        .find(|asset| {
            let name = asset.name.to_lowercase();
            name.ends_with(".exe") && 
            (name.contains("setup") || name.contains("x64-setup")) &&
            matches_app_name(&asset.name)
        })
        .map(|asset| asset.browser_download_url.clone())
        .or_else(|| {
            // Fallback to MSI installer
            assets.iter()
                .find(|asset| {
                    asset.name.ends_with(".msi") && matches_app_name(&asset.name)
                })
                .map(|asset| asset.browser_download_url.clone())
        })
        .or_else(|| {
            // Last resort: any .exe file
            assets.iter()
                .find(|asset| asset.name.ends_with(".exe") && matches_app_name(&asset.name))
                .map(|asset| asset.browser_download_url.clone())
        })
}

/// Detect Linux asset (prefers AppImage, then .deb, then .rpm)
#[cfg(target_os = "linux")]
fn detect_platform_asset(assets: &[GitHubAsset]) -> Option<String> {
    // First try AppImage
    assets.iter()
        .find(|asset| {
            asset.name.ends_with(".AppImage") && matches_app_name(&asset.name)
        })
        .map(|asset| asset.browser_download_url.clone())
        .or_else(|| {
            // Fallback to .deb package
            assets.iter()
                .find(|asset| {
                    asset.name.ends_with(".deb") && matches_app_name(&asset.name)
                })
                .map(|asset| asset.browser_download_url.clone())
        })
        .or_else(|| {
            // Last resort: .rpm package
            assets.iter()
                .find(|asset| asset.name.ends_with(".rpm") && matches_app_name(&asset.name))
                .map(|asset| asset.browser_download_url.clone())
        })
}

/// Check for updates by querying GitHub releases API
pub async fn check_for_updates() -> Result<UpdateInfo, String> {
    let current_version = get_current_version();
    
    // Fetch latest release from GitHub
    let client = reqwest::Client::builder()
        .user_agent("PHP-Version-Manager")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let url = format!("{}/{}/releases/latest", GITHUB_API_BASE, GITHUB_REPO);
    
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "GitHub API returned error: {}",
            response.status()
        ));
    }
    
    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse release info: {}", e))?;
    
    // Remove 'v' prefix if present and normalize version string
    let latest_version = release.tag_name.trim_start_matches('v').to_string();
    let update_available = compare_versions(&current_version, &latest_version) < 0;
    
    // Find platform-specific asset
    let download_url = detect_platform_asset(&release.assets);
    
    Ok(UpdateInfo {
        current_version,
        latest_version,
        update_available,
        download_url,
        release_url: Some(release.html_url),
    })
}

/// Compare two version strings (e.g., "0.1.0" vs "0.2.0")
/// Returns: -1 if v1 < v2, 0 if v1 == v2, 1 if v1 > v2
fn compare_versions(v1: &str, v2: &str) -> i32 {
    let v1_parts: Vec<u32> = v1
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    let v2_parts: Vec<u32> = v2
        .split('.')
        .filter_map(|s| s.parse().ok())
        .collect();
    
    let max_len = v1_parts.len().max(v2_parts.len());
    
    for i in 0..max_len {
        let v1_val = v1_parts.get(i).copied().unwrap_or(0);
        let v2_val = v2_parts.get(i).copied().unwrap_or(0);
        
        if v1_val < v2_val {
            return -1;
        } else if v1_val > v2_val {
            return 1;
        }
    }
    
    0
}

/// Get file extension from URL or determine from content type
fn get_file_extension_from_url(url: &str) -> &str {
    // Extract extension from URL
    if let Some(query_start) = url.find('?') {
        let base_url = &url[..query_start];
        if let Some(dot_pos) = base_url.rfind('.') {
            let ext = &base_url[dot_pos + 1..];
            if !ext.is_empty() && ext.len() <= 10 {
                return ext;
            }
        }
    } else if let Some(dot_pos) = url.rfind('.') {
        let ext = &url[dot_pos + 1..];
        if !ext.is_empty() && ext.len() <= 10 {
            return ext;
        }
    }
    
    // Default based on platform
    #[cfg(windows)]
    {
        ".exe"
    }
    #[cfg(target_os = "linux")]
    {
        ".AppImage"
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        ""
    }
}

/// Download the update file to a temporary location
pub async fn download_update(download_url: &str, mut progress_callback: Option<Box<dyn FnMut(u64, u64, f64) + Send + Sync>>) -> Result<PathBuf, String> {
    let client = reqwest::Client::builder()
        .user_agent("PHP-Version-Manager")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    
    let response = client
        .get(download_url)
        .send()
        .await
        .map_err(|e| format!("Failed to download update: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }
    
    let total_size = response
        .content_length()
        .ok_or_else(|| "Unknown content length".to_string())?;
    
    // Create temp directory if it doesn't exist
    let temp_dir = std::env::temp_dir().join("phpvm-update");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;
    
    // Determine file extension from URL
    let file_ext = get_file_extension_from_url(download_url);
    let temp_file_name = format!("phpvm-update{}", file_ext);
    let temp_file = temp_dir.join(temp_file_name);
    
    // Download with progress tracking
    let mut file = std::fs::File::create(&temp_file)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();
    let start_time = std::time::Instant::now();
    let mut last_callback_time = std::time::Instant::now();
    
    use futures::StreamExt;
    use std::io::Write;
    
    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Download error: {}", e))?;
        file.write_all(&chunk)
            .map_err(|e| format!("Write error: {}", e))?;
        
        downloaded += chunk.len() as u64;
        
        // Calculate speed
        let elapsed = start_time.elapsed().as_secs_f64();
        let speed_mbps = if elapsed > 0.0 {
            (downloaded as f64 / 1_048_576.0) / elapsed
        } else {
            0.0
        };
        
        // Call progress callback every 100ms
        if let Some(ref mut callback) = progress_callback {
            let now = std::time::Instant::now();
            if now.duration_since(last_callback_time).as_millis() >= 100 || downloaded == total_size {
                callback(downloaded, total_size, speed_mbps);
                last_callback_time = now;
            }
        }
    }
    
    Ok(temp_file)
}

/// Apply the update (platform-specific implementation)
#[cfg(windows)]
pub fn apply_update(update_file: PathBuf) -> Result<(), String> {
    // Get current executable path
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    
    eprintln!("[Update] Current exe: {}", current_exe.display());
    eprintln!("[Update] Update file: {}", update_file.display());
    
    // Verify update file exists
    if !update_file.exists() {
        return Err(format!("Update file does not exist: {}", update_file.display()));
    }
    
    let file_ext = update_file.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    // Handle MSI installer
    if file_ext.eq_ignore_ascii_case("msi") {
        eprintln!("[Update] Detected MSI installer, launching installation...");
        // Launch MSI installer
        Command::new("msiexec")
            .args(&["/i", update_file.to_str().unwrap(), "/quiet", "/norestart"])
            .spawn()
            .map_err(|e| format!("Failed to launch MSI installer: {}", e))?;
        return Ok(());
    }
    
    // Handle EXE installer or executable
    // Create a batch script that will replace the exe after the app closes
    let temp_dir = std::env::temp_dir().join("phpvm-update");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;
    
    let batch_script = temp_dir.join("apply_update.bat");
    eprintln!("[Update] Batch script will be created at: {}", batch_script.display());
    
    // Create batch script to replace the executable
    // Escape backslashes in paths for batch script
    let update_path = update_file.to_string_lossy().replace('\\', "\\\\");
    let exe_path = current_exe.to_string_lossy().replace('\\', "\\\\");
    
    // Use proper quoting for paths that may contain spaces
    let script_content = format!(
        r#"@echo off
setlocal enabledelayedexpansion
echo Applying update...
timeout /t 2 /nobreak >nul
copy /Y "{}" "{}"
if %ERRORLEVEL% EQU 0 (
    echo Update applied successfully.
    del "{}"
    del "%~f0"
    timeout /t 1 /nobreak >nul
    start "" "{}"
) else (
    echo Update failed with error code: %ERRORLEVEL%
    echo Please update manually.
    pause
)
"#,
        update_path,
        exe_path,
        update_path,
        exe_path
    );
    
    std::fs::write(&batch_script, script_content)
        .map_err(|e| format!("Failed to create update script: {}", e))?;
    
    eprintln!("[Update] Batch script created successfully");
    eprintln!("[Update] Application will close and update will be applied");
    
    // Execute the batch script (it will run after this process exits)
    Command::new("cmd")
        .args(&["/C", "start", "/MIN", &batch_script.to_string_lossy()])
        .spawn()
        .map_err(|e| format!("Failed to start update script: {}", e))?;
    
    eprintln!("[Update] Update script scheduled to run after app closes");
    
    Ok(())
}

/// Apply the update on Linux
#[cfg(target_os = "linux")]
pub fn apply_update(update_file: PathBuf) -> Result<(), String> {
    eprintln!("[Update] Update file: {}", update_file.display());
    
    // Verify update file exists
    if !update_file.exists() {
        return Err(format!("Update file does not exist: {}", update_file.display()));
    }
    
    let file_ext = update_file.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    
    // Handle AppImage - make executable and suggest user run it
    if file_ext.eq_ignore_ascii_case("AppImage") {
        eprintln!("[Update] Detected AppImage, making executable...");
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&update_file)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&update_file, perms)
            .map_err(|e| format!("Failed to set executable permissions: {}", e))?;
        eprintln!("[Update] AppImage is ready. Please run it manually to complete the update.");
        eprintln!("[Update] Location: {}", update_file.display());
        return Ok(());
    }
    
    // Handle .deb package
    if file_ext.eq_ignore_ascii_case("deb") {
        eprintln!("[Update] Detected DEB package, launching installation...");
        Command::new("pkexec")
            .args(&["dpkg", "-i", update_file.to_str().unwrap()])
            .spawn()
            .map_err(|_e| format!("Failed to launch package installer. Please install manually with: sudo dpkg -i {}", update_file.display()))?;
        return Ok(());
    }
    
    // Handle .rpm package
    if file_ext.eq_ignore_ascii_case("rpm") {
        eprintln!("[Update] Detected RPM package, launching installation...");
        Command::new("pkexec")
            .args(&["rpm", "-i", update_file.to_str().unwrap()])
            .spawn()
            .map_err(|_e| format!("Failed to launch package installer. Please install manually with: sudo rpm -i {}", update_file.display()))?;
        return Ok(());
    }
    
    Err(format!("Unsupported file type: {}", file_ext))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_comparison() {
        assert_eq!(compare_versions("0.1.0", "0.2.0"), -1);
        assert_eq!(compare_versions("0.2.0", "0.1.0"), 1);
        assert_eq!(compare_versions("0.1.0", "0.1.0"), 0);
        assert_eq!(compare_versions("0.1.0.0", "0.1.0"), 0);
        assert_eq!(compare_versions("1.0.0", "0.9.9"), 1);
    }
    
    #[test]
    fn test_matches_app_name() {
        assert!(matches_app_name("PHP Version Manager_0.1.0_x64-setup.exe"));
        assert!(matches_app_name("PHP.Version.Manager_0.1.0_x64_en-US.msi"));
        assert!(matches_app_name("php-version-manager_0.1.0_amd64.AppImage"));
        assert!(matches_app_name("phpvm_0.1.0_amd64.deb"));
        assert!(!matches_app_name("some-other-app.exe"));
    }
}
