use crate::config;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_FILE: Mutex<Option<PathBuf>> = Mutex::new(None);
static MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10 MB

pub fn init() -> anyhow::Result<()> {
    let log_path = config::get_log_path();
    
    // Create logs directory if it doesn't exist
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Rotate log if it's too large
    if log_path.exists() {
        let metadata = fs::metadata(&log_path)?;
        if metadata.len() > MAX_LOG_SIZE {
            rotate_log(&log_path)?;
        }
    }
    
    *LOG_FILE.lock().unwrap() = Some(log_path);
    
    log_message("INFO", "Logging initialized");
    Ok(())
}

fn rotate_log(log_path: &PathBuf) -> anyhow::Result<()> {
    // Create backup with timestamp
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let backup_path = log_path.parent()
        .unwrap()
        .join(format!("phpvm.{}.log", timestamp));
    
    if log_path.exists() {
        fs::copy(log_path, &backup_path)?;
        fs::remove_file(log_path)?;
    }
    
    Ok(())
}

pub fn log_message(level: &str, message: &str) {
    if let Some(ref log_path) = *LOG_FILE.lock().unwrap() {
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
        {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let datetime = format_timestamp(timestamp);
            let _ = writeln!(file, "[{}] [{}] {}", datetime, level, message);
        }
    }
    
    // Also write to tracing
    match level {
        "ERROR" => tracing::error!("{}", message),
        "WARN" => tracing::warn!("{}", message),
        "INFO" => tracing::info!("{}", message),
        "DEBUG" => tracing::debug!("{}", message),
        _ => tracing::info!("{}", message),
    }
}

fn format_timestamp(secs: u64) -> String {
    // Simple timestamp format: Unix timestamp
    // For a proper implementation, use chrono crate
    format!("{}", secs)
}
