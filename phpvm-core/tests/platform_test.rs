/**
 * Integration tests for Platform
 * Tests the public API of the platform module
 */
use phpvm_core::{get_php_executable_path, get_current_path, get_path_env_var};
use std::path::PathBuf;

#[test]
fn test_get_php_executable_path_public_api() {
    let version_dir = PathBuf::from("test/php-8.2.0");
    let exe_path = get_php_executable_path(&version_dir);
    
    #[cfg(target_os = "windows")]
    assert!(exe_path.to_string_lossy().ends_with("php.exe"));
    
    #[cfg(not(target_os = "windows"))]
    assert!(exe_path.to_string_lossy().ends_with("bin/php"));
}

#[test]
fn test_get_path_env_var_public_api() {
    let env_var = get_path_env_var();
    #[cfg(target_os = "windows")]
    assert_eq!(env_var, "Path");
    #[cfg(not(target_os = "windows"))]
    assert_eq!(env_var, "PATH");
}

#[test]
fn test_get_current_path_public_api() {
    let current_path = get_current_path();
    assert!(current_path.to_string_lossy().contains("current"));
    #[cfg(target_os = "windows")]
    assert!(current_path.to_string_lossy().contains("php.bat"));
    #[cfg(not(target_os = "windows"))]
    assert!(current_path.to_string_lossy().contains("php"));
}
