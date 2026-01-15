use crate::app_state::AppState;
use crate::update;
use phpvm_core::config;
use phpvm_core::platform;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, State};
use tokio::sync::mpsc;

#[derive(Serialize, Deserialize)]
pub struct VersionStatus {
    pub version: String,
    pub installed: bool,
    pub active: bool,
    pub online: bool,
    pub install_path: Option<String>,
    pub release_date: Option<String>,
    pub eol_date: Option<String>,
    pub thread_safe: Option<bool>, // true = TS, false = NTS, None = unknown
}

#[derive(Serialize, Deserialize)]
pub struct PathStatus {
    pub is_set: bool,
    pub current_path: String,
}

#[derive(Deserialize)]
pub struct InstallVersionParams {
    pub version: String,
    // CRITICAL: Use struct to ensure proper deserialization in Tauri v2
    // "ts" = thread safe, "nts" = non-thread safe
    // Frontend always sends "ts" or "nts", never null
    pub thread_safe: String,
}

#[tauri::command]
pub async fn install_version(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    params: InstallVersionParams,
) -> Result<(), String> {
    let version = params.version;
    let thread_safe = params.thread_safe;
    // Log the received parameters for debugging
    eprintln!("[Tauri Command] install_version called:");
    eprintln!("  - version: {}", version);
    eprintln!("  - raw thread_safe value: {:?}", thread_safe);
    eprintln!("  - thread_safe type: String (required parameter)");
    eprintln!("  - thread_safe == \"nts\": {}", thread_safe == "nts");
    eprintln!("  - thread_safe == \"ts\": {}", thread_safe == "ts");
    eprintln!("  - thread_safe.len(): {}", thread_safe.len());
    eprintln!("  - thread_safe.as_str(): {:?}", thread_safe.as_str());
    
    // Convert string to Option<bool>
    // "nts" -> Some(false), "ts" -> Some(true), anything else -> Some(true) (defaults to TS)
    let thread_safe_bool: Option<bool> = match thread_safe.as_str() {
        "nts" => {
            eprintln!("  - ✓✓✓ Received 'nts' - NTS WILL BE INSTALLED!");
            Some(false)
        }
        "ts" => {
            eprintln!("  - ✓ Received 'ts' - TS will be installed");
            Some(true)
        }
        other => {
            eprintln!("  - ⚠ WARNING: Unexpected thread_safe value: {:?}, defaulting to TS", other);
            eprintln!("  - ⚠ This should not happen - frontend should only send 'ts' or 'nts'");
            Some(true) // Default to TS for unexpected values
        }
    };
    
    eprintln!("  - converted thread_safe: {:?}", thread_safe_bool);
    eprintln!("  - thread_safe.is_some(): {}", thread_safe_bool.is_some());
    eprintln!("  - thread_safe.is_none(): {}", thread_safe_bool.is_none());
    if let Some(val) = thread_safe_bool {
        eprintln!("  - thread_safe.unwrap(): {} (this should be false for NTS)", val);
        if !val {
            eprintln!("  - ✓ Confirmed: thread_safe is Some(false) - will install NTS");
        } else {
            eprintln!("  - ✓ Confirmed: thread_safe is Some(true) - will install TS");
        }
    } else {
        eprintln!("  - ⚠ WARNING: thread_safe is None - will default to TS");
    }
    
    // Create a channel for progress updates
    let (tx, mut rx) = mpsc::unbounded_channel::<(u64, u64, f64)>();
    let app_for_events = app.clone();
    
    // Spawn a task to listen for progress updates and emit events
    tokio::spawn(async move {
        let mut last_emitted = std::time::Instant::now();
        while let Some((downloaded, total, speed_mbps)) = rx.recv().await {
            let now = std::time::Instant::now();
            
            // Emit progress event every 100ms or when complete
            if now.duration_since(last_emitted).as_millis() >= 100 || downloaded == total {
                let percent = if total > 0 { (downloaded * 100) / total } else { 0 };
                eprintln!("[Download Progress] Emitting: downloaded={}, total={}, speed={:.2} MB/s, percent={}%", 
                    downloaded, total, speed_mbps, percent);
                
                let payload = serde_json::json!({
                    "downloaded": downloaded,
                    "total": total,
                    "speed_mbps": speed_mbps,
                    "percent": percent
                });
                
                if let Err(e) = app_for_events.emit("download-progress", &payload) {
                    eprintln!("[Download Progress] Failed to emit event: {}", e);
                } else {
                    eprintln!("[Download Progress] Event emitted successfully");
                }
                
                last_emitted = now;
            }
        }
    });
    
    // Create progress callback that sends to channel
    let last_sent = std::sync::Arc::new(std::sync::Mutex::new((0u64, std::time::Instant::now())));
    let tx_clone = tx.clone();
    let progress_callback: Box<dyn FnMut(u64, u64, f64) + Send + Sync> = Box::new(move |downloaded: u64, total: u64, speed_mbps: f64| {
        eprintln!("[Download Progress Callback] Called with: downloaded={}, total={}, speed={:.2} MB/s", downloaded, total, speed_mbps);
        let mut last = last_sent.lock().unwrap();
        let now = std::time::Instant::now();
        
        // Send progress update every 100ms or when complete (or immediately if total > 0 and we haven't sent anything yet)
        if now.duration_since(last.1).as_millis() >= 100 || downloaded == total || (total > 0 && last.0 == 0) {
            eprintln!("[Download Progress Callback] Sending to channel: downloaded={}, total={}, speed={:.2} MB/s", downloaded, total, speed_mbps);
            if let Err(e) = tx_clone.send((downloaded, total, speed_mbps)) {
                eprintln!("[Download Progress Callback] Failed to send to channel: {}", e);
            } else {
                eprintln!("[Download Progress Callback] Successfully sent to channel");
            }
            *last = (downloaded, now);
        }
    });
    
    let manager = state.manager.lock().await;
    let result = manager.install(&version, thread_safe_bool, Some(progress_callback)).await.map_err(|e| e.to_string());
    
    // Drop the sender to close the channel when done
    drop(tx);
    
    result
}

#[tauri::command]
pub async fn remove_version(
    state: State<'_, AppState>,
    version: String,
) -> Result<(), String> {
    let manager = state.manager.lock().await;
    manager.remove(&version).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn switch_version(
    state: State<'_, AppState>,
    version: String,
) -> Result<(), String> {
    let manager = state.manager.lock().await;
    manager.switch(&version).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_installed(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let manager = state.manager.lock().await;
    manager.list_installed().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_available(state: State<'_, AppState>) -> Result<Vec<String>, String> {
    let manager = state.manager.lock().await;
    manager.list_available().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let manager = state.manager.lock().await;
    manager.get_active().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_install_path() -> Result<String, String> {
    let config = config::Config::load().map_err(|e| e.to_string())?;
    Ok(config.install_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_log_path() -> Result<String, String> {
    let log_path = config::get_log_path();
    Ok(log_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_current_dir() -> Result<String, String> {
    let current_path = platform::get_current_path();
    let current_dir = current_path
        .parent()
        .ok_or_else(|| "Invalid current path".to_string())?;
    Ok(current_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn check_path_status() -> Result<PathStatus, String> {
    let current_path = platform::get_current_path();
    let current_dir = current_path
        .parent()
        .ok_or_else(|| "Invalid current path".to_string())?;

    let current_dir_buf = current_dir.to_path_buf();
    let is_set = platform::is_path_set(&current_dir_buf).unwrap_or(false);

    Ok(PathStatus {
        is_set,
        current_path: current_dir.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn set_path() -> Result<(), String> {
    let current_path = platform::get_current_path();
    let current_dir = current_path
        .parent()
        .ok_or_else(|| "Invalid current path".to_string())?;

    let current_dir_buf = current_dir.to_path_buf();
    platform::add_to_path(&current_dir_buf).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_version_status(
    state: State<'_, AppState>,
    version: String,
) -> Result<VersionStatus, String> {
    let manager = state.manager.lock().await;
    let installed = manager.list_installed().map_err(|e| e.to_string())?;
    let active = manager.get_active().map_err(|e| e.to_string())?;

    // Check if TS or NTS variant is installed
    let ts_installed = installed.contains(&format!("{}-ts", version));
    let nts_installed = installed.contains(&format!("{}-nts", version));
    let is_installed = ts_installed || nts_installed;
    
    // Check if any variant is active (active version is base version without suffix)
    let is_active = active.as_ref().map(|a| a == &version).unwrap_or(false);

    // Get install path if installed (prefer TS if both exist)
    let install_path = if is_installed {
        let config = config::Config::load().map_err(|e| e.to_string())?;
        let variant = if ts_installed { "ts" } else { "nts" };
        Some(
            config
                .install_dir
                .join(format!("php-{}-{}", version, variant))
                .to_string_lossy()
                .to_string(),
        )
    } else {
        None
    };

    // Get release date, EOL date, and online status from provider
    // This fetches from network, but only once per version (not multiple times)
    let (release_date, eol_date, is_online) = match manager.get_version_info(&version).await {
        Ok(Some(info)) => (
            info.release_date,
            info.eol_date,
            info.download_url.is_some(),
        ),
        Ok(None) => (None, None, false),
        Err(_) => (None, None, false),
    };

    // Get thread-safe status - check which variants are installed
    let thread_safe = if is_installed {
        if ts_installed && nts_installed {
            // Both installed - return None to indicate both are available
            None
        } else if ts_installed {
            Some(true) // Only TS installed
        } else {
            Some(false) // Only NTS installed
        }
    } else {
        None
    };

    Ok(VersionStatus {
        version,
        installed: is_installed,
        active: is_active,
        online: is_online, // Use info from provider
        install_path,
        release_date,
        eol_date,
        thread_safe,
    })
}

#[derive(Serialize, Deserialize)]
pub struct CachedFile {
    pub hash: String,
    pub size: u64,
    pub modified: String,
    pub version: Option<String>, // e.g., "8.5.1-ts" or "8.5.1-nts"
}

// Helper function to hash a URL (same as in download.rs)
fn hash_url(url: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

// Helper function to determine VS/VC version based on PHP version
// PHP 8.4+ uses vs17 (Visual Studio 2017/2019)
// PHP 8.0-8.3 uses vs16 (Visual Studio 2016)
// PHP 7.4 uses vc15 (Visual C++ 2017) or vs16
// PHP 7.2-7.3 uses VC15 (Visual C++ 2017) - archived, capital VC
// PHP 7.0-7.1 uses VC14 (Visual C++ 2015) - archived, capital VC
// PHP 5.6 uses VC11 (Visual C++ 2012) - archived, capital VC
fn get_vs_version(major: u8, minor: u8) -> &'static str {
    if major > 8 || (major == 8 && minor >= 4) {
        "vs17"
    } else if major == 8 {
        "vs16"
    } else if major == 7 {
        if minor >= 4 {
            "vc15"
        } else if minor >= 2 {
            "VC15"
        } else {
            "VC14"
        }
    } else {
        "VC11" // PHP 5.6 and earlier
    }
}

// Helper function to get base URL for PHP version
fn get_base_url(major: u8, minor: u8) -> &'static str {
    if major < 7 || (major == 7 && minor < 4) {
        "https://windows.php.net/downloads/releases/archives/"
    } else {
        "https://windows.php.net/downloads/releases/"
    }
}

#[tauri::command]
pub async fn list_cached_files(state: State<'_, AppState>) -> Result<Vec<CachedFile>, String> {
    let cache_dir = config::get_base_directory().join("cache");
    
    if !cache_dir.exists() {
        return Ok(vec![]);
    }
    
    // Get all available versions to match against
    let manager = state.manager.lock().await;
    let available_versions = manager.list_available().await.map_err(|e| e.to_string())?;
    drop(manager);
    
    // Build a hash map of hash -> version once (O(m * p) instead of O(n * m * p))
    // This dramatically improves performance when there are many cached files
    use std::collections::HashMap;
    let mut hash_to_version: HashMap<String, String> = HashMap::new();
    
    for version_str in &available_versions {
        // Parse version
        let parts: Vec<&str> = version_str.split('.').collect();
        if parts.len() >= 3 {
            if let (Ok(major), Ok(minor), Ok(_patch)) = (
                parts[0].parse::<u8>(),
                parts[1].parse::<u8>(),
                parts[2].parse::<u8>(),
            ) {
                let vs_version = get_vs_version(major, minor);
                let base_url = get_base_url(major, minor);
                
                // Try TS URL
                let ts_url = format!(
                    "{}php-{}-Win32-{}-x64.zip",
                    base_url, version_str, vs_version
                );
                hash_to_version.insert(hash_url(&ts_url), format!("{}-ts", version_str));
                
                // Try NTS URL
                let nts_url = format!(
                    "{}php-{}-nts-Win32-{}-x64.zip",
                    base_url, version_str, vs_version
                );
                hash_to_version.insert(hash_url(&nts_url), format!("{}-nts", version_str));
                
                // Try fallback versions
                // For vs17, try vs16 as fallback
                if vs_version == "vs17" {
                    let fallback_base = "https://windows.php.net/downloads/releases/";
                    let ts_url_vs16 = format!(
                        "{}php-{}-Win32-vs16-x64.zip",
                        fallback_base, version_str
                    );
                    hash_to_version.insert(hash_url(&ts_url_vs16), format!("{}-ts", version_str));
                    
                    let nts_url_vs16 = format!(
                        "{}php-{}-nts-Win32-vs16-x64.zip",
                        fallback_base, version_str
                    );
                    hash_to_version.insert(hash_url(&nts_url_vs16), format!("{}-nts", version_str));
                }
                // For vs16, try vc15 as fallback (for PHP 7.4)
                if vs_version == "vs16" {
                    let fallback_base = "https://windows.php.net/downloads/releases/";
                    let ts_url_vc15 = format!(
                        "{}php-{}-Win32-vc15-x64.zip",
                        fallback_base, version_str
                    );
                    hash_to_version.insert(hash_url(&ts_url_vc15), format!("{}-ts", version_str));
                    
                    let nts_url_vc15 = format!(
                        "{}php-{}-nts-Win32-vc15-x64.zip",
                        fallback_base, version_str
                    );
                    hash_to_version.insert(hash_url(&nts_url_vc15), format!("{}-nts", version_str));
                }
                // For VC15 (PHP 7.2-7.3), try VC14 as fallback
                if vs_version == "VC15" {
                    let fallback_base = "https://windows.php.net/downloads/releases/archives/";
                    let ts_url_vc14 = format!(
                        "{}php-{}-Win32-VC14-x64.zip",
                        fallback_base, version_str
                    );
                    hash_to_version.insert(hash_url(&ts_url_vc14), format!("{}-ts", version_str));
                    
                    let nts_url_vc14 = format!(
                        "{}php-{}-nts-Win32-VC14-x64.zip",
                        fallback_base, version_str
                    );
                    hash_to_version.insert(hash_url(&nts_url_vc14), format!("{}-nts", version_str));
                }
            }
        }
    }
    
    let mut cached_files = Vec::new();
    
    match std::fs::read_dir(&cache_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(hash) = path.file_name().and_then(|n| n.to_str()) {
                            let metadata = match std::fs::metadata(&path) {
                                Ok(m) => m,
                                Err(_) => continue,
                            };
                            
                            let size = metadata.len();
                            let modified = match metadata.modified() {
                                Ok(time) => {
                                    match time.duration_since(std::time::UNIX_EPOCH) {
                                        Ok(duration) => {
                                            format!("{}", duration.as_secs())
                                        }
                                        Err(_) => "0".to_string(),
                                    }
                                }
                                Err(_) => "0".to_string(),
                            };
                            
                            // O(1) lookup instead of O(m * p) iteration
                            let matched_version = hash_to_version.get(hash).cloned();
                            
                            cached_files.push(CachedFile {
                                hash: hash.to_string(),
                                size,
                                modified,
                                version: matched_version,
                            });
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to read cache directory: {}", e));
        }
    }
    
    // Sort by modified date (newest first)
    cached_files.sort_by(|a, b| b.modified.cmp(&a.modified));
    
    Ok(cached_files)
}

#[tauri::command]
pub async fn remove_cached_file(hash: String) -> Result<(), String> {
    let cache_dir = config::get_base_directory().join("cache");
    let file_path = cache_dir.join(&hash);
    
    if !file_path.exists() {
        return Err(format!("Cached file not found: {}", hash));
    }
    
    std::fs::remove_file(&file_path)
        .map_err(|e| format!("Failed to remove cached file: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn clear_all_cache() -> Result<(), String> {
    let cache_dir = config::get_base_directory().join("cache");
    
    if !cache_dir.exists() {
        return Ok(()); // Already empty
    }
    
    match std::fs::read_dir(&cache_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() {
                        if let Err(e) = std::fs::remove_file(&path) {
                            eprintln!("Failed to remove cached file {:?}: {}", path, e);
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Err(format!("Failed to read cache directory: {}", e));
        }
    }
    
    Ok(())
}

// ==================== Update Commands ====================

#[tauri::command]
pub async fn get_app_version() -> Result<String, String> {
    Ok(update::get_current_version())
}

#[tauri::command]
pub async fn check_for_updates() -> Result<update::UpdateInfo, String> {
    update::check_for_updates().await
}

#[tauri::command]
pub async fn download_update(
    app: tauri::AppHandle,
    download_url: String,
) -> Result<String, String> {
    // Create a channel for progress updates
    let (tx, mut rx) = mpsc::unbounded_channel::<(u64, u64, f64)>();
    let app_for_events = app.clone();
    
    // Spawn a task to listen for progress updates and emit events
    tokio::spawn(async move {
        let mut last_emitted = std::time::Instant::now();
        while let Some((downloaded, total, speed_mbps)) = rx.recv().await {
            let now = std::time::Instant::now();
            
            // Emit progress event every 100ms or when complete
            if now.duration_since(last_emitted).as_millis() >= 100 || downloaded == total {
                let percent = if total > 0 { (downloaded * 100) / total } else { 0 };
                
                let payload = serde_json::json!({
                    "downloaded": downloaded,
                    "total": total,
                    "speed_mbps": speed_mbps,
                    "percent": percent
                });
                
                if let Err(e) = app_for_events.emit("update-download-progress", &payload) {
                    eprintln!("[Update Download] Failed to emit event: {}", e);
                }
                
                last_emitted = now;
            }
        }
    });
    
    // Create progress callback that sends to channel
    let last_sent = std::sync::Arc::new(std::sync::Mutex::new((0u64, std::time::Instant::now())));
    let tx_clone = tx.clone();
    let progress_callback: Box<dyn FnMut(u64, u64, f64) + Send + Sync> = Box::new(move |downloaded: u64, total: u64, speed_mbps: f64| {
        let mut last = last_sent.lock().unwrap();
        let now = std::time::Instant::now();
        
        // Send progress update every 100ms or when complete
        if now.duration_since(last.1).as_millis() >= 100 || downloaded == total || (total > 0 && last.0 == 0) {
            if let Err(e) = tx_clone.send((downloaded, total, speed_mbps)) {
                eprintln!("[Update Download] Failed to send to channel: {}", e);
            }
            *last = (downloaded, now);
        }
    });
    
    let update_file = update::download_update(&download_url, Some(progress_callback)).await?;
    
    // Send final progress update (100% complete) before closing channel
    let file_size = std::fs::metadata(&update_file)
        .map(|m| m.len())
        .unwrap_or(0);
    
    // Send final update
    if let Err(e) = tx.send((file_size, file_size, 0.0)) {
        eprintln!("[Update Download] Failed to send final progress: {}", e);
    }
    
    // Give a small delay to ensure the final event is processed
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Drop the sender to close the channel when done
    drop(tx);
    
    eprintln!("[Update Download] Download complete: {}", update_file.display());
    Ok(update_file.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn apply_update(update_file_path: String) -> Result<(), String> {
    let update_file = std::path::PathBuf::from(update_file_path);
    if !update_file.exists() {
        return Err("Update file not found".to_string());
    }
    
    update::apply_update(update_file)
}

#[tauri::command]
pub async fn open_url(url: String) -> Result<(), String> {
    use std::process::Command;
    
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }
    
    Ok(())
}
