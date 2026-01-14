use crate::config;
use anyhow::Context;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
pub fn get_php_executable_path(version_dir: &PathBuf) -> PathBuf {
    version_dir.join("php.exe")
}

#[cfg(not(target_os = "windows"))]
pub fn get_php_executable_path(version_dir: &PathBuf) -> PathBuf {
    version_dir.join("bin").join("php")
}

#[cfg(target_os = "windows")]
pub fn get_current_path() -> PathBuf {
    config::get_base_directory().join("current").join("php.bat")
}

#[cfg(not(target_os = "windows"))]
pub fn get_current_path() -> PathBuf {
    config::get_base_directory().join("current").join("php")
}

pub fn get_path_env_var() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "Path"
    }
    #[cfg(not(target_os = "windows"))]
    {
        "PATH"
    }
}

#[cfg(target_os = "windows")]
pub fn add_to_path(current_dir: &PathBuf) -> anyhow::Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;
    use std::ffi::CString;
    use std::ptr;

    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let environment = current_user
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("Failed to open Environment registry key")?;

    let path_value: String = environment
        .get_value("Path")
        .unwrap_or_else(|_| String::new());

    let current_str = current_dir.to_string_lossy().to_string();
    
    // Check if already in PATH (exact match or as part of a path)
    let path_entries: Vec<&str> = path_value.split(';').collect();
    let already_in_path = path_entries.iter().any(|entry| {
        entry.trim().eq_ignore_ascii_case(&current_str)
    });
    
    if !already_in_path {
        // Remove any existing phpvm entries to avoid duplicates
        let cleaned_path: Vec<&str> = path_entries
            .iter()
            .filter(|entry| {
                let entry_trimmed = entry.trim();
                // Keep entries that don't look like phpvm current directory
                !entry_trimmed.contains("phpvm") || !entry_trimmed.contains("current")
            })
            .copied()
            .collect();
        
        let new_path = if cleaned_path.is_empty() || cleaned_path.iter().all(|s| s.trim().is_empty()) {
            current_str
        } else {
            // Add to the beginning of PATH for priority
            format!("{};{}", current_str, cleaned_path.join(";"))
        };
        
        environment
            .set_value("Path", &new_path)
            .context("Failed to set Path in registry")?;
        
        // Broadcast WM_SETTINGCHANGE to notify Windows of environment change
        unsafe {
            use winapi::um::winuser::{SendMessageTimeoutW, HWND_BROADCAST, WM_SETTINGCHANGE, SMTO_ABORTIFHUNG, SMTO_NORMAL};
            
            let l_param = CString::new("Environment").unwrap();
            let _ = SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                l_param.as_ptr() as _,
                SMTO_ABORTIFHUNG | SMTO_NORMAL,
                1000,
                ptr::null_mut(),
            );
        }
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn add_to_path(current_dir: &PathBuf) -> anyhow::Result<()> {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let rc_file = if shell.contains("zsh") {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("No home directory"))?
            .join(".zshrc")
    } else {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("No home directory"))?
            .join(".bashrc")
    };

    let current_str = format!("export PATH=\"{}$PATH\"", current_dir.to_string_lossy());
    let content = fs::read_to_string(&rc_file).unwrap_or_default();

    if !content.contains(&current_str) {
        let new_content = format!("{}\n{}", content, current_str);
        fs::write(&rc_file, new_content)?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn remove_from_path(current_dir: &PathBuf) -> anyhow::Result<()> {
    use winreg::enums::*;
    use winreg::RegKey;

    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let environment = current_user
        .open_subkey_with_flags("Environment", KEY_READ | KEY_WRITE)
        .context("Failed to open Environment registry key")?;

    let path_value: String = environment
        .get_value("Path")
        .unwrap_or_else(|_| String::new());

    let current_str = current_dir.to_string_lossy().to_string();
    if path_value.contains(&current_str) {
        let new_path = path_value
            .split(';')
            .filter(|p| !p.contains(&current_str))
            .collect::<Vec<_>>()
            .join(";");
        environment
            .set_value("Path", &new_path)
            .context("Failed to update Path in registry")?;
    }

    Ok(())
}

#[cfg(not(target_os = "windows"))]
pub fn remove_from_path(current_dir: &PathBuf) -> anyhow::Result<()> {
    use std::fs;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let rc_file = if shell.contains("zsh") {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("No home directory"))?
            .join(".zshrc")
    } else {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("No home directory"))?
            .join(".bashrc")
    };

    let current_str = format!("export PATH=\"{}$PATH\"", current_dir.to_string_lossy());
    let content = fs::read_to_string(&rc_file).unwrap_or_default();

    if content.contains(&current_str) {
        let new_content = content.replace(&format!("{}\n", current_str), "");
        fs::write(&rc_file, new_content)?;
    }

    Ok(())
}

#[cfg(target_os = "windows")]
pub fn is_path_set(current_dir: &PathBuf) -> anyhow::Result<bool> {
    use winreg::enums::*;
    use winreg::RegKey;

    let current_user = RegKey::predef(HKEY_CURRENT_USER);
    let environment = current_user
        .open_subkey_with_flags("Environment", KEY_READ)
        .context("Failed to open Environment registry key")?;

    let path_value: String = environment
        .get_value("Path")
        .unwrap_or_else(|_| String::new());

    let current_str = current_dir.to_string_lossy().to_string();
    Ok(path_value.contains(&current_str))
}

#[cfg(not(target_os = "windows"))]
pub fn is_path_set(current_dir: &PathBuf) -> anyhow::Result<bool> {
    use std::fs;

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let rc_file = if shell.contains("zsh") {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("No home directory"))?
            .join(".zshrc")
    } else {
        dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("No home directory"))?
            .join(".bashrc")
    };

    if !rc_file.exists() {
        return Ok(false);
    }

    let current_str = format!("export PATH=\"{}$PATH\"", current_dir.to_string_lossy());
    let content = fs::read_to_string(&rc_file).unwrap_or_default();
    Ok(content.contains(&current_str))
}
