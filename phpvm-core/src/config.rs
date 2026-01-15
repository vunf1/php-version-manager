use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub install_dir: PathBuf,
    pub active_version: Option<String>,
    pub download_cache: PathBuf,
    pub providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub url: String,
    pub verify_checksum: bool,
}

impl Default for Config {
    fn default() -> Self {
        let base_dir = get_base_directory();
        Config {
            install_dir: base_dir.join("versions"),
            active_version: None,
            download_cache: base_dir.join("cache"),
            providers: vec![ProviderConfig {
                name: "official".to_string(),
                url: "https://windows.php.net/downloads/releases/".to_string(),
                verify_checksum: true,
            }],
        }
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = get_config_path();
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = get_config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}

pub fn get_base_directory() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("phpvm")
}

pub fn get_config_path() -> PathBuf {
    get_base_directory().join("config.json")
}

pub fn get_state_path() -> PathBuf {
    get_base_directory().join("state.json")
}

pub fn get_log_path() -> PathBuf {
    get_base_directory().join("logs").join("phpvm.log")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.install_dir.ends_with("versions"));
        assert!(config.download_cache.ends_with("cache"));
        assert_eq!(config.active_version, None);
        assert_eq!(config.providers.len(), 1);
        assert_eq!(config.providers[0].name, "official");
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.json");
        
        // Override the base directory for testing
        let _original_base = get_base_directory();
        
        // Create a test config
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
    fn test_get_base_directory() {
        let base = get_base_directory();
        assert!(base.to_string_lossy().contains("phpvm"));
    }

    #[test]
    fn test_get_config_path() {
        let config_path = get_config_path();
        assert!(config_path.to_string_lossy().ends_with("config.json"));
        assert!(config_path.to_string_lossy().contains("phpvm"));
    }

    #[test]
    fn test_get_state_path() {
        let state_path = get_state_path();
        assert!(state_path.to_string_lossy().ends_with("state.json"));
        assert!(state_path.to_string_lossy().contains("phpvm"));
    }

    #[test]
    fn test_get_log_path() {
        let log_path = get_log_path();
        assert!(log_path.to_string_lossy().ends_with("phpvm.log"));
        assert!(log_path.to_string_lossy().contains("phpvm"));
        assert!(log_path.to_string_lossy().contains("logs"));
    }

    #[test]
    fn test_provider_config() {
        let provider = ProviderConfig {
            name: "test".to_string(),
            url: "https://example.com".to_string(),
            verify_checksum: true,
        };
        
        assert_eq!(provider.name, "test");
        assert_eq!(provider.url, "https://example.com");
        assert!(provider.verify_checksum);
    }
}
