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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
