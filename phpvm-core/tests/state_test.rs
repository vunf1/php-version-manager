/**
 * Integration tests for PhpState
 * Tests the public API of the state module
 */
use phpvm_core::{PhpState, InstallMetadata};
use std::path::PathBuf;

#[test]
fn test_state_default_public_api() {
    let state = PhpState::default();
    assert!(state.installed_versions.is_empty());
    assert_eq!(state.active_version, None);
    assert_eq!(state.last_known_good, None);
    assert!(state.install_metadata.is_empty());
}

#[test]
fn test_state_add_version_public_api() {
    let mut state = PhpState::default();
    let metadata = InstallMetadata {
        version: "8.2.0".to_string(),
        install_path: PathBuf::from("/test/php-8.2.0"),
        installed_at: "2024-01-01".to_string(),
        checksum: Some("abc123".to_string()),
        source: "official".to_string(),
    };

    state.add_version("8.2.0".to_string(), metadata.clone());
    
    assert_eq!(state.installed_versions.len(), 1);
    assert!(state.installed_versions.contains(&"8.2.0".to_string()));
    assert_eq!(state.install_metadata.get("8.2.0"), Some(&metadata));
}

#[test]
fn test_state_remove_version_public_api() {
    let mut state = PhpState::default();
    let metadata = InstallMetadata {
        version: "8.2.0".to_string(),
        install_path: PathBuf::from("/test/php-8.2.0"),
        installed_at: "2024-01-01".to_string(),
        checksum: None,
        source: "official".to_string(),
    };

    state.add_version("8.2.0".to_string(), metadata);
    state.set_active("8.2.0".to_string());
    state.remove_version("8.2.0");
    
    assert!(!state.installed_versions.contains(&"8.2.0".to_string()));
    assert!(state.install_metadata.get("8.2.0").is_none());
}

#[test]
fn test_state_set_active_public_api() {
    let mut state = PhpState::default();
    
    state.set_active("8.2.0".to_string());
    assert_eq!(state.active_version, Some("8.2.0".to_string()));
    assert_eq!(state.last_known_good, None);
    
    state.set_active("8.3.0".to_string());
    assert_eq!(state.active_version, Some("8.3.0".to_string()));
    assert_eq!(state.last_known_good, Some("8.2.0".to_string()));
}

#[test]
fn test_state_get_metadata_public_api() {
    let mut state = PhpState::default();
    let metadata = InstallMetadata {
        version: "8.2.0".to_string(),
        install_path: PathBuf::from("/test/php-8.2.0"),
        installed_at: "2024-01-01".to_string(),
        checksum: Some("abc123".to_string()),
        source: "official".to_string(),
    };

    state.add_version("8.2.0".to_string(), metadata.clone());
    
    assert_eq!(state.get_metadata("8.2.0"), Some(&metadata));
    assert_eq!(state.get_metadata("8.3.0"), None);
}
