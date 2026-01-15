/**
 * Integration tests for Config
 * Tests the public API of the config module
 */
use phpvm_core::config::{Config, ProviderConfig};
use std::path::PathBuf;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_config_default_public_api() {
    let config = Config::default();
    assert!(config.install_dir.ends_with("versions"));
    assert!(config.download_cache.ends_with("cache"));
    assert_eq!(config.active_version, None);
    assert_eq!(config.providers.len(), 1);
}

#[test]
fn test_config_save_and_load_public_api() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    
    let mut config = Config::default();
    config.active_version = Some("8.2.0".to_string());
    config.install_dir = temp_dir.path().join("versions");
    config.download_cache = temp_dir.path().join("cache");
    
    // Save config
    let content = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&config_path, content).unwrap();
    
    // Load config
    let content = fs::read_to_string(&config_path).unwrap();
    let loaded: Config = serde_json::from_str(&content).unwrap();
    
    assert_eq!(loaded.active_version, Some("8.2.0".to_string()));
    assert_eq!(loaded.providers.len(), 1);
}

#[test]
fn test_provider_config_public_api() {
    let provider = ProviderConfig {
        name: "test".to_string(),
        url: "https://example.com".to_string(),
        verify_checksum: true,
    };
    
    assert_eq!(provider.name, "test");
    assert_eq!(provider.url, "https://example.com");
    assert!(provider.verify_checksum);
}
