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
