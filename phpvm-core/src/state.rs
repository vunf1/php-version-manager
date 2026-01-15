use crate::config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhpState {
    pub installed_versions: Vec<String>,
    pub active_version: Option<String>,
    pub last_known_good: Option<String>,
    pub install_metadata: HashMap<String, InstallMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallMetadata {
    pub version: String,
    pub install_path: PathBuf,
    pub installed_at: String,
    pub checksum: Option<String>,
    pub source: String,
}

impl PhpState {
    pub fn load() -> anyhow::Result<Self> {
        let state_path = config::get_state_path();
        if state_path.exists() {
            let content = std::fs::read_to_string(&state_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(PhpState::default())
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let state_path = config::get_state_path();
        if let Some(parent) = state_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&state_path, content)?;
        Ok(())
    }

    pub fn add_version(&mut self, version: String, metadata: InstallMetadata) {
        if !self.installed_versions.contains(&version) {
            self.installed_versions.push(version.clone());
        }
        self.install_metadata.insert(version, metadata);
    }

    pub fn remove_version(&mut self, version: &str) {
        self.installed_versions.retain(|v| v != version);
        self.install_metadata.remove(version);
        if self.active_version.as_deref() == Some(version) {
            self.active_version = self.last_known_good.clone();
        }
    }

    pub fn set_active(&mut self, version: String) {
        if self.active_version.is_some() {
            self.last_known_good = self.active_version.clone();
        }
        self.active_version = Some(version);
    }
    
    pub fn get_metadata(&self, version: &str) -> Option<&InstallMetadata> {
        self.install_metadata.get(version)
    }
}

impl Default for PhpState {
    fn default() -> Self {
        PhpState {
            installed_versions: Vec::new(),
            active_version: None,
            last_known_good: None,
            install_metadata: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_state_default() {
        let state = PhpState::default();
        assert!(state.installed_versions.is_empty());
        assert_eq!(state.active_version, None);
        assert_eq!(state.last_known_good, None);
        assert!(state.install_metadata.is_empty());
    }

    #[test]
    fn test_state_add_version() {
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
    fn test_state_add_version_duplicate() {
        let mut state = PhpState::default();
        let metadata1 = InstallMetadata {
            version: "8.2.0".to_string(),
            install_path: PathBuf::from("/test/php-8.2.0"),
            installed_at: "2024-01-01".to_string(),
            checksum: None,
            source: "official".to_string(),
        };
        let metadata2 = InstallMetadata {
            version: "8.2.0".to_string(),
            install_path: PathBuf::from("/test/php-8.2.0-updated"),
            installed_at: "2024-01-02".to_string(),
            checksum: None,
            source: "official".to_string(),
        };

        state.add_version("8.2.0".to_string(), metadata1);
        state.add_version("8.2.0".to_string(), metadata2.clone());
        
        // Should only have one entry in installed_versions
        assert_eq!(state.installed_versions.len(), 1);
        // But metadata should be updated
        assert_eq!(state.install_metadata.get("8.2.0"), Some(&metadata2));
    }

    #[test]
    fn test_state_remove_version() {
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
        // Active version should be reset to last_known_good
        assert_eq!(state.active_version, None);
    }

    #[test]
    fn test_state_set_active() {
        let mut state = PhpState::default();
        
        state.set_active("8.2.0".to_string());
        assert_eq!(state.active_version, Some("8.2.0".to_string()));
        assert_eq!(state.last_known_good, None);
        
        state.set_active("8.3.0".to_string());
        assert_eq!(state.active_version, Some("8.3.0".to_string()));
        assert_eq!(state.last_known_good, Some("8.2.0".to_string()));
    }

    #[test]
    fn test_state_get_metadata() {
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

    #[test]
    fn test_state_remove_active_version() {
        let mut state = PhpState::default();
        let metadata1 = InstallMetadata {
            version: "8.2.0".to_string(),
            install_path: PathBuf::from("/test/php-8.2.0"),
            installed_at: "2024-01-01".to_string(),
            checksum: None,
            source: "official".to_string(),
        };
        let metadata2 = InstallMetadata {
            version: "8.1.0".to_string(),
            install_path: PathBuf::from("/test/php-8.1.0"),
            installed_at: "2024-01-01".to_string(),
            checksum: None,
            source: "official".to_string(),
        };

        state.add_version("8.1.0".to_string(), metadata2);
        state.set_active("8.1.0".to_string());
        state.add_version("8.2.0".to_string(), metadata1);
        state.set_active("8.2.0".to_string());
        
        // Now remove 8.2.0, should fall back to 8.1.0
        state.remove_version("8.2.0");
        assert_eq!(state.active_version, Some("8.1.0".to_string()));
    }
}
